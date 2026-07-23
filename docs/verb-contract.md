# Verb Contract — Endpoint Reference

This document is the canonical reference for the verb contract's endpoints, payloads, and error shapes, as designated by the canonical `verb-contract` spec. It currently covers the verb-parity endpoints (validate / analyze / delete change / task move); for every other verb the canonical specs remain the contract:

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

## Capability declaration

The `GET /binding` handshake declares these verbs per membership role:

```json
"capabilities": { "validate": true, "analyze": true, "deleteChange": true, "moveTask": true, … }
```

`validate`/`analyze` are `true` for every role; `deleteChange`/`moveTask` are `true` only for editors. Clients disable the corresponding affordances when a capability is `false`; the server's request-time role check remains the final enforcement point.
