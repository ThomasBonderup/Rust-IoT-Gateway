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

## API Docs

### Swagger UI (interactive)
Once the gateway is running, open:
- **Swagger UI:** `http://127.0.0.1:8000/docs`
- **RAW OpenAPI JSON:** `http://127.0.0.1:8000/docs/openapi.json`

> The UI includes **Try it out** for `POST /v1/ingest/{device_id}` and shows all statuses (200/202/400/413/503).

---
### Endpoints

| Method | Path                            | Purpose                     |
|-------:|---------------------------------|-----------------------------|
|   GET  | `/readyz`                       | Readiness (routing only)    |
|   GET  | `/healthz`                      | Health (component status)   |
|   GET  | `/metrics`                      | Prometheus metrics          |
|  POST  | `/v1/ingest/{device_id}`        | Ingest device telemetry     |

**Versioning:** This is **v1**. Breaking changes will land under a new prefix (`/v2/...`).

**Auth:** None yet (planned: HMAC/JWT). Keep endpoints private in prod.

---

### `POST /v1/ingest/{device_id}`

**Path params**
- `device_id` — `^[A-Za-z0-9_-]{1,64}$`

**Body (JSON)**
```json
{
  "ts": "2025-09-23T11:18:41Z",        // optional; server fills if omitted
  "seq": 42,                           // optional; used for idempotency later
  "metrics": { "temp_c": 21.5 },       // numbers only; ≤32 keys
  "tags": { "site": "AAL" },           // short strings; ≤16 keys
  "payload": { "raw": "ok" }           // optional structured details
}
```

### Validation rules
- Metric/tag keys: `^[a-z][a-z0-9_]{0,31}$`
- `metrics` ≤ **32** keys; tags ≤ **16** keys
- Metric values must be finite numbers (no NaN/Inf)

### Content
- `Content-Type: application/json`
- **Compression:** request body compression is not supported yet (send plain JSON).

### Limits & backpressure
- Body size capped by config → **413 Payload Too Large**
- During drain or when the queue is full → **503 Service Unavailable**
(Optionally expose `Retry-After: <seconds>` when enabled; otherwise clients should back off with jitter.)

### Status codes

- **202 Accepted** — enqueued (AckMode: `enqueue`)

- **200 OK** — accepted (AckMode: `sink`, first cut still enqueues internally)

- **400 Bad Request** — validation failed (bad key, too many metrics/tags, non-finite)

- **413 Payload Too Large** — body exceeds configured limit

- **503 Service Unavailable** — not accepting (draining) or queue full

### Error body
- Current: empty body with status code.
- Optional (planned):
{ "code": "too_many_metrics", "message": "metrics must have ≤ 32 keys" }


### Idempotency (planned)
- `(device_id, seq)` used to drop duplicates; until then, clients should avoid replaying the same seq.

## Quick start
```bash
# Run the gateway
cargo run --release

# Open the docs
open http://127.0.0.1:8000/docs        # macOS
# xdg-open http://127.0.0.1:8000/docs  # Linux
```

## Example requests
### Ready / Not Ready
```bash
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8000/readyz
```

### Ingest (happy path)
```bash
curl -i -X POST 'http://127.0.0.1:8000/v1/ingest/device_1' \
  -H 'Content-Type: application/json' \
  -d '{"seq":1,"metrics":{"temp_c":21.5},"tags":{"site":"AAL"},"payload":{"raw":"ok"}}'
```

## Export OpenAPI Spec (Planned)
TODO: Add a --dump_openapi flag to rust binary.

```bash
cargo run -- --dump_openapi > openapi.json
```


### Clients (Planned)
TODO: Use external CLI to generate a Python client from OpenAPI spec.

## Metrics
- /metrics exposes HTTP + app metrics (e.g., gateway_events_received_total)