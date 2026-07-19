use reqwest::header::HeaderMap;

/// Distributed trace propagation is unavailable in Anime's local-only build.
pub fn current_traceparent() -> Option<String> {
    None
}

/// Preserve request construction while intentionally adding no remote trace
/// headers.
pub fn inject_trace_context_into_request(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder
}

/// Callers use this when building middleware requests. The local-only build
/// never adds trace propagation headers.
pub(crate) fn trace_context_headers() -> HeaderMap {
    HeaderMap::new()
}

/// Compatibility no-op for callers holding a mutable header map.
pub(crate) fn inject_trace_context(_headers: &mut HeaderMap) {}

/// Creates the normal local tracing span without adopting a remote parent.
pub fn span_from_meta_traceparent(
    _meta: &serde_json::Map<String, serde_json::Value>,
) -> tracing::Span {
    tracing::info_span!("acp_dispatch")
}

/// Remote parent propagation is disabled in the local-only build.
pub fn link_current_span_to_meta(_meta: &serde_json::Value) {}
