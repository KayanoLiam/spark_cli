use std::{fs, path::Path, path::PathBuf};

use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

const APP_DIR_NAME: &str = ".spark_cli";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub provider: String,
    pub api_key: Option<String>,
    /// Preferred model for the active provider
    pub model: Option<String>,
    /// Base URL for OpenAI-compatible providers (DeepSeek/Qwen/OpenAI proxy)
    pub base_url: Option<String>,
    /// Automatically extract and write code blocks from responses
    pub auto_code_write: bool,
    /// Default directory for auto-written code (relative to project root)
    pub output_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "openrouter".to_string(),
            api_key: None,
            model: Some("openrouter/auto".to_string()),
            base_url: None,
            auto_code_write: true,
            output_dir: Some("generated".to_string()),
        }
    }
}

impl Settings {
    pub fn load_with(project_root: Option<&Path>, explicit: Option<&Path>) -> Result<Self> {
        let path = resolve_config_path(project_root, explicit)?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let value: Self = toml::from_str(&content)
            .with_context(|| format!("Invalid config TOML at {}", path.display()))?;
        Ok(value)
    }

    pub fn save_with(&self, project_root: Option<&Path>, explicit: Option<&Path>) -> Result<()> {
        let (dir, path) = resolve_config_dir_and_file(project_root, explicit)?;
        if !dir.exists() {
            fs::create_dir_all(&dir).with_context(|| format!(
                "Failed to create config directory at {}",
                dir.display()
            ))?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config at {}", path.display()))?;
        Ok(())
    }

    pub fn load() -> Result<Self> { Self::load_with(None, None) }
    pub fn save(&self) -> Result<()> { self.save_with(None, None) }

    pub fn init(force: bool) -> Result<()> {
        let path = config_file_path()?;
        if path.exists() && !force {
            anyhow::bail!("Config already exists at {} (use --force to overwrite)", path.display());
        }
        let default = Self::default();
        default.save()
    }

    pub fn init_scoped(force: bool, project_root: Option<&Path>) -> Result<()> {
        let (dir, file) = if let Some(root) = project_root {
            (root.to_path_buf(), root.join(CONFIG_FILE_NAME))
        } else {
            (config_dir_path()?, config_file_path()?)
        };
        if file.exists() && !force {
            anyhow::bail!("Config already exists at {} (use --force to overwrite)", file.display());
        }
        if !dir.exists() { fs::create_dir_all(&dir)?; }
        let default = Self::default();
        let content = toml::to_string_pretty(&default)?;
        fs::write(&file, content)?;
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

fn resolve_config_path(project_root: Option<&Path>, explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit { return Ok(p.to_path_buf()); }
    if let Some(root) = project_root { return Ok(root.join(CONFIG_FILE_NAME)); }
    config_file_path()
}

fn resolve_config_dir_and_file(project_root: Option<&Path>, explicit: Option<&Path>) -> Result<(PathBuf, PathBuf)> {
    if let Some(p) = explicit { 
        let dir = p.parent().unwrap_or_else(|| Path::new("."));
        return Ok((dir.to_path_buf(), p.to_path_buf())); 
    }
    if let Some(root) = project_root { 
        return Ok((root.to_path_buf(), root.join(CONFIG_FILE_NAME))); 
    }
    let dir = config_dir_path()?;
    Ok((dir.clone(), dir.join(CONFIG_FILE_NAME)))
}
