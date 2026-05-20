# speclink

SpecLink — Spec-Driven Development workflow CLI for AI agents.

This repository hosts the Cargo workspace with four crates:

| crate            | role                                                             |
| ---------------- | ---------------------------------------------------------------- |
| `cli`            | `speclink` binary, command surface (clap)                        |
| `runtime`        | workflow orchestration over the `Provider` trait                 |
| `provider`       | `Provider` async trait, shared types, config + resolution        |
| `provider-local` | local filesystem implementation of `Provider` (no remote calls)  |

## Status

Pre-alpha. The local-first AI workflow surface is built up vertically through
sequential changes under `openspec/changes/`. The CLI currently supports
`propose create`, multi-kind `artifact write`, and `status` against the local
provider.

## CLI usage

All commands accept the machine-interface flags `--json`, `--no-color`,
`--quiet` (and a per-command `--stdin` where documented). Exit codes follow the
table pinned in `cli-machine-interface`: `0` success, `1` general failure,
`2` user-input error, `5` provider unavailable, `6` auth required.

### Create a proposal

```sh
speclink propose create --change add-feature --summary "one-line why" --json
```

Output (one-line JSON envelope, pretty-printed here for clarity):

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "state": "proposed",
    "artifactPath": ".speclink/changes/add-feature/proposal.md",
    "mode": "local"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

### Write design / tasks artifact

```sh
echo "design content" | speclink artifact write design \
  --change add-feature --stdin --json
```

```sh
echo "- [ ] task body" | speclink artifact write tasks \
  --change add-feature --stdin --json
```

The `data` payload contains `artifactId`, `kind`, and the POSIX-style `path`
(`forward slashes on all platforms`).

### Write a spec artifact (per capability)

```sh
cat <<'EOF' | speclink artifact write spec \
  --change add-feature --capability user-auth --stdin --json
## ADDED Requirements

### Requirement: ...
EOF
```

Output:

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "artifactId": "spec:user-auth",
    "kind": "spec",
    "path": ".speclink/changes/add-feature/specs/user-auth/spec.md",
    "mode": "local"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

### Observe change status

```sh
speclink status --change add-feature --json
```

Output:

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "state": "proposed",
    "artifacts": [
      {
        "id": "proposal",
        "kind": "proposal",
        "path": ".speclink/changes/add-feature/proposal.md",
        "status": "done",
        "required": true,
        "dependencies": []
      },
      {
        "id": "design",
        "kind": "design",
        "path": ".speclink/changes/add-feature/design.md",
        "status": "done",
        "required": false,
        "dependencies": ["proposal"]
      },
      {
        "id": "tasks",
        "kind": "tasks",
        "path": ".speclink/changes/add-feature/tasks.md",
        "status": "missing",
        "required": false,
        "dependencies": ["proposal", "spec"]
      },
      {
        "id": "spec:user-auth",
        "kind": "spec",
        "path": ".speclink/changes/add-feature/specs/user-auth/spec.md",
        "status": "done",
        "required": true,
        "dependencies": ["proposal"]
      }
    ]
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

`status` is side-effect-free: it never creates or modifies any files under
`.speclink/`. Spec entries appear after the three fixed-name kinds, sorted
ascending by capability name.

## License

Dual-licensed under MIT OR Apache-2.0. See `LICENSE` for details.
