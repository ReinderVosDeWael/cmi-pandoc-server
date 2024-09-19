use axum::Router;

#[path = "routers/pandoc.rs"]
mod pandoc_router;

#[path = "routers/health.rs"]
mod health_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .merge(pandoc_router::init_router())
        .merge(health_router::init_router());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
