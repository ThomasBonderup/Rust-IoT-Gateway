use axum::{Extension, Json};
use http::Response;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tracing::Span;
use tracing_subscriber::fmt::format::json;

use axum::http::{self, StatusCode};
use axum::{Router, response::IntoResponse, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::config::GatewayGfg;
use crate::readiness::{self, Readiness, start_readisness_probes};

#[derive(Serialize)]
struct ReadyReport {
    disk_ok: bool,
}

pub async fn serve(addr: std::net::SocketAddr, cfg: Arc<GatewayGfg>) -> anyhow::Result<()> {
    let (prom_layer, prom_handle) = PrometheusMetricLayer::pair();

    let readiness = Arc::new(readiness::Readiness::new());
    start_readisness_probes(cfg.clone(), readiness.clone());

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
        .layer(Extension(readiness.clone()))
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

#[tracing::instrument(skip_all, fields(kind = "health"))]
async fn healthz() -> impl IntoResponse {
    tracing::info!("health probe ok");
    "ok"
}

#[tracing::instrument(skip_all)]
async fn readyz(Extension(r): Extension<Arc<Readiness>>) -> impl IntoResponse {
    let report = ReadyReport {
        disk_ok: r.disk_ok.load(Ordering::Relaxed),
    };
    if r.all_ok() {
        (StatusCode::OK, Json(report))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(report))
    }
}

async fn metrics() -> impl IntoResponse {
    "metrics go here\n"
}
