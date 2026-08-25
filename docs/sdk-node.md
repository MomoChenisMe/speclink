# Node SDK (@speclink/engine)

> **Status:** This document describes the currently implemented Node SDK surface. The typed Command Runtime, the TeamStore contract, and the Host boundary are canonically defined by `command-runtime`, `teamstore-contract`, `host-runtime`, and `node-sdk` under `openspec/specs/`; Copilot Tool packaging is not implemented yet — see the [Project Roadmap](roadmap.md) for that direction.

`@speclink/engine` embeds the Speclink engine in a Node.js process: your
server (or AI-agent host) dispatches speclink verbs in-process, stores spec
documents in its own database through a `Store` object, and renders the
workflow knowledge (skill files) for whatever harness it runs.

It is the same Rust engine the CLI ships — bound with [napi-rs](https://napi.rs),
not re-implemented — so verb behavior, `--json` payload shapes, and rendered
content are identical by construction. (The Rust SDK is simply the
`speclink-core` crate.)

This SDK has two uses. First, wire Speclink into an existing flow: a script, or
an internal tool. Second, use it as **the engine of your own server**. The
official `speclink-server` is only a reference implementation of the Host
contract. You can build your own against `host-runtime` and `client-protocol`
under `openspec/specs/`, with your own authentication, database, and permission
model. The CLI and the desktop app still connect to it.

## Obtaining the package and platform notes

> **Not published to npm yet.** `@speclink/engine` is currently available only
> by building it from this repository; the package does not exist on the npm
> registry. For today's status see the Node SDK row in
> [Project Capability Status](product-status.md); for what the npm channel has to
> solve, where it stands, and the observable next step, see the
> [Project Roadmap](roadmap.md).

Build from the repository and load it:

```bash
git clone https://github.com/MomoChenisMe/speclink.git
cd speclink/crates/speclink-node
npm ci
npm run build          # napi builds the .node for your current platform
```

Reference the build output from your project by path:

```js
const { createEngine } = require('/path/to/speclink/crates/speclink-node')
```

- This is a **native module**: the engine is compiled Rust, loaded as a Node
  addon. The `npm run build` above produces a binary for the **current
  platform**, so run it on (or cross-build it for) your deploy target.
- Building requires a Rust toolchain (`rustup`).
- The engine itself supports Windows x64, macOS x64 and arm64, and Linux x64
  and arm64 (glibc). Once published to npm these ship as prebuilt
  sub-packages and the toolchain is no longer required.

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

**`actor` (optional) — this engine's operator identity.** Both storage forms
accept it, in `"Name <email>"` form:

```js
const engine = createEngine({ store: myStore, actor: 'Alice <alice@example.com>' })
```

It decides who every stamp this engine writes is attributed to: `created_by`
(`new change`) and `reviewed_by` / `verified_by` (`review stamp` /
`verify stamp`).

- **One instance, one identity.** The identity is bound at construction and
  `dispatch` deliberately takes none — a caller cannot claim someone else's.
  A multi-tenant host builds one engine per request (or per identity); an
  engine is just an object, not a connection pool.
- **When omitted**: the fs form falls back to the workspace's git identity
  (byte-for-byte what the CLI stamps); the host-store form has no local
  workspace and stamps no identity at all. A blank string (after trimming)
  reads as omitted.
- **Who may claim which identity is yours to decide.** Authentication and
  authorization belong to the host; the SDK only takes the result.

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
return values are **labels shown in payloads**, not filesystem paths that the
engine opens.

| Group | Methods | Notes |
|---|---|---|
| Changes | `listChanges`, `findChange`, `changeExists`, `createChange`, `updatedAtSecs` | `listChanges` returns `{name, dir?, meta?}` sorted by name; `meta` mirrors `.openspec.yaml` (`schema`, `created`, `createdBy`, `createdWith`, `fromDiscussion`). `updatedAtSecs` is the "most recently updated" sort key (whole seconds; missing change → 0). |
| Artifacts | `readArtifact`, `writeArtifact`, `artifactExists`, `deleteArtifact` (optional) | Artifact ids are schema output paths relative to the change: `proposal.md`, `design.md`, `tasks.md`, `specs/<capability>/spec.md`. An empty document counts as existing. `deleteArtifact` is only reached by review/verify stamping (which deletes the ticket); without it only that path fails. |
| Change metadata (optional) | `readChangeMeta`, `writeChangeMeta` | The raw metadata document of a change (the `.openspec.yaml` text). Stamping is a read-modify-write of this document, so the stamp verbs treat this pair plus `deleteArtifact` as prerequisites: missing any of the three, the stamp refuses before touching anything (ticket intact). No other verb calls them. |
| Completion evidence (optional) | `readEvidence`, `writeEvidence` | The change's completion-evidence record text (the `.evidence.json` content; the store never interprets it). Without `readEvidence` the engine reads "no record" — a normal state anyway; without `writeEvidence` the call fails loudly the moment a completion actually has files to record, never dropping evidence silently. |
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
`claim`, `review add-round`, `review stamp`, `verify add-round`,
`verify stamp`. The vocabulary grows toward full CLI parity; an unroutable
verb rejects with `invalid_argv`.

### Stamping verbs — `review` and `verify`

Each quality station routes two verbs, with the CLI's argv vocabulary:

```js
// Open a round: the content rides the stdin parameter (same mechanism as
// `new artifact --stdin`).
await engine.dispatch(['review', 'add-round', 'add-auth', '--stdin'], { stdin: round })
// → { change: 'add-auth', round: 1 }

// Stamp: fingerprints do not fit in argv, so they ride stdin as JSON.
await engine.dispatch(['review', 'stamp', 'add-auth', '--accept', '--agent', 'claude', '--stdin'], {
  stdin: JSON.stringify({
    scope: [{ path: 'src/auth.ts', hash: '0f9c' }],
    missing: [],
  }),
})
// → { change: 'add-auth' }
```

- The stdin JSON is the **`scope`/`missing` SUBSET** of the server's stamp
  request body — `accept`/`agent` are argv flags here, and unknown fields
  are rejected (`invalid_argv`), not ignored.
- `scope` holds **fingerprints you computed** — a host has no work tree and
  the engine never re-hashes; `missing` declares which paths of the ticket's
  scope are gone. The engine checks that `scope ∪ missing` equals the
  ticket's union and that the two are disjoint, and refuses otherwise. Both
  fields default to empty, and omitting `--stdin` is the same as both empty.
- The stamped `reviewed_by` / `verified_by` is the construction-time `actor`
  (see createEngine above); `--agent` stamps `reviewed_with` /
  `verified_with`.
- **The gates pass through untouched**: unfinished tasks, or unresolved
  CRITICAL/WARNING findings in the last round (`--accept` waives the
  must-fix condition; SUGGESTION never blocks) reject with the engine's
  semantic message.
- Stamping has three prerequisite store methods: `deleteArtifact` (removes
  the ticket) plus `readChangeMeta` / `writeChangeMeta` (the metadata
  read-modify-write). Missing any of them, the stamp refuses up front —
  ticket and metadata untouched.
- **Concurrency**: stamps within one engine instance serialize automatically
  (two stamps never clobber each other); coordinating stamps on the same
  change across instances or processes is your store's job.

## Render API

Workflow knowledge for your harness — the same generation code `speclink
init`/`update` uses, so content cannot drift from the CLI:

```js
const { skills } = require('@speclink/engine')

skills.list() // [{ name: 'propose', description: '…' }, …]

// The render matrix: target (claude|codex|neutral) × invocation (cli|tool-call)
const skillMd = skills.render('propose', {
  target: 'neutral',
  invocation: 'tool-call',
})
```

- `target: 'neutral'` renders for a custom harness: no `/speclink-` slash
  prefix, no plan-mode references; `toolName` (default `"speclink"`)
  substitutes `{{TOOL}}`.
- `invocation: 'tool-call'` words verb references as "call the speclink tool
  with an argv array" — matching a `dispatch`-backed tool; `'cli'` words them
  as shell commands.
- Feed `skills.render(...)` files to your agent (e.g. write them under a
  directory you pass as `skillDirectories`). Routing rides those files: each
  skill's `description` states when to use it, and its closing **Next steps**
  section states what to suggest afterwards — there is no separate instructions
  block to inject, and none is generated any more.

## Complete integration example — Copilot SDK

One tool named `speclink` whose parameter is the argv array, plus generated
skills on disk:

```js
const { createEngine, skills } = require('@speclink/engine')
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
    skills.render(name, { target: 'neutral', invocation: 'tool-call' }),
  )
}

// 3. Wire both into the agent session. Routing needs no system prompt of its
//    own: each skill's description says when to use it.
const client = new CopilotClient({
  tools: [speclinkTool],
  skillDirectories: [skillsRoot],
})
```

The generated skills reference verbs as speclink tool calls, the tool routes
them into the in-process engine, and the engine persists through your store —
no CLI, no child processes, no local `openspec/` tree.

Your harness has to load those skill descriptions for routing to work; a
harness that ignores them has no workflow routing at all.

## See also

- [`index.d.ts`](../crates/speclink-node/index.d.ts) — the currently shipped
  Node API and payload types.
