//! Local diagnostics for Anime sessions.
//!
//! Structured tracing and local debug logs remain available. Product analytics,
//! Sentry, Mixpanel, and OTLP exporters are intentionally disabled.

mod appender;
pub mod client;
pub mod config;
pub mod context;
pub mod debug_log;
pub mod enums;
pub mod events;
#[path = "external_noop.rs"]
pub mod external;
pub mod hooks_log;
pub mod http;
pub mod id;
pub mod instrumentation;
pub mod memory_log;
pub mod memory_telemetry;
#[path = "otel_layer_noop.rs"]
pub mod otel_layer;
pub mod prompt_timing;
pub(crate) mod redact_common;
pub mod sampling_log;
pub mod sentry;
pub mod session_ctx;
pub mod session_metrics;
pub mod unified_log;

pub use client::{
    Metadata, TelemetryClient, UserContext, init, init_if_needed, is_enabled,
    is_session_metrics_enabled,
};
pub use events::TelemetryEvent;
pub use session_ctx::{
    EmitterOrigin, TelemetryCtx, emit_event, emit_event_with_origin, log_event, log_session_event,
    log_session_event_with_origin, with_session_ctx,
};

/// Compile-time privacy policy for this distribution.
pub fn data_collection_enabled() -> bool {
    false
}
