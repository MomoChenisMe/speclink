# Speclink Verb Contract (v1)

This document is the **canonical reference** for the REST contract between Speclink thin clients (the `speclink` CLI in remote mode, and any other host acting as a client) and a server that embeds the Speclink engine (via the Rust crate or the Node SDK). Team systems implement the server side of this contract; the CLI's remote mode is its first consumer.

> **Canon vs. server freedom.** The state-transition semantics and the error `reason` catalog defined here are **contract canon**: every server implementation must adjudicate them identically — hosts must not invent variant semantics. A reference implementation of the adjudication logic will live in the optional `speclink-team` module (future work; see the design record, discussion round 17). Server implementation freedom is limited to exactly two areas:
>
> 1. **Gate policy configuration** — *which* transitions require human approval (zero, one, or many gates; who may approve). The *semantics* of a gated transition are canon; the *policy data* is the server's.
> 2. **Repos registry management** — how repos are registered to a project, and the admin surface for it.
>
> Everything else — payload shapes, status codes, `reason` values, guard conditions, ownership rules — is fixed by this document.

---

## 1. Conventions

### 1.1 Base URL and project scope

The connection URL stored in `.speclink.remote.yaml` is **project-scoped**:

```
https://team.example.com/api/speclink/v1/projects/{project}
```

All endpoint paths in this document are relative to that base. The project scope is therefore part of the connection, never a query parameter. Version `v1` appears in the path for routing hygiene; the authoritative version check is the header below.

### 1.2 Required headers

Every request carries:

| Header | Value | Rules |
|---|---|---|
| `Authorization` | `Bearer <PAT>` | Personal access token, issued by the host system. Tokens never appear in URLs, query strings, or request bodies. |
| `X-Speclink-Api-Version` | `1` | Contract major version. A server that does not support the requested version — or receives no version header — **must reject** the request (`400 api_version_unsupported`, body lists `supportedVersions`). Servers never guess. |
| `X-Speclink-Repo` | registered repo name | The caller's repo identity (from the connection file's `repo` field). Required when the project has more than one registered repo; on a single-repo project the server resolves an absent header to the sole registered repo. Absent on a multi-repo project → `400 repo_required`. |

### 1.3 Payload style

- All request and response bodies are JSON with **camelCase** field names, aligned with the field names of the CLI's existing `--json` output (the fs-mode payloads are the naming authority).
- Servers may include **additional** fields in responses; clients must ignore unknown fields (forward compatibility). Clients render their output from the documented fields only, so CLI stdout stays byte-identical between fs and remote modes.
- No caching fallback: a client that cannot reach the server fails loudly. There is no offline mode, no outbox, no stale local answer.

### 1.4 Error envelope

Every non-2xx response (except transport-level 5xx where the server may be unable to produce a body) carries:

```json
{ "reason": "<machine-readable-token>", "message": "<human text>", "...context": "…" }
```

- `reason` is the **only** field clients branch on. `message` is advisory; clients must never parse it.
- 409 responses **always** carry a `reason` (hard rule).
- The CLI translates every non-2xx into a single-line semantic message plus a suggested action. **Bare HTTP status codes are never surfaced to the user or the agent as the primary error output.**

### 1.5 Optimistic concurrency (`If-Match`)

- Reading an artifact returns its `version` (a monotonically increasing integer, starting at `1` on creation).
- Writing an artifact (`PUT`) **must** carry `If-Match: <version>` — the version obtained when the content was read.
  - `If-Match: 0` means "create; the artifact must not exist yet". If it already exists → `409 version_conflict` (with `currentVersion`).
  - Stale version → `409 version_conflict`.
  - A `PUT` **without** `If-Match` is rejected with `428 if_match_required`. There is no force-write, no "last writer wins" mode, and no way to disable this check.
- Lifecycle transitions (claim, archive) do not require the client to send a state version; their atomicity is a server-side guard (conditional update). `GET /changes/{name}` exposes `statusVersion` for hosts that need it (kanban rendering, host-side locking); the CLI does not send it.

---

## 2. Repo identity and change ownership

**v1 rule: one change belongs to exactly one repo.**

- **Creation** — `POST /changes` assigns the change's repo from the request's `X-Speclink-Repo` (single-repo projects: the sole registered repo). There is no request field to choose a different repo; the contract offers no cross-repo change shape.
- **Listing** — `GET /changes` returns only the changes belonging to the requesting repo. Changes owned by sibling repos of the same project do not appear.
- **Every change-scoped verb** validates the chain: PAT valid → repo ∈ project registry → `change.repo == request repo`. A mismatch fails loudly with `403 repo_mismatch`, and the body names **both** repos (`changeRepo`, `requestRepo`).
- **Cross-repo work** is handled by splitting into multiple changes, one per repo (a discussion can fan out into several changes; lineage is carried by the discussion slug).

The repos registry is server-managed (server freedom #2). `GET /whoami` exposes the registered repos — including an optional `gitUrl` reference value per repo, which clients use only for an advisory fork/mirror warning (never as identity; URL-based inference was explicitly rejected).

---

## 3. Change lifecycle (contract canon)

### 3.1 States

The wire-visible lifecycle of a change is one of six values:

```
drafting → (review) → ready → applying → archived
                                  ↕
                                busy (transient)
```

| `lifecycle` | Meaning |
|---|---|
| `drafting` | Artifacts are being authored; the change is not yet claimable. |
| `review` | All required artifacts are complete and a **proposal gate** is enabled and pending. Exists only on projects whose gate policy enables it. |
| `ready` | Complete, gate (if any) satisfied, unclaimed. Claimable. |
| `applying` | Claimed by an engineer (`claimedBy` set). Task completion and implementation-stage writes happen here. |
| `busy` | A transient server-side operation holds the change (e.g. a host-driven artifact merge). Ownership is retained; writes are refused with `409 change_busy` until the operation completes. |
| `archived` | Terminal. Deltas merged into the canonical specs. |

`lifecycle` is **derived by the engine** from facts (artifact completeness, gate approvals, claim state) — hosts render it, they do not store an independent status of their own invention.

### 3.2 Transitions and guards

| Transition | Trigger | Guard (canon) |
|---|---|---|
| *(create)* → `drafting` | `POST /changes`, or promoting a discussion | name unique in project → else `409 already_exists` |
| `drafting` → `review` | last required artifact completed, proposal gate **enabled** | artifact completeness is engine-derived |
| `drafting` → `ready` | last required artifact completed, proposal gate **disabled** | — |
| `review` → `ready` | gate approved (host UI; approval endpoints are host surface, semantics are canon) | approval recorded for current content |
| `review` → `drafting` | gate rejected, **or any artifact write while in `review`** | a content change voids the pending/granted approval — approvals apply to the content that was reviewed, never silently to newer content |
| `ready` → `applying` | `POST /changes/{name}/claim` | **atomic** compare-and-set: succeeds iff unclaimed and `ready`. Claimed by someone → `409 ownership_lost` (with `claimedBy`); gate pending → `409 gate_pending`; incomplete/transient → `409 change_busy` |
| `applying` → `applying` | `POST .../tasks/{taskId}/done`, artifact `PUT` | writer must be the owner: `claimedBy == caller`, else `409 ownership_lost` |
| `applying` → `ready` | ownership released/revoked (host governance; no v1 CLI verb) | subsequent writes by the former owner → `409 ownership_lost` |
| `applying` ↔ `busy` | server-side transient operation begins/ends | ownership retained throughout; writes during `busy` → `409 change_busy` |
| `applying` → `archived` | `POST /changes/{name}/archive` | **check-all-then-apply**, atomic: all tasks done (else `409 tasks_incomplete`) → every delta's base spec version current (else `409 version_conflict` with `conflicts[]`) → archive gate (if enabled) approved (else `409 gate_pending`) → merge and archive in one transaction. No partial state on failure. |

Notes:

- **Verify** is a local discipline in v1 (the `/speclink-verify` skill runs client-side against fetched artifacts); it is not a server state. If a future revision adds a verify gate, it arrives as a contract version bump together with the `speclink-team` reference implementation.
- **Ingest** is not an endpoint. Requirement changes mid-work rewrite artifacts through `PUT` with `If-Match` — the same optimistic-concurrency path as any other write. `busy`/`change_busy` exists for hosts whose UI performs server-side merges.
- `archived` is terminal and read-only. Canon: **any write verb against an archived change returns `409 change_busy` with `lifecycle: "archived"` in the body**; the CLI reports "this change is archived". Reads (`GET` change/artifacts) remain valid as long as the host keeps the change addressable; a host that removes archived changes from the active namespace returns `404 not_found`.

### 3.3 Adjudication is canon

For every 409 in the table above, *which* reason comes back for *which* factual situation is fixed by this document. Two servers given the same facts must return the same `reason`. This is what keeps CLI guidance (and any agent acting on it) portable across hosts.

---

## 4. Error reason catalog

Complete catalog. Every error path a client can hit maps to exactly one row; there are no undefined error paths.

| HTTP | `reason` | When | Context fields | CLI suggested action |
|---|---|---|---|---|
| 400 | `api_version_unsupported` | version header missing or major not supported | `supportedVersions` | "Server does not support this CLI's API version — upgrade the CLI or the server." |
| 400 | `repo_required` | multi-repo project, no `X-Speclink-Repo` | `availableRepos` | "Set `repo:` in `.speclink.remote.yaml` (see `speclink link`)." |
| 400 | `bad_request` | malformed payload (client bug) | — | "Internal CLI error — update speclink or report a bug." (fail loud, not agent-interpreted) |
| 401 | `token_missing` | no `Authorization` header | — | "Run `speclink auth login`." (the CLI normally catches this locally before sending) |
| 401 | `token_invalid` | token unknown | — | "Credentials rejected — run `speclink auth login`." |
| 401 | `token_expired` | token past expiry | — | "Credentials expired — run `speclink auth login`." |
| 401 | `token_revoked` | token revoked | — | "Credentials revoked — run `speclink auth login`." |
| 403 | `access_denied` | valid token, no access to this project | — | "Your account has no access to this project — ask a project admin." |
| 403 | `repo_unknown` | `X-Speclink-Repo` not in the project registry | `availableRepos` | "Repo not registered in this project. Available: <list>. Fix `repo:` or re-run `speclink link`." |
| 403 | `repo_mismatch` | change belongs to another repo | `changeRepo`, `requestRepo` | "Change '<name>' belongs to repo '<changeRepo>'; you are '<requestRepo>'. Run this verb from the owning repo." |
| 404 | `not_found` | change / artifact / task / discussion / capability / language document does not exist | `resource`, `name` | "'<name>' not found — run `speclink list` (or the matching list verb) to check the name." |
| 409 | `already_exists` | `POST /changes` or `POST /discussions` with a taken name/slug | `name` | "Name already in use — pick another." |
| 409 | `version_conflict` | stale `If-Match` on artifact write; or archive delta base behind canonical spec | `currentVersion` — or `conflicts[]` (`{capability, baseVersion, currentVersion}`) on archive | "Content changed since you read it — re-read (`speclink artifact cat`) and re-apply your edit." Archive: "Spec(s) <capabilities> moved since propose — resolve in the team system, then retry." |
| 409 | `ownership_lost` | claim on an already-claimed change; or a write by a non-owner | `claimedBy` | "Change is held by <claimedBy> — coordinate, or re-claim if it was released." |
| 409 | `change_busy` | verb against a change in a transient or non-eligible state (mid-merge, `drafting` claim attempt, archived) | `lifecycle` | "Change is <lifecycle> — wait for the in-flight operation to finish, then retry." |
| 409 | `gate_pending` | claim or archive while a gate approval is outstanding | `gate` (`"proposal"` \| `"archive"`) | "Waiting for <gate> approval in the team system — ask the approver." |
| 409 | `tasks_incomplete` | archive with unchecked tasks | `remaining` | "<remaining> task(s) still open — finish them (`speclink task done`) before archiving." |
| 409 | `discussion_archived` | `add-round` / `conclude` / `promote` against an archived discussion | `slug` | "Discussion '<slug>' is archived — restore it in the team system before continuing it." |
| 409 | `project_not_empty` | **reserved** for `store push` (future migration verb) | — | "Target project already contains changes — push requires an empty project." |
| 422 | `validation_failed` | user-supplied content rejected by the engine (invalid change name, artifact id not in the schema, artifact content failing validation) | `errors[]` | Print each validation error verbatim — this is a user-input problem, not a CLI bug. |
| 428 | `if_match_required` | artifact `PUT` without `If-Match` | — | "Internal CLI error — update speclink or report a bug." |
| 5xx / network | *(no envelope guaranteed)* | server crash, timeout, refused connection | — | "Server unavailable — check the connection URL in `.speclink.remote.yaml` (or `SPECLINK_STORE_URL`)." **No retry loop, no cache fallback, stdout stays empty.** |

### 4.1 Example bodies (one per 409 reason)

```json
{ "reason": "already_exists", "message": "change 'add-auth' already exists", "name": "add-auth" }
```

```json
{ "reason": "version_conflict", "message": "design was updated since you read it", "currentVersion": 7 }
```

```json
{ "reason": "version_conflict", "message": "canonical spec moved since propose",
  "conflicts": [ { "capability": "user-auth", "baseVersion": 3, "currentVersion": 5 } ] }
```

```json
{ "reason": "ownership_lost", "message": "change is claimed by chiang", "claimedBy": "chiang" }
```

```json
{ "reason": "change_busy", "message": "change is being updated by the team system", "lifecycle": "busy" }
```

```json
{ "reason": "gate_pending", "message": "archive requires approval", "gate": "archive" }
```

```json
{ "reason": "tasks_incomplete", "message": "3 tasks still open", "remaining": 3 }
```

```json
{ "reason": "discussion_archived", "message": "discussion 'auth-scope' is archived", "slug": "auth-scope" }
```

```json
{ "reason": "project_not_empty", "message": "target project already has changes" }
```

---

## 5. Endpoint reference

Column "CLI verb" is the remote-mode command that maps 1:1 onto the endpoint.

### 5.1 Identity and policy side-car

| Endpoint | CLI verb |
|---|---|
| `GET /whoami` | `speclink auth status` (also used by `link`/`init` validation) |
| `GET /config` | *(internal — policy side-car for instructions, tdd/audit, locale)* |
| `GET /language` | `speclink language show` |

**`GET /whoami` → 200**

```json
{
  "user": { "id": "u_42", "name": "王小明", "handle": "xiaoming" },
  "project": "erp",
  "repos": [
    { "name": "backend",  "gitUrl": "git@github.com:acme/erp-backend.git" },
    { "name": "frontend" }
  ]
}
```

`repos[].gitUrl` is optional — a reference value for the advisory fork/mirror warning only. When absent, clients silently skip that check.

**`GET /config` → 200** — the effective `WorkflowConfig` (already resolved server-side; clients never merge policy layers themselves in remote mode):

```json
{ "schema": "spec-driven", "locale": "tw", "specLocale": "tw",
  "tdd": true, "audit": true,
  "context": "…project context…", "rules": { "proposal": ["…"] } }
```

**`GET /language` → 200** `{ "content": "…LANGUAGE document…" }` — 404 `not_found` (`resource: "language"`) when the project has no shared-vocabulary document; the CLI exits non-zero with a semantic message (skills treat that as "skip vocabulary load").

### 5.2 Canonical specs

| Endpoint | CLI verb |
|---|---|
| `GET /specs` | `speclink list --specs` |
| `GET /specs/{capability}` | *(spec reads inside skills)* |

**`GET /specs` → 200** `{ "specs": [ { "id": "user-auth", "path": "specs/user-auth" } ] }` — `path` is the store-logical location (in fs mode the CLI prints the real directory; the field name and shape are identical).

**`GET /specs/{capability}` → 200** `{ "capability": "user-auth", "content": "…spec.md…", "version": 5 }`

### 5.3 Changes

| Endpoint | CLI verb |
|---|---|
| `GET /changes` | `speclink list` |
| `POST /changes` | `speclink new change` |
| `GET /changes/{name}` | `speclink status` |
| `POST /changes/{name}/claim` | `speclink claim` |
| `POST /changes/{name}/archive` | `speclink archive` |

**`GET /changes` → 200** — filtered by the requesting repo. Optional query `?lifecycle=<value>`.

```json
{ "changes": [
  { "name": "add-rate-limit", "summary": "Protect the public API…",
    "status": "in-progress", "completedTasks": 3, "totalTasks": 9,
    "repo": "backend", "lifecycle": "applying", "claimedBy": "chiang" } ] }
```

`name`/`summary`/`status`/`completedTasks`/`totalTasks` are exactly the fs-mode `speclink list --json` fields (`status` stays task-derived: `done` iff all tasks checked, else `in-progress`); `repo`/`lifecycle`/`claimedBy` are remote-mode additions the CLI does not print in the parity view.

**`POST /changes`** — request `{ "name": "add-rate-limit", "schema": "spec-driven", "description": "…", "fromDiscussion": "some-slug" }` (all but `name` optional; schema defaults to the project's config) → 201:

```json
{ "name": "add-rate-limit", "schema": "spec-driven", "repo": "backend", "lifecycle": "drafting" }
```

Failure paths: `409 already_exists` (name taken), `422 validation_failed` (name violates the engine's naming rules).

**`GET /changes/{name}` → 200** — superset of the fs-mode `speclink status --json` report:

```json
{
  "changeName": "add-rate-limit", "schemaName": "spec-driven",
  "isComplete": false, "applyRequires": ["tasks"],
  "artifacts": [
    { "id": "proposal", "outputPath": "proposal.md", "status": "done",    "version": 3 },
    { "id": "design",   "outputPath": "design.md",   "status": "ready" },
    { "id": "specs",    "outputPath": "specs/**/*.md", "status": "blocked", "missingDeps": ["design"] },
    { "id": "tasks",    "outputPath": "tasks.md",    "status": "blocked", "missingDeps": ["specs"] }
  ],
  "repo": "backend", "lifecycle": "drafting", "statusVersion": 4, "claimedBy": null
}
```

**`POST /changes/{name}/claim`** — empty body → 200 `{ "claimed": true, "claimedBy": "you", "statusVersion": 5 }`. Atomic: two concurrent claims — exactly one succeeds; the loser gets `409 ownership_lost`.

**`POST /changes/{name}/archive`** — empty body → 200:

```json
{ "archived": true, "change": "add-rate-limit",
  "specs": [ { "capability": "api-quota", "version": 6 } ] }
```

Failure paths: `tasks_incomplete` / `version_conflict` (with `conflicts[]`) / `gate_pending` / `repo_mismatch` — see §4.

### 5.4 Artifacts (read/write with optimistic concurrency)

| Endpoint | CLI verb |
|---|---|
| `GET /changes/{name}/artifacts/{artifact}` | `speclink artifact cat` |
| `PUT /changes/{name}/artifacts/{artifact}` | `speclink new artifact` / skill-driven artifact writes |

`{artifact}` ∈ `proposal` \| `design` \| `tasks` \| `specs/{capability}` (path-nested for delta specs).

**GET → 200** `{ "artifact": "design", "content": "…", "version": 7 }` — 404 `not_found` when the artifact has not been created.

**PUT** — request `{ "content": "…full document…" }`, header `If-Match: <version>` (`0` = create-only):

- 200 `{ "artifact": "design", "version": 8 }`
- `409 version_conflict` (stale), `428 if_match_required` (missing header), `409 ownership_lost` (change is `applying` and caller is not the owner), `409 change_busy` (transient/archived), `422 validation_failed` (artifact id not in the change's schema, or content failing engine validation — `errors[]` lists each finding).
- Writes are whole-document replacement. There is no PATCH, no merge on the server for client writes.

### 5.5 Tasks

**`POST /changes/{name}/tasks/{taskId}/done`** — CLI verb `speclink task done`. `{taskId}` is the task's ordinal id exactly as listed in the `tasks` payload of the instructions endpoint (string of a 1-based number in the spec-driven schema).

Request (optional attribution): `{ "touchedFiles": ["src/api/quota.rs"] }` — servers may persist attribution; clients send it when they can compute it.

- 200 `{ "change": "add-rate-limit", "taskId": "3", "taskDesc": "…", "status": "done", "alreadyDone": false, "tasksVersion": 12 }`
  - When the task was already checked, `alreadyDone: true` and nothing changes server-side; the CLI reproduces the fs-mode behavior (error message "Task 3 is already done", non-zero exit).
- Guards: owner-only (`ownership_lost`), applying-state only (`change_busy`), task exists (`not_found` with `resource: "task"`).
- Note: the CLI's fs-parity `--json` stdout keeps its existing keys (`change`, `status`, `task_desc`, `task_id`); the mapping from the contract's camelCase response is the CLI's concern.

### 5.6 Instructions (server-computed)

**`GET /changes/{name}/instructions/{artifact}`** — CLI verb `speclink instructions`. `{artifact}` is a schema artifact id or the literal `apply`.

The server runs the engine's instruction builders against its store and the project's effective policy, and returns the **same payload shape** the fs-mode CLI produces:

- Artifact form: `changeName`, `artifactId`, `schemaName`, `changeDir`, `outputPath`, `description`, `instruction?`, `context?`, `rules?`, `locale`, `template`, `dependencies[]`, `unlocks[]`.
- Apply form: `changeName`, `changeDir`, `schemaName`, `contextFiles{}`, `progress{total,complete,remaining}`, `tasks[{id,description,done,parallel}]`, `state` (`blocked`\|`ready`\|`all_done`), `missingArtifacts?`, `locale`, `instruction?`.

Remote-mode value semantics for path-shaped fields: `changeDir` and `contextFiles` values carry **store-logical paths** (`changes/<name>`, `proposal.md`, …). They identify documents, not local files — skills in remote mode read documents through verbs (`speclink artifact cat`), never by opening these paths. The fs-only `preflight` block (local file existence checks) is omitted in remote mode.

### 5.7 Discussions

| Endpoint | CLI verb |
|---|---|
| `GET /discussions?archived=` | `speclink discuss list [--archived]` |
| `POST /discussions` | `speclink discuss new` |
| `GET /discussions/{slug}` | `speclink discuss show` |
| `PUT /discussions/{slug}/context` | `speclink discuss context` |
| `POST /discussions/{slug}/rounds` | `speclink discuss add-round` |
| `POST /discussions/{slug}/conclude` | `speclink discuss conclude` |
| `POST /discussions/{slug}/archive` | `speclink discuss archive` |
| `POST /discussions/{slug}/promote` | `speclink discuss promote` |

Speclink discussions are **structured, append-only documents**, so the write surface is verb-shaped (a round can only be appended, a conclusion concluded) rather than a generic document PATCH. The server enforces the document rules.

- **list → 200** `{ "discussions": [ { "slug": "…", "topic": "…", "status": "open", "rounds": 4, "created": "2026-07-03", "archived": false, "path": "discussions/….md" } ] }` (`path` store-logical).
- **new** — `{ "topic": "…" }` → 201 `{ "slug": "…", "topic": "…", "path": "…" }`; duplicate slug → `409 already_exists`.
- **show → 200** `{ "info": { …list item… }, "content": "…full document…" }`.
- **context** — `{ "content": "…" }` → 200 `{ "slug": "…", "context": "set" }` (idempotent replace of the Context section).
- **add-round** — `{ "mode": "assumptions", "content": "…" }` → 200 `{ "slug": "…", "round": 5, "mode": "assumptions" }` (append-only; earlier rounds are immutable).
- **conclude** — `{ "content": "…" }` → 200 `{ "slug": "…", "status": "concluded" }`.
- **archive** → 200 `{ "slug": "…", "archivedTo": "discussions/archive/<file>" }`.
- **promote** — `{ "name": "change-name"? }` → 201 `{ "change": "…", "slug": "…", "status": "promoted" }`. Creates a change (in the requesting repo, per §2) seeded from the conclusion; one discussion may be promoted into several changes.
- `add-round` / `conclude` / `promote` against an archived discussion → `409 discussion_archived`.

`speclink discuss discard` is **not part of v1** — destructive discussion removal stays a host-governance action in remote mode (fs mode keeps the local verb).

---

## 6. Coverage map (CLI remote verbs ↔ endpoints)

Every CLI verb that operates in remote mode, and where it lands:

| CLI verb | Endpoint(s) |
|---|---|
| `speclink list` / `list --specs` | `GET /changes`, `GET /specs` |
| `speclink status` | `GET /changes/{name}` |
| `speclink instructions [artifact\|apply]` | `GET /changes/{name}/instructions/{artifact}` |
| `speclink new change` | `POST /changes` |
| `speclink new artifact` | `PUT /changes/{name}/artifacts/{artifact}` (`If-Match: 0`) |
| `speclink task done` | `POST /changes/{name}/tasks/{taskId}/done` |
| `speclink claim` | `POST /changes/{name}/claim` |
| `speclink archive` | `POST /changes/{name}/archive` |
| `speclink artifact cat` | `GET /changes/{name}/artifacts/{artifact}` |
| `speclink language show` | `GET /language` |
| `speclink discuss list/new/show/context/add-round/conclude/archive/promote` | §5.7 |
| `speclink auth status` / `link` / `init --store remote` | `GET /whoami` |
| *(policy side-car, internal)* | `GET /config` |

Deliberately **client-side** in remote mode (no endpoint; the CLI embeds the engine and runs these against verb-fetched documents): `speclink analyze`, `speclink drift`, `speclink validate`, `speclink show`. Deliberately **local-only**: `init`/`update`/`config`/`completion`/`feedback`/`schemas`/`templates`/`schema`, `speclink discuss discard`, `speclink in-progress` (fs-mode bookkeeping; remote lifecycle supersedes it).

---

## 7. Item-by-item comparison with wadpilot `04-speclink-final-design.md` §5.3

The wadpilot design (04) is the evidence base for this contract. Where this contract differs, the difference and its reason are recorded here. (04 endpoints are quoted with their original casing.)

| 04 §5.3 | This contract | Difference & reason |
|---|---|---|
| `POST/GET/DELETE /tokens` | *(out of contract)* | PAT issuance/revocation is host-governance UI. The contract only *consumes* a Bearer token. |
| `GET /whoami` (401 `token_invalid\|token_expired\|token_revoked`) | `GET /whoami` | **Adopted**, including the three 401 reasons (plus `token_missing`). Extended with `repos[]` (+ optional `gitUrl` reference) because the repos registry drives link validation and the fork warning. |
| `POST /changes` (JWT) | `POST /changes` (PAT) | Adopted. Speclink's client is always PAT; JWT-vs-PAT split is host-internal. |
| `GET /changes?project=&status=` | `GET /changes` (+ `?lifecycle=`) | Project scope moved into the base path (connection URL is project-scoped). Filtering by **repo** (header) is added — 04 had no per-repo filtering; speclink v1's one-change-one-repo rule requires it. `sourceDiscussKey` dropped: visible keys / 單號 are host presentation, not contract. |
| `GET /changes/:id` (+`artifactVersions`) | `GET /changes/{name}` | Change identity is the **name** (engine vocabulary), not a DB id — visible keys are host presentation. `artifactVersions{}` map folded into `artifacts[].version` to align with the existing `speclink status --json` array. |
| `GET /changes/:id/bundle` | *(none)* | Speclink v1 has no local materialization, no outbox, no cache — verbs read documents on demand (`artifact cat`, `instructions`). 04's motivation (Bash 30k output truncation of a mega-payload) does not arise for per-document reads. An aggregate endpoint can be added later as a pure optimization. |
| `GET /changes/:id/instructions/:artifact` (apply form removed, LC-4) | kept, `{artifact}` ∈ schema ∪ `apply` | Speclink keeps the **apply** instruction form because there is no bundle sidecar to carry `locale`/state, and the apply skill's entry point must be shape-identical between fs and remote modes. |
| `POST /changes/:id/analyze` | *(client-side)* | The CLI embeds the engine, so analyze runs locally over verb-fetched artifacts. A server-side analyze is a host option (04 needs it because its client has no engine), not a contract endpoint. |
| `POST /changes/:id/approve` / `reject` | *(host surface)* | Gate approvals happen in the host UI. The **semantics** (review state, `gate_pending`, approval voided by content writes) are canon (§3.2); the endpoints are not part of the client contract because the CLI never approves. |
| `POST /changes/:id/claim` (409 `already_claimed\|wrong_status`) | `POST /changes/{name}/claim` (409 `ownership_lost\|change_busy\|gate_pending`) | Adopted atomically. Reasons renamed/split so that **each reason maps to exactly one CLI action**: `already_claimed` → `ownership_lost` (same advice as losing ownership later: coordinate/re-claim); `wrong_status` split into `change_busy` (wait) and `gate_pending` (seek approval) because the correct user action differs. |
| `POST /changes/:id/release` | *(host surface / future verb)* | No v1 CLI verb releases a claim; host governance can. The client-visible consequence is fully covered by `ownership_lost`. |
| `PATCH /changes/:id/tasks/:stableId/done` (`{touchedFiles, appliedAgainstVersion}`; 409 `version_mismatch` with fresh bundle / `ownership_lost` / `change_busy`) | `POST /changes/{name}/tasks/{taskId}/done` | `ownership_lost`/`change_busy` adjudication **adopted verbatim** (canon). Differences: POST (action-verb endpoints are uniformly POST; idempotency is semantic, not method-borne); task id is the ordinal id from the tasks payload (speclink's engine has no stable-id comment convention); no `appliedAgainstVersion`/fresh-bundle piggyback — with no local cache there is no stale checkbox file to reconcile, so version arbitration for tasks collapses into the server's own `tasksVersion`. |
| `POST .../request-verify`, `POST .../verify-result` | *(none)* | Verify is a local discipline in speclink v1 (`/speclink-verify` runs client-side); no `Verifying` wire state. If a verify gate is later promoted into the contract, it ships as a version bump with the `speclink-team` reference implementation. |
| `POST /changes/:id/ingest` | *(none — `PUT` artifacts)* | 04's client has no engine, so merging must be a server call. Speclink's ingest skill runs the merge client-side and writes results through `If-Match` PUTs. Host-driven merges surface as `busy`/`change_busy`. |
| `POST /changes/:id/archive` (→ `ArchiveConflict` state, `resolve-conflict` endpoint) | `POST /changes/{name}/archive` (409 `version_conflict` + `conflicts[]`) | Check-all-then-apply adopted. **No `ArchiveConflict` state and no resolve endpoint**: a failed archive leaves the change in `applying` and reports the conflicting capabilities; resolution (re-basing against the moved spec) is host-side. Fewer wire states, same information. |
| `POST .../tasks/:stableId/claim` (deferred in 04) | *(none)* | Same deferral. |
| `GET /discussions` / `POST /discussions` | adopted | — |
| `PATCH /discussions/:id` (summary/status) | `PUT …/context`, `POST …/rounds`, `POST …/conclude` | Speclink discussions are structured append-only documents with engine-enforced rules (rounds immutable, one live conclusion) — a generic PATCH cannot express "append-only", so the write surface is verb-shaped. |
| `DELETE /discussions/:id` | *(not in v1)* | Destructive removal stays host-governed; the CLI's fs-only `discard` does not cross the wire in v1. |
| `POST /discussions/:id/propose` (+`planRef`) | `POST /discussions/{slug}/promote` | Renamed to match the CLI verb (`promote`). `planRef` dropped from v1: multi-change fan-out lineage is carried by the discussion slug on each promoted change; a plan-reference field can be added additively if a host needs it. |
| `GET/PUT /projects/:projectId/config` | `GET /config` (read-only) | Project scope already in the base path. `PUT` config is host-admin surface — exactly the "gate policy configuration" freedom; key-format DSL fields are host extensions and never cross this contract. |
| `GET /specs?project=`, `GET /specs/search` | `GET /specs`, `GET /specs/{capability}` | Search is a host-UI concern (no CLI verb needs it); direct capability read added because skills read canonical specs through verbs. |
| `GET /projects?repo=` (reverse lookup from git remote) | *(none)* | Rejected on principle: git-remote-URL inference is unreliable (forks/mirrors — discussion round 15). Binding is declared in the connection file and validated via `whoami.repos[]`; git URLs are advisory-warning material only. |
| `GET /changes/:id/version` (cache stale-check) | *(none)* | No local cache → nothing to stale-check. `statusVersion` is still exposed on `GET /changes/{name}` for hosts. |
| `GET /version` (boot-time major compare) | `X-Speclink-Api-Version` header | Per-request negotiation instead of a boot-time check: stateless, race-free (no "checked at start, server upgraded mid-session"), and it works for every host without an extra round-trip. Server rejects rather than warns. |
| Error envelope `{reason, message, ...context}`; "409 always carries `reason`"; "CLI is the only status-code interpreter" | adopted verbatim | These three 04 rules are contract canon here (§1.4, §4). |
| 409 `version_mismatch` | `version_conflict` | Renamed: one reason now covers every stale-version case (artifact write **and** archive spec-merge), and the `_conflict` suffix matches the HTTP 409 status name. |
| outbox + offline queueing (§7.6) | *(none)* | Explicitly rejected for speclink v1 (discussion round 3): connectivity failure is a loud failure, never a queued write — no cache, no outbox, no divergent local truth. |

---

## 8. Contract versioning

- This document defines **major version 1** (`X-Speclink-Api-Version: 1`).
- Additive changes (new endpoints, new optional fields, new `reason` values on *new* error paths) do not bump the major version. Clients ignore unknown fields; clients treat an unknown `reason` on a known path via the generic fallback ("unexpected server response — update speclink").
- Any change to existing guard semantics, state derivation, `reason` adjudication, or field meaning is a **major** bump — servers may support several majors side by side and select on the request header.
