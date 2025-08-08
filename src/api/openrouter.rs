use anyhow::{anyhow, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
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

    // Build headers per OpenRouter docs
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "HTTP-Referer",
        HeaderValue::from_static("https://github.com/your-org/spark_cli"),
    );
    headers.insert("X-Title", HeaderValue::from_static("spark_cli"));

    let resp = client
        .post(DEFAULT_ENDPOINT)
        .headers(headers)
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
