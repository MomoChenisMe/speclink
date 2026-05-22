## ADDED Requirements

### Requirement: Artifact kind whitelist SHALL be `proposal`, `design`, `tasks`, `spec`

The CLI SHALL accept exactly four artifact kinds: `proposal`, `design`, `tasks`, and `spec`. Any other kind value SHALL be rejected with error code `artifact.kind_invalid` and exit code 2.

#### Scenario: Invalid kind rejected

- **WHEN** the user runs `speclink new artifact summary --change billing-system --stdin` with input data
- **THEN** the CLI SHALL exit with code 2, emit error code `artifact.kind_invalid`, and SHALL NOT modify the filesystem

#### Scenario: Each valid kind is accepted

- **WHEN** the user runs `speclink new artifact <kind> --change <name>` for `<kind>` in `{proposal, design, tasks, spec}` on an initialized change
- **THEN** the CLI SHALL proceed past kind validation

### Requirement: Artifact filesystem path SHALL be derived from change name and kind

The path of an artifact SHALL be computed by the following rules and the engine SHALL NOT permit path overrides. All paths SHALL be expressed relative to the artifact root `.speclink/`.

| Kind     | Path                                          | Requires `--capability` |
| -------- | --------------------------------------------- | ----------------------- |
| proposal | `changes/<name>/proposal.md`                  | no                      |
| design   | `changes/<name>/design.md`                    | no                      |
| tasks    | `changes/<name>/tasks.md`                     | no                      |
| spec     | `changes/<name>/specs/<capability>/spec.md`   | yes                     |

#### Scenario: `kind=spec` without `--capability` rejected

- **WHEN** the user runs `speclink new artifact spec --change billing-system` without `--capability <id>`
- **THEN** the CLI SHALL exit with code 2 and emit error code `artifact.capability_required`

#### Scenario: `--capability` ignored for non-spec kinds

- **WHEN** the user runs `speclink new artifact proposal --change billing-system --capability foo` and provides stdin content
- **THEN** the CLI SHALL emit a warning `artifact.capability_ignored` in the success envelope and SHALL write the file to `proposal.md` (the `--capability` flag SHALL NOT affect the path)

##### Example: warning shape

```json
{
  "ok": true,
  "data": { "etag": "sha256:..." , "kind": "proposal", "path": "changes/billing-system/proposal.md" },
  "warnings": [
    { "code": "artifact.capability_ignored", "message": "`--capability` is only meaningful when kind=spec" }
  ],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

### Requirement: Capability id grammar SHALL match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` with byte length 1–64

The capability identifier supplied to `--capability` SHALL be validated by the same grammar and length rules as change names.

#### Scenario: Invalid capability id rejected

- **WHEN** the user runs `speclink new artifact spec --change billing-system --capability User_Auth`
- **THEN** the CLI SHALL exit with code 2 and emit error code `artifact.kind_invalid` with hint `invalid capability id`

### Requirement: `speclink artifact read` SHALL return content and an Etag computed from file bytes

The CLI command `speclink artifact read <kind> --change <name> [--capability <cap>]` SHALL read the resolved file from the filesystem, compute `sha256(bytes)` as the Etag, and emit both `content` and `etag` in the success envelope. The Etag value SHALL be the lowercase hexadecimal digest prefixed with the literal string `sha256:`.

#### Scenario: Reading an existing artifact

- **WHEN** the file `.speclink/changes/billing-system/proposal.md` exists with byte content `B`
- **AND** the user runs `speclink artifact read proposal --change billing-system`
- **THEN** the CLI SHALL exit with code 0, emit `data.content` equal to the UTF-8 string of `B`, and emit `data.etag` equal to `sha256:<hex(sha256(B))>`

#### Scenario: Reading a non-existent artifact

- **WHEN** the file does not exist on the filesystem
- **THEN** the CLI SHALL exit with code 2 and emit error code `artifact.not_found`

#### Scenario: Reading from a non-existent change

- **WHEN** the user runs `speclink artifact read proposal --change unknown` and no `change` row with `name='unknown'` exists
- **THEN** the CLI SHALL exit with code 2 and emit error code `change.not_found` (the change-level check SHALL precede any filesystem read)

##### Example: read success envelope

```json
{
  "ok": true,
  "data": {
    "kind": "proposal",
    "capability": null,
    "path": "changes/billing-system/proposal.md",
    "content": "## Why\n\nWe need ...\n",
    "etag": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

### Requirement: `speclink new artifact` SHALL enforce sha256-based optimistic concurrency

The CLI command `speclink new artifact <kind> --change <name> [--capability <cap>] [--expected-etag <etag>] --stdin` SHALL accept artifact bytes from stdin and SHALL apply the following Etag rules. The new Etag SHALL be `sha256:<hex(sha256(new_bytes))>` and SHALL be returned in the success envelope.

#### Scenario: Creating a new artifact without `--expected-etag`

- **WHEN** the user runs the command with `--expected-etag` omitted (null) and the resolved file does not exist
- **THEN** the CLI SHALL exit with code 0, the file SHALL be written with the stdin bytes, and the response SHALL contain the new Etag

#### Scenario: Overwriting an existing artifact with the correct `--expected-etag`

- **WHEN** the file exists with bytes `B0`, the user supplies `--expected-etag sha256:<hex(sha256(B0))>` and stdin `B1`
- **THEN** the CLI SHALL exit with code 0, the file SHALL contain exactly `B1`, and the response SHALL contain `sha256:<hex(sha256(B1))>`

#### Scenario: Overwriting an existing artifact without `--expected-etag`

- **WHEN** the file already exists and the user supplies no `--expected-etag`
- **THEN** the CLI SHALL exit with code 7, emit error code `artifact.version_conflict`, and SHALL NOT modify the file

#### Scenario: `--expected-etag` mismatch

- **WHEN** the file exists with bytes `B0`, the user supplies `--expected-etag sha256:<hex(sha256(B_other))>` where `B_other != B0`, and provides stdin `B1`
- **THEN** the CLI SHALL exit with code 7, emit error code `artifact.version_conflict`, and SHALL NOT modify the file

#### Scenario: `--expected-etag` supplied but file does not exist

- **WHEN** the resolved file does not exist and the user supplies a non-null `--expected-etag`
- **THEN** the CLI SHALL exit with code 2 and emit error code `artifact.not_found`

##### Example: concurrency matrix

| File exists? | `--expected-etag` value                | Outcome                          | Exit | Error code                   |
| ------------ | -------------------------------------- | -------------------------------- | ---- | ---------------------------- |
| no           | (omitted)                              | file written                     | 0    | (none)                       |
| no           | any non-null value                     | refused                          | 2    | `artifact.not_found`         |
| yes          | (omitted)                              | refused                          | 7    | `artifact.version_conflict`  |
| yes          | matches sha256 of current bytes        | file overwritten                 | 0    | (none)                       |
| yes          | non-null but mismatches current sha256 | refused                          | 7    | `artifact.version_conflict`  |

##### Example: write success envelope

```json
{
  "ok": true,
  "data": {
    "kind": "proposal",
    "capability": null,
    "path": "changes/billing-system/proposal.md",
    "etag": "sha256:7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730",
    "bytesWritten": 1024
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

### Requirement: Artifact writes SHALL use a tempfile-then-rename atomic sequence

The engine SHALL write artifact bytes to a temporary file in the same parent directory as the resolved path, then SHALL rename it onto the target path so the operation is atomic on the filesystem layer. If the parent directory does not exist (for example, `specs/<capability>/`), the engine SHALL create it before writing the tempfile. Partial writes SHALL NOT remain on disk after failure.

#### Scenario: Mid-write failure leaves no partial file

- **WHEN** an injected failure aborts the write after the tempfile is created but before the rename
- **THEN** the target path SHALL NOT exist and the tempfile SHALL NOT remain on disk after the operation returns

#### Scenario: Parent directory created for new spec capability

- **WHEN** the user writes `--kind spec --capability user-auth` and `.speclink/changes/<name>/specs/user-auth/` does not yet exist
- **THEN** the engine SHALL create the directory chain and SHALL write the file

### Requirement: All artifact operations SHALL require the change row to exist

For every `artifact.read`, `artifact.write`, and `spec.list-in-change` call, the engine SHALL look up the row in the `change` table by `name` before any filesystem access. If the row does not exist the engine SHALL emit `change.not_found` and exit with code 2.

#### Scenario: Filesystem present but no change row

- **WHEN** the directory `.speclink/changes/orphan/` exists with files inside, but no row with `name='orphan'` exists in the `change` table
- **AND** the user runs any artifact operation against `--change orphan`
- **THEN** the CLI SHALL exit with code 2 and emit error code `change.not_found`

### Requirement: `speclink list --specs --change <name>` SHALL enumerate spec capabilities from the filesystem

The CLI command `speclink list --specs --change <name>` SHALL list immediate sub-directories of `.speclink/changes/<name>/specs/` that contain a file named `spec.md`, and SHALL emit them in `data.capabilities` sorted lexicographically. The command SHALL NOT touch the `change` table beyond the existence check required for all artifact operations.

#### Scenario: No specs present

- **WHEN** the change row exists but the directory `.speclink/changes/<name>/specs/` is absent or empty
- **THEN** the CLI SHALL exit with code 0 and emit `data.capabilities` as an empty array

#### Scenario: Multiple specs sorted lexicographically

- **WHEN** the directory contains `specs/rate-limiting/spec.md` and `specs/user-auth/spec.md`
- **THEN** the CLI SHALL exit with code 0 and emit `data.capabilities` equal to `["rate-limiting", "user-auth"]`

#### Scenario: Sub-directory without spec.md ignored

- **WHEN** the directory contains `specs/incomplete/` without a `spec.md` file
- **THEN** `incomplete` SHALL NOT appear in `data.capabilities`

##### Example: success envelope

```json
{
  "ok": true,
  "data": {
    "change": "billing-system",
    "capabilities": ["rate-limiting", "user-auth"]
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

### Requirement: Error envelope SHALL preserve the standard shape from slice A onward

Every error response from artifact operations SHALL conform to the envelope established by the bootstrap slice with fields `ok=false`, `error.code`, `error.message`, optional `error.hint`, boolean `error.retryable`, optional `error.retry_after_ms`, and `requestId`.

#### Scenario: Version conflict error envelope

- **WHEN** an `artifact.write` call fails with `artifact.version_conflict`
- **THEN** the response envelope SHALL contain `error.code='artifact.version_conflict'`, `error.retryable=true`, and SHALL NOT contain any artifact content
