# Tool Binding for speclink-drift

This document maps the canonical operations referenced in `workflow.md` to their typed Tool call invocations.

---

## Operation Reference

| Operation | Tool call |
|---|---|
| `change.list` | `list_changes({})` |
| `change.show` | `show_change({ change_id })` |
| `drift.run` | `drift_change({ change_id })` |

---

## SDK Convenience

```typescript
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ /* config */ });

const drift = await speclink.drift.run({ changeId: "add-auth" });

if (drift.severity === "heavy") {
  // Recommend archive + restart
} else if (drift.severity === "medium") {
  // Recommend /speclink-ingest
} else {
  // Light — safe to apply
}
```

---

## Common Patterns

### Branch on severity

```typescript
const drift = await drift_change({ change_id });

const presentation = {
  light: {
    conclusion: "Minor drift — safe to proceed with apply.",
    recommended: "Continue with /speclink-apply"
  },
  medium: {
    conclusion: "Plan may be stale — refresh before applying.",
    recommended: "Refresh with /speclink-ingest"
  },
  heavy: {
    conclusion: "Plan likely unsuitable — restart recommended.",
    recommended: drift.primary_recommendation
  }
}[drift.severity];
```

### Error handling

```typescript
try {
  const drift = await drift_change({ change_id });
} catch (e) {
  if (e.code === "drift.failed") {
    // Engine internal error
  } else if (e.code === "drift.anchor_cap_exceeded") {
    // Partial results; show what we have
  } else if (e.code === "change.not_found") {
    const candidates = await list_changes({});
  } else {
    throw e;
  }
}
```

### Used inline from apply skill

`speclink-apply` may call `drift_change` directly:

```typescript
// Inside speclink-apply Step 3d
const change = await show_change({ change_id });
const createdDaysAgo = (Date.now() - new Date(change.value.created_at)) / 86400000;
const lastCommitDaysAgo = await getLastCommitDays(change_id); // host git facility

if (createdDaysAgo > 5 && lastCommitDaysAgo > 3) {
  const drift = await drift_change({ change_id });
  // present inline; AskUserQuestion for next step
}
```

---

## Per-Step Notes

- **Step 2 — drift_change op**: programmatic engine call. Returns full structured drift result (severity / dimensions / anchors / task collisions / recommendation).
- **Step 3 — presentation**: AI value-add — translate engine result into conclusion-first markdown.
- **Step 4 — AskUserQuestion**: not invokable inside a fork — return options to the main thread.
- **Dual path**: tool hosts can `drift_change` directly (from `speclink-apply` Step 3d) OR via this skill. Same op, different presentation context.
- **Tier 1 Helpers-only users**: drift is engine logic (anchor resolution against codebase + task collision via git log). Tier 1 user has no engine — must implement equivalent or skip drift entirely.
