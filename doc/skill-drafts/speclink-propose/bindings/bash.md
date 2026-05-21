# Bash Binding for speclink-propose

This document maps the canonical operations referenced in `workflow.md` to their concrete `speclink` CLI invocations. When invoked from a Bash binding host (Claude Code, Codex, shell scripts, CI runners), use the table below to translate each `<op>` reference into a `speclink ...` subprocess call.

All commands include `--json` for machine-readable output. Parse the JSON envelope (see design.md §17.1) — `ok: true` means success; `ok: false` carries `error.code` / `error.message` / `error.retryable`.

---

## Operation Reference

| Operation | Bash invocation |
|---|---|
| `project.status` | `speclink status --json` |
| `discuss.list` | `speclink discuss list [--include-converged] [--linked-change <id>] --json` |
| `discuss.show` | `speclink discuss show <discussion-id> --json` |
| `spec.list` | `speclink list --specs [--updated-since <iso-date>] --json` |
| `spec.show` | `speclink show spec <capability> [--purpose-only] --json` |
| `change.list` | `speclink list --changes [--state <state>] [--include-archived] --json` |
| `change.show` | `speclink show change <change-id> --json` |
| `change.create` | `speclink new change <name> --description "<text>" [--schema <id>] --json` |
| `instructions.get` | `speclink instructions <kind> --change <change-id> --json` |
| `artifact.read` | `speclink artifact read <kind> --change <change-id> [--capability <name>] --json` |
| `artifact.write` (single-instance) | `cat <<'ARTIFACT_EOF' \| speclink new artifact <kind> --change <change-id> --stdin [--overwrite] --json`<br>`<content>`<br>`ARTIFACT_EOF` |
| `artifact.write` (multi-instance, e.g., spec) | `cat <<'ARTIFACT_EOF' \| speclink new artifact spec --change <change-id> --capability <capability> --stdin [--overwrite] --json`<br>`<delta spec content>`<br>`ARTIFACT_EOF` |
| `analyze.run` | `speclink analyze <change-id> --json` |
| `validate.run` | `speclink validate <change-id> [--strict] --json` |
| `schema.list` | `speclink schemas --json` |
| `config.read` | `speclink config show [--key <path>] --json` |

---

## Common Patterns

### Multi-line content via stdin

The `artifact.write` operation accepts content from stdin to avoid shell argv limits. Use heredoc with a quoted delimiter (`'ARTIFACT_EOF'`) to prevent variable expansion inside content:

```bash
cat <<'ARTIFACT_EOF' | speclink new artifact proposal --change add-dark-mode --stdin --json
## Why
Users have requested dark mode for the settings page.

## What Changes
Add a `theme` field to user preferences with `light` / `dark` / `system` values.
ARTIFACT_EOF
```

### Overwrite vs first write

- First write: omit `--overwrite`. If artifact already exists, CLI returns `artifact.already_exists`.
- Intentional rewrite (e.g., post-analyze fix): pass `--overwrite` to allow rewriting the body.
- **NEVER** pass `--force` from this skill. `--force` is reserved for destructive operations (delete / uninstall / etc.) and the AI must not invoke it.

### Error envelope

All commands return JSON like:

```json
{
  "ok": false,
  "error": {
    "code": "change.duplicate_id",
    "message": "Change 'add-dark-mode' already exists.",
    "hint": "Run `speclink show change add-dark-mode` to inspect.",
    "retryable": false
  }
}
```

Inspect `error.code` for programmatic handling; show `error.message` + `error.hint` to the user.

### Exit codes

- 0 — success (`ok: true`)
- 1 — recoverable error (e.g., `lock.not_acquired`, retryable)
- 2 — unrecoverable error (validation, not found, etc.)
- 3+ — see design.md §17.2

---

## Per-Step Notes

- **Step 5 / Step 7 — content writes**: Always use the heredoc-stdin pattern; never pass artifact body as an argv string (shell quoting hell + 128KB argv limit).
- **Step 9 — overwrite on fix**: When the analyze-fix loop rewrites an artifact, pass `--overwrite`; otherwise CLI rejects with `artifact.already_exists`.
- **Step 10 — `--strict` flag**: `validate.run` defaults to non-strict (warnings don't fail). The propose flow uses `--strict` to require zero warnings before transition.
