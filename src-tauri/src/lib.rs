use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Window, App,
            Emitter, Error, LogicalSize,
            Manager, State, WebviewWindow, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::config::{AppConfig, ConfigManager};
use crate::settings::{get_current_config, save_config, save_asterisk_config};
use crate::asterisk::{make_call, register_client};

use asterisk_manager::Manager as AsManager;

use log::{debug, error, info};
use anyhow::Result;

/// Include all models from project
mod config;
mod models;
mod asterisk;
mod settings;


/// Command to change size of windows
#[tauri::command]
async fn resize_window(webview_window: WebviewWindow, width: f64, height: f64) -> tauri::Result<()> {
    webview_window.set_size(LogicalSize::new(width, height))
}


/// Command to retrieve engineer current selected engineer
#[tauri::command]
async fn retrieve_current_engineer(app: AppHandle, state: State<'_, Mutex<AppConfig>>) -> tauri::Result<String> {
    let config = state.lock().unwrap();
    if config.current_duty_index >= config.engineers.len() {
        let _ = app.dialog()
            .message("Індекс вийшов за межі у файлі, переведіть його у нульове положення")
            .title("Помилка при зчитуванні конфігурації")
            .kind(MessageDialogKind::Error)
            .blocking_show();
        app.exit(1);
    }
    let engineer = config.engineers[config.current_duty_index].clone();
    Ok(serde_json::to_string(&engineer).unwrap())
}


/// Initiates a call to the engineer on duty using Asterisk Manager Interface (AMI).
/// 
/// If AMI is not configured or operator settings are missing, it displays
/// an information dialog to the user and returns early.
#[tauri::command]
async fn call_engineer(app: AppHandle) -> Result<(), String> {
    // Retrieve manager from app state 
    debug!("Before retrieving manager in ");
    let manager = app.try_state::<AsManager>();
    let manager = match manager {
        Some(manager) => manager,
        None => {
            // Show a dialog message that AMI is not configured
            app.dialog()
                .message("Спочатку нашатуйте Asterisk, щоб виконувати двінок")
                .kind(MessageDialogKind::Info)
                .title("Налаштуйте Asterisk")
                .show(|_| {});
            return Ok(())
        }
    };
    debug!("After retrieving manager. Starting retrieving AppConfig");

    // Scope to ensure the MutexGuard is dropped before the .await point.
    // This prevents holding the lock during an async call, avoiding potential deadlocks.
    let (phone_number, ami_operator) = {
        let config = app.state::<Mutex<AppConfig>>();
        let config = config.lock().unwrap();
        
        let phone = config.engineers[config.current_duty_index].phone_number.clone();
        let operator = &config.ami_operator;
        // Check configuration and show dialog when settings are unset
        let operator = match operator {
            Some(operator) => operator.clone(),
            None => {
                    app.dialog()
                    .message("Відсутні налаштування телефону оператору, для виконання дзівнків, налаштуйте ці поля в файлі конфігурації")
                    .kind(MessageDialogKind::Info)
                    .title("Налаштуйте телефон оператора")
                    .show(|_| {});
                return Ok(())
            }
        };
        
        (phone, operator) // The lock is released here when exiting {}
    };
    // Make a call via AMI interface
    make_call(&manager, &phone_number, &ami_operator).await.unwrap();
    Ok(())
}


/// Set up all menu options, that displays in task bar
/// 
/// This method set up handlers for quit amd change duty engineer action
fn create_tray_menu(app: &mut App) -> Result<(), Error> {
    let settings = MenuItem::with_id(app, "settings", "Налаштування", true, None::<&str>)?;
    let change_engineer = MenuItem::with_id(app, "change", "Змінити чергового", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?; // Add separate line
    let quit_i = MenuItem::with_id(app, "quit", "Вихід", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &change_engineer, &separator, &quit_i])?;

    let _ = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                info!("quit menu item was clicked");
                app.exit(0);
            },
            "settings" => {
                check_config(app);
                let windows = app.webview_windows();
                if windows.contains_key("settings") {
                    return 
                }
                let window = tauri::
                WebviewWindowBuilder::new(
                    app,
                    "settings",
                    tauri::WebviewUrl::App("settings/index.html".into())
                )
                    .build().unwrap();
                window.set_title("Налаштування").unwrap();
                // window.set_size(LogicalSize::new(343, 485)).unwrap();
                window.set_decorations(false).unwrap();
                // window.set_min_size(Some(LogicalSize::new(343, 485))).unwrap();
            },
            "change" => {
                // retrieve app state
                let config = app.state::<Mutex<AppConfig>>();
                let mut config = config.lock().unwrap();

                let mut current_index = config.current_duty_index;
                let max_index = config.engineers.len(); // Total engineers
                // Return, if we only have one engineer
                if max_index == 1 { return  }

                // Increase index
                current_index += 1;
                if current_index > max_index - 1 {
                    current_index = 0;
                }

                // Apply changes to app state and save into file
                config.current_duty_index = current_index;
                let engineer = config.engineers[current_index].clone();
                let manager = ConfigManager::new();
                if let Err(e) = manager.save(&config) {
                    error!("Error saving: {}", e);
                }

                // Drop lock before sending  data
                drop(config);

                // Serialize data before sending to frontend
                let current_engineer = serde_json::to_string(&engineer).unwrap();
                let _ = app.emit("duty-changed", current_engineer).unwrap();
                info!("new engineer: {}", current_index);
            },
            _ => {
                info!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;
    Ok(())
}


/// Reads configuration data from the app's roaming directory.
/// 
/// If a syntax error occurs, displays an error dialog and terminates the application.
fn check_config(app: &AppHandle) -> config::AppConfig {
    let config_result = config::ConfigManager::new().load();
    if let Err(err) = &config_result {
        let _ = app.dialog()
            .message(format!("Ошибка в файле конфигурации:\n{}", err))
            .kind(MessageDialogKind::Error)
            .title("Ошибка")
            .blocking_show();
        app.exit(0);
    };
    config_result.unwrap()
}


/// Delegated function for setup process of program
fn app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration file and parse them
    let config = check_config(app.handle());

    // Create app state variables
    let manager = tauri::async_runtime::block_on(async {
        register_client(&app.handle(), &config).await
    });
    if let Some(manager) = manager {
        app.manage(manager);
    }
    // Load config to app state
    app.manage(Mutex::new(config));

    // Create menu in task bar
    create_tray_menu(app)?;
    Ok(())
}


/// React on window event
///
/// Now is using for debug purpose
fn window_event_handler(window: &Window, event: &WindowEvent) {
    // THIS IS USED FOR DEBUG PURPOSE
    // Listening all events
    if let WindowEvent::Resized(physical_size) = event {
        // Get scale factor for window and transform into pixel size
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let logical_size = physical_size.to_logical::<f64>(scale_factor);

        // Show message in terminal
        debug!(
            "Window resized! Physical: {:?}, Logical: {}x{}",
            physical_size, logical_size.width, logical_size.height
        );
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(app_setup)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            resize_window, 
            retrieve_current_engineer,
            call_engineer,
            get_current_config,
            save_config,
            save_asterisk_config,
        ])
        .on_window_event(window_event_handler).run(tauri::generate_context!())
        .expect("error while running tauri application");
}
