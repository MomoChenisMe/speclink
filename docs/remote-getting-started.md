# Speclink Remote Server, Desktop, and CLI Getting Started

[繁體中文](remote-getting-started.zh-TW.md) · **English**

This document takes you from nothing to a working Remote Store. The path is: start a server → complete first-run setup → grant membership → create credentials → connect the Desktop and CLI → recover when the connection is lost.

It documents only the entries that work today. [Project Capability Status](product-status.md) is the canon for what you can use now. For the path that needs no server at all, see [Local Repo Getting Started](getting-started.md).

The `speclink-server` used here is the official **reference implementation**. It exists so you can start out of the box, or try the remote features directly. Remote mode itself does not bind you to it: Host and Protocol are public contracts, so you can build your own server on the Speclink engine and plug in your own authentication, database, and permission model. The steps below belong to the official server. What the CLI and the desktop app do once connected follows the contracts, so it stays the same on your own server.

## 1. Before you begin / 開始前

You need:

- A machine that can run the server — either Node.js or Docker, without cloning this repo.
- The `speclink` CLI (installation is in the [README](../README.md#install--安裝)).
- The Desktop app as well, if you want the graphical interface.

Understand the split first: a Remote Store **never** syncs into a second writable local truth. An Agent with a checkout reads the read-only `.speclink/context/`, and writes always go through Host commands.

## 2. Start a server / 啟動 Server

The shortest path is npx, anywhere Node runs:

```bash
npx @speclink/server
```

**Expected output**: one first-run line carrying a setup link that is shown only once —

```text
Speclink 首次啟動：開啟 http://localhost:8080/setup?token=spk_setup_… 完成初始設定（此連結 24 小時內有效，且僅顯示這一次）。
```

It also creates `speclink-data/` in the current directory, holding `config.yaml` (the single configuration source the launcher derives from environment variables), `store.db`, and `identity.db`. To change the port or the backend, use environment variables:

```bash
SPECLINK_PORT=8099 SPECLINK_STORE=serverfs npx @speclink/server
```

Check that it is alive:

```bash
curl -o /dev/null -w "%{http_code}\n" http://localhost:8080/healthz
```

**Expected output**: `200`.

Two other paths exist. Production deployments use Docker or compose; see [Server Deployment](server-deployment.zh-TW.md) (Traditional Chinese only). Development inside a checkout of this repo uses `npm run dev`, or `npm run dev:server` for the backend alone; see [Development Entries](development.md).

On the checkout path, `npm run dev` **builds the current checkout's `speclink-cli` first**. It starts the server and Desktop only after that build succeeds. A failed build exits non-zero and leaves no long-running process behind. That order is deliberate. It makes sure the CLI you check with in section 7 came from the same source as this server.

Running the `speclink-server` binary directly **requires** `--config <yaml>` — it does not assemble configuration from environment variables, which is the launcher's job.

Leave that terminal running; `Ctrl+C` stops it cleanly.

## 3. Complete first-run setup / 完成首次設定

Open the `/setup?token=…` link the terminal printed:

![The Speclink server first-run setup screen](assets/screenshots/server-setup.png)

(Screenshots are captured with the interface in Traditional Chinese; the interface language is switchable.)

Create three things on that screen:

1. The first administrator: email, display name, and password.
2. The first Project: this guide uses the key `demo`.
3. The first Repo: this guide uses the key `backend`.

The completion page shows the service URL, project key, and store key. With this example, the project-scoped URL (the connection address `link` needs in section 7) is:

```text
http://localhost:8080/api/speclink/v1/projects/demo
```

After a restart, `/setup` closes and the setup token is not reprinted — **that means identity and Store are persisted, not that startup failed**.

## 4. Grant Project membership / 授予 Project membership

Being able to sign in is not the same as seeing a project; membership is a separate layer. Two ways:

**From the console** — sign in as the administrator and open `/admin/users`, then add the person to the `demo` project and pick their role. Without membership, a user who signs in still gets a `404` when reading that project's resources. This is deliberate. An unauthorized caller cannot infer from the status code whether the project exists.

**From the command line** (headless, scriptable) — invite a new member with the project attached:

```bash
speclink-server invite --config ./speclink-data/config.yaml \
  --email teammate@example.com --display "Teammate" --project demo
```

**Expected output**: a one-time acceptance URL to hand to the invitee so they can finish registering.

Other administrative actions have headless entries too: `speclink-server user suspend|reactivate`, `speclink-server token revoke`, `speclink-server project create`, and `speclink-server repo create`. All of them need `--config` pointing at the configuration file in the data directory.

## 5. Create a PAT safely / 安全建立 PAT

Everyday sign-in does not need a PAT — the CLI uses device authorization by default. PATs are for CI and environments without a browser.

Create one on the `/account` page. Create there sends one same-origin browser-API request: POST `/api/speclink/v1/web/account/tokens`. That endpoint accepts POST only. It is not a page you can open, so do not GET it in a browser.

When creating one: scope it to only the projects it needs, set an expiry, and note that **the full value is shown exactly once**. Once you leave the page it cannot be retrieved, only revoked and reissued:

```bash
speclink-server token revoke --config ./speclink-data/config.yaml <token-id>
```

Never put a PAT in a repo, in shell history, or in a screenshot.

## 6. Connect the Desktop app / 連接 Desktop app

In Desktop, open Settings → Servers, add a connection with the service URL, and sign in. The default is the device flow, which needs one browser authorization; where no browser is available you can paste a PAT instead. Credentials live in the OS keychain, never in project files.

The connection list, device login, PAT fallback, and logout all work. **The full Remote Workspace is not closed yet**, though: signing in does not yet open a remote board the way a local workspace does. All three session shapes — spec-only (remote specs with no local checkout), remote-plus-checkout (remote specs alongside local source), and offline/conflict handling — are still in later changes. The Desktop Remote Workspace row of [Project Capability Status](product-status.md) itemizes the current completeness and the remaining gaps. Trust that row rather than this guide.

## 7. Connect the CLI / 連接 CLI

In the repo directory you want to attach to the remote:

```bash
speclink link http://localhost:8080/api/speclink/v1/projects/demo --repo backend
speclink auth login
speclink auth status
```

`link` records the connection and this repo's registered name on the remote. `auth login` uses device authorization by default and prints a code and a URL. `auth status` reports the current identity and the repo validation result.

Once attached, the everyday verbs are unchanged — `list`, `show`, `status`, `instructions`, `new`, `task`, `in-progress`, `discuss`, `review`, `verify`, and `archive` all have a remote arm that acts on the remote Store instead of the local one. Two exceptions: `demo` is local-only and `claim` is remote-only, and each refuses explicitly in the wrong mode rather than silently rerouting. Verb mode assignments are in the [Verb and Flag Contract](verb-contract.md).

With a checkout present, the Agent reads the read-only `.speclink/context/` projection. **Never edit it directly** — that is not a remote write, and the next command will reject the projection as modified. Re-fetch instructions to refresh it.

**When verifying inside a checkout of this repo, always go through the wrapper rather than the `speclink` on your PATH**:

```bash
npm run cli -- auth status
npm run --silent cli -- list --json
```

`npm run cli -- <args>` always runs this checkout's CLI, building it first when the binary is missing, and never falls back to PATH. Add `--silent` when you need machine-readable stdout with nothing else mixed in. The Node SDK is not an npm workspace member, so verifying it takes `npm --prefix crates/speclink-node test`.

## 8. Recover from a lost connection / 失聯恢復

| Symptom | How to get back |
| --- | --- |
| Credentials expired or were revoked | Re-run `speclink auth login`; the device flow re-authorizes. |
| `auth status` reports repo validation failure | The remote's registered repo name does not match `link --repo`; re-run `speclink link` with the right name. |
| The projection is marked STALE or modified | Do not repair it by hand; re-run `speclink instructions ... --json` to rematerialize it. |
| The server moved to a new URL | `speclink unlink`, then `link` again with the new URL. |
| You want this device fully signed out | `speclink auth logout` — revokes this device's credential family and clears local credentials. |
| The server will not start, or `/healthz` is not 200 | Read the error in the server terminal; invalid configuration exits non-zero rather than starting in a degraded state. |

## 9. Reset and clean up / 重置與清除

For the npx path all state lives in `speclink-data/`, so deleting it returns you to a fresh `/setup`:

```bash
rm -rf ./speclink-data
```

For in-repo development with `npm run dev`, the equivalent is `npm run dev:reset` (which clears only `.dev/` and leaves `.env` alone). For migrations and scheduled backups that preserve data, see [Server Backup and Restore](server-backup.zh-TW.md) (Traditional Chinese only).

## 10. Troubleshooting / 故障排除

- **`/setup` will not open and the token is gone**: that is expected — setup completed once, and the token is shown only once. Clear the data directory to start over.
- **Running `speclink-server` directly says `missing required argument --config`**: the binary does not read environment variables. Use `npx @speclink/server`, or pass `--config` yourself.
- **A member can sign in but sees no project**: membership was not granted; return to section 4.
- **A PAT was lost**: it cannot be retrieved. Revoke and reissue.
- **CLI behavior does not match the documentation**: usually a stale `speclink` on your PATH (installed from another checkout, or an installed build that fell behind). Inside this repo use `npm run cli -- <args>`, which only ever runs the current checkout's CLI; to see which one you have, compare the engine versions from `speclink --version` and `npm run --silent cli -- --version`.
- **Unsure whether a remote capability exists**: check the Local and Remote table in [Project Capability Status](product-status.md), where one row shows both sides.
