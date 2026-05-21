# Bash Binding for speclink-ingest

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `project.status` | `speclink status --json` |
| `discuss.list` | `speclink discuss list [--include-converged] [--linked-change <id>] --json` |
| `discuss.show` | `speclink discuss show <discussion-id> --json` |
| `change.list` | `speclink list --changes --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `instructions.get` | `speclink instructions <kind> --change <change-id> --json` |
| `artifact.read` | `speclink artifact read <kind> --change <change-id> [--capability <name>] --json` |
| `artifact.write` (rewrite) | `cat <<'ARTIFACT_EOF' \| speclink new artifact <kind> --change <change-id> [--capability <name>] --stdin --overwrite [--expected-etag <etag>] --json`<br>`<content>`<br>`ARTIFACT_EOF` |
| `analyze.run` | `speclink analyze <change-id> --json` |
| `validate.run` | `speclink validate <change-id> --strict --json` |

---

## Common Patterns

### Etag-protected rewrite (recommended for ingest)

```bash
# 1. Read current artifact + etag
RESULT=$(speclink artifact read proposal --change add-auth --json)
ETAG=$(echo "$RESULT" | jq -r '.data.etag')

# 2. Generate merged content based on current + new source context

# 3. Write with expected_etag — fails fast if another agent edited mid-stream
cat <<'ARTIFACT_EOF' | speclink new artifact proposal --change add-auth --stdin --overwrite --expected-etag "$ETAG" --json
<merged content>
ARTIFACT_EOF
```

### Blind overwrite (skip etag check)

Only use when you're certain no concurrent editing is possible:

```bash
cat <<'ARTIFACT_EOF' | speclink new artifact tasks --change add-auth --stdin --overwrite --json
<new tasks content>
ARTIFACT_EOF
```

### Error envelope

```json
{
  "ok": false,
  "error": {
    "code": "state.etag_mismatch",
    "message": "Artifact 'proposal' was modified concurrently.",
    "hint": "Re-read the latest artifact and re-apply your changes.",
    "retryable": true,
    "current_etag": "v3.4"
  }
}
```

### Exit codes

- 0 — success
- 1 — recoverable (etag_mismatch, lock retry exhausted but retryable)
- 2 — unrecoverable (validation, state.transition_invalid, change.not_found)

---

## Per-Step Notes

- **Step 1 — `--from-discussion` arg**: 此 skill 接受 `--from-discussion <topic-id>` 並 forward 給 `discuss.show`；CLI flag 純為 skill invocation 方便、不是 op 參數。
- **Step 4 — etag capture**: 從 `change.show` 不會拿到 individual artifact 的 etag；要對每個 artifact 個別 `artifact.read` 拿。建議 ingest 階段一次 read 完所有要改的 artifact + 紀錄 etag、在 Step 5 寫入時帶上。
- **Step 5 — multi-capability spec**: 對 spec 類 artifact、每個 capability 一次 invocation。Engine 不接受一次寫多 capability。
- **Step 7 — analyze-fix loop**: 每次 fix 都要重新 read（拿新 etag）才能 overwrite；否則第二次 fix 就會 etag_mismatch。
