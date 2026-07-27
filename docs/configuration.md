# Configuration

> **Status:** This document describes the currently implemented local-workspace configuration surface. In remote mode, workflow policy is authoritative in the versioned Store and local overrides must not silently replace team policy; see [the platform architecture blueprint](platform-architecture.zh-TW.md#48-workflow-policy-的歸屬).

Speclink's configuration is split across two files and one directory, each with a distinct ownership rule:

| Location | Owns | Travels with |
|---|---|---|
| `openspec/config.yaml` | Workflow policy: `locale`, `spec_locale`, `tdd`, `audit` — plus `schema`, `context`, `rules` | The **spec store** (wherever your spec documents live) |
| `.speclink.yaml` | Workspace binding: `tools` (which AI harnesses get generated files), `spec_dir` (where the spec store is) | The **repo checkout** |
| `.speclink/` | Host work data: touched-file records, archive snapshots, generated-tool footprints | The **machine** (gitignored) |

The rule of thumb for "where does this setting go":

- **Policy follows the store.** Anything that changes what the workflow produces (artifact language, spec language, TDD discipline, audit discipline) lives in `openspec/config.yaml`. Whoever reads the specs — locally or through a remote store — sees the same single truth.
- **Binding follows the repo.** `.speclink.yaml` only says how *this checkout* connects to the store and which AI tools are wired up. It never carries policy.
- **Personal differences follow the environment.** A `SPECLINK_*` environment variable overrides everything for one shell or one CI job, without touching any file.

## Resolution order

Effective policy values are resolved through four layers; the first layer where a key is present wins:

| Priority | Layer | Notes |
|---|---|---|
| 1 (highest) | `SPECLINK_LOCALE` / `SPECLINK_SPEC_LOCALE` / `SPECLINK_TDD` / `SPECLINK_AUDIT` | Boolean variables accept only `true` / `false` (case-insensitive). Any other value — `yes`, `1`, empty — is treated as **unset** and falls through to the next layer. |
| 2 | Legacy keys in `.speclink.yaml` | Deprecated compatibility layer (see below). A key that is *present* wins, even with value `false`. |
| 3 | `openspec/config.yaml` | The canonical home. |
| 4 (lowest) | Built-in defaults | `locale` unset = English, `tdd` = false, `audit` = false. |

## Deprecated: policy keys in `.speclink.yaml`

Older projects carry `locale`, `spec_locale`, `tdd`, or `audit` in `.speclink.yaml`. They still work — their values keep winning over `openspec/config.yaml`, so nothing changes silently — but every command prints one line to stderr:

```
speclink: warning: deprecated policy keys in .speclink.yaml: tdd, audit (move them to openspec/config.yaml)
```

The line has a fixed prefix (`speclink: warning:`), appears exactly once per invocation, and never touches stdout — `--json` output is unaffected.

### Migrating from the old layout

1. Move the four policy keys (whichever you use) from `.speclink.yaml` into `openspec/config.yaml`, keeping the same values.
2. Delete them from `.speclink.yaml`, leaving only `tools` and (if customized) `spec_dir`.
3. Run any command — the warning is gone, and effective values are unchanged (the canonical layer now supplies what the legacy layer did).

Before → after:

```yaml
# .speclink.yaml (before)          # .speclink.yaml (after)
locale: tw                         tools:
tdd: true                            - claude
tools:
  - claude                         # openspec/config.yaml (after)
                                   schema: spec-driven
# openspec/config.yaml (before)    locale: tw
schema: spec-driven                tdd: true
```

## Managing `openspec/config.yaml` with the `workflow-config` verb

`speclink workflow-config` manages the **workflow policy file**; `speclink config` manages the project-independent global key-value store. The two are unrelated.

| Subcommand | What it does |
|---|---|
| `show [--json]` | Prints the four policy fields, `context` (line count) and `rules` (entries per section). Shows the **canonical values** — environment variables and deprecated keys are NOT applied (resolving effective values is `speclink instructions`' job). The `--json` payload is camelCase: `locale`, `specLocale`, `tdd`, `audit`, `context`, `rules`; unset fields are `null`, unset toggles are `false`. |
| `set <key> <value>` | Writes one of `locale`, `spec_locale`, `tdd`, `audit`. Any other key exits non-zero; `tdd`/`audit` accept only `true`/`false`. Setting `false` (or a locale to an empty string) **removes the key**, keeping unset-means-default intact. |
| `context --stdin` | Sets `context` to the full stdin text; whitespace-only input removes the key. |
| `rules <artifact> --stdin` | Replaces that artifact's rule section wholesale (one entry per line, blank lines ignored); empty stdin removes the section. `artifact` must be an artifact id of the active schema — an unknown id exits non-zero. |

All three write subcommands support `--dry-run`: the unified diff goes to stdout, nothing is written, exit code 0. The preview and the real write share the exact same rewrite path, so the diff IS what would land.

```bash
speclink workflow-config set tdd true --dry-run   # look first
speclink workflow-config set tdd true             # then write
cat CONTEXT.md | speclink workflow-config context --stdin
```

**fs and remote mode.** The mode comes from the existing binding: fs mode reads and writes `openspec/config.yaml` directly; remote mode reads the server's config document with its revision, applies the same rewrite, and writes back guarded by that revision. The revision never appears in the command interface — a concurrent write by someone else exits non-zero asking you to re-run, and never overwrites their write. Being offline or losing authentication also exits non-zero; nothing is spooled or queued.

**Known trade-off: template comments are lost.** Writes are read-modify-write (parse the whole document, change the target key, write it all back). Every other key and value is preserved, but the original file's template comments do not survive the rewrite — the same trade-off the desktop settings page makes. Run `--dry-run` first to see the diff before deciding. An unparseable document always fails closed: both reads and writes exit non-zero (rewriting a broken file would destroy its content).

The built-in `speclink-config` skill is built on this verb — it composes `context` and `rules` from a fixed set of codebase sources and always presents a diff for approval before writing.

## Custom tool descriptors

The `tools` list accepts built-in names (`claude`, `codex`) and custom descriptor objects for any other AI harness:

```yaml
tools:
  - claude
  - name: wad-harness
    skills_dir: .wad/skills
    instructions_file: WAD.md
    invocation: tool-call
```

| Field | Required | Rules |
|---|---|---|
| `name` | yes | kebab-case, 2–50 chars of `[a-z0-9-]`; must not collide with a built-in tool name |
| `skills_dir` | yes | project-root-relative path; must not escape the project root |
| `instructions_file` | yes | project-root-relative path; must not escape the project root |
| `invocation` | no | `cli` (default) or `tool-call` — decides how generated text tells the harness to run speclink verbs: "run `speclink <verb>`" vs "call the speclink tool with an argv array" |

A validation failure (name conflict, bad casing, path escape, unknown invocation) makes the command exit non-zero with a single-line error naming the field.

Descriptors share the full lifecycle of built-in tools:

- **Generate** — `speclink init` / `speclink update` writes `speclink-*/SKILL.md` skills under `skills_dir` and upserts the `SPECLINK` marker block into `instructions_file`.
- **Sync** — `speclink update` regenerates everything for descriptors still on the list.
- **Clean up** — remove the descriptor from `tools` and the next `speclink update` deletes its `speclink-*` skill directories (dropping directories left empty), strips the marker block from `instructions_file`, and deletes that file if nothing else remains in it.

Descriptor-generated content uses the **neutral rendering**: no `/speclink-` slash prefixes, no plan-mode references, and verb wording chosen by `invocation`. Built-in claude and codex output is unaffected.

## Reference: all keys

### `openspec/config.yaml`

| Key | Default | Meaning |
|---|---|---|
| `schema` | `spec-driven` | Workflow schema for new changes |
| `locale` | English | Language for AI-generated artifacts (`tw`, `ja`, …) |
| `spec_locale` | English | Language for spec files; `auto` follows `locale` |
| `tdd` | `false` | Ask implementers to follow Red-Green-Refactor discipline |
| `audit` | `false` | Ask implementers to apply the sharp-edges audit discipline |
| `context` | — | Project context shown to AI when creating artifacts |
| `rules` | — | Per-artifact authoring rules |

### `.speclink.yaml`

| Key | Default | Meaning |
|---|---|---|
| `spec_dir` | `openspec` | Spec-store directory, relative to project root |
| `tools` | — | AI harnesses to generate instruction files for (names or descriptors) |
| `locale` / `spec_locale` / `tdd` / `audit` | — | **Deprecated** — still honored, warns on every command |

### Environment variables

| Variable | Values |
|---|---|
| `SPECLINK_LOCALE` | any locale code |
| `SPECLINK_SPEC_LOCALE` | any locale code, or `auto` |
| `SPECLINK_TDD` | `true` / `false` |
| `SPECLINK_AUDIT` | `true` / `false` |
