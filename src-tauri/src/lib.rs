mod codex;
mod settings;
mod tray;
mod window;

use codex::{
    normalize::{apply_limits, merge_limits},
    CodexAppServerClient, CodexUsageState, RawRateLimits,
};
use serde::Serialize;
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

const AUTH_REQUIRED: &str = "auth-required";

pub struct AppState {
    stopped: Arc<AtomicBool>,
    refresh: Arc<AtomicBool>,
    pinned: Arc<AtomicBool>,
    usage: Mutex<CodexUsageState>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayPreferences {
    opacity: f64,
    show_border: bool,
    show_title: bool,
    width: u32,
}
impl AppState {
    fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
    }
}

fn emit(app: &AppHandle, state: &CodexUsageState) {
    if let Some(app_state) = app.try_state::<AppState>() {
        if let Ok(mut saved) = app_state.usage.lock() {
            *saved = state.clone();
        }
    }
    let _ = app.emit("usage-state", state);
}
fn parse_limits(value: &Value) -> (RawRateLimits, Option<i64>) {
    let result = value.get("result").unwrap_or(value);
    let limits = serde_json::from_value(result.get("rateLimits").cloned().unwrap_or_default())
        .unwrap_or_default();
    let credits = result
        .get("rateLimitResetCredits")
        .and_then(|v| v.get("availableCount"))
        .and_then(Value::as_i64);
    (limits, credits)
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum AccountAuth {
    Missing,
    ChatGpt,
    Unsupported,
}

fn account_auth(value: &Value) -> AccountAuth {
    let Some(account) = value.get("result").and_then(|result| result.get("account")) else {
        return AccountAuth::Missing;
    };
    if account.is_null() || !account.is_object() {
        return AccountAuth::Missing;
    }
    let auth_type = account
        .get("type")
        .or_else(|| account.get("authType"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .replace(['_', '-'], "");
    if auth_type == "chatgpt" {
        AccountAuth::ChatGpt
    } else {
        AccountAuth::Unsupported
    }
}

fn emit_auth_state(app: &AppHandle, auth: AccountAuth) {
    let mut state = CodexUsageState::default();
    state.status = match auth {
        AccountAuth::Unsupported => "unsupported-auth",
        _ => "logged-out",
    }
    .into();
    emit(app, &state);
}

fn connection_error(error: codex::client::ClientError, fallback: &str) -> String {
    match error {
        codex::client::ClientError::Authentication => AUTH_REQUIRED.into(),
        codex::client::ClientError::Process(message) => message,
        _ => fallback.into(),
    }
}

async fn connect_once(
    app: AppHandle,
    custom_path: Option<String>,
    stopped: Arc<AtomicBool>,
    refresh: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut client = CodexAppServerClient::spawn(custom_path.as_deref())
        .await
        .map_err(|error| connection_error(error, "Could not start Codex app-server"))?;
    client
        .initialize()
        .await
        .map_err(|error| connection_error(error, "Codex app-server initialization failed"))?;
    tracing::info!("Codex app-server initialized");
    let account = client
        .read_account()
        .await
        .map_err(|error| connection_error(error, "Could not read Codex account"))?;
    let mut state = CodexUsageState::default();
    let mut auth = account_auth(&account);
    if auth != AccountAuth::ChatGpt {
        emit_auth_state(&app, auth);
        let mut auth_tick = tokio::time::interval(std::time::Duration::from_secs(30));
        auth_tick.tick().await;
        loop {
            tokio::select! {
                _=auth_tick.tick() => {
                    let updated = client.read_account().await.map_err(|error| connection_error(error, "Could not read Codex account"))?;
                    let next = account_auth(&updated);
                    if next == AccountAuth::ChatGpt { break; }
                    if next != auth { auth = next; emit_auth_state(&app, auth); }
                },
                _=tokio::time::sleep(std::time::Duration::from_millis(250)), if stopped.load(Ordering::Relaxed) => {client.shutdown().await;return Ok(())},
                _=tokio::time::sleep(std::time::Duration::from_millis(250)), if refresh.swap(false, Ordering::Relaxed) => {client.shutdown().await;return Err("Manual refresh requested".into())}
            }
        }
    }
    let response = client
        .refresh_rate_limits()
        .await
        .map_err(|error| connection_error(error, "Could not read rate limits"))?;
    let (mut raw, mut credits) = parse_limits(&response);
    apply_limits(&mut state, &raw, credits);
    emit(&app, &state);
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        tokio::select! {
            _=tick.tick() => match client.refresh_rate_limits().await { Ok(v)=> {let (updated,c)=parse_limits(&v); merge_limits(&mut raw,updated); if c.is_some(){credits=c}; apply_limits(&mut state,&raw,credits);emit(&app,&state);tracing::info!("Codex usage refreshed");}, Err(error)=>return Err(connection_error(error, "Rate-limit refresh failed")) },
            notification=client.next_notification() => match notification { Some(value) if value.get("method").and_then(Value::as_str)==Some("account/rateLimits/updated") => { let (updated,c)=parse_limits(value.get("params").unwrap_or(&value)); merge_limits(&mut raw,updated);if c.is_some(){credits=c};apply_limits(&mut state,&raw,credits);emit(&app,&state);}, Some(_)=>{}, None=>return Err("Codex app-server exited".into()) },
            _=tokio::time::sleep(std::time::Duration::from_millis(250)), if stopped.load(Ordering::Relaxed) => {client.shutdown().await;return Ok(())},
            _=tokio::time::sleep(std::time::Duration::from_millis(250)), if refresh.swap(false, Ordering::Relaxed) => {client.shutdown().await;return Err("Manual refresh requested".into())}
        }
    }
}

fn start_supervisor(app: AppHandle, stopped: Arc<AtomicBool>, refresh: Arc<AtomicBool>) {
    tauri::async_runtime::spawn(async move {
        if std::env::var("CODEX_OVERLAY_MOCK").ok().as_deref() == Some("1") {
            let mut state = CodexUsageState::default();
            state.status = "connected".into();
            state.primary = Some(codex::normalize::QuotaWindow {
                used_percent: Some(28.),
                remaining_percent: Some(72.),
                window_duration_mins: Some(300),
                label: "5h".into(),
                resets_at: Some(chrono::Utc::now().timestamp() + 8100),
            });
            state.secondary = Some(codex::normalize::QuotaWindow {
                used_percent: Some(52.),
                remaining_percent: Some(48.),
                window_duration_mins: Some(10080),
                label: "7d".into(),
                resets_at: Some(chrono::Utc::now().timestamp() + 250000),
            });
            emit(&app, &state);
            return;
        }
        let settings = settings::load(&app);
        let mut delay = 1u64;
        let mut show_reconnecting = true;
        loop {
            let mut state = CodexUsageState::default();
            if show_reconnecting {
                state.status = "reconnecting".into();
                emit(&app, &state);
            }
            let result = connect_once(
                app.clone(),
                settings.codex_path.clone(),
                stopped.clone(),
                refresh.clone(),
            )
            .await;
            match result {
                Ok(()) if stopped.load(Ordering::Relaxed) => break,
                Ok(()) => {
                    show_reconnecting = true;
                    state.status = "stale".into();
                    state.error_message = Some("Codex app-server is reconnecting".into());
                    emit(&app, &state);
                    let jitter = rand::random::<u64>() % 400;
                    tokio::time::sleep(std::time::Duration::from_millis(delay * 1000 + jitter))
                        .await;
                    delay = (delay * 2).min(30);
                }
                Err(error) => {
                    let auth_required = error == AUTH_REQUIRED;
                    state.status = if auth_required {
                        "logged-out".into()
                    } else if error.contains("not found") {
                        "codex-not-found".into()
                    } else {
                        "stale".into()
                    };
                    state.error_message = if auth_required {
                        Some("Run codex login".into())
                    } else {
                        Some("Codex app-server is reconnecting".into())
                    };
                    emit(&app, &state);
                    show_reconnecting = !auth_required;
                    let retry_delay = if auth_required { 30 } else { delay };
                    let jitter = rand::random::<u64>() % 400;
                    tokio::time::sleep(std::time::Duration::from_millis(
                        retry_delay * 1000 + jitter,
                    ))
                    .await;
                    delay = if auth_required {
                        1
                    } else {
                        (delay * 2).min(30)
                    };
                }
            }
        }
    });
}

#[tauri::command]
fn set_pinned(app: AppHandle, pinned: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Overlay window unavailable")?;
    window::placement::save_current(&app, &window, pinned);
    window::placement::pin(&window, pinned)?;
    app.state::<AppState>()
        .pinned
        .store(pinned, Ordering::Relaxed);
    if pinned {
        show_tray_tip(&app);
    }
    Ok(())
}

fn show_tray_tip(app: &AppHandle) {
    let mut settings = settings::load(app);
    if settings.tray_tip_shown {
        return;
    }
    if app
        .notification()
        .builder()
        .title("Overlay pinned")
        .body("Settings are in the tray menu.")
        .show()
        .is_ok()
    {
        settings.tray_tip_shown = true;
        settings::save(app, &settings);
    }
}
#[tauri::command]
fn toggle_placement(app: AppHandle) -> Result<(), String> {
    if app.state::<AppState>().pinned.load(Ordering::Relaxed) {
        enter_placement(&app)
    } else {
        set_pinned(app, true)
    }
}
#[tauri::command]
fn refresh_usage(app: AppHandle) {
    refresh_background(app);
}
#[tauri::command]
fn show_details(app: AppHandle) {
    let _ = app.emit("show-details", ());
}
#[tauri::command]
fn current_usage_state(app: AppHandle) -> CodexUsageState {
    app.state::<AppState>()
        .usage
        .lock()
        .map(|state| state.clone())
        .unwrap_or_default()
}
#[tauri::command]
fn is_pinned(app: AppHandle) -> bool {
    app.state::<AppState>().pinned.load(Ordering::Relaxed)
}
#[tauri::command]
fn overlay_preferences(app: AppHandle) -> OverlayPreferences {
    let settings = settings::load(&app);
    OverlayPreferences {
        opacity: settings.opacity,
        show_border: settings.show_border,
        show_title: settings.show_title,
        width: settings.width,
    }
}
#[tauri::command]
fn update_overlay_preferences(
    app: AppHandle,
    opacity: f64,
    show_border: bool,
    show_title: bool,
    width: u32,
) -> Result<(), String> {
    let mut settings = settings::load(&app);
    settings.opacity = opacity.clamp(0.55, 1.0);
    settings.show_border = show_border;
    settings.show_title = show_title;
    settings.width = width.clamp(300, 520);
    let window = app
        .get_webview_window("main")
        .ok_or("Overlay window unavailable")?;
    window
        .set_size(tauri::PhysicalSize::new(settings.width, 42))
        .map_err(|e| e.to_string())?;
    settings::save(&app, &settings);
    Ok(())
}
#[tauri::command]
fn set_settings_open(app: AppHandle, open: bool) -> Result<(), String> {
    let settings = settings::load(&app);
    let window = app
        .get_webview_window("main")
        .ok_or("Overlay window unavailable")?;
    window
        .set_size(tauri::PhysicalSize::new(
            settings.width,
            if open { 180 } else { 42 },
        ))
        .map_err(|e| e.to_string())
}
fn enter_placement(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Overlay window unavailable")?;
    window::placement::pin(&window, false)?;
    app.state::<AppState>()
        .pinned
        .store(false, Ordering::Relaxed);
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    let _ = app.emit("placement-mode", true);
    Ok(())
}
fn refresh_background(app: AppHandle) {
    app.state::<AppState>()
        .refresh
        .store(true, Ordering::Relaxed);
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .compact()
        .init();
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = toggle_placement(app.clone());
                    }
                })
                .build(),
        )
        .manage(AppState {
            stopped: Arc::new(AtomicBool::new(false)),
            refresh: Arc::new(AtomicBool::new(false)),
            pinned: Arc::new(AtomicBool::new(false)),
            usage: Mutex::new(CodexUsageState::default()),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = settings::load(&handle);
            let window = app.get_webview_window("main").expect("main window");
            let _ = window.set_size(tauri::PhysicalSize::new(settings.width, 42));
            let restored = window::placement::restore(&handle, &window, &settings);
            if !restored {
                if let Some(m) = window.primary_monitor()? {
                    let x = m.position().x + ((m.size().width.saturating_sub(370)) / 2) as i32;
                    let _ =
                        window.set_position(tauri::PhysicalPosition::new(x, m.position().y + 24));
                }
            };
            // Always restore interaction on launch so the overlay can be repositioned immediately.
            window::placement::pin(&window, false)
                .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
            handle
                .state::<AppState>()
                .pinned
                .store(false, Ordering::Relaxed);
            let _ = handle.emit("placement-mode", true);
            tray::build(app)?;
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            let shortcut = settings
                .shortcut
                .parse::<tauri_plugin_global_shortcut::Shortcut>()
                .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
            handle
                .global_shortcut()
                .register(shortcut)
                .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
            let (stopped, refresh) = {
                let state = handle.state::<AppState>();
                (state.stopped.clone(), state.refresh.clone())
            };
            start_supervisor(handle, stopped, refresh);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_pinned,
            toggle_placement,
            refresh_usage,
            show_details,
            current_usage_state,
            is_pinned,
            overlay_preferences,
            update_overlay_preferences,
            set_settings_open
        ])
        .run(tauri::generate_context!())
        .expect("error while running Codex Usage Overlay");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_missing_auth() {
        assert_eq!(
            account_auth(&json!({"result":{"account":null}})),
            AccountAuth::Missing
        );
        assert_eq!(account_auth(&json!({"result":{}})), AccountAuth::Missing);
        assert_eq!(account_auth(&json!({})), AccountAuth::Missing);
    }

    #[test]
    fn detects_supported_and_unsupported_auth() {
        assert_eq!(
            account_auth(&json!({"result":{"account":{"type":"chatgpt"}}})),
            AccountAuth::ChatGpt
        );
        assert_eq!(
            account_auth(&json!({"result":{"account":{"type":"apiKey"}}})),
            AccountAuth::Unsupported
        );
        assert_eq!(
            account_auth(&json!({"result":{"account":{"authType":"api_key"}}})),
            AccountAuth::Unsupported
        );
        assert_eq!(
            account_auth(&json!({"result":{"account":{"type":"amazonBedrock"}}})),
            AccountAuth::Unsupported
        );
    }
}
