use std::fs;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use universal_console_hub_lib::application::theme_service::ThemePackageService;
use universal_console_hub_lib::infrastructure::system::windows_gamepad::{
    apply_scaled_radial_deadzone, GamepadNavAction, RepeatGate, WindowsGamepadMonitor,
};

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn valid_theme_json(id: &str, preview: Option<&str>) -> String {
    let preview_line = match preview {
        Some(p) => format!("\"previewImage\": \"{p}\","),
        None => "\"previewImage\": null,".to_string(),
    };
    format!(
        r##"{{
  "id": "{id}",
  "name": "Nordic Slate Custom",
  "author": "Community Author",
  "version": "1.2.0",
  "description": "Custom declarative theme package with verified PNG preview.",
  "hubVersionRange": ">=0.1.0",
  {preview_line}
  "backgroundImage": null,
  "fonts": {{
    "display": "'Segoe UI Variable Display', sans-serif",
    "body": "'Segoe UI', sans-serif",
    "mono": "'Cascadia Code', monospace"
  }},
  "assets": {{}},
  "palette": {{
    "canvas": "#0b1118",
    "rail": "#101823",
    "surface": "#162130",
    "surfaceRaised": "#1d2b3e",
    "input": "#0e151f",
    "borderSubtle": "#26364c",
    "borderStrong": "#384f6e",
    "textPrimary": "#f0f6fc",
    "textSecondary": "#a3b5c9",
    "textMuted": "#6d8299",
    "accentPrimary": "#38bdf8",
    "accentHover": "#7dd3fc",
    "accentMuted": "rgba(56, 189, 248, 0.16)",
    "accentSecondary": "#f59e0b",
    "focusRing": "#38bdf8"
  }},
  "layout": {{
    "shell": "sidebar",
    "libraryDefaultView": "grid",
    "cardAspect": "poster",
    "density": "comfortable"
  }},
  "components": {{
    "focusStyle": "ring",
    "surfaceStyle": "matte",
    "backdropBlur": true,
    "showHeroArt": true,
    "showClock": true,
    "radiusSm": 6,
    "radiusMd": 10,
    "radiusLg": 16
  }},
  "motion": {{
    "pageTransition": "fade",
    "durationMs": 160,
    "respectReducedMotion": true
  }},
  "customizableSettings": [
    {{
      "key": "accentColor",
      "label": "Cor de destaque",
      "kind": "color",
      "defaultValue": "#38bdf8"
    }}
  ]
}}"##
    )
}

#[test]
fn imports_lists_and_removes_safe_theme_package_and_blocks_unsafe_packages() {
    let tmp = TempDir::new().expect("tempdir");
    let installed_dir = tmp.path().join("installed_themes");
    let svc = ThemePackageService::new(installed_dir.clone());

    // 1. Valid local theme package directory with genuine PNG preview
    let pkg_dir = tmp.path().join("pkg_nordic");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(pkg_dir.join("preview.png"), MINIMAL_PNG).unwrap();
    fs::write(
        pkg_dir.join("theme.json"),
        valid_theme_json("nordic-slate", Some("preview.png")),
    )
    .unwrap();

    let imported = svc
        .import_theme_package(&pkg_dir)
        .expect("import valid theme");
    assert_eq!(imported.manifest.id, "nordic-slate");
    assert!(imported
        .preview_image_path
        .as_deref()
        .unwrap()
        .contains("nordic-slate"));

    let listed = svc.list_installed_themes().expect("list themes");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].manifest.name, "Nordic Slate Custom");

    // 2. Reject spoofed non-PNG image & executable extension
    let bad_dir = tmp.path().join("pkg_spoofed");
    fs::create_dir_all(&bad_dir).unwrap();
    fs::write(bad_dir.join("preview.png"), b"<script>alert(1)</script>").unwrap();
    fs::write(
        bad_dir.join("theme.json"),
        valid_theme_json("spoofed-theme", Some("preview.png")),
    )
    .unwrap();
    assert!(
        svc.import_theme_package(&bad_dir).is_err(),
        "spoofed PNG must be rejected by magic-byte check"
    );

    // 3. Reject CSS injection in palette/font & reserved builtin ID
    let xss_manifest =
        valid_theme_json("css-inject", None).replace("#0b1118", "url(javascript:alert(1))");
    let xss_file = tmp.path().join("xss.json");
    fs::write(&xss_file, xss_manifest).unwrap();
    assert!(svc.import_theme_package(&xss_file).is_err());

    let reserved_file = tmp.path().join("reserved.json");
    fs::write(&reserved_file, valid_theme_json("universal-modern", None)).unwrap();
    assert!(svc.import_theme_package(&reserved_file).is_err());

    // 4. Reject removing built-in theme, allow removing user theme cleanly
    assert!(svc.remove_installed_theme("universal-modern").is_err());
    assert!(svc
        .remove_installed_theme("nordic-slate")
        .expect("remove custom theme"));
    assert!(svc.list_installed_themes().unwrap().is_empty());
}

#[test]
fn gamepad_deadzone_repeat_gate_and_emulator_handoff_pause_work_deterministically() {
    // Scaled radial deadzone suppresses drift and scales smoothly outside deadzone
    assert_eq!(apply_scaled_radial_deadzone(0.10, -0.10, 0.24), (0.0, 0.0));
    let (sx, sy) = apply_scaled_radial_deadzone(0.95, 0.0, 0.24);
    assert!(sx > 0.9 && sy.abs() < 0.001);

    // Anti-repeat gate: Confirm fires strictly once per press; Down repeats cleanly after delay
    let mut gate = RepeatGate::new(Duration::from_millis(380), Duration::from_millis(100));
    let t0 = Instant::now();
    assert_eq!(
        gate.update(GamepadNavAction::Confirm, true, false, t0),
        Some(false)
    );
    assert_eq!(
        gate.update(
            GamepadNavAction::Confirm,
            true,
            false,
            t0 + Duration::from_millis(500)
        ),
        None,
        "non-repeatable button must never trigger multiple activations while held"
    );
    assert_eq!(
        gate.update(
            GamepadNavAction::Confirm,
            false,
            false,
            t0 + Duration::from_millis(550)
        ),
        None
    );
    assert_eq!(
        gate.update(
            GamepadNavAction::Confirm,
            true,
            false,
            t0 + Duration::from_millis(600)
        ),
        Some(false)
    );

    assert_eq!(
        gate.update(GamepadNavAction::Down, true, true, t0),
        Some(false)
    );
    assert_eq!(
        gate.update(
            GamepadNavAction::Down,
            true,
            true,
            t0 + Duration::from_millis(200)
        ),
        None
    );
    assert_eq!(
        gate.update(
            GamepadNavAction::Down,
            true,
            true,
            t0 + Duration::from_millis(400)
        ),
        Some(true)
    );

    // Emulator handoff pause/resume state
    let monitor = WindowsGamepadMonitor::new();
    assert!(!monitor.is_paused());
    monitor.pause();
    assert!(monitor.is_paused());
    monitor.resume_with_drain(Duration::from_millis(350));
    assert!(!monitor.is_paused());
}
