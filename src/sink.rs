use axum::async_trait;

use crate::domain::Event;

#[derive(thiserror::Error, Debug)]
pub enum SinkError {
    #[error("transient: {0}")]
    Transiet(String),
    #[error("fatal: {0}")]
    Fatal(String),
}

#[async_trait]
pub trait Sink: Send + Sync {
    async fn send(&self, ev: Event) -> Result<(), SinkError>;
}

#[async_trait]
pub trait EnqueueSink: Send + Sync {
    fn try_enqueue(&self, ev: Event) -> Result<(), SinkError>;
}
