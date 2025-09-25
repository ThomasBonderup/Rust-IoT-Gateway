use metrics::{Unit, counter, describe_counter};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct AppMetrics;

impl AppMetrics {
    pub fn new() -> Arc<Self> {
        describe_counter!(
            "gateway_events_received_total",
            Unit::Count,
            "Number of events received via endpoints"
        );
        describe_counter!(
            "ingest_rejected_total",
            Unit::Count,
            "Rejected ingest requests by reason"
        );
        Arc::new(Self)
    }

    pub fn ingest_rejected_total(&self, reason: &'static str) {
        counter!("ingest_rejected_total", "reason" => reason).increment(1);
    }
    pub fn events_received(&self) {
        counter!("gateway_events_received_total").increment(1);
    }
}
