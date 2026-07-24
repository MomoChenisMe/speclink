# Remote Server, Desktop, and CLI Getting Started

[繁體中文](remote-getting-started.zh-TW.md) · **English**

This guide uses the repository's local development harness to build a Remote Workspace shared by Desktop and CLI from clean data. Before deploying to a production environment, also read the Traditional Chinese guides for [Server deployment](server-deployment.zh-TW.md), [Store drivers](server-store-drivers.zh-TW.md), and [backup/restore](server-backup.zh-TW.md).

The currently delivered path is a single-node Server, Remote Desktop Workspace, and Remote CLI. MCP, Copilot Tools, SSO, and Cluster mode are outside this guide; see [Product capability status](product-status.md) for the current boundary.

## 1. Before you begin / 開始前

You need:

- A stable Rust toolchain, Node.js, and npm.
- A macOS, Windows, or Linux development environment capable of running Tauri Desktop.
- Two terminals: one for the long-running Server plus Desktop and one for CLI testing.

The examples consistently use:

| Name | Example | Purpose |
| --- | --- | --- |
| Server base URL | `http://localhost:8080` | Browser login, account, and administration pages |
| Project key | `demo` | Project in the Server registry |
| Repo key | `backend` | Repo under `demo` |
| project-scoped URL | `http://localhost:8080/api/speclink/v1/projects/demo` | CLI and Client Protocol connection |

Do not interchange these URLs. Browser pages such as `/account` and `/admin` are relative to the Server base URL; the project-scoped URL binds Remote CLI and Desktop clients to a Project.

Preserve any uncommitted repository work. Run the Remote CLI smoke test in a separate test directory; do not run `speclink link` at the product repository root.

## 2. Start a clean development server / 啟動全新開發 Server

From the Speclink repository root, clear existing local development data:

```bash
npm run dev:reset
```

This removes only `.dev/`, not `.env`. To test the default SQLite configuration, first confirm that `.env` does not override `SPECLINK_*` values.

Start the harness:

```bash
npm run dev
```

It generates `.dev/config.yaml`, builds `speclink-cli` and the Desktop frontend from the current checkout, and only then starts both `speclink-server` and Tauri Desktop. If the CLI build fails, the harness exits non-zero and leaves no long-running process behind — this is what guarantees that the CLI you verify with in section 7 comes from the same source as Server and Desktop. The terminal prints a first-run-only URL:

```text
http://localhost:8080/setup?token=...
```

Keep this terminal running. If either child process exits, the harness stops the other; press `Ctrl+C` for a normal shutdown.

## 3. Complete first-run setup / 完成首次設定

Open the `/setup?token=...` URL printed in the terminal and create:

1. The first Admin: email, display name, and password.
2. The first Project, using key `demo` in this guide.
3. The first Repo, using key `backend` in this guide.

The completion page shows the public URL, Project key, and Repo key. The example project-scoped URL is:

```text
http://localhost:8080/api/speclink/v1/projects/demo
```

After an ordinary restart, `/setup` is closed and the setup token is not printed again. That means identity and Store data persisted; it is not a startup failure.

## 4. Grant Project membership / 授予 Project membership

Creating the Project/Repo registry does not grant account access. Server Admin is installation administration; Project membership controls Project data access. Even the first Admin does not bypass membership checks.

1. Open [http://localhost:8080/admin/users](http://localhost:8080/admin/users).
2. If redirected, sign in with the Admin email and password created during `/setup`.
3. Find the account that will actually sign in to Desktop.
4. Select `demo` in its membership form.
5. Choose a role:
   - `editor`: read/write access, suitable for this smoke test.
   - `reader`: read access; writes are disabled or rejected by the Server.
6. Select “加入／更新” (add/update).

To test a regular user, create an invitation from `/admin/users`, assign Project membership, and let the recipient open the one-time invitation URL and set a password. Do not share the Admin's PAT or password.

## 5. Create a PAT safely / 安全建立 PAT

A PAT (Personal Access Token) is a credential for CLI and the Desktop fallback. Open:

[http://localhost:8080/account](http://localhost:8080/account)

After signing in, enter a recognizable name such as `local-cli` in the Personal Access Tokens form, optionally leave expiry blank, and create the PAT. Its plaintext is shown once; copy it immediately to a secure location.

`/account` is a single-page app page. Create a PAT from its Personal Access Tokens form; the SPA submits it to the browser API **POST `/api/speclink/v1/web/account/tokens`** and shows the plaintext once. `/account/tokens` itself is not a browsable page — opening it directly with GET returns a JSON 404.

Never put a PAT in:

- A URL or shell argument.
- `.speclink.yaml`, the repository, documentation, or logs.
- Desktop localStorage.

Interactive `speclink auth login` reads it from stdin. Desktop stores credentials in the OS Keychain.

## 6. Open a Remote Desktop Workspace / 開啟 Remote Desktop Workspace

In Desktop:

1. Select “新增 Workspace” (add Workspace).
2. Select “Speclink Server.”
3. Enter the Server base URL, `http://localhost:8080`, not the project-scoped URL.
4. Prefer Device Login:
   - Desktop opens `/activate` in the browser.
   - Sign in if required.
   - Confirm that the user code matches Desktop, then approve it.
5. If Device Login is unavailable, choose the PAT fallback and paste the PAT created from `/account`.
6. Select `demo` / `backend`.

If the list says that the account has no Project/Repo membership, the signed-in account lacks `demo` membership. Grant `reader` or `editor` at `/admin/users`, then close and reopen the chooser or go back one step to reload it.

Choose a workspace form:

- **spec-only**: skip checkout and use Server specifications directly; suitable for PM/PO work.
- **remote + checkout**: select a local Git repository. Desktop validates and writes a remote marker when none exists; an existing marker must match the selected Server origin and Repo.

A remote tab is created only after a successful handshake. Tab restoration, role capabilities, and offline state remain attached to the Remote Workspace and never silently fall back to local mode.

## 7. Connect and smoke-test the Remote CLI / 連接並測試 Remote CLI

The `npm run dev` step in section 2 already built the CLI from the current checkout. Running it through `npm run cli` needs no CLI installation and never picks up a different version from `PATH`:

- At the Speclink repository root: `npm run cli -- <args>`.
- From any other directory, including this section's test directory: `npm --prefix /path/to/speclink run cli -- <args>`. `--prefix` only selects which checkout provides the CLI; the CLI still acts on the directory you are in.
- To parse `--json` output directly, add `--silent`: `npm run --silent cli -- <args>` keeps npm lifecycle messages out of stdout.
- Everything after `--` is forwarded to the CLI verbatim; omitting `--` lets npm consume the flags instead.

Run the following in a separate test directory, not at the Speclink product repository root:

```bash
mkdir -p /tmp/speclink-remote-smoke
cd /tmp/speclink-remote-smoke
npm --prefix /path/to/speclink run cli -- link \
  http://localhost:8080/api/speclink/v1/projects/demo \
  --repo backend
npm --prefix /path/to/speclink run cli -- auth login
```

Replace `/path/to/speclink` with the actual absolute repository path. Paste the PAT only when `auth login` prompts, keeping it out of shell history. Test reads:

```bash
npm --prefix /path/to/speclink run cli -- auth status
npm --prefix /path/to/speclink run cli -- list
npm --prefix /path/to/speclink run --silent cli -- list --json
```

As an `editor`, test a minimal write and structural checks:

```bash
npm --prefix /path/to/speclink run cli -- new change remote-smoke-test
npm --prefix /path/to/speclink run cli -- status --change remote-smoke-test
npm --prefix /path/to/speclink run cli -- validate remote-smoke-test
npm --prefix /path/to/speclink run cli -- analyze remote-smoke-test
```

Return to Desktop and confirm that `remote-smoke-test` appears on the same `demo` / `backend` board. Remote CLI writes a remote binding and read-only Context Projection in the test directory; specification writes still pass through the Server Host.

## 8. Verify persistence and recovery / 驗證持久化與恢復

### Ordinary restart

Press `Ctrl+C` in the terminal running `npm run dev`, then run:

```bash
npm run dev
```

Expect:

- No new setup token.
- Projects, Repos, membership, accounts, and changes remain.
- Saved Desktop remote tabs can be restored.

### offline / stale

Keep a remote tab open and stop the Server. Expect:

- The remote tab retains its last snapshot and displays offline / stale state.
- The snapshot is read-only; writes fail immediately without a hidden local write queue.
- Local tabs remain usable.

Run `npm run dev` again. Desktop should converge through Query plus ETag, resubscribe to SSE, reload current data, and clear stale state.

### Expired credentials

If the Server returns 401 or the credential family is revoked, Desktop displays a reauthentication state. Sign in again from Server settings; the existing remote tab should recover in place and must not become a local workspace.

## 9. Reset the development environment / 重置開發環境

Stop `npm run dev`, then perform a complete local reset:

```bash
npm run dev:reset
npm run dev
```

A new `/setup?token=...` should appear. This deletes the default SQLite Store and identity data under `.dev/`, so old accounts, PATs, membership, Projects/Repos, and changes no longer work.

When `.env` selects PostgreSQL, `npm run dev:reset` does not delete the external database. Drop and recreate that database separately for a complete reset.

## 10. Troubleshooting / 故障排除

| Symptom | Cause | Resolution |
| --- | --- | --- |
| Opening `/account/tokens` directly returns 404 | It is a browser-API endpoint, not a page | Open `/account` and create a PAT from the Personal Access Tokens form |
| Desktop says there is no Project/Repo membership | Registry resources exist, but the account lacks Project membership; Admin does not bypass it | Grant `reader` or `editor` to the actual account at `/admin/users`, then reopen the chooser |
| The Project/Repo list remains empty | Membership was granted to another account, or the chooser has not reloaded | Check the email at `/account`, correct membership, and reopen the chooser; sign out/in if needed |
| 401 / reauthentication required | The PAT, access token, or device credential family expired or was revoked | Sign in again from Desktop Server settings, or create a new PAT at `/account` |
| The Server is offline and the tab is stale | SSE/HTTP is temporarily unavailable | Keep the snapshot read-only, restart the Server, and wait for Query plus ETag convergence |
| The checkout marker conflicts | `.speclink.yaml` points to another Server origin or Repo | Do not mask it with manual edits; choose the correct checkout or use the Desktop conflict choices |
| CLI reports not logged in | The directory is linked, but that Server origin has no credential | Run `npm --prefix /path/to/speclink run cli -- auth login` in the same test directory |
| CLI behaviour disagrees with Server / Desktop | The `speclink` on `PATH` is a different version or comes from another checkout | Use `npm run cli` (or `npm --prefix <checkout> run cli`) to run the current checkout's binary; no `PATH` change needed |
| `npm run cli` reports it cannot run the checkout CLI | The current checkout has no debug binary yet | Run `npm run dev` or `cargo build -p speclink-cli`; the message names both the binary path and the working directory |
| No setup token appears after restart | Setup is complete and data still exists | Sign in at `/account` or `/admin`; only a full `npm run dev:reset` reopens setup |
| Old Desktop connections fail after reset | Identity, Project, and credentials were deleted | Complete setup again, grant membership, then add or sign in to the connection again |

After completing this path, the same Remote Project/Repo has been exercised through Server, Desktop, and CLI. For full role, event recovery, and multi-tab coverage, run the project tests traced from the [Phase 3 acceptance spec](../openspec/specs/phase3-acceptance/spec.md).
