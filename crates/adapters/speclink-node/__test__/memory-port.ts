// A test-only JS store port: the three functions of the store contract over
// an in-memory map, with a switch for Promise results and a broken variant
// that ignores `expected`. Not exported by the SDK — a host writes its own.

export type PortDoc = { content: string; version: number; modifiedAt: number }
export type PortOp =
  | { op: 'put'; key: string; content: string; expected: number | null }
  | { op: 'delete'; key: string; expected: number }
export type PortCommit = {
  actor: { id: string; displayName: string }
  command: string
  at: number
  ops: PortOp[]
  events: { name: string; payload: unknown }[]
}
export type KeptEvent = { name: string; payload: unknown; version: number }

type ScopeState = { version: number; docs: Map<string, PortDoc>; events: KeptEvent[] }

export type MemoryPortOptions = {
  /** Every function returns a Promise that settles a little later. */
  async?: boolean
  /** The commit writes whatever it gets: the store a conformance run must fail. */
  ignoreExpected?: boolean
}

/** Byte order of two keys (UTF-8), as the contract orders them. */
function byBytes(a: string, b: string): number {
  return Buffer.compare(Buffer.from(a, 'utf8'), Buffer.from(b, 'utf8'))
}

export function memoryPort(options: MemoryPortOptions = {}) {
  const scopes = new Map<string, ScopeState>()
  const commits: { scope: string; commit: PortCommit }[] = []
  const calls = { list: 0, read: 0, commit: 0 }
  const state = (scope: string): ScopeState => {
    let s = scopes.get(scope)
    if (!s) {
      s = { version: 0, docs: new Map(), events: [] }
      scopes.set(scope, s)
    }
    return s
  }
  // The work runs when the result settles, in one go: a read or a commit
  // never sees half of another commit, also in the async form.
  function settle<T>(work: () => T): T | Promise<T> {
    if (!options.async) return work()
    return new Promise<T>((resolve, reject) =>
      setTimeout(() => {
        try {
          resolve(work())
        } catch (e) {
          reject(e)
        }
      }, 1),
    )
  }

  const store = {
    list(scope: string, prefix: string) {
      calls.list += 1
      return settle(() => {
        const s = state(scope)
        const entries = [...s.docs.entries()]
          .filter(([key]) => key.startsWith(prefix))
          .sort(([a], [b]) => byBytes(a, b))
          .map(([key, doc]) => ({ key, version: doc.version, modifiedAt: doc.modifiedAt }))
        return { version: s.version, entries }
      })
    },
    read(scope: string, keys: string[]) {
      calls.read += 1
      return settle(() => {
        const s = state(scope)
        return keys.map((key) => {
          const doc = s.docs.get(key)
          return doc ? { ...doc } : null
        })
      })
    },
    commit(scope: string, commit: PortCommit) {
      calls.commit += 1
      commits.push({ scope, commit })
      return settle(() => {
        const s = state(scope)
        if (!options.ignoreExpected) {
          for (const op of commit.ops) {
            const doc = s.docs.get(op.key)
            const actual = doc ? doc.version : null
            if (op.expected !== actual) {
              throw Object.assign(new Error(`conflict on ${op.key}`), { code: 'conflict', key: op.key, actual })
            }
          }
        }
        const version = s.version + 1
        for (const op of commit.ops) {
          if (op.op === 'put') s.docs.set(op.key, { content: op.content, version, modifiedAt: commit.at })
          else s.docs.delete(op.key)
        }
        for (const event of commit.events) s.events.push({ ...event, version })
        s.version = version
        return version
      })
    },
  }

  return {
    store,
    commits,
    calls,
    recordedEvents: (scope: string): KeptEvent[] => state(scope).events.map((e) => ({ ...e })),
    /**
     * Write `docs` into `scope` in one commit, outside any dispatch. `version`
     * sets the commit's scope version (it must be larger than the current one).
     */
    seed(scope: string, docs: Record<string, string>, options: { version?: number; at?: number } = {}) {
      const s = state(scope)
      const version = options.version ?? s.version + 1
      const at = options.at ?? 1_700_000_000_000
      for (const [key, content] of Object.entries(docs)) s.docs.set(key, { content, version, modifiedAt: at })
      s.version = version
    },
    doc: (scope: string, key: string): PortDoc | undefined => state(scope).docs.get(key),
    version: (scope: string): number => state(scope).version,
  }
}
