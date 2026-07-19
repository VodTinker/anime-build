//! Compatibility API for telemetry that is permanently disabled in Anime.

pub mod config {
    pub const ENV_MASTER_SWITCH: &str = "GROK_EXTERNAL_OTEL";

    #[derive(Debug, Clone, Default)]
    pub struct ExternalClientInfo {
        pub service_version: String,
        pub client_version: String,
        pub app_entrypoint: String,
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExternalOtelConfig {
    pub client: config::ExternalClientInfo,
    pub internal_pipeline_consumed_otel_vars: bool,
}

impl ExternalOtelConfig {
    pub fn resolve_with(
        _getenv: impl Fn(&str) -> Option<String>,
        _file: Option<&ExternalOtelFileConfig>,
    ) -> Option<Self> {
        None
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ExternalOtelFileConfig {
    pub enabled: Option<bool>,
    pub metrics_exporter: Option<String>,
    pub logs_exporter: Option<String>,
    pub endpoint: Option<String>,
    pub protocol: Option<String>,
    pub log_user_prompts: Option<bool>,
    pub log_tool_details: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalOtelRemotePolicy {
    pub force_disable: bool,
    pub lock_content_gates: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IdentityAttrs;

impl IdentityAttrs {
    pub fn from_snapshot<T>(_snapshot: &T) -> Self {
        Self
    }
}

pub fn init(_config: Option<ExternalOtelConfig>) {}
pub fn is_active() -> bool { false }
pub fn emit<T: crate::events::TelemetryEvent>(_data: &T) {}
pub fn set_identity(_attrs: IdentityAttrs) {}
pub fn apply_remote_policy(_policy: ExternalOtelRemotePolicy) {}
pub fn flush() {}
pub fn shutdown() {}

pub mod schema {
    #[derive(Debug, Clone)]
    pub struct ExternalRecord;

    macro_rules! noop_mappers {
        ($($name:ident),+ $(,)?) => {
            $(pub fn $name<T>(_event: &T) -> Option<ExternalRecord> { None })+
        };
    }

    noop_mappers!(
        map_auth,
        map_plan_mode_toggled,
        map_contextual_tip,
        map_yolo_toggled,
        map_tool_decision,
        map_compaction,
        map_subagent_launched,
        map_subagent_completed,
        map_model_switched,
        map_plugin_installed,
        map_plugin_used,
        map_skill_activated,
        map_mcp_server_connected,
        map_mcp_server_failed,
        map_session_start,
        map_session_new,
        map_user_prompt,
        map_turn_completed,
        map_tool_result,
        map_api_request,
        map_session_end,
        map_rate_limit_hit,
        map_api_error,
        map_internal_error,
    );
}
