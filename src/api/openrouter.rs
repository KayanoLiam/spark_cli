use anyhow::{anyhow, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::api::models::{ChatMessage, ChatRequest};

const DEFAULT_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_MODEL: &str = "openrouter/auto";

#[derive(Debug, Serialize, Deserialize)]
struct OrChoiceMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrChoice {
    message: OrChoiceMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrResponse {
    choices: Vec<OrChoice>,
}

pub async fn chat_complete(
    client: &reqwest::Client,
    api_key: &str,
    user_messages: Vec<ChatMessage>,
    model: Option<&str>,
) -> Result<String> {
    let model_name = model.unwrap_or(DEFAULT_MODEL);

    let req = ChatRequest {
        model: model_name.to_string(),
        messages: user_messages,
        stream: None,
    };

    let resp = client
        .post(DEFAULT_ENDPOINT)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .header(CONTENT_TYPE, "application/json")
        // Optional but recommended headers for OpenRouter
        .header("X-Title", "spark_cli")
        .json(&req)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("OpenRouter error {}: {}", status, text));
    }

    let body: OrResponse = resp.json().await?;
    let content = body
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| anyhow!("OpenRouter response has no choices"))?;
    Ok(content)
}
