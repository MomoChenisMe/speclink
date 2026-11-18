# Verb and Flag Contract

[繁體中文](verb-contract.zh-TW.md) · **English**

## <a id="audience"></a>Who this document is for

Read this document if you write your own client, embed Speclink in your own system, or call Speclink over HTTP. If you only use the desktop app or the CLI, skip it. The [Complete SDD Workflow](workflow.md) covers daily use.

This document answers four questions:

- Which CLI verbs work in a local project and in a remote project, and how their output differs. A local project keeps the documents in the repo's `speclink/` (or `openspec/`, for a project that an older version created). In a remote project, one Server holds the documents, and `.speclink.yaml` has a `remote:` section. Below, "local mode" and "remote mode" mean how the CLI works in these two kinds of project.
- How to use the verbs for landing reports, free fields, and evidence, and which fields `--json` has now.
- What the shared rules, error codes, and endpoints of the remote HTTP API are.
- What the request and response bodies look like.

The HTTP entry point of the host kit serves these endpoints. A host that embeds the engine gives its requests to the entry point and gets the same set of endpoints. The official demonstration server (`speclink-server`) uses the same host kit, so the CLI and Speclink Desktop work the same way with both. To connect the entry point to your own backend, see ["Serve the CLI and Speclink Desktop" in the Node SDK document](sdk-node.md#http-entry) and [Embed Speclink in your system](../README.md#embedding) in the README.

If this document and the canonical specs disagree, the canonical specs win:

- [Canonical verb-contract spec](../speclink/specs/verb-contract/spec.md)
- [Canonical client-protocol spec](../speclink/specs/client-protocol/spec.md)

To start the demonstration server and make an access key (PAT), see [Remote Getting Started](remote-getting-started.md).

## <a id="mode-assignment"></a>Mode assignment of verbs

All verbs work in local projects and in remote projects, except the "Local only" and "Remote only" rows below.

| Assignment | Verbs | In the unsupported mode |
| --- | --- | --- |
| Local only | `demo` | Remote mode refuses with a non-zero exit code. The CLI sends no request, and it refuses offline too. |
| Remote only | `claim` | Local mode refuses with a non-zero exit code. stderr says that the verb needs a remote project. |
| Both modes | `list`, `show`, `validate`, `analyze`, `drift`, `trace`, `plan` (also with `--strict-overlap`), `change` (`depends`, `schema`, `rank`, `landed`, `field`), `archive`, `discard`, `artifact`, `evidence`, `language`, `status`, `instructions`, `new`, `workflow-config`, `task`, `in-progress`, `discuss`, `review`, `verify` | Not applicable. Remote mode reads and writes only the Server. It never falls back to local files. |
| Mode-free | `init`, `update`, `link`, `unlink`, `auth`, `schemas`, `templates`, `feedback`, `schema`, `config`, `completion` | Not applicable. These verbs do not read or write spec documents. |

`demo` is local only, because it writes a demo change into the local spec folder. `claim` is remote only: a claim records who on the team works on a change, and a local project has no such state.

In a remote project, the Server runs `trace`, `change rank`, and `plan --strict-overlap`. The output is the same as in a local project:

- `trace`: the Server builds the provenance chain with the same engine (`GET /trace/{capability}`).
- `change rank`: the order goes into the board order on the Server (`POST /changes/{name}/rank`).
- `plan --strict-overlap`: the Server computes the strict-overlap plan (`GET /plan?strictOverlap=true`).

With an older Server that does not have these routes, all three stop with a non-zero exit code. `trace` and `change rank` get 404. The response to `plan --strict-overlap` has no `strictOverlap` mark, so the CLI says that this Server does not support it. It does not print the plain plan as the strict-overlap plan.

Four subcommands read git and the working tree on your machine: `drift`, `review prepare`, `review scope`, and `verify scope`. The CLI runs them in the checkout of a remote project as usual. A host without a checkout refuses them when a call comes through the call entry point (`host.dispatch`). See [Known gaps](product-status.md#known-gaps) in Product status.

## <a id="output-differences"></a>Output differences between modes

For verbs that work in both modes, the human-readable output (also with `--no-color`) must be identical. The table below shows all known differences.

| # | Verb | What remote mode does differently |
| --- | --- | --- |
| 1 | `new change` | It does not print the `Path:` line. A path on the Server means nothing to you. |
| 2 | `list` | It shows no worktree marker. A worktree belongs to the local checkout. |
| 3 | `status --schema` | It refuses. The Server's workflow config decides the schema. |
| 4 | `workflow-config` | The document label is always `config.yaml`. |
| 5 | `discuss promote` | It does not print the `Path:` line or the hint line after it. |
| 6 | `list` | Local mode sorts by last modification by default. Remote mode sorts by name. `--sort created` and `--sort modified` also give name order in remote mode. |
| 7 | `validate` (several changes) | Local mode orders the results by last modification. Remote mode orders them by name. Remote mode never validates changes in strict mode; `--strict` applies to `--specs` only. |
| 8 | `instructions --schema` | It refuses, for the same reason as item 3. |
| 9 | `archive` | It archives one named change at a time. It refuses `--all` and several names. `--skip-specs`, `--no-validate`, and `--mark-tasks-complete` have no effect. |

Items 1–5 are the differences that the canonical spec declares. Items 6–9 show the current code; the canonical spec does not list them yet.

The `--json` field set and its camelCase names are a frozen contract. Fields do not change names and do not disappear. No `--json` output contains ticket text. A new field is always optional. See [the next section](#verb-reference).

## <a id="verb-reference"></a>Verbs, flags, and `--json` fields

This section lists the verbs for landing reports, free fields, and evidence, the login method flags of `auth login`, and the optional fields of `list --json` and `show --json`. `auth login` is only for remote projects. All other verbs and flags here work in local projects and in remote projects, with the same output. Each verb accepts `--no-color`.

### `speclink evidence show <change>`

It prints the files that each completed task of the change touched. `speclink task done` writes this record. `evidence show` only reads it.

- `<change>` is an active change first. If no active change has the name, it is an archived change. For an archived change, give the archived name with its date, or only the change name (the newest archive with that name).
- Flag: `--json`.
- A change with no record gives one line of text and exit code 0. An unknown name gives a non-zero exit code.
- The `--json` fields:

| Field | Content |
| --- | --- |
| `change` | The change name |
| `archivedAs` | The archived name with its date. It appears only when the record comes from an archived change. |
| `entries` | One entry for each task: `taskId`, `taskDesc`, `touchedFiles`, and `recordedAt`. `actor`, `repo`, and `headCommit` appear only when they have a value. |
| `files` | The files of all tasks, each one once, in sorted order |

### `speclink change landed <change>`

It reports the commit that the code of the change landed on.

- Flags: `--commit <sha>`, `--branch <name>`, `--json`.
- Without `--commit`, the CLI uses the HEAD of the current checkout. Outside a git checkout and without `--commit`, it stops with a non-zero exit code.
- `--branch` replaces the branch that the change records. Without it, the recorded branch stays.
- **It only records. It checks nothing.** The engine stores the commit, the branch (when you give one), the time of the report, and the reporter. It does not read git. It does not check that the commit exists or that it is on the main branch. A rule such as "no archive before the code lands" belongs to the host, in its `beforeCommand` hook.
- `--commit` must have the correct shape: 7 to 64 hexadecimal characters. A branch name has 1 to 255 characters, with no whitespace and no control characters. A wrong shape is refused, and nothing is written.
- A new report replaces the last one. You can also report for a change that is still Proposed. The report does not change its stage.
- `--json` prints `{"change":"add-login","commit":"<sha>","branch":"speclink/add-login"}`. `branch` appears only when this call has `--branch`.

The `commit` and `worktree-merge` skills run it for you after a commit or a merge. See [the workflow](workflow.md#commit).

### Free fields: `change field` and `discuss field`

Free fields are your own key/value data, for example the number of the related ticket. Speclink stores them and never reads them: no verb changes its behavior because of a field. Do not put a password or a key in a field.

| Verb | What it does |
| --- | --- |
| `speclink change field set <change> <key>=<value>...` | Sets one or more fields of a change |
| `speclink change field unset <change> <key>...` | Removes fields of a change |
| `speclink discuss field set <slug> <key>=<value>...` | Sets one or more fields of a discussion |
| `speclink discuss field unset <slug> <key>...` | Removes fields of a discussion |

- All four verbs accept `--json`.
- For `set`, the value is all text after the first `=`. With several pairs, the verb writes all of them or none.
- For `unset`, a key that does not exist is not an error.
- A key starts with a lowercase letter. After that, it has lowercase letters, digits, `_`, `.`, and `-`. Its maximum length is 64 characters.
- A value is one line of text with no control characters. Its maximum size is 1024 bytes. A value is always a string.
- A change or a discussion holds 32 keys at most.
- An unknown change or discussion, a key or value that breaks the rules, or more than 32 keys gives a non-zero exit code, and nothing is written. You cannot change the fields of an archived change or discussion.
- `--json` prints all fields after the write: `{"change":"add-login","fields":{"ticket":"PROJ-1"}}`. For a discussion, the key is `discussion`.

To set fields at creation, use `--field <key>=<value>`. You can repeat it:

```bash
speclink new change add-login --field ticket=PROJ-1 --field source=portal
speclink discuss new "Board search bar" --field owner=alice
```

The fields go into the same write as the creation. An older Server that does not know free fields still creates the change or the discussion, but it ignores the fields. The CLI then prints a warning.

### Login method flags of `auth login`

Without a flag, `speclink auth login` first asks the Server which login methods it supports (`GET /api/speclink/v1/auth-methods`). Then it uses the first one in this fixed order: browser login, device login, access key. If the Server declares no methods, the CLI tries device login first, as before. Four flags select a method:

| Flag | What it does |
| --- | --- |
| `--browser` | Signs in through the login service that the Server declares, in a browser. If the Server does not declare browser login, the CLI stops with a non-zero exit code. |
| `--device` | Authorizes this device with a code |
| `--pat` | Asks you to paste an access key |
| `--token-stdin` | Reads an access key from stdin, for CI and scripts |

For the full login steps, see [Remote Getting Started](remote-getting-started.md#cli).

### New fields of `list --json` and `show --json`

All fields below are optional. When a field has no value, its key does not appear. Thus a project that does not use these features gets the same output as before, byte for byte.

Each change in `speclink list --json`:

| Field | Content |
| --- | --- |
| `created` | The creation date, `YYYY-MM-DD` |
| `modifiedAt` | The time of the last modification of a document of the change, as an RFC 3339 UTC time. It does not appear when the store cannot tell. In a local project, the value comes from file times, and a checkout or a copy also changes them. Use it only to estimate how long a change was idle. |
| `fields` | The free fields |

`list --json` has no quality station fields, also in a remote project.

`speclink show <change> --json`:

| Field | Content |
| --- | --- |
| `review`, `verify` | The state of the review station and of the verify station. `state` is `inProgress` (an open ticket) or `stamped`. A stamped station also has `stampedAt`, `rounds`, and `tasksTotal`. `stampedBy`, `stampedWith`, and `commit` (the HEAD of the checkout at the stamp) appear only when they have a value. A station with no ticket and no stamp has no key. |
| `branch` | The branch that the start marker recorded, or the branch from the landing report |
| `landed` | The landing record: `commit` and `at` (the report time, RFC 3339). `by` appears only when the report has a reporter. |
| `fields` | The free fields |

```json
{
  "name": "add-login",
  "review": { "state": "stamped", "stampedAt": "2026-11-02T14:03:44+08:00", "rounds": 2, "tasksTotal": 5, "commit": "0123456789ab" },
  "verify": { "state": "inProgress" },
  "branch": "speclink/add-login",
  "landed": { "commit": "0123456789ab", "at": "2026-11-03T10:10:00+08:00", "by": "Demo <demo@example.com>" },
  "fields": { "ticket": "PROJ-1" }
}
```

The example leaves out the fields that existed before, such as `schema`, `proposal`, and `tasks`. The free fields also appear in the discussion data of `discuss list --json`, `discuss show --json`, and `discuss search --json`.

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

The Server decides which credentials it accepts. The demonstration server accepts two kinds. One is an access key (PAT) that starts with `spk_pat_`; you make it on the account page of the server. The other is the device login credential from `speclink auth login`; it starts with `spk_at_`. A valid credential of a user who is not a project member gives 403. A host that embeds the engine verifies the credential itself and gives the identity to the entry point. When it cannot verify an identity, it passes an empty value, and the entry point answers 401. When the host declares browser login, the credential is the access token from its login service.

### Roles

On the demonstration server, a project member has the `reader` role or the `editor` role. A reader can only read. Every write endpoint (an endpoint whose method is not GET) needs `editor`. A reader gets 403 `permission_denied` with the `message` `your role in project '<key>' is reader; this action needs the editor role`, and the Server writes nothing. There are three exceptions:

- `POST /context` and `POST /skills` send their query with POST, but they only read data, so a reader can call them.
- `GET /export` gives out all documents of the scope in one response, so it needs `editor`.

A host that embeds the engine does not have these two role names. Its scope function says, for each repo, whether the caller can write, and the entry point applies that answer. In the endpoint tables below, the minimum role `editor` means that the caller must be able to write.

### Handshake

Call `GET /binding` first. It tells you who you are, which project and repo you use, the Server version, and the capabilities of your role:

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
    },
    "skills": { "version": "v1.58.0" },
    "fields": true
  }
}
```

- If `apiVersion` is not the version that you sent, stop. Do not call other endpoints.
- `policyWrite`, `deleteChange`, and `moveTask` are `true` only for editors. `validate` and `analyze` are `true` for all roles.
- `events.transports` lists the push streams of the Server. The example above is the response of the demonstration server. A host that has no stream of its own sends an empty array. The client then polls `GET /sync-state`, which `polling` names.
- `skills.version` is the version of the skills that the Server provides. `POST /skills` gives the skills. `fields` is `true` when the Server stores free fields. An older Server does not send these two keys.
- `capabilities` is only a hint for disabling buttons in a UI. The Server checks permissions on each request.

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

A stale version gives 409 `revision_conflict`, and the Server writes nothing. The first two endpoints give 400 `invalid_argument` when `If-Match` is missing. Other write endpoints do not check versions.

## <a id="repo-ownership"></a>Repo ownership of changes

Each change belongs to exactly one repo. Split a requirement that touches two repos into several changes.

- When remote mode creates a change (`POST /changes`), the change belongs to the repo in `X-Speclink-Repo`. In a project with one repo, the Server uses that repo. The `repo` field of the response shows the owner.
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
| `permission_denied` | 403 | The account is not a project member, or a reader calls a write endpoint. |
| `not_found` | 404 | The project or repo is not registered, or the change, discussion, artifact, spec, or capability does not exist. |
| `invalid_argument` | 400 | A value is wrong: a bad slug, a missing `If-Match`, a non-numeric `If-Match`, an unknown artifact, or an empty search query. Or the written content has a bad format: a `tasks.md` with no `- [ ]` checkbox, a proposal without `## Why`, `## Problem`, or `## Summary`, a design without `## Context`, a delta spec without an operation section, or a review or verify round without a `**Scope**:` line, with a `**Phase**:` that does not fit the ticket state, or with a bad applied-rules record (`**Rules**:` or `**Rule**:` lines in a bad format, or in a round that is not a discovery round). |
| `invalid_config` | 422 | The change metadata is corrupt, or the Server cannot parse the workflow config. |
| `refused` | 409 | A precondition fails: API version mismatch, no repo in a multi-repo project, `force` needed, task number out of range, change claimed by someone else, dependency cycle, import into a non-empty target, or an import bundle with an unknown format version. |
| `refused` | 413 | The request body is larger than the size limit (the host sets the import limit; the default is 32 MiB), or the board-order content is larger than 1 MiB. |
| `revision_conflict` | 409 | The version that you sent with a write is stale (see [Shared request rules](#request-rules)). |
| `unavailable` | 503 | The store is temporarily unavailable, or the repo changes all the time during an export. |
| `internal` | 500 | All other failures. |

## <a id="endpoints"></a>Endpoint reference

All paths follow the [project URL](#request-rules). `{name}` is a change name, and `{slug}` is a discussion slug. `{artifact}` is `proposal`, `design`, `tasks`, or `specs/<capability>`. "Minimum role" `reader` means that all members can call the endpoint.

The routes below are in the HTTP entry point of the host kit, except where a row says something different. A host that embeds the engine and the demonstration server both have them.

### Connection and sync

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/binding` | reader | Handshake: identity, project, repo, versions, and capabilities |
| GET | `/whoami` | reader | The current user and all repos of the project |
| GET | `/sync-state` | reader | Only the `ETag`. With a matching `If-None-Match`, it answers 304. |
| GET | `/events` | reader | SSE event stream. See [Event stream](#events) below. This route belongs to the demonstration server. It is not in the entry point of the host kit. |
| POST | `/context` | reader | One consistent snapshot of the documents. The body can set `change` to narrow it. |
| POST | `/skills` | reader | Get the skill files of this project (`speclink update` uses it in a remote project) |

### Changes

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/changes` | reader | List the changes of this repo (`speclink list`) |
| POST | `/changes` | editor | Create a change (`speclink new change`) |
| GET | `/changes/{name}` | reader | Status and metadata of one change (`speclink status`, `show`) |
| DELETE | `/changes/{name}?force=<bool>` | editor | Discard a change (`speclink discard`) |
| GET | `/changes/{name}/drift` | reader | Drift data from the spec side (`speclink drift`) |
| GET | `/changes/{name}/validate` | reader | Validate one change (`speclink validate`) |
| GET | `/changes/{name}/analyze` | reader | Cross-artifact analysis report (`speclink analyze`) |
| GET | `/changes/{name}/instructions/{kind}` | reader | Artifact instructions. `{kind}` is `proposal`, `design`, `specs`, or `tasks`; `apply` gives the implementation progress (`speclink instructions`). |
| GET | `/changes/{name}/artifacts/{artifact}` | reader | Read an artifact and its `version` (`speclink artifact cat`) |
| PUT | `/changes/{name}/artifacts/{artifact}` | editor | Write an artifact. It needs `If-Match`. |
| GET | `/changes/{name}/evidence` | reader | Completion evidence: the files that `task done` records for each task (`speclink evidence show`). `{name}` can also be the name of an archived change. |
| POST | `/changes/{name}/in-progress` | editor | Mark the change as started (`speclink in-progress add`) |
| DELETE | `/changes/{name}/in-progress` | editor | Remove the start marker (`speclink in-progress remove`) |
| POST | `/changes/{name}/claim` | editor | Claim the change (`speclink claim`) |
| POST | `/changes/{name}/depends` | editor | Declare or remove prerequisite changes (`speclink change depends`) |
| POST | `/changes/{name}/schema` | editor | Switch the schema of the change (`speclink change schema`) |
| POST | `/changes/{name}/rank` | editor | Put the change before or after another change in the same column (`speclink change rank`) |
| POST | `/changes/{name}/landed` | editor | Report the commit that the code landed on (`speclink change landed`) |
| PUT | `/changes/{name}/fields` | editor | Set and remove free fields of the change (`speclink change field set`, `unset`) |
| POST | `/changes/{name}/archive?carryReview=<bool>&carryVerify=<bool>` | editor | Archive (`speclink archive`) |

### Tasks

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| POST | `/changes/{name}/tasks/{taskId}/done` | editor | Check a task (`speclink task done`). `{taskId}` is a task number or a stable ID that starts with `tsk_`. |
| POST | `/changes/{name}/tasks/{taskId}/undone` | editor | Uncheck a task (`speclink task undone`) |
| POST | `/changes/{name}/tasks/move` | editor | Move a task and renumber the list (drag in the desktop app) |

### Quality gates

`{station}` is `review` or `verify`. Both stations use the same endpoints and payloads.

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/changes/{name}/{station}` | reader | Read the ticket (`speclink review show`) |
| POST | `/changes/{name}/{station}/rounds` | editor | Add a round (`speclink review add-round`) |
| POST | `/changes/{name}/{station}/stamp` | editor | Stamp (`speclink review stamp`) |
| DELETE | `/changes/{name}/{station}` | editor | Discard the ticket (`speclink review discard`) |

### Discussions

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/discussions?archived=<bool>` | reader | List discussions. `archived=true` lists the archived ones. |
| POST | `/discussions` | editor | Create a discussion (`speclink discuss new`) |
| GET | `/discussions/search?q=<keywords>` | reader | Search discussions. Separate keywords with spaces (`speclink discuss search`). |
| GET | `/discussions/{slug}` | reader | Read one discussion and its full text (`speclink discuss show`) |
| DELETE | `/discussions/{slug}?force=<bool>` | editor | Delete a discussion (`speclink discuss discard`) |
| PUT | `/discussions/{slug}/context` | editor | Write the Context section (`speclink discuss context`) |
| PUT | `/discussions/{slug}/fields` | editor | Set and remove free fields of the discussion (`speclink discuss field set`, `unset`) |
| POST | `/discussions/{slug}/rounds` | editor | Add a round (`speclink discuss add-round`) |
| POST | `/discussions/{slug}/conclude` | editor | Write the conclusion (`speclink discuss conclude`) |
| POST | `/discussions/{slug}/archive` | editor | Archive the discussion (`speclink discuss archive`) |
| POST | `/discussions/{slug}/promote` | editor | Turn the discussion into a change (`speclink discuss promote`) |
| POST | `/discussions/{slug}/link` | editor | Link an existing change to the discussion (`speclink discuss link`) |
| POST | `/discussions/{slug}/seal` | editor | Mark the discussion as promoted after its content lands (`speclink discuss seal`) |

### Project data

| Method | Path | Minimum role | Purpose |
| --- | --- | --- | --- |
| GET | `/plan` | reader | Execution order of the changes (`speclink plan`) |
| GET | `/plan?strictOverlap=true` | reader | Execution order with strict overlap (`speclink plan --strict-overlap`). The response also has `strictOverlap: true`. |
| GET | `/trace/{capability}` | reader | Provenance chain of one capability (`speclink trace`) |
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
| GET | `/export` | editor | Export all documents of this repo as one version 2 bundle (see [Export and import bundles](#bundles)) |
| POST | `/import` | editor | Write one bundle into this repo in one step (the desktop app uses it to move a local project). The target repo must be empty. |

The CLI has no export or import verb at this time. The desktop app and the host's own code use these two routes.

### Endpoints outside the project URL

These endpoints follow `https://<server>` directly. The first three are in the entry point of the host kit:

| Method | Path | Needs | Purpose |
| --- | --- | --- | --- |
| GET | `/api/speclink/v1/auth-methods` | Nothing | The login methods that the Server supports. A client calls it before login, with no credential and no version header. |
| GET | `/api/speclink/v1/whoami` | A credential only | The owner of the credential (use it before you select a project) |
| GET | `/api/speclink/v1/scopes` | A credential only | The projects and repos that this account can see |

The response of `GET /api/speclink/v1/auth-methods`:

```json
{ "methods": ["browser", "device", "token"], "browser": { "issuer": "https://id.example.com", "clientId": "speclink", "scopes": ["openid"] } }
```

- The values of `methods` are `browser` (browser login), `device` (device login), and `token` (access key). A client uses the first one in this fixed order that the Server lists. An unknown value is not an error.
- The `browser` object appears only with browser login. `issuer` is the URL of the login service, and `clientId` is the client ID. `scopes` is optional.
- A host that declares no login methods answers 404. The client then tries device login first, as before, and asks for an access key if that fails. The demonstration server answers `device` and `token`.

The routes below belong to the demonstration server. They are not in the entry point of the host kit:

| Method | Path | Needs | Purpose |
| --- | --- | --- | --- |
| GET | `/healthz` | Nothing | 200 while the process runs |
| GET | `/readyz` | Nothing | 200 when the store can serve, else 503 |
| GET | `/auth/whoami` | A credential only | The owner of the credential. This is the older path; a new client asks `/api/speclink/v1/whoami` first. |

`/auth/device`, `/auth/device/token`, `/auth/refresh`, and `/auth/revoke` are the device login flow of the demonstration server. The device login of `speclink auth login` uses them.

### <a id="events"></a>Event stream

Each commit of a write carries events. The Server decides how it pushes events to clients, and it declares that in `events.transports` of the handshake. The demonstration server has `GET /events`, an SSE stream (Server-Sent Events: the Server pushes events over one long-lived connection). A host that embeds the engine with Node has no ready stream at this time, so its clients poll `GET /sync-state`.

- `invalidate` event: a resource changed. The `data` is, for example, `{"eventId":"42","scope":"change","resourceId":"add-login","revision":42}`. Read the resource again when you get it.
- `reset` event: the Server removed your resume position. Read all data again.
- When you reconnect, send `Last-Event-ID` to get the events that you missed.
- When the stream is idle, the Server sends heartbeat comment lines to keep the connection open.

The kind of the event decides `scope` and `resourceId`. A host that pushes events itself uses the same table:

| Event kind | `scope` | `resourceId` |
| --- | --- | --- |
| `change-created`, `artifact-created`, `task-completed`, `task-uncompleted`, `task-moved`, `change-claimed`, `change-marked-in-progress`, `change-in-progress-removed`, `change-depends-changed`, `change-schema-changed`, `change-ranked`, `change-fields-changed`, `change-landed`, `change-discarded` | `change` | The change name |
| `review-round-added`, `review-stamped`, `review-discarded`, `verify-round-added`, `verify-stamped`, `verify-discarded` | `change` | The change name |
| `change-archived` | `spec` | The change name |
| `discussion-created`, `discussion-context-set`, `discussion-round-added`, `discussion-concluded`, `discussion-promoted`, `discussion-linked`, `discussion-sealed`, `discussion-archived`, `discussion-fields-changed`, `discussion-discarded` | `discussion` | The discussion slug |
| All others (for example `scope-imported` of an import) | `unknown` | An empty string |

- The six events of the review station and the verify station belong to the change: a new round, a stamp, and a discarded ticket. `resourceId` is the change name. A client that reads the change again gets the new station state.
- The payload of `change-landed` has `change` and `commit`. It has `branch` only when the report gave a branch.
- The payload of the two free-field events has only the name and the changed keys (`keys`). It has no values.
- When you get `unknown`, read all data again. An import changes the full scope, so it is in this group on purpose.

## <a id="payload-examples"></a>Request and response examples

The field shapes below come from real responses of the demonstration server. If the request body column says "None", send `{}` or no body. All other endpoints need a JSON body and `Content-Type: application/json`.

### Changes and tasks

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `POST /changes` | `{"name":"add-login","agent":"claude"}` | `{"name":"add-login","schema":"spec-driven","repo":"backend"}` |
| `DELETE /changes/{name}?force=true` | None | `{"change":"add-search-bar","unlinkedDiscussions":[{"slug":"board-search-bar","status":"concluded"}]}` |
| `POST /changes/{name}/claim` | None | `{"claimedBy":"Demo <demo@example.com>"}` |
| `POST /changes/{name}/depends` | `{"on":["add-login"],"remove":false}` | `{"change":"add-logout","dependsOn":["add-login"]}` |
| `POST /changes/{name}/schema` | `{"schema":"no-spec-change"}` | `{"change":"tidy","schema":"no-spec-change","previous":"spec-driven","changed":true,"verifyStampCleared":false}` |
| `POST /changes/{name}/rank` | `{"anchor":"add-login","position":"before"}` | `{"change":"add-logout","anchor":"add-login","position":"before"}` |
| `POST /changes/{name}/landed` | `{"commit":"0123456789ab","branch":"speclink/add-login"}` | `{"change":"add-login","commit":"0123456789ab","branch":"speclink/add-login"}` |
| `PUT /changes/{name}/fields` | `{"set":{"ticket":"PROJ-1"},"unset":["source"]}` | `{"fields":{"ticket":"PROJ-1"},"changed":["source","ticket"]}` |
| `POST /changes/{name}/in-progress` | None, or `{"branch":"speclink/add-login"}` | `{}` |
| `DELETE /changes/{name}/in-progress` | None | `{"removed":true}`; `{"removed":false}` when the change was not started |
| `POST /changes/{name}/archive` | None | `{"specs":[],"datedName":"2026-10-07-fix-typo","snapshotCreated":false,"archivedDiscussions":[],"evidenceRecorded":false}` |
| `POST /changes/{name}/tasks/{taskId}/done` | `{"touchedFiles":["src/login.rs"],"headCommit":"0123…"}` | `{"taskDesc":"1.1 Add the login form","alreadyDone":false}` |
| `POST /changes/{name}/tasks/{taskId}/undone` | None | `{"taskDesc":"1.1 Add the login form","alreadyUndone":false}` |
| `POST /changes/{name}/tasks/move` | `{"from":2,"to":1}` | `{"change":"add-login","description":"1.1 Add the session check"}` |

Notes:

- `POST /changes` has these optional fields: `schema`, `description`, `agent`, `fromDiscussion` (the source discussion slug), `last`, and `fields`. With `fromDiscussion`, `last` marks the final change that the conclusion plans. `fields` holds the free fields that the change starts with; they go into the same write as the creation.
- `force` on `DELETE /changes/{name}` is `false` by default. If the change has a start marker or a checked task, the Server needs `force=true`. Without it, the Server answers 409 `refused` and deletes nothing. Corrupt metadata gives 422 `invalid_config`, also with `force=true`. `unlinkedDiscussions` lists the source discussions that the Server unlinked, with their new status.
- `POST /changes/{name}/claim`: a repeat claim by the same person succeeds and writes nothing. If another person holds the claim, the Server answers 409 `refused`. The `message` names the current holder.
- `POST /changes/{name}/depends`: `on` is required, and `remove` is `false` by default. The Server answers 409 `refused` for a self-dependency, a missing or archived target, or a dependency cycle.
- `POST /changes/{name}/schema`: the Server resolves built-in schemas only. It answers 409 `refused` for an unknown schema, for a switch to a schema without specs while the change has delta specs, and for an open verify ticket. The same schema gives 200 with `changed` set to `false`, and the Server writes nothing. `verifyStampCleared` is `true` when the switch removed the verify stamp.
- `POST /changes/{name}/rank`: `anchor` is another change in the same column. `position` is `before` or `after`; any other value gives 400 `invalid_argument`. With the optional `force` set to `true`, the Server also moves a change that someone already ordered by hand.
- `POST /changes/{name}/landed`: `commit` is required, and `branch` is optional. The Server checks only the shape of the two values. It does not check that the commit exists. The response has `branch` only when the request has it.
- `PUT /changes/{name}/fields`: `set` holds the fields to set, and `unset` holds the keys to remove. One of the two must not be empty, or the Server answers 400. In the response, `fields` holds all fields after the write, and `changed` holds the keys that this write changed. When nothing changed, the Server writes nothing and the `ETag` stays the same. `PUT /discussions/{slug}/fields` uses the same request and response.
- `POST /changes/{name}/in-progress`: a repeat call, or an unknown change name, also gives 200 `{}`. The Server writes nothing. The body can have `branch`, the current branch of the checkout, and the Server records it.
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
- When you write `tasks.md`, the Server adds a stable ID comment (`<!-- speclink-task:tsk_… -->`) to the end of each task line.

### Quality gates

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `POST /changes/{name}/review/rounds` | `{"content":"<Markdown of one ticket round>"}` | `{"round":1}` |
| `POST /changes/{name}/review/stamp` | `{"accept":false,"agent":"claude-code","scope":[{"path":"src/login.rs","hash":"<64 characters>"}],"missing":[]}` | `{"change":"add-login"}` |
| `DELETE /changes/{name}/review` | None | `{"change":"add-login"}` |

- A round uses the same format as the stdin of `speclink review add-round`. It must have a `**Scope**:` line.
- `GET /changes/{name}/review` gives `change`, `rounds`, `lastRound`, and `content` (the ticket text). Each round has `index`, `phase`, `patchHash`, `rules`, `scope`, and `findings`. Each finding has `severity`, `path`, and `text`. `rules` is null, or an object with `baseline` (`extend` or `replace`) and the lists `critical`, `warning`, and `suggestion`; only a discovery round that recorded its applied rules has the object.
- The `scope` of a stamp holds content fingerprints from your working tree. It covers the files that the Scope lines of the ticket list. `hash` is the SHA-256 of the file content in lowercase hex. For UTF-8 text files, change CRLF to LF before you hash.
- Put files that no longer exist in your working tree into `missing`. `scope` plus `missing` must be exactly the files that the ticket lists. The Server does not hash files again; it only compares the file sets.
- All code tasks of the change must be done before a stamp. `accept: true` stamps even with must-fix findings (CRITICAL or WARNING) in the last round.
- The stamp body can have the optional `headCommit`: the current commit of your checkout. The Server checks only its shape and records it on the stamp. It then appears as `review.commit` (or `verify.commit`) in the change responses.

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

- `POST /discussions`: `slug` is optional; without it, the Server makes one from `topic`. A slug uses lowercase letters and digits with single hyphens. A bad slug gives 400 `invalid_argument`. The optional `kind` accepts only `improve` today. The optional `fields` holds the free fields that the discussion starts with.
- `conclude`: `hold: true` keeps the record live after the conclusion, because it still owes a change. `restaleFlagged` lists the changes that a new conclusion sends back for a refresh. `autoArchived: true` appears only when the conclusion also archives the record.
- `promote`: `name` is optional. The optional `fields` holds the free fields that the new change starts with. `promote` and `seal` accept an optional `last: true`. It marks the final change that the conclusion plans, and it clears the hold flag of the record.
- `seal`: first link the change to the discussion with `link`. Without the link, the Server answers 409 `refused`.
- `DELETE /discussions/{slug}`: `force` is `false` by default. A discussion with rounds needs `force=true`; without it, the Server answers 409 `refused`. The Server does not delete an archived discussion; it also answers 409 `refused`.

### Project data

| Endpoint | Request body | Success response |
| --- | --- | --- |
| `PUT /config` | `{"content":"schema: spec-driven\n","expectedRevision":16}` | `{"revision":17}` |
| `PUT /board-order` (with `If-Match: "16"`) | `{"content":"{\"changes\":{\"add-logout\":\"a\"}}"}` | `{"revision":17}` |

- `PUT /config` takes the full config document. The Server parses it first and answers 422 `invalid_config` when it cannot.
- `POST /context` can take `change` in the body to get only the documents of that change. Send `{}` to get all documents. The response has `snapshotId`, `digest`, and `documents`. Each document has `path`, `content`, `revision`, and `digest`. The CLI uses it to mirror the documents of the Server into the read-only local folder `.speclink/context/`.
- The board-order `content` is JSON text in the form `{"changes":{"<change>":"<rank key>"},"discussions":{…}}`. `PUT /board-order` stores the content as it is. `GET /plan` and `POST /changes/{name}/rank` read it, and they treat an unreadable section as "no order". When `rank` writes it back, it keeps only the rank keys of active changes, and it returns the discussions section unchanged.

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

`GET /plan?strictOverlap=true` gives the same shape, plus `"strictOverlap": true`. Only the strict-overlap plan has this mark. An older Server does not know the query parameter. It answers the plain plan without the mark. A client that gets no mark must treat the answer as "this Server does not support it", not as the strict-overlap plan.

The response of `GET /trace/{capability}` is the same as `speclink trace --json`:

```json
{
  "capability": "csv-export",
  "requirements": [{ "name": "Export report as CSV", "source": "add-csv-export" }],
  "changes": [
    { "name": "add-csv-export", "archivedDir": "2026-10-07-add-csv-export", "fromDiscussion": "csv-export-scope", "evidence": [{ "taskId": "tsk_01M4…", "files": ["src/export.rs"] }] }
  ],
  "discussions": [{ "slug": "csv-export-scope", "archived": true, "promotedTo": [{ "change": "add-csv-export", "capabilities": ["csv-export"] }] }]
}
```

- `changes` lists the archived changes that touched the capability, oldest first. `fromDiscussion` is `null` when the change has no source discussion. `evidence` is `null` when the change has no record.
- An unknown capability gives 404 `not_found`. The `message` lists the capabilities with similar names.

The response of `GET /changes/{name}/evidence`:

```json
{ "change": "add-login", "entries": [{ "taskId": "tsk_01M4…", "taskDesc": "1.1 Add the login form", "touchedFiles": ["src/login.rs"], "recordedAt": "2026-10-07T03:12:45Z" }] }
```

- `{name}` is an active change first. If no active change has the name, it is an archived change (the archived name, or the change name for the newest archive). A record from an archived change also has `archivedAs`.
- A change with no record gives an empty `entries` array. A name that is not an active change and not an archived change gives 404 `not_found`.
- `actor`, `repo`, and `headCommit` of an entry appear only when they have a value.
- The response always has `change`. An older Server reads only active changes and does not send this key. Without it, `evidence show` in the CLI stops with a non-zero exit code. It does not print the empty set as "no record".

### Skills

`POST /skills` gives the skills that a checkout of this project installs. It only reads data, so a reader can call it.

```text
POST /skills
{"targets":[{"tool":"claude"},{"tool":"my-agent","custom":{"invocation":"tool-call"}}],"contents":true}
→ {"version":"v1.58.0","targets":[{"tool":"claude","skills":[{"name":"apply","digest":"<digest>","content":"…"}]},{"tool":"my-agent","skills":[…]}]}
```

- `targets` lists the tools that you want skills for: one at least, and 32 at most. A built-in tool has only `tool`. A custom tool also has `custom.invocation`. An unknown tool gives 400 `invalid_argument`.
- Only `contents: true` gives the full text of each skill (`content`). Without it, the response has only `name` and `digest`. Use them to check that the local skill files are up to date.
- `version` is the version of the skills. It is the same as `skills.version` in the handshake.
- The skill text is the version for a project whose checkout has no spec files: the skills read and write specs through verbs, and the text has no path of the spec folder. The two worktree skills appear only when the workflow config has `worktree` on. When the host sets a skill list, the response has only the skills in the list.

### <a id="bundles"></a>Export and import bundles

`GET /export` gives one bundle with all documents of this repo. `POST /import` writes one bundle into an empty repo in one step. A bundle has one of two versions, and `formatVersion` tells them apart:

| | Version 1 | Version 2 |
| --- | --- | --- |
| How each document is named | By document kind (the `document` object) | By string key (`key`) |
| Who makes it | Older desktop apps | `GET /export` and the current desktop app |
| Does `POST /import` accept it | Yes | Yes |
| Import response | One result for each document | The scope version and the document count |

Version 2:

```json
{
  "formatVersion": 2,
  "scopeVersion": 16,
  "documents": [
    { "key": "changes/add-login/proposal.md", "content": "## Why\n…", "digest": "sha256:<64 hexadecimal characters>" },
    { "key": "config.yaml", "content": "schema: spec-driven\n", "digest": "sha256:<…>" }
  ]
}
```

- `key` is the key of the document in the store, for example `changes/<change>/.meta.yaml`, `changes/<change>/<artifact path>`, `changes/<change>/.evidence.json`, `specs/<capability>/spec.md`, `discussions/<slug>.md`, `config.yaml`, `LANGUAGE.md`, `board/order`, `archive/changes/<archived name>/<document>`, and `archive/discussions/<creation date>-<slug>.md`.
- `digest` is `sha256:` and then the SHA-256 of `content` in lowercase hex.
- `scopeVersion` is the scope version at the time of the export. It is also the `ETag` of the response. An import ignores it, so you can import an export response as it is.
- An export sorts the documents by key. If the repo changes all the time during the export, the Server answers 503 `unavailable`. Try again later.
- A successful import answers `{"scopeVersion":17,"documents":2}`.

Version 1:

```json
{
  "formatVersion": 1,
  "scope": { "project": "acme", "repo": "backend" },
  "projectRevision": 0,
  "documents": [
    { "document": { "type": "changeArtifact", "change": "add-login", "artifact": "proposal.md" }, "content": "## Why\n…", "digest": "sha256:<…>" }
  ]
}
```

- `document.type` is `changeMeta`, `changeArtifact`, `changeEvidence`, `canonicalSpec`, `discussion`, `workflowConfig`, `archivedChange`, `language`, or `boardOrder`. Each kind has the name fields that it needs. The entry point changes the kind into a key and then imports the document.
- `scope` must be the project and repo of the request. If not, the Server answers 409 `refused`.
- A successful import answers `{"projectRevision":17,"documents":[{"document":{…},"outcome":"created"}]}`.

Rules for both versions:

- The target repo must be empty. If not, the Server answers 409 `refused` and writes nothing.
- One commit writes all documents: all of them, or none. The commit carries one `scope-imported` event.
- The Server checks each document: the key must follow a naming rule of the engine, `digest` must be the digest of the content, and a key must not appear twice. A failed check gives 400 `invalid_argument`. The `message` names the first document that is wrong.
- A body that is not JSON gives 400. JSON that is not a bundle gives 422. A `formatVersion` that is not 1 or 2 gives 409 `refused`; the `message` starts with `unsupported bundle format version`.
- A body larger than the import limit gives 413. The host sets the limit, and the default is 32 MiB. The body limit of the host's web framework must not be smaller than this limit.

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
| `modifiedAt` | The time of the last modification of a document of the change, as an RFC 3339 UTC time. It does not appear when the store cannot tell. |
| `review`, `verify` | The state of the review station and of the verify station. See [The station state object](#station-state) below. A list item has no `scope`. |
| `branch` | The branch that the start marker recorded, or the branch from the landing report |
| `landedCommit` | The commit of the landing report |
| `fields` | The free fields. It does not appear when the change has none. |

```json
{ "name": "add-login", "summary": "", "status": "in-progress", "completedTasks": 0, "totalTasks": 2, "startedAt": "2026-10-07", "createdBy": "Demo <demo@example.com>", "created": "2026-10-07", "modifiedAt": "2026-10-07T03:12:45Z", "review": { "state": "inProgress" }, "verify": { "state": "none" }, "branch": "speclink/add-login" }
```

Of these fields, `speclink list --json` prints only `created`, `modifiedAt`, and `fields`. It does not print `review`, `verify`, `branch`, or `landedCommit`.

### `GET /changes/{name}`

Always present: `changeName`, `schemaName`, `isComplete`, `applyRequires`, and `artifacts`. Each artifact has `id`, `outputPath`, and `status`, plus `missingDeps` when it is blocked.

Optional fields:

- `created`, `fromDiscussions`, `deltaCapabilities`, `createdBy`, `createdWith`, `startedAt`, `startedBy`, and `claimedBy`. `created` appears only when the metadata holds both the schema and the creation date.
- `review` and `verify`: the station state objects. A stamped station also has `scope` here.
- `branch`: the branch that the start marker recorded, or the branch from the landing report.
- `landedCommit`, `landedAt`, and `landedBy`: the commit of the landing report, the report time (RFC 3339), and the reporter.
- `fields`: the free fields.

CLI `show` in remote mode and the detail page of the desktop app use these fields.

```json
{ "changeName": "add-login", "schemaName": "spec-driven", "isComplete": false, "applyRequires": ["tasks"], "artifacts": [{ "id": "tasks", "outputPath": "tasks.md", "status": "done" }], "created": "2026-10-07", "createdBy": "Demo <demo@example.com>", "createdWith": "claude", "startedAt": "2026-10-07", "startedBy": "Demo <demo@example.com>", "branch": "speclink/add-login", "landedCommit": "0123456789ab", "landedAt": "2026-10-08T10:10:00+08:00", "landedBy": "Demo <demo@example.com>", "fields": { "ticket": "PROJ-1" } }
```

### <a id="station-state"></a>The station state object

`review` and `verify` have the same shape in a change list item and in one change:

| Field | Content |
| --- | --- |
| `state` | `none` (no ticket and no stamp), `inProgress` (an open ticket), or `stamped`. Treat an unknown value as "no data". |
| `stampedAt`, `stampedBy`, `stampedWith` | The time of the stamp, the person, and the agent. They appear only for a stamped station. |
| `rounds` | The number of ticket rounds before the stamp |
| `tasksTotal` | The total number of tasks at the stamp |
| `commit` | The commit that the stamping client reported. The Server does not check it. |
| `scope` | The `path` and `hash` of each file that the stamp covers. It appears only in the response for one change. |

```json
{ "state": "stamped", "stampedAt": "2026-11-02T14:03:44+08:00", "stampedBy": "Demo <demo@example.com>", "stampedWith": "claude-code", "rounds": 2, "tasksTotal": 5, "commit": "0123456789ab", "scope": [{ "path": "src/login.rs", "hash": "<64 characters>" }] }
```

- The Server reports facts only. A client with a checkout decides whether files changed after the stamp: it compares the fingerprints in `scope` with the working tree, and `tasksTotal` with the current number of tasks.
- When the metadata of the change is corrupt, or when the Server does not report station states yet, the `review` and `verify` keys do not appear. A client treats both cases as "no data".

### Each item of `GET /archived`

Always present: `datedName`, `date`, `name`, `specCount`, `fromDiscussions`.

Optional: `tasksTotal`, `tasksDone`, `createdBy`, `fields`, and the two station outcomes `reviewOutcome` and `verifyOutcome`. An outcome is `none` (the station did not run), `stamped`, or `notPassed` (the archive took an open ticket). If the archived documents hold a ticket, the outcome is `notPassed`, also when the metadata still has a stamp.

### Each item of `GET /discussions`

Always present: `slug`, `topic`, `status`, `rounds`, `created`, `path`, `archived`. The demonstration server also always fills `concluded` (the conclusion has text) and `hold` (the record stays live).

Optional: `createdBy`, `kind` (`improve` for an improve discussion), `fields` (the free fields), and `promotedTo`. `promotedTo` lists the changes that the discussion turned into, in the order of promotion. The Server takes it directly from the discussion record. It does not appear when the discussion has no promotion.

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
- Remote-mode `list` never has this field, because the Server does not know your local checkout. Without worktrees, both modes therefore give the same fields.
- When the field appears, `completedTasks`, `totalTasks`, `status`, and `metaError` of the item come from the copy in the worktree, not from the main checkout.
