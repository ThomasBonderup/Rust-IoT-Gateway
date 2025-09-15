use http::Response;
use std::time::Duration;
use tracing::Span;

use axum::http::{self, StatusCode};
use axum::{Router, response::IntoResponse, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub async fn serve(addr: std::net::SocketAddr) -> anyhow::Result<()> {
    let (prom_layer, prom_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/metrics",
            get({
                let prom_handle = prom_handle.clone();
                move || async move { prom_handle.render() }
            }),
        )
        .layer(prom_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &http::Request<_>| {
                    tracing::info_span!(
                      "http_request",
                      method = %req.method(),
                      path = %req.uri().path(),
                    )
                })
                .on_response(|res: &Response<_>, latency: Duration, _span: &Span| {
                    tracing::info!(
                      status = %res.status(),
                      latency_ms = %latency.as_millis(),
                      "response"
                    )
                })
                .on_failure(|error: _, latency: Duration, _span: &Span| {
                    tracing::warn!(latency_ms = %latency.as_millis(), "request_failed");
                }),
        );

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
