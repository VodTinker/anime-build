# Changelog

All notable changes to **Anime (`anibuild`)** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.201] - 2026-07-27

### Fixed
- **CLI Updater Endpoint**: Updated built-in updater (`anibuild update`) to check `VodTinker/anime-build` releases and `anibuild.online` installation scripts instead of upstream xAI endpoints.

---

## [v0.2.200] - 2026-07-26

### Added
- **ChatGPT Pro / Plus & API Key Workflows**: Full support for custom OpenAI API keys and ChatGPT Pro / Plus authentication.
- **Enhanced Statusline & Context Bar**: Real-time token usage, percentage tracking, and cost estimations rendered directly in the shortcuts bar.
- **Zero-Telemetry Terminal Harness**: 100% private, local execution for terminal-native AI agent workflows.

---

## [v0.2.104] - 2026-07-25

### Added
- **ChatGPT OAuth Integration**: Default authentication flow for `anibuild` and `anime` now uses ChatGPT Plus / Pro OAuth (OpenAI Codex).
- **Auto-updating installer script**: `install.sh` dynamically queries GitHub Releases API for the latest version tag.

### Fixed
- **CI Build Performance**: Optimized release workflows for native Linux x86_64, macOS x86_64, and macOS aarch64.
- **`tikv-jemalloc-sys` configuration**: Disabled forced 64KB page size overrides to prevent compilation deadlocks.

---

## [v0.2.102] - 2026-07-24

### Added
- Initial multi-platform release distribution for Linux (`x86_64`) and macOS (`x86_64` + `aarch64`).
- TUI terminal interface with Codex Responses API support.
