use anyhow::{anyhow, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::api::models::ChatMessage;
use crate::api::openrouter::{chat_complete as or_chat, chat_complete_stream as or_chat_stream};
use crate::api::openai_compat::{chat_complete as oa_chat, chat_complete_stream as oa_chat_stream};
use reqwest::Client;
use crate::config::settings::Settings;
use crate::cli::args::{RuntimeArgs, IoArgs};
use crate::session::manager::SessionManager;
use crate::session::history::MessageRecord;
use crate::utils::code::{extract_code_blocks, choose_best_block, guess_ext_from_lang};

fn auto_write_code(text: &str, settings: &Settings, lang_hint: Option<&str>) -> Result<()> {
    let blocks = extract_code_blocks(text);
    if blocks.is_empty() { return Ok(()); }
    let dir = settings.output_dir.as_deref().unwrap_or("generated");
    std::fs::create_dir_all(dir)?;
    if blocks.len() == 1 {
        let b = &blocks[0];
        let filename = b.filename.clone().unwrap_or_else(|| {
            let ext = b.language.as_deref().or(lang_hint).map(guess_ext_from_lang).unwrap_or("txt");
            format!("snippet.{}", ext)
        });
        let path = std::path::Path::new(dir).join(filename);
        std::fs::write(&path, &b.content)?;
        eprintln!("Saved code to {}", path.display());
    } else {
        if let Some(lang) = lang_hint {
            if let Some(b) = choose_best_block(&blocks, &[lang]) {
                let filename = b.filename.clone().unwrap_or_else(|| {
                    let ext = b.language.as_deref().or(Some(lang)).map(guess_ext_from_lang).unwrap_or("txt");
                    format!("snippet.{}", ext)
                });
                let path = std::path::Path::new(dir).join(filename);
                std::fs::write(&path, &b.content)?;
                eprintln!("Saved code to {}", path.display());
                return Ok(());
            }
        }
        for (idx, b) in blocks.iter().enumerate() {
            let filename = b.filename.clone().unwrap_or_else(|| {
                let ext = b.language.as_deref().map(guess_ext_from_lang).unwrap_or("txt");
                format!("snippet_{}.{}", idx + 1, ext)
            });
            let path = std::path::Path::new(dir).join(filename);
            std::fs::write(&path, &b.content)?;
        }
        eprintln!("Saved {} code blocks to {}", blocks.len(), dir);
    }
    Ok(())
}

pub async fn handle_interactive(settings: &Settings, runtime: &RuntimeArgs, io: &IoArgs, http: &Client) -> Result<()> {
    use dialoguer::Input;
    println!("{}", style("Interactive mode. Ctrl+C to exit.").cyan());
    loop {
        let line: String = Input::new().with_prompt("You").interact_text()?;
        if line.trim().is_empty() { continue; }
        handle_chat(settings, Some(line), runtime, io, http).await?;
    }
}

pub async fn handle_chat(settings: &Settings, prompt: Option<String>, runtime: &RuntimeArgs, io: &IoArgs, http: &Client) -> Result<()> {
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
        let client = http;
        if runtime.stream {
            let mut buffer = String::new();
            let content = or_chat_stream(client, &api_key, messages, model, |chunk| {
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
            else if settings.auto_code_write { auto_write_code(&final_text, settings, runtime.model.as_deref())?; }
            return Ok(());
        } else {
            let pb = ProgressBar::new_spinner().with_message("Contacting OpenRouter...");
            pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let result = or_chat(client, &api_key, messages, model).await;
            pb.finish_and_clear();
            let content = match result {
                Ok(c) => c,
                Err(e) => { eprintln!("{}", style(format!("Request failed: {}", e)).red()); return Err(e); }
            };
            // append to session if any
            let mgr = SessionManager::new();
            if let Some(sid) = mgr.current_session_id() {
                let now = chrono::Utc::now().timestamp_millis();
                mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt.clone(), timestamp_ms: now })?;
                mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: content.clone(), timestamp_ms: now })?;
            }
            // write to file if requested
            if let Some(out) = &io.output_file { if let Err(e) = crate::utils::io::write_string(out, &content) { eprintln!("{}", style(format!("Failed to write output: {}", e)).red()); } }
            else if settings.auto_code_write { auto_write_code(&content, settings, runtime.model.as_deref())?; }
            println!("{}", content);
            return Ok(());
        }
    }

    // OpenAI-compatible providers: deepseek, qwen, openai, custom proxy
    if matches!(provider.to_lowercase().as_str(), "deepseek" | "qwen" | "openai" | "openai-compatible") {
        let base = settings.base_url.as_deref().ok_or_else(|| anyhow!("Missing base_url in config for OpenAI-compatible provider"))?;
        let messages = vec![ChatMessage { role: "user".to_string(), content: prompt.clone() }];
        if runtime.stream {
            let mut buffer = String::new();
            let content = oa_chat_stream(http, base, &api_key, messages, model, |chunk| { print!("{}", chunk); let _ = std::io::Write::flush(&mut std::io::stdout()); buffer.push_str(chunk); }).await?;
            println!();
            let final_text = if content.is_empty() { buffer } else { content };
            let mgr = SessionManager::new();
            if let Some(sid) = mgr.current_session_id() {
                let now = chrono::Utc::now().timestamp_millis();
                mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt.clone(), timestamp_ms: now })?;
                mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: final_text.clone(), timestamp_ms: now })?;
            }
            if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &final_text)?; }
            else if settings.auto_code_write { auto_write_code(&final_text, settings, runtime.model.as_deref())?; }
            return Ok(());
        } else {
            let pb = ProgressBar::new_spinner().with_message("Contacting provider...");
            pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let content = oa_chat(http, base, &api_key, messages, model).await?;
            pb.finish_and_clear();
            let mgr = SessionManager::new();
            if let Some(sid) = mgr.current_session_id() {
                let now = chrono::Utc::now().timestamp_millis();
                mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt.clone(), timestamp_ms: now })?;
                mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: content.clone(), timestamp_ms: now })?;
            }
            if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
            else if settings.auto_code_write { auto_write_code(&content, settings, runtime.model.as_deref())?; }
            println!("{}", content);
            return Ok(());
        }
    }

    eprintln!("{}", style("Selected provider not supported yet; falling back to OpenRouter").yellow());
    let messages = vec![ChatMessage { role: "user".to_string(), content: prompt.clone() }];
    let client = http;
    let content = or_chat(client, &api_key, messages, None).await?;
    let mgr = SessionManager::new();
    if let Some(sid) = mgr.current_session_id() {
        let now = chrono::Utc::now().timestamp_millis();
        mgr.append_message(&sid, &MessageRecord { role: "user".into(), content: prompt, timestamp_ms: now })?;
        mgr.append_message(&sid, &MessageRecord { role: "assistant".into(), content: content.clone(), timestamp_ms: now })?;
    }
    if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
    else if settings.auto_code_write { auto_write_code(&content, settings, runtime.model.as_deref())?; }
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

pub async fn handle_code_generate(settings: &Settings, lang: &str, kind: &str, runtime: &RuntimeArgs, io: &IoArgs, http: &Client, code_only: bool, out_dir: &Option<String>) -> Result<()> {
    let api_key = settings
        .api_key
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .ok_or_else(|| anyhow!("API key is not set. Use `config set api-key ...` or set env OPENROUTER_API_KEY"))?;
    let api_key = crate::utils::secrets::normalize_api_key(&api_key);

    let provider = runtime.provider.as_deref().unwrap_or(&settings.provider);
    let model = runtime.model.as_deref().or(settings.model.as_deref());

    let system = format!(
        "You are a senior software engineer. Create a minimal, runnable example in {} for a {}. Include clear comments and dependency instructions. If multiple files are required, consolidate into a single-file presentation.",
        lang, kind
    );
    let messages = vec![
        ChatMessage { role: "system".into(), content: system },
        ChatMessage { role: "user".into(), content: "Provide the implementation and a brief usage guide.".into() },
    ];

    let pb = ProgressBar::new_spinner().with_message("Generating code...");
    pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let content = if matches!(provider.to_lowercase().as_str(), "deepseek" | "qwen" | "openai" | "openai-compatible") {
        let base = settings.base_url.as_deref().ok_or_else(|| anyhow!("Missing base_url in config for OpenAI-compatible provider"))?;
        oa_chat(http, base, &api_key, messages, model).await?
    } else {
        or_chat(http, &api_key, messages, model).await?
    };
    pb.finish_and_clear();

    // Post-process content
    if code_only || out_dir.is_some() {
        use crate::utils::code::{extract_code_blocks, choose_best_block, guess_ext_from_lang};
        let blocks = extract_code_blocks(&content);
        if let Some(dir) = out_dir {
            // write each block into dir, filename or fallback
            std::fs::create_dir_all(dir)?;
            for (idx, b) in blocks.iter().enumerate() {
                let filename = b.filename.clone().unwrap_or_else(|| {
                    let ext = b.language.as_deref().map(guess_ext_from_lang).unwrap_or("txt");
                    format!("snippet_{}.{}", idx + 1, ext)
                });
                let path = std::path::Path::new(dir).join(filename);
                std::fs::write(&path, &b.content)?;
            }
            println!("Wrote {} code blocks to {}", blocks.len(), dir);
            return Ok(());
        } else {
            // choose best by language
            let preferred = [lang];
            if let Some(b) = choose_best_block(&blocks, &preferred) {
                if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &b.content)?; } else { println!("{}", b.content); }
                return Ok(());
            }
        }
    }

    if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
    println!("{}", content);
    Ok(())
}

pub async fn handle_code_review(settings: &Settings, file: &str, runtime: &RuntimeArgs, io: &IoArgs, http: &Client) -> Result<()> {
    let api_key = settings
        .api_key
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .ok_or_else(|| anyhow!("API key is not set. Use `config set api-key ...` or set env OPENROUTER_API_KEY"))?;
    let api_key = crate::utils::secrets::normalize_api_key(&api_key);

    let provider = runtime.provider.as_deref().unwrap_or(&settings.provider);
    let model = runtime.model.as_deref().or(settings.model.as_deref());
    let code = crate::utils::io::read_to_string(file)?;
    let messages = vec![
        ChatMessage { role: "system".into(), content: "You are a rigorous and friendly code reviewer. Identify issues, risks, and improvements, and provide refactoring examples when necessary.".into() },
        ChatMessage { role: "user".into(), content: format!("Please review the following file {}:\n\n```\n{}\n```", file, code) },
    ];
    let content = if runtime.stream {
        if matches!(provider.to_lowercase().as_str(), "deepseek" | "qwen" | "openai" | "openai-compatible") {
            let base = settings.base_url.as_deref().ok_or_else(|| anyhow!("Missing base_url in config for OpenAI-compatible provider"))?;
            oa_chat_stream(http, base, &api_key, messages, model, |chunk| { print!("{}", chunk); let _ = std::io::Write::flush(&mut std::io::stdout()); }).await?
        } else {
            or_chat_stream(http, &api_key, messages, model, |chunk| { print!("{}", chunk); let _ = std::io::Write::flush(&mut std::io::stdout()); }).await?
        }
    } else {
        let pb = ProgressBar::new_spinner().with_message("Reviewing...");
        pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        let r = if matches!(provider.to_lowercase().as_str(), "deepseek" | "qwen" | "openai" | "openai-compatible") {
            let base = settings.base_url.as_deref().ok_or_else(|| anyhow!("Missing base_url in config for OpenAI-compatible provider"))?;
            oa_chat(http, base, &api_key, messages, model).await
        } else {
            or_chat(http, &api_key, messages, model).await
        };
        pb.finish_and_clear();
        match r {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{}", style(format!("Review failed: {}", e)).red());
                return Err(e);
            }
        }
    };
    if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
    if !runtime.stream { println!("{}", content); }
    Ok(())
}

pub async fn handle_code_optimize(settings: &Settings, file: &str, runtime: &RuntimeArgs, io: &IoArgs, http: &Client) -> Result<()> {
    let api_key = settings
        .api_key
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .ok_or_else(|| anyhow!("API key is not set. Use `config set api-key ...` or set env OPENROUTER_API_KEY"))?;
    let api_key = crate::utils::secrets::normalize_api_key(&api_key);

    let provider = runtime.provider.as_deref().unwrap_or(&settings.provider);
    let model = runtime.model.as_deref().or(settings.model.as_deref());
    let code = crate::utils::io::read_to_string(file)?;
    let messages = vec![
        ChatMessage { role: "system".into(), content: "You are a senior performance engineer. Optimize performance, readability, and error handling without changing semantics. Provide step-by-step suggestions and a final refactored version.".into() },
        ChatMessage { role: "user".into(), content: format!("Please optimize the following code {}:\n\n```\n{}\n```", file, code) },
    ];
    let content = if runtime.stream {
        if matches!(provider.to_lowercase().as_str(), "deepseek" | "qwen" | "openai" | "openai-compatible") {
            let base = settings.base_url.as_deref().ok_or_else(|| anyhow!("Missing base_url in config for OpenAI-compatible provider"))?;
            oa_chat_stream(http, base, &api_key, messages, model, |chunk| { print!("{}", chunk); let _ = std::io::Write::flush(&mut std::io::stdout()); }).await?
        } else {
            or_chat_stream(http, &api_key, messages, model, |chunk| { print!("{}", chunk); let _ = std::io::Write::flush(&mut std::io::stdout()); }).await?
        }
    } else {
        let pb = ProgressBar::new_spinner().with_message("Optimizing...");
        pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        let r = if matches!(provider.to_lowercase().as_str(), "deepseek" | "qwen" | "openai" | "openai-compatible") {
            let base = settings.base_url.as_deref().ok_or_else(|| anyhow!("Missing base_url in config for OpenAI-compatible provider"))?;
            oa_chat(http, base, &api_key, messages, model).await
        } else {
            or_chat(http, &api_key, messages, model).await
        };
        pb.finish_and_clear();
        match r { Ok(x) => x, Err(e) => { eprintln!("{}", style(format!("Optimize failed: {}", e)).red()); return Err(e); } }
    };
    if let Some(out) = &io.output_file { crate::utils::io::write_string(out, &content)?; }
    if !runtime.stream { println!("{}", content); }
    Ok(())
}
