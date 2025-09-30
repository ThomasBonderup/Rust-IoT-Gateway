use std::sync::Arc;

use crate::{domain::Event, sink::EnqueueSink};
pub struct FanoutSink {
    sinks: Vec<Arc<dyn EnqueueSink>>,
}

impl FanoutSink {
    pub fn new(sinks: Vec<Arc<dyn EnqueueSink>>) -> Self {
        Self { sinks }
    }
    pub fn try_enqueue(&self, ev: Event) -> usize {
        let mut accepted = 0;
        for s in &self.sinks {
            if s.try_enqueue(ev.clone()).is_ok() {
                accepted += 1;
            }
        }
        accepted
    }
}
