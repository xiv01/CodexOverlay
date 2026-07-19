use crate::settings::{self, SavedWindowPlacement, Settings};
use tauri::{PhysicalPosition, WebviewWindow};

pub fn pin(window: &WebviewWindow, pinned: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        window
            .set_ignore_cursor_events(pinned)
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        let _ = pinned;
    }
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn save_current(app: &tauri::AppHandle, window: &WebviewWindow, pinned: bool) {
    let mut config = settings::load(app);
    if let Ok(position) = window.outer_position() {
        let monitor = window.current_monitor().ok().flatten();
        config.placement = Some(SavedWindowPlacement {
            x: position.x as f64,
            y: position.y as f64,
            monitor_name: monitor.and_then(|m| m.name().cloned()),
            scale_factor: window.scale_factor().unwrap_or(1.0),
            pinned,
        });
        settings::save(app, &config);
    }
}

pub fn restore(_app: &tauri::AppHandle, window: &WebviewWindow, settings: &Settings) -> bool {
    let Some(saved) = &settings.placement else {
        return false;
    };
    let monitors = window.available_monitors().unwrap_or_default();
    let size = window.outer_size().unwrap_or_default();
    // Preserve negative coordinates; only recover positions with no meaningful monitor intersection.
    let target = monitors
        .iter()
        .find(|m| m.name() == saved.monitor_name.as_ref())
        .or_else(|| {
            monitors.iter().find(|m| {
                intersects(
                    saved.x as i32,
                    saved.y as i32,
                    size.width as i32,
                    size.height as i32,
                    m.position().x,
                    m.position().y,
                    m.size().width as i32,
                    m.size().height as i32,
                )
            })
        })
        .or_else(|| monitors.first());
    let Some(monitor) = target else {
        return false;
    };
    let x = saved.x as i32;
    let y = saved.y as i32;
    let valid = intersects(
        x,
        y,
        size.width as i32,
        size.height as i32,
        monitor.position().x,
        monitor.position().y,
        monitor.size().width as i32,
        monitor.size().height as i32,
    );
    let (x, y) = if valid {
        (x, y)
    } else {
        (
            monitor.position().x + ((monitor.size().width.saturating_sub(size.width)) / 2) as i32,
            monitor.position().y + 24,
        )
    };
    window.set_position(PhysicalPosition::new(x, y)).is_ok()
}
fn intersects(x: i32, y: i32, w: i32, h: i32, mx: i32, my: i32, mw: i32, mh: i32) -> bool {
    x < mx + mw && x + w > mx && y < my + mh && y + h > my
}
