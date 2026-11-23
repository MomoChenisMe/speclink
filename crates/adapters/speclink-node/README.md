# @speclink/engine

The Node SDK of [Speclink](https://github.com/MomoChenisMe/speclink), a Spec-Driven Development (SDD) engine written in Rust.

Use it to embed Speclink in a Node.js backend. You implement three store functions, and the specs go into your own database. After that, your system is the Server of a remote project: the Speclink CLI and the desktop app connect to it, and your own AI agent uses the same engine.

It is the same Rust engine that the CLI uses, connected to Node through [napi-rs](https://napi.rs). The verb behavior, the `--json` output, and the skill content are the same as in the CLI.

## Install

```bash
npm install @speclink/engine
```

You need Node.js 18 or later. Prebuilt packages are available for Windows x64, macOS x64 and arm64, and Linux x64 and arm64 (glibc). The install downloads only the package for your platform, and you do not need Rust. There is no prebuilt package for musl Linux such as Alpine.

The package includes TypeScript types.

## What the package contains

| Part | What it does | Names you use |
| --- | --- | --- |
| Engine | Runs verbs: reads specs, runs the workflow, and returns the content to write | `createEngine`, `skills` |
| Host kit | The layer for your system: each call brings an identity and a scope, two hooks, two entry points | `createHost`, `agentTool`, `commandNames` |
| Store port contract | The three functions that you implement, and their rules | `StorePort` (type), `StoreConflictError`, `StoreUnavailableError` |
| Conformance tests | Check that your three functions obey the contract | `runStoreConformance` |

## Run verbs on a local project

`createEngine` reads and writes a local folder: the project root that contains `speclink/`. Use it for scripts, internal tools, and tests.

```js
const { createEngine } = require('@speclink/engine')

const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })

const list = await engine.dispatch(['list', '--json'])
await engine.dispatch(
  ['new', 'artifact', 'proposal', '--change', 'add-auth', '--stdin'],
  { stdin: '## Why\n…' },
)
```

`dispatch` takes the CLI command line without the program name, and its result is the same as the CLI `--json` output.

## Embed the engine in your backend

`createHost` builds one host kit for the full process. Your store implements `list`, `read`, and `commit` in your database.

```js
const { createHost } = require('@speclink/engine')

const host = createHost({ store }) // store: your three store functions

// For your own AI agent: the identity and the scope come with each call.
await host.dispatch({
  identity: { id: 'u_42', displayName: 'Alice' },
  scope: 'acme/web',
  argv: ['list', '--json'],
})

// For the CLI and the desktop app: give all requests below /api/speclink/v1/ to one route.
const answer = await host.handleRequest(
  { method, path, query, headers, body },
  identity, // the identity that your login verified, or null
)
```

Use `runStoreConformance` in your tests to check that your store obeys the contract. For the store rules, the hooks, the Agent tool, login, and a complete example that runs, see the [Node SDK](https://github.com/MomoChenisMe/speclink/blob/main/docs/sdk-node.md) guide.

## Documentation

- [Node SDK](https://github.com/MomoChenisMe/speclink/blob/main/docs/sdk-node.md): `createHost`, the three store functions, the conformance tests, the hooks, and the two entry points
- [Embed Speclink in your system](https://github.com/MomoChenisMe/speclink#embedding): when to embed, and what your system does
- [Store port examples](https://github.com/MomoChenisMe/speclink/blob/main/docs/server-store-drivers.md): the SQLite, PostgreSQL, and file system examples
- [Verb and flag contract](https://github.com/MomoChenisMe/speclink/blob/main/docs/verb-contract.md): the verbs and their output shapes
- [Known gaps](https://github.com/MomoChenisMe/speclink/blob/main/docs/product-status.md#known-gaps): the parts that are not done yet
- [Changelog](https://github.com/MomoChenisMe/speclink/blob/main/CHANGELOG.md)

## License

[MIT](https://github.com/MomoChenisMe/speclink/blob/main/LICENSE)
