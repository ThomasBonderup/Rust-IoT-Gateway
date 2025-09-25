use crate::app::AppState;
use crate::ingest::types::{Event, IngestBody};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use time::OffsetDateTime;

const MAX_METRICS: usize = 32;

fn validate_maps(body: &IngestBody) -> Result<(), &'static str> {
    if body.metrics.len() > MAX_METRICS {
        return Err("too_many_metrics");
    }
    Ok(())
}

pub async fn ingest(
    State(st): State<AppState>,
    Path(device_id): Path<String>,
    Json(body): Json<IngestBody>,
) -> impl IntoResponse {
    if !st.ready.is_ready(&st.cfg.health) {
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    if let Err(reason) = validate_maps(&body) {
        st.metrics.ingest_rejected_total(reason);
        return StatusCode::BAD_REQUEST;
    }

    let now = OffsetDateTime::now_utc();
    let event = Event {
        device_id,
        ts: body.ts.unwrap_or(now),
        seq: body.seq,
        metrics: body.metrics,
        tags: body.tags,
        payload: body.payload,
        received_at: now,
        bytes: 0,
    };

    match st.cfg.ingest.ack_mode {
        crate::config::AckMode::Enqueue => {
            if st.tx.try_send(event).is_err() {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::ACCEPTED
            }
        }
        crate::config::AckMode::Sink => {
            if st.tx.try_send(event).is_err() {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            }
        }
    }
}
