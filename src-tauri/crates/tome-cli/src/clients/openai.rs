use anyhow::Result;
use serde::{Deserialize, Serialize};
use futures_util::{Stream, StreamExt};
use crate::models::ChatMessage;

#[derive(Debug, Deserialize)]
pub struct OpenAIModel {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

pub async fn get_models(base_url: &str, api_key: Option<&str>) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let mut request = client.get(format!("{}/v1/models", base_url));

    if let Some(key) = api_key {
        request = request.bearer_auth(key);
    }

    let response = request.send().await?;
    let json: OpenAIModelsResponse = response.json().await?;
    Ok(json.data.into_iter().map(|m| m.id).collect())
}


#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatChoiceDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatChunk {
    choices: Vec<OpenAIChatChoiceDelta>,
}

pub async fn chat_stream(
    base_url: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    api_key: Option<&str>,
) -> Result<impl Stream<Item = Result<String>>> {
    let client = reqwest::Client::new();
    let request_body = OpenAIChatRequest {
        model: model.to_string(),
        messages,
        stream: true,
    };

    let mut request = client.post(format!("{}/v1/chat/completions", base_url)).json(&request_body);
    if let Some(key) = api_key {
        request = request.bearer_auth(key);
    }

    let response = request.send().await?;

    let stream = response.bytes_stream().map(|res| {
        res.map_err(anyhow::Error::from).and_then(|bytes| {
            let s = std::str::from_utf8(&bytes)?;
            // OpenAI streaming responses are prefixed with "data: "
            if s.starts_with("data: ") {
                let json_str = &s[6..];
                if json_str.trim() == "[DONE]" {
                    return Ok(None);
                }
                serde_json::from_str::<OpenAIChatChunk>(json_str)
                    .map_err(anyhow::Error::from)
                    .map(|chunk| chunk.choices.into_iter().next().and_then(|delta| delta.content))
            } else {
                Ok(None)
            }
        })
    });

    // We need to filter out the None values from the stream
    let content_stream = stream.filter_map(|res| async move {
        match res {
            Ok(Some(content)) => Some(Ok(content)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    });

    Ok(content_stream)
}
