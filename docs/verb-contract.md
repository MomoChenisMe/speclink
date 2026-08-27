# Verb and Flag Contract

> **Who needs this**: anyone wiring up the Speclink remote API themselves, writing a client, or checking how a given verb behaves in remote mode.
> If you just use the desktop app or the CLI, skip this entirely — the [Complete SDD Workflow](workflow.md) covers day-to-day use.

This document answers the verb-level contract. It covers how the CLI assigns verbs across local and remote modes, and what their output guarantees are. It also covers the endpoints, payloads, and error shapes that the canonical `verb-contract` spec designates.

The endpoint half currently covers the verb-parity endpoints: validate, analyze, delete change, task move, discussion create-with-slug, discussion discard, discussion link, discussion seal, change in-progress, and change claim. For every other verb the canonical specs remain the contract:

- [Canonical verb contract](../openspec/specs/verb-contract/spec.md)
- [Client Protocol spec](../openspec/specs/client-protocol/spec.md)

All endpoints below live under the project base `/api/speclink/v1/projects/{key}` and require the standard contract headers (`Authorization: Bearer …`, `X-Speclink-Api-Version`, `X-Speclink-Repo` when selected). Every success response carries the scope ETag header (the project revision).

## CLI verb mode assignment

Every top-level verb belongs to one of four mode shapes, declared in one place in the dispatch layer rather than scattered across verb functions:

| Shape | Verbs | Semantics |
| --- | --- | --- |
| **ModeFree** | `init`, `update`, `link`, `unlink`, `auth`, `schemas`, `templates`, `feedback`, `schema`, `config`, `completion` | Never triggers store mode resolution. Verbs that do not read project settings (`completion`, `config`) are unaffected by a broken `.speclink.yaml`. |
| **Dual** | `list`, `show`, `validate`, `analyze`, `drift`, `archive`, `discard`, `artifact`, `language`, `status`, `instructions`, `new`, `workflow-config`, `task`, `in-progress`, `discuss`, `review`, `verify` | Local mode acts on the local store and remote mode acts on the remote store; it **never** silently falls back to the local store in remote mode. A missing arm is a build failure, not a runtime fallback. |
| **FsOnly** | `demo`, `trace` | Remote mode refuses with a non-zero exit code and issues no server request at all — it refuses offline too. |
| **RemoteOnly** | `claim` | Local mode refuses with a non-zero exit code and explains on stderr that a remote store is required. |

Mode resolution is lazy. The CLI resolves the mode only when the declared shape needs it. It opens a connection only when the remote arm is about to run.

## Output parity across modes

For Dual verbs the human-readable output (stdout text, including under `--no-color`) is byte-identical across both modes, with exactly five declared divergences:

1. The Path line from `new change` — printed locally, omitted remotely (a server-side path means nothing to a local user).
2. The worktree marker in `list` — always absent remotely (worktrees are an observation of the local main checkout).
3. The schema override flag on `status` — refused explicitly in remote mode (the server's workflow config decides the schema).
4. The document label in `workflow-config` — remote labels it `config.yaml`.
5. The Path line from `discuss promote` and the prompt line after it — printed locally, omitted remotely (the two travel together).

Any output difference outside that list is a defect. Mode differences exist only in data acquisition and gate refusals, never in the typesetting of the output text.

The `--json` field set and camelCase naming are a frozen contract: no renames, no removals of existing fields, and ticket prose never appears in any `--json` output.

For when to use each verb and what counts as done, this is not the document — that is the [Complete SDD Workflow](workflow.md).

## Error envelope

Every non-2xx response is the protocol error envelope:

```json
{ "status": 409, "reason": "refused", "message": "…human-readable, engine-frozen text…" }
```

`reason` is the machine-readable registry value (`not_found`, `permission_denied`, `refused`, `invalid_argument`, `invalid_config`, `revision_conflict`, `unavailable`, `internal`).

## GET /changes/{name}/validate

Read-only derived query, available to **reader and editor**. Runs the same engine computation as fs-mode `speclink validate` for a single change (spec-driven schema, non-strict). Never writes, never publishes events, never advances the scope revision.

Response `200`:

```json
{ "change": "demo", "valid": false, "errors": ["…"], "warnings": ["…"] }
```

Errors: `404 not_found` when the change does not exist.

**Aggregation rule**: the endpoint is fixed to one change. The CLI composes its aggregate modes **client-side**: `speclink validate` with no argument, `--all`, and `--changes`. It lists the changes, then calls this endpoint once per change. The aggregate output shape matches fs mode. Any invalid change makes the CLI exit non-zero.

## GET /changes/{name}/analyze

Read-only derived query, available to **reader and editor**. Returns the engine's full `AnalyzeReport`. Never writes, never publishes events.

Response `200`:

```json
{
  "changeId": "demo",
  "dimensions": [{ "dimension": "Coverage", "status": "Clean", "findingCount": 0 }],
  "findings": [{
    "id": "AMB-1", "dimension": "Ambiguity", "severity": "Suggestion",
    "location": "specs/auth/spec.md", "summary": "…", "recommendation": "…",
    "summaryMsg": { "key": "…", "params": { "scenario": "…" } },
    "recommendationMsg": { "key": "…", "params": {} }
  }],
  "artifactsAnalyzed": ["proposal.md"],
  "artifactsMissing": ["design.md"]
}
```

Errors: `404 not_found` when the change does not exist.

## DELETE /changes/{name}?force={bool}

**Editor only** (reader gets `403 permission_denied`). Full discard semantics through the Command gateway: fail-closed metadata check, started-work guard, source-discussion unlink, atomic deletion of every change document, touched-record cleanup. The commit publishes a `change-discarded` event, so SSE subscribers receive an invalidation.

Query parameter `force` defaults to `false`.

- `force=false` on a change with started work (`started_at` stamped, or any task checked) → `409 refused`; the message is the engine's frozen needs-force text. **On this endpoint, `reason: "refused"` is the machine-readable needs-force signal.** Nothing is written.
- `force=true` deletes regardless of started work. Corrupt change metadata refuses even with `force=true` (`invalid_config`).

Response `200`:

```json
{ "change": "demo", "unlinkedDiscussions": [{ "slug": "auth-flow", "status": "concluded" }] }
```

**Force semantics per entry point.** The CLI passes the user's `--force` flag through, which gives guard parity with local discard. The desktop remote delete always sends `force=true`. That matches the local desktop delete, whose confirmation lives in the UI layer.

## POST /changes/{name}/tasks/move

**Editor only** (reader gets `403 permission_denied`). Moves one checkbox task and renumbers `digit.digit` prefixes, byte-identical to the local drag-reorder (the engine's single move implementation).

Request:

```json
{ "from": 1, "to": 3, "before": null }
```

`from`/`to` are 1-based checkbox ordinals (the same addressing domain as task done/undone). `before` is the optional explicit side. `true` inserts before the anchor line; a move across a heading joins the anchor's group. `false` inserts after. Omitted or `null` infers from direction: upward → before, downward → after.

Response `200`:

```json
{ "change": "demo", "description": "2.2 甲" }
```

`description` is the moved task's cleaned description **after** the move (prefixes already renumbered). The commit publishes a `task-moved` event → SSE invalidation.

Errors:

- `409 refused` with a `task index out of range (1..=N)` message when `from`/`to` is out of range (a stale index under concurrent edits is an expected race; the SSE invalidation corrects the client's view). Nothing is written.
- `404 not_found` when the change has no `tasks.md`.

## POST /discussions — optional slug override

The create-discussion request accepts an optional `slug` field (camelCase, omitted when absent — an old client's body stays byte-identical):

```json
{ "topic": "看板搜尋列", "slug": "board-search-bar" }
```

Validation lives in the engine only (ASCII kebab-case: lowercase letters/digits separated by single hyphens). An invalid value → `400 invalid_argument` with the engine's frozen message; nothing is written. Without `slug` the server derives one from the topic, exactly as before.

Response `200` (unchanged shape; `slug` echoes the override when given):

```json
{ "slug": "board-search-bar", "topic": "看板搜尋列", "path": "discussions/board-search-bar.md" }
```

## DELETE /discussions/{slug}?force={bool}

**Editor only** (reader gets `403 permission_denied`). Direct pass-through of the engine's discussion discard. A zero-round record deletes immediately. After rounds exist, the engine refuses unless `force=true`. It returns `409 refused` with the frozen needs-force text and leaves the record byte-identical. The commit publishes `discussion-discarded`.

Query parameter `force` defaults to `false`. Response `200`:

```json
{ "slug": "board-search-bar" }
```

Errors: `404 not_found` when no live discussion has the slug; an archived record refuses (`409 refused` — archived records are kept, not discarded).

## POST /discussions/{slug}/link

Forges the change-side `from_discussion` chain (the engine's link semantics: comma-accumulating, idempotent per pair). Request / response:

```json
{ "change": "add-auth" }
```

```json
{ "slug": "auth-scope", "change": "add-auth" }
```

The commit publishes `discussion-linked`. Errors: `404 not_found` when the discussion or the change does not exist (the engine's frozen message names the missing subject).

## POST /discussions/{slug}/seal

Marks the discussion promoted after its content lands. Status becomes `promoted`, `promoted_to` accumulates the change, and the change's re-ingest flag for this slug clears. Same request and response shape as link. Guard: the change must already carry the `from_discussion` chain for this slug. Otherwise it returns `409 refused` with the engine's run-link-first text. The commit publishes `discussion-sealed`. Errors: `404 not_found` for a missing discussion or change.

## POST /changes/{name}/in-progress

Silent lifecycle stamp through the Command gateway. The first call on an existing, unstarted change writes `started_at` and `started_by` into the change meta. `started_by` is the caller's authenticated identity, the same attribution mechanism as `created_*`. The call then publishes `change-marked-in-progress` and advances the scope revision. A repeat call or an unknown change name gives the engine's frozen silent success: `200` with zero writes, zero events, and no revision advance. The body is the empty object both ways:

```json
{}
```

## POST /changes/{name}/claim

Durable claim through the Command gateway. On a change nobody holds, the call writes `claimed_by` and `claimed_at` into the change meta with the caller's authenticated identity — the write commits with the unit of work, publishes `change-claimed`, and advances the scope revision, so the claim survives server restarts and shows on every device. A repeat call by the same identity is an idempotent success with zero writes. The response carries the holder:

```json
{ "claimedBy": "Demo <d@e.com>" }
```

A change someone else holds returns `409 refused` — the message names the current holder and the suggested action, and the meta stays untouched. There is no dedicated ownership reason on the wire; the conflict travels as `refused` inside the eight-value registry. The endpoint is editor-gated like the other write verbs: a reader gets `403` with no scope change, and an unknown change returns `404 not_found`. Local fs mode refuses the verb outright (remote-only), so this endpoint is the verb's only home.

## Change-list meta fields

`GET /changes` list items carry optional camelCase fields sourced from the change meta: `startedAt` (from `started_at`; an unstarted change omits it), `createdBy` and `created` (creation attribution), `fromDiscussions` (the source-discussion chain, omitted when empty), and `claimedBy` (from `claimed_by`, omitted while unclaimed). Consumers use `startedAt` for stage derivation ("started ⇒ in-progress", with the completed-tasks fallback retained for tool-bypassing writes):

```json
{ "name": "demo", "status": "in-progress", "completedTasks": 0, "totalTasks": 15, "startedAt": "2026-07-30", "createdBy": "Demo <d@e.com>", "created": "2026-07-29", "fromDiscussions": ["auth-scope"], "claimedBy": "Demo <d@e.com>" }
```

## `speclink list --json` — local-only `worktree` field

`list --json` change items may carry an optional `worktree` object — the local observation surface for a change being implemented in a linked git worktree:

```json
{ "name": "add-dark-mode", "completedTasks": 3, "totalTasks": 5, "worktree": { "path": "/repos/speclink.worktrees/add-dark-mode", "branch": "speclink/add-dark-mode" } }
```

- `path` — string, absolute path of the worktree directory. `branch` — string, full branch name (`speclink/<change>`).
- Present **only** in fs mode, from the **main checkout**, where the workspace root's `.git` is a directory. It also needs the `worktree` workflow policy on, and it appears only for changes whose mapping holds. It is absent in every other case, and **never serialized** when absent.
- Remote-mode `list` items **always** omit the field: it describes the caller's local checkout, of which the server knows nothing. With no worktree present, the fs and remote payloads are therefore field-for-field identical.
- When the field is present, the item's `completedTasks` / `totalTasks` / `status` / `metaError` values come from that worktree's copy of the change, not the main checkout's. Field names and types are unchanged.

## GET /changes/{name} — show-composition meta fields

The single-change read also carries seven optional fields that feed the CLI's remote `show` composition and the desktop detail drawer. `created` appears only when the meta holds the schema and created pair, which is the engine's report-as-one-unit rule. `fromDiscussions` and `deltaCapabilities` are both omitted when empty. The attribution quartet `createdBy`, `createdWith`, `startedAt`, and `startedBy` mirrors the meta and is omitted field by field when the meta lacks it. `claimedBy` joins them, assembled from the meta's `claimed_by` and omitted while unclaimed. An older server never sends them, old clients ignore them, and a client never fabricates a default for an absent field.

```json
{ "changeName": "demo", "schemaName": "spec-driven", "…": "…", "created": "2026-07-29", "fromDiscussions": ["auth-scope"], "deltaCapabilities": ["auth"], "createdBy": "Demo <d@e.com>", "createdWith": "claude-code", "startedAt": "2026-08-25T00:00:00Z", "startedBy": "Demo <d@e.com>", "claimedBy": "Demo <d@e.com>" }
```

## GET /discussions — `promotedTo` field

Discussion list items carry an optional `promotedTo` — the changes this discussion was promoted into, in frontmatter accumulation order. The server assembles it at the route edge from the engine's promoted-to query; the engine's discussion-list structure and the local `discuss list --json` output stay byte-identical. An empty list is omitted, and a lookup failure for a single discussion degrades to the field's absence rather than failing the listing.

## Capability declaration

The `GET /binding` handshake declares these verbs per membership role:

```json
"capabilities": { "validate": true, "analyze": true, "deleteChange": true, "moveTask": true, … }
```

`validate`/`analyze` are `true` for every role; `deleteChange`/`moveTask` are `true` only for editors. Clients disable the corresponding affordances when a capability is `false`; the server's request-time role check remains the final enforcement point.
