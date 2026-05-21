# Bash Binding for speclink-verify

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations.

This skill is **AI-heavy + CLI-light** — most of the work is codebase grep + reasoning. CLI only provides change context.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `change.list` | `speclink list --changes --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `instructions.get` (kind=apply) | `speclink instructions apply --change <change-id> --json` |
| `artifact.read` | `speclink artifact read <kind> --change <change-id> [--capability <name>] --json` |

---

## Codebase Search (Not SpecLink Operations)

Use your shell facilities (the host's grep / find / rg) — these are NOT SpecLink CLI:

```bash
# Find files referencing a keyword
rg -l "auth_middleware" --type rust

# Find test cases for a scenario
rg "scenario" tests/

# Check file existence
test -f src/auth.rs && echo "exists"
```

---

## Common Patterns

### Load all change context for verification

```bash
CHANGE=add-auth
SHOW=$(speclink show change "$CHANGE" --json)
STATE=$(echo "$SHOW" | jq -r '.data.value.state')

# Verify only makes sense post-apply
case "$STATE" in
  proposing|reviewing|ready)
    echo "No implementation to verify yet"; exit 0;;
esac

# Read all artifacts
PROPOSAL=$(speclink artifact read proposal --change "$CHANGE" --json | jq -r '.data.value.content')
TASKS=$(speclink artifact read tasks --change "$CHANGE" --json | jq -r '.data.value.content')

# For each capability in artifacts[] with kind=spec, read it
echo "$SHOW" | jq -r '.data.value.artifacts[] | select(.kind == "spec") | .capability' | while read -r CAP; do
  SPEC=$(speclink artifact read spec --change "$CHANGE" --capability "$CAP" --json | jq -r '.data.value.content')
  # AI: analyze $SPEC against codebase
done
```

### Parse task checkboxes

```bash
echo "$TASKS" | grep -E "^- \[[ x]\]" | awk '{print $2, $0}' | awk '/^\[x\]/ {done++} /^\[ \]/ {total++; pending++} END {print "Done:", done, "Pending:", pending}'
```

### Error envelope

```json
{
  "ok": false,
  "error": {
    "code": "change.not_found",
    "message": "Change 'add-auth' not found.",
    "retryable": false
  }
}
```

### Exit codes

- 0 — verification ran (regardless of findings count)
- 2 — change not found / not in verifiable state

---

## Per-Step Notes

- **Step 2 — state precondition**: verify only makes sense for changes with implementation (state `in_progress` / `code_reviewing` / `archived`).
- **Step 4-6 — file-search**: the bulk of verify is codebase grep + reasoning. SpecLink CLI only provides artifact text. Use shell `rg` / `grep` / `find`.
- **Spec capability iteration**: `change.show` returns `artifacts[]` with each entry's `kind` + (for `spec`) `capability`. Loop those to issue per-capability `artifact.read` calls.
- **No `verify.run` CLI op**: SpecLink does NOT have a `speclink verify` command. Verification is intentionally AI-only — engine has nothing to add beyond providing artifact text.
