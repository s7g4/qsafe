# 10. Observability, Tracing, and Telemetry

## Status

Proposed

## Context

In production environments, standard prints (`println!`) do not scale. They lack execution context, cannot correlate requests across async boundaries, and do not format logs in a structured layout (e.g. JSON) suitable for downstream ingestion by log collectors like Elasticsearch, Datadog, or Grafana Loki.

Furthermore, we need raw visibility into gateway system metrics (such as API request rates, handler execution latencies, and active WebSocket connection counts) to proactively diagnose connection leaks, performance regressions, or database pool exhaustion.

## Decision

We will:
1. **Structured JSON Logging**: Integrate the `tracing` and `tracing-subscriber` ecosystems.
   - Configure a global tracing subscriber in [main.rs](../../host-server/src/main.rs) at startup to format logs in a structured format.
   - Inject Request Correlation IDs into every request's tracing span using `tower-http` middleware, allowing log aggregation tools to group all log lines associated with a single HTTP transaction.
2. **Prometheus Telemetry Metrics**: Integrate the `metrics` and `metrics-exporter-prometheus` crates.
   - Expose a `/metrics` route on the Axum gateway that outputs formatted statistics in a standard Prometheus text layout.
   - Instrument key gateway checkpoints (such as WebSocket join/disconnect loops in [websocket.rs](../../host-server/src/websocket.rs)).

## Consequences

- **Visibility**: Improved operational visibility, request tracking, and post-incident forensic log investigation capabilities.
- **Portability**: Metrics are published in standard format, compatible out-of-the-box with standard monitoring systems like Prometheus and Grafana.
- **Performance**: The selected libraries execute with minimal allocation overhead, preventing throughput degradation under high concurrency.
