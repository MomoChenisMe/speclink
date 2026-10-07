# Node SDK (@speclink/engine)

[繁體中文](sdk-node.zh-TW.md) · **English**

`@speclink/engine` lets a Node.js program run Speclink verbs, store specs in its own database, and make skill files for any AI tool. It is the same Rust engine that the CLI uses, bound to Node with [napi-rs](https://napi.rs). It is not a second implementation, so verb behavior, `--json` output shapes, and skill content match the CLI.

Two common uses:

- Connect Speclink to an existing flow, such as a script, an internal tool, or a service that runs an AI agent.
- Use it as **the engine of your own server**. The official `speclink-server` is only a reference implementation. Two public contracts under `openspec/specs/` define remote mode: `host-runtime` and `client-protocol`. You can follow these contracts and build your own server on this engine, with your own authentication, database, and permission model. The CLI and the desktop app connect to it the same way. For the official server, see [Remote Getting Started](remote-getting-started.md).

## <a id="install"></a>Install

```bash
npm install @speclink/engine
```

You can install it from npm from version 0.2.0 on. Its version follows the Speclink release. It needs Node.js 18 or later. Prebuilt packages exist for five platforms: Windows x64, macOS x64 and arm64, Linux x64 and arm64 (glibc). npm downloads only the package for your platform, and you do not need Rust. No prebuilt package exists for musl Linux, such as Alpine.

```js
const { createEngine, skills } = require('@speclink/engine')
```

### Alternative: build from the repository

Build from source when you change the engine itself, or when your platform has no prebuilt package. You need a Rust toolchain:

```bash
git clone https://github.com/MomoChenisMe/speclink.git
cd speclink/crates/adapters/speclink-node
npm ci
npm run build          # builds the .node file for the current platform
```

Then require it by path:

```js
const { createEngine } = require('/path/to/speclink/crates/adapters/speclink-node')
```

`npm run build` builds a file for the current platform only. Build it on the platform that you deploy to.

## <a id="create-engine"></a>Create an engine

`createEngine` supports two storage forms.

**Local files**: point the engine at a project root that contains `openspec/`. This form suits local tools and tests.

```js
const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })
// optional: specDir (default "openspec")
```

**Your own Store**: pass an object that implements the Store interface (see [Implement your own Store](#store)). The engine reads and writes documents through it.

```js
const engine = createEngine({ store: myStore })
```

If the object does not have all required methods, `createEngine` throws at once and lists every missing method name.

**`actor` (optional)**: the operator of this engine, in `"Name <email>"` form. The engine writes it as `created_by` (`new change`), `reviewed_by` (`review stamp`), and `verified_by` (`verify stamp`).

```js
const engine = createEngine({ store: myStore, actor: 'Alice <alice@example.com>' })
```

- One instance has one identity. `dispatch` has no identity parameter, so a caller cannot use another person's identity. In a multi-user system, make one instance per request (or per user). An instance is only an object, so this costs little.
- Without `actor`, the local-files form uses the git identity of the workspace (the same as the CLI). Your own Store records no identity. A blank value counts as no value.
- Your system decides who can use which identity. The SDK only takes the result.

## <a id="dispatch"></a>Run verbs

```js
const list = await engine.dispatch(['list', '--json'])
const status = await engine.dispatch(['status', '--change', 'add-auth', '--json'])
await engine.dispatch(
  ['new', 'artifact', 'proposal', '--change', 'add-auth', '--stdin'],
  { stdin: '## Why\n…' },
)
```

- **Input**: an array of strings. Write it like the CLI arguments, without the program name. A verb that reads stdin in the CLI takes its content from the second parameter, `{ stdin }`.
- **Output**: a Promise. The result is the same as the CLI `--json` output, with camelCase fields. A verb without a `--json` form (for example `new change`) returns `{ output: string }`. The TypeScript type is `Promise<unknown>`. For the field shapes, see [Verb and Flag Contract](verb-contract.md).
- **No blocked event loop**: each dispatch runs on a background thread. You can run many at the same time.

Supported verbs today: `list`, `status`, `new change`, `new artifact`, `claim`, `review add-round`, `review stamp`, `verify add-round`, `verify stamp`. Other verbs fail with `invalid_argv`.

On failure, the Promise rejects with an `Error`. The `message` is the same text that the CLI prints, so you can show it to a user or an agent. The `code` gives the category:

| `code` | Meaning |
| --- | --- |
| `invalid_argv` | The arguments are wrong, the SDK does not support the verb yet, or the written content has a bad format (for example, tasks with no `- [ ]`, or a review round without a `**Scope**:` line) |
| `not_found` | The change or the discussion does not exist |
| `invalid_config` | A config file exists but cannot be parsed (Speclink never uses defaults instead) |
| `refused` | A precondition is not met |
| `error` | Any other engine error. `claim` on the local-files form, or on a Store without `claim`, also gives this code |
| A code from your Store | Your Store method throws an error with a `code`, and the SDK passes it on unchanged, for example `ownership_lost` |
| `store_error` | Your Store method throws an error without a `code` |
| `panic` | An internal engine error |

## <a id="store"></a>Implement your own Store

A Store is an object that the engine uses to read and write changes, artifacts, specs, and discussions. You decide how to keep the data: tables, files, or object storage. [`index.d.ts`](../crates/adapters/speclink-node/index.d.ts) has the full method signatures. Each method can return a value or a Promise.

**Required methods** (31):

| Group | Methods |
| --- | --- |
| Changes | `listChanges`, `findChange`, `changeExists`, `createChange`, `updatedAtSecs` |
| Artifacts | `readArtifact`, `writeArtifact`, `artifactExists` |
| Delta specs | `deltaCapabilities`, `hasCapabilityDirs` |
| Canonical specs | `listCanonicalCapabilities`, `canonicalSpecExists`, `readCanonicalSpec`, `writeCanonicalSpec`, `canonicalSpecPath` |
| Archive | `archivedChangeExists`, `archiveChange`, `readArchivedMeta`, `writeArchivedMeta` |
| Discussions | `liveDiscussionExists`, `archivedDiscussionExists`, `liveDiscussionPath`, `readLiveDiscussion`, `writeLiveDiscussion`, `deleteLiveDiscussion`, `readDiscussion`, `listLiveDiscussions`, `listArchivedDiscussions`, `archiveDiscussion` |
| Config and vocabulary | `readWorkflowConfig`, `readLanguage` |

**Optional methods**:

| Method | When you need it |
| --- | --- |
| `readChangeMeta`, `writeChangeMeta`, `deleteArtifact` | For `review stamp` and `verify stamp`. If one is missing, the stamp fails before it changes anything |
| `claim` | For the `claim` verb. See below |
| `deleteChange`, `readEvidence`, `writeEvidence` | The supported verbs do not use them yet, so you can skip them |

Points to watch:

- **The change meta uses two naming styles.** `createChange(name, metaText)` gets YAML text with snake_case keys, for example `created_by: Alice <alice@example.com>`. `listChanges` and `findChange` must return objects with camelCase meta keys, for example `createdBy`.
- **Return the correct types.** If a boolean, number, or string has the wrong type, the engine does not fail. It uses a default value instead (for example, a non-boolean from `changeExists` reads as `false`). A wrong shape from `listChanges` or `findChange` causes an error.
- **Values such as `path` and `dir` are display labels only.** The engine never opens them as files.
- **Artifact ids** are paths relative to the change: `proposal.md`, `design.md`, `tasks.md`, `specs/<capability>/spec.md`. An empty document counts as present.
- **Only the built-in schema works.** Your own Store has no local workspace, so the engine cannot find project-level or user-level custom schemas.
- **Errors**: when a Store method throws or rejects, the running `dispatch` rejects with an `Error`. The message starts with the method name, for example `listChanges: connection refused`.
- **Do not wait synchronously for another `dispatch` of the same engine inside a Store method.** The two calls wait for each other and stop. A new dispatch after the Store method returns is fine.

This fragment keeps data in a `Map` in memory and shows the two naming styles:

```js
const changes = new Map() // name → { metaText, artifacts: Map }

const store = {
  createChange(name, metaText) {
    // metaText is like "schema: spec-driven\ncreated: 2026-10-07\ncreated_by: Alice <alice@example.com>\n"
    changes.set(name, { metaText, artifacts: new Map() })
    return `changes/${name}`
  },
  findChange(name) {
    const c = changes.get(name)
    if (!c) return null
    const meta = parseYaml(c.metaText) // use your usual YAML package
    return { name, meta: { schema: meta.schema, created: meta.created, createdBy: meta.created_by } }
  },
  listChanges() {
    return [...changes.keys()].sort().map((name) => store.findChange(name))
  },
  changeExists: (name) => changes.has(name),
  readArtifact: (change, artifact) => changes.get(change)?.artifacts.get(artifact) ?? null,
  writeArtifact(change, artifact, content) {
    changes.get(change).artifacts.set(artifact, content)
    return `changes/${change}/${artifact}`
  },
  // …add the other required methods the same way
}
```

### `claim` (optional)

A claim is a team-system concept, so your Store decides who can claim a change. When you implement `claim(name)`, `dispatch(['claim', '<name>'])` calls it:

- On success, return your own data, for example `{ claimed: true, claimedBy: 'alice' }`. The SDK gives it to the caller unchanged.
- On a conflict, throw an `Error` with a `code` (for example `ownership_lost`). The message must say who holds the change and what to do. The SDK passes it on unchanged.

Without `claim`, the verb fails with `error`.

## <a id="stamp"></a>Quality stations: `review` and `verify`

Each quality station supports two verbs, with the same arguments as the CLI:

```js
// Add a round: pass the content in the stdin parameter
await engine.dispatch(['review', 'add-round', 'add-auth', '--stdin'], { stdin: round })
// → { change: 'add-auth', round: 1 }

// Stamp: put the file fingerprints in JSON on stdin
await engine.dispatch(['review', 'stamp', 'add-auth', '--agent', 'claude', '--stdin'], {
  stdin: JSON.stringify({
    scope: [{ path: 'src/auth.ts', hash: '<sha256>' }],
    missing: [],
  }),
})
// → { change: 'add-auth' }
```

- `scope` holds **file fingerprints that you calculate**. The engine cannot see your work tree, so it does not calculate them. To match the CLI, use the SHA-256 of the file content in hex. For a text file, change CRLF to LF first.
- `missing` lists files in the ticket scope that no longer exist. Together, `scope` and `missing` must equal the files of the ticket, with no path in both. Otherwise the stamp fails. You can leave out either field (it reads as an empty list). An extra field fails with `invalid_argv`.
- The stamp fails if a task is not done. Manual tasks marked `[M]` do not count. The stamp also fails if the last round has a CRITICAL or WARNING finding. `--accept` skips the must-fix findings. A SUGGESTION never blocks a stamp.
- `reviewed_by` / `verified_by` is the `actor` of the engine. `--agent` goes into `reviewed_with` / `verified_with`.
- A stamp needs `readChangeMeta`, `writeChangeMeta`, and `deleteArtifact`. If one is missing, the stamp fails before it starts, and the ticket and the meta stay unchanged.
- Stamps in one engine instance run one at a time, so they never overwrite each other. Your Store must coordinate stamps on the same change across instances or processes.

## <a id="render"></a>Make skill files

`skills` uses the same generator as `speclink init` and `speclink update`, so its content always matches the CLI.

```js
skills.list() // [{ name: 'analyze', description: '…' }, …]

const skillMd = skills.render('propose', { target: 'neutral', invocation: 'tool-call' })
```

| Option | Values | Notes |
| --- | --- | --- |
| `target` | `claude`, `codex`, `neutral` | `neutral` is for custom tools: no `/speclink-` slash commands and no plan-mode text |
| `invocation` | `cli` (default), `tool-call` | `tool-call` writes verbs as "call the speclink tool with an argv array", for a tool built on `dispatch`. `cli` writes shell commands |
| `specDir` | A string | The spec directory name in the skill text. Default: `openspec` |
| `toolName` | A string | `neutral` only: the tool name in the skill text. Default: `speclink` |

Write each `SKILL.md` into a directory, and let your agent load it. The `description` of each skill says when to use it. The Next steps section at the end says what to suggest after it. You do not need a separate system prompt. Your agent must read the skill descriptions for this to work.

## <a id="example"></a>Full example: Copilot SDK

This example makes one tool named `speclink` with an argv array parameter, and adds the generated skill files. The official Copilot SDK documents are the authority for its API. This example uses `@github/copilot-sdk` 1.0.16.

```js
const { createEngine, skills } = require('@speclink/engine')
const { CopilotClient, defineTool } = require('@github/copilot-sdk')
const { mkdirSync, writeFileSync } = require('node:fs')
const { join } = require('node:path')

const engine = createEngine({ store: myDatabaseStore, actor: 'Alice <alice@example.com>' })

// 1. The speclink tool: argv in, result out; errors go back to the agent as text.
const speclinkTool = defineTool('speclink', {
  description:
    'Run a speclink verb. Pass the argv array exactly as the skill says, ' +
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
      return { error: err.message, code: err.code }
    }
  },
})

// 2. Make the skill files.
const skillsRoot = join(process.cwd(), '.my-agent', 'skills')
for (const { name } of skills.list()) {
  const dir = join(skillsRoot, `speclink-${name}`)
  mkdirSync(dir, { recursive: true })
  writeFileSync(join(dir, 'SKILL.md'), skills.render(name, { target: 'neutral', invocation: 'tool-call' }))
}

// 3. Give the tool and the skill directory to the session.
async function startSession() {
  const client = new CopilotClient()
  return client.createSession({ tools: [speclinkTool], skillDirectories: [skillsRoot] })
}
```

The skills describe each verb as a tool call. The tool sends the call to the engine in the same process, and the engine saves through your Store. No CLI, no child process, and no local `openspec/` directory take part.

## <a id="limits"></a>Current limits

- The SDK supports nine verbs only (see [Run verbs](#dispatch)). You cannot run `archive`, `task`, `discuss`, or other verbs through the SDK yet.
- Your own Store can use only the built-in schema.
- No prebuilt package exists for musl Linux.
- When a Store method throws, stderr can show one extra Rust panic line. The Promise still rejects as described above.
- The return value of `dispatch` has no per-verb TypeScript type. [Verb and Flag Contract](verb-contract.md) defines the shapes.
