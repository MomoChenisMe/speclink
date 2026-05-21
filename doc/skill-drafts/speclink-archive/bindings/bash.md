# Bash Binding for speclink-archive

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `project.status` | `speclink status --json` |
| `change.list` | `speclink list --changes [--include-archived] --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `spec.show` | `speclink show spec <capability> --json` |
| `artifact.read` | `speclink artifact read <kind> --change <change-id> [--capability <name>] --json` |
| `archive.run` | `speclink archive <change-id> [--skip-specs] [--no-validate] [--mark-tasks-complete] [--yes] --json` |
| `instructions.get` (kind=commit) | `speclink instructions commit --change <change-id> --json` |
| `review.approve` | `speclink review approve --change <change-id> --reviewer <id> --phase artifact\|code [--note "..."] --json` |

---

## Git Commands (Step 7 commit sub-flow)

These are **NOT** SpecLink operations — use your AI tool's shell facility:

```bash
git status --short
git add <file-1> <file-2> ...
git commit -m "<message>"
```

---

## Common Patterns

### Archive with protective flags

```bash
speclink archive add-auth --skip-specs --yes --json
```

`--yes` bypasses the interactive confirmation prompt that protective flags trigger.

### Error envelope

```json
{
  "ok": false,
  "error": {
    "code": "change.code_review_pending",
    "message": "Change 'add-auth' requires code review approval before archive.",
    "hint": "Run `speclink review approve --change add-auth --reviewer <id> --phase code`.",
    "retryable": false
  }
}
```

### Exit codes

- 0 — success
- 1 — recoverable error
- 2 — unrecoverable (review pending / state invalid / target exists)

---

## Per-Step Notes

- **Step 5 — spec delta diff**: 沒有單一 op 直接 diff delta vs canonical；workflow 需自己 read 兩邊（`artifact.read` + `spec.show`）然後文字比對。可考慮未來加 `archive.preview` 或 `archive.dry_run` op，目前不在 MVP。
- **Step 6 — `--yes` 用法**: 對 `skip_specs` / `no_validate` / `mark_tasks_complete` 任一 flag、CLI 預設 prompt 確認；non-interactive 或 user 已 explicit 決定後可帶 `--yes` 跳。
- **Step 7 — Git commands**: 走 user 的 shell facility（不是 speclink CLI）。`speclink` 不直接控制 git；commit sub-flow 純粹 composing message + 呼叫 user shell。
