# Tool Binding for speclink-ingest

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations.

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `project.status` | `project_status({})` |
| `discuss.list` | `list_discussions({ include_converged?: boolean, linked_change_id?: string })` |
| `discuss.show` | `show_discussion({ discussion_id })` |
| `change.list` | `list_changes({})` |
| `change.show` | `show_change({ change_id })` |
| `instructions.get` | `get_instructions({ kind, change_id })` |
| `artifact.read` | `read_artifact({ change_id, kind, capability?: string })` |
| `artifact.write` | `write_artifact({ change_id, kind, content, capability?: string, overwrite: true, expected_etag?: string })` |
| `analyze.run` | `analyze_change({ change_id })` |
| `validate.run` | `validate_change({ change_id, strict: true })` |

---

## SDK Convenience

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const { value, etag } = await speclink.artifacts.read({
  changeId: "add-auth",
  kind: "proposal"
});

const merged = mergeWithNewSource(value.content, discussionContent);

await speclink.artifacts.write({
  changeId: "add-auth",
  kind: "proposal",
  content: merged,
  overwrite: true,
  expectedEtag: etag
});
```

---

## Common Patterns

### Etag-protected rewrite (recommended for ingest)

```typescript
// 1. Read current artifact + etag
const { value, etag } = await read_artifact({
  change_id: "add-auth",
  kind: "proposal"
});

// 2. Generate merged content
const merged = mergeIngestSource(value.content, sourceContext);

// 3. Write with expected_etag
try {
  await write_artifact({
    change_id: "add-auth",
    kind: "proposal",
    content: merged,
    overwrite: true,
    expected_etag: etag
  });
} catch (e) {
  if (e.code === "state.etag_mismatch") {
    // Another agent edited mid-stream — re-read and retry
    const fresh = await read_artifact({ change_id: "add-auth", kind: "proposal" });
    const reMerged = mergeIngestSource(fresh.value.content, sourceContext);
    await write_artifact({
      change_id: "add-auth",
      kind: "proposal",
      content: reMerged,
      overwrite: true,
      expected_etag: fresh.etag
    });
  } else {
    throw e;
  }
}
```

### Multi-capability spec batching

For changes with N capability specs to update, loop and invoke `write_artifact` per capability:

```typescript
for (const capability of changedCapabilities) {
  const { value, etag } = await read_artifact({
    change_id,
    kind: "spec",
    capability
  });
  const merged = mergeSpecDelta(value.content, sourceContext, capability);
  await write_artifact({
    change_id,
    kind: "spec",
    capability,
    content: merged,
    overwrite: true,
    expected_etag: etag
  });
}
```

### Error handling

```typescript
try {
  await write_artifact({ change_id, kind, content, overwrite: true });
} catch (e) {
  if (e.code === "state.transition_invalid") {
    // Change archived; ingest not valid
  } else if (e.code === "state.etag_mismatch" && e.retryable) {
    // Re-read + retry
  } else if (e.code === "validation.archive_failed" || e.code.startsWith("validation.")) {
    // Engine validation caught issue in new content
  } else {
    throw e;
  }
}
```

---

## Per-Step Notes

- **Step 1 — `--from-discussion` arg**: Skill invocation 層的 arg；handler 內 forward 給 `show_discussion`。
- **Step 4 — etag capture**: `show_change` 不會回 individual artifact etag；需 per-artifact 呼 `read_artifact` 拿。建議 ingest 開始時一次 read 完所有要改的 artifact + cache etag、寫入時帶上。
- **Step 5 — multi-capability spec**: 每 capability 一次 `write_artifact`；engine 不接受一次寫多 capability。
- **Step 7 — analyze-fix loop**: 每次 fix 都要重新 `read_artifact`（拿新 etag）才能 `write_artifact` with overwrite；否則第二次 fix 就會 etag_mismatch。
- **Tier 1 Helpers-only users**: ingest 涉及大量 engine 邏輯（preservation 規則 / synthetic task 不可改 / validation 自動 trigger）。Tier 1 user 須在 handler 內**完整重新實作**這些 logic 才能符合 SpecLink 語意；Tier 2/3 user 直接 invoke 即得到完整行為。
