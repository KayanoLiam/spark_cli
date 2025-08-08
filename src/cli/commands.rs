use anyhow::{anyhow, Result};
use console::style;

use crate::api::models::ChatMessage;
use crate::api::openrouter::chat_complete as or_chat;
use crate::config::settings::Settings;

pub async fn handle_interactive(_settings: &Settings) -> Result<()> {
    println!("{}", style("[interactive] not implemented yet").yellow());
    Ok(())
}

pub async fn handle_chat(settings: &Settings, prompt: Option<String>) -> Result<()> {
    let prompt = match prompt {
        Some(p) if !p.trim().is_empty() => p,
        _ => return Err(anyhow!("Prompt is empty. Provide text or use interactive/chat mode.")),
    };

    // Resolve API key: config first, then env OPENROUTER_API_KEY
    let api_key = settings
        .api_key
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .ok_or_else(|| anyhow!("API key is not set. Use `config set api-key ...` or set env OPENROUTER_API_KEY"))?;
    let api_key = crate::utils::secrets::normalize_api_key(&api_key);

    // For now we default to OpenRouter if user says they only have it
    if settings.provider.to_lowercase() == "openrouter" || settings.provider.is_empty() {
        let messages = vec![ChatMessage { role: "user".to_string(), content: prompt }];
        let client = reqwest::Client::new();
        let content = or_chat(&client, &api_key, messages, None).await?;
        println!("{}", content);
        return Ok(());
    }

    println!("{}", style("Selected provider not supported yet; falling back to OpenRouter").yellow());
    let messages = vec![ChatMessage { role: "user".to_string(), content: prompt }];
    let client = reqwest::Client::new();
    let content = or_chat(&client, &api_key, messages, None).await?;
    println!("{}", content);
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
