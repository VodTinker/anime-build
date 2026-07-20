# Anime installer for Windows (PowerShell)
# Usage: irm https://anibuild.online/install.ps1 | iex
#
# Environment variables:
#   ANIME_INSTALL_DIR  — override the install directory
#   ANIME_VERSION      — override the version to install

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ANIME_VERSION = if ($env:ANIME_VERSION) { $env:ANIME_VERSION } else { "0.2.102" }
$ANIME_REPO = "VodTinker/anime-build"
$ANIME_BASE_URL = "https://github.com/$ANIME_REPO/releases/download"

# ─── Architecture detection ──────────────────────────────────────────────────

function Get-AnimeArch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        'AMD64'  { return 'x86_64' }
        'ARM64'  { return 'aarch64' }
        default  { throw "Unsupported architecture: $arch" }
    }
}

# ─── Main ─────────────────────────────────────────────────────────────────────

function Install-Anime {
    Write-Host ""
    Write-Host "  Installing " -NoNewline
    Write-Host "Anime v$ANIME_VERSION" -ForegroundColor Cyan -NoNewline
    Write-Host "..."
    Write-Host ""

    $arch = Get-AnimeArch
    Write-Host "  Detected " -NoNewline
    Write-Host "windows-$arch" -ForegroundColor Cyan

    $url = "$ANIME_BASE_URL/v$ANIME_VERSION/anime-windows-$arch.zip"

    Write-Host "  Downloading from GitHub Releases..."

    # Create temp directory
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("anime-install-" + [System.Guid]::NewGuid().ToString("N").Substring(0, 8))
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tmpDir "anime.zip"

        # Download
        try {
            Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        }
        catch {
            Write-Host ""
            Write-Host "  ERROR: " -ForegroundColor Red -NoNewline
            Write-Host "Failed to download from $url"
            Write-Host "        Make sure version v$ANIME_VERSION exists in GitHub Releases."
            Write-Host ""
            throw
        }

        # Extract
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        # Determine install directory
        $installDir = if ($env:ANIME_INSTALL_DIR) {
            $env:ANIME_INSTALL_DIR
        } else {
            Join-Path $env:USERPROFILE ".local\bin"
        }

        New-Item -ItemType Directory -Path $installDir -Force | Out-Null

        # Find and copy the binary
        $binaryPath = Get-ChildItem -Path $tmpDir -Recurse -Filter "anime.exe" | Select-Object -First 1
        if (-not $binaryPath) {
            throw "Could not find anime.exe in the downloaded archive."
        }

        Copy-Item -Path $binaryPath.FullName -Destination (Join-Path $installDir "anime.exe") -Force

        Write-Host ""
        Write-Host "  ✓ " -ForegroundColor Green -NoNewline
        Write-Host "Anime v$ANIME_VERSION installed to " -NoNewline
        Write-Host "$installDir\anime.exe" -ForegroundColor Cyan
        Write-Host ""

        # Check and update PATH
        $userPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
        if ($userPath -notlike "*$installDir*") {
            [Environment]::SetEnvironmentVariable('PATH', "$userPath;$installDir", 'User')
            # Also update current session
            $env:PATH = "$env:PATH;$installDir"

            Write-Host "  ⚠ " -ForegroundColor Yellow -NoNewline
            Write-Host "Added $installDir to your PATH."
            Write-Host "     Restart your terminal for changes to take effect."
            Write-Host ""
        }

        Write-Host "  Run " -NoNewline
        Write-Host "anime" -ForegroundColor Cyan -NoNewline
        Write-Host " to start."
        Write-Host ""
    }
    finally {
        # Cleanup
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

Install-Anime
