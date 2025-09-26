use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct IngestBody {
    #[serde(default)]
    pub ts: Option<OffsetDateTime>,
    #[serde(default)]
    pub seq: Option<u64>,
    #[serde(default)]
    pub metrics: BTreeMap<String, f64>,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

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
