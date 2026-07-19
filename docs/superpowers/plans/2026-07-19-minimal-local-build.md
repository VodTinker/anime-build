# Minimal Local Build Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the default local build omit direct GCS/AWS-LC, remote telemetry, and Computer Hub metrics while retaining the normal TUI, MCP, authentication, proxy/S3 uploads, and local tracing.

**Architecture:** Add opt-in Cargo features at the crates that own each integration. `xai-file-utils/gcs` controls direct GCS support; `xai-grok-workspace/hub-metrics` controls Prometheus metric donation; `xai-grok-telemetry/telemetry` separates remote exporters from a compatible local/no-op API. Feature propagation is explicit from the pager binary so default builds retain no remote dependency path.

**Tech Stack:** Rust 2024, Cargo feature resolution v2, `cfg(feature)`, existing Rust integration/unit tests, `cargo tree`.

## Global Constraints

- Default builds must preserve local `tracing` diagnostics and must not create remote telemetry clients.
- Default builds must not include `gcloud-storage`, `aws-lc-rs`, `sentry`, `opentelemetry-otlp`, or `prometheus` in the dependency graph.
- Proxy and S3 upload behavior must remain unchanged.
- Direct GCS requests in a no-`gcs` build must return an actionable `anyhow` error, not panic.
- Computer Hub authentication, MCP bridging, tool handlers, and local tool harnesses remain compiled and operational.
- Do not stage, edit, revert, or commit pre-existing unrelated workspace changes.

---

### Task 1: Gate direct GCS support

**Files:**
- Modify: `crates/codegen/xai-file-utils/Cargo.toml`
- Modify: `crates/codegen/xai-file-utils/src/gcs.rs`
- Modify: `crates/codegen/xai-file-utils/src/lib.rs` if the existing module declaration needs feature-aware visibility
- Test: existing unit-test module in `crates/codegen/xai-file-utils/src/gcs.rs`, or create `crates/codegen/xai-file-utils/tests/gcs_feature.rs`

**Interfaces:**
- Consumes: existing `UploadMethod::{Direct, Proxy, S3}` and public `upload_bytes`, `upload_bytes_signed`, `upload_file`, and `upload_stream` functions.
- Produces: a `gcs` opt-in feature; public uploads retain their current signatures and return `anyhow::Result<String>`.

- [ ] **Step 1: Write the failing disabled-GCS test**

Add a test compiled without `gcs` that constructs the existing direct upload configuration and checks that `upload_bytes` fails with an error containing both `direct GCS support is disabled` and `--features gcs`.

```rust
#[cfg(not(feature = "gcs"))]
#[tokio::test]
async fn direct_upload_requires_gcs_feature() {
    let config = direct_test_config();

    let error = upload_bytes(&config, "test/object", b"body", "text/plain")
        .await
        .expect_err("direct upload must be unavailable without gcs");

    assert!(error.to_string().contains("direct GCS support is disabled"));
    assert!(error.to_string().contains("--features gcs"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p xai-file-utils direct_upload_requires_gcs_feature`

Expected: FAIL because the existing direct branch attempts to compile/use `gcloud-storage` rather than returning the specified unsupported-feature error.

- [ ] **Step 3: Make `gcloud-storage` optional and add the feature**

In `crates/codegen/xai-file-utils/Cargo.toml`, declare:

```toml
[features]
default = []
gcs = ["dep:gcloud-storage"]

[dependencies]
gcloud-storage = { workspace = true, optional = true }
```

Keep every non-GCS dependency unchanged.

- [ ] **Step 4: Gate direct-GCS implementations without changing proxy/S3 behavior**

Wrap imports and helper functions that name `gcloud_storage` in `#[cfg(feature = "gcs")]`. In every public upload function, delegate direct uploads through a small helper whose no-feature implementation is:

```rust
#[cfg(not(feature = "gcs"))]
fn direct_gcs_disabled() -> anyhow::Error {
    anyhow::anyhow!(
        "direct GCS support is disabled in this build; rebuild with --features gcs"
    )
}
```

In each `UploadMethod::Direct { .. }` branch, return `Err(direct_gcs_disabled())` under `#[cfg(not(feature = "gcs"))]`; preserve the current direct operation under `#[cfg(feature = "gcs")]`. Do not alter `Proxy` or `S3` branches.

- [ ] **Step 5: Run the focused test to verify it passes**

Run: `cargo test -p xai-file-utils direct_upload_requires_gcs_feature`

Expected: PASS.

- [ ] **Step 6: Compile and test the GCS-enabled path**

Run: `cargo test -p xai-file-utils --features gcs`

Expected: PASS, including all existing direct GCS test compilation.

- [ ] **Step 7: Commit only the task files**

```bash
git add crates/codegen/xai-file-utils/Cargo.toml crates/codegen/xai-file-utils/src/gcs.rs crates/codegen/xai-file-utils/src/lib.rs
git commit -m "feat: make direct GCS uploads optional"
```

### Task 2: Gate Computer Hub Prometheus metrics

**Files:**
- Modify: `crates/codegen/xai-grok-workspace/Cargo.toml`
- Modify: `crates/codegen/xai-grok-workspace/src/handle.rs`
- Modify: only the caller(s) of `WorkspaceHandle::metric_donation_reporter` returned by repository search
- Test: existing unit tests in `crates/codegen/xai-grok-workspace/src/handle.rs`

**Interfaces:**
- Consumes: `xai-computer-hub-sdk` optional `metrics` feature and `WorkspaceHandle::metric_donation_reporter(&self, service_name) -> Option<MetricDonationPump>`.
- Produces: `xai-grok-workspace/hub-metrics`; without it, `metric_donation_reporter` retains its signature and always returns `None`.

- [ ] **Step 1: Write the failing no-feature behavior test**

Add a test that creates the existing minimal `WorkspaceHandle` fixture and asserts that metric donation is unavailable without `hub-metrics`.

```rust
#[cfg(not(feature = "hub-metrics"))]
#[tokio::test]
async fn metric_donation_is_unavailable_without_hub_metrics() {
    let handle = workspace_handle_fixture().await;

    assert!(handle.metric_donation_reporter("test-service").await.is_none());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p xai-grok-workspace metric_donation_is_unavailable_without_hub_metrics`

Expected: FAIL because the workspace currently enables the SDK `metrics` feature unconditionally and creates a metric reporter when a hub exists.

- [ ] **Step 3: Add explicit feature propagation**

Replace the unconditional SDK feature in `crates/codegen/xai-grok-workspace/Cargo.toml` with an optional feature:

```toml
[features]
default = ["sandbox-enforce"]
hub-metrics = ["xai-computer-hub-sdk/metrics"]
```

Keep `xai-computer-hub-sdk = { workspace = true }` without `features = ["metrics"]`.

- [ ] **Step 4: Preserve the public method with a no-metrics implementation**

Provide the existing method body only under `#[cfg(feature = "hub-metrics")]`, and a matching no-feature method:

```rust
#[cfg(not(feature = "hub-metrics"))]
pub async fn metric_donation_reporter(
    &self,
    _service_name: &str,
) -> Option<xai_computer_hub_sdk::MetricDonationPump> {
    None
}
```

Gate callers that directly construct or hold Prometheus-backed metric reporters so no `MetricDonationPump` implementation details are referenced without the feature.

- [ ] **Step 5: Run the focused test to verify it passes**

Run: `cargo test -p xai-grok-workspace metric_donation_is_unavailable_without_hub_metrics`

Expected: PASS.

- [ ] **Step 6: Verify both feature configurations**

Run:

```bash
cargo test -p xai-grok-workspace
cargo test -p xai-grok-workspace --features hub-metrics
```

Expected: Both commands pass.

- [ ] **Step 7: Commit only the task files**

```bash
git add crates/codegen/xai-grok-workspace/Cargo.toml crates/codegen/xai-grok-workspace/src/handle.rs
git commit -m "feat: make Computer Hub metrics optional"
```

### Task 3: Split telemetry dependencies from local telemetry compatibility

**Files:**
- Modify: `crates/codegen/xai-grok-telemetry/Cargo.toml`
- Modify: `crates/codegen/xai-grok-telemetry/src/lib.rs`
- Modify: telemetry modules that import Sentry, Mixpanel, OTLP, Prometheus, or telemetry-only HTTP exporters
- Modify: `crates/codegen/xai-grok-pager-bin/Cargo.toml`
- Modify: `crates/codegen/xai-grok-pager-bin/src/main.rs`
- Test: existing telemetry unit/integration tests under `crates/codegen/xai-grok-telemetry/tests/`
- Test: add a minimal no-feature compile or behavior test in `crates/codegen/xai-grok-telemetry/tests/`

**Interfaces:**
- Consumes: existing `xai_grok_telemetry` exported modules called from pager, shell, MCP, memory, HTTP, and renderer crates.
- Produces: `xai-grok-telemetry/telemetry` for remote exporters and a default-compatible no-op public API for all call sites that remain compiled.

- [ ] **Step 1: Inventory the public call surface before editing**

Run:

```bash
rg -n "xai_grok_telemetry::" crates/codegen --glob '*.rs' > /tmp/xai-grok-telemetry-callers.txt
rg -n "^(pub )?(mod|fn|struct|enum|trait)" crates/codegen/xai-grok-telemetry/src --glob '*.rs'
```

Classify each export used outside the crate as either local tracing support, product event API, Sentry API, OTLP API, or debug-log API. Do not change callers until their replacement API is defined.

- [ ] **Step 2: Write a failing default-feature compatibility test**

Add a test which invokes the no-op-safe lifecycle boundary expected by the pager: initialize the guard/layer entry point if applicable, emit one product event through the existing public API, then call shutdown/flush twice. It must compile and complete successfully with no telemetry feature.

```rust
#[cfg(not(feature = "telemetry"))]
#[test]
fn no_feature_telemetry_lifecycle_is_safe_and_idempotent() {
    let _guard = xai_grok_telemetry::otel_layer::otel_guard();
    xai_grok_telemetry::sentry::flush_on_shutdown();
    xai_grok_telemetry::otel_layer::shutdown_otel();
    xai_grok_telemetry::sentry::flush_on_shutdown();
    xai_grok_telemetry::otel_layer::shutdown_otel();
}
```

Adapt the test to the exact current public constructors discovered in Step 1, without introducing a test-only API.

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p xai-grok-telemetry no_feature_telemetry_lifecycle_is_safe_and_idempotent`

Expected: FAIL because the crate currently compiles remote exporter implementations and has no feature-separated no-op boundary.

- [ ] **Step 4: Add the telemetry feature and optional dependency groups**

Move product-telemetry-only dependencies into the feature definition. The feature must include the optional dependencies for Sentry, Mixpanel, OTLP, OpenTelemetry SDK/proto/http, fastrace OTel adapters, telemetry-only `tonic`/`prost` use, and remote exporter HTTP requirements. Keep local `tracing`, `tracing-subscriber`, and any modules needed solely for local logs non-optional.

Use Cargo’s `dep:` syntax so optional dependencies do not create implicit public features:

```toml
[features]
default = []
telemetry = [
    "dep:sentry",
    "dep:xai-mixpanel",
    "dep:opentelemetry",
    "dep:opentelemetry_sdk",
    "dep:opentelemetry-otlp",
    "dep:opentelemetry-http",
    "dep:opentelemetry-proto",
    "dep:tracing-opentelemetry",
    "dep:fastrace-opentelemetry",
]
```

Extend this list with every remote-only dependency identified in Step 1; do not make shared non-telemetry dependencies optional merely because telemetry also uses them.

- [ ] **Step 5: Implement feature-gated remote modules and no-op counterparts**

Use `#[cfg(feature = "telemetry")]` for the existing remote implementations. For every public lifecycle API still invoked by the binary, provide a non-telemetry counterpart with identical signatures and safe behavior:

```rust
#[cfg(not(feature = "telemetry"))]
pub fn flush_on_shutdown() {}

#[cfg(not(feature = "telemetry"))]
pub fn shutdown_otel() {}
```

For layers or guards, return an inert value compatible with existing subscriber composition. Product-event methods must discard events locally and return their current success/no-error result. Do not instantiate HTTP, Sentry, OTLP, Mixpanel, Prometheus, or exporter clients in no-feature implementations.

- [ ] **Step 6: Propagate the feature from final binary crates**

In `xai-grok-pager-bin`, add a `telemetry` feature that enables `xai-grok-telemetry/telemetry`; leave it out of default features. Update `main.rs` so remote-only layer insertion, external OTLP initialization, panic/error reporting setup, and remote shutdown flushes are only called with `#[cfg(feature = "telemetry")]`. Keep normal `tracing_subscriber` formatting/filter initialization in both builds.

Apply equivalent feature propagation to every direct consumer that needs remote-only APIs, based on the Step 1 inventory. Do not force `telemetry` through crates that need only no-op event symbols.

- [ ] **Step 7: Run no-feature compatibility test to verify it passes**

Run: `cargo test -p xai-grok-telemetry no_feature_telemetry_lifecycle_is_safe_and_idempotent`

Expected: PASS.

- [ ] **Step 8: Verify remote telemetry remains buildable**

Run:

```bash
cargo test -p xai-grok-telemetry --features telemetry
cargo check -p xai-grok-pager-bin --features telemetry
```

Expected: Both commands pass.

- [ ] **Step 9: Commit only the task files**

```bash
git add crates/codegen/xai-grok-telemetry crates/codegen/xai-grok-pager-bin
git commit -m "feat: make remote telemetry optional"
```

### Task 4: Expose and verify the minimal versus full build modes

**Files:**
- Modify: root `Cargo.toml` only if a workspace-level feature aggregation mechanism is already established; otherwise do not add one
- Modify: `README.md` or the existing developer build documentation to state exact commands
- Test: no new source test required; verification is Cargo graph and compile commands

**Interfaces:**
- Consumes: `xai-file-utils/gcs`, `xai-grok-workspace/hub-metrics`, and `xai-grok-telemetry/telemetry` features.
- Produces: documented default-local and full-feature build commands.

- [ ] **Step 1: Write the expected build-mode documentation assertions**

Add a concise developer-facing section stating the exact intended commands:

```bash
# Minimal local build
cargo build -p xai-grok-pager-bin

# Full integrations build
cargo build -p xai-grok-pager-bin --features telemetry,gcs,hub-metrics
```

If the binary cannot directly propagate `gcs` or `hub-metrics`, define named pager-bin forwarding features with the same names and use those commands. Do not document feature names that Cargo cannot resolve from the selected package.

- [ ] **Step 2: Run default dependency graph checks and verify they pass**

Run each command and require a non-zero result (the package must be absent):

```bash
! cargo tree -p xai-grok-pager-bin -i aws-lc-rs
! cargo tree -p xai-grok-pager-bin -i gcloud-storage
! cargo tree -p xai-grok-pager-bin -i sentry
! cargo tree -p xai-grok-pager-bin -i opentelemetry-otlp
! cargo tree -p xai-grok-pager-bin -i prometheus
```

Expected: Every reverse lookup reports that the package is not present in the dependency tree.

- [ ] **Step 3: Build and test the minimal configuration**

Run:

```bash
cargo build -p xai-grok-pager-bin
cargo test -p xai-file-utils
cargo test -p xai-grok-workspace
cargo test -p xai-grok-telemetry
```

Expected: All commands pass.

- [ ] **Step 4: Build and test the full integration configuration**

Run:

```bash
cargo build -p xai-grok-pager-bin --features telemetry,gcs,hub-metrics
cargo test -p xai-file-utils --features gcs
cargo test -p xai-grok-workspace --features hub-metrics
cargo test -p xai-grok-telemetry --features telemetry
```

Expected: All commands pass.

- [ ] **Step 5: Inspect the diff and workspace status**

Run:

```bash
git diff --check
git diff --cached --check
git status --short
```

Expected: No whitespace errors. Confirm only intended files are staged before committing; pre-existing unrelated files remain untouched.

- [ ] **Step 6: Commit only documentation and intentional build wiring**

```bash
git add README.md crates/codegen/xai-grok-pager-bin/Cargo.toml
# Add another build-documentation file only if that is where the commands were documented.
git commit -m "docs: document minimal local build"
```
