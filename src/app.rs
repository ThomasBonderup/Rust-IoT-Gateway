use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::GatewayGfg;
use crate::ingest::types::Event;
use crate::metrics::AppMetrics;
use crate::readiness::Readiness;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<GatewayGfg>,
    pub ready: Arc<Readiness>,
    pub ingest_tx: mpsc::Sender<Event>,
    pub metrics: Arc<AppMetrics>,
}
