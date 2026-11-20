'use strict'

// Public entry of @speclink/engine. The generated `binding.js` loads the
// platform-specific native addon; this wrapper shapes the public API:
// createEngine and createHost, Promise-based dispatch, and Error objects
// carrying a semantic `code` property.

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

  /** Every document of the local project as a bundle by key, for host.importScope. */
  async exportScope() {
    return JSON.parse(unwrap(await this.#native.exportScope()))
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
  // Only an export or an import failure with a code has one: `'reason' in
  // err` is false for every other failure.
  if ('reason' in envelope) err.reason = envelope.reason
  throw err
}

/**
 * The invoker the native side calls through one ThreadsafeFunction, for the
 * host kit's store port, its hooks and the conformance harness alike: runs
 * `target[method](...args)` (plain values and Promises both accepted,
 * synchronous throws captured) and settles exactly once with `(error, value)`.
 * The error carries the thrown object's `code`, `message`, `key` and `actual`.
 * Every caller builds `target` with the functions the native side names.
 */
function makeInvoker(target) {
  return (method, args, settle) => {
    // The error crosses as JSON text, like an `asJson` result.
    const fail = (fields) => settle(JSON.stringify(fields), null)
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

/**
 * Build an engine over a local project folder:
 * `{ store: { type: 'fs', root, specDir? } }`. A store of your own goes to
 * `createHost`, as three functions (list, read, commit).
 *
 * `options.actor` ("Name <email>") is the operator every stamp this engine
 * writes is attributed to; without it the engine uses the project's git
 * identity.
 */
function createEngine(options) {
  const store = options?.store
  if (!store || typeof store !== 'object') {
    throw new TypeError(
      "createEngine: options.store must be an object — pass { type: 'fs', root }, or use createHost for a custom store (list, read, commit)",
    )
  }
  // `type` is the only property read before this check, so an object of
  // another shape gets none of its methods called.
  if (store.type !== 'fs') {
    throw new TypeError(
      "createEngine: only { type: 'fs', root } is accepted — use createHost for a custom store (list, read, commit)",
    )
  }
  // The operator identity is bound here, once — dispatch never takes one.
  const actor = options?.actor
  if (actor !== undefined && typeof actor !== 'string') {
    throw new TypeError("createEngine: options.actor must be a string (\"Name <email>\")")
  }
  // Each property is read once: the value that was checked is the value used.
  const { root, specDir } = store
  if (typeof root !== 'string') {
    throw new TypeError("createEngine: fs store requires a string 'root'")
  }
  return new Engine(binding.engineFromFs(root, specDir, actor))
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

/**
 * The free fields a host attaches to one call, as JSON text for the native
 * side. Absent or null is no field. Anything but a plain object whose values
 * are all strings crosses as `null`, which the native side refuses. The check
 * is here because JSON would hide the mistake: it drops a key whose value is
 * `undefined`, a function or a symbol key, and writes a Date as text.
 */
function fieldsText(fields) {
  if (fields === undefined || fields === null) return undefined
  const proto = typeof fields === 'object' ? Object.getPrototypeOf(fields) : undefined
  const plain = proto === Object.prototype || proto === null
  const strings =
    plain &&
    Object.getOwnPropertySymbols(fields).length === 0 &&
    Object.values(fields).every((value) => typeof value === 'string')
  return strings ? JSON.stringify(fields) : 'null'
}

class Host {
  #native

  constructor(native) {
    this.#native = native
  }

  /**
   * Run one speclink verb for `identity` on `scope`. `argv`, `stdin`, the
   * result and the errors are those of `engine.dispatch`. `fields` are free
   * fields from your own code: written when the verb creates a change or a
   * discussion, ignored by every other verb.
   */
  async dispatch(call) {
    const argv = call?.argv
    checkArgv(argv, 'host.dispatch')
    return unwrap(await this.#native.dispatch(...whoAndWhere(call), argv, call.stdin, fieldsText(call.fields)))
  }

  /**
   * Answer one HTTP request of the original CLI or Speclink Desktop for
   * `identity` (null when the host could not verify one). Resolves to
   * `{ status, headers, body }`, or to null when the path is not the door's.
   * `extras.fields` are free fields from your own code, as for `dispatch`.
   */
  async handleRequest(request, identity, extras) {
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
    const who = identity == null ? [undefined, undefined] : [text(identity.id), text(identity.displayName)]
    if (extras !== undefined && extras !== null && typeof extras !== 'object') {
      throw new TypeError('host.handleRequest: extras must be an object')
    }
    // A misspelled key would drop the fields without a word.
    const unknown = Object.keys(extras ?? {}).filter((name) => name !== 'fields')
    if (unknown.length > 0) {
      throw new TypeError(`host.handleRequest: unknown extras: ${unknown.join(', ')} (the only one is fields)`)
    }
    const line = JSON.stringify({ method, path, query: query ?? undefined, headers })
    const response = unwrap(await this.#native.handleRequest(line, bytes, ...who, fieldsText(extras?.fields)))
    if (response === null) return null
    return { status: response.status, headers: response.headers, body: new TextEncoder().encode(response.body) }
  }

  /**
   * Every document of `scope`, for `identity`: `{ formatVersion: 2,
   * scopeVersion, documents: [{ key, content, digest }] }`, in key order.
   */
  async exportScope(call) {
    // The bundle crosses as JSON text: one string, and exact numbers.
    return JSON.parse(unwrap(await this.#native.exportScope(...whoAndWhere(call))))
  }

  /**
   * Write `bundle` (of either version) into `scope`, which must be empty,
   * for `identity`. Resolves to `{ scopeVersion, documents }`.
   */
  async importScope(call) {
    // What is not a bundle crosses as `null`, which the native side refuses
    // with invalid_argv.
    const bundle = JSON.stringify(call?.bundle) ?? 'null'
    return unwrap(await this.#native.importScope(...whoAndWhere(call), bundle))
  }
}

/**
 * A field of an identity or a scope as it crosses: one that is not a string
 * reads as empty, which the native side refuses with invalid_argv before any
 * store call.
 */
const text = (value) => (typeof value === 'string' ? value : '')

/** The identity's id and display name and the scope of one call. */
function whoAndWhere(call) {
  return [text(call?.identity?.id), text(call?.identity?.displayName), text(call?.scope)]
}

const HOST_OPTIONS = ['store', 'hooks', 'scopes', 'authMethods', 'pushTransports', 'skills', 'limits']

/**
 * `createHost`'s `limits`, checked here for what the native side cannot see:
 * it would read an array as the limits in order, and JSON would write NaN as
 * null, which reads as "no limit given". The native side refuses a limit it
 * does not know.
 */
function checkLimits(limits) {
  if (limits === undefined) return
  if (limits === null || typeof limits !== 'object' || Array.isArray(limits)) {
    throw new TypeError('createHost: limits must be an object')
  }
  const { importBytes } = limits
  if (importBytes !== undefined && !(Number.isSafeInteger(importBytes) && importBytes > 0)) {
    throw new TypeError('createHost: limits.importBytes must be a positive integer (bytes)')
  }
}

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
 * The second argument of one `beforeCommand` call, and the function that
 * ends it. `read` runs a verb that writes nothing, as the identity and on the
 * scope of this call, and does not ask the hook again. A `read` kept past the
 * hook's answer is refused here, before the native side.
 */
function readTools(native, info) {
  // Taken before the hook runs: the hook cannot change who reads, or where.
  const who = whoAndWhere(info)
  let open = true
  const read = async (argv) => {
    if (!open) {
      const err = new Error('read works only while its beforeCommand call runs, and that call has ended')
      err.code = 'invalid_argv'
      throw err
    }
    checkArgv(argv, 'read(argv)')
    return unwrap(await native.read(...who, argv))
  }
  return [{ read }, () => (open = false)]
}

/**
 * Build a host kit over a store port (`list`, `read`, `commit`), optional
 * hooks (`beforeCommand`, `afterCommit`) and the HTTP door's optional inputs
 * (`scopes`, `authMethods`, `pushTransports`, `skills`, `limits`). One host
 * serves every user: each call brings its own identity.
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
  checkLimits(options.limits)
  // The hooks are called only during a call of the host built below.
  let native
  const hookTarget = {
    beforeCommand: asJson(async (info) => {
      const [tools, close] = readTools(native, info)
      try {
        return decisionOf(await hooks.beforeCommand(info, tools))
      } finally {
        close()
      }
    }),
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
    limits: options.limits,
  })
  native = binding.hostFromPort(
    makeInvoker(target),
    makeInvoker(hookTarget),
    typeof hooks.beforeCommand === 'function',
    typeof hooks.afterCommit === 'function',
    typeof options.scopes === 'function',
    doorOptions,
  )
  return new Host(native)
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

/**
 * Every command name `beforeCommand` and a store's `commit` can be given:
 * the engine's commands and the host kit's own `scope-export`,
 * `scope-import` and `put-board-order`. Taken once at load and frozen.
 */
const commandNames = Object.freeze(binding.commandNames())

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
  commandNames,
  skills,
  runStoreConformance,
  StoreConflictError,
  StoreUnavailableError,
}
