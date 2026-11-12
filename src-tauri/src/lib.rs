mod config;
mod desktop;
mod openrouter;

use config::{Config, Prompt};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

// Keep the original credential namespace so the rename preserves saved API keys.
const KEY_SERVICE: &str = "com.ronnyschlidt.promptkey";
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray.png");
#[cfg(target_os = "macos")]
const TRAY_DONE_ICON: &[u8] = include_bytes!("../icons/tray-done.png");

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    phase: String,
    message: String,
    prompt_name: Option<String>,
    result: Option<String>,
}

impl Status {
    fn new(phase: &str, message: &str) -> Self {
        Self {
            phase: phase.into(),
            message: message.into(),
            prompt_name: None,
            result: None,
        }
    }
}

struct AppState {
    config: Mutex<Config>,
    status: Mutex<Status>,
    shortcut_errors: Mutex<Vec<String>>,
    busy: AtomicBool,
    recording: AtomicBool,
    /// Counts `publish` calls so a delayed tray-icon reset can tell whether a newer status took over.
    published: AtomicU64,
    client: reqwest::Client,
}

fn credential() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEY_SERVICE, "openrouter")
        .map_err(|_| "The system credential store is unavailable.".into())
}

fn read_key() -> Result<String, String> {
    credential()?
        .get_password()
        .map_err(|_| "Add your OpenRouter API key in Settings first.".into())
}

/// macOS: show a Dock icon only while the settings window is open; otherwise live in the menu bar.
fn set_dock_icon(app: &tauri::AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(if visible {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    });
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        set_dock_icon(app, true);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn publish(app: &tauri::AppHandle, status: Status) {
    let state = app.state::<AppState>();
    if let Ok(mut current) = state.status.lock() {
        *current = status.clone();
    }
    let generation = state.published.fetch_add(1, Ordering::SeqCst) + 1;
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(format!("KeyDock — {}", status.message)));
        #[cfg(target_os = "macos")]
        match status.phase.as_str() {
            // `pulse_tray` owns the icon while an operation runs.
            "working" => {}
            "success" => {
                set_tray_icon(app, TRAY_DONE_ICON);
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    if app.state::<AppState>().published.load(Ordering::SeqCst) == generation {
                        set_tray_icon(&app, TRAY_ICON);
                    }
                });
            }
            _ => set_tray_icon(app, TRAY_ICON),
        }
    }
    #[cfg(target_os = "macos")]
    if status.phase != "idle" {
        show_toast(app);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = generation;
    let _ = app.emit("status", status);
}

/// macOS: a click-through toast at the bottom of the main screen shows progress, results and errors.
/// It never takes focus, because a focused window would turn every run into review. `toast.html`
/// renders the status events and hides the window itself once a run has finished.
#[cfg(target_os = "macos")]
fn show_toast(app: &tauri::AppHandle) {
    let Some(toast) = app.get_webview_window("toast") else {
        return;
    };
    if let (Ok(Some(monitor)), Ok(size)) = (toast.primary_monitor(), toast.outer_size()) {
        let area = monitor.work_area();
        let _ = toast.set_position(tauri::PhysicalPosition::new(
            area.position.x + (area.size.width as i32 - size.width as i32) / 2,
            area.position.y + area.size.height as i32 - size.height as i32,
        ));
    }
    let window = toast.clone();
    let _ = toast.run_on_main_thread(move || {
        use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
        let Ok(pointer) = window.ns_window() else {
            return;
        };
        // SAFETY: Tauri returns the live NSWindow of this webview window; we are on the main thread.
        let ns_window = unsafe { &*pointer.cast::<NSWindow>() };
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
        // `show()` would call makeKeyAndOrderFront; this only orders the window in front.
        ns_window.orderFrontRegardless();
    });
}

/// Plain `set_icon` drops the macOS template flag, which would turn the icon black on a dark menu bar.
#[cfg(target_os = "macos")]
fn set_tray_image(app: &tauri::AppHandle, image: tauri::image::Image<'static>) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_icon_with_as_template(Some(image), true);
    }
}

#[cfg(target_os = "macos")]
fn set_tray_icon(app: &tauri::AppHandle, png: &'static [u8]) {
    if let Ok(image) = tauri::image::Image::from_bytes(png) {
        set_tray_image(app, image);
    }
}

/// macOS: let the menu-bar icon breathe (fade between 100 % and 30 % opacity) until the task is aborted.
/// Windows is not animated and keeps the tooltip only.
#[cfg(target_os = "macos")]
async fn pulse_tray(app: tauri::AppHandle) {
    const FRAMES: usize = 24;
    let Ok(base) = tauri::image::Image::from_bytes(TRAY_ICON) else {
        return;
    };
    let frames: Vec<_> = (0..FRAMES)
        .map(|i| {
            let phase = i as f32 / FRAMES as f32 * std::f32::consts::TAU;
            let opacity = 0.3 + 0.35 * (1.0 + phase.cos());
            let rgba = base
                .rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2], (p[3] as f32 * opacity).round() as u8])
                .collect();
            tauri::image::Image::new_owned(rgba, base.width(), base.height())
        })
        .collect();
    for frame in frames.iter().cycle() {
        set_tray_image(&app, frame.clone());
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    }
}

fn register_shortcuts(app: &tauri::AppHandle, config: &Config) -> Vec<String> {
    let mut errors = Vec::new();
    if let Err(e) = app.global_shortcut().unregister_all() {
        errors.push(format!("Could not release shortcuts: {e}"));
    }
    for prompt in &config.prompts {
        if prompt.enabled && !prompt.shortcut.is_empty() {
            if let Err(e) = app.global_shortcut().register(prompt.shortcut.as_str()) {
                errors.push(format!("{}: shortcut unavailable ({e}). Choose another combination or release it in the other app.", prompt.name));
            }
        }
    }
    errors
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Bootstrap {
    config: Config,
    has_key: bool,
    platform: &'static str,
    accessibility: bool,
    status: Status,
    shortcut_errors: Vec<String>,
}

#[tauri::command]
fn bootstrap(state: tauri::State<AppState>) -> Result<Bootstrap, String> {
    Ok(Bootstrap {
        config: state
            .config
            .lock()
            .map_err(|_| "Settings unavailable")?
            .clone(),
        has_key: read_key().is_ok(),
        platform: if cfg!(target_os = "macos") {
            "macos"
        } else {
            "windows"
        },
        accessibility: desktop::has_permissions(),
        status: state
            .status
            .lock()
            .map_err(|_| "Status unavailable")?
            .clone(),
        shortcut_errors: state
            .shortcut_errors
            .lock()
            .map_err(|_| "Shortcuts unavailable")?
            .clone(),
    })
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: Config) -> Result<Vec<String>, String> {
    config.validate()?;
    let state = app.state::<AppState>();
    if state.busy.load(Ordering::SeqCst) {
        return Err("Wait for the current prompt to finish before saving.".into());
    }
    let old = state
        .config
        .lock()
        .map_err(|_| "Settings unavailable")?
        .clone();
    if config.launch_at_login != old.launch_at_login {
        let manager = app.autolaunch();
        if config.launch_at_login {
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|e| format!("Could not change launch at login: {e}"))?;
    }
    if let Err(error) = config::save(&app, &config) {
        if config.launch_at_login != old.launch_at_login {
            let manager = app.autolaunch();
            let _ = if old.launch_at_login {
                manager.enable()
            } else {
                manager.disable()
            };
        }
        return Err(error);
    }
    *state.config.lock().map_err(|_| "Settings unavailable")? = config.clone();
    let errors = register_shortcuts(&app, &config);
    *state
        .shortcut_errors
        .lock()
        .map_err(|_| "Shortcuts unavailable")? = errors.clone();
    Ok(errors)
}

#[tauri::command]
fn set_api_key(key: String) -> Result<(), String> {
    let key = key.trim();
    if !key.starts_with("sk-or-") || key.len() < 20 || key.chars().any(char::is_whitespace) {
        return Err("Enter a valid OpenRouter API key (starts with sk-or-).".into());
    }
    credential()?
        .set_password(key)
        .map_err(|_| "Could not save the API key in the system credential store.".into())
}

#[tauri::command]
fn delete_api_key() -> Result<(), String> {
    match credential()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not remove the API key.".into()),
    }
}

#[tauri::command]
async fn check_api_key(
    app: tauri::AppHandle,
    model: String,
) -> Result<openrouter::KeyCheck, String> {
    openrouter::check(&app.state::<AppState>().client, &read_key()?, model.trim()).await
}

#[tauri::command]
fn set_recording(app: tauri::AppHandle, active: bool) -> Result<Vec<String>, String> {
    let state = app.state::<AppState>();
    state.recording.store(active, Ordering::SeqCst);
    if active {
        app.global_shortcut()
            .unregister_all()
            .map_err(|e| e.to_string())?;
        Ok(vec![])
    } else {
        let config = state
            .config
            .lock()
            .map_err(|_| "Settings unavailable")?
            .clone();
        let errors = register_shortcuts(&app, &config);
        *state
            .shortcut_errors
            .lock()
            .map_err(|_| "Shortcuts unavailable")? = errors.clone();
        Ok(errors)
    }
}

#[tauri::command]
fn request_accessibility() -> bool {
    desktop::request_permissions()
}

#[tauri::command]
fn copy_result(app: tauri::AppHandle) -> Result<(), String> {
    if app.state::<AppState>().busy.load(Ordering::SeqCst) {
        return Err("Wait for the current prompt to finish.".into());
    }
    let result = app
        .state::<AppState>()
        .status
        .lock()
        .map_err(|_| "Result unavailable")?
        .result
        .clone()
        .ok_or("No result to copy.")?;
    desktop::set_clipboard_text(&result)
}

#[tauri::command]
async fn test_prompt(
    app: tauri::AppHandle,
    prompt: Prompt,
    text: String,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Enter some sample text first.".into());
    }
    let config = app
        .state::<AppState>()
        .config
        .lock()
        .map_err(|_| "Settings unavailable")?
        .clone();
    prompt.validate()?;
    openrouter::complete(
        &app.state::<AppState>().client,
        &read_key()?,
        &prompt,
        &config.default_model,
        &text,
    )
    .await
}

fn execute(app: tauri::AppHandle, prompt: Prompt) {
    let state = app.state::<AppState>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let mut working = Status::new("working", "Reading selection…");
        working.prompt_name = Some(prompt.name.clone());
        publish(&app, working.clone());
        #[cfg(target_os = "macos")]
        let pulse = tauri::async_runtime::spawn(pulse_tray(app.clone()));
        let outcome = execute_inner(&app, &prompt, &mut working).await;
        // Wait until the pulse has stopped so it cannot overwrite the final icon.
        #[cfg(target_os = "macos")]
        {
            pulse.abort();
            let _ = pulse.await;
        }
        match outcome {
            Ok((text, replaced)) => {
                let mut status = Status::new(
                    if replaced { "success" } else { "review" },
                    if replaced {
                        "Selection updated"
                    } else {
                        "Result ready — selection was not changed"
                    },
                );
                status.prompt_name = Some(prompt.name);
                status.result = Some(text);
                publish(&app, status);
                if !replaced {
                    show(&app);
                }
            }
            Err(failure) => {
                let mut status = Status::new("error", &failure.message);
                status.prompt_name = Some(prompt.name);
                publish(&app, status);
                if failure.quiet {
                    desktop::alert();
                } else if cfg!(not(target_os = "macos")) {
                    // macOS shows the error in the toast instead of opening settings.
                    show(&app);
                }
            }
        }
        app.state::<AppState>().busy.store(false, Ordering::SeqCst);
    });
}

/// A shortcut run that failed. Quiet failures only need another selection or
/// attempt, so they update the tray tooltip without opening the window.
struct Failure {
    message: String,
    quiet: bool,
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self {
            message,
            quiet: false,
        }
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}

impl From<desktop::CaptureError> for Failure {
    fn from(error: desktop::CaptureError) -> Self {
        Self {
            message: error.message,
            quiet: error.retry,
        }
    }
}

async fn execute_inner(
    app: &tauri::AppHandle,
    prompt: &Prompt,
    status: &mut Status,
) -> Result<(String, bool), Failure> {
    let key = read_key()?;
    let app_capture = app.clone();
    let selection = tauri::async_runtime::spawn_blocking(move || {
        desktop::capture_selection(
            app_capture
                .get_webview_window("main")
                .and_then(|w| w.is_focused().ok())
                .unwrap_or(false),
        )
    })
    .await
    .map_err(|_| "Could not read selection.")??;
    status.message = format!("Running {}…", prompt.name);
    publish(app, status.clone());
    let model = app
        .state::<AppState>()
        .config
        .lock()
        .map_err(|_| "Settings unavailable")?
        .default_model
        .clone();
    let client = app.state::<AppState>().client.clone();
    let mut request = Box::pin(openrouter::complete(
        &client,
        &key,
        prompt,
        &model,
        &selection.text,
    ));
    let mut still_same_target = true;
    let text = loop {
        tokio::select! {
            result = &mut request => break result?,
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                if !desktop::target_unchanged(&selection.target) { still_same_target = false; }
            }
        }
    };
    drop(request);
    if prompt.output == "review" || !still_same_target {
        return Ok((text, false));
    }
    let output = text.clone();
    let replaced = tauri::async_runtime::spawn_blocking(move || {
        desktop::replace_selection(&selection, &output)
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or(false);
    Ok((text, replaced))
}

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--background"])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<AppState>();
                    if state.recording.load(Ordering::SeqCst) {
                        return;
                    }
                    let config = match state.config.lock() {
                        Ok(config) => config.clone(),
                        Err(_) => return,
                    };
                    if let Some(prompt) = config.prompts.iter().find(|p| {
                        p.enabled && p.shortcut.parse::<Shortcut>().ok().as_ref() == Some(shortcut)
                    }) {
                        execute(app.clone(), prompt.clone());
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            save_config,
            set_api_key,
            delete_api_key,
            check_api_key,
            set_recording,
            request_accessibility,
            copy_result,
            test_prompt
        ])
        .setup(|app| {
            let (config, warning) = config::load(app.handle());
            let status = warning
                .map(|e| Status::new("error", &e))
                .unwrap_or_else(|| Status::new("idle", "Ready when you are"));
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(45))
                .build()?;
            app.manage(AppState {
                config: Mutex::new(config.clone()),
                status: Mutex::new(status),
                shortcut_errors: Mutex::new(vec![]),
                busy: AtomicBool::new(false),
                recording: AtomicBool::new(false),
                published: AtomicU64::new(0),
                client,
            });
            let errors = register_shortcuts(app.handle(), &config);
            *app.state::<AppState>().shortcut_errors.lock().unwrap() = errors;
            use tauri::menu::{Menu, MenuItem};
            let settings = MenuItem::with_id(app, "settings", "Open KeyDock", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit KeyDock", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &quit])?;
            tauri::tray::TrayIconBuilder::with_id("main")
                .icon(tauri::image::Image::from_bytes(TRAY_ICON)?)
                .icon_as_template(true)
                .tooltip("KeyDock — Ready")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => show(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            #[cfg(target_os = "macos")]
            tauri::WebviewWindowBuilder::new(
                app,
                "toast",
                tauri::WebviewUrl::App("toast.html".into()),
            )
            .title("KeyDock")
            .inner_size(520.0, 132.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(false)
            .focusable(false)
            .visible(false)
            .build()?
            .set_ignore_cursor_events(true)?;
            if std::env::args().any(|a| a == "--background") {
                set_dock_icon(app.handle(), false);
            } else {
                show(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                set_dock_icon(window.app_handle(), false);
                let state = window.state::<AppState>();
                if state.recording.swap(false, Ordering::SeqCst) {
                    if let Ok(config) = state.config.lock() {
                        let errors = register_shortcuts(window.app_handle(), &config);
                        if let Ok(mut current) = state.shortcut_errors.lock() {
                            *current = errors;
                        }
                    }
                }
            }
        });
    builder
        .run(tauri::generate_context!())
        .expect("Could not start KeyDock");
}
