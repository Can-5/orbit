use serde::Serialize;

use orbit_core::capabilities::{capability_report, Capability, Platform};

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityDto {
    pub capability: String,
    pub state: String,
    pub note: Option<String>,
    pub usable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityReport {
    pub platform: String,
    pub capabilities: Vec<CapabilityDto>,
}

#[tauri::command]
pub fn get_capabilities() -> CmdResult<CapabilityReport> {
    let platform = Platform::current();
    let capabilities = capability_report(platform)
        .into_iter()
        .map(|(cap, support)| {
            let (state, note) = match &support {
                orbit_core::capabilities::Support::Supported => ("supported".to_string(), None),
                orbit_core::capabilities::Support::Limited { note } => {
                    ("limited".to_string(), Some(note.clone()))
                }
                orbit_core::capabilities::Support::Unsupported { note } => {
                    ("unsupported".to_string(), Some(note.clone()))
                }
            };
            let _ = Capability::IslandOverlayWindow; // keep import used if matrix evolves
            CapabilityDto {
                capability: format!("{cap:?}"),
                state,
                note,
                usable: support.is_usable(),
            }
        })
        .collect();
    Ok(CapabilityReport {
        platform: platform.display_name().to_string(),
        capabilities,
    })
}
