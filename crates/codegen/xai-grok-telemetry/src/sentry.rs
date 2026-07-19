//! Sentry compatibility API with remote error reporting disabled.

/// Retained host configuration shape. No value is sent or persisted.
pub struct Config {
    pub client: &'static str,
    pub client_version: &'static str,
    pub release: &'static str,
    pub disabled: bool,
}

/// Process-lifetime no-op guard.
#[derive(Debug, Default)]
pub struct ClientInitGuard;

/// Remote crash reporting is permanently disabled in Anime.
pub fn init(_config: Config) -> ClientInitGuard {
    ClientInitGuard
}

/// Nothing to flush because no remote events are created.
pub fn flush_on_shutdown() {}
