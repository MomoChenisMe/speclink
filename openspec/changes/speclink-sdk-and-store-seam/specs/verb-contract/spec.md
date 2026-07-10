## ADDED Requirements

### Requirement: 遠端模式 analyze/validate/drift 由 server 端運算

In remote mode，`analyze`、`validate`、`drift` SHALL be computed server-side：the contract SHALL expose a read-only endpoint per verb，and the CLI in remote mode SHALL call the endpoint and print its report rather than computing locally。The human and `--json` output SHALL stay byte-identical to fs mode。In fs mode these verbs SHALL keep running locally（unchanged）。The rationale is team consistency——a server-pinned computation keeps every client's report identical for the same server data, independent of each client's engine version。

#### Scenario: remote analyze calls the server endpoint

- **WHEN** the CLI in remote mode runs `speclink analyze <change> --json`
- **THEN** it requests the server's analyze endpoint and prints the returned report；stdout is byte-identical to fs-mode `speclink analyze --json` for the same documents

#### Scenario: server-pinned result is client-version independent

- **WHEN** two clients on different engine versions run `speclink validate <change>` against the same remote server
- **THEN** both receive the server-computed result — the outcome does not diverge by client engine version

#### Scenario: fs mode still computes locally

- **WHEN** `speclink analyze <change>` runs in fs mode（no `remote:` section）
- **THEN** the engine computes locally over the local store, unchanged

### Requirement: 可選推播通道宣告

Server metadata（whoami/config）SHALL support an optional `events` object `{ url, transport }` that advertises a live-update channel。Push itself is outside the request/response contract；this field exists only for a client to discover a server's channel。The `transport` value SHALL be an open string（e.g. `"sse"`、`"websocket"`），so servers are not bound to one transport。When the field is absent，a client SHALL treat the server as having no push channel and SHALL NOT raise an error。When the advertised `transport` is one the client does not support，the client SHALL ignore it and fall back to its baseline sync。

#### Scenario: server advertises an SSE channel

- **WHEN** a client reads server metadata carrying `events: { url: "/events", transport: "sse" }`
- **THEN** the client discovers the channel and connects if it supports `sse`

#### Scenario: no advertisement means no push, not an error

- **WHEN** server metadata carries no `events` field
- **THEN** the client treats the server as having no push channel and continues on its baseline sync without error

#### Scenario: unsupported transport falls back

- **WHEN** the advertised `transport` is one the client does not implement（e.g. `"websocket"` on a client that only speaks `sse`）
- **THEN** the client ignores the channel and falls back to its baseline sync
