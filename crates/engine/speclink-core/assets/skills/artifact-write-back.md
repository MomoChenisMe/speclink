**Write-back: changing an artifact that already exists.** Every fix and update this skill makes to an artifact that already exists is a write-back. Read the artifact's full text with `speclink artifact cat <artifact-id> --change "<name>"` — a delta spec is `specs/<capability>`, which the `deltaSpecs` field of `speclink show <name> --json` lists as `<capability>/spec.md` — make the change on that full text, and write the whole result back with `--stdin --force`:

```bash
speclink new artifact <artifact-id> --change "<name>" --stdin --force <<'ARTIFACT_EOF'
<full updated content>
ARTIFACT_EOF
```

A delta spec is written back with `speclink new artifact spec <capability> --change "<name>" --stdin --force`. The same verbs work when the documents live in a remote store. When `new artifact` refuses a write-back with an error (for example, a task list whose tasks are all checked), stop and tell the user the message — do not edit the file instead.
