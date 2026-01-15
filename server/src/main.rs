use server::{app, error, state};
use shared::{config::load_config, db::connection};

#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let config = load_config().expect("Config Error");

    let pool = connection::create_pool(&config.database).await?;
    connection::run_migrations(&pool).await?;

    let state = state::AppState::new(pool);
    let app = app::create_router(state);

    let bind = format!("{}:{}", config.server.host, config.server.port);
    println!("[+] Server running on {bind:?}...");

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
