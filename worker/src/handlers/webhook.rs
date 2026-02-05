use reqwest::Client;
use sqlx::types::JsonValue;
use tracing::info;

use crate::error::WorkerError;

pub async fn send_webhook(
    client: Client,
    payload: JsonValue,
) -> Result<Option<JsonValue>, WorkerError> {
    let Some(url) = payload["url"].as_str() else {
        return Err(WorkerError::Webhook("Invalid url".to_string()));
    };
    let method = payload["method"].as_str().unwrap_or("POST");
    let body = payload["body"].clone();

    let request = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => return Err(WorkerError::Webhook("Invalid method".to_string())),
    };

    let response = request
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    info!("response: {:?}", response);

    let response_json = response.json::<JsonValue>().await?;
    info!("response_json: {:?}", response_json);

    Ok(Some(response_json))
}
