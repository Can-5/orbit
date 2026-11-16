//! Static registry of Orbit's modules.
//!
//! The shell uses this to render navigation and to enforce honesty: modules
//! not yet implemented are labelled with their phase instead of being shown
//! as working features. In the 12-repo ecosystem the canonical module
//! implementations live in their own repositories; this list describes what
//! the Orbit shell can host and under which phase they land.

use crate::models::ModuleInfo;
use crate::permissions::Permission;

/// The modules Orbit can host, in navigation order.
pub fn modules() -> Vec<ModuleInfo> {
    vec![
    ModuleInfo {
        id: "ghostdrop".into(),
        name: "GhostDrop".into(),
        description: "Geçici dosya ve içerik alanı — dosyalarını bırak, gerektiğinde al.".into(),
        phase: 2,
        required_permissions: vec![],
    },
    ModuleInfo {
        id: "recall".into(),
        name: "Recall".into(),
        description: "Görsel pano geçmişi — arama, favoriler, otomatik silme.".into(),
        phase: 2,
        required_permissions: vec![
            Permission::ClipboardRead.id().into(),
            Permission::ClipboardMonitor.id().into(),
        ],
    },
    ModuleInfo {
        id: "snapspace".into(),
        name: "SnapSpace".into(),
        description: "Çalışma alanı profilleri — pencere düzenlerini kaydet ve geri yükle.".into(),
        phase: 3,
        required_permissions: vec![Permission::WindowManagement.id().into()],
    },
    ModuleInfo {
        id: "renamex".into(),
        name: "RenameX".into(),
        description: "Toplu dosya isimlendirme — regex, tarih, EXIF ve geri alma.".into(),
        phase: 2,
        required_permissions: vec![Permission::FileSystemWatch.id().into()],
    },
    ModuleInfo {
        id: "pixelpick".into(),
        name: "PixelPick".into(),
        description: "Ekran cetveli, renk seçici ve palet yöneticisi.".into(),
        phase: 2,
        required_permissions: vec![Permission::ScreenCapture.id().into()],
    },
    ModuleInfo {
        id: "linknest".into(),
        name: "LinkNest".into(),
        description: "Yerel bağlantı ve yer imi yöneticisi — etiketler, arama, dışa aktarma.".into(),
        phase: 3,
        required_permissions: vec![Permission::Network.id().into()],
    },
    ModuleInfo {
        id: "keyglow".into(),
        name: "KeyGlow".into(),
        description: "Klavye görselleştirme ve ses efektleri — yazdığını asla kaydetmez.".into(),
        phase: 4,
        required_permissions: vec![Permission::KeyboardListen.id().into()],
    },
    ModuleInfo {
        id: "cleanshot".into(),
        name: "CleanShot".into(),
        description: "Ekran görüntüsü al, işaretle, bulanıklaştır, arşivle.".into(),
        phase: 3,
        required_permissions: vec![Permission::ScreenCapture.id().into()],
    },
    ModuleInfo {
        id: "apppulse".into(),
        name: "AppPulse".into(),
        description: "CPU/RAM ve işlem izleme — kaynak kullanım geçmişi.".into(),
        phase: 3,
        required_permissions: vec![Permission::ProcessMetrics.id().into()],
    },
    ModuleInfo {
        id: "themeport".into(),
        name: "ThemePort".into(),
        description: "Görünüm profilleri — duvar kâğıdı, palet, içe/dışa aktarma.".into(),
        phase: 4,
        required_permissions: vec![Permission::ThemeControl.id().into()],
    },
    ModuleInfo {
        id: "orbit_island".into(),
        name: "Orbit Island".into(),
        description: "Dynamic Island benzeri canlı bildirim kapsülü.".into(),
        phase: 1,
        required_permissions: vec![],
    },
    ]
}

pub fn module_by_id(id: &str) -> Option<ModuleInfo> {
    modules().into_iter().find(|m| m.id == id)
}
