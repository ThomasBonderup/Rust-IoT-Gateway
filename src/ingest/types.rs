use std::collections::BTreeMap;
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
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
    pub payload: serde_json::value,
}

pub struct Event {
    pub device_id: String,
    pub ts: OffsetDateTime,
    pub seq: Option<u64>,
    pub metrics: BTreeMap<String, f64>,
    pub tags: BTreeMap<String, String>,
    pub payload: serde_json::value,
    pub received_at: OffsetDateTime,
    pub bytes: usize,
}
