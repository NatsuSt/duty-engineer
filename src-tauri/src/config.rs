use chrono::{Local, NaiveDate};
use asterisk_manager::ManagerOptions;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::models::Engineer;


/// Represents the persistent application configuration and state.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub last_rotation: NaiveDate,
    pub current_duty_index: usize,
    pub engineers: Vec<Engineer>,
    pub ami_manager: Option<ManagerOptions>,
    pub ami_operator: Option<AmiOperator>
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_rotation: Local::now().date_naive(),
            current_duty_index: 0,
            engineers: vec![Engineer::default()],
            ami_manager: None,
            ami_operator: None
        }
    }
}


/// Manager struct, that load settings file
#[derive(Debug)]
pub struct ConfigManager {
    path: PathBuf,
}


impl ConfigManager {
    /// Create a new config manager with default path
    pub fn new() -> Self {
        let proj_dir = ProjectDirs::from("com", "Serhii", "DutyEngineers")
            .expect("Не удалось определить системную директорию");
        let config_dir = proj_dir.config_dir();
        fs::create_dir_all(config_dir).expect("Не получилось создать директорию с настройками");
        println!("Filepath: {:#?}", config_dir);

        Self {
            path: PathBuf::from(config_dir.join("config.toml")),
        }
    }

    /// Load settings from file into program, or create default
    pub fn load(&self) -> Result<AppConfig> {
        if !self.path.exists() {
            let conf = AppConfig::default();
            self.save(&conf).unwrap();
            return Ok(conf);
        }
        let conf = fs::read_to_string(&self.path).expect("Не опознання ошибка при чтении файла");
        Ok(toml::from_str::<AppConfig>(&conf)?)
    }

    /// Save current settings into file
    pub fn save(&self, config: &AppConfig) -> Result<()> {
        let conf = toml::to_string(config)?;
        fs::write(&self.path, conf).expect("Не возможно записать файл конфигурации");
        Ok(())
    }
}

/// Structure that store context and phone_number of operator to make a call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmiOperator {
    pub operator_number: u32,
    pub context: String
}
