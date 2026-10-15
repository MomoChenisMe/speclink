# Development Environment

[繁體中文](development.zh-TW.md) · **English**

This document is for developers who clone the source tree. It explains the five one-command entry points at the repo root, how to run the tests, and the repo layout. The last section shows how to open unsigned installers from Releases.

If you only want to host a server outside a checkout, you do not need this document. Use `npx @speclink/server` and see [Remote Getting Started](remote-getting-started.md).

## <a id="prerequisites"></a>Prerequisites

- A stable Rust toolchain (`cargo` must work).
- Node.js 21 or later, and npm. CI runs each test area separately on Node 20. `npm run test:all` needs Node 21 or later (see [Tests](#tests)).
- After the first clone, install the frontend dependencies at the repo root:

  ```bash
  npm install
  ```

- The entry points that start `speclink-server` (`npm run dev`, `npm run dev:server`) and the Rust tests need one build of the server web console first:

  ```bash
  npm run build -w apps/server-web
  ```

  `speclink-server` does not compile without the `apps/server-web/dist` folder, and in development it reads that folder directly. When you change code in `apps/server-web`, build it again. Otherwise the console shows the old UI.

- The entry points that open a desktop window (`npm run dev`, `npm run dev:desktop`) need the Tauri system dependencies. On macOS, install the Xcode Command Line Tools. On Linux, install the GTK and WebKit packages (the list is in the "Install Tauri system dependencies" step of `.github/workflows/ci.yml`). For other cases, see the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## <a id="dev"></a>`npm run dev`: full dev environment

- **Purpose**: start the full dev environment with one command. It builds the `speclink-cli` of the current checkout first. If the build succeeds, it starts `speclink-server` and the desktop app (tauri dev) together. The Vite dev server supplies the desktop frontend, so frontend edits reload at once.
- **Prerequisites**: all items in Prerequisites. The settings come from `.env` at the repo root (copy `.env.example` and edit it). Without `.env`, all defaults apply: sqlite, data in `.dev/`, and the server on `127.0.0.1:8080`. If `.env` is not valid (for example, `SPECLINK_STORE_DRIVER=postgres` without `SPECLINK_POSTGRES_URL`), the command exits non-zero and starts nothing.
- **Expected observable results**:
  1. The terminal first shows `speclink dev: 建置當前 checkout 的 speclink-cli…`. If the build fails, the command exits non-zero and leaves no background process.
  2. On the first start, the server prints one line: `Speclink 首次啟動：開啟 http://localhost:8080/setup?token=… 完成初始設定（此連結 24 小時內有效，且僅顯示這一次）。`. Open that link to complete the initial setup.
  3. The desktop side puts the sidecar (the CLI inside the desktop app; see [`npm run dev:desktop`](#dev-desktop)) in place, then opens the desktop window.
  4. Ctrl+C stops the server and the desktop app together and leaves no background process. If one side stops first, the other side also stops. Data stays in `.dev/` (not in version control).

The CLI build comes first on purpose. When you then check things with [`npm run cli`](#cli), the CLI and the server come from the same source.

## <a id="dev-server"></a>`npm run dev:server`: server only

- **Purpose**: start only `speclink-server`, for backend work in this checkout. It does not build the CLI and opens no desktop window.
- **Prerequisites**: Rust, `npm install`, and a built `apps/server-web/dist`. The `.env` check is the same as in `npm run dev`. A bad `.env` makes the command exit non-zero.
- **Expected observable results**: on the first run, cargo compiles the server, which takes some time. When `.dev/` is new, the terminal prints a link that contains `/setup?token=`. No desktop window opens. After Ctrl+C, no background process remains. This entry point shares the `.dev/` data with `npm run dev`.

## <a id="dev-desktop"></a>`npm run dev:desktop`: desktop app only

- **Purpose**: start only the desktop app (tauri dev), for desktop work and local-mode trials. It starts no server and needs no remote settings.
- **Prerequisites**: Rust, `npm install`, and the Tauri system dependencies. The `.env` check is the same as in `npm run dev`.
- **Expected observable results**:
  1. It builds `speclink-cli` and copies it to the location that the desktop app needs. This CLI inside the desktop app is the sidecar. The terminal shows `sidecar 佈署完成（debug）：…`, or `sidecar 內容未變，跳過複製：…` when nothing changed. If this step fails, the command exits non-zero and opens no window.
  2. The Vite dev server starts, then the desktop window opens. The UI comes from the current source, and frontend edits reload in place.
  3. The window runs in local mode and can open the `openspec/` board of this repo. It works with no server on the machine.

## <a id="dev-reset"></a>`npm run dev:reset`: remove local dev state

- **Purpose**: delete `.dev/` (the dev server settings and databases), so the next `npm run dev` starts again at a fresh `/setup`. It builds nothing and never touches `.env` or `deploy/`.
- **Prerequisites**: none. It also succeeds when `.dev/` does not exist.
- **Expected observable results**: the terminal shows `speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。` and the command exits at once. PostgreSQL data lives in the external database, so this command does not remove it. To start over, drop and create that database yourself.

## <a id="cli"></a>`npm run cli -- <args>`: CLI from this checkout

- **Purpose**: run `target/debug/speclink` from this checkout. The wrapper never uses the `speclink` on your PATH. "Start the environment" and "check with the same CLI version" then use the same source.
- **Prerequisites**: a Rust toolchain. When the binary does not exist, the wrapper runs `cargo build -p speclink-cli` at the checkout root, then runs the result. If the build fails, it exits non-zero and runs no CLI. When the binary exists, the wrapper does **not** build it again. After you change Rust code, run `cargo build -p speclink-cli` yourself. Otherwise you run the old version.
- **Expected observable results**:

  ```bash
  npm run cli -- --version
  ```

  ```text
  > speclink-workspace@0.1.3 cli
  > node scripts/dev/cli.mjs --version

  speclink 0.8.0 (arm64, engine v1.41.0)
  ```

  The version and the architecture depend on your checkout and machine. On the first run (or after you delete the binary), stderr shows the cargo build first.

Add `--silent` when you need a clean stdout (for example, JSON for another program). npm then adds no lines of its own:

```bash
npm run --silent cli -- list --json
```

To use the CLI of this checkout in another folder (for example, a test repo):

```bash
npm --prefix <speclink-checkout> run cli -- list
```

The CLI runs in the folder where you type the command. The wrapper passes on the exit code, stdin, stdout, and stderr unchanged.

### Check a remote environment

When you check a remote environment from a checkout (for example, `auth status` or `list --json`), always use `npm run cli -- <args>`. Do not use the `speclink` on your PATH. That binary can come from another checkout or from an old installed version. Compare the engine versions with these two lines:

```bash
speclink --version
npm run --silent cli -- --version
```

If the two `engine v…` values differ, the `speclink` on your PATH does not come from this source.

## <a id="tests"></a>Tests

In a fresh clone, the Rust tests do not compile until three build outputs exist. Run these steps in the same order as CI:

```bash
npm install
npm run build -w apps/desktop                              # the desktop crate needs apps/desktop/dist
npm run build -w apps/server-web                           # speclink-server needs apps/server-web/dist
node scripts/desktop/desktop-sidecar.mjs --profile debug   # the desktop crate needs the sidecar
npm run test:all
```

`npm run test:all` runs the repo scripts, the three frontend packages, the Rust workspace, and the Node SDK (install, build, test), in that order. It builds `apps/server-web/dist` again by itself. It does not build `apps/desktop/dist` and does not put the sidecar in place, so a fresh clone needs the three lines above first. You do not need them again unless you delete those outputs.

To run one area:

```bash
node --test "scripts/**/*.test.mjs"                     # repo scripts (Node 21 or later)
node --test scripts/*.test.mjs scripts/*/*.test.mjs     # the same, for Node 20
npm test -w packages/ui                                 # shared UI
npm test -w apps/desktop                                # desktop frontend
npm test -w apps/server-web                             # server web console
cargo test --workspace                                  # Rust: engine, CLI, Host, Store drivers, server, desktop app
npm --prefix crates/adapters/speclink-node ci           # Node SDK: install dependencies first
npm --prefix crates/adapters/speclink-node run build    #           then build the .node file
npm --prefix crates/adapters/speclink-node test         #           then run the tests
```

- `cargo test --workspace` takes a long time. When you change only one crate, use `cargo test -p <crate>`.
- The PostgreSQL Store driver tests skip when `SPECLINK_TEST_POSTGRES_URL` is not set. To run them, point it at a working database (CI uses PostgreSQL 15).
- The Node SDK is not an npm workspace member, so you must give its path with `npm --prefix`. The `-w` option does not find it.

## <a id="layout"></a>Repo layout

| Path | Contents |
| --- | --- |
| `apps/desktop/` | The desktop app (Tauri): `src/` is the frontend; `core/` and `src-tauri/` are two Rust crates |
| `apps/server-web/` | The server web console (React). Its build goes into `speclink-server` |
| `crates/engine/` | The engine core: `speclink-core` (verbs, workflow, skill generation) and `speclink-fs` (reads and writes the local `openspec/`) |
| `crates/protocol/` | The remote protocol: `speclink-protocol` (data types that the client and the server share) and `speclink-remote` (the remote client and sign-in for the CLI and the desktop app) |
| `crates/store/` | The server Store drivers: the shared interface `speclink-store`, plus sqlite, file (serverfs), and postgres drivers |
| `crates/host/` | `speclink-host` (the shared layer between the engine and the CLI, desktop app, and server) and `speclink-server` (the official reference server) |
| `crates/adapters/` | The entry points: `speclink-cli` (the `speclink` command) and `speclink-node` (the Node SDK `@speclink/engine`) |
| `packages/ui/` | UI components that the desktop app and the web console share |
| `packages/cli-npm/`, `packages/server-npm/` | The npm packages `@speclink/cli` and `@speclink/server`. Each platform binary ships as a separate package |
| `integrations/claude-code/` | The Claude Code mod (`speclink-skills`: skill bar and side panel) |
| `scripts/` | Scripts for the dev entry points, documentation checks, desktop builds, npm packaging, and releases |
| `deploy/` | Docker Compose files (one for sqlite, one for postgres) |
| `docs/` | User documentation |
| `openspec/` | The specs, changes, and discussions of Speclink itself |

## <a id="unsigned"></a>Open unsigned installers

The desktop installers are on [Releases](https://github.com/MomoChenisMe/speclink/releases/latest):

| Platform | File name | Bypass needed |
| --- | --- | --- |
| macOS (one file for Apple Silicon and Intel) | `Speclink_<version>_universal.dmg` | No: the project signs it, and Apple notarizes it |
| Windows (x64) | `Speclink_<version>_x64-setup.exe` | Yes: the installer has no code signature |
| Linux (x86_64) | `Speclink_<version>_amd64.AppImage` | No: make the file executable |
| Linux (aarch64) | `Speclink_<version>_aarch64.AppImage` | No: make the file executable |

### macOS

Open the dmg and drag Speclink into Applications. You do not need a bypass. macOS blocks only an unsigned app that you build on your own machine:

1. Open the app once. When macOS says that it cannot open the app, close the message.
2. Go to System Settings > Privacy & Security. Find Speclink in the Security area and click "Open Anyway".

### Windows

1. Run `Speclink_<version>_x64-setup.exe`. SmartScreen shows "Windows protected your PC".
2. Click "More info", then click "Run anyway" to continue the installation.

### Linux

```bash
chmod +x Speclink_<version>_amd64.AppImage
./Speclink_<version>_amd64.AppImage
```

On an aarch64 machine, use `Speclink_<version>_aarch64.AppImage`. An AppImage needs FUSE to run. If you see a FUSE error, install `libfuse2` first (on Ubuntu 24.04 and later, the package is `libfuse2t64`).
