'use strict'

// Public entry of @speclink/engine. The generated `binding.js` loads the
// platform-specific native addon; this wrapper shapes the public API:
// createEngine forms, Promise-based dispatch, and Error objects carrying a
// semantic `code` property.

const binding = require('./binding.js')

class Engine {
  #native

  constructor(native) {
    this.#native = native
  }

  /**
   * Dispatch one speclink verb. `argv` mirrors the CLI vocabulary
   * (e.g. ['list', '--json']); `options.stdin` carries content for verbs
   * that read stdin in the CLI (e.g. `new artifact --stdin`).
   */
  async dispatch(argv, options) {
    checkArgv(argv, 'dispatch(argv)')
    return unwrap(await this.#native.dispatch(argv, options?.stdin))
  }
}

function checkArgv(argv, caller) {
  if (!Array.isArray(argv) || argv.some((a) => typeof a !== 'string')) {
    throw new TypeError(`${caller}: argv must be an array of strings`)
  }
}

/** The native envelope as a value, or an Error carrying its `code`. */
function unwrap(envelope) {
  if (envelope.ok) return envelope.value
  const err = new Error(envelope.message)
  err.code = envelope.code
  throw err
}

/**
 * Build an engine over one of the two storage forms:
 * - `{ store: { type: 'fs', root, specDir? } }` — built-in filesystem store
 * - `{ store: <object implementing the Store interface> }` — host storage
 *
 * `options.actor` ("Name <email>") is the operator every stamp this engine
 * writes is attributed to. Both forms accept it; without it the fs form falls
 * back to the workspace's git identity and a host store stamps nothing.
 */
/**
 * The invoker the native bridge calls through one ThreadsafeFunction, for the
 * engine's Store object and for the host kit's store port and hooks alike:
 * runs `target[method](...args)` (plain values and Promises both accepted,
 * synchronous throws captured) and settles exactly once with `(error, value)`.
 * The error carries the thrown object's `code`, `message`, `key` and `actual`.
 */
function makeInvoker(target) {
  return (method, args, settle) => {
    // The error crosses as JSON text, like an `asJson` result.
    const fail = (fields) => settle(JSON.stringify(fields), null)
    if (typeof target[method] !== 'function') {
      // Optional methods (e.g. claim) surface as a marker the native side
      // translates into a semantic "not supported by this store" error.
      fail({ message: `store does not implement ${method}`, code: '__missing__' })
      return
    }
    Promise.resolve()
      .then(() => target[method](...args))
      .then(
        (value) => settle(null, value === undefined ? null : value),
        (e) => fail(thrownFields(e)),
      )
      // The settle must happen once, also when the thrown value cannot be read.
      .catch(() => fail({ message: 'the function failed with a value that cannot be read' }))
  }
}

/** Text of any value; a value whose conversion throws gets a fixed text. */
function asText(value) {
  try {
    return String(value)
  } catch {
    return 'a value that cannot be shown as text'
  }
}

function thrownFields(e) {
  const isObject = e !== null && (typeof e === 'object' || typeof e === 'function')
  const actual = isObject ? e.actual : undefined
  return {
    code: isObject && e.code != null ? asText(e.code) : null,
    message: isObject && e.message !== undefined ? asText(e.message) : asText(e),
    key: isObject && typeof e.key === 'string' ? e.key : null,
    // JSON writes NaN and Infinity as null, which reads as "no document":
    // they cross as text instead, so the native side refuses them.
    actual: typeof actual === 'number' && !Number.isFinite(actual) ? String(actual) : (actual ?? null),
  }
}

function createEngine(options) {
  const store = options?.store
  if (!store || typeof store !== 'object') {
    throw new TypeError("createEngine: options.store is required — pass { type: 'fs', root } or a Store object")
  }
  // The operator identity is bound here, once — dispatch never takes one.
  const actor = options?.actor
  if (actor !== undefined && typeof actor !== 'string') {
    throw new TypeError("createEngine: options.actor must be a string (\"Name <email>\")")
  }
  if (store.type === 'fs') {
    if (typeof store.root !== 'string') {
      throw new TypeError("createEngine: fs store requires a string 'root'")
    }
    return new Engine(binding.engineFromFs(store.root, store.specDir, actor))
  }
  return new Engine(binding.engineFromStore(store, makeInvoker(store), actor))
}

/**
 * A function whose result crosses to the native side as JSON text. JSON keeps
 * every number exact (a tiny fraction stays a fraction), and a value JSON
 * cannot hold — a cycle, a BigInt — fails here as a thrown error instead of
 * reaching the native conversion.
 */
function asJson(fn) {
  return async (...args) => JSON.stringify(await fn(...args)) ?? 'null'
}

const PORT_FUNCTIONS = ['list', 'read', 'commit']

/** The store port's three functions, checked once; missing names are listed. */
function portFunctions(store, caller) {
  if (!store || typeof store !== 'object') {
    throw new TypeError(`${caller}: store must be an object with list, read and commit`)
  }
  const missing = PORT_FUNCTIONS.filter((name) => typeof store[name] !== 'function')
  if (missing.length > 0) {
    throw new TypeError(`${caller}: store is missing required functions: ${missing.join(', ')}`)
  }
  return {
    list: asJson((...args) => store.list(...args)),
    read: asJson((...args) => store.read(...args)),
    commit: asJson((...args) => store.commit(...args)),
  }
}

/** A store's version conflict: `actual` is the document's current version, or null when it does not exist. */
class StoreConflictError extends Error {
  constructor(key, actual) {
    super(`conflict on ${key}`)
    this.name = 'StoreConflictError'
    this.code = 'conflict'
    this.key = key
    this.actual = actual === undefined ? null : actual
  }
}

/** A temporary store failure; the caller can retry. */
class StoreUnavailableError extends Error {
  constructor(message) {
    super(message)
    this.name = 'StoreUnavailableError'
    this.code = 'unavailable'
  }
}

const HOOK_NAMES = ['beforeCommand', 'afterCommit']

class Host {
  #native

  constructor(native) {
    this.#native = native
  }

  /**
   * Run one speclink verb for `identity` on `scope`. `argv`, `stdin`, the
   * result and the errors are those of `engine.dispatch`.
   */
  async dispatch(call) {
    const argv = call?.argv
    checkArgv(argv, 'host.dispatch')
    // A field that is not a string reads as empty, which the native side
    // refuses with invalid_argv before any store call.
    const text = (value) => (typeof value === 'string' ? value : '')
    const identity = call.identity
    return unwrap(
      await this.#native.dispatch(text(identity?.id), text(identity?.displayName), text(call.scope), argv, call.stdin),
    )
  }

  /**
   * Answer one HTTP request of the original CLI or Speclink Desktop for
   * `identity` (null when the host could not verify one). Resolves to
   * `{ status, headers, body }`, or to null when the path is not the door's.
   */
  async handleRequest(request, identity) {
    if (!request || typeof request !== 'object') {
      throw new TypeError('host.handleRequest: request must be an object')
    }
    const { method, path, query, headers, body } = request
    if (typeof method !== 'string' || typeof path !== 'string') {
      throw new TypeError('host.handleRequest: request.method and request.path must be strings')
    }
    if (query !== undefined && query !== null && typeof query !== 'string') {
      throw new TypeError('host.handleRequest: request.query must be a string')
    }
    if (headers === null || typeof headers !== 'object') {
      throw new TypeError('host.handleRequest: request.headers must be an object of strings')
    }
    if (body !== undefined && !(body instanceof Uint8Array)) {
      throw new TypeError('host.handleRequest: request.body must be a Uint8Array')
    }
    const bytes = body === undefined ? Buffer.alloc(0) : Buffer.from(body.buffer, body.byteOffset, body.byteLength)
    // A field that is not a string reads as empty, which the native side
    // refuses with invalid_argv before any store call.
    const text = (value) => (typeof value === 'string' ? value : '')
    const who = identity == null ? [undefined, undefined] : [text(identity.id), text(identity.displayName)]
    const fields = JSON.stringify({ method, path, query: query ?? undefined, headers })
    const response = unwrap(await this.#native.handleRequest(fields, bytes, ...who))
    if (response === null) return null
    return { status: response.status, headers: response.headers, body: new TextEncoder().encode(response.body) }
  }
}

const HOST_OPTIONS = ['store', 'hooks', 'scopes', 'authMethods', 'pushTransports', 'skills']

/**
 * The before-command answer as one of three plain shapes, so only these
 * cross to the native side: `{ allow: true }`, `{ allow: false, reason }`,
 * or `{ invalid }` naming what came back.
 */
function decisionOf(answer) {
  if (answer === undefined) return { allow: true }
  if (answer !== null && typeof answer === 'object') {
    if (answer.allow === true) return { allow: true }
    if (answer.allow === false && (answer.reason === undefined || typeof answer.reason === 'string')) {
      return { allow: false, reason: answer.reason ?? '' }
    }
    return { invalid: 'an object of another shape' }
  }
  return { invalid: answer === null ? 'null' : typeof answer === 'string' ? JSON.stringify(answer) : asText(answer) }
}

/**
 * Build a host kit over a store port (`list`, `read`, `commit`), optional
 * hooks (`beforeCommand`, `afterCommit`) and the HTTP door's optional inputs
 * (`scopes`, `authMethods`, `pushTransports`, `skills`). One host serves every user:
 * each `dispatch` and each `handleRequest` brings its own identity.
 */
function createHost(options) {
  const target = portFunctions(options?.store, 'createHost')
  // `actor` belongs to createEngine: a host gets the identity with every call.
  const unknownOptions = Object.keys(options).filter((name) => !HOST_OPTIONS.includes(name))
  if (unknownOptions.length > 0) {
    throw new TypeError(
      `createHost: unknown options: ${unknownOptions.join(', ')} (the options are ${HOST_OPTIONS.join(', ')}; the identity comes with every call)`,
    )
  }
  const hooks = options.hooks ?? {}
  if (typeof hooks !== 'object') {
    throw new TypeError('createHost: hooks must be an object')
  }
  // A misspelled hook would leave every call allowed without a word.
  const unknown = Object.keys(hooks).filter((name) => !HOOK_NAMES.includes(name))
  if (unknown.length > 0) {
    throw new TypeError(`createHost: unknown hooks: ${unknown.join(', ')} (the hooks are ${HOOK_NAMES.join(' and ')})`)
  }
  for (const name of HOOK_NAMES) {
    if (hooks[name] !== undefined && typeof hooks[name] !== 'function') {
      throw new TypeError(`createHost: hooks.${name} must be a function`)
    }
  }
  if (options.scopes !== undefined && typeof options.scopes !== 'function') {
    throw new TypeError('createHost: scopes must be a function')
  }
  const hookTarget = {
    beforeCommand: asJson(async (info) => decisionOf(await hooks.beforeCommand(info))),
    // Nothing reads what afterCommit returns, so none of it crosses back.
    afterCommit: async (info) => {
      await hooks.afterCommit(info)
    },
    scopes: asJson(async (identity) => options.scopes(identity)),
  }
  // The native side checks the shapes and throws on a field it does not know,
  // and on a skill name the engine does not have.
  const doorOptions = JSON.stringify({
    authMethods: options.authMethods,
    pushTransports: options.pushTransports,
    skills: options.skills,
  })
  return new Host(
    binding.hostFromPort(
      makeInvoker(target),
      makeInvoker(hookTarget),
      typeof hooks.beforeCommand === 'function',
      typeof hooks.afterCommit === 'function',
      typeof options.scopes === 'function',
      doorOptions,
    ),
  )
}

function deepFreeze(value) {
  if (value !== null && typeof value === 'object') {
    for (const inner of Object.values(value)) deepFreeze(inner)
    Object.freeze(value)
  }
  return value
}

/**
 * The tool a host registers in its own Agent framework: `name`, the
 * `description` the model reads (with the verb list), and the `inputSchema`
 * (`argv` and an optional `stdin`). Identity and scope are not in it: the host
 * adds them when it calls `host.dispatch`.
 */
const agentTool = deepFreeze(binding.agentToolDefinition())

/** Run the store contract's conformance checks on a host's JS store port. */
async function runStoreConformance(harness) {
  const target = portFunctions(harness?.store, 'runStoreConformance')
  if (typeof harness.newScope !== 'function') {
    throw new TypeError('runStoreConformance: newScope must be a function')
  }
  const keepsEvents = typeof harness.recordedEvents === 'function'
  target.newScope = asJson(() => harness.newScope())
  if (keepsEvents) target.recordedEvents = asJson((scope) => harness.recordedEvents(scope))
  return binding.runStoreConformance(makeInvoker(target), keepsEvents)
}

/** Skill knowledge: list the registry, render one SKILL.md for a matrix point. */
const skills = {
  list: () => binding.skillsList(),
  render: (name, options) => binding.skillsRender(name, options),
}

module.exports = {
  createEngine,
  createHost,
  agentTool,
  skills,
  runStoreConformance,
  StoreConflictError,
  StoreUnavailableError,
}
