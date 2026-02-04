use reqwest::Client;
use sqlx::types::JsonValue;
use tracing::info;

pub async fn send_webhook(client: Client, payload: JsonValue) -> Result<Option<JsonValue>, String> {
    let url = payload["url"]
        .as_str()
        .ok_or_else(|| "Missing url field in payload".to_string())?;
    let method = payload["method"].as_str().unwrap_or("POST");
    let body = payload["body"].clone();

    let request = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => return Err(format!("Unsupported method: {}", method)),
    };

    let response = request
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Webhook request failed: {:?}", e))?;
    info!("response: {:?}", response);

    let response_json = response.json::<JsonValue>().await.unwrap();
    info!("response_json: {:?}", response_json);

    Ok(Some(response_json))
}
