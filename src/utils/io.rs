use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn read_to_string(path: &str) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))
}

pub fn write_string(path: &str, content: &str) -> Result<()> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
        }
    }
    fs::write(p, content).with_context(|| format!("Failed to write file: {}", p.display()))
}
