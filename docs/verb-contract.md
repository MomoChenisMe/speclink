# Verb and Flag Contract

[繁體中文](verb-contract.zh-TW.md) · **English**

## <a id="audience"></a>Who this document is for

Read this document if you write your own client, run your own server, or call Speclink over HTTP. If you only use the desktop app or the CLI, skip it. The [Complete SDD Workflow](workflow.md) covers daily use.

This document answers three questions:

- Which CLI verbs work in local mode and in remote mode, and how their output differs. Local mode keeps the documents in the repo's `openspec/`. Remote mode keeps them on a server; `.speclink.yaml` then has a `remote:` section.
- What the shared rules, error codes, and endpoints of the remote HTTP API are.
- What the request and response bodies look like.

The official `speclink-server` is a reference implementation. You can use this document to build your own server. If this document and the canonical specs disagree, the canonical specs win:

- [Canonical verb-contract spec](../openspec/specs/verb-contract/spec.md)
- [Canonical client-protocol spec](../openspec/specs/client-protocol/spec.md)

To start a server and make an access key (PAT), see [Remote Getting Started](remote-getting-started.md).

## <a id="mode-assignment"></a>Mode assignment of verbs

All verbs work in both modes, except the "Local only" and "Remote only" rows below.

| Assignment | Verbs | In the unsupported mode |
| --- | --- | --- |
| Local only | `demo`, `trace`, `change rank`, `plan --strict-overlap` | Remote mode refuses with a non-zero exit code. The CLI sends no request, and it refuses offline too. |
| Remote only | `claim` | Local mode refuses with a non-zero exit code. stderr says that the verb needs a remote store. |
| Both modes | `list`, `show`, `validate`, `analyze`, `drift`, `archive`, `discard`, `artifact`, `language`, `status`, `instructions`, `new`, `workflow-config`, `task`, `in-progress`, `discuss`, `review`, `verify`, `plan`, `change` (except `change rank`) | Not applicable. Remote mode reads and writes only the server. It never falls back to local files. |
| Mode-free | `init`, `update`, `link`, `unlink`, `auth`, `schemas`, `templates`, `feedback`, `schema`, `config`, `completion` | Not applicable. These verbs do not use the store. |

Why some verbs are local only:

- `demo` writes a demo change into the local `openspec/`.
- `trace` builds the provenance chain from the archived changes and discussion files in the local `openspec/`.
- `change rank` writes the order of the local cards. In remote mode, the board order lives at `/board-order`.
- `plan --strict-overlap` finds overlap by capability directory. The server finds overlap by requirement only.

`claim` is remote only. A claim records who on the team works on a change, and local mode has no such state.

## <a id="output-differences"></a>Output differences between modes

For verbs that work in both modes, the human-readable output (also with `--no-color`) must be identical. The table below shows all known differences.

| # | Verb | What remote mode does differently |
| --- | --- | --- |
| 1 | `new change` | It does not print the `Path:` line. A server path means nothing to you. |
| 2 | `list` | It shows no worktree marker. A worktree belongs to the local checkout. |
| 3 | `status --schema` | It refuses. The server's workflow config decides the schema. |
| 4 | `workflow-config` | The document label is always `config.yaml`. |
| 5 | `discuss promote` | It does not print the `Path:` line or the hint line after it. |
| 6 | `plan --strict-overlap` | It refuses. See the previous section. |
| 7 | `list` | Local mode sorts by last modification by default. Remote mode sorts by name. `--sort created` and `--sort modified` also give name order in remote mode. |
| 8 | `validate` (several changes) | Local mode orders the results by last modification. Remote mode orders them by name. Remote mode never validates changes in strict mode; `--strict` applies to `--specs` only. |
| 9 | `instructions --schema` | It refuses, for the same reason as item 3. |
| 10 | `archive` | It archives one named change at a time. It refuses `--all` and several names. `--skip-specs`, `--no-validate`, and `--mark-tasks-complete` have no effect. |

Items 1–6 are the differences that the canonical spec declares. Items 7–10 show the current code; the canonical spec does not list them yet.

The `--json` field set and its camelCase names are a frozen contract. Fields do not change names and do not disappear. No `--json` output contains ticket text.

## <a id="request-rules"></a>Shared request rules

### URL

The API of each project lives under this URL:

```text
https://<server>/api/speclink/v1/projects/<project-key>
```

All paths in the endpoint tables below follow this URL. The CLI keeps this URL in `remote.url` in `.speclink.yaml`. `speclink link <url>` writes it.

### Headers on every request

| Header | Value | When it is missing or wrong |
| --- | --- | --- |
| `Authorization` | `Bearer <credential>` | 401 `permission_denied` |
| `X-Speclink-Api-Version` | `1` | 409 `refused` (incompatible version) |
| `X-Speclink-Repo` | The repo key | Optional when the project has one repo. With several repos, a missing header gives 409 `refused`. An unregistered repo gives 404 `not_found`. |
| `Content-Type` | `application/json` (when a body is sent) | 415 with a plain-text body |

You can use two kinds of credential. One is an access key (PAT) that starts with `spk_pat_`; you make it on the account page of the server. The other is the device login credential from `speclink auth login`; it starts with `spk_at_`. A valid credential of a user who is not a project member gives 403.

### Roles

A project member has the `reader` role or the `editor` role. If the "Minimum role" column says `editor`, a reader gets 403 `permission_denied`. The official server checks the role only on those endpoints. Any member, a reader too, can call the other write endpoints today.

### Handshake

Call `GET /binding` first. It tells you who you are, which project and repo you use, the server version, and the capabilities of your role:

```json
{
  "actor": { "id": "usr_…", "name": "Demo" },
  "project": { "id": "prj_acme", "key": "acme", "name": "acme" },
  "repo": { "id": "repo_backend", "key": "backend", "name": "backend" },
  "apiVersion": "1",
  "engineVersion": "0.8.0",
  "capabilities": {
    "contextSnapshots": true,
    "policyWrite": true,
    "validate": true,
    "analyze": true,
    "deleteChange": true,
    "moveTask": true,
    "authentication": [],
    "events": {
      "transports": [{ "type": "sse", "url": "/events", "resume": true }],
      "polling": { "url": "/sync-state", "etag": true }
    }
  }
}
```

- If `apiVersion` is not the version that you sent, stop. Do not call other endpoints.
- `policyWrite`, `deleteChange`, and `moveTask` are `true` only for editors. `validate` and `analyze` are `true` for all roles.
- `capabilities` is only a hint for disabling buttons in a UI. The server checks permissions on each request.

### ETag and write preconditions

An ETag is an HTTP response header that tells you whether data changed.

- Most success responses carry `ETag`. Its value is the project revision, for example `"16"`. Each write in the project makes it larger. `GET /binding` and `GET /events` do not send it.
- To check for changes, call `GET /sync-state` with `If-None-Match: <ETag>`. It answers 304 when nothing changed. `POST /context` also accepts `If-None-Match`.
- Only three endpoints need a version before a write, and each one takes it differently:

| Endpoint | Where the version goes | Where the version comes from |
| --- | --- | --- |
| `PUT /changes/{name}/artifacts/{artifact}` | `If-Match: <number>`, without quotes | `version` in the `GET` response of the same artifact. `0` means "create only". |
| `PUT /board-order` | `If-Match: <ETag>`, quotes optional | The `ETag` of any response, for example `GET /board-order` |
| `PUT /config` | `expectedRevision` in the body | `revision` in the `GET /config` response |

A stale version gives 409 `revision_conflict`, and the server writes nothing. The first two endpoints give 400 `invalid_argument` when `If-Match` is missing. Other write endpoints do not check versions.

## <a id="repo-ownership"></a>Repo ownership of changes

Each change belongs to exactly one repo. Split a requirement that touches two repos into several changes.

- When remote mode creates a change (`POST /changes`), the change belongs to the repo in `X-Speclink-Repo`. In a project with one repo, the server uses that repo. The `repo` field of the response shows the owner.
- The change list (`GET /changes`) shows only the changes of the request's repo. Changes of other repos do not appear.
- If a requirement changes two repos (for example backend and frontend), make two changes. Make each one in its own repo. The contract has no change that spans repos.
- Discussions, canonical specs, archives, the workflow config, and the board order are also stored per repo.
- The CLI takes the repo name from `remote.repo` in `.speclink.yaml`. Set it with `speclink link <url> --repo <name>`.

## <a id="errors"></a>Error envelope and error codes

Every non-2xx response has the same JSON shape, the error envelope:

```json
{ "status": 409, "reason": "refused", "message": "change 'add-login' has started work (started_at set or tasks checked) — discard refuses to delete it; pass --force to discard anyway" }
```

- Programs use `reason`. People read `message`, which is the same line that the local CLI prints.
- `reason` has only the eight values in the table below. Treat an unknown value as a general error and show `message`.
- When `DELETE /changes/{name}/in-progress` refuses, the envelope also has `checkedTasks` and `touchedFiles`.
- Exception: a malformed request body gives a plain-text response, not an envelope. For example, a missing `Content-Type: application/json` gives 415, and a missing required field gives 422.

| `reason` | HTTP | When it occurs |
| --- | --- | --- |
| `permission_denied` | 401 | The credential is missing, invalid, expired, or revoked, or the account is suspended. |
| `permission_denied` | 403 | The account is not a project member, or a reader calls an editor-only endpoint. |
| `not_found` | 404 | The project or repo is not registered, or the change, discussion, artifact, or spec does not exist. |
| `invalid_argument` | 400 | A value is wrong: a bad slug, a missing `If-Match`, a non-numeric `If-Match`, an unknown artifact, or an empty search query. |
| `invalid_config` | 422 | The change metadata is corrupt, or the server cannot parse the workflow config. |
| `refused` | 409 | A precondition fails: API version mismatch, no repo in a multi-repo project, `force` needed, task number out of range, change claimed by someone else, dependency cycle, or import into a non-empty target. |
| `refused` | 413 | The request body is larger than 32 MiB, or the board-order content is larger than 1 MiB. |
| `revision_conflict` | 409 | The version that you sent with a write is stale (see [Shared request rules](#request-rules)). |
| `unavailable` | 503 | The store is temporarily unavailable. |
| `internal` | 500 | All other failures. |

Note: some content checks answer 500 `internal` today, but `message` still gives the cause. Two examples: a `tasks.md` write with no `- [ ]` checkbox, and a review round without a `**Scope**:` line. Use `message` to decide whether a retry can help.

## <a id="endpoints"></a>Endpoint reference

All paths follow the [project URL](#request-rules). `{name}` is a change name, and `{slug}` is a discussion slug. `{artifact}` is `proposal`, `design`, `tasks`, or `specs/<capability>`. "Minimum role" `reader` means that all members can call the endpoint.

### Connection and sync

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/binding` | reader | Handshake: identity, project, repo, versions, and capabilities |
| GET | `/whoami` | reader | The current user and all repos of the project |
| GET | `/sync-state` | reader | Only the `ETag`. With a matching `If-None-Match`, it answers 304. |
| GET | `/events` | reader | SSE event stream. See [Event stream](#events) below. |
| POST | `/context` | reader | One consistent snapshot of the documents. The body can set `change` to narrow it. |

### Changes

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/changes` | reader | List the changes of this repo (`speclink list`) |
| POST | `/changes` | reader | Create a change (`speclink new change`) |
| GET | `/changes/{name}` | reader | Status and metadata of one change (`speclink status`, `show`) |
| DELETE | `/changes/{name}?force=<bool>` | editor | Discard a change (`speclink discard`) |
| GET | `/changes/{name}/drift` | reader | Drift data from the spec side (`speclink drift`) |
| GET | `/changes/{name}/validate` | reader | Validate one change (`speclink validate`) |
| GET | `/changes/{name}/analyze` | reader | Cross-artifact analysis report (`speclink analyze`) |
| GET | `/changes/{name}/instructions/{kind}` | reader | Artifact instructions. `{kind}` is `proposal`, `design`, `specs`, or `tasks`; `apply` gives the implementation progress (`speclink instructions`). |
| GET | `/changes/{name}/artifacts/{artifact}` | reader | Read an artifact and its `version` (`speclink artifact cat`) |
| PUT | `/changes/{name}/artifacts/{artifact}` | reader | Write an artifact. It needs `If-Match`. |
| GET | `/changes/{name}/evidence` | reader | Completion evidence: the files that `task done` records for each task |
| POST | `/changes/{name}/in-progress` | reader | Mark the change as started (`speclink in-progress add`) |
| DELETE | `/changes/{name}/in-progress` | reader | Remove the start marker (`speclink in-progress remove`) |
| POST | `/changes/{name}/claim` | editor | Claim the change (`speclink claim`) |
| POST | `/changes/{name}/depends` | editor | Declare or remove prerequisite changes (`speclink change depends`) |
| POST | `/changes/{name}/archive?carryReview=<bool>&carryVerify=<bool>` | reader | Archive (`speclink archive`) |

### Tasks

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| POST | `/changes/{name}/tasks/{taskId}/done` | reader | Check a task (`speclink task done`). `{taskId}` is a task number or a stable ID that starts with `tsk_`. |
| POST | `/changes/{name}/tasks/{taskId}/undone` | reader | Uncheck a task (`speclink task undone`) |
| POST | `/changes/{name}/tasks/move` | editor | Move a task and renumber the list (drag in the desktop app) |

### Quality gates

`{station}` is `review` or `verify`. Both stations use the same endpoints and payloads.

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/changes/{name}/{station}` | reader | Read the ticket (`speclink review show`) |
| POST | `/changes/{name}/{station}/rounds` | reader | Add a round (`speclink review add-round`) |
| POST | `/changes/{name}/{station}/stamp` | editor | Stamp (`speclink review stamp`) |
| DELETE | `/changes/{name}/{station}` | editor | Discard the ticket (`speclink review discard`) |

### Discussions

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/discussions?archived=<bool>` | reader | List discussions. `archived=true` lists the archived ones. |
| POST | `/discussions` | reader | Create a discussion (`speclink discuss new`) |
| GET | `/discussions/search?q=<keywords>` | reader | Search discussions. Separate keywords with spaces (`speclink discuss search`). |
| GET | `/discussions/{slug}` | reader | Read one discussion and its full text (`speclink discuss show`) |
| DELETE | `/discussions/{slug}?force=<bool>` | editor | Delete a discussion (`speclink discuss discard`) |
| PUT | `/discussions/{slug}/context` | reader | Write the Context section (`speclink discuss context`) |
| POST | `/discussions/{slug}/rounds` | reader | Add a round (`speclink discuss add-round`) |
| POST | `/discussions/{slug}/conclude` | reader | Write the conclusion (`speclink discuss conclude`) |
| POST | `/discussions/{slug}/archive` | reader | Archive the discussion (`speclink discuss archive`) |
| POST | `/discussions/{slug}/promote` | reader | Turn the discussion into a change (`speclink discuss promote`) |
| POST | `/discussions/{slug}/link` | reader | Link an existing change to the discussion (`speclink discuss link`) |
| POST | `/discussions/{slug}/seal` | reader | Mark the discussion as promoted after its content lands (`speclink discuss seal`) |

### Project data

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/plan` | reader | Execution order of the changes (`speclink plan`) |
| GET | `/specs` | reader | List of canonical specs |
| GET | `/specs/{capability}/document` | reader | Full text of one canonical spec |
| GET | `/archived` | reader | List of archived changes |
| GET | `/archived/{datedName}/artifacts/{path}` | reader | One document of an archived change. `{path}` is, for example, `proposal.md`. |
| GET | `/archived/{datedName}/capabilities` | reader | Capability names that the archive changed |
| GET | `/search?q=<text>` | reader | Full-text search of the desktop app (live changes and discussions only) |
| GET | `/language` | reader | Shared vocabulary document (`speclink language show`) |
| GET | `/config` | reader | Workflow config text and revision |
| PUT | `/config` | editor | Write the workflow config (`speclink workflow-config set`) |
| GET | `/board-order` | reader | Board order |
| PUT | `/board-order` | editor | Replace the full board order. It needs `If-Match`. |
| POST | `/import` | editor | Move a complete local project to the server (the desktop app uses it). The target repo must be empty. |

### Endpoints outside the project URL

These endpoints follow `https://<server>` directly:

| Method | Path | Needs | Purpose |
| --- | --- | --- | --- |
| GET | `/healthz` | Nothing | 200 while the process runs |
| GET | `/readyz` | Nothing | 200 when the store can serve, else 503 |
| GET | `/auth/whoami` | A credential only | The owner of the credential (use it before you select a project) |
| GET | `/api/speclink/v1/scopes` | A credential only | The projects and repos that this account can see |

`/auth/device`, `/auth/device/token`, `/auth/refresh`, and `/auth/revoke` are the device login flow of `speclink auth login`.

### <a id="events"></a>Event stream

`GET /events` is an SSE stream (Server-Sent Events: the server pushes events over one long-lived connection).

- `invalidate` event: a resource changed. The `data` is, for example, `{"eventId":"42","scope":"change","resourceId":"add-login","revision":42}`. `scope` is `change`, `discussion`, or `spec`. Read the resource again when you get it.
- `reset` event: the server removed your resume position. Read all data again.
- When you reconnect, send `Last-Event-ID` to get the events that you missed.
- When the stream is idle, the server sends heartbeat comment lines to keep the connection open.

## <a id="payload-examples"></a>Request and response examples

The field shapes below come from real responses of the official server. If the request body column says "None", send `{}` or no body. All other endpoints need a JSON body and `Content-Type: application/json`.

### Changes and tasks

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `POST /changes` | `{"name":"add-login","agent":"claude"}` | `{"name":"add-login","schema":"spec-driven","repo":"backend"}` |
| `DELETE /changes/{name}?force=true` | None | `{"change":"add-search-bar","unlinkedDiscussions":[{"slug":"board-search-bar","status":"concluded"}]}` |
| `POST /changes/{name}/claim` | None | `{"claimedBy":"Demo <demo@example.com>"}` |
| `POST /changes/{name}/depends` | `{"on":["add-login"],"remove":false}` | `{"change":"add-logout","dependsOn":["add-login"]}` |
| `POST /changes/{name}/in-progress` | None | `{}` |
| `DELETE /changes/{name}/in-progress` | None | `{"removed":true}`; `{"removed":false}` when the change was not started |
| `POST /changes/{name}/archive` | None | `{"specs":[],"datedName":"2026-10-07-fix-typo","snapshotCreated":false,"archivedDiscussions":[],"evidenceRecorded":false}` |
| `POST /changes/{name}/tasks/{taskId}/done` | `{"touchedFiles":["src/login.rs"],"headCommit":"0123…"}` | `{"taskDesc":"1.1 Add the login form","alreadyDone":false}` |
| `POST /changes/{name}/tasks/{taskId}/undone` | None | `{"taskDesc":"1.1 Add the login form","alreadyUndone":false}` |
| `POST /changes/{name}/tasks/move` | `{"from":2,"to":1}` | `{"change":"add-login","description":"1.1 Add the session check"}` |

Notes:

- `POST /changes` has these optional fields: `schema`, `description`, `agent`, `fromDiscussion` (the source discussion slug), and `last`. With `fromDiscussion`, `last` marks the final change that the conclusion plans.
- `force` on `DELETE /changes/{name}` is `false` by default. If the change has a start marker or a checked task, the server needs `force=true`. Without it, the server answers 409 `refused` and deletes nothing. Corrupt metadata gives 422 `invalid_config`, also with `force=true`. `unlinkedDiscussions` lists the source discussions that the server unlinked, with their new status.
- `POST /changes/{name}/claim`: a repeat claim by the same person succeeds and writes nothing. If another person holds the claim, the server answers 409 `refused`. The `message` names the current holder.
- `POST /changes/{name}/depends`: `on` is required, and `remove` is `false` by default. The server answers 409 `refused` for a self-dependency, a missing or archived target, or a dependency cycle.
- `POST /changes/{name}/in-progress`: a repeat call, or an unknown change name, also gives 200 `{}`. The server writes nothing.
- `DELETE /changes/{name}/in-progress`: checked tasks or recorded files give 409 `refused`. The envelope then also has `"checkedTasks":1,"touchedFiles":["src/login.rs"]`.
- `POST /changes/{name}/archive`: each `specs` item has `capability` and four requirement counts: `added`, `modified`, `removed`, and `renamed`. `carryReview=true` and `carryVerify=true` archive the change with an open ticket of that station. The ticket moves into the archive with the change.
- `tasks/{taskId}/done`: `touchedFiles` lists the files of the task. `headCommit` is the commit that you saw them on. Both are optional, but send at least `{}`. A task that is already checked gives 200 with `alreadyDone` set to `true`.
- `tasks/move`: `from` and `to` are task numbers that start at 1. The optional `before` puts the task before the target when `true`, and after it when `false`. Without `before`, the move direction decides. A number out of range gives 409 `refused`.

### Write an artifact

First, read the artifact to get its `version`:

```text
GET /changes/add-login/artifacts/tasks
→ {"artifact":"tasks","content":"## 1. Login\n\n- [ ] 1.1 Add the login form <!-- speclink-task:tsk_01M4… -->\n…","version":2}
```

Then write it back with that `version`:

```text
PUT /changes/add-login/artifacts/tasks
If-Match: 2
{"content":"## 1. Login\n\n- [ ] 1.1 Add the login form\n…"}
→ {"artifact":"tasks","version":<new version>}
```

- To create an artifact that does not exist yet, use `If-Match: 0`.
- When you write `tasks.md`, the server adds a stable ID comment (`<!-- speclink-task:tsk_… -->`) to the end of each task line.

### Quality gates

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `POST /changes/{name}/review/rounds` | `{"content":"<Markdown of one ticket round>"}` | `{"round":1}` |
| `POST /changes/{name}/review/stamp` | `{"accept":false,"agent":"claude-code","scope":[{"path":"src/login.rs","hash":"<64 characters>"}],"missing":[]}` | `{"change":"add-login"}` |
| `DELETE /changes/{name}/review` | None | `{"change":"add-login"}` |

- A round uses the same format as the stdin of `speclink review add-round`. It must have a `**Scope**:` line.
- `GET /changes/{name}/review` gives `change`, `rounds`, `lastRound`, and `content` (the ticket text). Each round has `index`, `phase`, `patchHash`, `scope`, and `findings`. Each finding has `severity`, `path`, and `text`.
- The `scope` of a stamp holds content fingerprints from your working tree. It covers the files that the Scope lines of the ticket list. `hash` is the SHA-256 of the file content in lowercase hex. For UTF-8 text files, change CRLF to LF before you hash.
- Put files that no longer exist in your working tree into `missing`. `scope` plus `missing` must be exactly the files that the ticket lists. The server does not hash files again; it only compares the file sets.
- All code tasks of the change must be done before a stamp. `accept: true` stamps even with must-fix findings (CRITICAL or WARNING) in the last round.

### Discussions

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `POST /discussions` | `{"topic":"看板搜尋列","slug":"board-search-bar"}` | `{"slug":"board-search-bar","topic":"看板搜尋列","path":"discussions/board-search-bar.md"}` |
| `PUT /discussions/{slug}/context` | `{"content":"…"}` | `{}` |
| `POST /discussions/{slug}/rounds` | `{"mode":"interview","content":"…"}` | `{"round":1}` |
| `POST /discussions/{slug}/conclude` | `{"content":"…","hold":true}` | `{"restaleFlagged":[],"held":true}` |
| `POST /discussions/{slug}/promote` | `{"name":"add-search-bar"}` | `{"change":"add-search-bar"}` |
| `POST /discussions/{slug}/link` | `{"change":"add-auth"}` | `{"slug":"auth-scope","change":"add-auth"}` |
| `POST /discussions/{slug}/seal` | `{"change":"add-auth"}` | `{"slug":"auth-scope","change":"add-auth"}` |
| `POST /discussions/{slug}/archive` | None | `{"archivedTo":"discussions/archive/<file name>"}` |
| `DELETE /discussions/{slug}?force=true` | None | `{"slug":"board-search-bar"}` |

- `POST /discussions`: `slug` is optional; without it, the server makes one from `topic`. A slug uses lowercase letters and digits with single hyphens. A bad slug gives 400 `invalid_argument`. The optional `kind` accepts only `improve` today.
- `conclude`: `hold: true` keeps the record live after the conclusion, because it still owes a change. `restaleFlagged` lists the changes that a new conclusion sends back for a refresh. `autoArchived: true` appears only when the conclusion also archives the record.
- `promote`: `name` is optional. `promote` and `seal` accept an optional `last: true`. It marks the final change that the conclusion plans, and it clears the hold flag of the record.
- `seal`: first link the change to the discussion with `link`. Without the link, the server answers 409 `refused`.
- `DELETE /discussions/{slug}`: `force` is `false` by default. A discussion with rounds needs `force=true`; without it, the server answers 409 `refused`. The server does not delete an archived discussion; it also answers 409 `refused`.

### Project data

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `PUT /config` | `{"content":"schema: spec-driven\n","expectedRevision":16}` | `{"revision":17}` |
| `PUT /board-order` (with `If-Match: "16"`) | `{"content":"{\"changes\":{\"add-logout\":\"a\"}}"}` | `{"revision":17}` |

- `PUT /config` takes the full config document. The server parses it first and answers 422 `invalid_config` when it cannot.
- `POST /context` can take `change` in the body to get only the documents of that change. Send `{}` to get all documents. The response has `snapshotId`, `digest`, and `documents`. Each document has `path`, `content`, `revision`, and `digest`. The CLI uses it to mirror the server documents into the read-only local folder `.speclink/context/`.
- The board-order `content` is JSON text in the form `{"changes":{"<change>":"<rank key>"},"discussions":{…}}`. The server stores it but does not parse it. `GET /plan` reads it and treats unreadable content as "no order".

The response of `GET /changes/{name}/validate`:

```json
{ "change": "add-login", "valid": true, "errors": [], "warnings": ["No delta specs found"] }
```

The endpoint validates one change per call. To validate several changes, call `GET /changes` and then call this endpoint for each change. `validate --all` in the CLI does the same.

The response of `GET /plan`:

```json
{
  "waves": [{ "index": 1, "changes": ["add-login"] }, { "index": 2, "changes": ["add-logout"] }],
  "changes": [
    { "name": "add-login", "wave": 1, "stage": "in-progress", "dependsOn": [], "overlaps": [], "blockedBy": [], "ready": true, "requirementOverlap": [], "archiveAfter": [] },
    { "name": "add-logout", "wave": 2, "stage": "proposed", "dependsOn": ["add-login"], "overlaps": [], "blockedBy": ["add-login"], "ready": false, "requirementOverlap": [], "archiveAfter": [] }
  ],
  "next": null,
  "skipped": []
}
```

A dependency cycle gives 409 `refused`. The `message` is, for example, `dependency cycle: add-login -> add-logout -> add-login`.

## <a id="response-fields"></a>Optional fields in lists and details

When an optional field below has no value, its key does not appear. A client must not fill in a default for a missing field.

### Each item of `GET /changes`

Always present: `name`, `summary`, `status`, `completedTasks`, `totalTasks`.

Optional:

| Field | Content |
| --- | --- |
| `startedAt` | The start date, `YYYY-MM-DD`. It does not appear before the start. |
| `createdBy`, `created` | The creator and the creation date |
| `fromDiscussions` | Source discussion slugs |
| `claimedBy` | The claim holder. It does not appear without a claim. |
| `deltaCapabilities` | The capabilities that the delta specs of this change touch |
| `restaleFrom` | Source discussions with a new conclusion that this change must take in again |
| `metaError` | The cause, when the metadata is corrupt |

```json
{ "name": "add-login", "summary": "", "status": "in-progress", "completedTasks": 0, "totalTasks": 2, "startedAt": "2026-10-07", "createdBy": "Demo <demo@example.com>", "created": "2026-10-07" }
```

### `GET /changes/{name}`

Always present: `changeName`, `schemaName`, `isComplete`, `applyRequires`, and `artifacts`. Each artifact has `id`, `outputPath`, and `status`, plus `missingDeps` when it is blocked.

Eight optional fields: `created`, `fromDiscussions`, `deltaCapabilities`, `createdBy`, `createdWith`, `startedAt`, `startedBy`, and `claimedBy`. `created` appears only when the metadata holds both the schema and the creation date. CLI `show` in remote mode and the detail panel of the desktop app use these fields.

```json
{ "changeName": "add-login", "schemaName": "spec-driven", "isComplete": false, "applyRequires": ["tasks"], "artifacts": [{ "id": "tasks", "outputPath": "tasks.md", "status": "done" }], "created": "2026-10-07", "createdBy": "Demo <demo@example.com>", "createdWith": "claude", "startedAt": "2026-10-07", "startedBy": "Demo <demo@example.com>" }
```

### Each item of `GET /discussions`

Always present: `slug`, `topic`, `status`, `rounds`, `created`, `path`, `archived`. The official server also always fills `concluded` (the conclusion has text) and `hold` (the record stays live).

Optional: `createdBy`, `kind` (`improve` for an improve discussion), and `promotedTo`. `promotedTo` lists the changes that the discussion turned into, in the order of promotion. The server takes it directly from the discussion record. It does not appear when the discussion has no promotion.

```json
{ "slug": "board-search-bar", "topic": "看板搜尋列", "status": "promoted", "rounds": 1, "created": "2026-10-07", "createdBy": "Demo <demo@example.com>", "promotedTo": ["add-search-bar"], "concluded": true, "hold": true, "path": "discussions/board-search-bar.md", "archived": false }
```

### The local `worktree` field of `speclink list --json`

In local mode, `list --json` can add a `worktree` object to a change. It shows that a git worktree holds the implementation of the change:

```json
{ "completedTasks": 3, "name": "add-dark-mode", "status": "in-progress", "totalTasks": 5, "worktree": { "path": "/path/to/speclink.worktrees/add-dark-mode", "branch": "speclink/add-dark-mode" } }
```

- `path` is the absolute path of the worktree folder. `branch` is the full branch name (`speclink/<change>`).
- The field appears only when all these conditions are true: local mode, a run from the main checkout, the `worktree` policy on, and a worktree that matches the change. In all other cases, the key does not appear.
- Remote-mode `list` never has this field, because the server does not know your local checkout. Without worktrees, both modes therefore give the same fields.
- When the field appears, `completedTasks`, `totalTasks`, `status`, and `metaError` of the item come from the copy in the worktree, not from the main checkout.
