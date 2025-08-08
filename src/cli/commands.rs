use anyhow::{anyhow, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::api::models::ChatMessage;
use crate::api::openrouter::{chat_complete as or_chat, chat_complete_stream as or_chat_stream};
use crate::config::settings::Settings;
use crate::cli::args::{RuntimeArgs, IoArgs};
use crate::session::manager::SessionManager;
use crate::session::history::MessageRecord;

pub async fn handle_interactive(settings: &Settings, runtime: &RuntimeArgs, io: &IoArgs) -> Result<()> {
    use dialoguer::Input;
    println!("{}", style("Interactive mode. Ctrl+C to exit.").cyan());
    loop {
        let line: String = Input::new().with_prompt("You").interact_text()?;
        if line.trim().is_empty() { continue; }
        handle_chat(settings, Some(line), runtime, io).await?;
    }
}

pub async fn handle_chat(settings: &Settings, prompt: Option<String>, runtime: &RuntimeArgs, io: &IoArgs) -> Result<()> {
    // Prefer file input if provided
    let prompt = match (&io.input_file, &prompt) {
        (Some(path), _) => crate::utils::io::read_to_string(path)?.trim().to_string(),
        (None, Some(p)) if !p.trim().is_empty() => p.to_string(),
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
    let provider = runtime.provider.as_deref().unwrap_or(&settings.provider);
    let model = runtime.model.as_deref().or(settings.model.as_deref());

    if provider.to_lowercase() == "openrouter" || provider.is_empty() {
        let messages = vec![ChatMessage { role: "user".to_string(), content: prompt.clone() }];
        let client = reqwest::Client::new();
        if runtime.stream {
            let mut buffer = String::new();
            let content = or_chat_stream(&client, &api_key, messages, model, |chunk| {
                print!("{}", chunk);
                let _ = std::io::Write::flush(&mut std::io::stdout());
                buffer.push_str(chunk);
            }).await?;
            // newline after stream
            println!();
            let final_text = if content.is_empty() { buffer } else { content };
            let mgr = SessionManager::new();
            if let Some(sid) = mgr.current_session_id() {
                let now = chrono::Utc::now().timestamp_millis();
                mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt.clone(), timestamp_ms: now })?;
                mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: final_text.clone(), timestamp_ms: now })?;
            }
            if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &final_text)?; }
            return Ok(());
        } else {
            let pb = ProgressBar::new_spinner().with_message("Contacting OpenRouter...");
            pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let content = or_chat(&client, &api_key, messages, model).await?;
            pb.finish_and_clear();
            // append to session if any
            let mgr = SessionManager::new();
            if let Some(sid) = mgr.current_session_id() {
                let now = chrono::Utc::now().timestamp_millis();
                mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt.clone(), timestamp_ms: now })?;
                mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: content.clone(), timestamp_ms: now })?;
            }
            // write to file if requested
            if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
            println!("{}", content);
            return Ok(());
        }
    }

    println!("{}", style("Selected provider not supported yet; falling back to OpenRouter").yellow());
    let messages = vec![ChatMessage { role: "user".to_string(), content: prompt.clone() }];
    let client = reqwest::Client::new();
    let content = or_chat(&client, &api_key, messages, None).await?;
    let mgr = SessionManager::new();
    if let Some(sid) = mgr.current_session_id() {
        let now = chrono::Utc::now().timestamp_millis();
        mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt, timestamp_ms: now })?;
        mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: content.clone(), timestamp_ms: now })?;
    }
    if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
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
    let mgr = SessionManager::new();
    let id = mgr.create_session(name)?;
    mgr.set_current_session_id(&id)?;
    println!("Created session {} -> {}", name, id);
    Ok(())
}

pub async fn handle_session_list(_settings: &Settings) -> Result<()> {
    let mgr = SessionManager::new();
    let list = mgr.list_sessions()?;
    let current = mgr.current_session_id();
    for meta in list {
        let mark = if current.as_deref() == Some(&meta.id) { "*" } else { " " };
        println!("{} {} - {}", mark, meta.id, meta.name);
    }
    Ok(())
}

pub async fn handle_session_load(_settings: &Settings, id: &str) -> Result<()> {
    let mgr = SessionManager::new();
    mgr.set_current_session_id(id)?;
    println!("Switched to session {}", id);
    Ok(())
}

pub async fn handle_session_delete(_settings: &Settings, id: &str) -> Result<()> {
    let mgr = SessionManager::new();
    mgr.delete_session(id)?;
    println!("Deleted session {}", id);
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
