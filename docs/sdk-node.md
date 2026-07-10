# Node SDK (@speclink/engine)

> **Status:** This document describes the currently implemented Node SDK surface. The target typed Command Runtime, TeamStore contract, Host boundary, and Copilot Tool packaging are defined by [the platform architecture blueprint](platform-architecture.zh-TW.md).

`@speclink/engine` embeds the Speclink engine in a Node.js process: your
server (or AI-agent host) dispatches speclink verbs in-process, stores spec
documents in its own database through a `Store` object, and renders the
workflow knowledge (skills, instruction blocks) for whatever harness it runs.

It is the same Rust engine the CLI ships — bound with [napi-rs](https://napi.rs),
not re-implemented — so verb behavior, `--json` payload shapes, and rendered
content are identical by construction. (The Rust SDK is simply the
`speclink-core` crate.)

## Installation and platform notes

```bash
npm install @speclink/engine
```

- This is a **native module**: the engine is compiled Rust, loaded as a Node
  addon. Prebuilt binaries ship as platform sub-packages
  (`optionalDependencies`), so **`npm install` just works — no system
  dependencies, no toolchain** on the five supported targets:
  - Windows x64 (`win32-x64-msvc`)
  - macOS x64 (`darwin-x64`) and arm64 (`darwin-arm64`)
  - Linux x64 glibc (`linux-x64-gnu`) and arm64 glibc (`linux-arm64-gnu`)
- Deploying to a Linux server means the **linux-x64-gnu (or arm64) binary is
  installed on that platform** — run `npm install` on (or for) the deploy
  target, as with any native module.
- On any other platform npm falls back to building from source, which
  requires a Rust toolchain (`rustup`).

## createEngine — two storage forms

```js
const { createEngine } = require('@speclink/engine')
```

**Built-in fs store** — point the engine at a local project root (the
directory that contains `openspec/`). Zero bridging cost; ideal for local
tools and tests:

```js
const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })
// optional: specDir (default "openspec")
```

**Host Store object** — implement the storage interface yourself (e.g. over
Postgres) and the engine reads/writes documents through it:

```js
const engine = createEngine({ store: myStore })
```

Every `Store` method may return a value **or a Promise** — the bridge accepts
both. If the object is missing required methods, `createEngine` throws
synchronously and lists every missing method name (fail fast; no engine
instance is created).

> **Warning — never call the engine synchronously from inside a Store
> method.** `dispatch` runs on a background worker that waits for your store
> methods to settle. A store method that synchronously blocks on another
> `engine.dispatch(...)` of the same engine creates a wait cycle. Issuing a
> new dispatch *after* your store method has returned (or from unrelated
> code) is fine — concurrent dispatches are supported and tested.

## The Store interface — implementation guide

The interface is one-to-one with the engine core's storage seam
(`speclink-core`'s `Store` trait), camelCase. The engine speaks in domain
terms — changes, artifacts, delta/canonical specs, discussions, workflow
config — and your implementation owns the physical layout. Full signatures
live in [`index.d.ts`](../crates/speclink-node/index.d.ts); `path`/`dir`
return values are **display locations** (strings shown in payloads), not
filesystem paths the engine will open.

| Group | Methods | Notes |
|---|---|---|
| Changes | `listChanges`, `findChange`, `changeExists`, `createChange`, `updatedAtSecs` | `listChanges` returns `{name, dir?, meta?}` sorted by name; `meta` mirrors `.openspec.yaml` (`schema`, `created`, `createdBy`, `createdWith`, `fromDiscussion`). `updatedAtSecs` is the "most recently updated" sort key (whole seconds; missing change → 0). |
| Artifacts | `readArtifact`, `writeArtifact`, `artifactExists` | Artifact ids are schema output paths relative to the change: `proposal.md`, `design.md`, `tasks.md`, `specs/<capability>/spec.md`. An empty document counts as existing. |
| Delta specs | `deltaCapabilities`, `hasCapabilityDirs` | Capability names that have a delta spec inside a change, sorted. |
| Canonical specs | `listCanonicalCapabilities`, `canonicalSpecExists`, `readCanonicalSpec`, `writeCanonicalSpec`, `canonicalSpecPath` | The project-level truth that archiving merges deltas into. |
| Archive | `archivedChangeExists`, `archiveChange`, `readArchivedMeta`, `writeArchivedMeta` | `archiveChange(name, datedName)` moves an active change under its dated archive name (`YYYY-MM-DD-<name>`). |
| Discussions | `liveDiscussionExists`, `archivedDiscussionExists`, `liveDiscussionPath`, `readLiveDiscussion`, `writeLiveDiscussion`, `deleteLiveDiscussion`, `readDiscussion`, `listLiveDiscussions`, `listArchivedDiscussions`, `archiveDiscussion` | Documents are stored as raw text; parsing (rounds, conclusion) is engine logic. `readDiscussion` resolves live first, then the newest archived candidate. |
| Config / vocabulary | `readWorkflowConfig`, `readLanguage` | Raw `config.yaml` text (or null) and the LANGUAGE document (or null — a missing vocabulary is a normal state). |
| Optional | `claim` | Ownership adjudication for team systems — see below. |

A wadpilot-style database mapping, sketched:

```js
// Tables: changes(name PK, meta JSONB, updated_at),
//         artifacts(change_name, path, content, PRIMARY KEY (change_name, path)),
//         canonical_specs(capability PK, content),
//         discussions(slug, text, archived, stored_name)
const store = {
  async listChanges() {
    const rows = await db.query('SELECT name, meta FROM changes ORDER BY name')
    return rows.map((r) => ({ name: r.name, dir: `changes/${r.name}`, meta: r.meta }))
  },
  async readArtifact(change, artifact) {
    const row = await db.maybeOne(
      'SELECT content FROM artifacts WHERE change_name = $1 AND path = $2',
      [change, artifact],
    )
    return row ? row.content : null
  },
  async writeArtifact(change, artifact, content) {
    await db.query(
      `INSERT INTO artifacts (change_name, path, content) VALUES ($1, $2, $3)
       ON CONFLICT (change_name, path) DO UPDATE SET content = $3`,
      [change, artifact, content],
    )
    await db.query('UPDATE changes SET updated_at = now() WHERE name = $1', [change])
    return `changes/${change}/${artifact}`
  },
  async deltaCapabilities(change) {
    const rows = await db.query(
      `SELECT DISTINCT split_part(path, '/', 2) AS cap FROM artifacts
       WHERE change_name = $1 AND path LIKE 'specs/%/spec.md' ORDER BY cap`,
      [change],
    )
    return rows.map((r) => r.cap)
  },
  // …and so on for the remaining methods.
}
```

When a store method throws or rejects, the in-flight `dispatch` rejects with
an `Error` whose message is prefixed with the store method name
(`readArtifact: connection refused`) and whose `code` carries the JS error's
`code` (or `store_error`).

### `claim` (optional)

Ownership is a team-system concept; the engine does not adjudicate it. If
your store implements `claim(name)`, `dispatch(['claim', '<name>'])` routes
to it: resolve with your payload (e.g. `{ claimed: true, claimedBy: 'you' }`)
or reject with an `Error` whose `code` is the verb contract's 409 reason
(`ownership_lost`, `change_busy`, `gate_pending`) and whose message states
who holds the change and what to do — the SDK passes both through to the
caller. Without `claim`, the verb fails loud (as it does on the fs store).

## dispatch — the single entry point

```js
const result = await engine.dispatch(['list', '--json'])
const status = await engine.dispatch(['status', '--change', 'add-auth', '--json'])
await engine.dispatch(
  ['new', 'artifact', 'proposal', '--change', 'add-auth', '--stdin'],
  { stdin: '## Why\n…' },
)
```

- **Input**: a string array, one-to-one with the CLI verb vocabulary (shell
  argv without the program name). There is no interactive input — verbs that
  read stdin in the CLI take the content via the second parameter
  (`{ stdin }`).
- **Output**: a Promise resolving to the same structured object the CLI
  prints with `--json` (camelCase field names). Verbs without a `--json` form
  resolve to `{ output: string }`. The current TypeScript shapes live in
  [`index.d.ts`](../crates/speclink-node/index.d.ts); the future remote
  Command/Query payloads are governed by the platform blueprint and its
  versioned Protocol work.
- **Errors**: the Promise rejects with an `Error` — `message` is the CLI's
  semantic message (safe to hand straight back to an agent), `code`
  classifies it: `invalid_argv` (bad argv), `not_found` (change/discussion
  lookup), `error` (engine failure, the CLI's exit-1 category), a host
  store's 409 reason passed through (`ownership_lost`, …), `store_error`
  (store failure without a code), or `panic`.
- **Never blocks the event loop**: every dispatch runs on a background
  worker thread; concurrent dispatches are supported.

Currently routed verbs: `list`, `status`, `new change`, `new artifact`,
`claim`. The vocabulary grows toward full CLI parity; an unroutable verb
rejects with `invalid_argv`.

## Render API

Workflow knowledge for your harness — the same generation code `speclink
init`/`update` uses, so content cannot drift from the CLI:

```js
const { skills, instructions } = require('@speclink/engine')

skills.list() // [{ name: 'propose', description: '…' }, …]

// The render matrix: target (claude|codex|neutral) × invocation (cli|tool-call) × store (fs|remote)
const skillMd = skills.render('propose', {
  target: 'neutral',
  invocation: 'tool-call',
  store: 'remote',
})
const block = instructions.render({ target: 'neutral', invocation: 'tool-call', store: 'remote' })
```

- `target: 'neutral'` renders for a custom harness: no `/speclink-` slash
  prefix, no plan-mode references; `toolName` (default `"speclink"`)
  substitutes `{{TOOL}}`.
- `invocation: 'tool-call'` words verb references as "call the speclink tool
  with an argv array" — matching a `dispatch`-backed tool; `'cli'` words them
  as shell commands.
- `store: 'remote'` keeps the instructions block free of local spec paths
  (documents are reached through verbs).
- Inject `instructions.render(...)` into your system prompt, and feed
  `skills.render(...)` files to your agent (e.g. write them under a directory
  you pass as `skillDirectories`).

## Complete integration example — Copilot SDK

One tool named `speclink` whose parameter is the argv array, plus generated
skills on disk:

```js
const { createEngine, skills, instructions } = require('@speclink/engine')
const { defineTool, CopilotClient } = require('@github/copilot-sdk') // illustrative imports
const { mkdirSync, writeFileSync } = require('node:fs')
const { join } = require('node:path')

const engine = createEngine({ store: myDatabaseStore })

// 1. The speclink tool: argv in, structured payload out; errors go back as text.
const speclinkTool = defineTool('speclink', {
  description:
    'Run a speclink verb. Pass the argv array exactly as the skill documents say, ' +
    "e.g. ['status', '--change', 'add-auth', '--json'].",
  parameters: {
    type: 'object',
    properties: {
      argv: { type: 'array', items: { type: 'string' } },
      stdin: { type: 'string', description: 'Content for verbs that take --stdin' },
    },
    required: ['argv'],
  },
  async handler({ argv, stdin }) {
    try {
      return await engine.dispatch(argv, stdin === undefined ? undefined : { stdin })
    } catch (err) {
      // err.message is the semantic message — hand it straight to the agent.
      return { error: err.message, code: err.code }
    }
  },
})

// 2. Generate the skill files once (or at boot) and feed them as skillDirectories.
const skillsRoot = join(process.cwd(), '.wad', 'skills')
for (const { name } of skills.list()) {
  const dir = join(skillsRoot, `speclink-${name}`)
  mkdirSync(dir, { recursive: true })
  writeFileSync(
    join(dir, 'SKILL.md'),
    skills.render(name, { target: 'neutral', invocation: 'tool-call', store: 'remote' }),
  )
}

// 3. Wire both into the agent session, with the instructions block in the system prompt.
const client = new CopilotClient({
  tools: [speclinkTool],
  skillDirectories: [skillsRoot],
  systemPrompt: instructions.render({
    target: 'neutral',
    invocation: 'tool-call',
    store: 'remote',
  }),
})
```

The generated skills reference verbs as speclink tool calls, the tool routes
them into the in-process engine, and the engine persists through your store —
no CLI, no child processes, no local `openspec/` tree.

## See also

- [Platform architecture blueprint](platform-architecture.zh-TW.md) — the
  target typed Runtime, TeamStore, Host, Protocol, Server, and Tool design.
- [`index.d.ts`](../crates/speclink-node/index.d.ts) — the currently shipped
  Node API and payload types.
