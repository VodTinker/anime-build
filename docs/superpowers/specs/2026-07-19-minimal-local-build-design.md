# Minimal Local Build Design

## Goal

Keep Anime's local coding workflow while permanently removing cloud-upload and remote-observability dependency trees that make local compilation unnecessarily expensive.

## Included capabilities

- Interactive TUI, ACP, MCP, ChatGPT/Codex OAuth, agents, terminal tools, and normal local files.
- Image generation and image editing.
- PDF and PowerPoint reading.
- Proxy-configured uploads.
- Local `tracing`, local diagnostics, and Computer Hub's core tool/session functionality.

## Permanently removed capabilities

| Area | Removed behavior | Replacement / resulting behavior |
|---|---|---|
| Cloud storage | GCS, S3, AWS SDK/auth/signing, direct object uploads | `UploadMethod::Direct` and legacy `UploadMethod::S3` return a clear error directing users to proxy upload. |
| Video | Video generation and image-to-video/reference-video tools | Image generation and editing remain available. |
| Remote telemetry | Sentry, Mixpanel, OTLP exporters, external event streams, remote trace propagation | Compatible no-op lifecycle APIs; local tracing remains. |
| Computer Hub donation | Trace, log, and Prometheus metric donation | Hub connections, auth, MCP bridge, handlers, and local harness remain. |

No feature re-enables these removed paths. This is deliberately a smaller local product, not a feature-gated full distribution.

## Cargo performance policy

The development profile remains incremental and uses many codegen units; it must not enable dev LTO or reduce codegen units. Linux links through `clang`/`lld`.

`.cargo/config.toml` uses `scripts/cargo-rustc-wrapper`. When `sccache` is installed, the wrapper provides a local compiler cache across clean target directories and branch switches; without it, it invokes `rustc` normally. No shared or remote cache is required.

## Verification

Do not edit `Cargo.lock` manually. After source changes, validate the dependency graph and focused packages from a developer machine. The graph must not retain `gcloud-storage`, AWS SDK crates, `sentry`, `xai-mixpanel`, OTLP/OpenTelemetry exporters, or Computer Hub donation modules.
