use axum::http::{self, StatusCode};
use axum::{
    Extension, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use axum_prometheus::PrometheusMetricLayer;
use http::Response;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::AppState;
use crate::config::GatewayGfg;
use crate::ingest::types::IngestBody;
use crate::readiness::{self, Readiness, start_readisness_probes};

#[derive(OpenApi)]
#[openapi(
    paths(crate::ingest::handler::ingest),
    components(schemas(IngestBody)),
    tags((name = "ingest", description = "Device data ingestion"))
)]
pub struct ApiDoc;

#[derive(Serialize)]
struct HealthReport {
    accepting: bool,
    disk_ok: bool,
    mqtt_ok: bool,
}

pub async fn serve(addr: std::net::SocketAddr, cfg: Arc<GatewayGfg>) -> anyhow::Result<()> {
    let (prom_layer, prom_handle) = PrometheusMetricLayer::pair();
    let app_metrics = crate::metrics::AppMetrics::new();
    let readiness = Arc::new(readiness::Readiness::new());
    start_readisness_probes(cfg.clone(), readiness.clone());

    // pipeline queue
    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<crate::ingest::types::Event>(cfg.ingest.queue_capacity);

    let state = AppState {
        cfg: cfg.clone(),
        ready: readiness.clone(),
        tx,
        metrics: app_metrics.clone(),
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

    let openapi = ApiDoc::openapi();

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs/openapi.json", openapi))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/v1/ingest/:device_id",
            post(crate::ingest::handler::ingest),
        )
        .route("/metrics", get(|| async move { prom_handle.render() }))
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
async fn healthz(
    State(st): State<AppState>,
    Extension(r): Extension<Arc<Readiness>>,
) -> impl IntoResponse {
    st.metrics.events_received();
    let report = HealthReport {
        accepting: true,
        disk_ok: r.disk_ok.load(Ordering::Relaxed),
        mqtt_ok: r.mqtt_ok.load(Ordering::Relaxed),
    };
    (StatusCode::OK, axum::Json(report))
}

#[tracing::instrument(skip_all)]
async fn readyz(
    State(st): State<AppState>,
    Extension(r): Extension<Arc<Readiness>>,
    Extension(cfg): Extension<Arc<GatewayGfg>>,
) -> impl IntoResponse {
    st.metrics.events_received();
    let ok = r.is_ready(&cfg.health);
    if ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
