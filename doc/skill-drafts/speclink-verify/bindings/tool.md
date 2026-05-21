# Tool Binding for speclink-verify

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations.

This skill is **AI-heavy + Tool-light** — most of the work is codebase search + reasoning. Tool calls only provide change context.

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `change.list` | `list_changes({})` |
| `change.show` | `show_change({ change_id })` |
| `instructions.get` (kind=apply) | `get_instructions({ kind: "apply", change_id })` |
| `artifact.read` | `read_artifact({ change_id, kind, capability?: string })` |

---

## Codebase Search (Host-Provided Tools)

Use whatever search / file-read tools the host provides:

- **Copilot SDK / CopilotKit**: typically `search_codebase`, `read_file`, host-provided
- **OpenAI function calling**: developer-defined `grep` / `find` functions
- **LangChain**: `ShellTool` or filesystem tools

These are **NOT** SpecLink operations. Use the host's facilities.

---

## SDK Convenience

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const change = await speclink.changes.get({ changeId: "add-auth" });

if (!["in_progress", "code_reviewing", "archived"].includes(change.value.state)) {
  return { reason: "No implementation to verify yet" };
}

const tasks = await speclink.artifacts.read({ changeId: "add-auth", kind: "tasks" });
const specs = await Promise.all(
  change.value.artifacts
    .filter(a => a.kind === "spec")
    .map(a => speclink.artifacts.read({ changeId: "add-auth", kind: "spec", capability: a.capability }))
);

// AI semantic verification (out of scope for tool layer)
```

---

## Common Patterns

### Iterate spec capabilities

```typescript
const change = await show_change({ change_id });
const specCapabilities = change.value.artifacts
  .filter(a => a.kind === "spec")
  .map(a => a.capability);

const specs = await Promise.all(
  specCapabilities.map(capability =>
    read_artifact({ change_id, kind: "spec", capability })
  )
);
```

### Parse task checkboxes

```typescript
const tasks = await read_artifact({ change_id, kind: "tasks" });
const lines = tasks.value.content.split("\n");
const taskLines = lines.filter(l => /^- \[[ x]\]/.test(l));
const done = taskLines.filter(l => l.startsWith("- [x]")).length;
const total = taskLines.length;
```

### Error handling

```typescript
try {
  const change = await show_change({ change_id });
} catch (e) {
  if (e.code === "change.not_found") {
    const candidates = await list_changes({});
    // Show candidates
  } else {
    throw e;
  }
}
```

---

## Per-Step Notes

- **Step 2 — state precondition**: enforce in tool handler before AI reasoning kicks in.
- **Step 4-6 — host search**: verify's AI workflow needs codebase access. If the tool host has no search facility, skill returns a degraded report explaining what couldn't be checked.
- **Multiple parallel reads**: when reading N capability specs, issue them in parallel via `Promise.all` — engine handles concurrent reads safely.
- **Tier 1 Helpers-only users**: verify is mostly AI logic + a few engine reads. Tier 1 users with their own Provider can implement this skill almost identically — only `read_artifact` / `show_change` / `get_instructions` are engine-dependent.
- **No `verify.run` op**: there is no engine programmatic verify. All semantic work is AI.
