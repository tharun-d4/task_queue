use api_server::{app, db::connection, state};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = connection::create_pool().await.unwrap();
    connection::run_migrations(&pool).await.unwrap();

    let state = state::AppState::new(pool);
    let app = app::create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
