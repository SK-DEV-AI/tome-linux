use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use tauri::http::{HeaderMap, HeaderName, HeaderValue};

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
}

#[derive(Deserialize)]
pub struct ProxyOptions {
    #[serde(default = "default_method")]
    method: HttpMethod,
    body: Option<String>,
    headers: Option<HashMap<String, String>>,
    timeout: Option<u64>,
}

#[derive(Serialize)]
pub struct HTTPResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
}

fn default_method() -> HttpMethod {
    HttpMethod::GET
}

#[tauri::command]
pub async fn fetch(url: String, options: ProxyOptions) -> Result<HTTPResponse, String> {
    let client = reqwest::Client::new();

    let request = match options.method {
        HttpMethod::GET => client.get(&url),
        HttpMethod::POST => client.post(&url),
        HttpMethod::PUT => client.put(&url),
        HttpMethod::DELETE => client.delete(&url),
        HttpMethod::HEAD => client.head(&url),
    };

    let mut headers = HeaderMap::new();
    // These are known-good values, so expect is acceptable for programmer error.
    headers.insert("Origin", "tauri://runebook.ai".parse().expect("Static origin header should be valid"));
    headers.insert("Content-Type", "application/json".parse().expect("Static content-type header should be valid"));

    if let Some(h) = options.headers {
        for (k, v) in h.iter() {
            let header_name = HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| format!("Invalid header name '{}': {}", k, e))?;
            let header_value = HeaderValue::from_bytes(v.as_bytes())
                .map_err(|e| format!("Invalid header value for '{}': {}", k, e))?;
            headers.insert(header_name, header_value);
        }
    }

    let request = request.headers(headers);

    let request = if let Some(timeout) = options.timeout {
        request.timeout(Duration::from_millis(timeout))
    } else {
        request
    };

    let request = if let Some(json) = options.body {
        request.body(json)
    } else {
        request
    };

    let response = request
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status().into();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();

    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                String::from_utf8_lossy(v.as_bytes()).to_string(),
            )
        })
        .collect::<HashMap<_, _>>();

    let body = match options.method {
        HttpMethod::HEAD => String::new(),
        _ => response
            .text()
            .await
            .map_err(|e| format!("Failed to build response: failed to read body: {}", e))?,
    };

    Ok(HTTPResponse {
        status,
        status_text,
        headers,
        body,
    })
}
