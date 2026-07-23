#!/usr/bin/env sh
# Anime installer — cross-platform shell installer for Linux and macOS.
# Usage: curl -fsSL https://anibuild.online/install.sh | sh
#
# Environment variables:
#   ANIME_INSTALL_DIR  — override the install directory (default: ~/.local/bin)
#   ANIME_VERSION      — override the version to install

set -eu

ANIME_VERSION="${ANIME_VERSION:-0.2.102}"
ANIME_REPO="VodTinker/anime-build"
ANIME_BASE_URL="https://github.com/${ANIME_REPO}/releases/download"

# ─── Colour helpers (disabled when stdout is not a terminal) ──────────────────

if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' CYAN='' BOLD='' RESET=''
fi

info()  { printf "%b\n" "${CYAN}${BOLD}info${RESET}  $*"; }
ok()    { printf "%b\n" "${GREEN}${BOLD}  ✓${RESET}  $*"; }
warn()  { printf "%b\n" "${YELLOW}${BOLD}warn${RESET}  $*" >&2; }
error() { printf "%b\n" "${RED}${BOLD}error${RESET} $*" >&2; exit 1; }

# ─── Platform detection ──────────────────────────────────────────────────────

detect_platform() {
    platform="$(uname -s | tr '[:upper:]' '[:lower:]')"
    case "$platform" in
        linux)  echo "linux"  ;;
        darwin) echo "darwin" ;;
        mingw*|msys*|cygwin*)
            error "Windows is not supported by this installer.
       Use PowerShell instead:  irm https://anibuild.online/install.ps1 | iex" ;;
        *)      error "Unsupported platform: $platform" ;;
    esac
}

detect_arch() {
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)      echo "x86_64"  ;;
        aarch64|arm64)      echo "aarch64" ;;
        *)                  error "Unsupported architecture: $arch" ;;
    esac
}

# ─── Download helper (curl preferred, falls back to wget) ────────────────────

download() {
    url="$1"
    output="$2"
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget > /dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        error "curl or wget is required to download Anime."
    fi
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
    printf "\n"
    info "Installing ${BOLD}Anime v${ANIME_VERSION}${RESET}…"
    printf "\n"

    platform="$(detect_platform)"
    arch="$(detect_arch)"

    info "Detected ${BOLD}${platform}-${arch}${RESET}"

    url="${ANIME_BASE_URL}/v${ANIME_VERSION}/anime-${platform}-${arch}.tar.gz"

    info "Downloading from GitHub Releases…"

    tmpdir="$(mktemp -d)"
    # shellcheck disable=SC2064
    trap "rm -rf '$tmpdir'" EXIT INT TERM

    download "$url" "$tmpdir/anime.tar.gz"

    tar xzf "$tmpdir/anime.tar.gz" -C "$tmpdir"

    # Determine install directory
    install_dir="${ANIME_INSTALL_DIR:-}"
    if [ -z "$install_dir" ]; then
        if [ "$(id -u)" = "0" ]; then
            install_dir="/usr/local/bin"
        else
            install_dir="$HOME/.local/bin"
        fi
    fi

    mkdir -p "$install_dir"

    # Install the binary
    if [ -f "$tmpdir/anime" ]; then
        cp "$tmpdir/anime" "$install_dir/anime"
    elif [ -f "$tmpdir/anime-${platform}-${arch}/anime" ]; then
        cp "$tmpdir/anime-${platform}-${arch}/anime" "$install_dir/anime"
    else
        error "Could not find the anime binary in the downloaded archive."
    fi

    chmod +x "$install_dir/anime"
    rm -f "$install_dir/anibuild"
    ln -s anime "$install_dir/anibuild"

    printf "\n"
    ok "Anime v${ANIME_VERSION} installed executable to ${BOLD}${install_dir}/anime${RESET}"
    ok "Command alias installed to ${BOLD}${install_dir}/anibuild${RESET}"
    printf "\n"

    # Check if install dir is in PATH
    case ":${PATH}:" in
        *:"${install_dir}":*) ;;
        *)
            warn "${install_dir} is not in your PATH."
            printf "\n"
            printf "  Add it to your shell config:\n"
            printf "\n"
            # Detect shell for specific advice
            current_shell="$(basename "${SHELL:-sh}")"
            case "$current_shell" in
                zsh)
                    printf "    echo 'export PATH=\"\$PATH:%s\"' >> ~/.zshrc\n" "$install_dir"
                    printf "    source ~/.zshrc\n"
                    ;;
                bash)
                    printf "    echo 'export PATH=\"\$PATH:%s\"' >> ~/.bashrc\n" "$install_dir"
                    printf "    source ~/.bashrc\n"
                    ;;
                fish)
                    printf "    fish_add_path %s\n" "$install_dir"
                    ;;
                *)
                    printf "    export PATH=\"\$PATH:%s\"\n" "$install_dir"
                    ;;
            esac
            printf "\n"
            ;;
    esac

    info "Run ${BOLD}anibuild${RESET} to start. ${BOLD}anime${RESET} remains available for compatibility."
    printf "\n"
}

main "$@"
