# Minimal Local Build Design

## Goal

Make the default local Rust build omit the GCS/AWS-LC, remote telemetry, and Computer Hub metrics dependency paths while retaining the interactive TUI, local logging, MCP support, authentication, and non-GCS upload paths.

## Scope

The build will be controlled by three opt-in Cargo features:

- `gcs`: enables direct Google Cloud Storage uploads through `gcloud-storage`.
- `hub-metrics`: enables Prometheus collection and Computer Hub metric donation.
- `telemetry`: enables product telemetry: Sentry, Mixpanel, OpenTelemetry/OTLP, and remote event emission.

These features are disabled by default for the local development build. A full build can explicitly enable all three.

## Architecture

### GCS

`xai-file-utils` will make `gcloud-storage` optional behind a `gcs` feature. The direct GCS branches in `gcs.rs` will be conditionally compiled. Without that feature, choosing `UploadMethod::Direct` will return a deterministic error explaining that direct GCS support was omitted and how to enable it.

Proxy and S3 uploads are unchanged. This removes the `gcloud-storage → gcloud-auth → jsonwebtoken → aws-lc-rs/aws-lc-sys` path from the default dependency graph.

### Computer Hub Metrics

`xai-grok-workspace` will stop enabling the SDK `metrics` feature by default. A `hub-metrics` feature will propagate to `xai-computer-hub-sdk/metrics` and any adapter metrics feature. Metric donation setup will be conditionally compiled or return no reporter when disabled.

Computer Hub connections, authentication, MCP bridging, tool handlers, and local tool harnesses remain enabled.

### Telemetry

Remote telemetry will be feature-gated at the integration boundary. The binary will always initialize local `tracing` output, but will only initialize Sentry, OTLP exporters, telemetry-specific layers, and remote event flushes with `telemetry` enabled.

Crates that require lightweight telemetry symbols for local instrumentation must receive compatible no-op implementations when telemetry is disabled. The no-op surface preserves existing call sites and types where possible, avoids network clients and exporter dependencies, and does not transmit product analytics, crash reports, traces, logs, or metrics.

## Functional Behavior

| Build mode | GCS direct upload | Proxy/S3 upload | Computer Hub core | Hub metrics | Remote telemetry | Local tracing |
|---|---|---|---|---|---|---|
| Default local | Explicit unsupported-feature error | Available | Available | Disabled | Disabled | Available |
| `--features gcs,hub-metrics,telemetry` | Available | Available | Available | Available | Available | Available |

## Error Handling

When a user invokes a direct GCS operation in a build without `gcs`, the error must name the disabled capability and tell the user to rebuild with `--features gcs`. This is a runtime capability error, not a panic.

No-op telemetry functions must be safe to call before or after tracing initialization, and shutdown/flush functions must be idempotent.

## Testing and Verification

- Add feature-specific tests for the disabled direct-GCS error and preserve proxy/S3 upload tests.
- Compile the default package graph and assert the dependency inversions no longer find `aws-lc-rs`, `gcloud-storage`, `sentry`, `opentelemetry-otlp`, or `prometheus`.
- Compile the full feature set to prove production integrations remain buildable.
- Run focused crate tests and the workspace's relevant binary tests.

## Non-Goals

- Removing Computer Hub itself or redesigning its authentication, MCP bridge, tool handlers, or local tool harness.
- Removing local `tracing`/`tracing-subscriber` diagnostics.
- Removing proxy or S3 artifact upload paths.
- Editing `Cargo.lock` manually.
