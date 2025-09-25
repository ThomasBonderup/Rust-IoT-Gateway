# Rust IoT Gateway (Work in Progress)

This is a portfolio project demonstrating how to build a secure observable IoT gateway in Rust.
The gateway connect STM32-based IoT devices to AWS IoT Core over MQTT/TLS applying systems programming practices.

- OpenTelemetry traces + Prometheus metrics + Grafana dashboards

## Status
- [x] Health endpoint (`/health`, `/ready`)
- [x] Prometheus metrics endpoint (`/metrics`)
- [] Ingest handler
- [] MQTT Sink with publish to AWS IoT Core
- [] File Sink
- [] Reliability & safety (Idempotency, Write-Ahead Log / persistent queue, retry policy)
- [] Security
- [] Observability & SLOs (OTel Collector, Prometheus, Grafana, Tempo)
- [] Testing (Unit, integration and load test)
- [] Packaging & deploy

Nice to haves:
- Batch ingest, schema, OpenAPI UI, Client SDKs

## Tech Stack
IoT, Rust, async, MQTT, TLS, OpenTelemetry

## Smoke test with curl
Start the server
```
RUST_LOG=info cargo run
```
Then:
```
curl -i -X POST 'http://127.0.0.1:8000/v1/ingest/device_1' \
  -H 'Content-Type: application/json' \
  -d '{"seq":1,"metrics":{"temp_c":21.5},"tags":{"site":"AAL"},"payload":{"raw":"ok"}}'
```
