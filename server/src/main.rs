#[tokio::main]
async fn main() -> Result<(), server::error::ServerError> {
    server::init().await
}
