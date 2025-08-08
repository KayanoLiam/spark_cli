use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::models::{ChatMessage, ChatRequest};

const DEFAULT_PATH: &str = "/chat/completions";

fn build_endpoint(base_url: &str) -> String { format!("{}{}", base_url.trim_end_matches('/'), DEFAULT_PATH) }

fn build_headers(api_key: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(headers)
}

pub async fn chat_complete(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    user_messages: Vec<ChatMessage>,
    model: Option<&str>,
) -> Result<String> {
    let endpoint = build_endpoint(base_url);

    let req = ChatRequest {
        model: model.unwrap_or("").to_string(),
        messages: user_messages,
        stream: None,
    };

    let headers = build_headers(api_key)?;

    let resp = client
        .post(&endpoint)
        .headers(headers)
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Provider error {}: {}", status, text));
    }

    #[derive(Debug, Deserialize)]
    struct OaChoiceDelta { content: Option<String> }
    #[derive(Debug, Deserialize)]
    struct OaChoiceMsg { content: String }
    #[derive(Debug, Deserialize)]
    struct OaChoice { delta: Option<OaChoiceDelta>, message: Option<OaChoiceMsg> }
    #[derive(Debug, Deserialize)]
    struct OaResp { choices: Vec<OaChoice> }

    let body: OaResp = resp.json().await?;
    let content = body
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.map(|m| m.content))
        .ok_or_else(|| anyhow!("Response has no content"))?;
    Ok(content)
}

pub async fn chat_complete_stream<F: FnMut(&str)>(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    user_messages: Vec<ChatMessage>,
    model: Option<&str>,
    mut on_chunk: F,
) -> Result<String> {
    let endpoint = build_endpoint(base_url);

    #[derive(Serialize)]
    struct StreamReq<'a> { model: &'a str, messages: &'a [ChatMessage], stream: bool }
    let req = StreamReq { model: model.unwrap_or(""), messages: &user_messages, stream: true };

    let headers = build_headers(api_key)?;

    let resp = client
        .post(&endpoint)
        .headers(headers)
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Provider error {}: {}", status, text));
    }

    let mut stream = resp.bytes_stream();
    let mut buffer = Vec::new();
    let mut final_text = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend_from_slice(&chunk);
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer.drain(..=pos).collect::<Vec<u8>>();
            let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
            if line.is_empty() { continue; }
            let data = if let Some(rest) = line.strip_prefix("data: ") { rest } else if let Some(rest) = line.strip_prefix("data:") { rest } else { continue };
            if data == "[DONE]" { break; }
            if let Ok(value) = serde_json::from_str::<Value>(data) {
                if let Some(s) = value.get("choices").and_then(|v| v.get(0)).and_then(|v| v.get("delta")).and_then(|v| v.get("content")).and_then(|v| v.as_str()) {
                    if !s.is_empty() { on_chunk(s); final_text.push_str(s); }
                    continue;
                }
                if let Some(s) = value.get("choices").and_then(|v| v.get(0)).and_then(|v| v.get("message")).and_then(|v| v.get("content")).and_then(|v| v.as_str()) {
                    if !s.is_empty() { on_chunk(s); final_text.push_str(s); }
                    continue;
                }
            }
        }
    }

    Ok(final_text)
}
