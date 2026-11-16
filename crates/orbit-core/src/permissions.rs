//! Explicit, least-privilege permission model.
//!
//! Nothing is collected, monitored or hooked until the user explicitly grants
//! the corresponding permission. Grants are stored locally in SQLite and can
//! be revoked at any time. There is no telemetry permission because Orbit
//! ships with no telemetry — there is nothing to grant.

use serde::{Deserialize, Serialize};

use crate::error::{OrbitError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read clipboard contents (Recall). Gate for ANY clipboard access.
    ClipboardRead,
    /// Watch clipboard change events (Recall). Still requires ClipboardRead to store anything.
    ClipboardMonitor,
    /// Capture the screen (CleanShot, PixelPick).
    ScreenCapture,
    /// Listen to global keyboard events for visualization only (KeyGlow). Never stores characters.
    KeyboardListen,
    /// Read system process and resource metrics (AppPulse).
    ProcessMetrics,
    /// Move/resize other windows (SnapSpace).
    WindowManagement,
    /// Watch filesystem locations for rename/undo journals (RenameX).
    FileSystemWatch,
    /// Network access for link metadata/dead-link checks (LinkNest) and user-initiated updates.
    Network,
    /// Launch Orbit at login.
    Autostart,
    /// Control OS appearance/theme (ThemePort).
    ThemeControl,
}

impl Permission {
    pub fn all() -> &'static [Permission] {
        &[
            Permission::ClipboardRead,
            Permission::ClipboardMonitor,
            Permission::ScreenCapture,
            Permission::KeyboardListen,
            Permission::ProcessMetrics,
            Permission::WindowManagement,
            Permission::FileSystemWatch,
            Permission::Network,
            Permission::Autostart,
            Permission::ThemeControl,
        ]
    }

    pub fn id(self) -> &'static str {
        match self {
            Permission::ClipboardRead => "clipboard_read",
            Permission::ClipboardMonitor => "clipboard_monitor",
            Permission::ScreenCapture => "screen_capture",
            Permission::KeyboardListen => "keyboard_listen",
            Permission::ProcessMetrics => "process_metrics",
            Permission::WindowManagement => "window_management",
            Permission::FileSystemWatch => "file_system_watch",
            Permission::Network => "network",
            Permission::Autostart => "autostart",
            Permission::ThemeControl => "theme_control",
        }
    }

    /// Human-facing label + what the permission actually does. Kept in sync
    /// with the Settings → Permissions UI.
    pub fn description(self) -> (&'static str, &'static str) {
        match self {
            Permission::ClipboardRead => (
                "Panonu oku",
                "Recall pano geçmişini kaydetmek için panodaki metin/görselleri okur. Kapalıyken hiçbir şey okunmaz.",
            ),
            Permission::ClipboardMonitor => (
                "Pano değişikliklerini izle",
                "Recall'un panon değiştiğinde haberdar olmasını sağlar. İzleme tek başına içerik kaydetmez.",
            ),
            Permission::ScreenCapture => (
                "Ekran görüntüsü al",
                "CleanShot ve PixelPick'in ekran yakalamasına izin verir. Her yakalama kullanıcı eylemiyle tetiklenir.",
            ),
            Permission::KeyboardListen => (
                "Klavye olaylarını dinle",
                "KeyGlow tuş basma görselleri için yalnızca tuşun kimliğini alır. Yazdığın metin, şifreler ve karakterler asla kaydedilmez veya saklanmaz.",
            ),
            Permission::ProcessMetrics => (
                "Sistem kaynaklarını oku",
                "AppPulse CPU/RAM ve işlem bilgisini okur. Veriler yalnızca cihazında kalır.",
            ),
            Permission::WindowManagement => (
                "Pencereleri düzenle",
                "SnapSpace pencere konumu/boyutunu değiştirebilir. KDE/X11'de çalışır; Wayland'de sınırlı.",
            ),
            Permission::FileSystemWatch => (
                "Dosya değişikliklerini izle",
                "RenameX geri-alma günlüğü için dizinleri izler. Yalnızca senin seçtiğin klasörler.",
            ),
            Permission::Network => (
                "Ağ erişimi",
                "LinkNest bağlantı başlığı/kontrolü ve uygulama güncelleme denetimi. Varsayılan olarak kapalı; hiçbir şey sessizce gönderilmez.",
            ),
            Permission::Autostart => (
                "Açılışta başlat",
                "Orbit'in oturum açılışında başlamasını sağlar.",
            ),
            Permission::ThemeControl => (
                "Sistem temasını değiştir",
                "ThemePort'un duvar kâğıdı/görünüm ayarlarını değiştirmesine izin verir.",
            ),
        }
    }
}

impl std::str::FromStr for Permission {
    type Err = OrbitError;
    fn from_str(s: &str) -> Result<Self> {
        Permission::all()
            .iter()
            .copied()
            .find(|p| p.id() == s)
            .ok_or_else(|| OrbitError::InvalidInput(format!("unknown permission: {s}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantState {
    /// Explicitly granted by the user.
    Granted,
    /// Explicitly denied or never granted.
    Denied,
}

/// Snapshot of one permission's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub id: String,
    pub granted: bool,
    /// Set when granted/denied; used for audit display in Settings.
    pub updated_at: Option<i64>,
}

/// Pure decision logic, unit-tested without storage.
pub fn require_granted(granted: bool, permission: Permission) -> Result<()> {
    if granted {
        Ok(())
    } else {
        Err(OrbitError::PermissionDenied {
            permission: permission.id().to_string(),
        })
    }
}
