//! Local-only telemetry compatibility surface.
//!
//! Anime keeps structured local tracing, but never sends product analytics,
//! Mixpanel events, or session metrics to a remote service. These no-op APIs
//! preserve the host integration boundary without retaining those clients.

use chrono::{Local, SecondsFormat};

use crate::config::{TelemetryConfig, TelemetryMode};
use crate::http::OriginClientInfo;

/// Event property map retained for call-site compatibility.
pub type Metadata = serde_json::Map<String, serde_json::Value>;

/// Marker retained for integrations that previously held an analytics client.
#[derive(Clone, Debug, Default)]
pub struct TelemetryClient;

impl TelemetryClient {
    #[allow(clippy::too_many_arguments)]
    pub fn from_config(
        _config: TelemetryConfig,
        _mode: TelemetryMode,
        _user_id: Option<String>,
        _team_id: Option<String>,
        _deployment_key: Option<String>,
        _origin_client: Option<OriginClientInfo>,
        _shell_version: String,
        _subscription_tier: Option<String>,
        _http_client: reqwest::Client,
    ) -> Self {
        Self
    }
}

/// Product analytics is permanently disabled in Anime.
pub fn is_enabled() -> bool {
    false
}

/// Remote session metrics are permanently disabled in Anime.
pub fn is_session_metrics_enabled() -> bool {
    false
}

/// Local context shape retained for callers that construct telemetry events.
pub struct UserContext {
    pub country: String,
    pub language: String,
    pub timestamp: String,
}

impl UserContext {
    pub fn collect() -> Self {
        Self {
            country: "local".to_owned(),
            language: "local".to_owned(),
            timestamp: Local::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        }
    }
}

/// Deliberately drops product analytics events.
pub async fn track(_event_name: &str, _request_id: &str, _ctx: &UserContext, _metadata: Metadata) {}

/// Deliberately drops Mixpanel profile updates.
pub fn sync_profile() {}

/// Accepts existing host configuration while permanently disabling remote
/// analytics. Keeping the signature avoids coupling callers to this policy.
#[allow(clippy::too_many_arguments)]
pub fn init(
    _config: TelemetryConfig,
    _mode: TelemetryMode,
    _user_id: Option<String>,
    _team_id: Option<String>,
    _deployment_key: Option<String>,
    _origin_client: Option<OriginClientInfo>,
    _shell_version: String,
    _subscription_tier: Option<String>,
    _http_client: reqwest::Client,
) {
}

/// Same no-op policy as [`init`].
#[allow(clippy::too_many_arguments)]
pub fn init_if_needed(
    _config: TelemetryConfig,
    _mode: TelemetryMode,
    _user_id: Option<String>,
    _team_id: Option<String>,
    _deployment_key: Option<String>,
    _origin_client: Option<OriginClientInfo>,
    _shell_version: String,
    _subscription_tier: Option<String>,
    _http_client: reqwest::Client,
) {
}
