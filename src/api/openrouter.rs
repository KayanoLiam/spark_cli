use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    let headers = build_headers(api_key)?;

    let resp = client
        .post(DEFAULT_ENDPOINT)
        .headers(headers)
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {}", e))?;

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

pub async fn chat_complete_stream<F: FnMut(&str)>(
    client: &reqwest::Client,
    api_key: &str,
    user_messages: Vec<ChatMessage>,
    model: Option<&str>,
    mut on_chunk: F,
) -> Result<String> {
    let model_name = model.unwrap_or(DEFAULT_MODEL);

    // Enable stream
    #[derive(Serialize)]
    struct StreamReq<'a> {
        model: &'a str,
        messages: &'a [ChatMessage],
        stream: bool,
    }
    let req = StreamReq { model: model_name, messages: &user_messages, stream: true };

    let headers = build_headers(api_key)?;

    let resp = client
        .post(DEFAULT_ENDPOINT)
        .headers(headers)
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("OpenRouter error {}: {}", status, text));
    }

    let mut stream = resp.bytes_stream();
    let mut buffer = Vec::new();
    let mut final_text = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend_from_slice(&chunk);
        // Split by newlines to process SSE lines
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer.drain(..=pos).collect::<Vec<u8>>();
            let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
            if line.is_empty() { continue; }
            // Expect "data: ..."
            let data = if let Some(rest) = line.strip_prefix("data: ") {
                rest
            } else if let Some(rest) = line.strip_prefix("data:") {
                rest
            } else {
                continue;
            };
            if data == "[DONE]" { break; }
            // Try parse JSON
            if let Ok(value) = serde_json::from_str::<Value>(data) {
                // Prefer OpenAI-style delta { choices[0].delta.content }
                if let Some(s) = value
                    .get("choices").and_then(|v| v.get(0))
                    .and_then(|v| v.get("delta"))
                    .and_then(|v| v.get("content"))
                    .and_then(|v| v.as_str())
                {
                    if !s.is_empty() { on_chunk(s); final_text.push_str(s); }
                    continue;
                }
                // Fallback: some providers may send message { choices[0].message.content }
                if let Some(s) = value
                    .get("choices").and_then(|v| v.get(0))
                    .and_then(|v| v.get("message"))
                    .and_then(|v| v.get("content"))
                    .and_then(|v| v.as_str())
                {
                    if !s.is_empty() { on_chunk(s); final_text.push_str(s); }
                    continue;
                }
            }
        }
    }

    Ok(final_text)
}

fn build_headers(api_key: &str) -> Result<HeaderMap> {
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
    Ok(headers)
}
