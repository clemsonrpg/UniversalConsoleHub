use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::error::DomainError;

pub const BUILTIN_THEME_IDS: &[&str] = &[
    "universal-modern",
    "console-classic",
    "minimal",
    "xmb-inspired",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeShellLayout {
    Sidebar,
    TopbarBlades,
    MinimalDock,
    XmbCrossbar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeLibraryView {
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeCardAspect {
    Poster,
    Square,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeDensity {
    Comfortable,
    Compact,
    TvLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeFocusStyle {
    Ring,
    GlowBar,
    Bracket,
    XmbWave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeSurfaceStyle {
    Matte,
    Beveled,
    Flat,
    Layered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePageTransition {
    Fade,
    Slide,
    Scale,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeFonts {
    pub display: String,
    pub body: String,
    pub mono: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePalette {
    pub canvas: String,
    pub rail: String,
    pub surface: String,
    pub surface_raised: String,
    pub input: String,
    pub border_subtle: String,
    pub border_strong: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub accent_primary: String,
    pub accent_hover: String,
    pub accent_muted: String,
    pub accent_secondary: String,
    pub focus_ring: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeLayoutConfig {
    pub shell: ThemeShellLayout,
    pub library_default_view: ThemeLibraryView,
    pub card_aspect: ThemeCardAspect,
    pub density: ThemeDensity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeComponentsConfig {
    pub focus_style: ThemeFocusStyle,
    pub surface_style: ThemeSurfaceStyle,
    pub backdrop_blur: bool,
    pub show_hero_art: bool,
    pub show_clock: bool,
    pub radius_sm: u16,
    pub radius_md: u16,
    pub radius_lg: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeMotionConfig {
    pub page_transition: ThemePageTransition,
    pub duration_ms: u16,
    pub respect_reduced_motion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSettingOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeCustomSettingKind {
    Color,
    Select,
    Boolean,
    Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeCustomSettingDef {
    pub key: String,
    pub label: String,
    pub kind: ThemeCustomSettingKind,
    pub default_value: String,
    #[serde(default)]
    pub options: Vec<ThemeSettingOption>,
    #[serde(default)]
    pub min: Option<i32>,
    #[serde(default)]
    pub max: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub hub_version_range: String,
    #[serde(default)]
    pub preview_image: Option<String>,
    #[serde(default)]
    pub background_image: Option<String>,
    pub fonts: ThemeFonts,
    #[serde(default)]
    pub assets: BTreeMap<String, String>,
    pub palette: ThemePalette,
    pub layout: ThemeLayoutConfig,
    pub components: ThemeComponentsConfig,
    pub motion: ThemeMotionConfig,
    #[serde(default)]
    pub customizable_settings: Vec<ThemeCustomSettingDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledThemePackage {
    pub manifest: ThemeManifest,
    pub installed_dir: String,
    pub preview_image_path: Option<String>,
    pub background_image_path: Option<String>,
    pub asset_paths: BTreeMap<String, String>,
    pub installed_at: String,
}

pub fn validate_theme_id(id: &str) -> Result<(), DomainError> {
    let trimmed = id.trim();
    if trimmed.len() < 2 || trimmed.len() > 64 {
        return Err(DomainError::Validation(
            "theme id must be between 2 and 64 characters".to_string(),
        ));
    }
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return Err(DomainError::Validation(
            "theme id cannot be empty".to_string(),
        ));
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(DomainError::Validation(
            "theme id must start with a lowercase letter or digit".to_string(),
        ));
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(DomainError::Validation(
            "theme id may contain only lowercase ASCII letters, digits, '-' and '_'".to_string(),
        ));
    }
    Ok(())
}

pub fn is_builtin_theme_id(id: &str) -> bool {
    BUILTIN_THEME_IDS.contains(&id)
}

fn validate_safe_css_color(field: &str, raw: &str) -> Result<(), DomainError> {
    let v = raw.trim();
    if v.is_empty() || v.len() > 64 {
        return Err(DomainError::Validation(format!(
            "theme palette color '{field}' is invalid"
        )));
    }
    let lower = v.to_ascii_lowercase();
    if lower.contains("url(")
        || lower.contains("expression(")
        || lower.contains(';')
        || lower.contains('{')
        || lower.contains('}')
        || lower.contains('<')
        || lower.contains('>')
    {
        return Err(DomainError::Validation(format!(
            "theme palette color '{field}' contains forbidden characters"
        )));
    }
    let valid_hex = v.starts_with('#')
        && matches!(v.len(), 4 | 5 | 7 | 9)
        && v[1..].chars().all(|c| c.is_ascii_hexdigit());
    let valid_func = (lower.starts_with("rgb(") || lower.starts_with("rgba("))
        && v.ends_with(')')
        && v.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '(' | ')' | ',' | '.' | ' ' | '%' | '/')
        });

    if !valid_hex && !valid_func {
        return Err(DomainError::Validation(format!(
            "theme palette color '{field}' must be a valid hex or rgb/rgba color (got '{v}')"
        )));
    }
    Ok(())
}

fn validate_safe_font_stack(field: &str, raw: &str) -> Result<(), DomainError> {
    let v = raw.trim();
    if v.is_empty() || v.len() > 160 {
        return Err(DomainError::Validation(format!(
            "theme font '{field}' must be between 1 and 160 characters"
        )));
    }
    let lower = v.to_ascii_lowercase();
    if lower.contains("url(")
        || lower.contains("@import")
        || v.contains(';')
        || v.contains('{')
        || v.contains('}')
        || v.contains('<')
        || v.contains('>')
    {
        return Err(DomainError::Validation(format!(
            "theme font '{field}' contains forbidden CSS syntax"
        )));
    }
    Ok(())
}

fn validate_safe_relative_asset(field: &str, raw: &str) -> Result<(), DomainError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 200 {
        return Err(DomainError::Validation(format!(
            "theme asset '{field}' path is invalid"
        )));
    }
    if trimmed.contains("..")
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.contains(':')
    {
        return Err(DomainError::Validation(format!(
            "theme asset '{field}' must be a safe relative path without traversal"
        )));
    }
    let ext = trimmed
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp") {
        return Err(DomainError::Validation(format!(
            "theme asset '{field}' must reference a static PNG, JPEG, or WebP image (got '.{ext}')"
        )));
    }
    Ok(())
}

pub fn validate_theme_manifest(manifest: &ThemeManifest) -> Result<(), DomainError> {
    validate_theme_id(&manifest.id)?;
    if manifest.name.trim().is_empty() || manifest.name.len() > 80 {
        return Err(DomainError::Validation(
            "theme name must be between 1 and 80 characters".to_string(),
        ));
    }
    if manifest.author.trim().is_empty() || manifest.author.len() > 80 {
        return Err(DomainError::Validation(
            "theme author must be between 1 and 80 characters".to_string(),
        ));
    }
    if manifest.version.trim().is_empty() || manifest.version.len() > 32 {
        return Err(DomainError::Validation(
            "theme version must be between 1 and 32 characters".to_string(),
        ));
    }
    if manifest.description.trim().is_empty() || manifest.description.len() > 400 {
        return Err(DomainError::Validation(
            "theme description must be between 1 and 400 characters".to_string(),
        ));
    }
    if manifest.hub_version_range.trim().is_empty() || manifest.hub_version_range.len() > 32 {
        return Err(DomainError::Validation(
            "theme hubVersionRange must be specified".to_string(),
        ));
    }

    validate_safe_font_stack("display", &manifest.fonts.display)?;
    validate_safe_font_stack("body", &manifest.fonts.body)?;
    validate_safe_font_stack("mono", &manifest.fonts.mono)?;

    let p = &manifest.palette;
    for (field, val) in [
        ("canvas", p.canvas.as_str()),
        ("rail", p.rail.as_str()),
        ("surface", p.surface.as_str()),
        ("surfaceRaised", p.surface_raised.as_str()),
        ("input", p.input.as_str()),
        ("borderSubtle", p.border_subtle.as_str()),
        ("borderStrong", p.border_strong.as_str()),
        ("textPrimary", p.text_primary.as_str()),
        ("textSecondary", p.text_secondary.as_str()),
        ("textMuted", p.text_muted.as_str()),
        ("accentPrimary", p.accent_primary.as_str()),
        ("accentHover", p.accent_hover.as_str()),
        ("accentMuted", p.accent_muted.as_str()),
        ("accentSecondary", p.accent_secondary.as_str()),
        ("focusRing", p.focus_ring.as_str()),
    ] {
        validate_safe_css_color(field, val)?;
    }

    if let Some(preview) = manifest.preview_image.as_deref() {
        if !preview.trim().is_empty() {
            validate_safe_relative_asset("previewImage", preview)?;
        }
    }
    if let Some(bg) = manifest.background_image.as_deref() {
        if !bg.trim().is_empty() {
            validate_safe_relative_asset("backgroundImage", bg)?;
        }
    }
    for (k, v) in &manifest.assets {
        if k.trim().is_empty() || k.len() > 64 {
            return Err(DomainError::Validation(
                "theme asset key must be between 1 and 64 characters".to_string(),
            ));
        }
        validate_safe_relative_asset(k, v)?;
    }

    if manifest.components.radius_sm > 32
        || manifest.components.radius_md > 36
        || manifest.components.radius_lg > 44
    {
        return Err(DomainError::Validation(
            "theme border radius values exceed safe bounds".to_string(),
        ));
    }
    if manifest.motion.duration_ms > 500 {
        return Err(DomainError::Validation(
            "theme motion durationMs must be <= 500ms".to_string(),
        ));
    }

    for setting in &manifest.customizable_settings {
        if setting.key.trim().is_empty()
            || !setting
                .key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            return Err(DomainError::Validation(format!(
                "invalid customizable setting key '{}'",
                setting.key
            )));
        }
        if setting.kind == ThemeCustomSettingKind::Color {
            validate_safe_css_color(&setting.key, &setting.default_value)?;
        }
    }

    Ok(())
}
