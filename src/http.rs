use axum::http::{self, HeaderValue, StatusCode};
use axum::{Extension, Json, Router, response::IntoResponse, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use http::Response;
use prometheus::{Encoder, TextEncoder};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::config::GatewayGfg;
use crate::metrics::{self, EVENTS_RECEIVED};
use crate::readiness::{self, Readiness, start_readisness_probes};

#[derive(Serialize)]
struct ReadyReport {
    disk_ok: bool,
    mqtt_ok: bool,
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
                let h = prom_handle.clone();
                move || {
                    let text = h.render();
                    metrics(text)
                }
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

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(readiness.clone()))
        .await?;
    Ok(())
}

pub async fn shutdown_signal(readiness: Arc<Readiness>) {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
      _ = sigint.recv() => (),
      _ = sigterm.recv() => (),
    }

    readiness.disk_ok.store(false, Ordering::Relaxed);
    readiness.mqtt_ok.store(false, Ordering::Relaxed);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}

#[tracing::instrument(skip_all, fields(kind = "health"))]
async fn healthz() -> impl IntoResponse {
    EVENTS_RECEIVED.inc();
    tracing::info!("health probe ok");
    "ok"
}

#[tracing::instrument(skip_all)]
async fn readyz(Extension(r): Extension<Arc<Readiness>>) -> impl IntoResponse {
    EVENTS_RECEIVED.inc();
    let report = ReadyReport {
        disk_ok: r.disk_ok.load(Ordering::Relaxed),
        mqtt_ok: r.mqtt_ok.load(Ordering::Relaxed),
    };
    if r.all_ok() {
        (StatusCode::OK, Json(report))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(report))
    }
}

pub async fn metrics(prom_text: String) -> impl IntoResponse {
    let mut buf = Vec::with_capacity(prom_text.len() + 4096);
    buf.extend_from_slice(prom_text.as_bytes());

    let encoder = TextEncoder::new();
    let mfs = crate::metrics::REGISTRY.gather();
    encoder.encode(&mfs, &mut buf).unwrap();
    let body = String::from_utf8(buf).unwrap();
    (
        [(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4"),
        )],
        body,
    )
}
