# Anime

> **A terminal-native AI coding agent.**
> Inspect your codebase, edit files, and run commands from your local environment.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Platforms](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)](#installation)
[![Telemetry](https://img.shields.io/badge/telemetry-zero-purple.svg)](#privacy)

**Anime**—also available as `anibuild`—is a terminal-native AI coding agent. It works directly in your repository: it understands available project context, proposes and applies changes, and runs commands with visibility in your own terminal session.

- **Local workspace operations:** Your files and processes remain on your machine.
- **Flexible authentication:** ChatGPT Plus/Pro (Codex OAuth), API keys, and compatible providers.
- **Automation-ready:** An interactive TUI for hands-on work and a single-prompt mode for scripts and CI.

## Quick start

### Installation

**macOS and Linux**

```bash
curl -fsSL https://anibuild.online/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://anibuild.online/install.ps1 | iex
```

The installer downloads the latest published release, verifies its SHA-256 checksum when available, and installs `anime` as the primary command. It also creates `anibuild` as a compatibility alias.

> On Linux and macOS, the default install location for non-root users is `~/.local/bin`. If that directory is not on your `PATH`, the installer prints shell-specific instructions.

### Run Anime

Start an interactive session from your project directory:

```bash
anime
```

Configure a provider or authentication method:

```bash
anime provider
```

Run a single prompt, which is useful in scripts and automation:

```bash
anime --prompt "Run the test suite and summarize any failures"
```

## Core commands

| Command | Description |
| --- | --- |
| `anime` | Opens the interactive terminal user interface (TUI). |
| `anime provider` | Configures providers, models, and credentials. |
| `anime --prompt "…"` | Runs a single prompt and writes output to the terminal. |
| `anime update` | Checks for and installs the latest published release. |
| `anime --version` | Prints the installed version. |
| `anime --help` | Lists all available options. |

`anibuild` is an alias for `anime`; either command works wherever the compatibility alias is installed.

## Providers and authentication

Anime supports **ChatGPT Plus/Pro authentication through OpenAI Codex OAuth**, alongside API keys and provider-specific configurations. The `anime provider` setup assistant can configure:

- OpenRouter.
- Anthropic.
- OpenAI.
- DeepSeek.
- Ollama for local models.
- Custom OpenAI-compatible endpoints.

Choose the provider that fits your workflow, and review its policies, pricing, and limits before using it in automated environments.

## Features

| Capability | What it provides |
| --- | --- |
| **Codebase context** | Uses available repository structure and context while working on your project. |
| **File editing and commands** | Modifies files and runs processes with the permissions of your local environment. |
| **TUI and single-prompt modes** | Switches between an interactive terminal session and one-shot execution for scripts. |
| **Configurable providers** | Supports OAuth, API keys, local models, and compatible endpoints. |
| **Built-in updates** | Installs published releases from GitHub Releases. |

## Privacy

Anime performs workspace exploration, file modifications, and command execution on your machine. The project does not include first-party usage telemetry.

When you select a remote provider, requests required to generate responses are sent to that provider. Do not include secrets, personal data, or sensitive code in requests to external services without first understanding their data-handling policies.

## Installer options

On macOS and Linux, you can reinstall Anime or simulate an installation without changing your system:

```bash
# Reinstall even if the current version is already installed
curl -fsSL https://anibuild.online/install.sh | sh -s -- --force

# Print the version, URL, and target path without changing anything
curl -fsSL https://anibuild.online/install.sh | sh -s -- --dry-run
```

You can also select a specific destination directory or release version:

```bash
ANIME_INSTALL_DIR="$HOME/.local/bin" ANIME_VERSION="0.2.201" \
  curl -fsSL https://anibuild.online/install.sh | sh
```

## Build from source

### Requirements

- Rust with `rustc` and `cargo` version 1.85 or later.
- `clang`, `lld`, `cmake`, and `protobuf-compiler`.
- Git.

### Compile and run

```bash
# Clone the repository
git clone https://github.com/VodTinker/anime-build.git
cd anime-build

# Build the distribution binary
cargo build -p xai-grok-pager-bin --bin anime --release

# Run it from the working tree
./target/release/anime
```

The compiled binary is written to `target/release/anime`.

### Workspace structure

This repository is a Rust workspace, with crates under `crates/`. [`xai-grok-pager-bin`](crates/codegen/xai-grok-pager-bin) is the binary composition crate; supporting crates provide terminal UI, configuration, authentication, tools, updates, and shared functionality.

## Project and support

- [Changelog](CHANGELOG.md)
- [Security policy](SECURITY.md)
- [Contribution policy](CONTRIBUTING.md)
- [Apache-2.0 license](LICENSE)

The public source tree is provided for source transparency and local builds. Review [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes: external pull requests and unsolicited patches are not currently accepted.

## License

Anime is distributed under the [Apache License, Version 2.0](LICENSE).
