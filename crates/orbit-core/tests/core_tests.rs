use orbit_core::island::{IslandContent, IslandManager, IslandView, Priority};
use orbit_core::settings::AppSettings;
use orbit_core::storage::Database;
use orbit_core::permissions::{Permission, require_granted};
use orbit_core::capabilities::{capability_report, capability_support, Capability, Platform, Support};
use orbit_core::module_registry::{module_by_id, modules};

#[test]
fn settings_roundtrip_and_validation() {
    let db = Database::in_memory().unwrap();
    let defaults = db.load_settings().unwrap();
    assert_eq!(defaults.accent, "#6c8cff");

    let mut s = AppSettings::default();
    s.accent = "#FF8C00".to_string(); // normalized to lowercase
    let saved = db.save_settings(&s).unwrap();
    assert_eq!(saved.accent, "#ff8c00");

    let loaded = db.load_settings().unwrap();
    assert_eq!(loaded.accent, "#ff8c00");

    let mut bad = AppSettings::default();
    bad.accent = "red".to_string();
    assert!(db.save_settings(&bad).is_err());
}

#[test]
fn permissions_default_denied_then_granted() {
    let db = Database::in_memory().unwrap();
    assert!(!db.is_granted(Permission::ClipboardRead).unwrap());
    assert!(require_granted(false, Permission::ClipboardRead).is_err());

    db.set_permission(Permission::ClipboardRead, true).unwrap();
    assert!(db.is_granted(Permission::ClipboardRead).unwrap());
    assert!(require_granted(true, Permission::ClipboardRead).is_ok());

    // All 10 known permissions appear in the listing, defaulting to denied.
    let list = db.list_permissions().unwrap();
    assert_eq!(list.len(), Permission::all().len());
    assert!(list.iter().filter(|p| !p.granted).count() >= list.len() - 1);

    // Revoke works.
    db.set_permission(Permission::ClipboardRead, false).unwrap();
    assert!(!db.is_granted(Permission::ClipboardRead).unwrap());
}

#[test]
fn migrations_are_idempotent() {
    let dir = std::env::temp_dir().join(format!("orbit-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("orbit.db");
    {
        let db = Database::open(&path).unwrap();
        assert_eq!(db.schema_version().unwrap(), 1);
    }
    {
        let db = Database::open(&path).unwrap(); // reopen — migrations must not re-run/destruct
        assert_eq!(db.schema_version().unwrap(), 1);
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn island_priority_and_dismiss_flow() {
    let mut island = IslandManager::new();
    assert_eq!(island.snapshot().view, IslandView::Collapsed);

    let low = island.push(IslandContent::Toast { message: "low".into() }, Priority::Low, None);
    island.push(IslandContent::Toast { message: "high".into() }, Priority::High, None);
    // High preempted low.
    let snap = island.snapshot();
    assert_eq!(snap.active.as_ref().unwrap().content,
               IslandContent::Toast { message: "high".into() });
    assert_eq!(snap.queued, 1);

    island.dismiss(low);
    let snap = island.snapshot();
    assert_eq!(snap.queued, 0);

    island.dismiss_active();
    assert!(island.snapshot().active.is_none());

    let _ = island.toggle_expand();
    assert_eq!(island.snapshot().view, IslandView::Expanded);
}

#[test]
fn island_timer_lifecycle() {
    let mut island = IslandManager::new();
    island.start_timer("Odak", 25 * 60);
    let remaining = island.snapshot().timer_remaining_secs.unwrap();
    assert!(remaining <= 25 * 60 && remaining > 25 * 60 - 2);

    island.toggle_timer_pause();
    let paused = island.snapshot();
    assert!(paused.timer.as_ref().unwrap().paused);

    island.toggle_timer_pause(); // resume
    island.cancel_timer();
    assert!(island.snapshot().timer.is_none());
}

#[test]
fn capability_matrix_is_honest_everywhere() {
    // Every platform produces a complete report; no panics, no silent gaps.
    for platform in [Platform::Linux, Platform::Windows, Platform::MacOs, Platform::Android] {
        let report = capability_report(platform);
        assert_eq!(report.len(), 11);
        for (_, support) in report {
            match support {
                Support::Limited { note } | Support::Unsupported { note } => assert!(!note.is_empty()),
                Support::Supported => {}
            }
        }
    }
    // Wayland-relevant honesty: overlay is limited, not falsely "supported", on Linux.
    assert!(matches!(
        capability_support(Capability::IslandOverlayWindow, Platform::Linux),
        Support::Limited { .. }
    ));
    assert!(capability_support(Capability::ClipboardRead, Platform::Android).is_usable());
}

#[test]
fn module_registry_is_consistent() {
    let modules = modules();
    assert_eq!(modules.len(), 11);
    for module in &modules {
        assert!(!module.id.is_empty() && !module.name.is_empty());
        assert!((1..=5).contains(&module.phase));
        // Every declared permission id resolves.
        for pid in &module.required_permissions {
            assert!(pid.parse::<Permission>().is_ok(), "unknown permission {pid}");
        }
    }
    assert!(module_by_id("orbit_island").is_some());
    assert!(module_by_id("nope").is_none());
}

use orbit_core::manifest::{
    detect_installed_modules, is_semverish, is_slug, validate_manifest, InstallState, ModuleManifest,
};
use std::fs;

fn temp_modules_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("orbit-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sample_manifest_json() -> &'static str {
    r#"{"format":1,"id":"ghostdrop","name":"GhostDrop","version":"0.1.0","protocol":"1","description":"test module","permissions":["clipboard_read"],"island":false}"#
}

#[test]
fn manifest_validation_boundaries() {
    assert!(is_slug("ghostdrop-2"));
    assert!(!is_slug("GhostDrop"));
    assert!(!is_slug("-lead"));
    assert!(!is_slug("trail-"));
    assert!(is_semverish("0.1.0"));
    assert!(!is_semverish("0.1"));
    assert!(!is_semverish("0.1.0-rc1"));

    let ok: ModuleManifest = serde_json::from_str(sample_manifest_json()).unwrap();
    assert!(validate_manifest(&ok).is_ok());

    let mut bad = ok.clone();
    bad.format = 2;
    assert!(validate_manifest(&bad).is_err());

    let mut bad = ok.clone();
    bad.protocol = "2".into();
    assert!(validate_manifest(&bad).is_err());

    let mut bad = ok.clone();
    bad.permissions = vec!["teleportation".into()];
    assert!(validate_manifest(&bad).is_err());
}

#[test]
fn detect_installed_modules_classifies() {
    let root = temp_modules_dir("detect");
    fs::create_dir_all(root.join("ghostdrop")).unwrap();
    fs::write(root.join("ghostdrop").join("orbit-module.json"), sample_manifest_json()).unwrap();
    fs::create_dir_all(root.join("broken")).unwrap();
    fs::write(root.join("broken").join("orbit-module.json"), "{not json").unwrap();
    fs::create_dir_all(root.join("no-manifest")).unwrap(); // skipped

    let found = detect_installed_modules(&root);
    assert_eq!(found.len(), 2);
    let by_id: std::collections::HashMap<_, _> =
        found.into_iter().map(|m| (m.id.clone().unwrap_or_default(), m)).collect();
    assert!(matches!(by_id["ghostdrop"].state, InstallState::Ok { .. }));
    let broken = by_id.iter().find(|(k, _)| k.is_empty()).unwrap().1.clone();
    assert!(matches!(broken.state, InstallState::Invalid { .. }));

    let _ = fs::remove_dir_all(&root);
}
