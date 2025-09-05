use anyhow::Result;
use serde::Deserialize;

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
