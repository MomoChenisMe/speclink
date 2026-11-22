/**
 * @speclink/engine — Node.js bindings for the Speclink spec-driven
 * development engine.
 *
 * The engine core is synchronous Rust; every `dispatch` runs on a background
 * worker thread and returns a Promise — the JS event loop is never blocked.
 */

/** A function the host supplies may return its value directly or as a Promise. */
export type MaybePromise<T> = T | Promise<T>

/** The built-in filesystem store: a local project root with a spec directory. */
export interface FsStoreOptions {
  type: 'fs'
  /** Project root (the directory containing the spec directory). */
  root: string
  /**
   * Spec directory name relative to root. Default: the CLI's resolution —
   * `.speclink.yaml`'s `spec_dir`, else `speclink/`, else a legacy `openspec/`,
   * else `"speclink"`.
   */
  specDir?: string
}

export interface CreateEngineOptions {
  /** The local project folder. A store of your own goes to `createHost`, as a `StorePort`. */
  store: FsStoreOptions
  /**
   * The operator identity every stamp this engine writes is attributed to
   * (`created_by`, `reviewed_by`, `verified_by`), in `"Name <email>"` form.
   *
   * Bound once at construction — one engine instance, one identity. There
   * is deliberately no way to pass an identity at dispatch time, so a caller
   * cannot claim someone else's. A host that serves several users uses
   * `createHost`: there every call brings its own identity.
   *
   * Omitted (or blank after trimming): the engine falls back to the
   * workspace's git identity.
   */
  actor?: string
}

export interface DispatchOptions {
  /**
   * Content for verbs that read stdin in the CLI (the `--stdin` flag), e.g.
   * `dispatch(['new', 'artifact', 'proposal', '--change', 'x', '--stdin'], { stdin })`.
   */
  stdin?: string
}

export interface Engine {
  /**
   * Dispatch one speclink verb. `argv` mirrors the CLI vocabulary one-to-one
   * (shell argv without the program name, e.g. `['list', '--json']`).
   *
   * Every verb that has an engine command runs. The verbs that need a
   * checkout (`drift`, `review prepare`, `review scope`, `verify scope`,
   * `instructions --skill`) reject with `invalid_argv`. `claim` rejects as it
   * does in the CLI on a local project: it needs a remote store.
   *
   * Resolves to the same structured object the CLI prints with `--json`
   * (camelCase); verbs without a `--json` form resolve to `{ output: string }`,
   * where `output` is the CLI's stdout text with no color codes. When such a
   * verb also prints a note to stderr in the CLI (for example `archive` on a
   * change with no task evidence), the object carries it as `stderr`. Both
   * come from the output definition the CLI uses. The verb surface and
   * payload contract are described in docs/verb-contract.md.
   *
   * `discuss conclude` resolves even when its closing step could not archive
   * the record: the conclusion is written, and the object carries the reason
   * as `closingError`. Do not conclude again; run `discuss archive`.
   *
   * Rejects with an `Error` whose `message` is the CLI's semantic message and
   * whose `code` property classifies the failure (`error`, `not_found`,
   * `invalid_argv`, `invalid_config`, `refused`, or `panic`).
   */
  dispatch(argv: string[], options?: DispatchOptions): Promise<unknown>
  /**
   * Every document of the local project `createEngine` points at, as a
   * bundle by key: pass it as it is to `host.importScope` to move the project
   * into an empty scope of your store. The documents are the ones Speclink
   * Desktop's "Migrate to Server" sends: the active changes (metadata,
   * artifacts, evidence), the canonical specs, the live and the archived
   * discussions, every document of the archived changes, the workflow
   * config, the vocabulary and the board order. A change's schema is
   * resolved in the project, then in the user-level config directory of the
   * user that runs it.
   *
   * It reads and never writes, and runs on a background thread. The bundle
   * has no `scopeVersion`: a local project has none. Its `documents` are in
   * byte order of their keys; a spec folder with no document gives none.
   *
   * The bundle holds the full text of the workflow config and of every
   * discussion: decide yourself who may run an export. A large project makes
   * a large bundle, and `host.importScope` refuses one over its
   * `limits.importBytes`.
   *
   * Rejects with an `Error` whose `code` is `error`, and gives no bundle,
   * when a change (active or archived) has metadata that does not parse or a
   * document the engine's naming rules do not cover: the `message` names the
   * file or the document. A file that is there but cannot be read (no
   * permission, or not UTF-8) is no failure: the bundle has no document for
   * it.
   *
   * It reads no file outside the project. It rejects the same way, before it
   * reads a document, when a symbolic link is on the way to the spec folder,
   * is the spec folder, or is anything under it — or in the older evidence
   * folder `.speclink/touched/` (the `message` starts with `symbolic link in
   * the spec folder:` and names the item by its path from `root`, never what
   * it points at); when a folder there cannot be listed (the `message`
   * starts with `cannot list`); and when the spec folder's name — `spec_dir`
   * in `.speclink.yaml`, or `specDir` — is an absolute path or has a `..` or
   * `.` segment (the `message` starts with `spec folder outside the
   * project:` and names it). To export a spec folder somewhere else, point
   * `root` at the folder above it.
   *
   * After those checks it rejects the same way when `root` holds no spec
   * folder under that name — nothing there, or a folder with no `changes/`,
   * `specs/`, `discussions/` and no `config.yaml`, also when it holds a
   * `LANGUAGE.md` (the `message` starts with `no spec folder:` and names the
   * folder, never the path of `root`). This catches a download put in the
   * wrong place, which would otherwise export no document and look like a
   * success. To export an empty project, make its spec folder first.
   *
   * It reads the files one by one, with no snapshot: do not write to the
   * project — through this engine, the CLI or anything else — while it
   * exports, or the bundle can hold old and new content together.
   *
   * Six of these failures also give `error.reason`, a fixed string to show
   * your own text by; do not match the `message`, which can change:
   *
   * | `reason` | the failure |
   * | --- | --- |
   * | `no_spec_folder` | `root` holds no spec folder |
   * | `spec_folder_outside_project` | the spec folder's name points out of the project |
   * | `symlink_in_spec_folder` | a symbolic link on the way to the spec folder, at it or under it, or in the older evidence folder `.speclink/touched/` |
   * | `spec_folder_unreadable` | a folder under the spec folder, or the older evidence folder, cannot be listed |
   * | `invalid_change_metadata` | an active or archived change's metadata does not parse |
   * | `change_schema_unusable` | the schema an active or archived change names cannot be used: it is not found, its definition does not load, or it has a file pattern the engine cannot list |
   *
   * The last two are about one change, and only they give two more strings:
   * `error.change`, the change's folder name (an archived change's is its
   * dated name, `2026-03-03-old-one`), and `error.path`, its metadata file by
   * its path from `root` with forward slashes
   * (`speclink/changes/archive/2026-03-03-old-one/.meta.yaml`). The path
   * names the file that is there: the older name `.openspec.yaml` when only
   * that one is. When the change has no metadata file, the path names the
   * place of `.meta.yaml`, and no file is there. Every other failure has no
   * `change` and no `path` property.
   * The `message` of `change_schema_unusable` starts with `change '<name>': `.
   * Both values are names from the disk, as they are: show them, and the
   * `message`, as plain text.
   *
   * Every other failure has no `reason` property (`'reason' in error` is
   * false): a document whose names make no key under the engine's naming
   * rules, for one. Take "no reason" as a valid answer: a later version can
   * add a code to such a failure, never change the meaning of one above.
   */
  exportScope(): Promise<ScopeBundle>
}

/**
 * Build an engine over a local project folder (`{ type: 'fs', root }`). A
 * `store` of any other shape throws synchronously, and none of its methods is
 * called: a store of your own goes to `createHost`, as a `StorePort` of three
 * functions.
 */
export function createEngine(options: CreateEngineOptions): Engine

// --- the store port: three functions (createHost, runStoreConformance) ---

/** A document of the store port. Versions and times are integers from 0 to 2^53 − 1 (`Number.MAX_SAFE_INTEGER`); times are milliseconds since the Unix epoch. */
export interface PortDocument {
  content: string
  /** The scope version of the commit that last wrote this document. */
  version: number
  modifiedAt: number
}

export interface PortListEntry {
  key: string
  version: number
  modifiedAt: number
}

/** The scope's current version and the matching documents, in byte order of their keys. */
export interface PortListing {
  version: number
  entries: PortListEntry[]
}

/** One change in a commit. `expected: null` means the document must not exist; a number is the version it must have. */
export type PortOp =
  | { op: 'put'; key: string; content: string; expected: number | null }
  | { op: 'delete'; key: string; expected: number }

export interface PortEvent {
  name: string
  payload: unknown
}

/** Everything one command writes, in one commit. */
export interface PortCommit {
  actor: { id: string; displayName: string }
  /** The command's stable name, e.g. `new-change`. */
  command: string
  at: number
  ops: PortOp[]
  /** The domain events of the command: the only way events reach the host. */
  events: PortEvent[]
}

/**
 * The storage contract a host implements: three functions, each on one scope
 * (an opaque string). Each function may return its value or a Promise.
 *
 * - `commit` applies all of its operations or none: run it as one database
 *   transaction, and check every `expected` before the first write. One
 *   expectation that does not hold rejects the whole commit. Give the written
 *   documents the new scope version as their version and `at` as their time,
 *   and keep the events in the same transaction when you need them delivered.
 * - `read` returns one result per key, in the same order (`null` for a
 *   missing document), all from one completed commit. `list` returns the
 *   documents whose key starts with `prefix`, in byte order of the keys, each
 *   key once.
 * - Results cross to the engine as JSON, so every number keeps its exact
 *   value; a result JSON cannot hold (a cycle, a BigInt) fails the call.
 *
 * Errors: throw an object whose `code` is `'conflict'` (with the `key` that
 * does not match and its `actual` version, or null when it does not exist) or
 * `'unavailable'` (a temporary failure). Anything else is another failure.
 * `StoreConflictError` and `StoreUnavailableError` set these fields for you;
 * the SDK reads only the fields, never `instanceof`.
 *
 * Each function must settle: the dispatch waits for it with no time limit,
 * so a Promise that never settles keeps that dispatch (and its worker thread)
 * waiting. Put any timeout you need inside the function.
 *
 * WARNING: do not wait for a `dispatch` of the same host inside these
 * functions — the dispatch waits for them, so it would never finish.
 */
export interface StorePort {
  list(scope: string, prefix: string): MaybePromise<PortListing>
  read(scope: string, keys: string[]): MaybePromise<(PortDocument | null)[]>
  commit(scope: string, commit: PortCommit): MaybePromise<number>
}

// --- the host kit: createHost ---

/** Who acts: an id that stays when the user is renamed, and the name document stamps carry. Neither may be empty or only white space. */
export interface HostIdentity {
  id: string
  displayName: string
}

export interface BeforeCommandInfo {
  identity: HostIdentity
  scope: string
  /** The engine command's stable name, e.g. `archive` or `workflow-config-edit`. */
  command: string
  /** The command's params, camelCase (e.g. `{ change: 'demo', carryReview: true, … }`). Read them; changing them changes nothing. */
  params: Record<string, unknown>
  /** Only for the workflow-config writes: the keys the write changes, e.g. `['tdd']`. */
  changedKeys?: string[]
}

/** `undefined` or `{ allow: true }` allows; `{ allow: false, reason }` refuses with that reason. Any other answer, `null` included, does not allow. */
export type BeforeCommandAnswer = void | { allow: true } | { allow: false; reason: string }

/** What `beforeCommand` gets besides the call's info. */
export interface BeforeCommandTools {
  /**
   * Run one verb that writes nothing, as the identity and on the scope of
   * this call. Resolves to what `host.dispatch` gives for the same `argv`,
   * and rejects with the same Error (`message` and `code`). It takes no
   * stdin and no fields.
   *
   * The verbs are `list`, `show`, `status`, `plan`, `trace`, `evidence show`,
   * `review show`, `verify show`, `discuss list`, `discuss show`,
   * `discuss search`, `artifact cat`, `language show`,
   * `workflow-config show`, `validate`, `analyze` and `instructions`. Any
   * other verb rejects with code `invalid_argv` before the store gets a call.
   *
   * A read does not ask `beforeCommand` again and does not call
   * `afterCommit`. You can call it several times in one hook, also at once.
   * It works only while this `beforeCommand` call runs: after the hook
   * answered, it rejects with code `invalid_argv`.
   *
   * Each read is a call of its own to your store. What it gives and what the
   * command itself reads can differ when another commit lands between them;
   * the engine's own checks judge by what the command reads.
   */
  read(argv: string[]): Promise<unknown>
}

export interface AfterCommitInfo {
  identity: HostIdentity
  scope: string
  command: string
  /** The scope version the commit returned. */
  scopeVersion: number
  /** The events of the commit, the same the store's `commit` received. */
  events: PortEvent[]
}

/**
 * The host's two points to step in; both are optional. An unknown name is
 * refused at `createHost`.
 *
 * `beforeCommand` must settle: the dispatch waits for it with no time limit.
 *
 * To read data inside `beforeCommand`, use the `read` of its second argument.
 * Do not wait for a `dispatch` of the same host there: that call asks
 * `beforeCommand` again, and it hangs when your store holds a lock of its own.
 */
export interface HostHooks {
  /**
   * Asked exactly once per call, before anything is written. A refusal ends
   * the call with code `refused` and your reason; for any command but the
   * workflow-config writes it comes before the store sees a call. A throw, a
   * rejected Promise or an answer of another shape does not allow: the call
   * ends with code `internal`. `tools.read` reads what the decision needs,
   * for example `read(['show', info.params.change, '--json'])`; its rejection
   * that you do not catch is such a throw.
   */
  beforeCommand?(info: BeforeCommandInfo, tools: BeforeCommandTools): MaybePromise<BeforeCommandAnswer>
  /**
   * Called after a successful commit; it only watches. The dispatch does not
   * wait for it, what it returns is ignored, and its throw or rejected
   * Promise is swallowed. Delivery is
   * not guaranteed: a host that needs every event handles them in the store's
   * `commit`, in the same transaction as the writes.
   */
  afterCommit?(info: AfterCommitInfo): MaybePromise<void>
}

/** A repo of a project the caller may use: the store scope it reaches and what the caller may do there. */
export interface HostScopeRepo {
  /** The key the client names the repo by (the `X-Speclink-Repo` header). */
  key: string
  name: string
  /** The store scope every request for this repo reads and writes. */
  scope: string
  /** False: every request that writes is refused with 403. */
  canWrite: boolean
  /** False: a workflow-config write that changes a policy key (schema, locale, spec_locale, tdd, audit, worktree) is refused with 403. */
  canWritePolicy: boolean
}

/** A project the caller may use. */
export interface HostScopeProject {
  /** The key the URL names the project by. */
  key: string
  name: string
  repos: HostScopeRepo[]
}

/** The login methods the host supports, answered at `GET /api/speclink/v1/auth-methods` without a credential. */
export interface HostAuthMethods {
  /** `browser`, `device`, `token`. */
  methods: string[]
  /**
   * The login service behind `browser`. `scopes` is optional here. The answer
   * of the entry point always has `browser.scopes`: an empty array when it is
   * not declared.
   */
  browser?: { issuer: string; clientId: string; scopes?: string[] }
}

/**
 * A push stream the host serves itself; the handshake declares it, as
 * written, before the polling fallback. An idle `sse` stream sends a comment
 * line more often than every 45 seconds: a client that gets no data for 45
 * seconds subscribes again. The host ends the stream of a user who loses
 * access (a suspended account, a removed membership): the entry point serves
 * no stream, so it cannot. The client then subscribes again and polls, and
 * Speclink Desktop shows "Access denied" when the two are refused.
 */
export interface HostPushTransport {
  /** `sse` or `websocket`. Speclink Desktop uses the first `sse`, and polls when there is none. */
  type: string
  /**
   * A path that starts with `/`, relative to the project URL as `/sync-state`
   * is, or a full URL with the host's own origin (scheme, host, and port). A
   * client does not connect to another origin and does not send it the
   * credential: it polls.
   */
  url: string
  resume?: boolean
}

export interface CreateHostOptions {
  store: StorePort
  hooks?: HostHooks
  /**
   * The projects and repos `identity` may use, asked once per
   * `handleRequest` and never cached: cache it yourself if it is slow. A
   * throw or a rejected Promise answers the request with 503. Absent: the
   * scope list and every project path are 404, so a host with only the Agent
   * tool opens nothing by accident.
   */
  scopes?(identity: HostIdentity): MaybePromise<HostScopeProject[]>
  /** Absent: `auth-methods` is 404 and clients keep their old login behavior. */
  authMethods?: HostAuthMethods
  /** Absent: the handshake declares polling only. */
  pushTransports?: HostPushTransport[]
  /**
   * The skills `handleRequest` gives a client that asks which skills to
   * install, by name (`['discuss', 'propose']`; `skills.list()` has the
   * names). Absent: every skill of the engine. An empty array, or a name
   * the engine does not have, makes `createHost` throw at once. You choose which
   * skills, not their text: it always comes from the engine. `skills.render`
   * and `host.dispatch` do not read this list.
   */
  skills?: string[]
  /** Size limits. A value that is not a positive integer makes `createHost` throw at once. */
  limits?: HostLimits
}

export interface HostLimits {
  /**
   * The most an import takes, in bytes. Absent: 33554432 (32 MiB).
   * `handleRequest` compares it with the request body of the import route
   * before it parses anything, and answers 413 over it; `importScope`
   * compares it with the documents' content, and rejects with `invalid_argv`
   * over it. An export has no limit.
   *
   * `handleRequest` gets a body your web framework already read, so the
   * framework's own body limit for the import route must be at least this
   * value (the Express example on `handleRequest` reads 4 MB: raise it for
   * the import). The host kit does not compress or decompress: turn that on
   * in your route layer.
   */
  importBytes?: number
}

/** A request as your web framework received it. `path` is as sent (still percent-encoded); `query` is the text after `?`. */
export interface HostRequest {
  method: string
  path: string
  query?: string
  headers: Record<string, string>
  body?: Uint8Array
}

export interface HostResponse {
  status: number
  headers: Record<string, string>
  body: Uint8Array
}

/** One document of a {@link ScopeBundle}. `digest` is `sha256:` and the hex SHA-256 of `content` (UTF-8). */
export interface BundleDocument {
  /** The document's store key, as your store port sees it (`changes/add-x/proposal.md`). */
  key: string
  content: string
  digest: string
}

/** Every document of one scope, each named by its store key: the second version of the bundle. */
export interface ScopeBundle {
  formatVersion: 2
  /** The scope version an export was taken at. An export always has it; an import does not read it. */
  scopeVersion?: number
  /** In byte order of the keys when the bundle is an export. */
  documents: BundleDocument[]
}

/**
 * The first version of the bundle, which names each document by its kind
 * (`{ type: 'changeMeta', change }`, …): what an older Speclink Desktop
 * sends. `importScope` takes it; an export never gives it.
 */
export interface FirstVersionBundle {
  formatVersion: 1
  scope: { project: string; repo: string }
  projectRevision: number
  documents: { document: { type: string; [name: string]: unknown }; content: string; digest: string }[]
}

export interface HostExportCall {
  /** From your own code (your session). You decide who may export: this call asks for no write switch. */
  identity: HostIdentity
  scope: string
}

export interface HostImportCall {
  /** From your own code (your session). You decide who may import, and who may replace. */
  identity: HostIdentity
  /** The scope to write. In the `'create'` mode it must be empty. */
  scope: string
  bundle: ScopeBundle | FirstVersionBundle
  /**
   * `'create'` (the default) writes the bundle into an empty scope.
   * `'replace'` makes the scope equal the bundle: it adds, updates and
   * **deletes** documents. Any other value rejects with `invalid_argv`.
   */
  mode?: 'create' | 'replace'
  /**
   * `true` does every check and the compare, reports what a real run would
   * do, and commits nothing. The default is `false`: a real run. A value
   * that is not a boolean (the text `'false'`, the number `0`) rejects with
   * `invalid_argv`, and so does a `dryRun` key that holds `undefined`: for a
   * real run, leave the key out.
   */
  dryRun?: boolean
}

export interface HostImportReport {
  /**
   * The scope version after the commit. With no commit (a dry run, a replace
   * with nothing to write or delete, a create with no documents) it is the
   * version the scope had when it was listed.
   */
  scopeVersion: number
  /** How many documents were written: `added.length + changed.length`. */
  documents: number
  /** The keys the scope did not hold, in byte order. The `'create'` mode gives every key of the bundle. */
  added: string[]
  /** The keys both sides hold with different content, in byte order. Always empty in the `'create'` mode. */
  changed: string[]
  /** The keys only the scope held, which a replace deletes, in byte order. Always empty in the `'create'` mode. */
  removed: string[]
}

export interface HostDispatchCall {
  /** From your own code (your session), never from the model: argv and stdin cannot change it. */
  identity: HostIdentity
  scope: string
  /** The verb text without the program name, as for `engine.dispatch`. */
  argv: string[]
  stdin?: string
  /**
   * Free fields from your own code, never from the model: for example where
   * the call came from. They are written only when the verb creates a change
   * or a discussion (`new change`, `discuss new`, `discuss promote`), in the
   * commit that creates it, and they win over a `--field` of the same key.
   * Every other verb ignores them, so you can pass the same fields with every
   * call. `beforeCommand` sees the fields that will be written, in
   * `params.fields`. A value that is not a string, or fields outside the
   * rules of free fields (a key of `a-z`, `0-9`, `_`, `.`, `-` that starts
   * with a letter; a one-line value of at most 1024 bytes; at most 32 keys),
   * reject with `invalid_argv` and nothing is written. Do not put a secret in
   * a field: every reader of the change gets it.
   */
  fields?: Record<string, string>
}

/** What `host.handleRequest` takes besides the request and the identity. */
export interface HostRequestExtras {
  /**
   * Free fields from your own code, as `HostDispatchCall.fields`: written
   * only when the request creates a change or a discussion, over a field of
   * the same key in the request body; every other request ignores them. A
   * value that is not a string, or fields outside the rules, answers 400.
   */
  fields?: Record<string, string>
}

export interface Host {
  /**
   * Run one verb for `identity` on `scope`. The verbs, the stdin rules, the
   * result and the Error shape are those of `engine.dispatch`; the verbs that
   * need a checkout reject with `invalid_argv`. An identity or scope that is
   * empty rejects with `invalid_argv` before any store call. Store failures
   * reject with `revision_conflict`, `unavailable` or `internal`; a refusal
   * of `beforeCommand` with `refused`.
   *
   * One difference from `engine.dispatch`: the documents are in your store,
   * not in a local folder, so the `{ output }` text of `new change` has no
   * `Path:` line. `discuss new` and `discuss promote` return a structured
   * value, with no text; its `path` is still there. A `path` is the
   * document's key in the store, not a file to open. Read a document with
   * `artifact cat`, `show` or `discuss show`.
   */
  dispatch(call: HostDispatchCall): Promise<unknown>
  /**
   * The HTTP door for the original CLI and Speclink Desktop: every path is
   * under `/api/speclink/v1/`. Your one route verifies the token itself —
   * the door never reads `Authorization` — and passes the identity, or null
   * when it could not verify one:
   *
   * ```ts
   * // Express: mount it before any body parser, so the body is the raw bytes.
   * app.all('/api/speclink/v1/*', express.raw({ type: '*\/*', limit: '4mb' }), async (req, res, next) => {
   *   const identity = await verifyToken(req.headers.authorization) // yours; null when invalid
   *   const [path, query] = req.originalUrl.split('?')
   *   const answer = await host.handleRequest(
   *     // `path` and `query` as sent, not parsed; `body` as bytes (a Buffer is a Uint8Array).
   *     // The door reads single-value headers only, so Node's header object is passed as it is.
   *     {
   *       method: req.method,
   *       path,
   *       query,
   *       headers: req.headers as Record<string, string>,
   *       body: Buffer.isBuffer(req.body) ? req.body : undefined,
   *     },
   *     identity,
   *   )
   *   if (answer === null) return next() // not the door's path: your own routes, or your 404
   *   res.status(answer.status).set(answer.headers).send(Buffer.from(answer.body))
   * })
   * ```
   *
   * Resolves to null for a path that is not the door's. A method other than
   * GET, HEAD, POST, PUT and DELETE resolves to a 405. Runs on a background
   * thread; the hooks are the same as `dispatch`'s. `extras.fields` attaches
   * free fields from your own code to the request ({@link HostRequestExtras}).
   */
  handleRequest(
    request: HostRequest,
    identity: HostIdentity | null,
    extras?: HostRequestExtras,
  ): Promise<HostResponse | null>
  /**
   * Every document of `scope` — for a backup, or to move the scope — with the
   * scope version it was read at. It uses your store's `list` and `read`
   * only. When the scope changes during the export it starts again, three
   * times at most, then rejects with `unavailable`: it never gives old and
   * new content together. An empty scope is a bundle with no documents.
   * `beforeCommand` is asked first, as `scope-export` with no params. Runs
   * on a background thread.
   *
   * No failure of this method has a `reason` property for now: a refusal of
   * `beforeCommand`, a store failure and a scope that kept changing have
   * none (see `importScope` for the codes an import gives).
   */
  exportScope(call: HostExportCall): Promise<ScopeBundle>
  /**
   * Write a bundle into `scope` in one `commit` of your store. A failure
   * leaves the scope as it was. Runs on a background thread.
   *
   * **`mode: 'create'`** (the default) takes an empty scope only: every
   * document is written with `expected: null`, with one `scope-imported`
   * event (`{ documents }`). A bundle with no documents writes nothing and
   * is no failure.
   *
   * **`mode: 'replace'`** makes the scope equal the bundle. It deletes every
   * document the bundle does not hold: the workflow config, and the changes,
   * the discussions and the stamps made in your host after the first import.
   * The scope is listed once, the keys both sides hold are read and
   * compared, and one `commit` carries everything: a key only the bundle
   * holds is written with `expected: null`; a key with different content is
   * written, and a key only the scope holds is deleted, both with the
   * version it was listed at. A document with the same content is not
   * written again: its version and its `modifiedAt` stay. The commit has one
   * `scope-imported` event (`{ documents, removed }`). With nothing to write
   * or delete there is no commit and no event. A bundle with no documents is
   * refused: one call cannot empty a scope.
   *
   * **`dryRun: true`** does all of it but the commit, in both modes, and
   * resolves to the same `added`, `changed` and `removed` as the real run
   * that follows it at once. Show `removed` to the person who decides, then
   * call again without `dryRun`.
   *
   * A commit states what it expects of its own keys only. A replace is
   * refused whole (`scope_changed`) when another writer changed or deleted a
   * key it writes or deletes; it does not try again. It does not see another
   * writer's change to a key it has no operation for: a key whose content is
   * the same on both sides, or a key that neither side held when the scope
   * was listed. That document stays as the other writer left it, and a dry
   * run after the replace shows it. To make sure the scope equals the
   * bundle, refuse the other writes in `beforeCommand` while you replace.
   *
   * The replace mode and the dry run are for this call only: the HTTP door
   * (`handleRequest`, `POST /import`) has the create mode only.
   *
   * The two fields are checked before the store and `beforeCommand` are
   * called, and reject with `invalid_argv`: a `mode` that is neither value, a
   * `dryRun` key that holds no boolean, and, in the replace mode, a call with a
   * key other than `identity`, `scope`, `bundle`, `mode` and `dryRun` (so
   * `dryrun: true` cannot run as a real replace). Then the size limit, then
   * `beforeCommand`, which is asked once for every call, a dry run too, as
   * `scope-import` with `params: { documents, bytes, mode, dryRun }` — the
   * count, the bytes of the content, `'create'` or `'replace'`, and a
   * boolean; never the content. `afterCommit` is told about the commit, when
   * there is one.
   *
   * Rejects with `invalid_argv` when the bundle is not valid (a digest that
   * is not its content's, a key outside the engine's naming rules, a key
   * that appears twice, content over `limits.importBytes`; the message names
   * the key) or a replace has no documents, and with `refused` when the
   * scope of a create is not empty, the scope changed during a replace, or
   * `beforeCommand` refuses. Store failures reject with `revision_conflict`,
   * `unavailable` or `internal`.
   *
   * Five of these failures also give `error.reason`, a fixed string to show
   * your own text by; do not match the `message`, which can change:
   *
   * | `code` | `reason` | the failure |
   * | --- | --- | --- |
   * | `refused` | `scope_not_empty` | `'create'`: the scope already holds documents |
   * | `invalid_argv` | `empty_bundle` | `'replace'`: the bundle holds no documents |
   * | `refused` | `scope_changed` | `'replace'`: after the scope was listed, another writer changed or deleted a key this replace writes or deletes; nothing was written, run it again |
   * | `invalid_argv` | `bundle_too_large` | the content is over `limits.importBytes` |
   * | `invalid_argv` | `invalid_bundle_document` | a document breaks a rule: a key outside the naming rules, a digest that is not its content's, a key that appears twice, or a board order over its size |
   *
   * Every other failure has no `reason` property (`'reason' in error` is
   * false): a refusal of `beforeCommand`, a store failure, a format version
   * this engine does not take, a value that is no bundle, a document of a
   * version 1 bundle whose names make no key (the same problem in a version
   * 2 bundle is `invalid_bundle_document`), and a `mode`, a `dryRun` or a
   * key of a replace call that is refused. Take "no reason"
   * as a valid answer: a later version can add a code to such a failure,
   * never change the meaning of one above.
   */
  importScope(call: HostImportCall): Promise<HostImportReport>
}

/**
 * Build one host kit for every user over a store port, optional hooks and the
 * HTTP door's optional inputs. A store without `list`, `read` or `commit`
 * throws synchronously, naming the missing functions; so does an unknown
 * option (an `actor` included: the identity comes with every call), a
 * `scopes` that is not a function, an `authMethods` or `pushTransports` of
 * another shape, or a `limits.importBytes` that is not a positive integer.
 */
export function createHost(options: CreateHostOptions): Host

/**
 * The `speclink` tool for a host's Agent framework. The model fills
 * `inputSchema` (`argv`, optional `stdin`); the host adds the identity and
 * scope from its own session and calls `host.dispatch`. The description lists
 * every verb with an engine command, from the same definition the parser
 * uses; the verbs that need a checkout are not in it.
 */
export const agentTool: {
  readonly name: 'speclink'
  readonly description: string
  readonly inputSchema: Readonly<Record<string, unknown>>
}

/**
 * Every command name `beforeCommand` and a store port's `commit` can be
 * given, each once: the engine's commands, and the host kit's own three,
 * `scope-export` (`host.exportScope`), `scope-import` (`host.importScope`)
 * and `put-board-order` (the board order write of `host.handleRequest`).
 * Check that your permission table covers each name, so a command a new
 * version adds is not refused by mistake. The order is no contract, and the
 * list says nothing about a command but its name: whether it writes is not
 * here, since a command that writes nothing can still give whole documents.
 */
export const commandNames: readonly string[]

/** A version conflict for a store port's `commit` to throw. */
export class StoreConflictError extends Error {
  readonly code: 'conflict'
  readonly key: string
  /** The document's current version, or null when it does not exist. Any other value is refused as an `internal` failure. */
  readonly actual: number | null
  constructor(key: string, actual: number | null)
}

/** A temporary failure for a store port's function to throw; the caller can retry. */
export class StoreUnavailableError extends Error {
  readonly code: 'unavailable'
  constructor(message: string)
}

export interface ConformanceHarness {
  /** The store port under test. */
  store: StorePort
  /** A new, unused scope on every call. */
  newScope(): MaybePromise<string>
  /** OPTIONAL: the events the store kept for `scope`, oldest first, each with its commit's version. Without it, `events-with-commit` is skipped. */
  recordedEvents?(scope: string): MaybePromise<{ name: string; payload: unknown; version: number }[]>
}

export interface ConformanceCheck {
  /** The check's stable name, the same as in the Rust suite (e.g. `conflict-stale-version`). */
  name: string
  outcome: 'passed' | 'failed' | 'skipped'
  detail?: string
}

export interface ConformanceReport {
  /** True when no check failed. */
  passed: boolean
  checks: ConformanceCheck[]
}

/**
 * Run the store contract's conformance checks — the same sixteen checks the
 * Rust implementations run — on a host's store port. Runs on a background
 * thread; the two concurrent checks call the store from several threads at
 * once, so they test the host's transactions.
 */
export function runStoreConformance(harness: ConformanceHarness): Promise<ConformanceReport>

/** The render matrix: target × invocation × specInCheckout. */
export interface RenderOptions {
  target: 'claude' | 'codex' | 'copilot' | 'pi' | 'cursor' | 'neutral'
  /** How the harness executes speclink verbs. Default: `'cli'`. */
  invocation?: 'cli' | 'tool-call'
  /**
   * Whether the spec folder is in the checkout the skill is used in. Default:
   * `true` — the content a local project gets. Pass `false` for a remote
   * project, or when a host embeds the engine and has no local spec files: the
   * rendered skill then names no spec-folder path and no git step on spec files.
   * A value that is not a boolean throws.
   */
  specInCheckout?: boolean
  /** Spec directory name substituted into rendered content. Default: `"speclink"`. */
  specDir?: string
  /** Neutral target only: the harness name (`{{TOOL}}` substitution). Default: `"speclink"`. */
  toolName?: string
}

export interface SkillInfo {
  name: string
  description: string
}

export const skills: {
  /** The skill registry: names and descriptions of every generated skill. */
  list(): SkillInfo[]
  /**
   * Render one skill's SKILL.md content for a matrix point — identical to
   * what `speclink init` (or `speclink init --store remote`, for
   * `specInCheckout: false`) generates for equivalent parameters.
   */
  render(name: string, options: RenderOptions): string
}
