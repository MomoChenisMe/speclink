# Tool Binding for speclink-apply

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations. All tool calls return a typed result; errors are thrown as `SpecLinkError` with `code` / `message` / `retryable` fields (design.md §17.1).

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `project.status` | `project_status({})` |
| `change.list` | `list_changes({ state? })` |
| `change.show` | `show_change({ change_id })` |
| `apply.start` | `apply_start({ change_id, actor?: object })` |
| `apply.pause` | `apply_pause({ change_id })` |
| `instructions.get` | `get_instructions({ kind, change_id })` |
| `artifact.read` | `read_artifact({ change_id, kind, capability?: string })` |
| `artifact.write` | `write_artifact({ change_id, kind, content, capability?: string, overwrite?: boolean, expected_etag?: string })` |
| `analyze.run` | `analyze_change({ change_id })` |
| `drift.run` | `drift_change({ change_id })` |
| `task.done` | `task_done({ change_id, task_id, touched_files?: string[] })` |
| `review.approve` | `review_approve({ change_id, reviewer, phase, note?: string })` |
| `discuss.show` | `show_discussion({ discussion_id })` |

---

## SDK Convenience

The same operations are exposed as typed SDK methods (Tier 2/3 — see design.md §22.1):

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const change = await speclink.changes.get({ changeId: "add-auth" });
const result = await speclink.apply.start({ changeId: "add-auth" });
const taskResult = await speclink.tasks.done({
  changeId: "add-auth",
  taskId: "1.2",
  touchedFiles: ["src/auth.rs", "src/middleware.rs"]
});
```

---

## Common Patterns

### Task completion result handling

```typescript
const result = await task_done({ change_id, task_id, touched_files });
if (result.value.auto_transition) {
  // 最後一個 task done、engine auto-transitioned
  console.log(`State changed: ${result.value.auto_transition.from} → ${result.value.auto_transition.to}`);
}
if (result.value.feedback_task_check?.re_appended) {
  // Engine re-appended a missing feedback marker；surface to user
}
```

### Artifact rewrite for analyze-fix loop

```typescript
const { value, etag } = await read_artifact({ change_id, kind: "tasks" });
const fixed = applyAnalysisFixes(value.content);
await write_artifact({
  change_id,
  kind: "tasks",
  content: fixed,
  overwrite: true,
  expected_etag: etag
});
```

### Error handling

```typescript
try {
  await apply_start({ change_id });
} catch (e) {
  if (e.code === "state.transition_invalid") {
    // 不能 apply（state 不對）；show current state
    const change = await show_change({ change_id });
    console.log(`Current state: ${change.value.state}`);
  } else if (e.code === "lock.not_acquired" && e.retryable) {
    // Engine 已自動 retry、最終失敗；surface to user
  } else {
    throw e;
  }
}
```

---

## Per-Step Notes

- **Step 2b — apply start**: 即使 state 已是 `in_progress` 也呼叫 — `apply_start` ensure actor，會 no-op idempotent 並更新 actor 紀錄。
- **Step 3d — drift run**: 只在滿足 dormancy 條件時跑；drift.run 內含 git diff，避免每次 apply 都呼叫。
- **Step 7 — `[P]` parallel dispatch**: Tool binding hosts 多半支援 parallel tool call。若 host 允許、在 single message 內發多個 `task_done` 為彼此獨立的 tasks；不允許則 fallback sequential。
- **Tier 1 Helpers-only users**: 整個 apply 流程涉及 engine 邏輯（state machine、auto-transition、feedback task marker validation）。Tier 1 user 若不走 SpecLink engine、必須在自己 handler 內**完整重新實作**這些 logic。Tier 2/3 user 直接 invoke 即得到完整行為。
