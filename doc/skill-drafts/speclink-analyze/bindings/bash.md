# Bash Binding for speclink-analyze

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `change.list` | `speclink list --changes --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `analyze.run` | `speclink analyze <change-id> --json` |
| `artifact.read` | `speclink artifact read <kind> --change <change-id> [--capability <name>] --json` |
| `spec.show` | `speclink show spec <capability> --json` |

---

## Common Patterns

### Parse analyze output

```bash
RESULT=$(speclink analyze add-auth --json)
DIMS=$(echo "$RESULT" | jq '.data.dimensions')
FINDINGS=$(echo "$RESULT" | jq '.data.findings')
CRITICAL=$(echo "$RESULT" | jq '[.data.findings[] | select(.severity == "critical")] | length')
```

### Filter findings by severity

```bash
echo "$RESULT" | jq '.data.findings[] | select(.severity == "critical" or .severity == "warning")'
```

### Error envelope

```json
{
  "ok": false,
  "error": {
    "code": "change.not_found",
    "message": "Change 'add-auth' not found.",
    "hint": "Run `speclink list --changes --json` to see candidates.",
    "retryable": false
  }
}
```

### Exit codes

- 0 — success (analyze ran; findings may exist but are not errors)
- 1 — recoverable (lock retry exhausted but retryable, etag mismatch on artifact.read mid-flow)
- 2 — unrecoverable (change.not_found, analyze.failed)

---

## Per-Step Notes

- **Step 2 — programmatic analysis**: `speclink analyze --json` runs the engine's static analysis only. The AI semantic layer (Step 4) is done by reading artifacts via `speclink artifact read`.
- **Step 4 — artifact reads**: read all artifacts ONCE into memory; don't re-read mid-analysis. Engine guarantees `analyze.run` snapshot consistency for the duration of the call but artifact.read is a separate snapshot.
- **Passive trigger detection**: Bash hosts can poll `speclink show change <id> --json` and check `.data.value.is_complete` field to detect the trigger condition.
- **Fork context**: when running as a Claude Code fork, do not invoke `speclink list` interactively — script-mode only with `--json`.
