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

## <a id="desktop-tags"></a>Publish desktop builds with a tag

For this fork, finish implementation and the required pre-merge checks, merge the feature branch into `main`, then build installers from that exact `main` commit. Corporate Proxy acceptance happens after installing that build at the company. Record the source commit and installer filename; a dev window outside the company does not substitute. Merging does not complete the manual task or archive the change.

The Updates page checks `mosutw/speclink` and separately displays the original author's `MomoChenisMe/speclink` version and project link. Upstream versions are informational, never installation candidates. Missing signed update packages lead to manual Release downloads; TLS and signature failures remain errors. Metadata excludes drafts, prereleases and CLI-only releases, with a limit of 100 entries. Four-part build tags are prereleases in the current workflow, so they are not advertised as stable updates. Bundled version and changelog information stays visible offline.

### Shared security standard data

Settings → Security standard data offers a local default and a custom absolute directory, including Windows UNC such as `\\fileserver\team\speclink-standards`. macOS/Linux use an already mounted path. The app does not mount SMB or store share credentials. Desktop and CLI share the device setting; a remote server uses its own service account's setting, not a client's path.

A maintainer downloads/imports complete packages once and grants members read access to the files and lock files. Read-only shares allow review and export to a writable destination, but disable download/import. Deletion from custom roots is disabled because one device cannot know other members' references; maintain shared versions outside the app. Disconnection never causes a local download fallback. Switching or restoring the default does not move or delete files; active data, scan or knowledge jobs block path changes.

Only standards are shared. Private reports, credentials, device references and each checkout's `.security_kb/` stay local, so knowledge snapshots still consume local space. Review does not download standards; shares still require filesystem connectivity. Tests use local shared-root fixtures, not a verified corporate NAS/SMB environment.

History writers lock a dedicated `.checks-locks/` directory in the spec store so Windows can move proposals while retaining exclusion. Stop all old writers before upgrading: mixing old proposal-internal locks and new locks is unsupported. Lock files ignore themselves in Git and do not appear as proposal documents.

`Desktop Build` supports manual builds and automatic releases on `desktop-vX.Y.Z` tags. Merge the workflow changes into `main` first; the tagged commit must include this workflow. The tag version must match both `apps/desktop/src-tauri/tauri.conf.json` (`version`) and `Cargo.toml` (`[workspace.package].version`), or the workflow stops before building. Commit the corresponding `Cargo.lock` changes when bumping the version.

Prerelease tags such as `desktop-v0.9.0-build.m1` are also supported. Both App and CLI must use the full `0.9.0-build.m1` version. Tags with a prerelease suffix create a GitHub prerelease.

A four-part tag `desktop-vX.Y.Z.N` maps to the valid App/CLI version `X.Y.Z-build.N`: for example, `desktop-v0.10.0.1` maps to `0.10.0-build.1`. Update the App, Cargo.toml, and Cargo.lock to that version first. The GitHub release retains the four-part tag, uses the actual App version for asset names, and is marked as a prerelease. Build numbers with leading zeros are rejected.

Both manual and tag runs wait for the latest push CI on the default branch for the exact same commit before starting the four-platform build. A different SHA, a pull request run, or an older successful run cannot substitute. Failed, cancelled, or skipped CI stops the workflow immediately. Missing or unfinished CI is checked every 30 seconds for up to 40 minutes; lookup errors and timeout also block the build, with no bypass. Only a lightweight preparation job runs while waiting; the four platform jobs are not allocated. An unmerged feature branch cannot bypass this check through a manual build. Changes to the CI gate scripts run full CI.

This example uses the stable version `0.9.0`. Each release needs a new tag that matches the source version:

```bash
git switch main
git pull --ff-only
git tag -a desktop-v0.9.0 -m "Speclink Desktop 0.9.0"
git push origin desktop-v0.9.0
```

After all four platforms succeed, the repository's Releases page receives five assets: a macOS universal DMG, Windows x64 installer, Windows x64 portable ZIP, and Linux amd64/aarch64 AppImages. The Windows ZIP retains the extraction, CLI configuration, and relocation checks. A failed platform prevents the release job from running; rerun failed jobs in Actions. Manual runs still create only Artifacts retained for 14 days, without a Release.

The workflow uses the built-in `GITHUB_TOKEN`; only the release job receives `contents: write`. No additional private key is needed. macOS uses ad-hoc signing, and Windows has no publisher certificate; see the installation instructions below. This channel has no signed updater packages or `latest.json`; download updates manually. The app's updater URL now points to origin's Latest Release, but this desktop channel provides no signed update metadata. Fork auto-updates still need an update manifest and packages signed with a key matching the app's public key.

`desktop-v*` does not trigger the existing full `v*` Release, npm publishing, or Docker publishing. Desktop releases do not replace the repository's existing Latest Release. The original signed release workflow is unchanged.

The general CI and Desktop Build are separate workflows. A push or PR that changes only the explicitly listed desktop delivery configuration, version/CI scope scripts, related tests, and documentation runs `Workflow and scripts checks` only. It does not start application builds, smoke tests, React/Rust tests, Docker, or PostgreSQL. Any source code, dependency, or unlisted file change still runs the original full CI; an unknown change scope also runs full CI. Desktop tags continue to run the actual four-platform packaging pipeline.

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
