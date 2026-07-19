use crate::AppState;
use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

pub fn build(app: &tauri::App) -> tauri::Result<()> {
    let settings = crate::settings::load(&app.handle());
    let overlay_settings = MenuItemBuilder::with_id("settings", "Settings...").build(app)?;
    let edit = MenuItemBuilder::with_id("edit-position", "Edit position").build(app)?;
    let refresh = MenuItemBuilder::with_id("refresh", "Refresh now").build(app)?;
    let details = MenuItemBuilder::with_id("details", "Show details").build(app)?;
    let visible = MenuItemBuilder::with_id("toggle-visible", "Hide overlay").build(app)?;
    let startup = CheckMenuItemBuilder::with_id("autostart", "Start with Windows")
        .checked(settings.autostart)
        .build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .text("title", "Codex Usage Overlay")
        .separator()
        .items(&[
            &overlay_settings,
            &edit,
            &refresh,
            &details,
            &visible,
            &startup,
        ])
        .item(&PredefinedMenuItem::separator(app)?)
        .item(&quit)
        .build()?;
    let mut tray = TrayIconBuilder::with_id("main-tray");
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.menu(&menu)
        .tooltip("Codex Usage Overlay")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                let _ = crate::enter_placement(app);
                let _ = app.emit("show-settings", ());
            }
            "edit-position" => {
                let _ = crate::enter_placement(app);
            }
            "refresh" => crate::refresh_background(app.clone()),
            "details" => {
                let _ = app.emit("show-details", ());
                let _ = crate::enter_placement(app);
            }
            "toggle-visible" => {
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(true) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                    }
                }
            }
            "autostart" => {
                use tauri_plugin_autostart::ManagerExt;
                let enabled = app.autolaunch().is_enabled().unwrap_or(false);
                let result = if enabled {
                    app.autolaunch().disable()
                } else {
                    app.autolaunch().enable()
                };
                if result.is_ok() {
                    let mut settings = crate::settings::load(app);
                    settings.autostart = !enabled;
                    crate::settings::save(app, &settings);
                }
            }
            "quit" => {
                if let Some(state) = app.try_state::<AppState>() {
                    state.stop();
                }
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
