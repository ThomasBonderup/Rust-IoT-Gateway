# Rust IoT Gateway (Work in Progress)

This is a portfolio project demonstrating how to build a secure observable IoT gateway in Rust.
The gateway connect STM32-based IoT devices to AWS IoT Core over MQTT/TLS applying systems programming practices.

- OpenTelemetry traces + Prometheus metrics + Grafana dashboards

## Status
- [x] Health endpoint (`/health`, `/ready`)
- [x] Prometheus metrics endpoint (`/metrics`)
- [] MQTT publish to AWS IoT Core
- [] Observability stack (OTel Collector, Prometheus, Grafana, Tempo)

## Tech Stack
IoT, Rust, async, MQTT, TLS, OpenTelemetry

## Smoke test with curl
Start the server
```
RUST_LOG=info cargo run
```
Then:
```
curl -i -X POST 'http://127.0.0.1:8000/v1/ingest/device_1' \                                                                                                             13:20:41
  -H 'Content-Type: application/json' \
  -d '{"seq":1,"metrics":{"temp":21.5},"tags":{"site":"AAL"},"payload":{"raw":"ok"}}'
```
