<div align="center">

<h1><code>anibuild</code> — Anime</h1>

**Anime** is a terminal-based AI coding assistant created and maintained by
V01D. It runs as a full-screen TUI powered by OpenAI Codex through a ChatGPT
account. Anime understands your codebase, edits files, executes shell commands,
and manages long-running tasks — interactively, headlessly for scripting/CI, or
embedded in editors via the Agent Client Protocol (ACP).

[Getting started](#getting-started) ·
[Building from source](#building-from-source) ·
[Authentication and models](#authentication-and-models) ·
[Privacy](#privacy) ·
[Documentation](#documentation) ·
[Repository layout](#repository-layout) ·
[Development](#development) ·
[Contributing](#contributing) ·
[License](#license)

This repository contains the Rust source for the Anime CLI/TUI and its agent
runtime. It is based on a periodically synced upstream codebase; `SOURCE_REV`
records the corresponding upstream revision.

Anime is not affiliated with xAI or Grok.

</div>

---

## Getting started

Build Anime from source, then start it. Run `anibuild` to start Anime. The `anime` command remains available for compatibility. The source target remains `anime`:

```sh
cargo run -p xai-grok-pager-bin --bin anime
```

On an interactive first launch, Anime asks you to sign in with a ChatGPT Plus
or Pro account. You can also start the OAuth flow explicitly:

```sh
cargo run -p xai-grok-pager-bin --bin anime -- login --openai
```

The browser-based PKCE flow stores its credentials in a dedicated local
`openai-codex.json` file under Anime's home directory. It is intentionally
separate from the inherited Grok authentication store.

## Building from source

Requirements:

- **Rust** — the toolchain is pinned by [`rust-toolchain.toml`](rust-toolchain.toml);
  `rustup` installs it automatically on first build.
- **[DotSlash](https://dotslash-cli.com)** — required so hermetic tools under
  [`bin/`](bin/) (notably [`bin/protoc`](bin/protoc)) can download and run.
  Install it and ensure `dotslash` is on your `PATH` **before** building:

  ```sh
  cargo install dotslash
  # or: prebuilt packages — https://dotslash-cli.com/docs/installation/
  /usr/bin/env dotslash --help   # sanity check
  ```

- **protoc** — proto codegen resolves [`bin/protoc`](bin/protoc) via DotSlash,
  or falls back to a `protoc` on `PATH` / `$PROTOC`.
- macOS and Linux are supported build hosts; Windows builds are best-effort
  and not currently tested from this tree.

```sh
cargo run -p xai-grok-pager-bin --bin anime              # build + launch Anime
cargo build -p xai-grok-pager-bin --bin anime --release  # target/release/anime
cargo check -p xai-grok-pager-bin                         # fast validation
```

### Release preflight

Before creating a version tag, build the exact Linux distribution artifact
locally. This catches release-only compilation failures before the cross-platform
GitHub Actions release runs:

```sh
cargo build -p xai-grok-pager-bin --bin anime --profile release-dist --features release-dist
```

The `Release preflight` workflow runs the same Linux build for pushes to `main`,
pull requests, and manual dispatches. The tag-only `Release` workflow remains
the authoritative producer of the four published platform artifacts.

### Faster local rebuilds

The development profile prioritizes iteration speed, and the repository uses a
small portable wrapper that enables [`sccache`](https://github.com/mozilla/sccache)
when it is available. It falls back to `rustc` unchanged when it is not, so
installing it is optional. On Arch Linux:

```sh
sudo pacman -S sccache
sccache --show-stats
```

`sccache` stores compiler results only on the local machine. It is especially
useful after cleaning `target/` or switching branches; the first build still
has to compile its dependencies. Avoid workspace-wide commands while iterating:
target the package you changed instead.

The upstream-compatible `xai-grok-pager` binary remains available. The installed product command is `anibuild`; `anime` is an equivalent compatibility command.

## Authentication and models

Anime uses ChatGPT OAuth for the Codex Responses API. It does not fall back to
xAI credentials when launched as `anime`.

The built-in model picker exposes these aliases:

| Alias | Model | Default reasoning effort | Intended use |
|---|---|---:|---|
| `codex-terra` | `gpt-5.6-terra` | medium | Everyday coding work |
| `codex-sol` | `gpt-5.6-sol` | high | Complex reasoning and architecture |
| `codex-luna` | `gpt-5.6-luna` | low | Focused, lower-latency tasks |

Set `ANIME_CODEX_MODEL` to choose a model before launch, or use Anime's model
picker during a session. The Codex endpoint uses a 200,000-token context
window. Responses requests are adapted for Codex compatibility: system prompts
are sent as developer instructions, `temperature` is omitted, and streamed
function calls are reconstructed from SSE deltas.

## Local-only build and privacy

Anime is intentionally built for local development. The following features are
included:

- The TUI, MCP and ACP integrations, ChatGPT/Codex OAuth, agents, and terminal
  tools.
- Normal local file tools, including PDF and PowerPoint (`.pptx`) reading.
- Image generation and image editing.
- Proxy uploads and local diagnostics through Rust `tracing`.

To reduce the build closure and prevent remote diagnostic delivery, this fork
permanently excludes GCS, S3/AWS, AWS-LC, video generation, Sentry, Mixpanel,
OTLP/OpenTelemetry exporters, distributed trace propagation, and Computer Hub
trace/log/Prometheus-metric donation. Legacy direct-cloud upload configuration
is kept only for compatibility and reports an error that instructs users to
configure proxy uploads instead.

ChatGPT OAuth credentials are stored locally with owner-only file permissions
and are not written to the inherited Grok `auth.json` store. For the technical
scope of the dependency-reduced build, see
[`docs/superpowers/specs/2026-07-19-minimal-local-build-design.md`](docs/superpowers/specs/2026-07-19-minimal-local-build-design.md).

## Documentation

Full online documentation is available at
[docs.x.ai/build/overview](https://docs.x.ai/build/overview).

The user guide ships with the pager crate:
[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)
— getting started, keyboard shortcuts, slash commands, configuration, theming,
MCP servers, skills, plugins, hooks, headless mode, sandboxing, and more.

## Repository layout

| Path | Contents |
|------|----------|
| `crates/codegen/xai-grok-pager-bin` | Composition-root package; builds the `xai-grok-pager` binary |
| `crates/codegen/xai-grok-pager` | The TUI: scrollback, prompt, modals, rendering |
| `crates/codegen/xai-grok-shell` | Agent runtime + leader/stdio/headless entry points |
| `crates/codegen/xai-grok-tools` | Tool implementations (terminal, file edit, search, ...) |
| `crates/codegen/xai-grok-workspace` | Host filesystem, VCS, execution, checkpoints |
| `crates/codegen/...` | The rest of the CLI crate closure (config, MCP, markdown, sandbox, ...) |
| `crates/common/`, `crates/build/`, `prod/mc/` | Small shared leaf crates pulled in by the closure |
| `third_party/` | Vendored upstream source (Mermaid diagram stack) — see below |

> [!IMPORTANT]
> The root `Cargo.toml` (workspace members, dependency versions, lints,
> profiles) is **generated** — treat it as read-only. Prefer editing per-crate
> `Cargo.toml` files.

## Development

```sh
cargo check -p <crate>        # always target specific crates; full-workspace builds are slow
cargo test -p xai-grok-config # per-crate tests
cargo clippy -p <crate>       # lint config: clippy.toml at the repo root
cargo fmt --all               # rustfmt.toml at the repo root
```

## Contributing

> [!NOTE]
> External contributions are not accepted. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

First-party code in this repository is licensed under the **Apache License,
Version 2.0** — see [`LICENSE`](LICENSE).

Third-party and vendored code remains under its original licenses. See:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) — crates.io / git dependencies,
  bundled UI themes, and **in-tree source ports** (including openai/codex and
  sst/opencode tool implementations)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
  — crate-local notice for the codex and opencode ports (license texts +
  Apache §4(b) change notice)
- [`third_party/NOTICE`](third_party/NOTICE) — vendored Mermaid-stack index
