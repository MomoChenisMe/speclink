## ADDED Requirements

### Requirement: Headless server serves the verb contract over PostgreSQL

The server SHALL embed the completed `@speclink/engine`（由 ① speclink-sdk-and-store-seam 補完的 SDK）via `createEngine` and expose all `docs/verb-contract.md` v1 REST endpoints——including the analyze/validate/drift server-side computation endpoints——with PostgreSQL as the store of truth（文件即真相，lifecycle 由文件派生，不另設狀態表）。In remote mode the CLI's human and `--json` output SHALL be byte-identical to fs mode（欄位 camelCase，與 fs payload 對齊）。Clients SHALL ignore response fields they do not recognize（forward compatibility）。

#### Scenario: list changes returns fs-parity fields plus remote additions

- **WHEN** an authenticated `GET /changes` is made
- **THEN** the response is `200` with a `changes[]` array whose items carry `name`/`summary`/`status`/`completedTasks`/`totalTasks`（與 fs `list --json` 同），加上 remote 專屬 `repo`/`lifecycle`/`claimedBy`

#### Scenario: read an existing artifact

- **WHEN** `GET /changes/{name}/artifacts/design` targets an existing artifact
- **THEN** the response is `200` with `{ artifact, content, version }`

#### Scenario: read a missing artifact

- **WHEN** `GET /changes/{name}/artifacts/design` targets an artifact that has not been created
- **THEN** the response is `404` with `reason: "not_found"`

#### Scenario: create a change

- **WHEN** `POST /changes` is made with `{ name }`
- **THEN** the response is `201` with `{ name, schema, repo, lifecycle: "drafting" }`

#### Scenario: CLI remote output matches fs mode

- **WHEN** the CLI in remote mode runs `speclink list --json` against the server
- **THEN** stdout is byte-identical to fs-mode `speclink list --json` for the same set of changes

#### Scenario: server computes analyze on request

- **WHEN** an authenticated request hits the server's analyze endpoint for a change
- **THEN** the server runs the embedded engine over its store and returns the analyze report, so the CLI in remote mode prints output byte-identical to fs-mode `speclink analyze --json`

### Requirement: Request identity, API version negotiation, and repo ownership

Every request SHALL carry `Authorization: Bearer <PAT>`, `X-Speclink-Api-Version: 1`, and（多 repo 專案）`X-Speclink-Repo`。A missing or unsupported API version SHALL be rejected with `400 api_version_unsupported`（body 列 `supportedVersions`）。A multi-repo project request without a repo header SHALL be rejected with `400 repo_required`；a single-repo project SHALL resolve an absent header to the sole registered repo。One change SHALL belong to exactly one repo：`GET /changes` SHALL be filtered by the requesting repo，and a change-scoped verb against another repo's change SHALL fail with `403 repo_mismatch`（body names `changeRepo` and `requestRepo`）。

#### Scenario: missing API version header

- **WHEN** a request arrives without `X-Speclink-Api-Version`
- **THEN** the response is `400` with `reason: "api_version_unsupported"` and a `supportedVersions` field

#### Scenario: listing is filtered by requesting repo

- **WHEN** `GET /changes` is made with `X-Speclink-Repo: frontend`
- **THEN** only changes owned by `frontend` are returned; sibling-repo changes are absent

#### Scenario: cross-repo verb is rejected

- **WHEN** a task-done request targets a change owned by `backend` while `X-Speclink-Repo: frontend`
- **THEN** the response is `403` with `reason: "repo_mismatch"`, `changeRepo: "backend"`, `requestRepo: "frontend"`

### Requirement: Optimistic concurrency on artifact writes

Artifact reads SHALL return a `version`（monotonically increasing integer，starting at `1` on creation）。An artifact `PUT` SHALL require an `If-Match` header：`If-Match: 0` means create-only（already exists → `409 version_conflict` with `currentVersion`）；a stale version → `409 version_conflict`；a `PUT` without `If-Match` → `428 if_match_required`。There SHALL be no force-write and no last-writer-wins mode。

#### Scenario: write with matching version succeeds

- **WHEN** an artifact `PUT` carries `If-Match` equal to the current version
- **THEN** the response is `200` with the artifact's `version` incremented by one

#### Scenario: If-Match matrix

- **WHEN** an artifact `PUT` is made with the given `If-Match` against the given store state
- **THEN** the server responds as tabulated

##### Example: If-Match outcomes

| If-Match | store state | response |
| -------- | ----------- | -------- |
| current version | artifact exists at that version | `200`, version incremented |
| stale version | artifact exists at a newer version | `409 version_conflict` with `currentVersion` |
| `0` | artifact does not exist | `200`, version `1` |
| `0` | artifact already exists | `409 version_conflict` with `currentVersion` |
| absent | any | `428 if_match_required` |

### Requirement: Change lifecycle adjudication

`lifecycle` SHALL be derived from document facts（six states `drafting`/`review`/`ready`/`applying`/`busy`/`archived`）。`claim` SHALL be an atomic compare-and-set：succeeds only when unclaimed and eligible；already claimed → `409 ownership_lost`（`claimedBy`）；gate pending → `409 gate_pending`；not eligible → `409 change_busy`。Writes while `applying` SHALL be owner-only，else `409 ownership_lost`。`archive` SHALL be check-all-then-apply and atomic：unchecked tasks → `409 tasks_incomplete`（`remaining`）；a delta's base spec behind canonical → `409 version_conflict`（`conflicts[]`）；otherwise merge in one transaction with no partial state。Any write verb against an archived change SHALL return `409 change_busy` with `lifecycle: "archived"`。

#### Scenario: concurrent claims resolve to one winner

- **WHEN** two clients `POST /changes/{name}/claim` concurrently on a `ready` change
- **THEN** exactly one gets `200`; the other gets `409` with `reason: "ownership_lost"` and `claimedBy` naming the winner

#### Scenario: non-owner write is rejected

- **WHEN** a change is `applying` and claimed by user A, and user B issues an artifact `PUT`
- **THEN** the response is `409` with `reason: "ownership_lost"`

#### Scenario: archive with open tasks

- **WHEN** `POST /changes/{name}/archive` runs with 3 unchecked tasks
- **THEN** the response is `409` with `reason: "tasks_incomplete"` and `remaining: 3`; no archive occurs

#### Scenario: write against an archived change

- **WHEN** a task-done request targets an archived change
- **THEN** the response is `409` with `reason: "change_busy"` and `lifecycle: "archived"`

### Requirement: Error reason envelope aligned with the contract catalog

Every non-2xx response（except transport-level 5xx where no body is guaranteed）SHALL carry `{ reason, message, ...context }`。`reason` SHALL be the only field clients branch on；every `409` SHALL carry a `reason`。The `reason` values SHALL align with the `docs/verb-contract.md` §4 catalog。A client that cannot reach the server SHALL fail loudly with no cache fallback。

#### Scenario: every 409 carries a reason

- **WHEN** any request produces a `409`
- **THEN** the body contains a `reason` field

#### Scenario: content validation failure

- **WHEN** an artifact `PUT` carries content the engine rejects
- **THEN** the response is `422` with `reason: "validation_failed"` and an `errors[]` list

#### Scenario: unreachable server fails loud

- **WHEN** the server is unreachable or returns 5xx
- **THEN** the client emits a single-line "server unavailable" message, does not retry in a loop, and does not fall back to local data

### Requirement: PAT authentication and admin management

The server SHALL validate the Bearer PAT and derive caller identity；an invalid credential SHALL fail with `401` and one of `token_missing`/`token_invalid`/`token_expired`/`token_revoked`。`GET /whoami` SHALL return the caller's `user`（id/name/handle）and the project's `repos` registry。Contract-external admin REST endpoints SHALL create, list, and revoke tokens, and register and list repos, gated by admin authority。The first admin token SHALL be seeded from a server environment variable at startup。

#### Scenario: whoami with a valid token

- **WHEN** `GET /whoami` is made with a valid PAT
- **THEN** the response is `200` with `user: { id, name, handle }` and a `repos[]` registry

#### Scenario: unknown token rejected

- **WHEN** a request carries an unknown Bearer token
- **THEN** the response is `401` with `reason: "token_invalid"`

#### Scenario: admin issues a token

- **WHEN** the admin calls the admin token-create endpoint with admin authority
- **THEN** a new PAT is returned that authorizes `GET /whoami` for the named identity

#### Scenario: admin bootstrap from environment

- **WHEN** the server starts with the admin token environment variable set
- **THEN** that token authorizes the admin endpoints, and requests without admin authority to those endpoints are rejected

### Requirement: SSE live invalidation via LISTEN/NOTIFY

The server SHALL expose an authenticated, repo-scoped Server-Sent Events endpoint。On a document change the server SHALL emit an invalidate event `{ type: "invalidate", scope }` to connected clients of the affected scope, driven by PostgreSQL `LISTEN`/`NOTIFY`。Events SHALL NOT carry document content——clients re-read through verbs。The server SHALL advertise this channel through the contract's optional `events` field as `{ url, transport: "sse" }`（① 定義的傳輸無關宣告約定），so a transport-agnostic client discovers it rather than assuming a hardcoded path。

#### Scenario: artifact write notifies subscribers

- **WHEN** a change's artifact is written and an SSE client of that repo is connected
- **THEN** the client receives an `invalidate` event whose `scope` identifies the change

#### Scenario: discussion change notifies subscribers

- **WHEN** a discussion round is appended
- **THEN** connected clients of that repo receive an `invalidate` event with `scope: "discussions"`

#### Scenario: unauthenticated SSE connection rejected

- **WHEN** an SSE connection is opened without a valid PAT
- **THEN** the connection is rejected with `401`

#### Scenario: server advertises its SSE channel

- **WHEN** a client reads the server's whoami or config metadata
- **THEN** it carries `events: { url, transport: "sse" }`, letting a transport-agnostic client discover and connect to the SSE channel

### Requirement: One-command Docker deployment

The server SHALL ship a docker-compose definition that starts the server and PostgreSQL together and applies the schema migration on first run。

#### Scenario: fresh deployment is usable end-to-end

- **WHEN** `docker-compose up` runs on a fresh host and an admin token and repo are provisioned
- **THEN** the server listens, the PostgreSQL schema is migrated, and a CLI `speclink link` + `speclink auth login` followed by a full propose → apply → archive flow succeeds against it

### Requirement: Team-mode positioning documentation

`docs/team-mode.md` and the `README` SHALL document the local file mode's fit and limits, state that the "prevent verb bypass" strong guarantee is reachable only through the remote server, and name when to switch to remote mode。

#### Scenario: docs state the mode boundary

- **WHEN** a reader consults `docs/team-mode.md`
- **THEN** it states the file-mode limits, the remote-only strong guarantee, and the criteria for switching to remote mode
