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
    if (!Array.isArray(argv) || argv.some((a) => typeof a !== 'string')) {
      throw new TypeError('dispatch(argv): argv must be an array of strings')
    }
    const envelope = await this.#native.dispatch(argv, options?.stdin)
    if (envelope.ok) return envelope.value
    const err = new Error(envelope.message)
    err.code = envelope.code
    throw err
  }
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
 * The generic store invoker the native bridge calls through one
 * ThreadsafeFunction: executes `store[method](...args)` (plain values and
 * Promises both accepted, synchronous throws captured) and settles exactly
 * once with `(error, value)`.
 */
function makeInvoker(store) {
  return (method, args, settle) => {
    if (typeof store[method] !== 'function') {
      // Optional methods (e.g. claim) surface as a marker the native side
      // translates into a semantic "not supported by this store" error.
      settle({ message: `store does not implement ${method}`, code: '__missing__' }, null)
      return
    }
    Promise.resolve()
      .then(() => store[method](...args))
      .then(
        (value) => settle(null, value === undefined ? null : value),
        (e) =>
          settle(
            {
              message: e && e.message !== undefined ? String(e.message) : String(e),
              code: e && e.code != null ? String(e.code) : null,
            },
            null,
          ),
      )
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
 * The invoker the native port bridge calls for a host's store port: it runs
 * `target[method](...args)` (a value or a Promise) and always settles with a
 * value — `{ ok: true, value }`, or `{ ok: false, error }` with the thrown
 * object's `code`, `message`, `key` and `actual` — so the native side reads the
 * fields of any thrown object, an Error or a plain one.
 */
function makePortInvoker(target) {
  return (method, args, settle) => {
    Promise.resolve()
      .then(() => target[method](...args))
      .then(
        (value) => settle(null, { ok: true, value: value === undefined ? null : value }),
        (e) => settle(null, { ok: false, error: thrownFields(e) }),
      )
  }
}

function thrownFields(e) {
  const isObject = e !== null && typeof e === 'object'
  return {
    code: isObject && e.code != null ? String(e.code) : null,
    message: isObject && e.message !== undefined ? String(e.message) : String(e),
    key: isObject && typeof e.key === 'string' ? e.key : null,
    actual: isObject && typeof e.actual === 'number' ? e.actual : null,
  }
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
    list: (...args) => store.list(...args),
    read: (...args) => store.read(...args),
    commit: (...args) => store.commit(...args),
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
    if (!Array.isArray(argv) || argv.some((a) => typeof a !== 'string')) {
      throw new TypeError('host.dispatch: argv must be an array of strings')
    }
    // A field that is not a string reads as empty, which the native side
    // refuses with invalid_argv before any store call.
    const text = (value) => (typeof value === 'string' ? value : '')
    const identity = call.identity
    const envelope = await this.#native.dispatch(
      text(identity?.id),
      text(identity?.displayName),
      text(call.scope),
      argv,
      call.stdin,
    )
    if (envelope.ok) return envelope.value
    const err = new Error(envelope.message)
    err.code = envelope.code
    throw err
  }
}

/**
 * Build a host kit over a store port (`list`, `read`, `commit`) and optional
 * hooks (`beforeCommand`, `afterCommit`). One host serves every user: each
 * `dispatch` brings its own identity and scope.
 */
function createHost(options) {
  const target = portFunctions(options?.store, 'createHost')
  const hooks = options?.hooks ?? {}
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
  const hookTarget = {
    beforeCommand: (info) => hooks.beforeCommand(info),
    afterCommit: (info) => hooks.afterCommit(info),
  }
  return new Host(
    binding.hostFromPort(
      makePortInvoker(target),
      makePortInvoker(hookTarget),
      typeof hooks.beforeCommand === 'function',
      typeof hooks.afterCommit === 'function',
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
  target.newScope = () => harness.newScope()
  if (keepsEvents) target.recordedEvents = (scope) => harness.recordedEvents(scope)
  return binding.runStoreConformance(makePortInvoker(target), keepsEvents)
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
