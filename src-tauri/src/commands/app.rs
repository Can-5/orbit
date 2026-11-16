use serde::Serialize;

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub slogan: String,
    pub platform: String,
    pub build: String,
}

#[tauri::command]
pub fn app_info() -> CmdResult<AppInfo> {
    Ok(AppInfo {
        name: "Orbit".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        slogan: "Your desktop, organized around you.".to_string(),
        platform: orbit_core::capabilities::Platform::current()
            .display_name()
            .to_string(),
        build: if cfg!(debug_assertions) { "debug" } else { "release" }.to_string(),
    })
}
