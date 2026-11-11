mod config;
mod desktop;
mod openrouter;

use config::{Config, Prompt};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const KEY_SERVICE: &str = "com.ronnyschlidt.promptkey";

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

fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn publish(app: &tauri::AppHandle, status: Status) {
    if let Ok(mut current) = app.state::<AppState>().status.lock() {
        *current = status.clone();
    }
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(format!("PromptKey — {}", status.message)));
        #[cfg(target_os = "macos")]
        let _ = tray.set_title(if status.phase == "working" {
            Some("···")
        } else {
            None
        });
    }
    let _ = app.emit("status", status);
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
        let outcome = execute_inner(&app, &prompt, &mut working).await;
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
            Err(error) => {
                let mut status = Status::new("error", &error);
                status.prompt_name = Some(prompt.name);
                publish(&app, status);
                show(&app);
            }
        }
        app.state::<AppState>().busy.store(false, Ordering::SeqCst);
    });
}

async fn execute_inner(
    app: &tauri::AppHandle,
    prompt: &Prompt,
    status: &mut Status,
) -> Result<(String, bool), String> {
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
                client,
            });
            let errors = register_shortcuts(app.handle(), &config);
            *app.state::<AppState>().shortcut_errors.lock().unwrap() = errors;
            use tauri::menu::{Menu, MenuItem};
            let settings =
                MenuItem::with_id(app, "settings", "Open PromptKey", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit PromptKey", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &quit])?;
            tauri::tray::TrayIconBuilder::with_id("main")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray.png"
                ))?)
                .icon_as_template(true)
                .tooltip("PromptKey — Ready")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => show(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            if !std::env::args().any(|a| a == "--background") {
                show(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
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
        .expect("Could not start PromptKey");
}
