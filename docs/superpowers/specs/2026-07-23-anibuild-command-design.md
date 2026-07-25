# `anibuild` Command Design

## Objective

Make `anibuild` the documented command users run after installing Anime, while preserving `anime` as a compatibility command.

## Scope

- Update the POSIX installer (`scripts/install.sh`) to install the release executable as `anime` and create an `anibuild` symlink in the same directory.
- Update the PowerShell installer (`scripts/install.ps1`) to install `anime.exe` and create `anibuild.exe` as a hard link. If creating the hard link fails, copy `anime.exe` to `anibuild.exe` instead.
- Update installer completion output and the README to recommend `anibuild`.
- Do not rename Cargo binary targets, application-internal identifiers, configuration directories, or existing `anime` command behavior.

## Rationale

Keeping `anime` avoids breaking existing scripts and user installations. A same-directory link makes `anibuild` available whenever `anime` is on `PATH`, without editing shell startup files or depending on a particular shell.

## Behavior

### POSIX installer

1. Extract the release archive and copy its `anime` executable to `<install-dir>/anime`.
2. Mark `<install-dir>/anime` executable.
3. Replace `<install-dir>/anibuild` with a symbolic link whose target is `anime`.
4. Report both installed paths and instruct the user to start the application with `anibuild`.

The relative symlink target keeps the installation portable if the full install directory is moved as a unit.

### Windows installer

1. Copy `anime.exe` to `<install-dir>\anime.exe`.
2. Replace any existing `<install-dir>\anibuild.exe`.
3. Try `New-Item -ItemType HardLink` pointing `anibuild.exe` to `anime.exe`.
4. If a hard link cannot be created, copy `anime.exe` to `anibuild.exe` and continue successfully.
5. Report both commands and instruct the user to start the application with `anibuild`.

The fallback avoids requiring privileges or filesystem features beyond ordinary file copying.

## Error Handling

- Existing aliases are deliberately replaced on reinstall, matching the existing overwrite behavior for `anime` / `anime.exe`.
- Installer failures while creating the alias must stop the POSIX install; on Windows, only failure of both hard-link creation and copy fallback stops the install.
- No user shell configuration is modified.

## Verification

- POSIX: run `sh -n scripts/install.sh`; run the relevant extraction/install block against a temporary directory and assert `anibuild` is a symlink resolving to the installed `anime` executable.
- Windows: run a PowerShell parser check if PowerShell is available; otherwise inspect the script changes and include a CI-compatible PowerShell smoke command in the implementation plan.
- Confirm the README and installer messages name `anibuild` as the primary command and `anime` as compatibility.

## Non-goals

- Removing or replacing the `anime` executable.
- Changing `grok`-named internal code, upstream compatibility paths, config locations, or authentication storage.
- Publishing a release, pushing commits, or invoking GitHub Actions.
