# SpecLink Operation Catalogue

> **狀態**：MVP draft，37 ops 完整。
> **角色**：Single source of truth；所有 surface（CLI / Tool binding / SDK method / HTTP endpoint / Provider method）皆 derive from 本檔。
> **Cross-references**：`doc/speclink-design.md` §21（catalogue 角色與映射規則）、§17.4（error code）、§17.6（audit events）、§6.2（lifecycle）、§12（並發與 lock）。

---

## Conventions

### Schema dialect

- 所有 `inputs` / `outputs` 採 **JSON Schema Draft 2020-12** 表示
- 預設 `additionalProperties: false`（嚴格）；放寬時各 op 內 explicit 標示
- 時間欄位一律 ISO 8601 with timezone（`format: "date-time"`）

### Etag 與並發

- 所有 read 類 method 回傳 `etag: string`
- 所有 read-modify-write method 接受 optional `expected_etag`（`None` = blind overwrite；`Some` = If-Match 語意）
- 寫入回傳採 `Versioned<T>` shape：`{ value: T, etag: string }`
- 詳細語意見 design.md §12.8（Lock vs Etag）

### 命名 derivation

各 surface 對應規則見 design.md §21.4：

- **Catalogue ID**：`<noun>.<verb>` snake_case（如 `change.create`）
- **CLI**：`speclink <noun> <verb>` 或動詞前置例外 `speclink new <noun>`（catalogue 標 `cli_verb_first: true`）
- **Tool binding**：`<verb>_<noun>` 或 `<noun>_<verb>` snake_case
- **SDK method**：`speclink.<noun_plural>.<verb>()` camelCase namespace
- **HTTP default endpoint**：`<METHOD> /api/projects/{project_id}/<noun_plural>[/{id}][/<verb>]`

### MVP 與 Destructive 標記

- **MVP** 欄：`✓` = MVP 必做；`[deferred]` = post-MVP
- **Destructive** 欄：`⚠` = 標 `destructive: true`，CLI 需 `--force`、SDK 走 alias 時觸發 audit event `skill.alias_applied`（design.md §17.6）

### Retry 語意

每個 op 的 errors 表標註 retry 行為（design.md §17.4 完整定義）：

- `backoff` — jittered exponential retry（design.md §12.5.2）
- `read-then-retry` — 重讀 etag 後重試一次（design.md §12.5.4）
- `read-then-bail` — 重讀後仍衝突即放棄、bubble up
- `no` — 永不重試（validation / 語意錯誤等）

### Audit events

每個 state-mutating op 必發 audit event，與 state mutation **同一 transaction** commit（design.md §17.6）。

---

## Index

| # | Catalogue ID | Category | CLI | Tool binding | MVP | Idempotency | Lock | Destructive |
|---|---|---|---|---|---|---|---|---|
| 1 | `project.init` | project | `init <name>` | `project_init` | ✓ | non-idemp. | global-short | — |
| 2 | `project.link` | project | `link <url>` | `project_link` | [deferred] | idemp. | none | — |
| 3 | `project.unlink` | project | `unlink` | `project_unlink` | [deferred] | idemp. | none | — |
| 4 | `project.status` | project | `status` | `project_status` | ✓ | idemp. | none | — |
| 5 | `config.read` | config | `config show` | `read_config` | ✓ | idemp. | none | — |
| 6 | `config.write` | config | `config set` / `config edit` | `write_config` | ✓ | idemp.-with-version | global-short | — |
| 7 | `schema.list` | schema | `schemas` | `list_schemas` | ✓ | idemp. | none | — |
| 8 | `schema.show` | schema | `schema show <id>` | `show_schema` | ✓ | idemp. | none | — |
| 9 | `schema.fork` | schema | `schema fork` | `fork_schema` | ✓ | non-idemp. | global-short | — |
| 10 | `schema.delete` | schema | `schema delete <id>` | `delete_schema` | ✓ | non-idemp. | global-short | ⚠ |
| 11 | `discuss.new` | discuss | `discuss new "<topic>"` | `new_discussion` | ✓ | non-idemp. | discuss-excl. | — |
| 12 | `discuss.list` | discuss | `discuss list` | `list_discussions` | ✓ | idemp. | none | — |
| 13 | `discuss.show` | discuss | `discuss show <id>` | `show_discussion` | ✓ | idemp. | none | — |
| 14 | `discuss.patch` | discuss | `discuss patch <id>` | `patch_discussion` | ✓ | idemp.-with-version | discuss-excl. | — |
| 15 | `discuss.conclude` | discuss | `discuss conclude <id>` | `conclude_discussion` | ✓ | non-idemp. | discuss-excl. | — |
| 16 | `discuss.delete` | discuss | `discuss delete <id>` | `delete_discussion` | ✓ | non-idemp. | discuss-excl. | ⚠ |
| 17 | `change.create` | change | `new change <name>` | `new_change` | ✓ (slice-A) | non-idemp. | change-excl. | — |
| 18 | `change.list` | change | `list --changes` | `list_changes` | ✓ (slice-A) | idemp. | none | — |
| 19 | `change.show` | change | `show change <id>` | `show_change` | ✓ (slice-A) | idemp. | none | — |
| 20 | `change.delete` | change | `delete change <id>` | `delete_change` | ✓ (slice-A) | non-idemp. | change-excl. | ⚠ |
| 21 | `artifact.write` | artifact | `new artifact <kind>` | `write_artifact` | ✓ (slice-A) | idemp.-with-version | change-excl. | — |
| 22 | `artifact.read` | artifact | `artifact read <kind>` | `read_artifact` | ✓ (slice-A) | idemp. | none | — |
| 23 | `apply.start` | apply | `apply start <id>` | `apply_start` | ✓ | idemp. | change-excl. | — |
| 24 | `apply.pause` | apply | `apply pause <id>` | `apply_pause` | ✓ | idemp. | change-excl. | — |
| 25 | `task.done` | apply | `task done <task-id>` | `task_done` | ✓ | idemp. | change-excl. | — |
| 26 | `review.approve` | review | `review approve` | `review_approve` | ✓ | non-idemp. | change-excl. | — |
| 27 | `review.reject` | review | `review reject` | `review_reject` | ✓ | non-idemp. | change-excl. | — |
| 28 | `review.history` | review | `review history` | `review_history` | ✓ | idemp. | none | — |
| 29 | `archive.run` | archive | `archive <id>` | `archive_change` | ✓ | non-idemp. | change-excl. + global-short | — |
| 30 | `spec.list` | spec | `list --specs` | `list_specs` | ✓ | idemp. | none | — |
| 31 | `spec.show` | spec | `show spec <cap>` | `show_spec` | ✓ | idemp. | none | — |
| 32 | `instructions.get` | meta | `instructions <kind>` | `get_instructions` | ✓ | idemp. | none | — |
| 33 | `analyze.run` | meta | `analyze <id>` | `analyze_change` | ✓ | idemp. | none | — |
| 34 | `validate.run` | meta | `validate <id>` | `validate_change` | ✓ | idemp. | none | — |
| 35 | `drift.run` | meta | `drift <id>` | `drift_change` | ✓ | idemp. | none | — |
| 36 | `doctor.run` | doctor | `doctor` | `run_doctor` | ✓ | idemp. | none | — |
| 37 | `tool.describe` | tool | `describe-tools` | n/a | ✓ | idemp. | none | — |

---

## Operations

### `change.create`

> 建立新 change，進入 `proposing` state。

| 屬性 | 值 |
|---|---|
| **Category** | change |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | non-idempotent |
| **Lock** | change-exclusive（取得於新 change id） |
| **Provider method** | `Provider::create_change(CreateChangeRequest) -> Versioned<Change>` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["name", "description"],
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z0-9][a-z0-9-]*[a-z0-9]$",
      "minLength": 3,
      "maxLength": 64,
      "description": "Change 的 kebab-case 識別碼；project 內唯一。"
    },
    "description": {
      "type": "string",
      "minLength": 10,
      "maxLength": 500,
      "description": "一行摘要，描述此 change 要達成的目標。"
    },
    "schema_id": {
      "type": ["string", "null"],
      "default": null,
      "description": "Override 當前 active schema。預設使用 project 設定的 active schema。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "additionalProperties": false,
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "name", "state", "created_at", "schema_id"],
      "properties": {
        "change_id": { "type": "string", "description": "等同於 input name。" },
        "name": { "type": "string" },
        "state": { "const": "proposing" },
        "created_at": { "type": "string", "format": "date-time" },
        "schema_id": { "type": "string", "description": "實際採用的 schema。" }
      }
    },
    "etag": { "type": "string", "description": "Opaque version token；後續 RMW 操作的 `expected_etag` 來源。" }
  }
}
```

#### Bindings

- **CLI**：`speclink new change <name> --description "<text>" [--schema <id>] [--json]`
- **Tool**：`new_change({ name, description, schema_id? })`
- **SDK**：`speclink.changes.create({ name, description, schema_id? }): Promise<Versioned<Change>>`
- **HTTP**：`POST /api/projects/{project_id}/changes` — body = inputs, response = outputs

#### Semantics

1. 取得 `change-exclusive` lock on `<name>`（design.md §12.2.2）
2. 若同名 change 已存在 → `change.duplicate_id`，拋出
3. Validate inputs schema → validation 失敗 → `validation.*`
4. 依 active schema（或 `schema_id` override）建立初始空 artifact 檔（`proposal.md` / `spec.md` / `tasks.md` / ...）
5. Insert state row `state = 'proposing'`、生成 monotonic `etag`
6. 寫入 audit event `change.created`（**同 transaction**）
7. 釋放 lock
8. 回傳 `Versioned<Change>`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.duplicate_id` | 同名 change 已存在 | `no` |
| `validation.name_invalid` | name 不符 pattern / 長度規則 | `no` |
| `validation.description_too_short` | description 少於 10 字 | `no` |
| `schema.not_found` | 指定的 `schema_id` 不存在 | `no` |
| `lock.not_acquired` | retry budget 內無法取得 lock | `backoff` |
| `state.etag_mismatch` | Project-level 並發寫入（罕見） | `read-then-retry` |

#### Audit events

- `change.created` — 與 state row insert 同 transaction

#### Examples

**成功**：

```bash
$ speclink new change add-dark-mode --description "Add dark mode toggle to settings page" --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "name": "add-dark-mode",
      "state": "proposing",
      "created_at": "2026-05-21T14:32:11Z",
      "schema_id": "default"
    },
    "etag": "v1.0"
  }
}
```

**重複**：

```bash
$ speclink new change add-dark-mode --description "..." --json
{
  "ok": false,
  "error": {
    "code": "change.duplicate_id",
    "message": "Change 'add-dark-mode' already exists.",
    "hint": "Run `speclink show change add-dark-mode` to inspect existing change.",
    "retryable": false
  }
}
$ echo $?
2
```

#### Cross-references

- design.md §6.2 — change 6-state lifecycle
- design.md §7 — artifact DAG（初始檔生成規則）
- design.md §10 — schema 抽象
- design.md §12.2 — lock hierarchy

---

### `artifact.write`

> 寫入或覆寫 change 內單一 artifact。

| 屬性 | 值 |
|---|---|
| **Category** | artifact |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent-with-version（透過 `expected_etag` 保證） |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::write_artifact(WriteArtifactRequest) -> Versioned<Artifact>` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["change_id", "kind", "content"],
  "additionalProperties": false,
  "properties": {
    "change_id": {
      "type": "string",
      "description": "目標 change 的 id。"
    },
    "kind": {
      "type": "string",
      "description": "Artifact 種類，依 active schema 動態決定（如 `proposal` / `spec` / `design` / `tasks`）。"
    },
    "capability": {
      "type": ["string", "null"],
      "default": null,
      "description": "若 artifact 為 spec.md，此為對應 capability slug；其他 kind 預設 null。"
    },
    "content": {
      "type": "string",
      "description": "Artifact 完整 markdown 內文（UTF-8）。"
    },
    "overwrite": {
      "type": "boolean",
      "default": false,
      "description": "是否允許覆寫既有 artifact body。false 且 artifact 已存在 → `artifact.already_exists`。"
    },
    "expected_etag": {
      "type": ["string", "null"],
      "default": null,
      "description": "Optimistic concurrency token。`null` = blind overwrite；`string` = If-Match 比對失敗回 `state.etag_mismatch`。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "kind", "size_bytes", "updated_at"],
      "properties": {
        "change_id": { "type": "string" },
        "kind": { "type": "string" },
        "capability": { "type": ["string", "null"] },
        "size_bytes": { "type": "integer", "minimum": 0 },
        "updated_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink new artifact <kind> --change <id> [--capability <name>] --stdin [--overwrite] [--json]`
  - Content 從 stdin 讀取，避免 shell argv 長度限制
- **Tool**：`write_artifact({ change_id, kind, capability?, content, overwrite?, expected_etag? })`
- **SDK**：`speclink.artifacts.write({ change_id, kind, capability?, content, overwrite?, expectedEtag? }): Promise<Versioned<Artifact>>`
- **HTTP**：`PUT /api/projects/{project_id}/changes/{change_id}/artifacts/{kind}` — body = inputs（不含 change_id/kind），header `If-Match: <expected_etag>`，response = outputs

#### Semantics

1. 取得 `change-exclusive` lock on `change_id`
2. 檢查 change 是否存在 → `change.not_found`
3. 檢查 change state 是否允許寫入此 artifact（依 schema 與 state；如 `archived` 時拒絕 → `state.transition_invalid`）
4. 檢查 kind 是否屬於當前 schema 的 artifact set → `artifact.unknown_kind`
5. 若 artifact 已存在且 `overwrite = false` → `artifact.already_exists`
6. 若 `expected_etag` 非 null：
   - 讀取當前 artifact etag，不符 → `state.etag_mismatch`（retry: `read-then-retry`）
7. Validate content 大小（MVP 上限 1MB）→ `artifact.too_large`
8. 寫入 artifact body 到 provider storage
9. 更新 etag（monotonic increment）
10. 寫 audit event `artifact.written`（同 transaction）
11. 釋放 lock
12. 回傳 `Versioned<Artifact>`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `state.transition_invalid` | Change 已 archived 等不可寫狀態 | `no` |
| `artifact.unknown_kind` | kind 不屬於當前 schema | `no` |
| `artifact.already_exists` | 已存在且 `overwrite=false` | `no` |
| `artifact.too_large` | content 超過 1MB | `no` |
| `state.etag_mismatch` | `expected_etag` 與當前不符 | `read-then-retry` |
| `lock.not_acquired` | retry budget 內無法取得 lock | `backoff` |

#### Audit events

- `artifact.written` — 含 `kind` / `capability` / `size_bytes` / 是否 overwrite

#### Examples

**首次寫入**：

```bash
$ cat proposal.md | speclink new artifact proposal --change add-dark-mode --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "kind": "proposal",
      "capability": null,
      "size_bytes": 1842,
      "updated_at": "2026-05-21T14:35:02Z"
    },
    "etag": "v2.1"
  }
}
```

**Etag 衝突**（被別的 agent 先寫）：

```bash
$ cat updated.md | speclink new artifact proposal --change add-dark-mode --overwrite --expected-etag v2.1 --json
{
  "ok": false,
  "error": {
    "code": "state.etag_mismatch",
    "message": "Artifact 'proposal' was modified concurrently. Expected v2.1, found v2.3.",
    "hint": "Re-read the latest artifact and re-apply your changes.",
    "retryable": true,
    "current_etag": "v2.3"
  }
}
```

#### Cross-references

- design.md §7 — artifact DAG
- design.md §10 — schema-driven artifact list
- design.md §12.5.4 — etag_mismatch retry semantics
- design.md §16.6 — `--overwrite` vs `--force` 拆分

---

### `task.done`

> 標記 task 為完成；可能觸發 lifecycle auto-transition。

| 屬性 | 值 |
|---|---|
| **Category** | apply |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent（重複呼叫同 task 為 no-op） |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::record_task_done(TaskDoneRequest) -> Versioned<Change>` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["change_id", "task_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": {
      "type": "string",
      "description": "目標 change 的 id。"
    },
    "task_id": {
      "type": "string",
      "description": "Task id（如 `1.2.3` 對應 tasks.md 內第 1.2.3 條 checkbox）。"
    },
    "touched_files": {
      "type": "array",
      "items": { "type": "string" },
      "default": [],
      "description": "本次 task 修改過的檔案相對路徑，給 drift detection 使用。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "state", "task_id", "all_tasks_done", "auto_transition"],
      "properties": {
        "change_id": { "type": "string" },
        "state": {
          "enum": ["in_progress", "code_reviewing"],
          "description": "task done 後的當前 state（可能因 auto-transition 變化）。"
        },
        "task_id": { "type": "string" },
        "all_tasks_done": {
          "type": "boolean",
          "description": "完成此 task 後，所有 task（含 feedback synthetic task）是否皆 `[x]`。"
        },
        "auto_transition": {
          "type": ["object", "null"],
          "description": "若觸發 state 自動轉移，記錄 from/to；否則 null。",
          "properties": {
            "from": { "type": "string" },
            "to": { "type": "string" },
            "trigger": { "const": "all_tasks_done" }
          }
        },
        "feedback_task_check": {
          "type": ["object", "null"],
          "description": "若 auto-transition 前驗證 feedback task marker 失敗，記錄細節；否則 null。",
          "properties": {
            "missing_markers": {
              "type": "array",
              "items": { "type": "string" },
              "description": "缺少 marker 的 feedback_id 列表。"
            },
            "re_appended": {
              "type": "boolean",
              "description": "Engine 是否已自動 re-append marker 到 tasks.md。"
            }
          }
        }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink task done <task-id> --change <id> [--touched-files <path>,...] [--json]`
- **Tool**：`task_done({ change_id, task_id, touched_files? })`
- **SDK**：`speclink.tasks.done({ changeId, taskId, touchedFiles? }): Promise<Versioned<ChangeWithTaskMeta>>`
- **HTTP**：`POST /api/projects/{project_id}/changes/{change_id}/tasks/{task_id}/done` — body = `{ touched_files? }`

#### Semantics

1. 取得 `change-exclusive` lock on `change_id`
2. 檢查 change 存在 + state ∈ {`in_progress`}（其他 state → `state.transition_invalid`）
3. Parse 當前 tasks.md，定位 `task_id` 對應 checkbox
4. 若 task 已 `[x]` → idempotent no-op，回傳當前 state（不重複 audit）
5. Mark checkbox `[ ]` → `[x]`、寫回 tasks.md
6. 記錄 `touched_files` 到 state.db（給 drift detection 用）
7. 檢查是否「全部 task `[x]`」：
   - **是**：進入 auto-transition 前驗證（design.md §6.2 step 6）：
     - 對每個 `feedback_tasks.status = 'pending'` 的紀錄、驗證 tasks.md 內仍有對應 marker 且 `[x]`
     - **任一缺漏 → `tasks.feedback_task_removed`**，state 不變；engine 自動 re-append 缺漏 marker 到 tasks.md；回傳含 `feedback_task_check.missing_markers`
     - **全 OK**：標 `feedback_tasks.status = 'done'`
     - 若 `config.require_code_review = true` → state 轉 `code_reviewing`，發 audit `change.state_changed`
     - 否則 state 不變，但設 `change.all_tasks_done = true` flag（`/speclink-archive` 此時可進）
   - **否**：state 不變
8. 寫 audit event `task.done`（**所有步驟同一 transaction**）
9. 釋放 lock
10. 回傳結果

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `state.transition_invalid` | Change 不在 `in_progress` state | `no` |
| `task.not_found` | tasks.md 內找不到 `task_id` | `no` |
| `tasks.feedback_task_removed` | Auto-transition 前驗證失敗（feedback marker 缺漏）；engine 已 re-append、需 engineer 處理 | `no` |
| `lock.not_acquired` | retry budget 內無法取得 lock | `backoff` |

#### Audit events

- `task.done` — 含 `task_id`、`touched_files`
- `change.state_changed` — 若觸發 auto-transition；含 `from`、`to`、`trigger`

#### Examples

**普通 task done、不觸發 transition**：

```bash
$ speclink task done 1.2 --change add-dark-mode --touched-files src/theme.ts,src/settings.svelte --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "in_progress",
      "task_id": "1.2",
      "all_tasks_done": false,
      "auto_transition": null,
      "feedback_task_check": null
    },
    "etag": "v5.3"
  }
}
```

**最後一個 task done、觸發 auto-transition**：

```bash
$ speclink task done 3.1 --change add-dark-mode --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "code_reviewing",
      "task_id": "3.1",
      "all_tasks_done": true,
      "auto_transition": {
        "from": "in_progress",
        "to": "code_reviewing",
        "trigger": "all_tasks_done"
      },
      "feedback_task_check": null
    },
    "etag": "v5.7"
  }
}
```

**Feedback marker 缺漏**：

```bash
$ speclink task done 3.1 --change add-dark-mode --json
{
  "ok": false,
  "error": {
    "code": "tasks.feedback_task_removed",
    "message": "Feedback task marker 'fb-2026-05-21-001' missing from tasks.md. Engine re-appended it.",
    "hint": "Open tasks.md, address the re-appended feedback task, then run `task done` again.",
    "retryable": false
  },
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "in_progress",
      "task_id": "3.1",
      "all_tasks_done": false,
      "auto_transition": null,
      "feedback_task_check": {
        "missing_markers": ["fb-2026-05-21-001"],
        "re_appended": true
      }
    }
  }
}
```

#### Cross-references

- design.md §6.2 — `task done` auto-transition 觸發條件
- design.md §6.2 — synthetic feedback task marker 機制
- design.md §16.7 — CLI 表面細節
- design.md §17.4 — `tasks.feedback_task_removed` 完整定義

---

### `review.reject`

> Reviewer 駁回 review；engine 退回 state 並自動加 synthetic feedback task。

| 屬性 | 值 |
|---|---|
| **Category** | review |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | non-idempotent（每次 reject 都產生新 feedback_id；但 partial unique index 對 normalized reason 去重） |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::record_review(RecordReviewRequest) -> Versioned<Change>`（含 reject 分支） |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["change_id", "reviewer", "phase", "reason"],
  "additionalProperties": false,
  "properties": {
    "change_id": {
      "type": "string"
    },
    "reviewer": {
      "type": "string",
      "description": "Reviewer 識別碼（如 GitHub username / email）。"
    },
    "phase": {
      "enum": ["artifact", "code"],
      "description": "`artifact` 退回 reviewing → 修 artifact；`code` 退回 in_progress + 自動加 feedback task。"
    },
    "reason": {
      "type": "string",
      "minLength": 1,
      "maxLength": 5000,
      "description": "Reject 理由；必填，empty 回 `review.reason_required`。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "state", "phase", "reviewer", "feedback_id", "synthetic_task"],
      "properties": {
        "change_id": { "type": "string" },
        "state": {
          "enum": ["reviewing", "in_progress"],
          "description": "Reject 後當前 state（依 phase 退回）。"
        },
        "phase": { "enum": ["artifact", "code"] },
        "reviewer": { "type": "string" },
        "feedback_id": {
          "type": "string",
          "description": "格式 `fb-<YYYY-MM-DD>-<NNN>`。"
        },
        "synthetic_task": {
          "type": ["object", "null"],
          "description": "若 phase=code，記錄追加到 tasks.md 的 task；artifact phase 為 null。",
          "properties": {
            "task_id": { "type": "string" },
            "marker": { "type": "string", "description": "HTML comment marker（含 feedback_id）。" },
            "appended_at": { "type": "string", "format": "date-time" }
          }
        },
        "dedup_merged": {
          "type": "boolean",
          "description": "若相同 normalized reason 在 30 分鐘內已 reject 過、本次與既有 row merge → true，且不重複 append marker。"
        },
        "similar_recent_warning": {
          "type": ["object", "null"],
          "description": "若 substring/相似度命中既有 reason，回 warning（不阻擋）；否則 null。"
        }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink review reject --change <id> --reviewer <id> --phase artifact|code --reason "<text>" [--json]`
- **Tool**：`review_reject({ change_id, reviewer, phase, reason })`
- **SDK**：`speclink.review.reject({ changeId, reviewer, phase, reason }): Promise<Versioned<RejectResult>>`
- **HTTP**：`POST /api/projects/{project_id}/changes/{change_id}/review/reject` — body = `{ reviewer, phase, reason }`

#### Semantics

1. 取得 `change-exclusive` lock on `change_id`
2. 驗證 inputs：reason 不為 empty（trim 後）→ `review.reason_required`
3. 驗證 phase 對應當前 state：
   - `phase = artifact` 必須在 `reviewing` state，否則 `review.wrong_phase`
   - `phase = code` 必須在 `code_reviewing` state，否則 `review.wrong_phase`
4. 計算 `reason_hash = sha256(normalize(reason))`（normalize = trim + lowercase + 連續 whitespace 摺疊）
5. 查詢 `feedback_tasks` 表 `(change_id, reason_hash)` 是否有 `status = 'pending'` row（partial unique index）：
   - **有**（dedup hit）→ 更新 `created_at` 不 insert 新 row；標 `dedup_merged = true`；不重複 append marker
   - **無**：
     - 生成 `feedback_id = fb-YYYY-MM-DD-NNN`（NNN = 當日序號）
     - Insert `feedback_tasks` row
6. State transition：
   - `phase = artifact` → `reviewing → proposing`，清 artifact review approval
   - `phase = code` → `code_reviewing → in_progress`，清 code review approval
7. 若 `phase = code` 且非 dedup：
   - 在 tasks.md 末尾 append synthetic task（含 HTML marker），format 見 design.md §6.2
   - Flush tasks.md（**同 transaction**）
8. 重複偵測：若 30 分鐘內同 reviewer 對同 change 用語意接近 reason（MVP 用 substring 比對）reject 過 → 回 `similar_recent_warning`（不阻擋）
9. 寫 audit events（同 transaction）：
   - `review.rejected` — 含 `reviewer` / `phase` / `feedback_id` / `dedup_merged`
   - `change.state_changed` — 含 `from` / `to`
10. 釋放 lock
11. 回傳結果

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `review.reason_required` | reason 為空或只有 whitespace | `no` |
| `review.wrong_phase` | phase 與當前 state 不對齊 | `no` |
| `state.transition_invalid` | Change 不在可 reject 的 state | `no` |
| `lock.not_acquired` | retry budget 內無法取得 lock | `backoff` |

#### Audit events

- `review.rejected` — 含 `reviewer` / `phase` / `feedback_id` / `dedup_merged`
- `change.state_changed` — 含 `from` / `to` / `trigger: "review_reject"`

#### Examples

**Code review reject、首次**：

```bash
$ speclink review reject --change add-dark-mode --reviewer alice --phase code \
    --reason "auth check missing in middleware" --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "in_progress",
      "phase": "code",
      "reviewer": "alice",
      "feedback_id": "fb-2026-05-21-001",
      "synthetic_task": {
        "task_id": "fb.1",
        "marker": "<!-- speclink:feedback id=fb-2026-05-21-001 reviewer=alice phase=code -->",
        "appended_at": "2026-05-21T16:08:33Z"
      },
      "dedup_merged": false,
      "similar_recent_warning": null
    },
    "etag": "v7.2"
  }
}
```

**重複 reject（30 分鐘內、相似 reason）**：

```bash
$ speclink review reject --change add-dark-mode --reviewer alice --phase code \
    --reason "still missing auth check in middleware" --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "in_progress",
      "phase": "code",
      "reviewer": "alice",
      "feedback_id": "fb-2026-05-21-001",
      "synthetic_task": null,
      "dedup_merged": true,
      "similar_recent_warning": {
        "message": "Reviewer 'alice' rejected with similar reason 12 minutes ago. Did the engineer actually address it?",
        "previous_feedback_id": "fb-2026-05-21-001",
        "previous_rejected_at": "2026-05-21T16:08:33Z"
      }
    },
    "etag": "v7.3"
  }
}
```

#### Cross-references

- design.md §6.2 — reject re-entry 機制完整流程
- design.md §6.2 — synthetic task marker 規格
- design.md §6.2 — `feedback_tasks` 表 schema 與 partial unique index
- design.md §16.8 — Review CLI surface

---

### `discuss.patch`

> 對 discussion 文件特定 section 做 patch（依 section 不同走 replace 或 append-only）。

| 屬性 | 值 |
|---|---|
| **Category** | discuss |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent-with-version（透過 `expected_etag`） |
| **Lock** | discussion-exclusive |
| **Provider method** | `Provider::write_discussion(WriteDiscussionRequest) -> Versioned<Discussion>` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["discussion_id", "section", "content"],
  "additionalProperties": false,
  "properties": {
    "discussion_id": {
      "type": "string",
      "description": "Discussion 的 id（如 `auth-refactor-2026-05-20`）。"
    },
    "section": {
      "enum": ["background", "open_questions", "decisions_made", "rounds", "conclusion"],
      "description": "目標 section。Engine 強制 mode：`decisions_made` / `rounds` 為 append-only；其他為 replace。"
    },
    "content": {
      "type": "string",
      "description": "Section 內容（markdown）。append-only section 傳新 entry；replace section 傳完整 section body。"
    },
    "expected_etag": {
      "type": ["string", "null"],
      "default": null,
      "description": "Optimistic concurrency token。`null` = blind overwrite。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["discussion_id", "section", "mode", "updated_at"],
      "properties": {
        "discussion_id": { "type": "string" },
        "section": { "type": "string" },
        "mode": {
          "enum": ["replace", "append"],
          "description": "Engine 實際採用的寫入模式（依 section 決定）。"
        },
        "appended_position": {
          "type": ["integer", "null"],
          "description": "若 mode=append，本次新加 entry 的 ordinal（1-based）；replace 時為 null。"
        },
        "updated_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink discuss patch <id> --section <name> --stdin [--expected-etag <etag>] [--json]`
  - Content 從 stdin 讀；append-only section 傳新 entry、replace section 傳完整 body
- **Tool**：`patch_discussion({ discussion_id, section, content, expected_etag? })`
- **SDK**：`speclink.discussions.patch({ discussionId, section, content, expectedEtag? }): Promise<Versioned<DiscussionPatchResult>>`
- **HTTP**：`PATCH /api/projects/{project_id}/discussions/{discussion_id}` — body = `{ section, content }`，header `If-Match: <expected_etag>`

#### Semantics

1. 取得 `discussion-exclusive` lock on `discussion_id`
2. 檢查 discussion 存在 → `discussion.not_found`
3. 檢查 discussion 是否已 `converged`：
   - 是、且 section ≠ `conclusion` → `discussion.locked`（converged 後只能補 conclusion）
4. 若 `expected_etag` 非 null：
   - 讀取當前 etag，不符 → `state.etag_mismatch`
5. 依 section 決定 mode：
   - `decisions_made` / `rounds` → **append-only**
     - 計算 `appended_position = current_count + 1`
     - 將 content 加到該 section 尾端
     - **Round 結束點 invariant**（design.md §8.5）：寫入後**立即 flush 到盤**（不能等到 skill 結束）
   - 其他 → **replace**
     - 完整覆寫該 section body
6. 驗證 content 大小（單 section 上限 50KB）→ `discussion.section_too_large`
7. 更新 etag
8. 寫 audit event `discussion.patched`（同 transaction）
9. 釋放 lock
10. 回傳結果

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `discussion.not_found` | discussion_id 不存在 | `no` |
| `discussion.locked` | Discussion 已 converged，且嘗試 patch 非 `conclusion` section | `no` |
| `discussion.section_too_large` | 單 section content 超過 50KB | `no` |
| `state.etag_mismatch` | `expected_etag` 與當前不符 | `read-then-retry` |
| `lock.not_acquired` | retry budget 內無法取得 lock | `backoff` |

#### Audit events

- `discussion.patched` — 含 `section` / `mode` / `appended_position`

#### Examples

**Append round entry**：

```bash
$ cat <<EOF | speclink discuss patch auth-refactor-2026-05-20 --section rounds --json
### Round 3 — 2026-05-21 — pm

User leaned toward "split gateway + provider". I argued against; ended up agreeing.
EOF
{
  "ok": true,
  "data": {
    "value": {
      "discussion_id": "auth-refactor-2026-05-20",
      "section": "rounds",
      "mode": "append",
      "appended_position": 3,
      "updated_at": "2026-05-21T18:22:14Z"
    },
    "etag": "v4.1"
  }
}
```

**Replace open_questions**：

```bash
$ echo "- ~~Q1: which library?~~ (decided: argon2)\n- Q2: rotation policy?" | \
    speclink discuss patch auth-refactor-2026-05-20 --section open_questions --json
{
  "ok": true,
  "data": {
    "value": {
      "discussion_id": "auth-refactor-2026-05-20",
      "section": "open_questions",
      "mode": "replace",
      "appended_position": null,
      "updated_at": "2026-05-21T18:24:05Z"
    },
    "etag": "v4.2"
  }
}
```

**已 converged，嘗試改 rounds**：

```bash
$ echo "..." | speclink discuss patch auth-refactor-2026-05-20 --section rounds --json
{
  "ok": false,
  "error": {
    "code": "discussion.locked",
    "message": "Discussion 'auth-refactor-2026-05-20' has converged. Only 'conclusion' section is patchable.",
    "hint": "Create a new discussion if you need to reopen the topic.",
    "retryable": false
  }
}
```

#### Cross-references

- design.md §8 — Discussion entity
- design.md §8.5 — Section patch 寫入機制與 invariant
- design.md §17.4 — `discussion.*` errors

---

### `project.init`

> 在當前 working dir 建立新 project（LocalProvider）或註冊新 project 到 HttpProvider。

| 屬性 | 值 |
|---|---|
| **Category** | project |
| **MVP** | ✓（local only；http 屬 [deferred]） |
| **Destructive** | — |
| **Idempotency** | non-idempotent |
| **Lock** | global-short |
| **Provider method** | `ProviderRegistry::register_project(RegisterProjectRequest) -> ProjectId` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["name", "provider"],
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z0-9][a-z0-9-]*[a-z0-9]$",
      "minLength": 3,
      "maxLength": 64,
      "description": "Project 識別碼；同 Provider 內唯一。"
    },
    "provider": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": { "const": "local" },
            "workspace_root": {
              "type": "string",
              "description": "LocalProvider 的 working dir 絕對路徑；預設 cwd。"
            }
          }
        },
        {
          "type": "object",
          "required": ["type", "base_url"],
          "properties": {
            "type": { "const": "http" },
            "base_url": { "type": "string", "format": "uri" },
            "auth": {
              "type": "object",
              "properties": {
                "type": { "enum": ["bearer", "header"] },
                "token_env": { "type": "string", "description": "從 ${VAR} 內插（design.md §11.2.1）。" }
              }
            }
          }
        }
      ]
    },
    "schema_id": {
      "type": ["string", "null"],
      "default": null,
      "description": "初始 active schema；預設 `default`。"
    },
    "force": {
      "type": "boolean",
      "default": false,
      "description": "若 .speclink/ 已存在、是否覆寫。Local provider only。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["project_id", "provider_type", "link_yaml_path", "created_at"],
  "additionalProperties": false,
  "properties": {
    "project_id": { "type": "string" },
    "provider_type": { "enum": ["local", "http"] },
    "link_yaml_path": { "type": "string", "description": "寫入的 link.yaml 絕對路徑。" },
    "created_at": { "type": "string", "format": "date-time" }
  }
}
```

#### Bindings

- **CLI**：`speclink init <name> [--provider local|http] [--base-url <url>] [--schema <id>] [--force] [--json]`
- **Tool**：`project_init({ name, provider: { type, ... }, schema_id?, force? })`
- **SDK**：`speclink.init({ name, provider, schemaId?, force? }): Promise<ProjectInitResult>`
- **HTTP**：（HttpProvider only）`POST /api/projects` — body = `{ name, schema_id? }`

#### Semantics

1. 取得 `global-short` lock（design.md §12.2.2）
2. Validate `name` pattern/長度
3. Provider 分流：
   - **local**：
     - 檢查 `<workspace_root>/.speclink/` 是否存在：存在且 `force=false` → `project.already_initialized`；存在且 `force=true` → 移除既有 `.speclink/`
     - 建立 `.speclink/` 目錄結構（design.md §14）：`changes/` / `discussions/` / `state.db` / `link.yaml`
     - 初始化 state.db（schema migration runs）
     - 寫入 link.yaml
   - **http**（[deferred]）：MVP build 直接拋 `provider.not_supported`
4. 寫 audit event `project.created`（同 transaction）
5. 釋放 lock
6. 回傳結果

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `project.duplicate_id` | 同 provider 內已有同 name project | `no` |
| `project.already_initialized` | local 的 .speclink/ 已存在且 `force=false` | `no` |
| `provider.not_supported` | MVP build 收到 http provider | `no` |
| `provider.connection_failed` | http provider 連不上 | `backoff` |
| `auth.failed` | http provider auth token 無效 | `no` |
| `validation.name_invalid` | name 不符 pattern | `no` |
| `lock.not_acquired` | retry budget 內無法取得 lock | `backoff` |

#### Audit events

- `project.created` — 含 `project_id` / `provider_type`

#### Examples

**Local provider success**：

```bash
$ speclink init billing-system --provider local --json
{
  "ok": true,
  "data": {
    "project_id": "billing-system",
    "provider_type": "local",
    "link_yaml_path": "/home/alice/repos/billing/.speclink/link.yaml",
    "created_at": "2026-05-21T19:02:11Z"
  }
}
```

**Already initialized**：

```bash
$ speclink init billing-system --provider local --json
{
  "ok": false,
  "error": {
    "code": "project.already_initialized",
    "message": "Directory '.speclink/' already exists. Use `--force` to overwrite.",
    "hint": "Run `speclink status` to inspect existing project.",
    "retryable": false
  }
}
```

#### Cross-references

- design.md §13.4 — project_id 分配
- design.md §13.5 — init / link / unlink 三指令職責
- design.md §14 — 檔案結構

---

### `project.link`

> 連到既有 project（純寫 link.yaml、不創建新 project）。

| 屬性 | 值 |
|---|---|
| **Category** | project |
| **MVP** | [deferred]（跟 HttpProvider 一起做） |
| **Destructive** | — |
| **Idempotency** | idempotent（重複呼叫覆寫既有 link.yaml） |
| **Lock** | none |
| **Provider method** | `ProviderRegistry::list_projects()` + 客戶端寫 link.yaml |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["base_url", "project_id"],
  "additionalProperties": false,
  "properties": {
    "base_url": { "type": "string", "format": "uri" },
    "project_id": { "type": "string" },
    "auth": {
      "type": "object",
      "properties": {
        "type": { "enum": ["bearer", "header"] },
        "token_env": { "type": "string" }
      }
    },
    "workspace_root": {
      "type": "string",
      "description": "寫 link.yaml 的目標 working dir 絕對路徑；預設 cwd。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["project_id", "link_yaml_path", "provider_metadata"],
  "properties": {
    "project_id": { "type": "string" },
    "link_yaml_path": { "type": "string" },
    "provider_metadata": {
      "type": "object",
      "description": "從 provider 拿到的 project 元資料（schema_id / created_at 等）。"
    }
  }
}
```

#### Bindings

- **CLI**：`speclink link <base-url> --project <id> [--auth-env <var>] [--json]`
- **Tool**：`project_link({ base_url, project_id, auth?, workspace_root? })`
- **SDK**：`speclink.link({ baseUrl, project, auth? }): Promise<ProjectLinkResult>`
- **HTTP**：n/a（client 端寫 link.yaml）

#### Semantics

1. MVP build：直接拋 `provider.not_supported`
2. 連 base_url、呼叫 provider 的 list/get project endpoint 驗證 project_id 存在
3. 取得 project metadata
4. 寫 `<workspace_root>/.speclink/link.yaml`（覆寫既有檔，idempotent）
5. 寫 audit event `project.linked`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `provider.not_supported` | MVP build | `no` |
| `provider.not_found` | base_url 無回應或非 SpecLink server | `backoff` |
| `project.not_found` | project_id 在 provider 端不存在 | `no` |
| `auth.failed` | auth token 無效 | `no` |
| `provider.connection_failed` | 連線失敗 | `backoff` |

#### Audit events

- `project.linked` — 含 `project_id` / `base_url`

#### Examples

**MVP build response**：

```bash
$ speclink link https://specs.team.internal --project billing-system --json
{
  "ok": false,
  "error": {
    "code": "provider.not_supported",
    "message": "HttpProvider not available in this build. Use `speclink init --provider local` instead.",
    "retryable": false
  }
}
```

#### Cross-references

- design.md §13.5 — init / link / unlink 職責劃分
- design.md §11.2 — link.yaml schema
- design.md §22.1 — Tier 3 bundled provider MVP scope

---

### `project.unlink`

> 解除當前 working dir 與 project 的綁定（**不**刪除 provider 端資料）。

| 屬性 | 值 |
|---|---|
| **Category** | project |
| **MVP** | [deferred] |
| **Destructive** | —（不刪 provider data） |
| **Idempotency** | idempotent |
| **Lock** | none |
| **Provider method** | n/a（純客戶端操作） |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "workspace_root": {
      "type": "string",
      "description": "目標 working dir；預設 cwd。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["unlinked_path"],
  "properties": {
    "unlinked_path": { "type": "string", "description": "刪除的 link.yaml 路徑。" }
  }
}
```

#### Bindings

- **CLI**：`speclink unlink [--json]`
- **Tool**：`project_unlink({ workspace_root? })`
- **SDK**：`speclink.unlink(): Promise<UnlinkResult>`
- **HTTP**：n/a

#### Semantics

1. MVP build：直接拋 `provider.not_supported`
2. 檢查 `<workspace_root>/.speclink/link.yaml` 存在 → 不存在 idempotent no-op
3. 刪除 link.yaml（保留 .speclink/ 目錄與 state.db，避免誤刪 LocalProvider 資料）
4. 寫 audit event `project.unlinked`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `provider.not_supported` | MVP build | `no` |

#### Audit events

- `project.unlinked` — 含 unlinked path

#### Cross-references

- design.md §13.5
- design.md §22.1 — Tier 2/3 MVP 限制

---

### `project.status`

> 取得當前 project 的元資料與運作狀態。

| 屬性 | 值 |
|---|---|
| **Category** | project |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent（read-only） |
| **Lock** | none |
| **Provider method** | 多 method 組合（`get_project_metadata` + counts queries） |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "properties": {}
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["provider_type", "project_id", "working_dir", "changes_count", "discussions_count", "schema_active"],
  "additionalProperties": false,
  "properties": {
    "provider_type": { "enum": ["local", "http"] },
    "project_id": { "type": "string" },
    "working_dir": { "type": "string" },
    "current_change": {
      "type": ["object", "null"],
      "description": "若有 change 處於 in_progress + actor 為當前 host；否則 null。",
      "properties": {
        "change_id": { "type": "string" },
        "state": { "type": "string" },
        "actor": { "type": "object" }
      }
    },
    "changes_count": {
      "type": "object",
      "properties": {
        "proposing": { "type": "integer", "minimum": 0 },
        "reviewing": { "type": "integer", "minimum": 0 },
        "ready": { "type": "integer", "minimum": 0 },
        "in_progress": { "type": "integer", "minimum": 0 },
        "code_reviewing": { "type": "integer", "minimum": 0 },
        "archived": { "type": "integer", "minimum": 0 }
      }
    },
    "discussions_count": {
      "type": "object",
      "properties": {
        "active": { "type": "integer", "minimum": 0 },
        "converged": { "type": "integer", "minimum": 0 }
      }
    },
    "schema_active": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink status [--json]`
- **Tool**：`project_status({})`
- **SDK**：`speclink.status(): Promise<ProjectStatus>`
- **HTTP**：`GET /api/projects/{project_id}`

#### Semantics

1. 讀 link.yaml → provider_type / project_id
2. 讀 state.db 統計 changes 各 state count
3. 讀 state.db 統計 discussions count
4. 讀 config.yaml → schema_active
5. 若有 change `state = 'in_progress'` 且 `actor.host_id` 為當前 host → 填 current_change
6. 回傳結果（**不寫 audit event**）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `project.not_found` | link.yaml 不存在 | `no` |
| `link.malformed` | link.yaml 格式錯誤 | `no` |
| `provider.connection_failed` | http provider 連不上 | `backoff` |

#### Examples

**Local**：

```bash
$ speclink status --json
{
  "ok": true,
  "data": {
    "provider_type": "local",
    "project_id": "billing-system",
    "working_dir": "/home/alice/repos/billing",
    "current_change": {
      "change_id": "add-dark-mode",
      "state": "in_progress",
      "actor": { "agent_host": "claude-code", "os_user": "alice" }
    },
    "changes_count": { "proposing": 1, "reviewing": 0, "ready": 2, "in_progress": 1, "code_reviewing": 0, "archived": 12 },
    "discussions_count": { "active": 2, "converged": 5 },
    "schema_active": "default"
  }
}
```

#### Cross-references

- design.md §13 — project identity
- design.md §6.2 — state set

---

### `config.read`

> 讀取 project 的 config.yaml（rules / roles / schema / tools / locale）。

| 屬性 | 值 |
|---|---|
| **Category** | config |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::read_config() -> Versioned<Config>` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "properties": {}
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "description": "Config.yaml 解析後的完整內容（rules / roles / schema / tools / locale 等）；shape 見 design.md §11.3。"
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink config show [--key <path>] [--json]`
- **Tool**：`read_config({})`
- **SDK**：`speclink.config.read(): Promise<Versioned<Config>>`
- **HTTP**：`GET /api/projects/{project_id}/config`

#### Semantics

1. 從 provider 讀 config（LocalProvider → 讀 config.yaml；HttpProvider → GET 端點）
2. 回傳 `Versioned<Config>`（含 etag 供後續 `config.write` 使用）
3. CLI `--key <path>` 模式下、用 JSONPath subset 提取部分

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `config.not_found` | config 不存在（罕見、project 初始化必建） | `no` |
| `config.malformed` | config.yaml 解析失敗 | `no` |
| `provider.connection_failed` | http provider | `backoff` |

#### Cross-references

- design.md §11 — Config 結構
- design.md §11.4 — CLI 統一介面

---

### `config.write`

> 寫入 config.yaml；支援 `set <key> <value>`（局部 patch）與 `edit`（完整覆寫）兩模式。

| 屬性 | 值 |
|---|---|
| **Category** | config |
| **MVP** | ✓ |
| **Destructive** | —（但會改變全團隊 SDD 規則） |
| **Idempotency** | idempotent-with-version |
| **Lock** | global-short |
| **Provider method** | `Provider::write_config(WriteConfigRequest) -> Versioned<Config>` |

#### Inputs

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "required": ["mode", "key", "value"],
      "properties": {
        "mode": { "const": "set" },
        "key": { "type": "string", "description": "JSONPath subset，如 `rules.require_code_review`。" },
        "value": { "description": "Any JSON-serializable value。" },
        "expected_etag": { "type": ["string", "null"], "default": null }
      }
    },
    {
      "type": "object",
      "required": ["mode", "content"],
      "properties": {
        "mode": { "const": "edit" },
        "content": { "type": "string", "description": "完整 config.yaml 內容（YAML 字串）。" },
        "expected_etag": { "type": ["string", "null"], "default": null }
      }
    }
  ]
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": { "type": "object", "description": "Patch 後的完整 Config 內容。" },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：
  - `speclink config set <key> <value> [--json]` — mode = set
  - `speclink config edit [--editor <cmd>] [--json]` — mode = edit（launch $EDITOR、保存後讀回 content）
- **Tool**：`write_config({ mode, ... })`
- **SDK**：`speclink.config.write({ mode, ... }): Promise<Versioned<Config>>`
- **HTTP**：`PATCH /api/projects/{project_id}/config` — body = inputs

#### Semantics

1. 取得 `global-short` lock
2. Validate inputs 依 mode
3. 若 `expected_etag` 非 null：讀當前 etag，不符 → `state.etag_mismatch`
4. 若 mode = set：
   - 讀當前 config
   - 依 key 套用 patch
   - Validate 全 config schema
5. 若 mode = edit：
   - Validate `content` 為合法 YAML
   - Validate schema
6. 寫回 provider；更新 etag
7. 寫 audit event `config.changed`（含 keys diff 摘要、同 transaction）
8. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `config.malformed` | YAML 解析失敗 / schema validation 失敗 | `no` |
| `config.key_not_found` | mode=set 但 key 路徑無效 | `no` |
| `state.etag_mismatch` | etag 衝突 | `read-then-retry` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `config.changed` — 含 `mode` / `keys_changed` / `etag_before` / `etag_after`

#### Examples

**Set mode**：

```bash
$ speclink config set rules.require_code_review false --json
{
  "ok": true,
  "data": {
    "value": { "rules": { "require_code_review": false, ... }, ... },
    "etag": "v3.4"
  }
}
```

#### Cross-references

- design.md §11.3 — config.yaml 完整範例
- design.md §11.4 — CLI 統一介面
- design.md §11.6 — Malformed 行為

---

### `schema.list`

> 列出 project 內所有 schema 定義。

| 屬性 | 值 |
|---|---|
| **Category** | schema |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::list_schemas() -> Vec<SchemaSummary>` |

#### Inputs

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {}
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["schemas"],
  "properties": {
    "schemas": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["schema_id", "is_active", "is_builtin"],
        "properties": {
          "schema_id": { "type": "string" },
          "is_active": { "type": "boolean" },
          "is_builtin": { "type": "boolean", "description": "true 為 SpecLink 內建（如 `default`）；false 為 fork。" },
          "forked_from": { "type": ["string", "null"], "description": "若 fork、源 schema id。" },
          "created_at": { "type": "string", "format": "date-time" }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink schemas [--json]`
- **Tool**：`list_schemas({})`
- **SDK**：`speclink.schemas.list(): Promise<SchemaSummary[]>`
- **HTTP**：`GET /api/projects/{project_id}/schemas`

#### Semantics

1. Read-only：讀 provider 內 schema 表
2. 標記 `is_active` 依 config.yaml `schema_active`
3. 回傳列表

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `provider.connection_failed` | http provider | `backoff` |

#### Cross-references

- design.md §10 — Schema 抽象

---

### `schema.show`

> 取得單一 schema 完整定義。

| 屬性 | 值 |
|---|---|
| **Category** | schema |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::get_schema(id: &str) -> Schema` |

#### Inputs

```json
{
  "type": "object",
  "required": ["schema_id"],
  "additionalProperties": false,
  "properties": {
    "schema_id": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["schema_id", "artifacts", "templates", "rules"],
  "properties": {
    "schema_id": { "type": "string" },
    "is_builtin": { "type": "boolean" },
    "forked_from": { "type": ["string", "null"] },
    "artifacts": {
      "type": "array",
      "description": "Schema 定義的 artifact kinds（如 proposal/spec/design/tasks）。",
      "items": {
        "type": "object",
        "properties": {
          "kind": { "type": "string" },
          "required": { "type": "boolean" },
          "capability_scoped": { "type": "boolean", "description": "如 spec.md 需要 capability slug。" }
        }
      }
    },
    "templates": {
      "type": "object",
      "description": "各 artifact kind 對應的 template 內容。",
      "additionalProperties": { "type": "string" }
    },
    "rules": {
      "type": "object",
      "description": "Schema-level validation rules。"
    }
  }
}
```

#### Bindings

- **CLI**：`speclink schema show <id> [--json]`
- **Tool**：`show_schema({ schema_id })`
- **SDK**：`speclink.schemas.get({ schemaId }): Promise<Schema>`
- **HTTP**：`GET /api/projects/{project_id}/schemas/{id}`

#### Semantics

1. Read-only：讀 provider 內 schema 完整內容
2. 不存在 → `schema.not_found`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `schema.not_found` | schema_id 不存在 | `no` |

#### Cross-references

- design.md §10 — Schema 抽象
- design.md §10.3 — Templates

---

### `schema.fork`

> 從既有 schema fork 新 schema（custom overlay）。

| 屬性 | 值 |
|---|---|
| **Category** | schema |
| **MVP** | ✓ |
| **Destructive** | —（但會影響 SDD 流程，需 admin scope） |
| **Idempotency** | non-idempotent |
| **Lock** | global-short |
| **Provider method** | `Provider::fork_schema(ForkSchemaRequest) -> ()`（post-MVP 搬到 Engine、見 §19.1.1） |

#### Inputs

```json
{
  "type": "object",
  "required": ["source_id", "target_id"],
  "additionalProperties": false,
  "properties": {
    "source_id": { "type": "string", "description": "Fork 源 schema。" },
    "target_id": {
      "type": "string",
      "pattern": "^[a-z0-9][a-z0-9-]*[a-z0-9]$",
      "description": "新 schema id。"
    },
    "overlay": {
      "type": ["object", "null"],
      "default": null,
      "description": "可選的 overlay patch（如改特定 template、加 artifact kind）。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["schema_id", "forked_from", "created_at"],
  "properties": {
    "schema_id": { "type": "string" },
    "forked_from": { "type": "string" },
    "created_at": { "type": "string", "format": "date-time" }
  }
}
```

#### Bindings

- **CLI**：`speclink schema fork <source> <target> [--overlay <yaml-file>] [--json]`
- **Tool**：`fork_schema({ source_id, target_id, overlay? })`
- **SDK**：`speclink.schemas.fork({ sourceId, targetId, overlay? }): Promise<SchemaForkResult>`
- **HTTP**：`POST /api/projects/{project_id}/schemas/{target_id}` — body = `{ source_id, overlay? }`

#### Semantics

1. 取得 `global-short` lock
2. Validate `target_id` pattern
3. 檢查 source schema 存在 → `schema.not_found`
4. 檢查 target_id 不重複 → `schema.duplicate_id`
5. Read source schema、apply overlay
6. Validate 結果 schema 完整性
7. Insert new schema row
8. 寫 audit event `schema.forked`（同 transaction）
9. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `schema.not_found` | source_id 不存在 | `no` |
| `schema.duplicate_id` | target_id 已存在 | `no` |
| `schema.malformed_overlay` | overlay 套用後 schema 不合法 | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `schema.forked` — 含 `source_id` / `target_id`

#### Cross-references

- design.md §10.1 — Schema fork
- design.md §19.6 — Config tampering 安全
- design.md §19.1.1 — Engine vs Provider 責任（post-MVP 搬到 Engine）

---

### `schema.delete` ⚠

> 刪除 schema（**destructive**）。Builtin schema 不可刪。

| 屬性 | 值 |
|---|---|
| **Category** | schema |
| **MVP** | ✓ |
| **Destructive** | ⚠ |
| **Idempotency** | non-idempotent（第二次拋 not_found） |
| **Lock** | global-short |
| **Provider method** | `Provider::delete_schema(schema_id) -> ()` |

#### Inputs

```json
{
  "type": "object",
  "required": ["schema_id", "force"],
  "additionalProperties": false,
  "properties": {
    "schema_id": { "type": "string" },
    "force": {
      "type": "boolean",
      "description": "必須為 true；catalogue 標 destructive。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["schema_id", "deleted_at"],
  "properties": {
    "schema_id": { "type": "string" },
    "deleted_at": { "type": "string", "format": "date-time" }
  }
}
```

#### Bindings

- **CLI**：`speclink schema delete <id> --force [--json]`
- **Tool**：`delete_schema({ schema_id, force: true })` ⚠
- **SDK**：`speclink.schemas.delete({ schemaId, force: true }): Promise<SchemaDeleteResult>`
- **HTTP**：`DELETE /api/projects/{project_id}/schemas/{id}` — header `X-Confirm-Force: true`

#### Semantics

1. `force = false` → `validation.force_required`
2. 取得 `global-short` lock
3. 檢查 schema 存在 → `schema.not_found`
4. 檢查非 builtin → `schema.builtin_protected`
5. 檢查非 active schema → `schema.in_use`（active schema 不可刪、先 switch）
6. 檢查無 change 引用此 schema → `schema.in_use`
7. Delete schema row
8. 寫 audit event `schema.deleted`（同 transaction）
9. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `validation.force_required` | `force` 非 true | `no` |
| `schema.not_found` | schema_id 不存在 | `no` |
| `schema.builtin_protected` | 嘗試刪內建 schema | `no` |
| `schema.in_use` | 仍是 active schema 或被 change 引用 | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `schema.deleted` — 含 `schema_id`、與 state mutation 同 transaction

#### Cross-references

- design.md §10 — Schema 抽象
- design.md §17.6 — Destructive audit
- design.md §22.3 — Aliases 安全規則（destructive op 套 alias 觸發 audit）

---

### `discuss.new`

> 建立新 discussion（含 background section 初值）。

| 屬性 | 值 |
|---|---|
| **Category** | discuss |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | non-idempotent |
| **Lock** | discussion-exclusive |
| **Provider method** | `Provider::create_discussion(CreateDiscussionRequest) -> Versioned<Discussion>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["topic"],
  "additionalProperties": false,
  "properties": {
    "topic": {
      "type": "string",
      "minLength": 3,
      "maxLength": 200,
      "description": "Discussion 主題（自然語句）。Engine 自動生成 id（`<slug>-<YYYY-MM-DD>`）。"
    },
    "linked_change_id": {
      "type": ["string", "null"],
      "default": null,
      "description": "可選的關聯 change id（雙向 link，design.md §8.4）。"
    },
    "role": {
      "type": ["string", "null"],
      "default": null,
      "description": "Discussion 主導角色（design.md §9）；如 `pm` / `sa` / `qa` / `rd`。"
    },
    "background": {
      "type": ["string", "null"],
      "default": null,
      "description": "初始 background section 內容（markdown）；可後續用 `discuss.patch` 修改。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["discussion_id", "topic", "state", "linked_change_id", "role", "created_at"],
      "properties": {
        "discussion_id": { "type": "string", "description": "Engine 生成的 id。" },
        "topic": { "type": "string" },
        "state": { "const": "active" },
        "linked_change_id": { "type": ["string", "null"] },
        "role": { "type": ["string", "null"] },
        "created_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink discuss new "<topic>" [--linked-change <id>] [--role <name>] [--background-stdin] [--json]`
- **Tool**：`new_discussion({ topic, linked_change_id?, role?, background? })`
- **SDK**：`speclink.discussions.create({ topic, linkedChangeId?, role?, background? }): Promise<Versioned<Discussion>>`
- **HTTP**：`POST /api/projects/{project_id}/discussions`

#### Semantics

1. 生成 discussion_id（topic slug + date）；衝突自動加 `-2` `-3` 後綴
2. 取得 `discussion-exclusive` lock
3. 若 `linked_change_id` 非 null：validate change 存在
4. 若 `role` 非 null：validate role 存在於 config.yaml
5. 建立 discussion entity（state = `active`，預設 5 section 結構：background / open_questions / decisions_made / rounds / conclusion）
6. 寫入 background section（若有）
7. 寫 audit event `discussion.created`
8. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | linked_change_id 不存在 | `no` |
| `role.not_found` | role 不存在 | `no` |
| `validation.topic_too_short` | topic < 3 char | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `discussion.created` — 含 `discussion_id` / `linked_change_id` / `role`

#### Cross-references

- design.md §8 — Discussion entity
- design.md §8.4 — 雙向連結
- design.md §9 — Role 機制

---

### `discuss.list`

> 列出 discussions；預設不顯示 converged。

| 屬性 | 值 |
|---|---|
| **Category** | discuss |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::list_discussions(DiscussionFilter) -> Vec<DiscussionSummary>` |

#### Inputs

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "include_converged": { "type": "boolean", "default": false },
    "linked_change_id": { "type": ["string", "null"], "default": null }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["discussions"],
  "properties": {
    "discussions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["discussion_id", "topic", "state", "created_at"],
        "properties": {
          "discussion_id": { "type": "string" },
          "topic": { "type": "string" },
          "state": { "enum": ["active", "converged"] },
          "linked_change_id": { "type": ["string", "null"] },
          "role": { "type": ["string", "null"] },
          "created_at": { "type": "string", "format": "date-time" },
          "concluded_at": { "type": ["string", "null"], "format": "date-time" }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink discuss list [--all] [--include-converged] [--linked-change <id>] [--json]`
- **Tool**：`list_discussions({ include_converged?, linked_change_id? })`
- **SDK**：`speclink.discussions.list({ ... }): Promise<DiscussionSummary[]>`
- **HTTP**：`GET /api/projects/{project_id}/discussions?include_converged=...`

#### Semantics

1. Read-only
2. 預設過濾 `state = active`
3. 依 filter 選項擴大範圍

#### Errors

無特殊。`provider.connection_failed` 同其他 read op。

#### Cross-references

- design.md §8 — Discussion entity

---

### `discuss.show`

> 取得 discussion 完整內容（5 section）。

| 屬性 | 值 |
|---|---|
| **Category** | discuss |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::get_discussion(id) -> Versioned<Discussion>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["discussion_id"],
  "additionalProperties": false,
  "properties": {
    "discussion_id": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["discussion_id", "topic", "state", "sections", "created_at"],
      "properties": {
        "discussion_id": { "type": "string" },
        "topic": { "type": "string" },
        "state": { "enum": ["active", "converged"] },
        "linked_change_id": { "type": ["string", "null"] },
        "role": { "type": ["string", "null"] },
        "sections": {
          "type": "object",
          "required": ["background", "open_questions", "decisions_made", "rounds", "conclusion"],
          "properties": {
            "background": { "type": "string" },
            "open_questions": { "type": "string" },
            "decisions_made": { "type": "string" },
            "rounds": { "type": "string" },
            "conclusion": { "type": "string" }
          }
        },
        "created_at": { "type": "string", "format": "date-time" },
        "concluded_at": { "type": ["string", "null"], "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink discuss show <id> [--section <name>] [--json]`
- **Tool**：`show_discussion({ discussion_id })`
- **SDK**：`speclink.discussions.get({ discussionId }): Promise<Versioned<Discussion>>`
- **HTTP**：`GET /api/projects/{project_id}/discussions/{id}`

#### Semantics

1. Read-only
2. CLI `--section` 模式只回該 section（仍含 etag）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `discussion.not_found` | discussion_id 不存在 | `no` |

#### Cross-references

- design.md §8 — Discussion entity
- design.md §8.2 — 檔案格式

---

### `discuss.conclude`

> 標記 discussion 為 `converged`（terminal action）+ 寫入 conclusion section。

| 屬性 | 值 |
|---|---|
| **Category** | discuss |
| **MVP** | ✓ |
| **Destructive** | —（但不可 reopen） |
| **Idempotency** | non-idempotent（converged 後再呼 → `discussion.already_converged`） |
| **Lock** | discussion-exclusive |
| **Provider method** | `Provider::conclude_discussion(ConcludeRequest) -> Versioned<Discussion>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["discussion_id", "conclusion"],
  "additionalProperties": false,
  "properties": {
    "discussion_id": { "type": "string" },
    "conclusion": {
      "type": "string",
      "minLength": 10,
      "description": "Conclusion section 內容（markdown）。"
    },
    "expected_etag": { "type": ["string", "null"], "default": null }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["discussion_id", "state", "concluded_at"],
      "properties": {
        "discussion_id": { "type": "string" },
        "state": { "const": "converged" },
        "concluded_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink discuss conclude <id> --conclusion-stdin [--expected-etag <etag>] [--json]`
- **Tool**：`conclude_discussion({ discussion_id, conclusion, expected_etag? })`
- **SDK**：`speclink.discussions.conclude({ discussionId, conclusion, expectedEtag? }): Promise<Versioned<Discussion>>`
- **HTTP**：`POST /api/projects/{project_id}/discussions/{id}/conclude`

#### Semantics

1. 取得 `discussion-exclusive` lock
2. 檢查 discussion 存在 → `discussion.not_found`
3. 檢查 state = `active` → 已 converged 拋 `discussion.already_converged`
4. 若 `expected_etag` 非 null：比對 → `state.etag_mismatch`
5. 寫入 conclusion section
6. State `active → converged`，紀錄 `concluded_at`
7. 寫 audit event `discussion.concluded`（同 transaction）
8. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `discussion.not_found` | discussion_id 不存在 | `no` |
| `discussion.already_converged` | 已 converged | `no` |
| `state.etag_mismatch` | etag 衝突 | `read-then-retry` |
| `validation.conclusion_too_short` | conclusion < 10 char | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `discussion.concluded` — 含 `discussion_id`

#### Cross-references

- design.md §8 — Discussion entity
- design.md §8.5 — Section patch（conclusion section 規則）
- design.md §6.1 — Discussion 2 states

---

### `discuss.delete` ⚠

> 刪除 discussion（**destructive**）。Converged 仍可刪。

| 屬性 | 值 |
|---|---|
| **Category** | discuss |
| **MVP** | ✓ |
| **Destructive** | ⚠ |
| **Idempotency** | non-idempotent |
| **Lock** | discussion-exclusive |
| **Provider method** | `Provider::delete_discussion(id) -> ()` |

#### Inputs

```json
{
  "type": "object",
  "required": ["discussion_id", "force"],
  "additionalProperties": false,
  "properties": {
    "discussion_id": { "type": "string" },
    "force": { "type": "boolean" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["discussion_id", "deleted_at"],
  "properties": {
    "discussion_id": { "type": "string" },
    "deleted_at": { "type": "string", "format": "date-time" }
  }
}
```

#### Bindings

- **CLI**：`speclink discuss delete <id> --force [--json]`
- **Tool**：`delete_discussion({ discussion_id, force: true })` ⚠
- **SDK**：`speclink.discussions.delete({ discussionId, force: true }): Promise<DiscussionDeleteResult>`
- **HTTP**：`DELETE /api/projects/{project_id}/discussions/{id}` — header `X-Confirm-Force: true`

#### Semantics

1. `force = false` → `validation.force_required`
2. 取得 `discussion-exclusive` lock
3. 檢查 discussion 存在 → `discussion.not_found`
4. 若有 linked_change → 解除雙向 link（不刪 change）
5. Delete discussion row + section content
6. 寫 audit event `discussion.deleted`（同 transaction）
7. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `validation.force_required` | force 非 true | `no` |
| `discussion.not_found` | discussion_id 不存在 | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `discussion.deleted` — 含 `discussion_id` / `was_converged`

#### Cross-references

- design.md §8.1 — Discussion 獨立於 Change
- design.md §17.6 — Destructive audit

---

### `change.list`

> 列出 changes；可 filter state。

| 屬性 | 值 |
|---|---|
| **Category** | change |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::list_changes(ChangeFilter) -> Vec<ChangeSummary>` |

#### Inputs

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "state": {
      "type": ["string", "array", "null"],
      "items": { "enum": ["proposing", "reviewing", "ready", "in_progress", "code_reviewing", "archived"] },
      "default": null
    },
    "include_archived": { "type": "boolean", "default": false }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["changes"],
  "properties": {
    "changes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["change_id", "name", "state", "created_at"],
        "properties": {
          "change_id": { "type": "string" },
          "name": { "type": "string" },
          "state": { "type": "string" },
          "tasks_total": { "type": "integer" },
          "tasks_done": { "type": "integer" },
          "created_at": { "type": "string", "format": "date-time" },
          "updated_at": { "type": "string", "format": "date-time" }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink list [--changes] [--state <name>...] [--include-archived] [--json]`
- **Tool**：`list_changes({ state?, include_archived? })`
- **SDK**：`speclink.changes.list({ ... }): Promise<ChangeSummary[]>`
- **HTTP**：`GET /api/projects/{project_id}/changes?state=...`

#### Semantics

1. Read-only
2. 預設過濾 `state != archived`
3. 排序：state（proposing > reviewing > ready > in_progress > code_reviewing > archived）+ updated_at desc

#### Errors

無特殊。

#### Cross-references

- design.md §6.2 — Change 6 states

---

### `change.show`

> 取得單一 change 元資料 + state + tasks 概覽。

| 屬性 | 值 |
|---|---|
| **Category** | change |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::get_change(id) -> Versioned<Change>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "name", "description", "state", "schema_id", "artifacts", "tasks", "created_at"],
      "properties": {
        "change_id": { "type": "string" },
        "name": { "type": "string" },
        "description": { "type": "string" },
        "state": { "type": "string" },
        "schema_id": { "type": "string" },
        "actor": { "type": ["object", "null"] },
        "artifacts": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "kind": { "type": "string" },
              "exists": { "type": "boolean" },
              "size_bytes": { "type": "integer" }
            }
          }
        },
        "tasks": {
          "type": "object",
          "properties": {
            "total": { "type": "integer" },
            "done": { "type": "integer" },
            "feedback_pending": { "type": "integer" }
          }
        },
        "review_status": {
          "type": "object",
          "properties": {
            "artifact_approved": { "type": "boolean" },
            "code_approved": { "type": "boolean" }
          }
        },
        "created_at": { "type": "string", "format": "date-time" },
        "updated_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink show change <id> [--json]`
- **Tool**：`show_change({ change_id })`
- **SDK**：`speclink.changes.get({ changeId }): Promise<Versioned<Change>>`
- **HTTP**：`GET /api/projects/{project_id}/changes/{id}`

#### Semantics

1. Read-only
2. 不存在 → `change.not_found`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |

#### Cross-references

- design.md §6.2 — Change states
- design.md §7 — Artifact DAG

---

### `change.delete` ⚠

> 刪除 change（**destructive**）。Linked discussions **不**一起刪。

| 屬性 | 值 |
|---|---|
| **Category** | change |
| **MVP** | ✓ |
| **Destructive** | ⚠ |
| **Idempotency** | non-idempotent |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::delete_change(id) -> ()` |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id", "force"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "force": { "type": "boolean" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["change_id", "deleted_at", "deleted_artifacts_count"],
  "properties": {
    "change_id": { "type": "string" },
    "deleted_at": { "type": "string", "format": "date-time" },
    "deleted_artifacts_count": { "type": "integer" },
    "unlinked_discussions": {
      "type": "array",
      "items": { "type": "string" },
      "description": "解除 linked 但保留的 discussion id 列表。"
    }
  }
}
```

#### Bindings

- **CLI**：`speclink delete change <id> --force [--json]`
- **Tool**：`delete_change({ change_id, force: true })` ⚠
- **SDK**：`speclink.changes.delete({ changeId, force: true }): Promise<ChangeDeleteResult>`
- **HTTP**：`DELETE /api/projects/{project_id}/changes/{id}` — header `X-Confirm-Force: true`

#### Semantics

1. `force = false` → `validation.force_required`
2. 取得 `change-exclusive` lock
3. 檢查 change 存在 → `change.not_found`
4. 查詢 linked discussions → 解除雙向 link（**不刪 discussion**）
5. 刪除所有 artifacts (cascade)
6. 刪除 review history / feedback_tasks / state row
7. 刪除 `.speclink/changes/<name>/` 目錄（LocalProvider）
8. 寫 audit event `change.deleted`（含 archived 前 state、artifact count、unlinked discussions）
9. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `validation.force_required` | force 非 true | `no` |
| `change.not_found` | change_id 不存在 | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `change.deleted` — 含 `change_id` / `prior_state` / `deleted_artifacts_count` / `unlinked_discussions`

#### Cross-references

- design.md §8.1 — Discussion 獨立、不 cascade
- design.md §17.6 — Destructive audit

---

### `artifact.read`

> 讀取 change 內單一 artifact body。

| 屬性 | 值 |
|---|---|
| **Category** | artifact |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::read_artifact(ReadArtifactRequest) -> Versioned<Artifact>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id", "kind"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "kind": { "type": "string" },
    "capability": { "type": ["string", "null"], "default": null }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "kind", "content", "size_bytes", "updated_at"],
      "properties": {
        "change_id": { "type": "string" },
        "kind": { "type": "string" },
        "capability": { "type": ["string", "null"] },
        "content": { "type": "string" },
        "size_bytes": { "type": "integer" },
        "updated_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink artifact read <kind> --change <id> [--capability <name>] [--json]`
- **Tool**：`read_artifact({ change_id, kind, capability? })`
- **SDK**：`speclink.artifacts.read({ changeId, kind, capability? }): Promise<Versioned<Artifact>>`
- **HTTP**：`GET /api/projects/{project_id}/changes/{change_id}/artifacts/{kind}`

#### Semantics

1. Read-only
2. 檢查 change 存在 → `change.not_found`
3. 檢查 artifact 存在 → `artifact.not_found`
4. 回傳 `Versioned<Artifact>`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `artifact.not_found` | artifact 不存在 | `no` |
| `artifact.unknown_kind` | kind 不屬於當前 schema | `no` |

#### Cross-references

- design.md §7 — Artifact DAG

---

### `apply.start`

> Ensure actor 進入 apply 階段；對非 transition state 回 success + state 描述。

| 屬性 | 值 |
|---|---|
| **Category** | apply |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent（雙向；見 design.md §6.2 表） |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::update_change_state(UpdateChangeStateRequest)` + actor assignment |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "actor": {
      "type": ["object", "null"],
      "default": null,
      "description": "Actor 識別（agent_host / os_user / host_id）；省略則 engine 自動推導當前環境。",
      "properties": {
        "agent_host": { "type": "string" },
        "os_user": { "type": "string" },
        "host_id": { "type": "string" }
      }
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "state", "actor", "message"],
      "properties": {
        "change_id": { "type": "string" },
        "state": { "type": "string", "description": "當前最新 state（可能非 in_progress）。" },
        "actor": {
          "type": ["object", "null"],
          "properties": {
            "agent_host": { "type": "string" },
            "os_user": { "type": "string" },
            "host_id": { "type": "string" }
          }
        },
        "message": {
          "type": ["string", "null"],
          "description": "若 state ∈ {code_reviewing, archived}，附 hint 訊息；否則 null。"
        }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink apply start <change-id> [--actor <id>] [--json]`
- **Tool**：`apply_start({ change_id, actor? })`
- **SDK**：`speclink.apply.start({ changeId, actor? }): Promise<ApplyStartResult>`
- **HTTP**：`POST /api/projects/{project_id}/changes/{change_id}/apply/start`

#### Semantics

| 當前 state | 行為 |
|---|---|
| `proposing` / `reviewing` | ✗ `state.transition_invalid` |
| `ready` | → `in_progress` + assign actor，回 state=in_progress |
| `in_progress` | no-op + assign actor，回 state=in_progress |
| `code_reviewing` | **不 transition**，回 state=code_reviewing + message |
| `archived` | **不 transition**，回 state=archived + message |

詳細見 design.md §6.2 表。

1. 取得 `change-exclusive` lock
2. 讀 change state → 依上表分流
3. State change → 寫 audit event `change.state_changed` + actor assignment
4. 若 no-op + actor reassignment → 寫 audit event `change.actor_assigned`
5. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `state.transition_invalid` | state ∈ {proposing, reviewing} | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `change.state_changed` — 若觸發 transition
- `change.actor_assigned` — 含 actor 細節（無論是否 transition）

#### Examples

**Ready → in_progress**：

```bash
$ speclink apply start add-dark-mode --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "in_progress",
      "actor": { "agent_host": "claude-code", "os_user": "alice", "host_id": "macbook-alice" },
      "message": null
    },
    "etag": "v6.1"
  }
}
```

**Already in code_reviewing**：

```bash
$ speclink apply start add-dark-mode --json
{
  "ok": true,
  "data": {
    "value": {
      "change_id": "add-dark-mode",
      "state": "code_reviewing",
      "actor": null,
      "message": "Already in code review; nothing to apply."
    },
    "etag": "v6.5"
  }
}
```

#### Cross-references

- design.md §6.2 — apply start/pause 詳細表
- design.md §16.7 — CLI 細節

---

### `apply.pause`

> in_progress → ready；ready 上 idempotent；其他 state 拋 error。

| 屬性 | 值 |
|---|---|
| **Category** | apply |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent（在 ready 上） |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::update_change_state(UpdateChangeStateRequest)` |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "state"],
      "properties": {
        "change_id": { "type": "string" },
        "state": { "enum": ["ready"] }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink apply pause <change-id> [--json]`
- **Tool**：`apply_pause({ change_id })`
- **SDK**：`speclink.apply.pause({ changeId }): Promise<ApplyPauseResult>`
- **HTTP**：`POST /api/projects/{project_id}/changes/{change_id}/apply/pause`

#### Semantics

| 當前 state | 行為 |
|---|---|
| `in_progress` | → `ready` + 清 actor |
| `ready` | no-op idempotent |
| 其他 | ✗ `state.transition_invalid` |

1. 取得 `change-exclusive` lock
2. 讀 state → 分流
3. Transition → 寫 audit `change.state_changed` + 清 actor
4. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `state.transition_invalid` | state ∉ {in_progress, ready} | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `change.state_changed` — 若觸發 transition

#### Cross-references

- design.md §6.2 — apply start/pause 詳細表

---

### `review.approve`

> Reviewer approve review（phase artifact / code）；觸發 state transition。

| 屬性 | 值 |
|---|---|
| **Category** | review |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | non-idempotent（已 approved 重呼 → `review.already_approved`） |
| **Lock** | change-exclusive |
| **Provider method** | `Provider::record_review(RecordReviewRequest)` |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id", "reviewer", "phase"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "reviewer": { "type": "string" },
    "phase": { "enum": ["artifact", "code"] },
    "note": { "type": ["string", "null"], "default": null, "maxLength": 2000 }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["change_id", "state", "phase", "reviewer", "approved_at"],
      "properties": {
        "change_id": { "type": "string" },
        "state": { "type": "string", "description": "Approve 後當前 state。" },
        "phase": { "enum": ["artifact", "code"] },
        "reviewer": { "type": "string" },
        "approved_at": { "type": "string", "format": "date-time" }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink review approve --change <id> --reviewer <id> --phase artifact|code [--note "..."] [--json]`
- **Tool**：`review_approve({ change_id, reviewer, phase, note? })`
- **SDK**：`speclink.review.approve({ changeId, reviewer, phase, note? }): Promise<ReviewApproveResult>`
- **HTTP**：`POST /api/projects/{project_id}/changes/{change_id}/review/approve`

#### Semantics

1. 取得 `change-exclusive` lock
2. Validate phase 對應當前 state：
   - `phase = artifact` 必須 `reviewing` state
   - `phase = code` 必須 `code_reviewing` state
   - mismatch → `review.wrong_phase`
3. 檢查不重複 approve → `review.already_approved`
4. 紀錄 approval（reviewer / phase / note / approved_at）
5. State transition：
   - `phase = artifact` → `reviewing → ready`
   - `phase = code` → 不直接 transition（archived 由 `archive.run` 觸發）；標 `code_approved = true`
6. 寫 audit events `review.approved`、必要時 `change.state_changed`
7. 釋放 lock

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |
| `review.wrong_phase` | phase 與當前 state 不對齊 | `no` |
| `review.already_approved` | 同 phase 已 approved | `no` |
| `lock.not_acquired` | lock 取不到 | `backoff` |

#### Audit events

- `review.approved` — 含 `reviewer` / `phase`
- `change.state_changed` — 若 phase=artifact，含 from=reviewing / to=ready

#### Cross-references

- design.md §6.2 — Review approve 觸發 state transition
- design.md §16.8 — Review CLI surface

---

### `review.history`

> 列出 change 全部 review approve/reject 紀錄。

| 屬性 | 值 |
|---|---|
| **Category** | review |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::get_review_history(change_id) -> Vec<ReviewRecord>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "phase": { "enum": ["artifact", "code", null], "default": null }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["history"],
  "properties": {
    "history": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["reviewer", "phase", "action", "timestamp"],
        "properties": {
          "feedback_id": { "type": ["string", "null"] },
          "reviewer": { "type": "string" },
          "phase": { "enum": ["artifact", "code"] },
          "action": { "enum": ["approve", "reject"] },
          "reason": { "type": ["string", "null"] },
          "note": { "type": ["string", "null"] },
          "timestamp": { "type": "string", "format": "date-time" },
          "feedback_task_status": { "type": ["string", "null"], "enum": ["pending", "done", null] }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink review history --change <id> [--phase <name>] [--json]`
- **Tool**：`review_history({ change_id, phase? })`
- **SDK**：`speclink.review.history({ changeId, phase? }): Promise<ReviewRecord[]>`
- **HTTP**：`GET /api/projects/{project_id}/changes/{change_id}/review/history?phase=...`

#### Semantics

1. Read-only
2. 依時間排序（asc）
3. 對應每筆 reject 紀錄附上 `feedback_task_status`（pending / done）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |

#### Cross-references

- design.md §16.8 — Review history 規格

---

### `archive.run`

> Spec delta merge + state → archived。

| 屬性 | 值 |
|---|---|
| **Category** | archive |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | non-idempotent（archived 後再呼 → `state.transition_invalid`） |
| **Lock** | change-exclusive + global-short |
| **Provider method** | `Provider::archive_change(ArchiveRequest)`（post-MVP 搬到 Engine） |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "skip_specs": {
      "type": "boolean",
      "default": false,
      "description": "Emergency escape hatch；跳過 spec delta merge 步驟、直接 state→archived。標記 audit。"
    },
    "no_validate": {
      "type": "boolean",
      "default": false,
      "description": "跳過 archive 前 validation。"
    },
    "mark_tasks_complete": {
      "type": "boolean",
      "default": false,
      "description": "若 tasks 仍有未完成項、強制標 complete（emergency 用）。"
    },
    "yes": {
      "type": "boolean",
      "default": false,
      "description": "跳過互動 confirm（CLI 預設 prompt）。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["change_id", "state", "merged_specs", "archived_at"],
  "properties": {
    "change_id": { "type": "string" },
    "state": { "const": "archived" },
    "merged_specs": {
      "type": "array",
      "description": "本次 merge 的 capability spec 列表。",
      "items": {
        "type": "object",
        "properties": {
          "capability": { "type": "string" },
          "lines_added": { "type": "integer" },
          "lines_removed": { "type": "integer" }
        }
      }
    },
    "archived_at": { "type": "string", "format": "date-time" }
  }
}
```

#### Bindings

- **CLI**：`speclink archive <change-id> [--skip-specs] [--no-validate] [--mark-tasks-complete] [--yes] [--json]`
- **Tool**：`archive_change({ change_id, skip_specs?, no_validate?, mark_tasks_complete?, yes? })`
- **SDK**：`speclink.archive.run({ changeId, ... }): Promise<ArchiveResult>`
- **HTTP**：`POST /api/projects/{project_id}/changes/{change_id}/archive`

#### Semantics

1. 取得 `change-exclusive` lock + `global-short` lock（spec merge 需跨 capability）
2. 檢查 state 允許 archive（design.md §6.2）：
   - 若 `require_code_review = true` → 必須 `code_reviewing` 且 `code_approved`
   - 否則 → `in_progress` 且 `all_tasks_done = true`
3. 若 `!no_validate`：跑 validation
4. 若 tasks 未全完 且 `mark_tasks_complete = false` → `change.tasks_incomplete`
5. 若 `mark_tasks_complete = true` → 強制標 tasks，寫 audit `tasks.force_completed`
6. 若 `!skip_specs`：套用 spec delta merge 到 capability spec
7. 若 `skip_specs = true` → 寫 audit `archive.specs_skipped`（紀錄此 escape hatch）
8. State `→ archived`，紀錄 `archived_at`
9. 寫 audit events `change.archived` + `change.state_changed`（同 transaction）
10. 釋放 locks

#### Errors

| Code | 觸發條件 | Retry | Status |
|---|---|---|---|
| `change.not_found` | change_id 不存在 | `no` | implemented (A4) |
| `state.transition_invalid` | state 不允許 archive | `no` | implemented (A4) |
| `change.tasks_incomplete` | tasks 未全完且未 force | `no` | implemented (A4) |
| `change.code_review_pending` | require_code_review=true 但未 code_approved | `no` | reserved (add-review slice) |
| `validation.archive_failed` | archive 前 validation 失敗 | `no` | reserved (add-analyze slice) |
| `lock.not_acquired` | lock 取不到 | `backoff` | reserved (add-locking-and-concurrency slice) |

#### Audit events

- `change.archived` — 含 `merged_specs` / `skip_specs` / `mark_tasks_complete` 等 flag
- `change.state_changed` — from=<prior> to=archived
- `archive.specs_skipped` — 若 `skip_specs = true`（emergency 路徑）
- `tasks.force_completed` — 若 `mark_tasks_complete = true`

#### Cross-references

- design.md §6.2 — Archive 進入條件
- design.md §16.9 — Archive CLI surface
- design.md §19.1.1 — Engine vs Provider 責任

---

### `spec.list`

> 列出 project 內所有 capability 的 canonical spec。

| 屬性 | 值 |
|---|---|
| **Category** | spec |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::list_specs(SpecFilter) -> Vec<SpecSummary>` |

#### Inputs

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "updated_since": {
      "type": ["string", "null"],
      "format": "date-time",
      "default": null,
      "description": "若提供、只回傳 `last_updated >= updated_since` 的 spec。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["specs"],
  "properties": {
    "specs": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["capability", "last_updated", "size_bytes"],
        "properties": {
          "capability": { "type": "string", "description": "Capability slug，如 `auth-gateway`。" },
          "last_updated": { "type": "string", "format": "date-time" },
          "size_bytes": { "type": "integer", "minimum": 0 },
          "last_change_id": {
            "type": ["string", "null"],
            "description": "最後一次更新此 spec 的 archived change id；null 為從未經 archive 寫入。"
          }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink list --specs [--updated-since <iso-date>] [--json]`
- **Tool**：`list_specs({ updated_since? })`
- **SDK**：`speclink.specs.list({ updatedSince? }): Promise<SpecSummary[]>`
- **HTTP**：`GET /api/projects/{project_id}/specs?updated_since=...`

#### Semantics

1. Read-only
2. LocalProvider：掃描 `.speclink/specs/` 目錄下所有 capability 子目錄
3. HttpProvider：呼叫對應 endpoint
4. 排序：`last_updated` 降序

#### Errors

無特殊。`provider.connection_failed` 同其他 read op。

#### Examples

```bash
$ speclink list --specs --json
{
  "ok": true,
  "data": {
    "specs": [
      { "capability": "auth-gateway", "last_updated": "2026-05-20T10:00:00Z", "size_bytes": 4521, "last_change_id": "add-oauth-provider" },
      { "capability": "search-engine", "last_updated": "2026-04-15T08:30:00Z", "size_bytes": 8273, "last_change_id": "search-relevance-v2" }
    ]
  }
}
```

#### Cross-references

- design.md §7 — Artifact DAG（canonical spec 為 archived change 合併產物）
- design.md §10 — Schema 抽象（spec.md 的 capability_scoped 規則）

---

### `spec.show`

> 讀取單一 capability 的 canonical spec 完整內容。

| 屬性 | 值 |
|---|---|
| **Category** | spec |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | `Provider::get_spec(capability) -> Versioned<Spec>` |

#### Inputs

```json
{
  "type": "object",
  "required": ["capability"],
  "additionalProperties": false,
  "properties": {
    "capability": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["value", "etag"],
  "properties": {
    "value": {
      "type": "object",
      "required": ["capability", "content", "purpose", "last_updated", "size_bytes"],
      "properties": {
        "capability": { "type": "string" },
        "content": { "type": "string", "description": "完整 markdown spec body。" },
        "purpose": {
          "type": ["string", "null"],
          "description": "從 content 解析的 `## Purpose` section body；若不存在為 null。給 propose Step 3 快速比對用。"
        },
        "last_updated": { "type": "string", "format": "date-time" },
        "size_bytes": { "type": "integer" },
        "last_change_id": { "type": ["string", "null"] }
      }
    },
    "etag": { "type": "string" }
  }
}
```

#### Bindings

- **CLI**：`speclink show spec <capability> [--purpose-only] [--json]`
- **Tool**：`show_spec({ capability })`
- **SDK**：`speclink.specs.get({ capability }): Promise<Versioned<Spec>>`
- **HTTP**：`GET /api/projects/{project_id}/specs/{capability}`

#### Semantics

1. Read-only
2. 檢查 capability 存在 → `spec.not_found`
3. 讀取完整 markdown body
4. 解析 `## Purpose` section（若存在）
5. 回傳 `Versioned<Spec>`

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `spec.not_found` | capability 不存在 | `no` |

#### Examples

```bash
$ speclink show spec auth-gateway --json
{
  "ok": true,
  "data": {
    "value": {
      "capability": "auth-gateway",
      "content": "# auth-gateway\n\n## Purpose\nCentralize authentication...\n\n## Requirements\n...",
      "purpose": "Centralize authentication...",
      "last_updated": "2026-05-20T10:00:00Z",
      "size_bytes": 4521,
      "last_change_id": "add-oauth-provider"
    },
    "etag": "v8.3"
  }
}
```

#### Cross-references

- design.md §7 — Artifact DAG
- design.md §10 — Schema 抽象

---

### `instructions.get`

> 取得指定 artifact kind 的 AI prompt body（給 skill 用）。

| 屬性 | 值 |
|---|---|
| **Category** | meta |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | 無；engine 內 template lookup |

#### Inputs

```json
{
  "type": "object",
  "required": ["kind"],
  "additionalProperties": false,
  "properties": {
    "kind": {
      "enum": ["proposal", "spec", "design", "tasks", "apply", "ingest", "archive", "discuss", "commit"],
      "description": "Artifact kind 或 skill phase。"
    },
    "change_id": {
      "type": ["string", "null"],
      "default": null,
      "description": "可選的 change context；某些 kind（如 commit / ingest）需要 change context 才能生成完整 prompt。"
    },
    "role": {
      "type": ["string", "null"],
      "default": null,
      "description": "適用於 `kind = discuss`。Role id（如 `pm` / `sa` / `qa` / `rd` 或 config.yaml 內 custom role）；engine 依此返回 role-specific instruction body。其他 kind 忽略此欄位。若 kind=discuss 且 role 未提供 → 退而求 `config.default_role`、再不行 → engine 回 `availableRoles[]` 供 caller 顯示給 user 挑。"
    },
    "discussion_id": {
      "type": ["string", "null"],
      "default": null,
      "description": "適用於 `kind = discuss`。若提供、engine 在回應內 hydrate `linked_changes_context`（linked changes 的 state / summary）+ 其他 discussion-specific context。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["kind", "instruction"],
  "properties": {
    "kind": { "type": "string" },
    "schema_id": {
      "type": ["string", "null"],
      "description": "Active schema id（如 `spec-driven`）。"
    },
    "instruction": {
      "type": "string",
      "description": "Schema-specific guidance for this kind（markdown body）。對於 artifact kinds (proposal/spec/design/tasks) 為產生指引；對 workflow phase kinds (apply/ingest/archive/discuss/commit) 為 skill prompt body。"
    },
    "template": {
      "type": ["string", "null"],
      "description": "Artifact 骨架（markdown 結構）。AI 依此填入內容。**Workflow phase kinds 為 null**（apply/ingest/archive/discuss/commit 不產 artifact）。對 artifact kinds 從 schema 的 templates 取得。"
    },
    "context": {
      "type": ["string", "null"],
      "description": "Project context（from `config.yaml#context`）。**約束 AI 行為，不應複製進 artifact 內容**。config 無 context 時為 null。"
    },
    "rules": {
      "type": "array",
      "description": "Artifact-specific rules（from `config.yaml#rules.<kind>[]`）。**約束 AI 行為，不應複製進 artifact 內容**。空 array = 無 rules。",
      "items": { "type": "string" }
    },
    "dependencies": {
      "type": "array",
      "description": "AI 應該先讀的前置 artifact（依 schema 的 artifact DAG 推導）。對於 artifact kinds 列出需先完成的 prerequisite artifacts；對 workflow phase kinds 列出 skill 啟動前該讀的 context files。",
      "items": {
        "type": "object",
        "required": ["kind"],
        "properties": {
          "kind": { "type": "string", "description": "Dependency 的 artifact kind。" },
          "capability": {
            "type": ["string", "null"],
            "description": "Multi-instance artifact (e.g., spec) 才有；其他為 null。"
          },
          "path": {
            "type": ["string", "null"],
            "description": "若 dependency 是 file-based artifact，給出相對路徑供 caller `artifact.read` 用。"
          }
        }
      }
    },
    "output_path": {
      "type": ["string", "null"],
      "description": "Artifact 寫入路徑（相對 `.speclink/changes/<change>/`）。**Workflow phase kinds 為 null**。"
    },
    "locale": {
      "type": ["string", "null"],
      "description": "Resolved locale（如 `Traditional Chinese (繁體中文)`，from `config.yaml#locale`）。決定 AI 產出 artifact body 用的語言。**Spec artifacts 一律英文**（normative SHALL/MUST 語法）。null = 無 locale 設定、AI 沿用 conversation 語言。"
    },
    "available_roles": {
      "type": ["array", "null"],
      "description": "僅 kind=discuss 且 caller 未指定 role（也無 config.default_role）時、engine 回此列表供 caller 提示 user 挑選；含 builtin (pm/sa/qa/rd) + config 自訂 role。其他狀況為 null。",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "description": { "type": "string" },
          "builtin": { "type": "boolean" }
        }
      }
    },
    "linked_changes_context": {
      "type": ["array", "null"],
      "description": "僅 kind=discuss 且 discussion_id 提供且 discussion 有 linked change 時、engine 預先載入該 change 的 snapshot；其他狀況為 null。",
      "items": {
        "type": "object",
        "properties": {
          "change_id": { "type": "string" },
          "state": { "type": "string" },
          "artifacts_summary": { "type": "object" }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink instructions <kind> [--change <id>] [--role <role-id>] [--discussion <id>] [--json]`
- **Tool**：`get_instructions({ kind, change_id?, role?, discussion_id? })`
- **SDK**：`speclink.instructions.get({ kind, changeId?, role?, discussionId? }): Promise<Instructions>`
- **HTTP**：`GET /api/projects/{project_id}/instructions/{kind}?change_id=...&role=...&discussion_id=...`

#### Semantics

1. Read-only
2. 從 active schema 拿對應 kind 的 template
3. 若 `change_id` 提供、套用 change-specific 插值（如 capability list、prior state）
4. **若 `kind = discuss`**：
   - 若 `role` 提供 → engine 用該 role 的 role-aware instruction body
   - 若 `role` 未提供 → 退至 `config.default_role`；仍無 → 回 `available_roles[]` 但 `content` 為通用 fallback prompt
   - 若 `discussion_id` 提供 → engine 載入 linked change snapshots（透過 `change.show`）填入 `linked_changes_context[]`
5. 其他 kind 忽略 `role` 與 `discussion_id` 欄位（傳了不報錯，但無作用）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `instructions.unknown_kind` | kind 不屬於當前 schema | `no` |
| `change.not_found` | 提供 change_id 但不存在 | `no` |
| `discussion.not_found` | 提供 discussion_id 但不存在 | `no` |
| `role.unknown` | 提供 role 但 config 內找不到該 role | `no` |

#### Examples

**Artifact kind (proposal)**：

```bash
$ speclink instructions proposal --change add-auth --json
{
  "ok": true,
  "data": {
    "kind": "proposal",
    "schema_id": "spec-driven",
    "instruction": "Write a proposal explaining why this change is needed...",
    "template": "# <Title>\n\n## Why\n<motivation>\n\n## What Changes\n<list>\n\n## Impact\n<files>\n",
    "context": "## 專案\nSpecLink — Spec-Driven Development workflow engine...",
    "rules": [
      "必須明確指出影響的 crate",
      "必須包含 Non-Goals 段落"
    ],
    "dependencies": [],
    "output_path": "proposal.md",
    "locale": "Traditional Chinese (繁體中文)",
    "available_roles": null,
    "linked_changes_context": null
  }
}
```

**Discuss kind + role**：

```bash
$ speclink instructions discuss --role sa --discussion sse-vs-websocket --json
{
  "ok": true,
  "data": {
    "kind": "discuss",
    "schema_id": "default",
    "instruction": "You are operating as an SA (Solutions Architect)...",
    "template": null,
    "context": "## 專案\nSpecLink — Spec-Driven Development workflow engine...",
    "rules": [],
    "dependencies": [],
    "output_path": null,
    "locale": "Traditional Chinese (繁體中文)",
    "available_roles": null,
    "linked_changes_context": [
      { "change_id": "add-notifications", "state": "proposing", "artifacts_summary": { "proposal": "done", "spec": "ready", "tasks": "blocked" } }
    ]
  }
}
```

**Discuss kind 未指定 role、config 無 default**：

```bash
$ speclink instructions discuss --json
{
  "ok": true,
  "data": {
    "kind": "discuss",
    "schema_id": "default",
    "instruction": "<generic discuss prompt without role specialization>",
    "template": null,
    "context": "<from config.yaml#context, may be null>",
    "rules": [],
    "dependencies": [],
    "output_path": null,
    "locale": "Traditional Chinese (繁體中文)",
    "available_roles": [
      { "id": "pm", "description": "Product Manager", "builtin": true },
      { "id": "sa", "description": "Solutions Architect", "builtin": true },
      { "id": "qa", "description": "QA Engineer", "builtin": true },
      { "id": "rd", "description": "Senior RD", "builtin": true }
    ],
    "linked_changes_context": null
  }
}
```

#### Cross-references

- design.md §16.5 — Instructions
- design.md §10.3 — Templates
- design.md §9 — Role 機制（含 role precedence / config.yaml 自訂）

---

### `analyze.run`

> Engine 內建分析（含 drift + completeness 摘要）。

| 屬性 | 值 |
|---|---|
| **Category** | meta |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | engine method（組合 `get_change` + `read_artifact` + drift check） |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["change_id", "state", "completeness", "drift_summary"],
  "properties": {
    "change_id": { "type": "string" },
    "state": { "type": "string" },
    "completeness": {
      "type": "object",
      "properties": {
        "proposal": { "type": "boolean" },
        "spec_capabilities": { "type": "array", "items": { "type": "string" } },
        "tasks_total": { "type": "integer" },
        "tasks_done": { "type": "integer" },
        "feedback_pending": { "type": "integer" }
      }
    },
    "drift_summary": {
      "type": "object",
      "properties": {
        "touched_files": { "type": "array", "items": { "type": "string" } },
        "untracked_changes": { "type": "integer" },
        "drift_score": { "type": "number", "minimum": 0, "maximum": 1 }
      }
    },
    "warnings": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Engine 找到的可疑點（如 stale review approval）。"
    }
  }
}
```

#### Bindings

- **CLI**：`speclink analyze <change-id> [--json]`
- **Tool**：`analyze_change({ change_id })`
- **SDK**：`speclink.analyze.run({ changeId }): Promise<AnalysisReport>`
- **HTTP**：`GET /api/projects/{project_id}/changes/{change_id}/analyze`

#### Semantics

1. Read-only
2. 組合 `change.show` + `validate.run` + `drift.run` 結果
3. 不寫任何 state；不發 audit

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |

#### Cross-references

- design.md §16.10 — Engine 內建分析

---

### `validate.run`

> Schema 驗證 runner（proposal / spec / tasks 完整性）。

| 屬性 | 值 |
|---|---|
| **Category** | meta |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | engine method（組合 schema rules + artifact reads） |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" },
    "strict": {
      "type": "boolean",
      "default": false,
      "description": "Strict mode 下 warning 也計入失敗。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["change_id", "ok", "findings"],
  "properties": {
    "change_id": { "type": "string" },
    "ok": { "type": "boolean" },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["code", "level", "message"],
        "properties": {
          "code": { "type": "string", "description": "如 `validation.proposal_missing` / `validation.tasks_empty`。" },
          "level": { "enum": ["error", "warning", "info"] },
          "artifact": { "type": ["string", "null"] },
          "message": { "type": "string" },
          "hint": { "type": ["string", "null"] }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink validate <change-id> [--strict] [--json]`
- **Tool**：`validate_change({ change_id, strict? })`
- **SDK**：`speclink.validate.run({ changeId, strict? }): Promise<ValidationReport>`
- **HTTP**：`GET /api/projects/{project_id}/changes/{change_id}/validate?strict=...`

#### Semantics

1. Read-only
2. 依 active schema 跑所有 validation rules
3. 蒐集 findings、計算 ok（無 error 即 true；strict 模式下 warning 也算 error）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |

#### Cross-references

- design.md §16.10 — Engine 內建分析
- design.md §10.4 — Schema 設計約束

---

### `drift.run`

> Drift detection（spec vs implementation 差異）。

| 屬性 | 值 |
|---|---|
| **Category** | meta |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | engine method（讀 touched_files + git diff 比對） |

#### Inputs

```json
{
  "type": "object",
  "required": ["change_id"],
  "additionalProperties": false,
  "properties": {
    "change_id": { "type": "string" }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["change_id", "drift_score", "items"],
  "properties": {
    "change_id": { "type": "string" },
    "drift_score": { "type": "number", "minimum": 0, "maximum": 1 },
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["kind", "path", "severity", "message"],
        "properties": {
          "kind": { "enum": ["untracked_file_change", "spec_unimplemented", "task_no_evidence", "test_failing"] },
          "path": { "type": "string" },
          "severity": { "enum": ["high", "medium", "low"] },
          "message": { "type": "string" }
        }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink drift <change-id> [--json]`
- **Tool**：`drift_change({ change_id })`
- **SDK**：`speclink.drift.run({ changeId }): Promise<DriftReport>`
- **HTTP**：`GET /api/projects/{project_id}/changes/{change_id}/drift`

#### Semantics

1. Read-only
2. 讀 task touched_files 紀錄（state.db）
3. 跑 git diff 比對 working dir
4. Score 越高、drift 越嚴重（MVP 用簡單啟發式：untracked / unimplemented spec 比例）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `change.not_found` | change_id 不存在 | `no` |

#### Cross-references

- design.md §16.10 — Engine 內建分析

---

### `doctor.run`

> 跑 9 個檢查類別、回 findings。

| 屬性 | 值 |
|---|---|
| **Category** | doctor |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent（finding 結果可變；但不寫 state） |
| **Lock** | none |
| **Provider method** | 無；engine 跑各 check |

#### Inputs

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "quick": { "type": "boolean", "default": false, "description": "跳過 artifacts 慢檢查。" },
    "check": {
      "type": ["string", "null"],
      "enum": ["cli", "project", "provider", "provider-mapping", "security", "config", "skill", "state", "artifacts", null],
      "default": null,
      "description": "只跑指定 category。"
    },
    "fix": { "type": "boolean", "default": false, "description": "對 allowlist 內的 finding 自動修。" },
    "check_mapping_live": {
      "type": "boolean",
      "default": false,
      "description": "provider-mapping 預設 dry-run；true 才打網路。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["findings", "summary"],
  "properties": {
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["code", "level", "message"],
        "properties": {
          "code": { "type": "string", "description": "形如 `doctor.<category>.<code>`；完整 26+ 條見 design.md §17.5。" },
          "category": { "type": "string" },
          "level": { "enum": ["error", "warning", "info"] },
          "message": { "type": "string" },
          "fix_message": { "type": ["string", "null"], "description": "人讀說明該怎麼修。" },
          "fix_command": { "type": ["string", "null"], "description": "機器可讀的 fix 指令；若 `fix=true` 且在 allowlist、engine 已執行。" },
          "auto_fixed": { "type": "boolean", "default": false }
        }
      }
    },
    "summary": {
      "type": "object",
      "properties": {
        "errors": { "type": "integer" },
        "warnings": { "type": "integer" },
        "auto_fixed": { "type": "integer" }
      }
    }
  }
}
```

#### Bindings

- **CLI**：`speclink doctor [--quick] [--check <cat>] [--fix] [--check-mapping --live] [--json]`
- **Tool**：`run_doctor({ quick?, check?, fix?, check_mapping_live? })`
- **SDK**：`speclink.doctor.run({ ... }): Promise<DoctorReport>`
- **HTTP**：n/a（HTTP 端拆各別 check endpoint，design.md §16.12）

#### Semantics

1. 依 `check` 過濾 category（null = 全跑）
2. `quick = true` 跳 artifacts category
3. 跑每個 category 內的 check function
4. 對 finding `auto_fixable: true` 且 `fix = true` 的：執行 fix（design.md §16.12.3 allowlist：gitignore / AGENTS markers / feedback task re-append）
5. 收集 findings 與 summary
6. 不寫 audit（doctor 是 diagnostic 工具）；auto-fix 操作走原本 op 的 audit（如 feedback re-append 走 `tasks.feedback_task_appended`）

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `doctor.check_unknown` | check 名稱無效 | `no` |

完整 finding code 表見 design.md §17.5；本 op 不重複列。

#### Exit codes（CLI）

- 0 — 無 error；可有 warning
- 1 — 有 warning（無 error）
- 2 — 有 error
- 詳見 design.md §16.12.2

#### Examples

**Quick run**：

```bash
$ speclink doctor --quick --json
{
  "ok": true,
  "data": {
    "findings": [
      {
        "code": "doctor.project.gitignore_missing_speclink",
        "category": "project",
        "level": "warning",
        "message": ".speclink/link.yaml not in .gitignore",
        "fix_message": "Add `.speclink/link.yaml` to your .gitignore.",
        "fix_command": "speclink doctor --fix --check project",
        "auto_fixed": false
      }
    ],
    "summary": { "errors": 0, "warnings": 1, "auto_fixed": 0 }
  }
}
```

#### Cross-references

- design.md §16.12 — Doctor 完整檢查項目
- design.md §17.5 — Doctor finding code reference

---

### `tool.describe`

> 輸出 tool catalogue（多格式）；給 SDK 或外部工具 consume。

| 屬性 | 值 |
|---|---|
| **Category** | tool |
| **MVP** | ✓ |
| **Destructive** | — |
| **Idempotency** | idempotent (read-only) |
| **Lock** | none |
| **Provider method** | 無；engine 內 catalogue lookup |

#### Inputs

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "format": {
      "enum": ["json", "text", "copilot-sdk"],
      "default": "json",
      "description": "輸出格式。MVP 支援 3 種；其他（copilotkit / openai / langchain）post-MVP。"
    },
    "filter": {
      "type": "array",
      "items": { "type": "string" },
      "description": "只輸出指定 operation id。"
    },
    "categories": {
      "type": "array",
      "items": { "type": "string" },
      "description": "只輸出指定 category。"
    },
    "phases": {
      "type": "array",
      "items": { "enum": ["discuss", "propose", "apply", "archive", "ingest"] },
      "description": "只輸出指定 skill phase 用到的 op。"
    },
    "full": {
      "type": "boolean",
      "default": false,
      "description": "true 輸出 35+ ops 全集；false 輸出 12 個 curated ops（design.md §22.2）。"
    }
  }
}
```

#### Outputs

```json
{
  "type": "object",
  "required": ["format", "content"],
  "properties": {
    "format": { "type": "string" },
    "content": {
      "description": "Format-dependent 結構。json 為陣列 of tool descriptors；text 為 markdown；copilot-sdk 為對應 SDK shape。",
      "oneOf": [
        { "type": "array" },
        { "type": "string" }
      ]
    }
  }
}
```

#### Bindings

- **CLI**：`speclink describe-tools [--format json|text|copilot-sdk] [--filter <op>,...] [--categories <c>,...] [--phases <p>,...] [--full] [--json]`
- **Tool**：n/a（meta-op；SDK 內呼 `makeXxxToolDescriptors`，design.md §22.3）
- **SDK**：`speclink.describeTools({ format, ... }): Promise<ToolDescription>`
- **HTTP**：`GET /api/projects/{project_id}/tool-catalogue?format=...`

#### Semantics

1. Read-only
2. 從 catalogue 拿全 op 列表
3. 套 filter / categories / phases / full 篩選
4. 依 `format` 序列化：
   - **json**：標準 JSON 陣列 of `{ id, name, description, parameters: JSONSchema }`
   - **text**：human-readable markdown 表
   - **copilot-sdk**：對應 `defineTool` 接受的 shape

#### Errors

| Code | 觸發條件 | Retry |
|---|---|---|
| `tool.format_not_supported` | format 不在 enum | `no` |
| `tool.unknown_op` | filter 內含未知 op | `no` |

#### Examples

**Default curated subset (json)**：

```bash
$ speclink describe-tools --format json
[
  { "id": "discuss.new", "name": "new_discussion", "description": "...", "parameters": { ... } },
  { "id": "change.create", "name": "new_change", "description": "...", "parameters": { ... } },
  ...
]
```

#### Cross-references

- design.md §16.2 — describe-tools format
- design.md §22.1 — Layer 1 curated 12 ops
- design.md §22.3 — `makeXxxToolDescriptors` 對應
