# README improvement design

**Date:** 2026-07-27  
**Status:** Approved — direct implementation requested

## Goal

Rewrite the English root README so it works both as a fast adoption guide for Anime users and as a reliable technical entry point for developers building the project from source.

## Audience

1. Users who want to install Anime, configure a provider, and start an interactive or one-shot terminal session.
2. Developers who need local build requirements, the correct Cargo command, and project-policy links.

## Structure

1. Product header with stable badges and a concise description.
2. Quick start covering macOS/Linux installation, Windows PowerShell installation, `anime` as the primary command, and `anibuild` as a compatibility alias.
3. A small command table for interactive mode, provider configuration, headless prompts, updating, and help.
4. A factual feature and privacy section.
5. A developer section for source builds and workspace orientation.
6. References to the changelog, security policy, contribution policy, and Apache-2.0 license.

## Accuracy constraints

- Use `anime` as the primary binary: the install script distributes `anime` and creates `anibuild` as a symbolic-link alias.
- Use `cargo build -p xai-grok-pager-bin --bin anime --release` as the source build command.
- Document Linux/macOS installation through `install.sh` and Windows installation through `install.ps1`.
- Keep the provider list aligned with `provider_cmd.rs`: OpenRouter, Anthropic, OpenAI, DeepSeek, Ollama, and compatible custom endpoints. Mention ChatGPT Plus/Pro OAuth separately as an available authentication workflow.
- Define privacy precisely: execution and repository operations are local; provider requests necessarily go to the provider selected by the user.
- Do not say the project accepts external PRs because `CONTRIBUTING.md` explicitly does not.

## Verification

- Verify Markdown structure and links to local files.
- Inspect the final diff to ensure commands and terminology match the installer and Cargo configuration.
