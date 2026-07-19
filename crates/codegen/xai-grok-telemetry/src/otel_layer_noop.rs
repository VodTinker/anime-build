//! Local tracing compatibility layer.
//!
//! The former OTLP exporter is intentionally absent. This module keeps host
//! setup code source-compatible while contributing no remote exporter.

use std::sync::Arc;

use xai_grok_auth::AuthCredentialProvider;

pub struct OtelLayerConfig {
    pub credentials: Arc<dyn AuthCredentialProvider>,
    pub token_header_value: String,
    pub alpha_test_key: Option<String>,
    pub exporter: OtelExporterConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct OtelClientInfo {
    pub client_name: &'static str,
    pub client_version: &'static str,
    pub service_version: &'static str,
    pub app_entrypoint: &'static str,
}

#[derive(Debug, Default, Clone)]
pub struct OtelExporterConfig {
    pub traces_url: String,
    pub extra_headers: Vec<(String, String)>,
    pub export_interval: Option<std::time::Duration>,
    pub timeout: Option<std::time::Duration>,
    pub enabled: bool,
}

#[derive(Default)]
pub struct NoopLayer;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for NoopLayer {}

pub fn build_otel_layer<S>(_client: OtelClientInfo, _config: OtelLayerConfig) -> NoopLayer
where
    S: tracing::Subscriber,
{
    NoopLayer
}

pub fn otel_guard() {}
pub fn shutdown_otel() {}
