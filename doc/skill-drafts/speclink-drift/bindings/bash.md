# Bash Binding for speclink-drift

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `change.list` | `speclink list --changes --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `drift.run` | `speclink drift <change-id> --json` |

---

## Common Patterns

### Parse drift output

```bash
RESULT=$(speclink drift add-auth --json)
SEVERITY=$(echo "$RESULT" | jq -r '.data.severity')
SCORE=$(echo "$RESULT" | jq -r '.data.total_score')
RECOMMENDATION=$(echo "$RESULT" | jq -r '.data.primary_recommendation')

case "$SEVERITY" in
  light)  echo "Minor drift; safe to apply" ;;
  medium) echo "Refresh recommended before apply" ;;
  heavy)  echo "Plan likely outdated; restart" ;;
esac
```

### List broken anchors

```bash
echo "$RESULT" | jq -r '.data.broken_anchors[] | "\(.kind): \(.reference) — \(.reason)"'
```

### List task collisions

```bash
echo "$RESULT" | jq -r '.data.tasks_blocked_external[] | "BLOCKED: \(.task) — \(.commit_sha) modified \(.file)"'
echo "$RESULT" | jq -r '.data.tasks_maybe_resolved[] | "MAYBE DONE: \(.task) — commit \(.commit_sha): \(.subject)"'
```

### Error envelope

```json
{
  "ok": false,
  "error": {
    "code": "drift.anchor_cap_exceeded",
    "message": "Anchor cap (50) exceeded; partial drift report.",
    "hint": "Drift checks limited; results below show first 50 anchors.",
    "retryable": false
  }
}
```

### Exit codes

- 0 — drift report generated (regardless of severity)
- 1 — recoverable (anchor cap hit, partial report)
- 2 — unrecoverable (change.not_found, drift.failed)

---

## Per-Step Notes

- **Step 2 — programmatic only**: `speclink drift --json` is engine-only. The AI layer (Step 3 presentation, Step 4 interactive) is the skill's value-add.
- **Dual invocation path**: `speclink-apply` Step 3d may invoke `speclink drift --json` directly inline (not via this skill). Both paths use the same CLI.
- **Anchor cap**: engine caps at 50 by `ANCHOR_CAP`. If `drift.anchor_cap_exceeded` warning fires, present partial findings honestly.
- **Fork context**: when running as a Claude Code fork, do not use interactive prompt — return findings to main thread.
