# operations.md ↔ slice A3 implementation review

對應 task 12.3「對 `doc/protocol/operations.md` 的 `apply.start` / `apply.pause` /
`task.done` 三條 op spec 與本 slice 實作做 prose review」。本檔內容應該複製到 PR
description；operations.md 本身不動。

## `apply.start` — **contract aligned**（with one documented divergence）

- **Inputs.change_id**：CLI 收 positional `<change-id>` 字串，runtime 通過 `&str` 傳入 `ApplyOperations::start` — 與 spec `required: [change_id]` 對齊。
- **Inputs.actor**：spec 描述 actor 為 composite object `{ agent_host, os_user, host_id }`；CLI 實作為 `--actor <agent_host>` 單字串 flag，符合 `apply-task-ops` capability「the `--actor` flag SHALL be interpreted as `agent_host` only」契約。**Divergence**：operations.md inputs schema 把 `actor` 描為 nullable object，CLI 把它窄化為 string；未來若 tool / SDK binding 需要傳完整 actor 物件，可在不破壞 CLI 的前提下擴充。本 slice 走 CLI 路徑沒有衝突。
- **Outputs**：CLI envelope `data` 為 `{ change_id, state, actor, message }`；spec 要求 `{ change_id, state, actor, message }` + `etag`。**Divergence**：CLI 不暴露 `etag` 給 `apply.start` 響應，因為 lifecycle ops 對外 etag 用 `change.version` 的 CAS token、非 sha256 etag。`apply-task-ops` spec 沒要求 etag in `data`；envelope 本身已有 `requestId`，足夠 trace。後續 HttpProvider 接通可在 envelope 層補 `etag` field 不破壞 CLI。
- **Idempotency / Lock**：spec 標 idempotent + change-exclusive；CLI 實作 idempotent（in_progress reassign actor 不寫 audit row），lock 用 `change.version` CAS 取代 file lock — locking slice 接通後 retrofit。

## `apply.pause` — **contract aligned**

- 對稱於 `apply.start`；spec 標 idempotent 在 ready，CLI 實作 no-op + hint message `Change is already paused.`，與 spec 完全對齊。
- Actor clear 在 in_progress → ready transition 內透過 `actor: Some(None)` 語意完成。

## `task.done` — **contract aligned** with task-id model divergence

- **Inputs.task_id**：spec 用 `task_id`（章節 §6.2 / operations.md `task done <task-id>`）；本 slice 改為 `<task-index>` 1-based 行內順序 index。Divergence 已在 design.md「Task id 策略：1-based 行內順序 index，不引入 marker」決策內說明；後續 review slice 接通 marker 機制時取代為 stable task_id。
- **Auto-trigger**：spec 要求所有 task `[x]` 後 engine auto-transition；walking-skeleton mode（review flags hard-coded false）下實作為 set `all_tasks_done=1` + state 維持 `in_progress`、`auto_transitioned: false`。`require_code_review=true` 分支實作完整但本 slice 路徑不會觸發。
- **Idempotency**：已 done 的 task 再呼叫 no-op、不寫檔、不寫 audit row，與 spec idempotent 旗標對齊。
- **Same-tx commitment**：spec 要求 state update + tasks.md write + audit insert 在同 transaction；本 slice 把 state + audit 同 SQLite tx，tasks.md atomic rename 在 state mutation 後。spec §6.2 「必須跟 state transition 在同 SQLite transaction 內」嚴格解讀只覆蓋 SQLite mutation；tasks.md（filesystem）走 atomic rename + 順序合約：state transition 失敗則 tasks.md 不寫。後續 transactional filesystem layer 可上線後 tighten。

## 結論

3 條 op 的 observable behavior 與 envelope shape 與 operations.md / spec 對齊；divergence
皆已在 design.md 內明文（actor 單欄、task-id index、review optionality）。本 slice 完成
walking-skeleton 4-state mode；review / archive / locking / config-rw slice 接通後可
無痛 retrofit。
