# Tool Binding for speclink-analyze

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations.

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `change.list` | `list_changes({})` |
| `change.show` | `show_change({ change_id })` |
| `analyze.run` | `analyze_change({ change_id })` |
| `artifact.read` | `read_artifact({ change_id, kind, capability?: string })` |
| `spec.show` | `show_spec({ capability })` |

---

## SDK Convenience

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const result = await speclink.analyze.run({ changeId: "add-auth" });
const critical = result.findings.filter(f => f.severity === "critical");

if (critical.length > 0) {
  // Fetch artifacts for AI semantic analysis
  const proposal = await speclink.artifacts.read({ changeId: "add-auth", kind: "proposal" });
  // ... AI semantic check ...
}
```

---

## Common Patterns

### Programmatic + semantic combined

```typescript
const analysis = await analyze_change({ change_id });

// Programmatic findings (from engine)
const programmatic = analysis.findings;

// Semantic layer: read artifacts and AI-check
const proposal = await read_artifact({ change_id, kind: "proposal" });
const design = await read_artifact({ change_id, kind: "design" });
const tasks = await read_artifact({ change_id, kind: "tasks" });

// (AI reasoning here — out of scope for tool layer)
```

### Error handling

```typescript
try {
  const result = await analyze_change({ change_id });
} catch (e) {
  if (e.code === "change.not_found") {
    const candidates = await list_changes({});
    // Surface candidate list to user
  } else if (e.code === "analyze.failed") {
    // Engine analysis error; surface details
  } else {
    throw e;
  }
}
```

### Passive trigger detection

```typescript
const change = await show_change({ change_id });
if (change.value.is_complete && ["reviewing", "ready"].includes(change.value.state)) {
  // Trigger /speclink-analyze
}
```

---

## Per-Step Notes

- **Step 2 — analyze_change**: returns structured `{ dimensions, findings, artifacts_analyzed, artifacts_missing }`. Engine handles concurrent reads — no lock needed.
- **Step 4 — multiple artifact reads**: typically 3-5 reads (proposal / design / tasks / specs). Issue them in parallel where the tool host supports it; engine handles concurrent reads safely.
- **Tier 1 Helpers-only users**: analyze is engine logic — Tier 1 users (no Provider) must reimplement static analysis themselves (or skip this skill). Tier 2/3 users get full engine analysis via `analyze_change`.
- **Fork context**: in Copilot SDK fork mode, do not use AskUserQuestion. Return analysis result to main thread.
