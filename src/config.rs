use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub start_time_default: String,
    pub end_time_default: String,
    pub db_file_path: String,
    pub google_sheets_id: Option<String>,
    pub google_credentials_path: Option<String>,
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
            start_time_default: "21:30".to_string(),
            end_time_default: "05:30".to_string(),
            db_file_path: default_db,
            google_sheets_id: None,
            google_credentials_path: None,
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
                Self::validate_time_format(&value)?;
                self.start_time_default = value;
                Ok(())
            }
            "end_time_default" => {
                Self::validate_time_format(&value)?;
                self.end_time_default = value;
                Ok(())
            }
            "db_file_path" => {
                self.db_file_path = value;
                Ok(())
            }
            "google_sheets_id" => {
                self.google_sheets_id = Some(value);
                Ok(())
            }
            "google_credentials_path" => {
                self.google_credentials_path = Some(value);
                Ok(())
            }
            _ => Err(format!("Unknown field: {}", field)),
        }
    }

    pub fn get_db_url(&self) -> String {
        format!("sqlite:{}", self.db_file_path)
    }

    fn validate_time_format(input: &str) -> Result<(), String> {
        let parts: Vec<&str> = input.split(':').collect();

        if parts.len() != 2 {
            return Err("Time must be in HH:MM format".to_string());
        }

        let hours = parts[0]
            .parse::<u32>()
            .map_err(|_| "Hours must be a valid number".to_string())?;
        let minutes = parts[1]
            .parse::<u32>()
            .map_err(|_| "Minutes must be a valid number".to_string())?;

        if hours > 23 {
            return Err("Hours must be between 0 and 23".to_string());
        }

        if minutes > 59 {
            return Err("Minutes must be between 0 and 59".to_string());
        }

        Ok(())
    }
}
