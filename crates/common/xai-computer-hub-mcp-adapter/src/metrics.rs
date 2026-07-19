//! Local no-op metrics hooks for the MCP adapter bridge.
//!
//! Anime does not include Prometheus metrics or donation paths. Keeping these
//! small functions preserves the bridge's instrumentation call sites without
//! registering or exporting metric data.

pub(crate) fn mcp_call_duration_observe(_secs: f64) {}
pub(crate) fn mcp_error() {}
pub(crate) fn mcp_tools_bridged_set(_count: i64) {}
