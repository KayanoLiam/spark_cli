use anyhow::Result;
use std::fs;

pub fn read_to_string(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn write_string(path: &str, content: &str) -> Result<()> {
    Ok(fs::write(path, content)?)
}
