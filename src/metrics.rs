use once_cell::sync::Lazy;
use prometheus::{IntCounter, Registry};

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static EVENTS_RECEIVED: Lazy<IntCounter> = Lazy::new(|| {
    let events_received = IntCounter::new(
        "gateway_events_received_total",
        "Number of events received via endpoints",
    )
    .unwrap();

    REGISTRY
        .register(Box::new(events_received.clone()))
        .unwrap();
    events_received
});
