# ANIBUILD

> **Terminal-native AI coding agent harness**  
> Codebase-aware, zero-telemetry, local execution with ChatGPT Pro/Plus & custom API key support.

![Release Version](https://img.shields.io/badge/version-v0.2.201-blue.svg)
![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-darkgray.svg)
![Telemetry](https://img.shields.io/badge/telemetry-zero-purple.svg)

---

## Overview

**Anibuild** ([anibuild.online](https://anibuild.online)) is a terminal-native AI coding agent designed to inspect codebase structure, edit files, and execute shell commands directly within your terminal, editor, or CI/CD environment. 

Inspired by terminal agent harnesses, Anibuild is engineered for complete privacy: it runs 100% locally with zero telemetry, zero remote code tracking, and direct process visibility.

---

## Quick Installation (macOS & Linux)

Install or upgrade to `v0.2.201` via the official one-liner script:

```bash
curl -fsSL https://anibuild.online/install.sh | sh
```

*Note: Windows is supported via WSL (Windows Subsystem for Linux) or PowerShell (`install.ps1`).*

### Installer Options & Flags

```bash
# Force reinstallation or overwrite existing binaries
curl -fsSL https://anibuild.online/install.sh | sh -s -- --force

# Perform a dry-run simulation without making disk changes
curl -fsSL https://anibuild.online/install.sh | sh -s -- --dry-run
```

---

## Key Features

| Feature | Description |
| :--- | :--- |
| **Codebase Awareness** | Maps repository structure, symbols, imports, and dependencies prior to proposing edits. |
| **ChatGPT & API Key Workflows** | Authenticate via your **ChatGPT Pro / Plus** subscription or connect custom API keys. |
| **Zero Telemetry** | 100% local execution. Prompts, diffs, and code context are never collected or stored. |
| **Terminal & Headless Modes** | Interactive full-screen TUI, pipe/headless mode for scripts, and Agent Client Protocol (ACP) server. |
| **Voice Dictation** | Native voice input integration directly within the terminal prompt interface. |
| **Self-Updater** | Built-in updater command (`anibuild update`) fetching verified releases dynamically from GitHub. |

---

## Usage Guide

### 1. Interactive Mode (TUI)

Launch the full-screen terminal interface:

```bash
anibuild
```

*(The `anime` command alias is also installed for backward compatibility).*

### 2. Provider Setup & Authentication

Run the interactive setup wizard to configure models, API keys, or ChatGPT OAuth authentication:

```bash
anibuild provider
```

Supported providers:
- **ChatGPT Pro / Plus** (OpenAI Codex OAuth)
- **OpenAI API** (`gpt-4o`, `o1`, `o3-mini`)
- **Anthropic API** (`claude-3-5-sonnet`, `claude-3-7-sonnet`)
- **DeepSeek API** (`deepseek-coder`, `deepseek-r1`)
- **Ollama / Local LLMs** (Self-hosted endpoints)
- **OpenRouter** & custom OpenAI-compatible endpoints

### 3. Headless Mode (CLI & CI/CD Pipelines)

Run non-interactively in shell scripts or automated CI runners:

```bash
anibuild --headless --prompt "Run tests and fix any broken assertions"
```

### 4. Updating Anibuild

Check and update to the latest release at any time:

```bash
anibuild update
```

---

## CLI Command Reference

| Command | Action |
| :--- | :--- |
| `anibuild` | Launches the interactive terminal user interface (TUI). |
| `anibuild provider` | Interactive model provider and authentication manager. |
| `anibuild update` | Checks GitHub Releases and updates the binary. |
| `anibuild --version` | Displays compiled version, commit hash, and release channel. |
| `anibuild --help` | Displays available CLI flags and arguments. |

---

## Building from Source

### Prerequisites

- **Rust toolchain**: `rustc` and `cargo` (v1.85+)
- **System dependencies**: `clang`, `lld`, `cmake`, and `protobuf-compiler`

### Compilation Steps

```bash
# 1. Clone repository
git clone https://github.com/VodTinker/anime-build.git
cd anime-build

# 2. Build release binary
cargo build -p xai-grok-pager-bin --bin anibuild --release
```

The compiled binary will be placed at `target/release/anibuild`.

---

## Architecture & Privacy Model

- **Local Execution**: All workspace indexing, file modifications, and process executions take place locally on your machine.
- **Strict Process Scoping**: Shell commands executed by the agent inherit your local environment permissions and are surfaced with full process visibility.
- **Zero Data Collection**: No telemetry, analytics, or usage metrics are gathered or sent.

---

## License

Distributed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for full details.
