//! ChatGPT Plus/Pro OAuth for the Codex Responses endpoint.
//!
//! This provider deliberately owns a separate token file. Do not route it through
//! `AuthManager`: that manager reads and writes xAI's `auth.json` and performs
//! xAI-specific enrichment requests.

use std::path::{Path, PathBuf};

use axum::{
    Router,
    extract::{Query, State},
    response::Html,
    routing::get,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use xai_grok_sampler::{AuthScheme, SamplerConfig};
use xai_grok_sampling_types::{ApiBackend, ReasoningEffort};

/// Models available through the ChatGPT Plus/Pro Codex OAuth route.
///
/// The aliases are the stable names surfaced by Anime's `/model` picker; the
/// model IDs are the values sent to ChatGPT's Codex Responses endpoint.
pub const CODEX_MODELS: [(&str, &str, ReasoningEffort); 3] = [
    ("codex-terra", "gpt-5.6-terra", ReasoningEffort::Medium),
    ("codex-sol", "gpt-5.6-sol", ReasoningEffort::High),
    ("codex-luna", "gpt-5.6-luna", ReasoningEffort::Low),
];

pub fn codex_models() -> &'static [(&'static str, &'static str, ReasoningEffort)] {
    &CODEX_MODELS
}

/// Public OAuth client used by the official Codex CLI.
pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
pub const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
pub const RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex";
pub const DEFAULT_MODEL: &str = "gpt-5.6-terra";
const TOKEN_FILE: &str = "openai-codex.json";
const CALLBACK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

#[derive(Debug, thiserror::Error)]
pub enum OpenAiCodexError {
    #[error("failed to bind the OpenAI OAuth callback server: {0}")]
    Bind(#[source] std::io::Error),
    #[error("OpenAI OAuth callback timed out after 10 minutes")]
    Timeout,
    #[error("OpenAI OAuth callback did not include an authorization code")]
    MissingCode,
    #[error("OpenAI OAuth state mismatch")]
    StateMismatch,
    #[error("OpenAI OAuth authorization failed: {0}")]
    Authorization(String),
    #[error("OpenAI OAuth token request failed: HTTP {status}: {body}")]
    Token { status: u16, body: String },
    #[error("OpenAI OAuth token response did not identify a ChatGPT account")]
    MissingAccount,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credentials {
    access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at: Option<DateTime<Utc>>,
    account_id: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("has_access_token", &!self.access_token.is_empty())
            .field("has_refresh_token", &self.refresh_token.is_some())
            .field("expires_at", &self.expires_at)
            .field("account_id", &self.account_id)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug)]
struct Callback {
    code: String,
    state: String,
}

pub fn default_path() -> anyhow::Result<PathBuf> {
    Ok(crate::util::grok_home::grok_home().join(TOKEN_FILE))
}

/// Whether Anime has a usable local ChatGPT/Codex OAuth credential.
///
/// This is intentionally separate from the inherited Grok `AuthManager`: the
/// two flows use different token files and authentication protocols.
pub fn is_logged_in() -> bool {
    default_path()
        .ok()
        .and_then(|path| load(&path).ok())
        .flatten()
        .is_some()
}

pub fn load(path: &Path) -> anyhow::Result<Option<Credentials>> {
    match std::fs::read(path) {
        Ok(bytes) => {
            xai_grok_shell_base::util::secure_file::ensure_owner_only_permissions(path)?;
            Ok(Some(serde_json::from_slice(&bytes)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn save(path: &Path, credentials: &Credentials) -> anyhow::Result<()> {
    let encoded = serde_json::to_vec_pretty(credentials)?;
    xai_grok_shell_base::util::secure_file::write_secure_file(path, &encoded)?;
    Ok(())
}

pub fn sampler_config(credentials: &Credentials, model: Option<&str>) -> SamplerConfig {
    let mut extra_headers = IndexMap::new();
    extra_headers.insert("chatgpt-account-id".into(), credentials.account_id.clone());
    extra_headers.insert("originator".into(), "codex_cli_rs".into());
    extra_headers.insert("session_id".into(), uuid::Uuid::now_v7().to_string());
    SamplerConfig {
        api_key: Some(credentials.access_token.clone()),
        base_url: RESPONSES_URL.into(),
        // An explicit model wins, followed by the user's launch-time choice.
        model: resolve_model(model),
        api_backend: ApiBackend::Responses,
        auth_scheme: AuthScheme::Bearer,
        extra_headers,
        context_window: 200_000,
        ..Default::default()
    }
}

fn resolve_model(model: Option<&str>) -> String {
    model
        .map(str::to_owned)
        .or_else(|| std::env::var("ANIME_CODEX_MODEL").ok())
        .unwrap_or_else(|| DEFAULT_MODEL.to_owned())
}

/// Load the isolated credential and refresh it before it becomes stale.
/// Callers should use this instead of `load` before creating a sampler client.
pub async fn load_fresh(path: &Path) -> anyhow::Result<Option<Credentials>> {
    let Some(mut credentials) = load(path)? else {
        return Ok(None);
    };
    if credentials
        .expires_at
        .is_some_and(|expires_at| expires_at <= Utc::now() + Duration::minutes(5))
    {
        refresh(path, &mut credentials).await?;
    }
    Ok(Some(credentials))
}

/// Build a ready-to-use Codex sampler configuration from the isolated store.
/// Returns `None` when the user has not run `grok login --openai`.
pub async fn load_sampler_config(
    path: &Path,
    model: Option<&str>,
) -> anyhow::Result<Option<SamplerConfig>> {
    Ok(load_fresh(path)
        .await?
        .as_ref()
        .map(|credentials| sampler_config(credentials, model)))
}

/// Synchronous variant for the model resolver, which runs before a Tokio
/// context is available. Refreshes still happen on the async session path.
pub fn load_sampler_config_sync(
    path: &Path,
    model: Option<&str>,
) -> anyhow::Result<Option<SamplerConfig>> {
    Ok(load(path)?
        .as_ref()
        .map(|credentials| sampler_config(credentials, model)))
}

/// Refresh in place and persist only after a successful token exchange.
/// A rotated refresh token replaces the old value; providers that omit one keep it.
pub async fn refresh(path: &Path, credentials: &mut Credentials) -> anyhow::Result<()> {
    let refresh_token = credentials.refresh_token.as_deref().ok_or_else(|| {
        anyhow::anyhow!("OpenAI credentials have no refresh token; run `grok login --openai`")
    })?;
    let response = token_request(&[
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
    ])
    .await?;
    apply_token_response(credentials, response)?;
    save(path, credentials)
}

/// Open a browser PKCE login and save the resulting credentials in the isolated store.
pub async fn login_default() -> anyhow::Result<()> {
    let path = default_path()?;
    login(&path).await
}

pub async fn login(path: &Path) -> anyhow::Result<()> {
    let verifier = random_urlsafe(48);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state = random_urlsafe(24);
    // OpenAI's public Codex OAuth client only registers this loopback callback.
    let listener = TcpListener::bind(("127.0.0.1", 1455))
        .await
        .map_err(OpenAiCodexError::Bind)?;
    let redirect_uri = "http://localhost:1455/auth/callback";
    let authorize_url = reqwest::Url::parse_with_params(
        AUTHORIZE_URL,
        [
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", redirect_uri),
            ("scope", "openid profile email offline_access"),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("state", state.as_str()),
            ("id_token_add_organizations", "true"),
            ("codex_cli_simplified_flow", "true"),
            ("originator", "grok-build"),
        ],
    )?;
    eprintln!("Signing in with ChatGPT for Codex...");
    if let Err(error) = webbrowser::open(authorize_url.as_str()) {
        tracing::debug!(%error, "failed to open OpenAI OAuth browser");
    }
    eprintln!("Open this URL to sign in:\n  {authorize_url}");

    let callback = wait_for_callback(listener).await?;
    if callback.state != state {
        return Err(OpenAiCodexError::StateMismatch.into());
    }
    let response = token_request(&[
        ("grant_type", "authorization_code"),
        ("code", callback.code.as_str()),
        ("redirect_uri", redirect_uri),
        ("client_id", CLIENT_ID),
        ("code_verifier", verifier.as_str()),
    ])
    .await?;
    let credentials = credentials_from_token_response(response)?;
    save(path, &credentials)?;
    eprintln!("ChatGPT Codex login complete.");
    Ok(())
}

/// Run the isolated Codex health check for the current local login.
pub async fn preflight_default() -> anyhow::Result<()> {
    let path = default_path()?;
    let credentials = load_fresh(&path)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Not signed in. Run `anime login --openai`."))?;
    preflight(&credentials).await
}

/// Verify the OAuth token and selected Codex model before entering the TUI.
/// The request contains only this fixed health-check prompt and no workspace data.
async fn preflight(credentials: &Credentials) -> anyhow::Result<()> {
    let response = crate::http::shared_client()
        .post(format!("{RESPONSES_URL}/responses"))
        .bearer_auth(&credentials.access_token)
        .header("chatgpt-account-id", &credentials.account_id)
        .header("originator", "anime")
        .header("session-id", uuid::Uuid::now_v7().to_string())
        .header("accept", "text/event-stream")
        .json(&serde_json::json!({
            "model": resolve_model(None),
            "input": [{
                "role": "user",
                "content": [{"type": "input_text", "text": "Reply with OK."}]
            }],
            "store": false,
            "stream": true,
        }))
        .send()
        .await?;
    if response.status().is_success() {
        return Ok(());
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    anyhow::bail!("Codex preflight failed (HTTP {status}): {body}")
}

async fn wait_for_callback(listener: TcpListener) -> anyhow::Result<Callback> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let app = Router::new()
        .route("/auth/callback", get(callback))
        .with_state(tx);
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let result = tokio::time::timeout(CALLBACK_TIMEOUT, rx.recv())
        .await
        .map_err(|_| OpenAiCodexError::Timeout)?
        .ok_or(OpenAiCodexError::MissingCode)?;
    server.abort();
    result.map_err(Into::into)
}

async fn callback(
    State(tx): State<tokio::sync::mpsc::Sender<Result<Callback, OpenAiCodexError>>>,
    Query(query): Query<CallbackQuery>,
) -> Html<&'static str> {
    let result = match query.error {
        Some(error) => Err(OpenAiCodexError::Authorization(
            match query.error_description {
                Some(description) => format!("{error}: {description}"),
                None => error,
            },
        )),
        None => match query.code {
            Some(code) => Ok(Callback {
                code,
                state: query.state.unwrap_or_default(),
            }),
            None => Err(OpenAiCodexError::MissingCode),
        },
    };
    let success = result.is_ok();
    let _ = tx.send(result).await;
    Html(if success {
        "<h1>Signed in</h1><p>You can return to Grok.</p>"
    } else {
        "<h1>Sign-in failed</h1><p>You can close this window and try again.</p>"
    })
}

async fn token_request(params: &[(&str, &str)]) -> anyhow::Result<TokenResponse> {
    let response = crate::http::shared_client()
        .post(TOKEN_URL)
        .form(params)
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(OpenAiCodexError::Token {
            status: status.as_u16(),
            body,
        }
        .into());
    }
    Ok(serde_json::from_str(&body)?)
}

fn credentials_from_token_response(response: TokenResponse) -> anyhow::Result<Credentials> {
    let account_id = response
        .id_token
        .as_deref()
        .and_then(account_id_from_jwt)
        .or_else(|| account_id_from_jwt(&response.access_token))
        .ok_or(OpenAiCodexError::MissingAccount)?;
    Ok(Credentials {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_at: response
            .expires_in
            .map(|seconds| Utc::now() + Duration::seconds(seconds)),
        account_id,
    })
}

fn apply_token_response(
    credentials: &mut Credentials,
    response: TokenResponse,
) -> anyhow::Result<()> {
    let replacement = credentials_from_token_response(response)?;
    credentials.access_token = replacement.access_token;
    credentials.expires_at = replacement.expires_at;
    if replacement.refresh_token.is_some() {
        credentials.refresh_token = replacement.refresh_token;
    }
    credentials.account_id = replacement.account_id;
    Ok(())
}

fn account_id_from_jwt(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value
        .get("chatgpt_account_id")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            value
                .get("https://api.openai.com/auth")
                .and_then(|auth| auth.get("chatgpt_account_id"))
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            value
                .get("organizations")
                .and_then(serde_json::Value::as_array)
                .and_then(|organizations| organizations.first())
                .and_then(|organization| organization.get("id"))
                .and_then(serde_json::Value::as_str)
        })
        .map(str::to_owned)
}

fn random_urlsafe(bytes: usize) -> String {
    let mut raw = vec![0; bytes];
    rand::fill(&mut raw[..]);
    URL_SAFE_NO_PAD.encode(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_is_distinct_from_xai_auth_json() {
        assert_ne!(TOKEN_FILE, "auth.json");
        assert_eq!(Path::new(TOKEN_FILE).file_name().unwrap(), TOKEN_FILE);
    }

    #[test]
    fn extracts_chatgpt_account_from_jwt_claim() {
        let payload = URL_SAFE_NO_PAD
            .encode(r#"{"https://api.openai.com/auth":{"chatgpt_account_id":"acct_123"}}"#);
        assert_eq!(
            account_id_from_jwt(&format!("a.{payload}.c")),
            Some("acct_123".into())
        );
    }

    #[test]
    fn codex_sampler_config_has_required_route_and_headers() {
        let credentials = Credentials {
            access_token: "access".into(),
            refresh_token: Some("refresh".into()),
            expires_at: None,
            account_id: "acct_123".into(),
        };
        let config = sampler_config(&credentials, None);
        assert_eq!(config.base_url, RESPONSES_URL);
        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.api_backend, ApiBackend::Responses);
        assert_eq!(config.auth_scheme, AuthScheme::Bearer);
        assert_eq!(
            config.extra_headers.get("chatgpt-account-id"),
            Some(&"acct_123".into())
        );
        assert_eq!(
            config.extra_headers.get("originator"),
            Some(&"codex_cli_rs".into())
        );
        assert!(config.extra_headers.contains_key("session_id"));
    }

    #[test]
    fn codex_models_expose_only_the_supported_chatgpt_variants_and_efforts() {
        let models = codex_models();
        assert_eq!(models.len(), 3);
        assert_eq!(
            models,
            [
                ("codex-terra", "gpt-5.6-terra", ReasoningEffort::Medium),
                ("codex-sol", "gpt-5.6-sol", ReasoningEffort::High),
                ("codex-luna", "gpt-5.6-luna", ReasoningEffort::Low),
            ]
        );
    }
}
