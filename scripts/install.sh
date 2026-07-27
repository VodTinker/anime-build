#!/usr/bin/env sh
# Anime installer — cross-platform shell installer for Linux and macOS.
# Usage: curl -fsSL https://anibuild.online/install.sh | sh
#
# Environment variables:
#   ANIME_INSTALL_DIR  — override the install directory (default: ~/.local/bin)
#   ANIME_VERSION      — override the version to install

ANIME_REPO="VodTinker/anime-build"
ANIME_BASE_URL="https://github.com/${ANIME_REPO}/releases/download"

get_latest_version() {
    latest=""
    # 1. Try GitHub API
    if command -v curl > /dev/null 2>&1; then
        latest="$(curl --proto '=https' --tlsv1.2 --retry 3 --retry-connrefused -fsSL "https://api.github.com/repos/${ANIME_REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/' || true)"
    fi
    # 2. Fallback: GitHub HTML Redirect (Bypasses API rate limits)
    if [ -z "$latest" ] && command -v curl > /dev/null 2>&1; then
        latest="$(curl --proto '=https' --tlsv1.2 -sI "https://github.com/${ANIME_REPO}/releases/latest" 2>/dev/null | grep -i '^location:' | sed -E 's/.*\/tag\/v?([^ \r\n]+).*/\1/' || true)"
    fi
    # 3. Fallback: wget
    if [ -z "$latest" ] && command -v wget > /dev/null 2>&1; then
        latest="$(wget --https-only --secure-protocol=TLSv1_2 --tries=3 -qO- "https://api.github.com/repos/${ANIME_REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/' || true)"
    fi
    if [ -n "$latest" ]; then
        echo "$latest"
    else
        error "Unable to resolve latest Anime release version from GitHub."
    fi
}

ANIME_VERSION="${ANIME_VERSION:-$(get_latest_version)}"

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

info()  { printf "%binfo%b  %b\n" "${CYAN}${BOLD}" "${RESET}" "$*"; }
ok()    { printf "%b  ✓%b  %b\n" "${GREEN}${BOLD}" "${RESET}" "$*"; }
warn()  { printf "%bwarn%b  %b\n" "${YELLOW}${BOLD}" "${RESET}" "$*" >&2; }
error() { printf "%berror%b %b\n" "${RED}${BOLD}" "${RESET}" "$*" >&2; exit 1; }


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

# ─── Download & Checksum helpers ──────────────────────────────────────────────

download() {
    url="$1"
    output="$2"
    if command -v curl > /dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 --retry 3 --retry-connrefused -fsSL "$url" -o "$output"
    elif command -v wget > /dev/null 2>&1; then
        wget --https-only --secure-protocol=TLSv1_2 --tries=3 -q "$url" -O "$output"
    else
        error "curl or wget is required to download Anime."
    fi
}

verify_checksum() {
    file_to_check="$1"
    url_to_check="$2"

    sha_url="${url_to_check}.sha256"
    tmp_sha="${file_to_check}.sha256"

    if download "$sha_url" "$tmp_sha" 2>/dev/null; then
        expected_hash="$(tr -d '[:space:]' < "$tmp_sha" | awk '{print $1}')"
        if [ -n "$expected_hash" ]; then
            actual_hash=""
            if command -v sha256sum >/dev/null 2>&1; then
                actual_hash="$(sha256sum "$file_to_check" | awk '{print $1}')"
            elif command -v shasum >/dev/null 2>&1; then
                actual_hash="$(shasum -a 256 "$file_to_check" | awk '{print $1}')"
            elif command -v openssl >/dev/null 2>&1; then
                actual_hash="$(openssl dgst -sha256 "$file_to_check" | awk '{print $2}')"
            fi

            if [ -n "$actual_hash" ]; then
                if [ "$expected_hash" = "$actual_hash" ]; then
                    ok "SHA256 checksum verified successfully."
                else
                    error "SHA256 checksum mismatch! Expected: ${expected_hash}, Got: ${actual_hash}"
                fi
            fi
        fi
    fi
}

# ─── Existing Installation & Version Detection ────────────────────────────────

get_installed_version() {
    target_dir="$1"
    target_bin=""
    if [ -x "${target_dir}/anime" ]; then
        target_bin="${target_dir}/anime"
    elif command -v anime >/dev/null 2>&1; then
        target_bin="$(command -v anime)"
    fi

    if [ -n "$target_bin" ]; then
        ver="$("$target_bin" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?' | head -n1 || true)"
        if [ -n "$ver" ]; then
            echo "$ver"
            return 0
        fi
    fi
    echo ""
}

compare_versions() {
    v1="$(echo "$1" | sed -E 's/^v//')"
    v2="$(echo "$2" | sed -E 's/^v//')"

    if [ "$v1" = "$v2" ]; then
        echo "eq"
        return 0
    fi

    m1="$(echo "$v1" | cut -d. -f1 | tr -dc '0-9')"
    n1="$(echo "$v1" | cut -d. -f2 | tr -dc '0-9')"
    p1="$(echo "$v1" | cut -d. -f3 | cut -d- -f1 | tr -dc '0-9')"

    m2="$(echo "$v2" | cut -d. -f1 | tr -dc '0-9')"
    n2="$(echo "$v2" | cut -d. -f2 | tr -dc '0-9')"
    p2="$(echo "$v2" | cut -d. -f3 | cut -d- -f1 | tr -dc '0-9')"

    m1="${m1:-0}"; n1="${n1:-0}"; p1="${p1:-0}"
    m2="${m2:-0}"; n2="${n2:-0}"; p2="${p2:-0}"

    if [ "$m1" -gt "$m2" ]; then echo "gt"; return 0; fi
    if [ "$m1" -lt "$m2" ]; then echo "lt"; return 0; fi
    if [ "$n1" -gt "$n2" ]; then echo "gt"; return 0; fi
    if [ "$n1" -lt "$n2" ]; then echo "lt"; return 0; fi
    if [ "$p1" -gt "$p2" ]; then echo "gt"; return 0; fi
    if [ "$p1" -lt "$p2" ]; then echo "lt"; return 0; fi

    echo "eq"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
    FORCE_INSTALL="false"
    DRY_RUN="false"
    for arg in "$@"; do
        case "$arg" in
            -f|--force) FORCE_INSTALL="true" ;;
            -d|--dry-run) DRY_RUN="true" ;;
        esac
    done

    # Determine install directory
    install_dir="${ANIME_INSTALL_DIR:-}"
    if [ -z "$install_dir" ]; then
        if [ "$(id -u)" = "0" ]; then
            install_dir="/usr/local/bin"
        else
            install_dir="$HOME/.local/bin"
        fi
    fi

    platform="$(detect_platform)"
    arch="$(detect_arch)"
    url="${ANIME_BASE_URL}/v${ANIME_VERSION}/anime-${platform}-${arch}.tar.gz"

    if [ "$DRY_RUN" = "true" ]; then
        printf "\n"
        info "${BOLD}[DRY RUN] Simulation mode enabled — no changes will be made to your system.${RESET}"
        info "[DRY RUN] Target version : ${BOLD}Anime v${ANIME_VERSION}${RESET}"
        info "[DRY RUN] Target system  : ${BOLD}${platform}-${arch}${RESET}"
        info "[DRY RUN] Download URL   : ${url}"
        info "[DRY RUN] Install path   : ${install_dir}/anime"
        info "[DRY RUN] Alias path     : ${install_dir}/anibuild"
        printf "\n"
        exit 0
    fi

    printf "\n"
    info "Detected system: ${BOLD}${platform}-${arch}${RESET}"

    # Check for existing installation
    installed_ver="$(get_installed_version "$install_dir")"

    if [ -n "$installed_ver" ]; then
        cmp="$(compare_versions "$installed_ver" "$ANIME_VERSION")"
        if [ "$cmp" = "eq" ] && [ "${ANIME_FORCE:-0}" != "1" ] && [ "$FORCE_INSTALL" != "true" ]; then
            ok "Anime ${BOLD}v${installed_ver}${RESET} is already installed and up to date!"
            info "Use ${BOLD}ANIME_FORCE=1${RESET} or ${BOLD}--force${RESET} to reinstall anyway."
            printf "\n"
            exit 0
        elif [ "$cmp" = "lt" ]; then
            info "Upgrading Anime: ${YELLOW}v${installed_ver}${RESET} -> ${GREEN}v${ANIME_VERSION}${RESET}…"
        elif [ "$cmp" = "gt" ]; then
            info "Downgrading Anime: ${YELLOW}v${installed_ver}${RESET} -> ${GREEN}v${ANIME_VERSION}${RESET}…"
        else
            info "Reinstalling Anime ${BOLD}v${ANIME_VERSION}${RESET}…"
        fi
    else
        info "Installing ${BOLD}Anime v${ANIME_VERSION}${RESET}…"
    fi
    printf "\n"

    info "Downloading from GitHub Releases…"

    # Restrict file permissions for temp workspace
    umask 077
    tmpdir="$(mktemp -d)"
    # shellcheck disable=SC2064
    trap "rm -rf '$tmpdir'" EXIT INT TERM HUP QUIT

    download "$url" "$tmpdir/anime.tar.gz"
    verify_checksum "$tmpdir/anime.tar.gz" "$url"

    tar xzf "$tmpdir/anime.tar.gz" -C "$tmpdir"

    mkdir -p "$install_dir"

    # Install the binary atomically (copy to tmp first, chmod +x, then atomic mv)
    new_bin=""
    if [ -f "$tmpdir/anime" ]; then
        new_bin="$tmpdir/anime"
    elif [ -f "$tmpdir/anime-${platform}-${arch}/anime" ]; then
        new_bin="$tmpdir/anime-${platform}-${arch}/anime"
    else
        error "Could not find the anime binary in the downloaded archive."
    fi

    rm -f "$install_dir/anime.tmp"
    cp "$new_bin" "$install_dir/anime.tmp"
    chmod +x "$install_dir/anime.tmp"
    mv -f "$install_dir/anime.tmp" "$install_dir/anime"
    rm -f "$install_dir/anibuild"
    ln -s anime "$install_dir/anibuild"

    printf "\n"
    if [ -n "$installed_ver" ]; then
        ok "Anime updated successfully to ${BOLD}v${ANIME_VERSION}${RESET} at ${BOLD}${install_dir}/anime${RESET}"
    else
        ok "Anime v${ANIME_VERSION} installed successfully to ${BOLD}${install_dir}/anime${RESET}"
    fi
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

