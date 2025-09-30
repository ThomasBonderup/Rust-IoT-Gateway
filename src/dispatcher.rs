use std::sync::Arc;

use crate::domain::Event;
use crate::fanout::FanoutSink;

pub struct Dispatcher {
    rx: tokio::sync::mpsc::Receiver<Event>,
    fanout: Arc<FanoutSink>,
}

impl Dispatcher {
    pub fn new(rx: tokio::sync::mpsc::Receiver<Event>, fanout: Arc<FanoutSink>) -> Self {
        Self { rx, fanout }
    }

    pub async fn run(mut self) {
        while let Some(ev) = self.rx.recv().await {
            let accepted = self.fanout.try_enqueue(ev);
        }
    }
}
