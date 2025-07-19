//! Application configuration

use serde::{Deserialize, Serialize};
use std::path::{PathBuf};
use std::fs;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub refresh_rate: u64,
    pub show_cpu: bool,
    pub show_memory: bool,
    pub show_gpu: bool,
    pub show_network: bool,
    pub selected_network_interface: Option<String>,

    #[serde(skip)]
    config_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let config_path = Self::default_config_path();

        Self {
            refresh_rate: 1000,
            show_cpu: true,
            show_memory: true,
            show_gpu: true,
            show_network: true,
            selected_network_interface: None,
            config_path,
        }
    }
}

impl Config {
    fn default_config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "yourname", "rkhtop") {
            proj_dirs.config_dir().join("config.toml")
        } else {
            // fallback to current directory if none found
            PathBuf::from("rkhtop_config.toml")
        }
    }

    pub fn load() -> Result<Self, std::io::Error> {
        let default = Self::default();

        if let Some(parent) = default.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config = match fs::read_to_string(&default.config_path) {
            Ok(contents) => {
                let mut loaded: Self = toml::from_str(&contents).unwrap_or(default.clone());
                loaded.config_path = default.config_path.clone(); // retain path
                loaded
            },
            Err(_) => default,
        };

        Ok(config)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(self).unwrap();
        fs::write(&self.config_path, toml)
    }
}