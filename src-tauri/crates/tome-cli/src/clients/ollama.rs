use anyhow::Result;
use serde::Deserialize;

use crate::models::ChatMessage;

#[derive(Debug, Deserialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

use futures_util::{Stream, StreamExt};
use serde::Serialize;

pub async fn get_models(url: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/api/tags", url)).send().await?;
    let json: OllamaTagsResponse = response.json().await?;
    Ok(json.models.into_iter().map(|m| m.name).collect())
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
}

pub async fn chat_stream(
    url: &str,
    model: &str,
    messages: Vec<ChatMessage>,
) -> Result<impl Stream<Item = Result<ChatResponse>>> {
    let client = reqwest::Client::new();
    let request_body = ChatRequest {
        model: model.to_string(),
        messages,
        stream: true,
    };

    let response = client
        .post(format!("{}/api/chat", url))
        .json(&request_body)
        .send()
        .await?;

    let stream = response
        .bytes_stream()
        .map(|res| {
            res.map_err(anyhow::Error::from)
                .and_then(|bytes| serde_json::from_slice(&bytes).map_err(anyhow::Error::from))
        });

    Ok(stream)
}
