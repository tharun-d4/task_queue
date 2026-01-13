use api_server::{app::create_router, db::connection::create_pool, state::AppState};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let pool = create_pool().await.unwrap();
    let state = AppState::new(pool);
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
