# Speclink Development Entry Points

[繁體中文](development.zh-TW.md) · **English**

This document is for developers who clone the source tree and users who want to self-host the backend. It covers the five one-command entry points at the repo root: the full dev environment, server only, desktop only, resetting local state, and running the CLI from this same checkout. It ends with the bypass steps for unsigned installers downloaded from Releases.

## Prerequisites / 前置需求

- A stable Rust toolchain (`cargo` available)
- Node.js and npm; after the first clone, run this at the repo root:

```bash
npm install
```

- `npm run dev` and `npm run dev:desktop` launch a Tauri desktop window and need the platform's Tauri system dependencies (Xcode Command Line Tools on macOS). `npm run dev:server` and `npm run cli` alone do not.

## `npm run dev` — full dev environment / 整套開發環境

- **Purpose**: start the complete dev environment in one command — build the CLI from the current checkout, build the desktop frontend assets, then start `speclink-server` and the desktop tauri dev together.
- **Prerequisites**: everything listed above. Configuration comes from `.env` at the repo root (see `.env.example`; without `.env` all defaults apply: sqlite, `.dev/store.db`, `127.0.0.1:8080`).
- **Expected observable results**: the terminal first shows `speclink dev: 建置當前 checkout 的 speclink-cli…` and `speclink dev: 建置當前前端資產…`; on first startup the server prints a line containing a `/setup?token=` link (shown only once — open it to complete initial setup); the desktop window opens. Ctrl+C shuts down both server and desktop with no leftover processes. State persists in the gitignored `.dev/`.

## `npm run dev:server` — server only / 只跑後端

- **Purpose**: validate the dev configuration and start `speclink-server` only — for self-hosting trials and pure backend development. No CLI build, no frontend build, no desktop window.
- **Prerequisites**: Rust toolchain and `npm install`. Starts with zero configuration (sqlite by default); invalid configuration (e.g. `SPECLINK_STORE_DRIVER=postgres` without `SPECLINK_POSTGRES_URL`) refuses to start with a non-zero exit code and the same error message as `npm run dev`.
- **Expected observable results**: in a fresh environment the terminal prints a link containing `/setup?token=`; no build steps run and no desktop window appears. After Ctrl+C no processes remain; `.dev/` persistence behaves exactly like `npm run dev`.

## `npm run dev:desktop` — desktop only / 只跑桌面

- **Purpose**: build the desktop frontend first (vite emits `dist/`), then start tauri dev — for desktop development and local-mode trials. Does not start a server and requires no remote configuration.
- **Prerequisites**: all prerequisites (including Tauri system dependencies). Configuration validation is shared with `npm run dev` — an invalid `.env` (e.g. postgres without `SPECLINK_POSTGRES_URL`) refuses to start with a non-zero exit code here too.
- **Expected observable results**: the terminal first shows `speclink dev: 建置當前前端資產…` and the vite build output; the desktop window opens only after the build finishes and shows the UI built from the current sources (tauri dev loads the static `dist/`, so every start rebuilds it to avoid a stale UI). If the frontend build fails, the command exits non-zero and no window opens. The window runs in local mode and can browse this repo's `openspec/` board — it works even when no server is running on the machine.

## `npm run dev:reset` — reset local dev state / 重置本機開發狀態

- **Purpose**: wipe `.dev/` (the dev server configuration and databases) so the next `npm run dev` returns to a fresh `/setup`. Builds nothing; never touches `.env` or `deploy/`.
- **Prerequisites**: none (idempotent success when `.dev/` does not exist).
- **Expected observable results**: the terminal prints `speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。` and exits immediately. Note: postgres data lives in the external database and is not part of the reset.

## `npm run cli -- <args>` — CLI from this checkout / checkout 內 CLI

- **Purpose**: always run `target/debug/speclink` from this same checkout, so "start the environment" and "verify with the same-version CLI" operate on one source tree; the `speclink` installed on PATH is never used.
- **Prerequisites**: Rust toolchain. When the binary has not been built yet, the wrapper first runs `cargo build -p speclink-cli` at the checkout root automatically, then executes the result (a failed build exits non-zero without running any CLI).
- **Expected observable results**:

```bash
npm run cli -- --version
```

On the first run (or after deleting the binary) cargo build progress appears on stderr first, followed by the version output. Add `--silent` when you need a pure machine-readable stdout:

```bash
npm run --silent cli -- list --json
```

stdout contains only the CLI's JSON payload (camelCase fields); build progress never mixes in. The CLI's exit code, stdin/stdout/stderr, and working directory (`INIT_CWD`) are all forwarded transparently.

## Unsigned installer bypass / 下載安裝檔的未簽章放行

The desktop installers are delivered by the desktop-installer-and-updater change and **do not appear on existing GitHub Releases yet**. Once delivered, the asset forms per the desktop-release spec are: macOS dmg (one each for aarch64 and x86_64), a Windows NSIS installer (x86_64), and Linux AppImage and deb (x86_64 and aarch64), all listed in `SHA256SUMS.txt`. Until the project configures OS code-signing keys, these installers are unsigned and need a one-time manual bypass on first launch:

### macOS

1. Open the dmg and drag **Speclink** into Applications.
2. The first launch is blocked ("cannot be opened because Apple cannot check it…").
3. Go to System Settings > Privacy & Security, find the message about Speclink being blocked in the Security section, click "Open Anyway", then confirm with "Open".
4. This bypass is needed only once; afterwards the app opens normally.

### Windows

1. When running the NSIS installer, SmartScreen shows "Windows protected your PC".
2. Click "More info", then "Run anyway" to continue the installation.

### Linux

AppImage and deb have no corresponding signature gate: make the AppImage executable (`chmod +x`) and install the deb the usual way for your distribution.
