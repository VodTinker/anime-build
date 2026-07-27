# ANIBUILD

> **Terminal-native AI coding agent harness**  
> Codebase-aware, zero-telemetry, local execution with ChatGPT Pro/Plus & custom API key support.

![Release Version](https://img.shields.io/badge/version-v0.2.201-blue.svg)
![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-darkgray.svg)
![Telemetry](https://img.shields.io/badge/telemetry-zero-purple.svg)

---

## Overview

**Anibuild** is a terminal-native AI agent designed to inspect codebase structure, edit files, and execute shell commands directly from your terminal or CI environment. Built for privacy and performance, Anibuild runs 100% locally with zero telemetry and zero remote code storage.

---

## Quick Installation (macOS & Linux)

Install the latest version (`v0.2.201`) via the one-line shell installer:

```bash
curl -fsSL https://anibuild.online/install.sh | sh
```

*Note: Windows is not natively supported. Use WSL, Linux, or macOS.*

### Installer Flags

```bash
# Force reinstallation / overwrite existing binary
curl -fsSL https://anibuild.online/install.sh | sh -s -- --force

# Dry-run simulation (no disk changes)
curl -fsSL https://anibuild.online/install.sh | sh -s -- --dry-run
```

---

## Key Capabilities

- **Codebase-Aware**: Maps repository structure, dependencies, and code patterns prior to executing edits.
- **Flexible Provider Workflows**: Seamlessly authenticate via your **ChatGPT Pro / Plus** subscription or supply custom **API keys** (OpenAI, Anthropic, DeepSeek, Ollama, OpenRouter).
- **Zero Telemetry**: Fully local process execution. Your code and prompts never leave your local workspace.
- **Terminal & CI Native**: Runs interactively as a TUI or headlessly inside CI/CD scripts and Agent Client Protocol (ACP) workflows.
- **Built-in Voice Dictation**: Hands-free interaction directly within the terminal prompt.

---

## Usage

Start the agent:

```bash
anibuild
```

*(The `anime` binary is also installed for backward compatibility).*

### Interactive Provider Configuration

Configure your model providers, ChatGPT OAuth, or custom API keys:

```bash
anibuild provider
```

### Self-Updater

Update to the latest release at any time:

```bash
anibuild update
```

---

## Building from Source

### Prerequisites

- Rust toolchain (`cargo`, `rustc` 1.85+)
- C compiler (`clang` or `gcc`), `cmake`, and `protobuf-compiler`

### Compilation

```bash
git clone https://github.com/VodTinker/anime-build.git
cd anime-build
cargo build -p xai-grok-pager-bin --bin anibuild --release
```

The compiled binary will be placed at `target/release/anibuild`.

---

## Security & Privacy

- **Local Execution**: All file operations, command executions, and context builds occur locally.
- **Zero Telemetry**: Anibuild contains no telemetry instrumentation or tracking metrics.
- **Direct Process Control**: Commands executed by the agent inherit your shell environment and run under your explicit process permissions.

---

## License

Distributed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for more details.
