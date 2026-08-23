/**
 * @speclink/engine — Node.js bindings for the Speclink spec-driven
 * development engine.
 *
 * The engine core is synchronous Rust; every `dispatch` runs on a background
 * worker thread and returns a Promise — the JS event loop is never blocked.
 * WARNING: never call back into the engine synchronously from inside a Store
 * method (a dispatch issued there and awaited synchronously would deadlock
 * against the store bridge).
 */

/** A Store method may return its value directly or as a Promise. */
export type MaybePromise<T> = T | Promise<T>

/** Per-change metadata (the parsed `.openspec.yaml` fields). */
export interface ChangeMeta {
  schema?: string
  created?: string
  createdBy?: string
  createdWith?: string
  /** Slug of the discussion this change was promoted from. */
  fromDiscussion?: string
}

/** A change as the store reports it. */
export interface ChangeInfo {
  name: string
  /**
   * Display location of the change's documents (store-logical, e.g.
   * `changes/<name>`). Optional — defaults to `changes/<name>`.
   */
  dir?: string
  meta?: ChangeMeta
}

/** A discussion document as stored. */
export interface DiscussionDoc {
  slug: string
  text: string
  /** Display location of the document (store-logical). */
  path: string
  archived: boolean
}

/**
 * The storage interface the engine reads and writes spec documents through —
 * one-to-one with the Rust core's `Store` trait (camelCase). Artifact
 * identifiers are schema-defined output paths relative to a change (e.g.
 * `proposal.md`, `specs/<capability>/spec.md`).
 *
 * Every method is required (createEngine fails fast listing missing ones);
 * `claim` is the one optional extension — ownership is a team-system concept
 * the host adjudicates.
 */
export interface Store {
  // --- changes ---
  /** Active changes with metadata, sorted by name. Missing storage → empty list. */
  listChanges(): MaybePromise<ChangeInfo[]>
  findChange(name: string): MaybePromise<ChangeInfo | null>
  changeExists(name: string): MaybePromise<boolean>
  /** Create a change with the given raw metadata document; returns its display location. */
  createChange(name: string, metaText: string): MaybePromise<string>
  /** Last-modified time in whole seconds since the Unix epoch (sort key). Missing change → 0. */
  updatedAtSecs(name: string): MaybePromise<number>

  // --- artifacts ---
  readArtifact(change: string, artifact: string): MaybePromise<string | null>
  /** Create or overwrite an artifact; returns its display location. */
  writeArtifact(change: string, artifact: string, content: string): MaybePromise<string>
  /** Whether an artifact exists (an empty document counts). */
  artifactExists(change: string, artifact: string): MaybePromise<boolean>
  /**
   * OPTIONAL: delete a document inside an active change. Only the review and
   * verify stations delete anything (stamping removes the ticket), so a store
   * without it works everywhere else and fails loud there.
   */
  deleteArtifact?(change: string, artifact: string): MaybePromise<void>
  /**
   * OPTIONAL: the raw metadata document of an active change (the
   * `.openspec.yaml` text). Stamping is a read-modify-write of this document,
   * so the review/verify stamp verbs require BOTH methods together with
   * `deleteArtifact` — a store missing any of the three is refused before the
   * stamp touches anything. All other verbs never call them.
   */
  readChangeMeta?(name: string): MaybePromise<string | null>
  /** OPTIONAL: overwrite the raw metadata document — see `readChangeMeta`. */
  writeChangeMeta?(name: string, content: string): MaybePromise<void>

  // --- delta specs ---
  /** Capability names that have a delta spec document in the change, sorted. */
  deltaCapabilities(change: string): MaybePromise<string[]>
  /** Whether the change has any capability container at all. */
  hasCapabilityDirs(change: string): MaybePromise<boolean>

  // --- canonical specs ---
  listCanonicalCapabilities(): MaybePromise<string[]>
  canonicalSpecExists(capability: string): MaybePromise<boolean>
  readCanonicalSpec(capability: string): MaybePromise<string | null>
  writeCanonicalSpec(capability: string, content: string): MaybePromise<void>
  /** Display location of a capability's canonical spec. */
  canonicalSpecPath(capability: string): MaybePromise<string>

  // --- archive ---
  archivedChangeExists(datedName: string): MaybePromise<boolean>
  /** Move an active change into the archive under its dated name. */
  archiveChange(name: string, datedName: string): MaybePromise<void>
  readArchivedMeta(datedName: string): MaybePromise<string | null>
  writeArchivedMeta(datedName: string, content: string): MaybePromise<void>

  // --- discussions ---
  liveDiscussionExists(slug: string): MaybePromise<boolean>
  archivedDiscussionExists(slug: string): MaybePromise<boolean>
  liveDiscussionPath(slug: string): MaybePromise<string>
  readLiveDiscussion(slug: string): MaybePromise<string | null>
  writeLiveDiscussion(slug: string, content: string): MaybePromise<string>
  deleteLiveDiscussion(slug: string): MaybePromise<void>
  /** Resolve a slug to its document: live first, then the newest archived candidate. */
  readDiscussion(slug: string): MaybePromise<DiscussionDoc | null>
  listLiveDiscussions(): MaybePromise<DiscussionDoc[]>
  /** Archived discussions, ordered by stored name (archive date order). */
  listArchivedDiscussions(): MaybePromise<DiscussionDoc[]>
  /**
   * Move a live discussion into the archive, named by its creation date;
   * returns the stored archive name, or null when no live document exists.
   */
  archiveDiscussion(slug: string, created: string): MaybePromise<string | null>

  // --- workflow config / shared vocabulary ---
  /** Raw workflow configuration document (config.yaml content), or null. */
  readWorkflowConfig(): MaybePromise<string | null>
  /** The project's LANGUAGE document (shared vocabulary), or null. */
  readLanguage(): MaybePromise<string | null>

  /**
   * OPTIONAL: claim a change for implementation. The host adjudicates
   * ownership (see the verb contract); a conflict should reject with an
   * Error whose `code` is the 409 reason (e.g. `ownership_lost`) and whose
   * message states who holds the change and what to do. Without this method,
   * `dispatch(['claim', …])` fails loud.
   */
  claim?(name: string): MaybePromise<unknown>
}

/** The built-in filesystem store: a local project root with an `openspec/` tree. */
export interface FsStoreOptions {
  type: 'fs'
  /** Project root (the directory containing the spec directory). */
  root: string
  /** Spec directory name relative to root. Default: `"openspec"`. */
  specDir?: string
}

export interface CreateEngineOptions {
  store: FsStoreOptions | Store
  /**
   * The operator identity every stamp this engine writes is attributed to
   * (`created_by`, `reviewed_by`, `verified_by`), in `"Name <email>"` form.
   *
   * Bound once at construction — one engine instance, one identity. A
   * multi-tenant host builds one engine per request (or per identity); there
   * is deliberately no way to pass an identity at dispatch time, so a caller
   * cannot claim someone else's. Who may claim which identity is the host's
   * call: the SDK only takes the result.
   *
   * Omitted (or blank after trimming): the fs form falls back to the
   * workspace's git identity, a host store stamps no identity at all.
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
   * Resolves to the same structured object the CLI prints with `--json`
   * (camelCase); verbs without a `--json` form resolve to `{ output: string }`.
   * The verb surface and payload contract are described in
   * docs/platform-architecture.zh-TW.md.
   *
   * Rejects with an `Error` whose `message` is the CLI's semantic message and
   * whose `code` property classifies the failure (`error`, `not_found`,
   * `invalid_argv`, a host store's 409 reason such as `ownership_lost`, or
   * `store_error`/`panic`).
   */
  dispatch(argv: string[], options?: DispatchOptions): Promise<unknown>
}

/**
 * Build an engine over the built-in fs store (`{ type: 'fs', root }`) or a
 * host-implemented Store object. A Store object missing required methods
 * throws synchronously, listing the missing method names.
 */
export function createEngine(options: CreateEngineOptions): Engine

/** The render matrix: target × invocation. */
export interface RenderOptions {
  target: 'claude' | 'codex' | 'neutral'
  /** How the harness executes speclink verbs. Default: `'cli'`. */
  invocation?: 'cli' | 'tool-call'
  /** Spec directory name substituted into rendered content. Default: `"openspec"`. */
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
   * what `speclink init` generates for equivalent parameters.
   */
  render(name: string, options: RenderOptions): string
}
