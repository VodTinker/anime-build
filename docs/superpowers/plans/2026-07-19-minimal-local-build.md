# Minimal Local Build Implementation Plan

## Objective

Produce a permanently leaner Anime build while retaining the local coding experience and document-reading support.

## Constraints

- Preserve TUI, ACP, MCP, ChatGPT/Codex OAuth, agents, terminal and local file tools.
- Preserve PDF/PPTX reading and image generation/editing.
- Preserve proxy uploads.
- Do not retain opt-in features for GCS, S3/AWS, video generation, remote telemetry, or Computer Hub donation.
- Keep local Rust `tracing`.
- Never manually edit `Cargo.lock`.

## Work items

1. **Cloud storage**
   - Remove GCS and AWS dependencies and implementation modules.
   - Keep legacy upload configuration parsing compatible.
   - Make direct and S3 methods return an actionable proxy-upload error.

2. **Video generation**
   - Remove the video tool module, registry entries, types, session configuration, UI handling, skills, and documentation.
   - Keep `image_gen` and `image_edit`.

3. **Remote telemetry**
   - Remove Mixpanel, Sentry, OTLP, OpenTelemetry exporter, and remote trace-context dependencies.
   - Replace only necessary public lifecycle boundaries with local no-ops.
   - Remove OTLP-specific tests and external configuration paths.

4. **Computer Hub donation**
   - Remove trace, log, and metric donation pumps and Prometheus metric-donation code.
   - Preserve normal Computer Hub transport, auth, MCP bridge, session handlers, and local tool harness.

5. **Cargo performance**
   - Preserve the fast incremental development profile and `lld` linker configuration.
   - Configure an optional local `sccache` wrapper that safely falls back to `rustc` when unavailable.

6. **Documentation and review**
   - Update README and this design documentation to accurately describe the local-only product.
   - Run whitespace/static-reference reviews and leave small, user-run Cargo verification commands.

## Success criteria

The package graph has no GCS/AWS SDK, video, Sentry, Mixpanel, OTLP/OpenTelemetry exporter, or Hub donation paths. Proxy uploads, document reading, image tools, local diagnostics, and the normal Anime coding workflow remain represented in the codebase.
