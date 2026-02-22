use std::sync::Mutex;
use tauri::{AppHandle, State, Emitter, Manager};
use asterisk_manager::ManagerOptions;
use crate::asterisk::register_client;
use crate::config::{AppConfig, ConfigManager, AmiOperator};


/// Command for retrieving current app config state.
///
/// Used in settings panel for showing ll configuration.
///
#[tauri::command]
pub async fn get_current_config(state: State<'_, Mutex<AppConfig>>) -> Result<String, String> {
    let config = state.lock().unwrap();
    let config = config.clone(); // Cause we cant Serialize MutexGuard
    Ok(serde_json::to_string(&config).unwrap())
}


/// Command for saving config from settings panel.
#[tauri::command]
pub async fn save_config(new_config: AppConfig, app: AppHandle, state: State<'_, Mutex<AppConfig>>) -> Result<(), String> {
    let mut config = state.lock().unwrap();
    *config = new_config;
    ConfigManager::new().save(&config).unwrap();
    // Need to crete a string 'cause there aa parse function on frontend
    let engineer = serde_json::to_string(&config.engineers[config.current_duty_index]).unwrap();
    app.emit("duty-changed", &engineer).unwrap();
    Ok(())
}


/// Command for saving only asterisk configs
#[tauri::command]
pub async fn save_asterisk_config(ami_operator: Option<AmiOperator>,
                                  manager_options: Option<ManagerOptions>,
                                  app: AppHandle,
                                  state: State<'_, Mutex<AppConfig>>
) -> Result<(), String> {
    let tmp_config;
    // New scope due to tauri work with asynchronous and multithread code
    {
        let mut config = state.lock().unwrap();
        config.ami_manager = manager_options;
        config.ami_operator = ami_operator;
        ConfigManager::new().save(&config).unwrap();
        tmp_config = config.clone();
    }
    // This code is copying from app setup
    let manager = register_client(&app, &tmp_config).await;
    if let Some(manager) = manager {
        app.manage(manager);
    }
    Ok(())
}

