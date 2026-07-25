param (
    [switch]$Force,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ANIME_REPO = "VodTinker/anime-build"
$ANIME_BASE_URL = "https://github.com/$ANIME_REPO/releases/download"

function Get-LatestAnimeVersion {
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$ANIME_REPO/releases/latest" -UseBasicParsing -ErrorAction Stop
        if ($release.tag_name -match 'v?(\d+\.\d+\.\d+)') {
            return $Matches[1]
        }
    } catch {}
    return "0.2.104"
}

$ANIME_VERSION = if ($env:ANIME_VERSION) { $env:ANIME_VERSION } else { Get-LatestAnimeVersion }

# ─── Architecture detection ──────────────────────────────────────────────────

function Get-AnimeArch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        'AMD64'  { return 'x86_64' }
        'ARM64'  { return 'aarch64' }
        default  { throw "Unsupported architecture: $arch" }
    }
}

# ─── Existing Installation & Version Detection ────────────────────────────────

function Get-InstalledAnimeVersion {
    param ([string]$InstallDir)
    $animeExe = Join-Path $InstallDir "anime.exe"
    if (Test-Path $animeExe) {
        try {
            $verOutput = & $animeExe --version 2>$null
            if ($verOutput -match '(\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)*)') {
                return $Matches[1]
            }
        } catch {}
    }
    $cmd = Get-Command anime -ErrorAction SilentlyContinue
    if ($cmd) {
        try {
            $verOutput = & $cmd.Source --version 2>$null
            if ($verOutput -match '(\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)*)') {
                return $Matches[1]
            }
        } catch {}
    }
    return $null
}

function Compare-Versions {
    param ([string]$v1, [string]$v2)
    try {
        $cleanV1 = ($v1 -replace '^v','').Split('-')[0]
        $cleanV2 = ($v2 -replace '^v','').Split('-')[0]
        $ver1 = [System.Version]::Parse($cleanV1)
        $ver2 = [System.Version]::Parse($cleanV2)
        return $ver1.CompareTo($ver2)
    } catch {
        if ($v1 -eq $v2) { return 0 }
        return 0
    }
}

# ─── Checksum Verification ───────────────────────────────────────────────────

function Verify-Checksum {
    param (
        [string]$FilePath,
        [string]$Url
    )
    $shaUrl = "$Url.sha256"
    $shaPath = "$FilePath.sha256"
    try {
        Invoke-WebRequest -Uri $shaUrl -OutFile $shaPath -UseBasicParsing -ErrorAction SilentlyContinue
        if (Test-Path $shaPath) {
            $expectedHash = (Get-Content $shaPath -Raw).Trim().Split()[0]
            $actualHash = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
            if ($expectedHash -and ($expectedHash.ToLower() -eq $actualHash)) {
                Write-Host "  ✓ " -ForegroundColor Green -NoNewline
                Write-Host "SHA256 checksum verified successfully."
            } elseif ($expectedHash) {
                throw "SHA256 checksum mismatch! Expected: $expectedHash, Actual: $actualHash"
            }
        }
    } catch {
        if ($_ -match "mismatch") { throw }
    }
}

# ─── Main ─────────────────────────────────────────────────────────────────────

function Install-Anime {
    Write-Host ""
    $arch = Get-AnimeArch

    # Determine install directory
    $installDir = if ($env:ANIME_INSTALL_DIR) {
        $env:ANIME_INSTALL_DIR
    } else {
        Join-Path $env:USERPROFILE ".local\bin"
    }

    $url = "$ANIME_BASE_URL/v$ANIME_VERSION/anime-windows-$arch.zip"

    if ($DryRun) {
        Write-Host "  [DRY RUN] " -ForegroundColor Yellow -NoNewline
        Write-Host "Simulation mode enabled — no changes will be made to your system."
        Write-Host "  [DRY RUN] Target version : " -NoNewline
        Write-Host "v$ANIME_VERSION" -ForegroundColor Cyan
        Write-Host "  [DRY RUN] Target system  : windows-$arch"
        Write-Host "  [DRY RUN] Download URL   : $url"
        Write-Host "  [DRY RUN] Install path   : $(Join-Path $installDir 'anime.exe')"
        Write-Host "  [DRY RUN] Alias path     : $(Join-Path $installDir 'anibuild.exe')"
        Write-Host ""
        return
    }

    Write-Host "  Detected " -NoNewline
    Write-Host "windows-$arch" -ForegroundColor Cyan

    # Check existing installation
    $installedVer = Get-InstalledAnimeVersion -InstallDir $installDir
    if ($installedVer) {
        $cmp = Compare-Versions $installedVer $ANIME_VERSION
        if ($cmp -eq 0 -and -not $env:ANIME_FORCE -and -not $Force) {
            Write-Host "  ✓ " -ForegroundColor Green -NoNewline
            Write-Host "Anime v$installedVer is already installed and up to date!"
            Write-Host "    Set `$env:ANIME_FORCE=1 or run with -Force to reinstall."
            Write-Host ""
            return
        } elseif ($cmp -lt 0) {
            Write-Host "  Upgrading Anime from " -NoNewline
            Write-Host "v$installedVer" -ForegroundColor Yellow -NoNewline
            Write-Host " -> " -NoNewline
            Write-Host "v$ANIME_VERSION" -ForegroundColor Green
        } elseif ($cmp -gt 0) {
            Write-Host "  Downgrading Anime from " -NoNewline
            Write-Host "v$installedVer" -ForegroundColor Yellow -NoNewline
            Write-Host " -> " -NoNewline
            Write-Host "v$ANIME_VERSION" -ForegroundColor Green
        } else {
            Write-Host "  Reinstalling Anime " -NoNewline
            Write-Host "v$ANIME_VERSION" -ForegroundColor Cyan
        }
    } else {
        Write-Host "  Installing Anime " -NoNewline
        Write-Host "v$ANIME_VERSION" -ForegroundColor Cyan -NoNewline
        Write-Host " (fresh install)..."
    }
    Write-Host ""

    Write-Host "  Downloading from GitHub Releases..."

    # Create temp directory
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("anime-install-" + [System.Guid]::NewGuid().ToString("N").Substring(0, 8))
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tmpDir "anime.zip"

        # Download
        try {
            Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
            Verify-Checksum -FilePath $zipPath -Url $url
        }
        catch {
            Write-Host ""
            Write-Host "  ERROR: " -ForegroundColor Red -NoNewline
            Write-Host "Failed to download or verify from $url"
            Write-Host "        Make sure version v$ANIME_VERSION exists in GitHub Releases."
            Write-Host ""
            throw
        }

        # Extract
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        New-Item -ItemType Directory -Path $installDir -Force | Out-Null

        # Find and copy the binary
        $binaryPath = Get-ChildItem -Path $tmpDir -Recurse -Filter "anime.exe" | Select-Object -First 1
        if (-not $binaryPath) {
            throw "Could not find anime.exe in the downloaded archive."
        }

        Copy-Item -Path $binaryPath.FullName -Destination (Join-Path $installDir "anime.exe") -Force
        $animePath = Join-Path $installDir 'anime.exe'
        $anibuildPath = Join-Path $installDir 'anibuild.exe'
        Remove-Item -Path $anibuildPath -Force -ErrorAction SilentlyContinue
        try {
            New-Item -ItemType HardLink -Path $anibuildPath -Target $animePath -ErrorAction Stop | Out-Null
        }
        catch {
            Copy-Item -Path $animePath -Destination $anibuildPath -Force
        }

        Write-Host ""
        Write-Host "  ✓ " -ForegroundColor Green -NoNewline
        if ($installedVer) {
            Write-Host "Anime updated successfully to v$ANIME_VERSION at " -NoNewline
        } else {
            Write-Host "Anime v$ANIME_VERSION installed to " -NoNewline
        }
        Write-Host "$animePath" -ForegroundColor Cyan
        Write-Host "  Alias installed to " -NoNewline
        Write-Host "$anibuildPath" -ForegroundColor Cyan
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
        Write-Host "anibuild" -ForegroundColor Cyan -NoNewline
        Write-Host " to start."
        Write-Host "  anime remains available as a compatibility command."
        Write-Host ""
    }
    finally {
        # Cleanup
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

Install-Anime


