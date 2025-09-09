# Rust IoT Gateway (Work in Progress)

This is a portfolio project demonstrating how to build a secure observable IoT gateway in Rust.
The gateway connect STM32-based IoT devices to AWS IoT Core over MQTT/TLS applying systems programming practices.

- OpenTelemetry traces + Prometheus metrics + Grafana dashboards

## Status
- [] Health endpoint (`/health`, `/ready`)
- [] Prometheus metrics endpoint (`/metrics`)
- [] MQTT publish to AWS IoT Core
- [] Observability stack (OTel Collector, Prometheus, Grafana, Tempo)

## Tech Stack
IoT, Rust, async, MQTT, TLS, OpenTelemetry