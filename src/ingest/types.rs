use serde::Deserialize;
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
