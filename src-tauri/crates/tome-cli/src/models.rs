use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientOptions {
    pub url: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug)]
pub struct Engine {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub options: ClientOptions,
}
