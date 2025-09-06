use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct ClientOptions {
    pub url: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Engine {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub options: ClientOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
