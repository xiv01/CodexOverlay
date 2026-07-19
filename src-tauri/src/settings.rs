use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::Manager;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedWindowPlacement {
    pub x: f64,
    pub y: f64,
    pub monitor_name: Option<String>,
    pub scale_factor: f64,
    pub pinned: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Settings {
    pub placement: Option<SavedWindowPlacement>,
    pub opacity: f64,
    pub show_border: bool,
    pub show_title: bool,
    pub width: u32,
    pub placement_on_startup: bool,
    pub autostart: bool,
    pub shortcut: String,
    pub codex_path: Option<String>,
    pub tray_tip_shown: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            placement: None,
            opacity: 0.88,
            show_border: true,
            show_title: false,
            width: 370,
            placement_on_startup: false,
            autostart: false,
            shortcut: "Ctrl+Alt+Shift+U".into(),
            codex_path: None,
            tray_tip_shown: false,
        }
    }
}
pub fn path(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("settings.json")
}
pub fn load(app: &tauri::AppHandle) -> Settings {
    fs::read(path(app))
        .ok()
        .and_then(|s| serde_json::from_slice(&s).ok())
        .unwrap_or_default()
}
pub fn save(app: &tauri::AppHandle, settings: &Settings) {
    let file = path(app);
    if let Some(parent) = file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_vec_pretty(settings) {
        let _ = fs::write(file, data);
    }
}
