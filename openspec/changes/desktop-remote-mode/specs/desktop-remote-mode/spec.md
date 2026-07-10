## ADDED Requirements

### Requirement: Remote-section presence selects the remote data backend

When `.speclink.yaml` contains a `remote:` section the desktop app SHALL run in remote mode，sourcing board / document / spec / discussion data from the remote server instead of the local filesystem，routed through the desktop core（`*_at` delegators）via the reused remote client。The connection URL SHALL resolve from `SPECLINK_STORE_URL`（when non-empty）else the section `url`；when both are empty the app SHALL fail loudly rather than silently fall back to file mode。The frontend browse UI SHALL be indistinguishable from file mode。

#### Scenario: remote section drives data from the server

- **WHEN** the desktop opens a project whose `.speclink.yaml` has a `remote:` section with a reachable `url`
- **THEN** the board, documents, specs, and discussions load from the remote server, and the browse UI renders identically to file mode

#### Scenario: environment override wins over the section url

- **WHEN** `SPECLINK_STORE_URL` is set to a non-empty value while a section `url` is also present
- **THEN** requests go to the environment url

#### Scenario: url absent in both places fails loud

- **WHEN** a `remote:` section is present but both the section `url` and `SPECLINK_STORE_URL` are empty
- **THEN** the app surfaces an explicit error naming both settings and does not fall back to file mode

### Requirement: Settings page splits remote-read-only config from local connection

In remote mode the settings page's config tab SHALL show the effective config（from `GET /config`）read-only——the workflow-config write actions SHALL be disabled because remote config is host-managed。The `.speclink.yaml` tab SHALL provide a remote card that writes the server `url` and `repo` into the `.speclink.yaml` `remote:` section（preserving other keys）and stores the PAT in the user-level credential store keyed by the connection origin。The PAT SHALL NOT be written into any file inside the project repo，and `SPECLINK_TOKEN` SHALL take precedence over the stored credential。The remote card SHALL use the project's self-built form controls（no bare native inputs）。

#### Scenario: config tab is read-only in remote mode

- **WHEN** the desktop is in remote mode and the user opens the config tab
- **THEN** the effective config from the server is shown read-only and the workflow-config save actions are disabled

#### Scenario: saving remote settings writes the section and credential

- **WHEN** the user fills the remote card with a `url`, `repo`, and PAT and saves
- **THEN** the `remote:` section of `.speclink.yaml` gains the `url` and `repo`（其他鍵不變），and the PAT lands in the user-level credential store — no new or changed credential file appears inside the project repo

#### Scenario: environment token overrides the stored credential

- **WHEN** `SPECLINK_TOKEN` is set to a non-empty value
- **THEN** the desktop uses it in preference to the stored credential

### Requirement: Remote-mode operations route to endpoints or degrade explicitly

Operations SHALL behave predictably in remote mode。`validate`、`analyze`、`drift` SHALL call the server's computation endpoints（server-computed，not local），with the same report shape as file mode。`archive` SHALL use the archive endpoint and `setTaskDone` the task-done endpoint。Task reordering and bulk task toggles SHALL be applied by rewriting the tasks document through an `If-Match` write——a stale version SHALL surface a conflict rather than overwrite。Change discard（`deleteChange`）SHALL be unsupported and surfaced as such。Board card reordering（`reorderCard`）SHALL NOT write to the server。The archived-changes view SHALL be disabled in remote mode（no listing endpoint in v1）。

#### Scenario: validate is server-computed

- **WHEN** the user runs `validate` in remote mode
- **THEN** the desktop calls the server's validate endpoint and shows the returned report, with the same shape as file mode——no local computation over fetched documents

#### Scenario: task reordering uses optimistic-concurrency write

- **WHEN** the user reorders tasks in remote mode
- **THEN** the desktop reads the tasks document, reorders it, and writes it back with an `If-Match` matching the version it read；a stale version surfaces a conflict rather than overwriting

#### Scenario: discard is unsupported

- **WHEN** the user attempts to discard a change in remote mode
- **THEN** the operation is unavailable and the UI reports it as unsupported, writing nothing to the server

#### Scenario: archived view is disabled

- **WHEN** the desktop is in remote mode
- **THEN** the archived-changes view is disabled and does not attempt a listing request

### Requirement: Live refresh uses a polling baseline with advertised push discovery

In remote mode the desktop's live-sync baseline SHALL be polling，so it stays in sync with any verb-contract server regardless of push transport。The desktop SHALL read the server's advertised `events` field；when the advertised `transport` is `sse` the desktop SHALL open an SSE client and refresh the affected view on an invalidate event；when the field is absent or names a transport the desktop does not implement，the desktop SHALL fall back to polling without error。

#### Scenario: SSE advertised and supported drives instant refresh

- **WHEN** the server advertises `events: { transport: "sse" }` and emits an invalidate event after another user changes a change
- **THEN** the desktop's SSE client receives it and the affected view refreshes without a manual reload

#### Scenario: no advertisement falls back to polling

- **WHEN** the server advertises no `events` field
- **THEN** the desktop keeps its data current by polling, without error

#### Scenario: unsupported transport falls back to polling

- **WHEN** the server advertises a `transport` the desktop does not implement（e.g. `"websocket"`）
- **THEN** the desktop ignores the channel and stays current by polling
