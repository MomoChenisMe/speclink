# Tool Binding for speclink-archive

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations.

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `project.status` | `project_status({})` |
| `change.list` | `list_changes({ include_archived?: boolean })` |
| `change.show` | `show_change({ change_id })` |
| `spec.show` | `show_spec({ capability })` |
| `artifact.read` | `read_artifact({ change_id, kind, capability?: string })` |
| `archive.run` | `archive_change({ change_id, skip_specs?: boolean, no_validate?: boolean, mark_tasks_complete?: boolean, yes?: boolean })` |
| `instructions.get` (kind=commit) | `get_instructions({ kind: "commit", change_id })` |
| `review.approve` | `review_approve({ change_id, reviewer, phase, note?: string })` |

---

## Git Commands (Step 7 commit sub-flow)

Tool binding hosts vary in how they expose shell access. Typical patterns:

- **Copilot SDK / CopilotKit**: host-provided `run_command` tool
- **OpenAI function calling**: developer-defined `run_shell` function
- **LangChain**: `ShellTool`

These are **NOT** SpecLink operations. Use whichever shell facility the host provides:

```typescript
await run_command({ cmd: "git status --short" });
await run_command({ cmd: `git add ${files.join(" ")}` });
await run_command({ cmd: `git commit -m ${JSON.stringify(message)}` });
```

---

## SDK Convenience

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const change = await speclink.changes.get({ changeId: "add-auth" });
const archiveResult = await speclink.archive.run({
  changeId: "add-auth",
  skipSpecs: false,
  markTasksComplete: false
});
```

---

## Common Patterns

### Archive with protective flags

```typescript
await archive_change({
  change_id: "add-auth",
  skip_specs: true,
  yes: true   // 跳過 interactive confirm（tool binding 通常 non-interactive）
});
```

### Error handling

```typescript
try {
  await archive_change({ change_id });
} catch (e) {
  if (e.code === "change.code_review_pending") {
    // 提示 reviewer approve
    console.log("Code review required:", e.hint);
  } else if (e.code === "archive.target_exists") {
    // Date+name collision
  } else if (e.code === "validation.archive_failed") {
    // Delta spec validation 失敗
  } else {
    throw e;
  }
}
```

---

## Per-Step Notes

- **Step 5 — spec delta diff**: 沒有單一 op 直接 diff delta vs canonical；需自行 read 兩邊（`read_artifact` + `show_spec`）然後文字比對。
- **Step 6 — `yes: true`**: Tool binding 通常 non-interactive、預設帶 `yes: true` 避免 hang。
- **Step 7 — Git commands**: Tool binding 環境（webapp / 伺服器）可能根本沒 git working tree；commit sub-flow 應該先檢測 host 是否支援 shell + 是否有 git context，不支援時直接 skip 並提示 user 手動 commit。
- **Tier 1 Helpers-only users**: archive 涉及大量 engine 邏輯（spec delta merge / state transition / audit events）。Tier 1 user 若不走 SpecLink engine、必須在自己 handler 內**完整重新實作**這些 logic — 否則 spec delta 就不會 merge、archive 變只是「flip 一個 flag」。Tier 2/3 user 直接 invoke 即得到完整行為。
