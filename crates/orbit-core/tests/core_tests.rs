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
