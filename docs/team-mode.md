# Team Mode (Remote Store)

In team mode, spec documents and change state live in a **team system** (a server embedding the Speclink engine), while your code and git stay local. The `speclink` CLI becomes a thin client of the [verb contract](verb-contract.md): every verb you already use — `list`, `status`, `instructions`, `task done`, `discuss …` — keeps the same output shape; only the storage behind it moves.

This document covers the client side: connecting a repo, authenticating, how repo identity works, and what the error messages mean. Server-side implementation belongs to the team system and is specified by the verb contract.

## The `remote:` section and mode resolution

One section of `.speclink.yaml` decides the mode:

| State of the repo | Mode |
|---|---|
| No `remote:` section in `.speclink.yaml` | **fs** — the classic local `openspec/` layout; nothing changes |
| `remote:` section present | **remote** — verbs call the team system's contract endpoints |
| Both the `remote:` section and `openspec/` present | **remote wins**; each command prints one stderr warning about the coexistence (typically migration leftovers — remove one side) |

The section has two fields:

```yaml
# .speclink.yaml — committed, like .lfsconfig: every clone gets the same binding
tools:
  - claude
remote:
  url: https://team.example.com/api/speclink/v1/projects/erp   # project-scoped; may be omitted and supplied via SPECLINK_STORE_URL
  repo: backend    # optional on single-repo projects — this repo's registered name
```

- `url` carries the **project scope** — one team server can host many projects; your repo binds to exactly one.
- `SPECLINK_STORE_URL` overrides (or supplies) the url for one shell or one CI job (staging server, port-forward, or a url the team prefers not to commit). It never turns an fs workspace into a remote one — the section is the mode signal.
- The section present with **no url anywhere** (neither `remote.url` nor `SPECLINK_STORE_URL`) is an explicit failure naming both settings — the CLI never silently falls back to fs mode.
- Credentials never live in this file (or anywhere else in the repo).

### Migrating from `.speclink.remote.yaml`

Earlier builds used a standalone connection file, `.speclink.remote.yaml`. It is no longer read: a leftover file triggers one stderr migration warning per command and does not affect the mode — an unmigrated project runs in fs mode (and fails loudly when no `openspec/` tree exists). To migrate, move the `url` and `repo` values into the `remote:` section of `.speclink.yaml` and delete the old file.

Note: `init --store remote`, `link`, and `unlink` rewrite `.speclink.yaml` preserving every field's **value**, but YAML comments do not survive the rewrite.

## Getting connected

### Fresh repo: `speclink init --store remote`

```bash
speclink init --store remote --url https://team.example.com/api/speclink/v1/projects/erp --repo backend
```

This performs the **workspace init only**: the `CLAUDE.md`/`AGENTS.md` marker block (in its remote wording), the skills, `.claude/settings.json`, `.gitignore` entries, and a `.speclink.yaml` carrying the `remote:` section. It deliberately does **not** create an `openspec/` tree — documents live on the server.

### Existing repo: `speclink link` / `speclink unlink`

```bash
speclink link https://team.example.com/api/speclink/v1/projects/erp --repo backend
speclink unlink   # removes the remote section; back to fs mode
```

`link` writes (or updates) the `remote:` section of `.speclink.yaml`, keeping the other fields — an existing `tools:` list survives untouched. `unlink` removes the section only, never the file. When you are already logged in, `link` validates immediately (see below); when you are not, it writes the section and reminds you to run `speclink auth login` — offline linking never blocks, the first verb validates instead.

### Authentication: `speclink auth login` / `speclink auth status`

```bash
speclink auth login             # paste a PAT interactively
speclink auth login --token-stdin   # scripted (CI)
speclink auth status            # who am I, is my repo registered, any fork warning
```

- The PAT is issued in the team system's UI. Treat it like an SSH private key: if it may have leaked, revoke it there immediately.
- `login` validates the token against the server **before** storing it — a rejected token is never written.
- Credentials are stored per server origin in the **user-level** config directory (`credentials.yaml`, mode 0600 on Unix; Windows relies on the user-profile ACL). One login covers every project and every clone on the same server.
- `SPECLINK_TOKEN` overrides the credentials file for CI/headless runs. An empty value counts as unset.
- Not logged in and running a remote verb? You get one line — `not logged in to <origin> — run `speclink auth login`` — and a non-zero exit. Nothing is guessed, nothing is cached.

## Repo identity — declared once, carried automatically, verified per verb

Three layers connect your repo to the project (you only feel them when something is wrong):

1. **Repo → project**: the connection `url` is project-scoped.
2. **Repo identity**: the `remote.repo` field is this repo's registered name in the project's repos registry (managed server-side). `init`/`link` validate it immediately when credentials exist — an unknown name fails loudly and lists the registered names, and the section is not written. Single-repo projects may omit `repo`; the server resolves it.
3. **Change ownership**: every change belongs to **exactly one repo** (v1 rule). Creating a change assigns it to your repo; `speclink list` shows only your repo's changes; running a change-scoped verb from the wrong repo fails with a message naming both repos. Cross-repo work is split into several changes, one per repo.

Every request carries your repo name automatically (`X-Speclink-Repo`); you never pass it per command.

**Advisory fork warning**: when the server's registry records a reference git URL for your repo and your local `git remote origin` differs, `link` and `auth status` print one stderr line suggesting you may be on a fork or mirror. It never changes the result or the exit code; without a reference value, or outside a git directory, the check is silently skipped.

## Error messages at a glance

Every server error is translated into one semantic line with a suggested action — the CLI never surfaces a bare HTTP status. The full canonical catalog lives in the [verb contract §4](verb-contract.md#4-error-reason-catalog); the ones you will actually meet:

| You see | It means | Do this |
|---|---|---|
| `not logged in to <origin> — run \`speclink auth login\`` | no credentials for this server | log in |
| `credentials expired/revoked — run \`speclink auth login\`` | the PAT is no longer valid | re-issue and log in again; the CLI never retries a dead token |
| `repo is not registered in this project (available: …)` | `remote.repo` names something the registry doesn't know | fix `remote.repo` or re-run `speclink link` |
| `change belongs to repo 'backend' but you are 'frontend' — run this verb from the owning repo` | wrong repo for this change | switch to the owning repo |
| `change is held by <user> — coordinate, or re-claim if it was released` | someone else claimed it (or claimed it first) | talk to them / `speclink claim` after release |
| `content changed since you read it — re-read it and re-apply your edit` | optimistic-concurrency conflict on an artifact write | re-read (`speclink artifact cat`), redo the edit |
| `change is <state> — wait for the in-flight operation to finish, then retry` | a server-side operation (e.g. an ingest merge) holds the change | wait, then retry — your claim survives |
| `waiting for <gate> approval in the team system — ask the approver` | a gate (proposal or archive) is pending | approval happens in the team system UI |
| `N task(s) still open — finish them before archiving` | archive with unchecked tasks | `speclink task done …` first |
| `server unreachable — check the connection url (\`remote.url\` in .speclink.yaml or SPECLINK_STORE_URL)` | can't connect | fix connectivity; there is **no offline mode and no cached answer** |
| `server does not support this CLI's API version — upgrade the CLI or the server` | contract version mismatch | upgrade whichever side is behind |

## Upgrading from a purely local project

`speclink store push` (bulk migration of an existing `openspec/` tree into an empty remote project) is planned but **not shipped yet**. Until then, the manual path:

1. In the team system, create the project and register your repo(s); have the PM re-create the **active** changes there (or re-run `/speclink-propose` against the remote store — often the cleaner option, since proposals get re-validated).
2. For canonical specs, paste each `openspec/specs/<capability>/spec.md` into the team system (or its import facility, if it has one).
3. In the repo: `speclink link <url> --repo <name>`, then `speclink auth login`.
4. Verify with `speclink list` — you should see the server's changes, not the local tree.
5. Remove the local `openspec/` directory (it is fully represented on the server now) — until you do, every command reminds you that both exist and remote wins.
6. Keep `openspec/changes/archive/` history in git if you value the local audit trail; the server's history starts at migration.

Rolling back is symmetric: `speclink unlink` restores fs mode against whatever `openspec/` content is still in the repo.

## What stays local in remote mode

- `.speclink.yaml` (tools list, workspace options, and the `remote:` connection section itself) — the workspace still decides which AI harness files are generated.
- `.speclink/` work data (touched-file records), `.gitignore`, generated skills and marker blocks.
- Workflow **policy** (`locale`, `tdd`, `audit`, schema, context, rules) is served by the team system (`GET /config`) — there is no local `openspec/config.yaml` in remote mode, so policy can never fork between the web side and your repo.
