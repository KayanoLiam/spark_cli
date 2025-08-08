use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

const APP_DIR_NAME: &str = ".spark_cli";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub provider: String,
    pub api_key: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: None,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = config_file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let value: Self = toml::from_str(&content)
            .with_context(|| format!("Invalid config TOML at {}", path.display()))?;
        Ok(value)
    }

    pub fn save(&self) -> Result<()> {
        let dir = config_dir_path()?;
        if !dir.exists() {
            fs::create_dir_all(&dir).with_context(|| format!(
                "Failed to create config directory at {}",
                dir.display()
            ))?;
        }
        let path = dir.join(CONFIG_FILE_NAME);
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config at {}", path.display()))?;
        Ok(())
    }
}

fn config_dir_path() -> Result<PathBuf> {
    let home = home_dir().context("Cannot resolve home directory")?;
    Ok(home.join(APP_DIR_NAME))
}

fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join(CONFIG_FILE_NAME))
}
