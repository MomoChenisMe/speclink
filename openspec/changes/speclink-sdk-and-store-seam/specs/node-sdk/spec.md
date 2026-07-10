## ADDED Requirements

### Requirement: dispatch covers the remote-hostable verb set

`engine.dispatch` SHALL route the verbs a remote server needs to serve the `docs/verb-contract.md` contract, beyond the existing `list`/`status`/`new`/`claim`：`archive`、`task done`、`artifact cat`、`instructions`、`language`、`config`、canonical `spec` read，and the full `discuss` verb set（`new`/`context`/`add-round`/`conclude`/`archive`/`promote`/`list`/`show`）。Each routed verb SHALL return a structured object aligned with the fs-mode `--json` payload for that verb（camelCase 欄位），and SHALL back its reads and writes through the host `Store`。A verb whose business logic already exists in the engine SHALL be exposed by routing，SHALL NOT be re-implemented，and SHALL NOT change its observable semantics。

#### Scenario: dispatch task done returns the fs-parity payload

- **WHEN** `engine.dispatch(['task', 'done', '--change', 'demo', '3'])` runs against a host store where task 3 is open
- **THEN** it resolves to an object carrying the same fields the fs-mode `speclink task done --json` produces（含 change / status / task id / task description），and the host store received the tasks-document write

#### Scenario: dispatch archive drives the engine merge

- **WHEN** `engine.dispatch(['archive', 'demo'])` runs against a host store whose change is complete
- **THEN** the engine performs the delta-into-canonical merge through the host store（正典寫入與封存移動），and resolves to the archive result payload

#### Scenario: dispatch discuss add-round appends through the store

- **WHEN** `engine.dispatch(['discuss', 'add-round', 'some-slug', '--mode', 'assumptions', '--stdin'], { stdin })` runs
- **THEN** the round is appended to the discussion document through the host store, and the result reports the new round number

#### Scenario: unknown verb still rejects

- **WHEN** `engine.dispatch` is called with a verb outside the routed set
- **THEN** it rejects with an `Error` whose `code` marks the invalid argv, unchanged from prior behavior

### Requirement: Engine computes analyze/validate/drift over a host store

`engine.dispatch` SHALL compute `analyze`、`validate`、`drift` against a host `Store`'s documents and return each verb's report payload，so a server embedding the SDK can serve these as server-side computation。The report payload SHALL align with the fs-mode `--json` shape for that verb。These verbs SHALL be read-only（讀 host store 文件、不寫）。

#### Scenario: dispatch analyze returns the four-dimension report

- **WHEN** `engine.dispatch(['analyze', 'demo', '--json'])` runs against a host store holding the change's artifacts
- **THEN** it resolves to the analyze report with the Coverage / Consistency / Ambiguity / Gaps dimensions，identical in shape to fs-mode `speclink analyze --json`

#### Scenario: dispatch validate returns pass or errors

- **WHEN** `engine.dispatch(['validate', 'demo'])` runs against a host store whose delta specs are well-formed
- **THEN** it resolves to a valid result；a malformed delta instead yields the same validation-error payload fs mode produces

#### Scenario: compute verbs do not mutate the store

- **WHEN** any of `analyze`/`validate`/`drift` runs against a host store
- **THEN** the host store receives only reads — no artifact, spec, or meta write occurs
