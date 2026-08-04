# Verb Contract — Endpoint Reference

This document is the canonical reference for the verb contract's endpoints, payloads, and error shapes, as designated by the canonical `verb-contract` spec. It currently covers the verb-parity endpoints (validate / analyze / delete change / task move / discussion create-with-slug / discussion discard / discussion link / discussion seal / change in-progress); for every other verb the canonical specs remain the contract:

- [Canonical verb contract](../openspec/specs/verb-contract/spec.md)
- [Client Protocol spec](../openspec/specs/client-protocol/spec.md)

All endpoints below live under the project base `/api/speclink/v1/projects/{key}` and require the standard contract headers (`Authorization: Bearer …`, `X-Speclink-Api-Version`, `X-Speclink-Repo` when selected). Every success response carries the scope ETag header (the project revision).

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

**Aggregation rule**: the endpoint is fixed to one change. The CLI's aggregate modes (`speclink validate` with no argument, `--all`, `--changes`) are composed **client-side**: list the changes, then call this endpoint per change; the aggregate output shape matches fs mode, and any invalid change makes the CLI exit non-zero.

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

**Force semantics per entry point**: the CLI passes the user's `--force` flag through (guard parity with local discard); the desktop remote delete always sends `force=true` (parity with the local desktop delete, whose confirmation lives in the UI layer).

## POST /changes/{name}/tasks/move

**Editor only** (reader gets `403 permission_denied`). Moves one checkbox task and renumbers `digit.digit` prefixes, byte-identical to the local drag-reorder (the engine's single move implementation).

Request:

```json
{ "from": 1, "to": 3, "before": null }
```

`from`/`to` are 1-based checkbox ordinals (the same addressing domain as task done/undone). `before` is the optional explicit side: `true` inserts before the anchor line (crossing a heading joins the anchor's group), `false` inserts after, omitted/`null` infers from direction (upward → before, downward → after).

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

**Editor only** (reader gets `403 permission_denied`). Direct pass-through of the engine's discussion discard: a zero-round record deletes immediately; once rounds exist the engine refuses unless `force=true` — `409 refused` with the frozen needs-force text, record byte-identical. The commit publishes `discussion-discarded`.

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

Marks the discussion promoted once content has landed (status `promoted`, `promoted_to` accumulates the change; clears the change's re-ingest flag for this slug). Same request/response shape as link. Guard: the change must already carry the `from_discussion` chain for this slug — otherwise `409 refused` with the engine's run-link-first text. The commit publishes `discussion-sealed`. Errors: `404 not_found` for a missing discussion or change.

## POST /changes/{name}/in-progress

Silent lifecycle stamp through the Command gateway. First call on an existing, unstarted change writes `started_at` plus `started_by` (the caller's authenticated identity — same attribution mechanism as `created_*`) into the change meta, publishes `change-marked-in-progress`, and advances the scope revision. A repeat call or an unknown change name is the engine's frozen silent success: `200` with zero writes, zero events, no revision advance. The body is the empty object both ways:

```json
{}
```

## Change-list `startedAt` field

`GET /changes` list items carry an optional `startedAt` (camelCase) sourced from the change meta's `started_at`; an unstarted change omits the field. Consumers use it for stage derivation ("started ⇒ in-progress", with the completed-tasks fallback retained for tool-bypassing writes):

```json
{ "name": "demo", "status": "in-progress", "completedTasks": 0, "totalTasks": 15, "startedAt": "2026-07-30" }
```

## `speclink list --json` — local-only `worktree` field

`list --json` change items may carry an optional `worktree` object — the local observation surface for a change being implemented in a linked git worktree:

```json
{ "name": "add-dark-mode", "completedTasks": 3, "totalTasks": 5, "worktree": { "path": "/repos/speclink.worktrees/add-dark-mode", "branch": "speclink/add-dark-mode" } }
```

- `path` — string, absolute path of the worktree directory. `branch` — string, full branch name (`speclink/<change>`).
- Present **only** in fs mode, from the **main checkout** (the workspace root's `.git` is a directory), with the `worktree` workflow policy on, and only for changes whose mapping holds. Absent in every other case, and **never serialized** when absent.
- Remote-mode `list` items **always** omit the field: it describes the caller's local checkout, of which the server knows nothing. With no worktree present, the fs and remote payloads are therefore field-for-field identical.
- When the field is present, the item's `completedTasks` / `totalTasks` / `status` / `metaError` values come from that worktree's copy of the change, not the main checkout's. Field names and types are unchanged.

## GET /changes/{name} — show-composition meta fields

The single-change read additionally carries three optional fields feeding the CLI's remote `show` composition: `created` (present only when the meta holds the schema+created pair — the engine's report-as-one-unit rule), `fromDiscussions`, and `deltaCapabilities` (both omitted when empty). An older server simply never sends them; old clients ignore them.

```json
{ "changeName": "demo", "schemaName": "spec-driven", "…": "…", "created": "2026-07-29", "fromDiscussions": ["auth-scope"], "deltaCapabilities": ["auth"] }
```

## Capability declaration

The `GET /binding` handshake declares these verbs per membership role:

```json
"capabilities": { "validate": true, "analyze": true, "deleteChange": true, "moveTask": true, … }
```

`validate`/`analyze` are `true` for every role; `deleteChange`/`moveTask` are `true` only for editors. Clients disable the corresponding affordances when a capability is `false`; the server's request-time role check remains the final enforcement point.
