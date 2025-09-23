use axum::http::{self, HeaderValue, StatusCode};
use axum::{
    Extension, Router,
    response::IntoResponse,
    routing::{get, post},
};
use axum_prometheus::PrometheusMetricLayer;
use http::Response;
use prometheus::{Encoder, TextEncoder};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::app::AppState;
use crate::config::GatewayGfg;
use crate::metrics::EVENTS_RECEIVED;
use crate::readiness::{self, Readiness, start_readisness_probes};

#[derive(Serialize)]
struct HealthReport {
    accepting: bool,
    disk_ok: bool,
    mqtt_ok: bool,
}

pub async fn serve(addr: std::net::SocketAddr, cfg: Arc<GatewayGfg>) -> anyhow::Result<()> {
    let (prom_layer, prom_handle) = PrometheusMetricLayer::pair();

    let readiness = Arc::new(readiness::Readiness::new());
    start_readisness_probes(cfg.clone(), readiness.clone());

    // pipeline queue
    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<crate::ingest::types::Event>(cfg.ingest.queue_capacity);

    let state = AppState {
        cfg: cfg.clone(),
        ready: readiness.clone(),
        tx,
    };

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            tracing::info!(
              device=%event.device_id,
              seq=?event.seq,
              bytes=event.bytes,
              "sink: processing event"
            );

            tracing::info!(device=%event.device_id, "sink: ok");
        }
    });

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/v1/ingest/:device_id",
            post(crate::ingest::handler::ingest),
        )
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
        .with_state(state.clone())
        .layer(RequestBodyLimitLayer::new(
            state.cfg.ingest.max_payload_bytes,
        ))
        .layer(Extension(readiness.clone()))
        .layer(Extension(cfg.clone()))
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
    readiness.set_accepting(true);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(readiness.clone()))
        .await?;
    Ok(())
}

pub async fn shutdown_signal(readiness: Arc<Readiness>) {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigint = signal(SignalKind::interrupt()).expect("sigint");
    let mut sigterm = signal(SignalKind::terminate()).expect("sigterm");

    tokio::select! {
      _ = sigint.recv() => (),
      _ = sigterm.recv() => (),
    }

    readiness.set_accepting(false);
    readiness.disk_ok.store(false, Ordering::Relaxed);
    readiness.mqtt_ok.store(false, Ordering::Relaxed);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}

#[tracing::instrument(skip_all, fields(kind = "health"))]
async fn healthz(Extension(r): Extension<Arc<Readiness>>) -> impl IntoResponse {
    EVENTS_RECEIVED.inc();
    let report = HealthReport {
        accepting: true,
        disk_ok: r.disk_ok.load(Ordering::Relaxed),
        mqtt_ok: r.mqtt_ok.load(Ordering::Relaxed),
    };
    (StatusCode::OK, axum::Json(report))
}

#[tracing::instrument(skip_all)]
async fn readyz(
    Extension(r): Extension<Arc<Readiness>>,
    Extension(cfg): Extension<Arc<GatewayGfg>>,
) -> impl IntoResponse {
    EVENTS_RECEIVED.inc();
    let ok = r.is_ready(&cfg.health);
    if ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
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
