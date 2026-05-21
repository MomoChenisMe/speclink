# Bash Binding for speclink-apply

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations. All commands include `--json` for machine-readable output.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `project.status` | `speclink status --json` |
| `change.list` | `speclink list --changes --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `apply.start` | `speclink apply start <change-id> [--actor <id>] --json` |
| `apply.pause` | `speclink apply pause <change-id> --json` |
| `instructions.get` | `speclink instructions <kind> --change <change-id> --json` |
| `artifact.read` | `speclink artifact read <kind> --change <change-id> [--capability <name>] --json` |
| `artifact.write` (rewrite) | `cat <<'ARTIFACT_EOF' \| speclink new artifact <kind> --change <change-id> --stdin --overwrite --json`<br>`<content>`<br>`ARTIFACT_EOF` |
| `analyze.run` | `speclink analyze <change-id> --json` |
| `drift.run` | `speclink drift <change-id> --json` |
| `task.done` | `speclink task done <task-id> --change <change-id> [--touched-files <path>,...] --json` |
| `review.approve` | `speclink review approve --change <change-id> --reviewer <id> --phase artifact\|code [--note "..."] --json` |
| `discuss.show` | `speclink discuss show <discussion-id> --json` |

---

## Common Patterns

### Task completion with touched files

When a task modifies source files, pass them via `--touched-files` for engine drift detection:

```bash
speclink task done 1.2 --change add-auth --touched-files src/auth.rs,src/middleware.rs --json
```

### Artifact rewrite for analyze-fix loop

```bash
cat <<'ARTIFACT_EOF' | speclink new artifact tasks --change add-auth --stdin --overwrite --json
<updated tasks markdown>
ARTIFACT_EOF
```

### Error envelope

All commands return JSON like:

```json
{
  "ok": false,
  "error": {
    "code": "state.transition_invalid",
    "message": "Change 'add-auth' is in state 'archived'; cannot apply.",
    "hint": "Check `speclink show change add-auth` for current state.",
    "retryable": false
  }
}
```

Inspect `error.code` for programmatic handling; show `error.message` + `error.hint` to the user.

### Exit codes

- 0 — success
- 1 — recoverable error (`lock.not_acquired`, `state.etag_mismatch`)
- 2 — unrecoverable error (validation, not found, etc.)
- 詳見 design.md §17.2

---

## Per-Step Notes

- **Step 2b — apply start**: 即使 state 已是 `in_progress`、也呼叫一次 `apply.start` 以 ensure actor 紀錄正確（CLI return success no-op）。
- **Step 3d — drift run**: 只有滿足 dormancy 條件時才呼叫；不要每次都跑（drift.run 可能跑 git diff、開銷有感）。
- **Step 7 — task.done**: 完成最後一個 task 時、回應的 `auto_transition` 欄位非 null；依此決定下一步告知 user 哪個 skill。
- **Step 7 — feedback_task_check**: 若 `tasks.feedback_task_removed` 返回、`data.value.feedback_task_check.re_appended = true`；告知 user engine 已 re-append、需先處理該 feedback task。
