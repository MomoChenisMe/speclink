# Speclink Development Entry Points

[繁體中文](development.zh-TW.md) · **English**

This document is for developers who clone the source tree, and for users who self-host the backend. It covers five one-command entry points at the repo root. They are: the full dev environment, server only, desktop only, a local-state reset, and the CLI from this checkout. It ends with the bypass steps for unsigned installers from Releases.

## Prerequisites / 前置需求

- A stable Rust toolchain (`cargo` available)
- Node.js and npm; after the first clone, run this at the repo root:

```bash
npm install
```

- `npm run dev` and `npm run dev:desktop` launch a Tauri desktop window and need the platform's Tauri system dependencies (Xcode Command Line Tools on macOS). `npm run dev:server` and `npm run cli` alone do not.

## `npm run dev` — full dev environment / 整套開發環境

- **Purpose**: start the complete dev environment in one command. It builds the CLI from the current checkout, builds the desktop frontend assets, then starts `speclink-server` and the desktop tauri dev together.
- **Prerequisites**: everything listed above. Configuration comes from `.env` at the repo root (see `.env.example`; without `.env` all defaults apply: sqlite, `.dev/store.db`, `127.0.0.1:8080`).
- **Expected observable results**: the terminal first shows `speclink dev: 建置當前 checkout 的 speclink-cli…` and `speclink dev: 建置當前前端資產…`. On the first startup the server prints one line with a `/setup?token=` link. It prints that link only once, so open it to complete the initial setup. The desktop window then opens. Ctrl+C stops both the server and the desktop with no leftover processes. State persists in the gitignored `.dev/`.

## `npm run dev:server` — server only / 只跑後端

- **Purpose**: validate the dev configuration and start `speclink-server` only — for self-hosting trials and pure backend development. No CLI build, no frontend build, no desktop window.
- **Prerequisites**: Rust toolchain and `npm install`. Starts with zero configuration (sqlite by default); invalid configuration (e.g. `SPECLINK_STORE_DRIVER=postgres` without `SPECLINK_POSTGRES_URL`) refuses to start with a non-zero exit code and the same error message as `npm run dev`.
- **Expected observable results**: in a fresh environment the terminal prints a link containing `/setup?token=`; no build steps run and no desktop window appears. After Ctrl+C no processes remain; `.dev/` persistence behaves exactly like `npm run dev`.

## `npm run dev:desktop` — desktop only / 只跑桌面

- **Purpose**: build the desktop frontend first (vite emits `dist/`), then start tauri dev — for desktop development and local-mode trials. Does not start a server and requires no remote configuration.
- **Prerequisites**: all prerequisites (including Tauri system dependencies). Configuration validation is shared with `npm run dev` — an invalid `.env` (e.g. postgres without `SPECLINK_POSTGRES_URL`) refuses to start with a non-zero exit code here too.
- **Expected observable results**: the terminal first shows `speclink dev: 建置當前前端資產…` and the vite build output. The desktop window opens only after the build finishes, and it shows the UI built from the current sources. Tauri dev loads the static `dist/`, so every start rebuilds it and you never see a stale UI. If the frontend build fails, the command exits non-zero and no window opens. The window runs in local mode and browses this repo's `openspec/` board. It works even when no server runs on the machine.

## `npm run dev:reset` — reset local dev state / 重置本機開發狀態

- **Purpose**: wipe `.dev/` (the dev server configuration and databases) so the next `npm run dev` returns to a fresh `/setup`. Builds nothing; never touches `.env` or `deploy/`.
- **Prerequisites**: none (idempotent success when `.dev/` does not exist).
- **Expected observable results**: the terminal prints `speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。` and exits immediately. Note: postgres data lives in the external database and is not part of the reset.

## `npm run cli -- <args>` — CLI from this checkout / checkout 內 CLI

- **Purpose**: always run `target/debug/speclink` from this same checkout. "Start the environment" and "check with the same-version CLI" then operate on one source tree. The wrapper never uses the `speclink` on your PATH.
- **Prerequisites**: Rust toolchain. When the binary does not exist yet, the wrapper runs `cargo build -p speclink-cli` at the checkout root, then runs the result. A failed build exits non-zero and runs no CLI.
- **Expected observable results**:

```bash
npm run cli -- --version
```

On the first run (or after deleting the binary) cargo build progress appears on stderr first, followed by the version output. Add `--silent` when you need a pure machine-readable stdout:

```bash
npm run --silent cli -- list --json
```

stdout contains only the CLI's JSON payload (camelCase fields); build progress never mixes in. The CLI's exit code, stdin/stdout/stderr, and working directory (`INIT_CWD`) are all forwarded transparently.

## Tests / 測試

Run the whole test surface at once (Rust workspace, the three frontend packages, scripts, and the Node SDK build and tests):

```bash
npm run test:all
```

To run one area at a time:

```bash
cargo test --workspace                      # Rust: engine, CLI, Host, Store drivers, Server
npm test -w apps/desktop                    # Desktop frontend
npm test -w packages/ui                     # Shared UI
npm test -w apps/server-web                 # Server console frontend
node --test "scripts/**/*.test.mjs"         # Repo scripts
npm --prefix crates/speclink-node test      # Node SDK (build first: npm --prefix crates/speclink-node run build)
```

The Node SDK is not an npm workspace member, so it needs `--prefix` with a path; `--workspace @speclink/engine` finds nothing. Desktop tests need the sidecar and the server console `dist/` in place, and `npm run test:all` already covers both steps.

Golden and CLI integration tests protect the CLI's human-readable output and `--json` shape. Per-area test prerequisites and current limits for Server, Store, and Desktop are in [Project Capability Status](product-status.md).

## Unsigned installer bypass / 下載安裝檔的未簽章放行

The desktop installers are on [Releases](https://github.com/MomoChenisMe/speclink/releases/latest). They are: macOS dmg for aarch64 and x86_64, a Windows NSIS installer for x86_64, and Linux AppImage and deb for x86_64 and aarch64. `SHA256SUMS.txt` lists them all. Signing status differs per platform, and only Windows needs a manual bypass:

### macOS

Code-signed and notarized, so opening the dmg and dragging Speclink into Applications is all it takes — no bypass step. The bypass (System Settings > Privacy & Security > "Open Anyway") is only needed for an unsigned bundle you built locally yourself.

### Windows

1. When running the NSIS installer, SmartScreen shows "Windows protected your PC".
2. Click "More info", then "Run anyway" to continue the installation.

### Linux

AppImage and deb have no corresponding signature gate: make the AppImage executable (`chmod +x`) and install the deb the usual way for your distribution.
