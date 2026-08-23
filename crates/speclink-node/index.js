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
  if (store.type === 'fs') {
    if (typeof store.root !== 'string') {
      throw new TypeError("createEngine: fs store requires a string 'root'")
    }
    return new Engine(binding.engineFromFs(store.root, store.specDir))
  }
  return new Engine(binding.engineFromStore(store, makeInvoker(store)))
}

/** Skill knowledge: list the registry, render one SKILL.md for a matrix point. */
const skills = {
  list: () => binding.skillsList(),
  render: (name, options) => binding.skillsRender(name, options),
}

module.exports = { createEngine, skills }
