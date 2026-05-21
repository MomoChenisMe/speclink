# Tool Binding for speclink-propose

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations. When invoked from a Tool binding host (GitHub Copilot SDK, CopilotKit, OpenAI function calling, LangChain tools), use the table below to translate each `<op>` reference into the corresponding tool function call.

All tool calls return a typed `Versioned<T>` or operation-specific result (see `@speclink/client` SDK types). Errors are thrown as `SpecLinkError` with `code` / `message` / `retryable` fields matching the CLI JSON envelope (design.md §17.1).

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `project.status` | `project_status({})` |
| `discuss.list` | `list_discussions({ include_converged?: boolean, linked_change_id?: string })` |
| `discuss.show` | `show_discussion({ discussion_id })` |
| `spec.list` | `list_specs({ updated_since?: string })` |
| `spec.show` | `show_spec({ capability })` |
| `change.list` | `list_changes({ state?: string \| string[], include_archived?: boolean })` |
| `change.show` | `show_change({ change_id })` |
| `change.create` | `new_change({ name, description, schema_id?: string })` |
| `instructions.get` | `get_instructions({ kind, change_id })` |
| `artifact.read` | `read_artifact({ change_id, kind, capability?: string })` |
| `artifact.write` | `write_artifact({ change_id, kind, content, capability?: string, overwrite?: boolean, expected_etag?: string })` |
| `analyze.run` | `analyze_change({ change_id })` |
| `validate.run` | `validate_change({ change_id, strict?: boolean })` |
| `schema.list` | `list_schemas({})` |
| `config.read` | `read_config({})` |

---

## SDK Convenience (when using `@speclink/client` directly)

The same operations are exposed as typed SDK methods. Tool host bindings under the hood call these:

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const status = await speclink.status();
const change = await speclink.changes.create({ name, description });
const artifact = await speclink.artifacts.write({ change_id, kind, content });
```

For Tier 1 Helpers-only (no Provider), import descriptors instead:

```typescript
import { schemas, makeCopilotSdkToolDescriptors } from "@speclink/client/helpers";

const tools = makeCopilotSdkToolDescriptors({ filter: ["change.create", "artifact.write"] });
// User writes handler that calls their own backend
```

See design.md §22.1 for Integration Tier details.

---

## Common Patterns

### Content as string parameter

Tool bindings pass artifact content as a plain string parameter — no stdin / heredoc gymnastics:

```typescript
await write_artifact({
  change_id: "add-dark-mode",
  kind: "proposal",
  content: `## Why\nUsers have requested dark mode...\n\n## What Changes\n...`
});
```

### Overwrite vs first write

- First write: omit `overwrite` (default `false`). If artifact already exists, throws `artifact.already_exists`.
- Intentional rewrite (e.g., post-analyze fix): pass `overwrite: true`.
- **NEVER** pass `force: true` from this skill. `force` is reserved for destructive operations and the AI must not invoke it.

### Error handling

Catch `SpecLinkError` and inspect `code`:

```typescript
try {
  const change = await new_change({ name, description });
} catch (e) {
  if (e.code === "change.duplicate_id") {
    // Suggest continuing existing change or pick different name
  } else if (e.code === "lock.not_acquired" && e.retryable) {
    // Engine already retried; surface to user
  } else {
    throw e;
  }
}
```

### Optimistic concurrency (etag)

For read-modify-write flows (e.g., re-writing artifact after analyze fix), capture `etag` from read and pass to write:

```typescript
const { value, etag } = await read_artifact({ change_id, kind: "proposal" });
const updated = applyFix(value.content);
await write_artifact({
  change_id,
  kind: "proposal",
  content: updated,
  overwrite: true,
  expected_etag: etag  // If artifact was modified concurrently, throws state.etag_mismatch
});
```

If `state.etag_mismatch` is thrown, re-read and retry the merge (engine's `read-then-retry` semantics, design.md §12.5.4).

---

## Per-Step Notes

- **Step 5 / Step 7 — content writes**: Pass full content as `content` parameter; no encoding tricks needed.
- **Step 9 — overwrite on fix**: When the analyze-fix loop rewrites an artifact, pass `overwrite: true`; otherwise throws `artifact.already_exists`.
- **Step 10 — strict validation**: `validate_change({ change_id, strict: true })` requires zero warnings before transition. Default is non-strict.
- **Tier 1 Helpers-only users** (custom handler): the workflow above describes the canonical operation semantics; your handler must enforce them. The engine logic (state machine / lock / audit) is **NOT** automatically applied in Tier 1 — that's the developer's responsibility.
