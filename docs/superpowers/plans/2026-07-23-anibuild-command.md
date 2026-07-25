# `anibuild` Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `anibuild` the primary documented installation command while retaining the existing `anime` command for compatibility.

**Architecture:** The release archive remains unchanged and provides `anime`. Each installer writes that executable under its current name, then creates a second launcher next to it. POSIX uses a relative symbolic link; Windows attempts a hard link and falls back to copying the executable.

**Tech Stack:** POSIX `sh`, PowerShell, Markdown documentation.

## Global Constraints

- Preserve the installed `anime` / `anime.exe` command and its behavior.
- Do not rename Cargo binary targets, application-internal identifiers, config paths, or authentication paths.
- POSIX alias target must be the relative name `anime`.
- Windows must fall back to copying `anime.exe` if hard-link creation fails.
- Do not modify the user’s existing `crates/codegen/xai-grok-pager/assets/logo/logo07.txt` change.
- Do not publish releases, push commits, or dispatch GitHub Actions.

---

### Task 1: Add the POSIX `anibuild` launcher

**Files:**
- Modify: `scripts/install.sh:105-118`

**Interfaces:**
- Consumes: extracted executable at `$tmpdir/anime` or `$tmpdir/anime-${platform}-${arch}/anime`.
- Produces: executable `$install_dir/anime` and symlink `$install_dir/anibuild` whose link text is `anime`.

- [ ] **Step 1: Create a temporary installer-fixture test script**

Create `/tmp/test-anibuild-posix.sh` with a minimal assertion of the desired launcher contract:

```sh
#!/usr/bin/env sh
set -eu
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
printf '#!/usr/bin/env sh\nprintf "anime\\n"\n' > "$fixture/anime"
chmod +x "$fixture/anime"
ln -s anime "$fixture/anibuild"
[ -L "$fixture/anibuild" ]
[ "$(readlink "$fixture/anibuild")" = anime ]
[ "$("$fixture/anibuild")" = anime ]
```

- [ ] **Step 2: Run the fixture to establish the launcher expectation**

Run: `sh /tmp/test-anibuild-posix.sh`

Expected: exit code `0`; the relative symbolic link executes the neighboring `anime` file.

- [ ] **Step 3: Add the launcher creation after the current `chmod` line**

In `scripts/install.sh`, immediately after:

```sh
chmod +x "$install_dir/anime"
```

insert:

```sh
# Keep the historical executable name while exposing the product command.
rm -f "$install_dir/anibuild"
ln -s anime "$install_dir/anibuild"
```

- [ ] **Step 4: Update the POSIX completion copy**

Replace the existing final command message:

```sh
info "Run ${BOLD}anime${RESET} to start."
```

with:

```sh
info "Run ${BOLD}anibuild${RESET} to start. (${BOLD}anime${RESET} remains available for compatibility.)"
```

Also replace the success output directly after `chmod` so it reports both paths:

```sh
ok "Anime v${ANIME_VERSION} installed to ${BOLD}${install_dir}/anime${RESET}"
ok "Command alias installed at ${BOLD}${install_dir}/anibuild${RESET}"
```

- [ ] **Step 5: Verify POSIX syntax and the real installer block in an isolated fixture**

Run:

```sh
sh -n scripts/install.sh
fixture="$(mktemp -d)"
printf '#!/usr/bin/env sh\nprintf "anime\\n"\n' > "$fixture/anime"
chmod +x "$fixture/anime"
rm -f "$fixture/anibuild"
ln -s anime "$fixture/anibuild"
[ -L "$fixture/anibuild" ]
[ "$(readlink "$fixture/anibuild")" = anime ]
[ "$("$fixture/anibuild")" = anime ]
rm -rf "$fixture"
```

Expected: all commands exit `0`.

- [ ] **Step 6: Commit the POSIX installer change**

```bash
git add scripts/install.sh
git commit -m "feat: add anibuild POSIX launcher"
```

### Task 2: Add the Windows `anibuild.exe` launcher

**Files:**
- Modify: `scripts/install.ps1:81-103`

**Interfaces:**
- Consumes: installed executable at `$installDir\anime.exe`.
- Produces: `$installDir\anibuild.exe`, a hard link to `anime.exe` when supported or a byte-for-byte copy otherwise.

- [ ] **Step 1: Create a failing PowerShell launcher-contract test**

Create `/tmp/Test-AnibuildLauncher.ps1`:

```powershell
$ErrorActionPreference = 'Stop'
$fixture = Join-Path ([System.IO.Path]::GetTempPath()) ('anibuild-test-' + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $fixture | Out-Null
try {
    $anime = Join-Path $fixture 'anime.exe'
    $anibuild = Join-Path $fixture 'anibuild.exe'
    [System.IO.File]::WriteAllBytes($anime, [byte[]](1, 2, 3))
    if (-not (Test-Path $anibuild)) { throw 'anibuild.exe launcher is missing' }
}
finally {
    Remove-Item -Recurse -Force $fixture -ErrorAction SilentlyContinue
}
```

- [ ] **Step 2: Run the contract test and verify the expected failure**

Run on a Windows PowerShell-capable host:

```powershell
pwsh -NoProfile -File /tmp/Test-AnibuildLauncher.ps1
```

Expected: failure with `anibuild.exe launcher is missing` before production code is changed.

- [ ] **Step 3: Add hard-link creation and copy fallback**

In `scripts/install.ps1`, directly after the current `Copy-Item` that writes `anime.exe`, insert:

```powershell
        $animePath = Join-Path $installDir 'anime.exe'
        $anibuildPath = Join-Path $installDir 'anibuild.exe'
        Remove-Item -Path $anibuildPath -Force -ErrorAction SilentlyContinue
        try {
            New-Item -ItemType HardLink -Path $anibuildPath -Target $animePath -ErrorAction Stop | Out-Null
        }
        catch {
            Copy-Item -Path $animePath -Destination $anibuildPath -Force
        }
```

- [ ] **Step 4: Make the PowerShell contract test pass**

Replace the body of `/tmp/Test-AnibuildLauncher.ps1` after writing `anime.exe` with the same hard-link/fallback code used by the installer, then add:

```powershell
    if (-not (Test-Path -Path $anibuild -PathType Leaf)) { throw 'anibuild.exe launcher is missing' }
    if (-not ([System.Linq.Enumerable]::SequenceEqual(
        [System.IO.File]::ReadAllBytes($anime),
        [System.IO.File]::ReadAllBytes($anibuild)
    ))) { throw 'anibuild.exe does not match anime.exe' }
```

Run: `pwsh -NoProfile -File /tmp/Test-AnibuildLauncher.ps1`

Expected: exit code `0`.

- [ ] **Step 5: Update PowerShell completion output**

After the existing installed-location output, add:

```powershell
        Write-Host "  ✓ " -ForegroundColor Green -NoNewline
        Write-Host "Command alias installed to " -NoNewline
        Write-Host "$installDir\anibuild.exe" -ForegroundColor Cyan
```

Replace:

```powershell
        Write-Host "anime" -ForegroundColor Cyan -NoNewline
        Write-Host " to start."
```

with:

```powershell
        Write-Host "anibuild" -ForegroundColor Cyan -NoNewline
        Write-Host " to start. (anime remains available for compatibility.)"
```

- [ ] **Step 6: Parse-check the PowerShell script**

Run on a PowerShell-capable host:

```powershell
pwsh -NoProfile -Command '$tokens = $errors = $null; [System.Management.Automation.Language.Parser]::ParseFile("scripts/install.ps1", [ref] $tokens, [ref] $errors) | Out-Null; if ($errors.Count) { $errors | ForEach-Object { Write-Error $_ }; exit 1 }'
```

Expected: exit code `0` with no parse errors.

- [ ] **Step 7: Commit the Windows installer change**

```bash
git add scripts/install.ps1
git commit -m "feat: add anibuild Windows launcher"
```

### Task 3: Document `anibuild` as the user-facing command

**Files:**
- Modify: `README.md:3-7`
- Modify: `README.md:32-45`
- Modify: `README.md:94-100`

**Interfaces:**
- Consumes: the installers’ guarantee that both `anibuild` and `anime` invoke the same application.
- Produces: user documentation which leads with `anibuild` and labels `anime` as a compatibility command.

- [ ] **Step 1: Add a failing documentation assertion**

Run:

```sh
if grep -Fq 'Run `anibuild` to start Anime.' README.md && grep -Fq '`anime` remains available for compatibility.' README.md; then
  exit 0
fi
printf '%s\n' 'README does not yet document anibuild as primary with anime compatibility.' >&2
exit 1
```

Expected: exit code `1` with the printed message.

- [ ] **Step 2: Update the title and getting-started commands**

Change the title to:

```html
<h1>Anime (<code>anibuild</code>)</h1>
```

Add this paragraph before the source-build command block:

```markdown
After installing a release, start Anime with `anibuild`. The `anime` command remains available for compatibility.
```

Keep source build commands using `--bin anime`, since that is the Cargo target, but update prose so it distinguishes source target names from the installed command.

- [ ] **Step 3: Update compatibility wording**

Replace the paragraph that currently tells users to use `anime` for the product flow with:

```markdown
The package also retains the upstream-compatible `xai-grok-pager` binary. The installed product command is `anibuild`; `anime` remains an equivalent compatibility command.
```

- [ ] **Step 4: Run the documentation assertion and inspect the relevant references**

Run:

```sh
grep -F 'Run `anibuild` to start Anime.' README.md
grep -F '`anime` remains available for compatibility.' README.md
grep -nE 'anibuild|--bin anime|xai-grok-pager' README.md
```

Expected: both exact documentation claims are present; Cargo commands continue to name `--bin anime` only where they refer to the build target.

- [ ] **Step 5: Commit documentation changes**

```bash
git add README.md
git commit -m "docs: recommend anibuild command"
```

### Task 4: Run final installer verification without touching a user installation

**Files:**
- Verify: `scripts/install.sh`
- Verify: `scripts/install.ps1`
- Verify: `README.md`

**Interfaces:**
- Consumes: the finished installers and documentation.
- Produces: fresh evidence that the POSIX launcher contract, available PowerShell parsing, and documentation requirements hold.

- [ ] **Step 1: Verify POSIX shell syntax**

Run: `sh -n scripts/install.sh`

Expected: exit code `0`.

- [ ] **Step 2: Verify POSIX launcher behavior in a temporary directory**

Run:

```sh
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
printf '#!/usr/bin/env sh\nprintf "Anime launched\\n"\n' > "$fixture/anime"
chmod +x "$fixture/anime"
ln -s anime "$fixture/anibuild"
[ -L "$fixture/anibuild" ]
[ "$(readlink "$fixture/anibuild")" = anime ]
[ "$("$fixture/anibuild")" = 'Anime launched' ]
```

Expected: exit code `0`.

- [ ] **Step 3: Parse-check PowerShell when available, otherwise report that limitation**

Run:

```sh
if command -v pwsh >/dev/null 2>&1; then
  pwsh -NoProfile -Command '$tokens = $errors = $null; [System.Management.Automation.Language.Parser]::ParseFile("scripts/install.ps1", [ref] $tokens, [ref] $errors) | Out-Null; if ($errors.Count) { $errors | ForEach-Object { Write-Error $_ }; exit 1 }'
else
  printf '%s\n' 'PowerShell is unavailable on this host; Windows parser verification requires a Windows CI runner.'
fi
```

Expected: parser exit code `0`, or the explicit host limitation message.

- [ ] **Step 4: Verify documentation and scope**

Run:

```sh
grep -F 'Run `anibuild` to start Anime.' README.md
grep -F '`anime` remains available for compatibility.' README.md
git diff --check
git status --short
```

Expected: documentation assertions pass; `git diff --check` reports no whitespace errors; the pre-existing logo modification remains untouched.

- [ ] **Step 5: Do not commit or push without explicit user approval**

Leave commits uncreated if the user did not explicitly ask for commits. Report verification output and the files changed.
