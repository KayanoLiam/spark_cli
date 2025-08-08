use anyhow::Result;
use console::style;

use crate::config::settings::Settings;

pub async fn handle_interactive(_settings: &Settings) -> Result<()> {
    println!("{}", style("[interactive] not implemented yet").yellow());
    Ok(())
}

pub async fn handle_chat(_settings: &Settings, _prompt: Option<String>) -> Result<()> {
    println!("{}", style("[chat] not implemented yet").yellow());
    Ok(())
}

pub async fn handle_config_list(settings: &Settings) -> Result<()> {
    println!("Current provider: {}", settings.provider);
    println!("API key set: {}", settings.api_key.is_some());
    Ok(())
}

pub async fn handle_config_set(settings: &mut Settings, key: &str, value: &str) -> Result<()> {
    match key {
        "api-key" | "api_key" => settings.api_key = Some(value.to_owned()),
        "provider" => settings.provider = value.to_owned(),
        _ => println!("Unknown config key: {}", key),
    }
    settings.save()?;
    Ok(())
}

pub async fn handle_session_new(_settings: &Settings, name: &str) -> Result<()> {
    println!("Create session: {} (not implemented)", name);
    Ok(())
}

pub async fn handle_session_list(_settings: &Settings) -> Result<()> {
    println!("List sessions (not implemented)");
    Ok(())
}

pub async fn handle_session_load(_settings: &Settings, id: &str) -> Result<()> {
    println!("Load session {} (not implemented)", id);
    Ok(())
}

pub async fn handle_session_delete(_settings: &Settings, id: &str) -> Result<()> {
    println!("Delete session {} (not implemented)", id);
    Ok(())
}

pub async fn handle_code_generate(_settings: &Settings, lang: &str, kind: &str) -> Result<()> {
    println!("Generate code: lang={}, type={} (not implemented)", lang, kind);
    Ok(())
}

pub async fn handle_code_review(_settings: &Settings, file: &str) -> Result<()> {
    println!("Code review for {} (not implemented)", file);
    Ok(())
}

pub async fn handle_code_optimize(_settings: &Settings, file: &str) -> Result<()> {
    println!("Optimize code for {} (not implemented)", file);
    Ok(())
}
