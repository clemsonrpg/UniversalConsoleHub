use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;
use crate::domain::theme::{
    is_builtin_theme_id, validate_theme_id, validate_theme_manifest, InstalledThemePackage,
    ThemeManifest,
};
use crate::infrastructure::system::cover_cache::{CoverCache, MAX_COVER_BYTES};
use crate::infrastructure::system::security::SecurityManager;

const MAX_MANIFEST_BYTES: u64 = 512 * 1024;

#[derive(Clone)]
pub struct ThemePackageService {
    themes_dir: PathBuf,
}

impl ThemePackageService {
    pub fn new(themes_dir: PathBuf) -> Self {
        Self { themes_dir }
    }

    pub fn themes_dir(&self) -> &Path {
        &self.themes_dir
    }

    pub fn list_installed_themes(&self) -> Result<Vec<InstalledThemePackage>, DomainError> {
        if !self.themes_dir.is_dir() {
            return Ok(Vec::new());
        }
        let entries = fs::read_dir(&self.themes_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to read installed themes directory '{}': {e}",
                self.themes_dir.display()
            ))
        })?;

        let mut installed = Vec::new();
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_symlink() || !ft.is_dir() {
                continue;
            }
            let dir = entry.path();
            let manifest_file = dir.join("theme.json");
            if !manifest_file.is_file() {
                continue;
            }
            if let Ok(pkg) = self.load_installed_dir(&dir) {
                installed.push(pkg);
            }
        }
        installed.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        Ok(installed)
    }

    pub fn import_theme_package(
        &self,
        source_path: &Path,
    ) -> Result<InstalledThemePackage, DomainError> {
        if !source_path.exists() {
            return Err(DomainError::NotFound(format!(
                "pacote de tema não encontrado em '{}'",
                source_path.display()
            )));
        }

        let (source_root, manifest_path) = if source_path.is_dir() {
            let m = source_path.join("theme.json");
            if !m.is_file() {
                return Err(DomainError::Validation(
                    "a pasta selecionada não contém um manifesto 'theme.json' válido".to_string(),
                ));
            }
            (source_path.to_path_buf(), m)
        } else {
            let valid_file = SecurityManager::validate_existing_file(source_path)?;
            let parent = valid_file
                .parent()
                .ok_or_else(|| {
                    DomainError::Validation(
                        "não foi possível determinar o diretório do tema".to_string(),
                    )
                })?
                .to_path_buf();
            (parent, valid_file)
        };

        let meta = fs::metadata(&manifest_path).map_err(|e| {
            DomainError::Io(format!(
                "failed to inspect theme manifest '{}': {e}",
                manifest_path.display()
            ))
        })?;
        if meta.len() == 0 || meta.len() > MAX_MANIFEST_BYTES {
            return Err(DomainError::Validation(
                "manifesto de tema vazio ou maior que o limite seguro de 512 KB".to_string(),
            ));
        }

        let raw_json = fs::read_to_string(&manifest_path).map_err(|e| {
            DomainError::Io(format!(
                "failed to read theme manifest '{}': {e}",
                manifest_path.display()
            ))
        })?;
        let mut manifest: ThemeManifest = serde_json::from_str(&raw_json).map_err(|e| {
            DomainError::Validation(format!("manifesto de tema JSON inválido: {e}"))
        })?;

        validate_theme_manifest(&manifest)?;
        if is_builtin_theme_id(&manifest.id) {
            return Err(DomainError::Conflict(format!(
                "o ID '{}' é reservado para um tema nativo do Universal Console Hub",
                manifest.id
            )));
        }

        fs::create_dir_all(&self.themes_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to create themes directory '{}': {e}",
                self.themes_dir.display()
            ))
        })?;

        let target_dir =
            SecurityManager::ensure_within_base(&self.themes_dir, Path::new(&manifest.id))?;
        let assets_dir = target_dir.join("assets");
        fs::create_dir_all(&assets_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to create theme assets directory '{}': {e}",
                assets_dir.display()
            ))
        })?;

        // Validate and copy static image assets strictly by magic bytes (zero code execution!)
        let preview_image_path = if let Some(rel) = manifest.preview_image.clone() {
            if rel.trim().is_empty() {
                manifest.preview_image = None;
                None
            } else {
                let copied =
                    self.copy_validated_image(&source_root, &rel, &assets_dir, "preview")?;
                manifest.preview_image = Some(
                    copied
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                );
                Some(copied.to_string_lossy().to_string())
            }
        } else {
            None
        };

        let background_image_path = if let Some(rel) = manifest.background_image.clone() {
            if rel.trim().is_empty() {
                manifest.background_image = None;
                None
            } else {
                let copied =
                    self.copy_validated_image(&source_root, &rel, &assets_dir, "background")?;
                manifest.background_image = Some(
                    copied
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                );
                Some(copied.to_string_lossy().to_string())
            }
        } else {
            None
        };

        let mut asset_paths = BTreeMap::new();
        let mut normalized_assets = BTreeMap::new();
        for (key, rel) in &manifest.assets {
            let safe_prefix: String = key
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            let copied = self.copy_validated_image(&source_root, rel, &assets_dir, &safe_prefix)?;
            let file_name = copied
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            normalized_assets.insert(key.clone(), file_name);
            asset_paths.insert(key.clone(), copied.to_string_lossy().to_string());
        }
        manifest.assets = normalized_assets;

        let serialized = serde_json::to_string_pretty(&manifest).map_err(|e| {
            DomainError::Io(format!(
                "failed to serialize normalized theme manifest: {e}"
            ))
        })?;
        fs::write(target_dir.join("theme.json"), serialized).map_err(|e| {
            DomainError::Io(format!("failed to save installed theme manifest: {e}"))
        })?;

        Ok(InstalledThemePackage {
            manifest,
            installed_dir: target_dir.to_string_lossy().to_string(),
            preview_image_path,
            background_image_path,
            asset_paths,
            installed_at: now_utc_rfc3339(),
        })
    }

    pub fn remove_installed_theme(&self, theme_id: &str) -> Result<bool, DomainError> {
        validate_theme_id(theme_id)?;
        if is_builtin_theme_id(theme_id) {
            return Err(DomainError::Validation(format!(
                "o tema padrão '{}' não pode ser desinstalado",
                theme_id
            )));
        }
        if !self.themes_dir.is_dir() {
            return Ok(false);
        }
        let target_dir =
            SecurityManager::ensure_within_base(&self.themes_dir, Path::new(theme_id))?;
        if !target_dir.is_dir() {
            return Ok(false);
        }
        fs::remove_dir_all(&target_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to remove installed theme '{}': {e}",
                target_dir.display()
            ))
        })?;
        Ok(true)
    }

    fn load_installed_dir(&self, dir: &Path) -> Result<InstalledThemePackage, DomainError> {
        let manifest_path = dir.join("theme.json");
        let raw = fs::read_to_string(&manifest_path)
            .map_err(|e| DomainError::Io(format!("failed to read theme.json: {e}")))?;
        let manifest: ThemeManifest = serde_json::from_str(&raw)
            .map_err(|e| DomainError::Validation(format!("invalid installed theme.json: {e}")))?;
        validate_theme_manifest(&manifest)?;

        let assets_dir = dir.join("assets");
        let preview_image_path = manifest
            .preview_image
            .as_deref()
            .and_then(|f| self.resolve_existing_asset(&assets_dir, f));
        let background_image_path = manifest
            .background_image
            .as_deref()
            .and_then(|f| self.resolve_existing_asset(&assets_dir, f));

        let mut asset_paths = BTreeMap::new();
        for (k, f) in &manifest.assets {
            if let Some(resolved) = self.resolve_existing_asset(&assets_dir, f) {
                asset_paths.insert(k.clone(), resolved);
            }
        }

        Ok(InstalledThemePackage {
            manifest,
            installed_dir: dir.to_string_lossy().to_string(),
            preview_image_path,
            background_image_path,
            asset_paths,
            installed_at: now_utc_rfc3339(),
        })
    }

    fn resolve_existing_asset(&self, assets_dir: &Path, file_name: &str) -> Option<String> {
        if !assets_dir.is_dir() {
            return None;
        }
        let candidate =
            SecurityManager::ensure_within_base(assets_dir, Path::new(file_name)).ok()?;
        if candidate.is_file() {
            Some(candidate.to_string_lossy().to_string())
        } else {
            None
        }
    }

    fn copy_validated_image(
        &self,
        source_root: &Path,
        relative_path: &str,
        dest_assets_dir: &Path,
        prefix: &str,
    ) -> Result<PathBuf, DomainError> {
        let candidate = SecurityManager::ensure_within_base(source_root, Path::new(relative_path))?;
        let valid_src = SecurityManager::validate_existing_file(&candidate)?;
        let meta = fs::metadata(&valid_src).map_err(|e| {
            DomainError::Io(format!(
                "failed to stat theme image '{}': {e}",
                valid_src.display()
            ))
        })?;
        if meta.len() == 0 || meta.len() > MAX_COVER_BYTES {
            return Err(DomainError::Validation(format!(
                "imagem de tema '{}' vazia ou acima de 10 MB",
                relative_path
            )));
        }

        let mut file = File::open(&valid_src).map_err(|e| {
            DomainError::Io(format!(
                "failed to open theme image '{}': {e}",
                valid_src.display()
            ))
        })?;
        let mut header = [0u8; 16];
        let n = file
            .read(&mut header)
            .map_err(|e| DomainError::Io(format!("failed to read theme image header: {e}")))?;
        drop(file);

        let ext = CoverCache::detect_image_extension(&header[..n]).ok_or_else(|| {
            DomainError::Validation(format!(
                "arquivo de imagem '{}' falsificado ou não suportado (apenas PNG, JPEG ou WebP reais)",
                relative_path
            ))
        })?;
        let (sha, _) = SecurityManager::compute_sha256_streaming(&valid_src)?;
        let short_hash = &sha[..16];
        let dest_name = format!("{prefix}_{short_hash}.{ext}");
        let dest = SecurityManager::ensure_within_base(dest_assets_dir, Path::new(&dest_name))?;
        fs::copy(&valid_src, &dest)
            .map_err(|e| DomainError::Io(format!("failed to copy validated theme image: {e}")))?;
        Ok(dest)
    }
}
