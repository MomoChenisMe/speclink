# Tool Binding for speclink-discuss

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations.

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `project.status` | `project_status({})` |
| `discuss.list` | `list_discussions({ include_converged?: boolean, linked_change_id?: string })` |
| `discuss.show` | `show_discussion({ discussion_id })` |
| `discuss.new` | `new_discussion({ topic, linked_change_id?: string, role?: string, background?: string })` |
| `discuss.patch` | `patch_discussion({ discussion_id, section, content, expected_etag?: string })` |
| `discuss.conclude` | `conclude_discussion({ discussion_id, conclusion, expected_etag?: string })` |
| `change.list` | `list_changes({})` |
| `change.show` | `show_change({ change_id })` |
| `config.read` | `read_config({})` |
| `instructions.get` (kind=discuss) | `get_instructions({ kind: "discuss", change_id?: string, role?: string, discussion_id?: string })` |

---

## SDK Convenience

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const { value, etag } = await speclink.discussions.create({
  topic: "Should we use SSE or WebSocket?",
  linkedChangeId: "add-notifications",
  role: "sa"
});

await speclink.discussions.patch({
  discussionId: value.discussion_id,
  section: "rounds",
  content: "### Round 1 — 2026-05-21 — sa\n\nAssumptions listed; user corrected #2..."
});

await speclink.discussions.conclude({
  discussionId: value.discussion_id,
  conclusion: "**Decision**: Use SSE for v1.\n**Capture to**: change add-sse-notifications"
});
```

---

## Common Patterns

### Section content as string parameter

```typescript
await patch_discussion({
  discussion_id: "sse-vs-websocket",
  section: "background",
  content: `The team is debating real-time delivery for the notifications feature.

Current state: HTTP long-polling at 5s interval. Latency complaints from product, but
backend prefers stateless. PM wants sub-second updates for chat.`
});
```

### Append vs replace (engine enforces per section)

Caller does not specify mode — engine decides based on `section`:

| Section | Mode |
|---|---|
| `background` | replace |
| `open_questions` | replace |
| `decisions_made` | append (engine concatenates) |
| `rounds` | append (engine concatenates) |
| `conclusion` | replace (only patchable when state=active; also auto-written by `discuss.conclude`) |

### Concurrency via etag

```typescript
const { value, etag } = await show_discussion({ discussion_id });
// ... interactive thinking ...
try {
  await patch_discussion({
    discussion_id,
    section: "rounds",
    content: "### Round 3 — ...",
    expected_etag: etag
  });
} catch (e) {
  if (e.code === "state.etag_mismatch") {
    // Another agent patched; re-read and merge
    const fresh = await show_discussion({ discussion_id });
    // decide whether to retry or abandon
  }
}
```

### Error handling

```typescript
try {
  await patch_discussion({ discussion_id, section, content });
} catch (e) {
  if (e.code === "discussion.locked") {
    // Already converged; suggest new discussion
  } else if (e.code === "discussion.already_converged") {
    // Same as locked
  } else if (e.code === "role.unknown") {
    const inst = await get_instructions({ kind: "discuss" });
    console.log("Available roles:", inst.availableRoles);
  } else {
    throw e;
  }
}
```

---

## Per-Step Notes

- **Step 9d — flush rhythm**: 每個 round 寫完**立即 `patch_discussion`**；Tool binding host 通常無 batching，每次 call 都到 provider，crash 安全自然成立。
- **Step 10 — conclude**: `conclude_discussion` 是 terminal；後續 `patch_discussion` 對非 `conclusion` section 拋 `discussion.locked`。
- **Tier 1 Helpers-only users**: discuss 涉及 engine 邏輯（section append-only 規則 / converged terminal 規則 / linked_change cascade）。Tier 1 user 須自己重新實作；Tier 2/3 user 直接 invoke。
