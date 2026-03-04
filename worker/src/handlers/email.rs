use lettre::{
    AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::AsyncSmtpTransport,
};
use sqlx::types::JsonValue;
use tracing::info;

use crate::{error::WorkerError, handlers::models::EmailInfo};

pub fn smtp_sender(server: &str, port: u16) -> AsyncSmtpTransport<Tokio1Executor> {
    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(server)
        .port(port)
        .build()
}

pub async fn send_email(
    sender: AsyncSmtpTransport<Tokio1Executor>,
    payload: JsonValue,
) -> Result<Option<JsonValue>, WorkerError> {
    let info: EmailInfo = serde_json::from_value(payload)
        .map_err(|e| WorkerError::permanent("Deserialization error of email info").set_source(e))?;

    let from = info
        .from
        .parse()
        .map_err(|e| WorkerError::permanent("Failed to deserialize 'from' email").set_source(e))?;

    let to = info
        .to
        .parse()
        .map_err(|e| WorkerError::permanent("Failed to deserialize 'to' email").set_source(e))?;

    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(&info.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(info.body)
        .map_err(|e| WorkerError::permanent("Failed to build email message").set_source(e))?;

    info!("Sending an email");
    sender
        .send(message)
        .await
        .map_err(|e| WorkerError::temporary("Failed to send email").set_source(e))?;

    Ok(None)
}
