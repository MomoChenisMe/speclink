# speclink

SpecLink — Spec-Driven Development workflow CLI for AI agents.

This repository hosts the Cargo workspace with four crates:

| crate            | role                                                             |
| ---------------- | ---------------------------------------------------------------- |
| `cli`            | `speclink` binary, command surface (clap)                        |
| `runtime`        | workflow orchestration over the `Provider` trait                 |
| `provider`       | `Provider` async trait, shared types, config + resolution        |
| `provider-local` | local filesystem implementation of `Provider` (no remote calls)  |

## Status

Pre-alpha. The local-first AI workflow surface is built up vertically through
sequential changes under `openspec/changes/`. The CLI currently supports
`propose create`, multi-kind `artifact write`, `status`, `archive` (with
spec delta merge), `instructions` (per-kind artifact guidance), and
`task done` (idempotent tasks.md checkbox update) against the local provider.

## CLI usage

All commands accept the machine-interface flags `--json`, `--no-color`,
`--quiet` (and a per-command `--stdin` where documented). Exit codes follow the
table pinned in `cli-machine-interface`: `0` success, `1` general failure,
`2` user-input error, `5` provider unavailable, `6` auth required.

### Create a proposal

```sh
speclink propose create --change add-feature --summary "one-line why" --json
```

Output (one-line JSON envelope, pretty-printed here for clarity):

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "state": "proposed",
    "artifactPath": ".speclink/changes/add-feature/proposal.md",
    "mode": "local"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

### Write design / tasks artifact

```sh
echo "design content" | speclink artifact write design \
  --change add-feature --stdin --json
```

```sh
echo "- [ ] task body" | speclink artifact write tasks \
  --change add-feature --stdin --json
```

The `data` payload contains `artifactId`, `kind`, and the POSIX-style `path`
(`forward slashes on all platforms`).

### Write a spec artifact (per capability)

```sh
cat <<'EOF' | speclink artifact write spec \
  --change add-feature --capability user-auth --stdin --json
## ADDED Requirements

### Requirement: ...
EOF
```

Output:

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "artifactId": "spec:user-auth",
    "kind": "spec",
    "path": ".speclink/changes/add-feature/specs/user-auth/spec.md",
    "mode": "local"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

### Observe change status

```sh
speclink status --change add-feature --json
```

Output:

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "state": "proposed",
    "artifacts": [
      {
        "id": "proposal",
        "kind": "proposal",
        "path": ".speclink/changes/add-feature/proposal.md",
        "status": "done",
        "required": true,
        "dependencies": []
      },
      {
        "id": "design",
        "kind": "design",
        "path": ".speclink/changes/add-feature/design.md",
        "status": "done",
        "required": false,
        "dependencies": ["proposal"]
      },
      {
        "id": "tasks",
        "kind": "tasks",
        "path": ".speclink/changes/add-feature/tasks.md",
        "status": "missing",
        "required": false,
        "dependencies": ["proposal", "spec"]
      },
      {
        "id": "spec:user-auth",
        "kind": "spec",
        "path": ".speclink/changes/add-feature/specs/user-auth/spec.md",
        "status": "done",
        "required": true,
        "dependencies": ["proposal"]
      }
    ]
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

`status` is side-effect-free: it never creates or modifies any files under
`.speclink/`. Spec entries appear after the three fixed-name kinds, sorted
ascending by capability name.

### Archive a completed change

`archive` 將 active change 移到 `.speclink/changes/archive/YYYY-MM-DD-<id>/`、
套用 `specs/<capability>/spec.md` 的 delta 至 `.speclink/specs/<capability>/spec.md`、
更新 metadata 為 `state: "archived"`，並清理 SQLite `in_progress_change` 表。

```sh
speclink archive add-feature --json
```

Output:

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "archivePath": ".speclink/changes/archive/2026-05-19-add-feature",
    "state": "archived",
    "archivedAt": "2026-05-19T12:34:56Z",
    "dryRun": false,
    "specSync": {
      "capabilitiesSynced": [
        {
          "capability": "user-auth",
          "mainSpecPath": ".speclink/specs/user-auth/spec.md",
          "addedCount": 2,
          "modifiedCount": 0,
          "removedCount": 0,
          "renamedCount": 0,
          "createdMainSpec": true
        }
      ]
    }
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

#### Dry-run preview

`--dry-run` 完成 delta merge 運算後立即回傳，不寫檔、不動 SQLite：

```sh
speclink archive add-feature --dry-run --json
```

`data.dryRun` 為 `true`、`data.archivePath` 為「將會寫入」的路徑；`data.specSync`
反映將會套用的 add / modify / remove / rename 計數。Filesystem 無任何變更。

#### Delta conflict

當 delta 與既有主 spec 衝突（如 ADDED 已存在、MODIFIED 找不到對應 requirement），
archive 回 exit code `7` 與 `error.code = "spec.delta_conflict"`：

```json
{
  "ok": false,
  "data": null,
  "warnings": [],
  "error": {
    "code": "spec.delta_conflict",
    "message": "provider error: spec delta conflict for capability 'user-auth': requirement 'User login' (ADDED)",
    "details": {}
  },
  "requestId": "req_..."
}
```

Dry-run 同樣會回 exit `7` 並回相同 `error.code` — 不寫檔即可發現衝突。

### Get artifact instructions

`instructions` 對 4 種 artifact kind 各回一份 instruction / template / rules，供 AI agent
產生內容時參考。`spec` kind 需附帶 `--capability`，其他三種 kind（`proposal` / `design` /
`tasks`）不可帶 `--capability`。指令完全 side-effect-free。

```sh
speclink instructions design --change add-feature --json
```

Output:

```json
{
  "ok": true,
  "data": {
    "artifactId": "design",
    "kind": "design",
    "outputPath": ".speclink/changes/add-feature/design.md",
    "dependencies": ["proposal"],
    "unlocks": ["tasks"],
    "instruction": "撰寫 `design.md`：說明「HOW」— ...",
    "template": "## Context\n\n...",
    "rules": [
      {
        "code": "design.must_include_context",
        "level": "error",
        "description": "Design SHALL 包含 `## Context` heading..."
      }
    ],
    "locale": "Traditional Chinese (繁體中文)"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

Spec kind 需 `--capability`：

```sh
speclink instructions spec --change add-feature --capability user-auth --json
```

`data.artifactId` 為 `"spec:user-auth"`、`data.outputPath` 為
`.speclink/changes/add-feature/specs/user-auth/spec.md`。

### Mark a task as done

`task done` 將 `tasks.md` 中對應 `N.M` checkbox 從 `- [ ]` 改為 `- [x]`，原子寫入。
idempotent：對已完成的 task 再次呼叫不視為錯誤（`previousStatus = "done"`）。

```sh
speclink task done 5.2 --change add-feature --json
```

Output:

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "taskId": "5.2",
    "previousStatus": "todo",
    "currentStatus": "done",
    "taskDescription": "Wire up SSO endpoint"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

`[P]` parallel marker 會保留於 `taskDescription` 內：對 `- [ ] 2.3 [P] Refactor parser`
呼叫 `task done 2.3` 後，`data.taskDescription = "[P] Refactor parser"`、tasks.md 行
為 `- [x] 2.3 [P] Refactor parser`。

失敗對應：

| 觸發條件 | error code | exit code |
| --- | --- | --- |
| change 不存在 | `change.not_found` | 1 |
| tasks.md 不存在 | `artifact.missing` | 1 |
| task id 格式不符（如三層 `1.1.2`） | `task.invalid_id` | 2 |
| task id 在 tasks.md 中找不到 | `task.not_found` | 2 |
| tasks.md 解析失敗 | `tasks.parse_error` | 1 |

## License

Dual-licensed under MIT OR Apache-2.0. See `LICENSE` for details.
