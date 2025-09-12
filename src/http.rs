use axum::http::StatusCode;
use axum::{Router, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

pub async fn serve(addr: std::net::SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics));

    let listener: TcpListener = TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    "ok"
}

async fn readyz() -> impl IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, "not ready")
}

async fn metrics() -> impl IntoResponse {
    "metrics go here\n"
}
