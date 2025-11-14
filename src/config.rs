use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub start_time_default: String,
    pub end_time_default: String,
    pub db_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().expect("Failed to get home directory");
        let default_db = home
            .join(".local")
            .join("share")
            .join("slog")
            .join("slog.db")
            .to_string_lossy()
            .to_string();

        Config {
            start_time_default: "21:30:00".to_string(),
            end_time_default: "05:30:00".to_string(),
            db_file_path: default_db,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().expect("Failed to get home directory");
        let config_dir = home.join(".config").join("slog");

        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
        config_dir.join("config.yml")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            Ok(default_config)
        } else {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = serde_yaml::from_str(&contents)?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        let yaml = serde_yaml::to_string(self)?;

        let mut file = fs::File::create(&config_path)?;
        file.write_all(yaml.as_bytes())?;

        println!("Config saved to: {}", config_path.display());
        Ok(())
    }

    pub fn update_field(&mut self, field: &str, value: String) -> Result<(), String> {
        match field {
            "start_time_default" => {
                self.start_time_default = value;
                Ok(())
            }
            "end_time_default" => {
                self.end_time_default = value;
                Ok(())
            }
            "db_file_path" => {
                self.db_file_path = value;
                Ok(())
            }
            _ => Err(format!("Unknown field: {}", field)),
        }
    }

    pub fn get_db_url(&self) -> String {
        format!("sqlite:{}", self.db_file_path)
    }
}
