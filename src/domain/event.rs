use serde::Serialize;
use std::collections::BTreeMap;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub device_id: String,
    pub ts: OffsetDateTime,
    pub seq: Option<u64>,
    pub metrics: BTreeMap<String, f64>,
    pub tags: BTreeMap<String, String>,
    pub payload: serde_json::Value,
    pub received_at: OffsetDateTime,
    pub bytes: usize,
}
