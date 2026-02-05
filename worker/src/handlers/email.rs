use lettre::{
    AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::AsyncSmtpTransport,
};

use crate::{error::WorkerError, handlers::models::EmailInfo};

pub fn smtp_sender(server: &str, port: u16) -> AsyncSmtpTransport<Tokio1Executor> {
    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(server)
        .port(port)
        .build()
}

pub async fn send_email(
    sender: AsyncSmtpTransport<Tokio1Executor>,
    info: EmailInfo,
) -> Result<(), WorkerError> {
    let message = Message::builder()
        .from(info.from.parse()?)
        .to(info.to.parse()?)
        .subject(&info.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(info.body)?;

    sender.send(message).await?;

    Ok(())
}
