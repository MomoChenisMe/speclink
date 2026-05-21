# Bash Binding for speclink-discuss

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `project.status` | `speclink status --json` |
| `discuss.list` | `speclink discuss list [--include-converged] [--linked-change <id>] --json` |
| `discuss.show` | `speclink discuss show <discussion-id> --json` |
| `discuss.new` | `speclink discuss new "<topic>" [--linked-change <id>] [--role <role-id>] [--background-stdin] --json` |
| `discuss.patch` | `speclink discuss patch <discussion-id> --section <name> --stdin [--expected-etag <etag>] --json` |
| `discuss.conclude` | `speclink discuss conclude <discussion-id> --conclusion-stdin [--expected-etag <etag>] --json` |
| `change.list` | `speclink list --changes --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `config.read` | `speclink config show [--key <path>] --json` |
| `instructions.get` (kind=discuss) | `speclink instructions discuss [--change <id>] [--role <role-id>] [--discussion <id>] --json` |

---

## Common Patterns

### Background / round / decision content via stdin

`discuss.patch` takes section content via stdin to avoid shell argv limits:

```bash
cat <<'PATCH_EOF' | speclink discuss patch sse-vs-websocket --section background --stdin --json
The team is debating real-time delivery for the notifications feature.

Current state: HTTP long-polling at 5s interval. Latency complaints from product, but
backend prefers stateless. PM wants sub-second updates for chat.
PATCH_EOF
```

### Append-only section

`decisions_made` and `rounds` sections are append-only — engine adds to the end of the existing section. Just pass the new entry:

```bash
echo "[2026-05-21 sa] We'll use SSE; bidir not needed for v1." | \
  speclink discuss patch sse-vs-websocket --section decisions_made --stdin --json
```

### Replace section

`background`, `open_questions`, `conclusion` are replace mode — pass the full new section body:

```bash
cat <<'PATCH_EOF' | speclink discuss patch sse-vs-websocket --section open_questions --stdin --json
- ~~Q1: which transport?~~ (decided: SSE)
- Q2: heartbeat interval?
- Q3: fallback for non-SSE clients?
PATCH_EOF
```

### Conclude

```bash
cat <<'CONCLUDE_EOF' | speclink discuss conclude sse-vs-websocket --conclusion-stdin --json
**Decision**: Use SSE for v1.
**Rationale**: Notifications are server-to-client only; SSE meets sub-second latency without the complexity of WebSocket connection lifecycle handling.
**Capture to**: change add-sse-notifications
CONCLUDE_EOF
```

### Error envelope

```json
{
  "ok": false,
  "error": {
    "code": "discussion.locked",
    "message": "Discussion 'sse-vs-websocket' has converged; only 'conclusion' section is patchable.",
    "retryable": false
  }
}
```

---

## Per-Step Notes

- **Step 9d — rounds patch**: 每個 round 寫完**立即 flush**（不要等到 skill end）；engine 強制 round-boundary invariant，但 caller 主動及早 flush 才能達成 crash-safety。
- **Step 10 — conclude**: `discuss.conclude` 是 terminal action；之後 `discuss.patch` 對非 `conclusion` section 一律返回 `discussion.locked`。
- **Pause mid-flight**: 不需要特殊 op — `active` discussion 可以無限期擱置；`/speclink-discuss <topic-id>` 重新拉起即可。徹底放棄用 `speclink discuss delete <id> --force`。
