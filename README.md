# speclink

SpecLink — Spec-Driven Development workflow CLI for AI agents.

This repository hosts the Cargo workspace with four crates:

| crate            | role                                                             |
| ---------------- | ---------------------------------------------------------------- |
| `cli`            | `speclink` binary, command surface (clap)                        |
| `runtime`        | workflow orchestration over the `Provider` trait                 |
| `provider`       | `Provider` async trait, shared types, config + resolution        |
| `provider-local` | local filesystem implementation of `Provider` (no remote calls)  |

## Quick start

The `add-project-bootstrap` change ships four CLI subcommands. Run them inside
a git working tree:

```bash
cd /path/to/your/project
git init                         # SpecLink rejects non-git working dirs
speclink init                    # creates .speclink/ and .git/speclink/
speclink status --json           # reports project_id, provider, roots, git_head
speclink link <project_id>       # rebind to an existing project row in state.db
speclink unlink                  # remove .speclink/link.yaml (keeps state.db)
```

Layout written by `speclink init`:

```
<repo>/.speclink/                # artifact root (git-tracked)
  link.yaml                      # binding metadata (.gitignored)
  schemas/                       # reserved for future capability changes
<repo>/.git/speclink/            # state root (not git-tracked, shared across worktrees)
  state.db                       # SQLite WAL, project rows + _migrations
  locks/                         # reserved for future capability changes
```

All commands accept `--json` and emit a stable envelope (`ok` / `data` /
`warnings` / `requestId`, or `ok: false` / `error` / `requestId`).

## Status

Pre-alpha. The local-first AI workflow surface is built up vertically through
sequential changes under `openspec/changes/`. As of slice A4 (`add-archive`)
the CLI supports project bootstrap, change / artifact I/O, the 6-state
lifecycle (`proposing → reviewing → ready ⇌ in_progress → code_reviewing →
archived`), apply / task ops, and end-to-end `archive` which closes the
walking-skeleton 4-state main path (`proposing → ready → in_progress →
archived`). Future slices add review, locking, schema management, config-rw,
discuss, and skill deployment.

## Walking skeleton 3 — state machine + apply / task

```bash
# 寫滿 DAG (proposal + tasks + 至少一份 spec) 後，artifact.write hook 會自動把
# state 從 proposing 推到 ready（walking-skeleton 4-state mode，review flags
# 硬編 false）。
speclink new artifact proposal --change demo --stdin < proposal.md
speclink new artifact spec --change demo --capability auth --stdin < spec.md
speclink new artifact tasks --change demo --stdin < tasks.md
# → warnings 含 { code: "state_transitioned", details: { from: "proposing", to: "ready" } }

# 進 apply：把 change 推進到 in_progress，actor 推導 fallback chain 為
# --actor flag → SPECLINK_AGENT_HOST env → 預設 "cli"。
speclink apply start demo --actor claude-code

# 依 1-based 行內順序逐 task 完成；最後一個 task done 後 walking-skeleton 設
# all_tasks_done=1 並保留 state=in_progress（review slice 上線後改為 code_reviewing）。
speclink task list --change demo
speclink task done 1 --change demo
speclink task done 2 --change demo
# → { all_tasks_done: true, state: "in_progress", auto_transitioned: false }

# 隨時可 pause 退回 ready 並清空 actor；再次 apply start 為 idempotent 且 reassign actor。
speclink apply pause demo
speclink apply start demo --actor cursor

# task indices 依當前 tasks.md 順序決定；在 task done 期間禁止改 tasks.md。
# undo 把 [x] → [ ]；若 change 在 code_reviewing state，會先 transition 回 in_progress。
speclink task undo 2 --change demo
```

## Walking skeleton 4 — archive（A4 = `add-archive`）

A4 接通 `in_progress + all_tasks_done=1 → archived` transition、把 change 內
`specs/<capability>/spec.md` delta merge 進 `.speclink/specs/<capability>/spec.md`、
把 change 目錄搬到 `.speclink/changes/archive/<YYYY-MM-DD>-<id>/`。walking-skeleton
mode（A3 既有硬編 `require_*_review=false`）下，user 從此可端到端跑完整個 SDD cycle：

```bash
# 接 A3 happy path 之後（state=in_progress, all_tasks_done=1）：
speclink archive demo
# → state.db: state=archived + archived_at=<UTC>
# → fs: .speclink/changes/demo/ 消失、.speclink/changes/archive/<date>-demo/ 出現
# → fs: .speclink/specs/<capability>/spec.md 整檔覆蓋寫入
# → state_transition 表多一筆 reason='archive_run'

# Emergency 路徑：跳過 spec merge（archive.specs_skipped warning carrier）：
speclink archive demo --skip-specs

# `--no-validate` flag 已 parse 但本 slice no-op（reserved for add-analyze slice）；
# `--yes` flag 已 parse 但本 slice 不 prompt（reserved for compatibility）。

# Archive 之後對該 change 再呼叫 apply/task 都會回終態 hint 或 reject：
speclink apply start demo    # exit 0, data.message="Change is archived."
speclink task done 1 --change demo  # exit 7, state.transition_invalid
```

## Walking skeleton 2 — change & artifact

Slice A adds 7 operations on top of the bootstrap surface. They form a
walking skeleton: build a change, write artifacts into it, read them back.

| Operation | Command |
| --------- | ------- |
| `change.create` | `speclink new change <name>` |
| `change.list` | `speclink list --changes` |
| `change.show` | `speclink show change <name>` |
| `change.delete` | `speclink delete change <name> --confirm-name <name>` |
| `artifact.write` | `speclink new artifact <kind> --change <name> [--capability <id>] [--expected-etag <etag>] --stdin` |
| `artifact.read` | `speclink artifact read <kind> --change <name> [--capability <id>]` |
| `spec.list-in-change` | `speclink list --specs --change <name>` |

Artifact `<kind>` is one of `proposal`, `design`, `tasks`, `spec`. `--capability`
is required for `kind=spec`. Etags are sha256 of file bytes
(`sha256:<64 lowercase hex>`) and the engine enforces optimistic concurrency:
new files must omit `--expected-etag`, overwrites must supply the current etag.

End-to-end demo (run inside a fresh git working tree):

```bash
git init                                                      # bootstrap requires git
speclink init                                                  # create .speclink/ + .git/speclink/
speclink --json new change billing-system                      # row inserted; .speclink/changes/billing-system/ scaffolded
printf '## Why\n\nWe need...\n' | \
  speclink --json new artifact proposal --change billing-system --stdin
speclink --json artifact read proposal --change billing-system # echo content + sha256 etag
speclink --json list --changes                                 # sorted desc by updated_at
speclink --json show change billing-system                     # row metadata + artifact list
```

A version-conflict example:

```bash
ETAG=$(speclink --json artifact read proposal --change billing-system \
  | python3 -c "import sys, json; print(json.load(sys.stdin)['data']['etag'])")
printf 'new body\n' | speclink --json new artifact proposal \
  --change billing-system --expected-etag "$ETAG" --stdin   # OK
printf 'stale body\n' | speclink --json new artifact proposal \
  --change billing-system --stdin                            # exit 7, artifact.version_conflict
```

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

## Walking skeleton 5 — describe-tools（P1 = `add-tool-describe-and-catalogue`）

第一條 Phase 1 dogfooding slice（design.md §18.4）。Catalogue 把 `doc/protocol/operations.md` 的 37 個 operation metadata 固化為 Rust single source of truth，並提供 `speclink describe-tools` 多 format 印出。

### Describe-tools usage

```bash
# 預設：印 12 個 curated subset（design.md §22.2）為 JSON
speclink describe-tools --json

# 完整 37 個 op
speclink describe-tools --full --json

# Markdown 表（human debug）
speclink describe-tools --format text

# CopilotKit SDK defineTool 形狀
speclink describe-tools --format copilot-sdk --json

# 過濾：category + filter 取 AND intersection
speclink describe-tools --full --categories change --filter change.delete --json

# 多 phase（comma-separated）
speclink describe-tools --full --phases discuss,apply --json
```

### Format flag

| `--format` | 行為 | 狀態 |
| --- | --- | --- |
| `json` (default) | array of `{ id, name, description, parameters }` | MVP ✓ |
| `text` | Markdown table | MVP ✓ |
| `copilot-sdk` | array of `{ name, description, parameters }` 對應 `defineTool` | MVP ✓ |
| `copilotkit` / `openai` / `langchain` / `mcp` / `claude` | clap 接受、runtime 拒絕並回 `tool.format_not_supported` | [deferred] |

### Read-only contract

`describe-tools` 不讀 `.speclink/`、不讀 state.db、不取 lock；任何工作目錄都能跑（無需 `speclink init`）。

### Exit codes

| 觸發條件 | error code | exit code |
| --- | --- | --- |
| `--format` 收到 deferred 5 種之一 | `tool.format_not_supported` | 2 |
| `--filter` 含未知 op id | `tool.unknown_op` | 2 |
| `--categories` 含未知 category | `tool.unknown_category` | 2 |
| `--format` 收到 enum 以外值 | clap parser 拒絕 | 2 |

## License

Dual-licensed under MIT OR Apache-2.0. See `LICENSE` for details.
