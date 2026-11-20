// node-sdk「宿主的匯出與匯入方法」: host.exportScope and host.importScope over
// a JS store port, the hooks they go through, and createHost's import limit.
import { describe, expect, it } from 'vitest'

import { createHost, StoreUnavailableError } from '../index.js'
import { digestOf, rejection } from './helpers'
import { memoryPort } from './memory-port'

const API = '/api/speclink/v1'
const ALICE = { id: 'u_42', displayName: 'Alice' }
const CONFIG = 'schema: spec-driven\n'

/** Project alpha with one repo, web, on scope s1, writable. */
const ONE_REPO = [
  { key: 'alpha', name: 'Alpha', repos: [{ key: 'web', name: 'Web', scope: 's1', canWrite: true, canWritePolicy: true }] },
]

type Doc = { key: string; content: string; digest: string }
type Bundle = { formatVersion: number; scopeVersion?: number; documents: Doc[] }

const doc = (key: string, content: string): Doc => ({ key, content, digest: digestOf(content) })
const keyed = (documents: Doc[]): Bundle => ({ formatVersion: 2, documents })

/** How many commits `port` received for `scope`. */
const commitsTo = (port: ReturnType<typeof memoryPort>, scope: string) =>
  port.commits.filter((c) => c.scope === scope).length

function post(tail: string, body: Uint8Array) {
  return {
    method: 'POST',
    path: `${API}/projects/alpha/${tail}`,
    headers: { 'x-speclink-api-version': '1', 'content-type': 'application/json' },
    body,
  }
}

describe('host.exportScope and host.importScope', () => {
  it('exports a scope and imports it into another one, with one commit', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })

    const bundle = (await host.exportScope({ identity: ALICE, scope: 's1' })) as Bundle
    expect(bundle.formatVersion).toBe(2)
    expect(bundle.scopeVersion).toBe(port.version('s1'))
    expect(bundle.documents.length).toBeGreaterThan(0)
    for (const document of bundle.documents) {
      expect(Object.keys(document).sort()).toEqual(['content', 'digest', 'key'])
      expect(document.digest).toBe(digestOf(document.content))
    }
    const keys = bundle.documents.map((d) => d.key)
    expect(keys).toEqual([...keys].sort())
    expect(keys).toContain('changes/demo/.meta.yaml')

    const report = await host.importScope({ identity: ALICE, scope: 's2', bundle })

    expect(report).toEqual({ scopeVersion: port.version('s2'), documents: bundle.documents.length })
    const listed = (await host.dispatch({ identity: ALICE, scope: 's2', argv: ['list', '--json'] })) as {
      changes: { name: string }[]
    }
    expect(listed.changes.map((c) => c.name)).toEqual(['demo'])
    expect(commitsTo(port, 's2')).toBe(1)
    const imported = port.commits.find((c) => c.scope === 's2')!.commit
    expect(imported.command).toBe('scope-import')
    expect(imported.actor).toEqual({ id: 'u_42', displayName: 'Alice' })
    expect(imported.events.map((e) => e.name)).toEqual(['scope-imported'])
  })

  it('refuses an import into a scope that holds a change, with no commit', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: ALICE, scope: 's2', argv: ['new', 'change', 'other'] })
    const before = commitsTo(port, 's2')

    const error = await rejection(
      host.importScope({ identity: ALICE, scope: 's2', bundle: keyed([doc('config.yaml', CONFIG)]) }),
    )

    expect(error.code).toBe('refused')
    expect(commitsTo(port, 's2')).toBe(before)
  })

  it('refuses a bundle whose digest is not the content’s, naming the key', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    const wrong = { key: 'specs/auth/spec.md', content: '# auth\n', digest: digestOf('# other\n') }

    const error = await rejection(
      host.importScope({ identity: ALICE, scope: 's1', bundle: keyed([doc('config.yaml', CONFIG), wrong]) }),
    )

    expect(error.code).toBe('invalid_argv')
    expect(error.message).toContain('specs/auth/spec.md')
    expect(port.calls.commit).toBe(0)
  })

  it('refuses what is not a bundle with invalid_argv', async () => {
    const host = createHost({ store: memoryPort().store })
    const bundles: unknown[] = [undefined, null, 'a bundle', {}, { formatVersion: 3, documents: [] }, { formatVersion: 2 }]
    for (const bundle of bundles) {
      const error = await rejection(host.importScope({ identity: ALICE, scope: 's1', bundle: bundle as never }))
      expect(error.code, JSON.stringify(bundle)).toBe('invalid_argv')
    }
  })

  it('takes a first-version bundle too', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    const bundle = {
      formatVersion: 1,
      scope: { project: 'alpha', repo: 'web' },
      projectRevision: 7,
      documents: [
        { document: { type: 'workflowConfig' }, content: CONFIG, digest: digestOf(CONFIG) },
        { document: { type: 'changeMeta', change: 'demo' }, content: CONFIG, digest: digestOf(CONFIG) },
      ],
    }

    const report = await host.importScope({ identity: ALICE, scope: 's1', bundle })

    expect(report).toEqual({ scopeVersion: port.version('s1'), documents: 2 })
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toBe(CONFIG)
  })

  it('asks beforeCommand about scope-import, and a refusal rejects with the host’s reason', async () => {
    const port = memoryPort()
    const asked: { command: string; params: unknown }[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: (info) => {
          asked.push({ command: info.command, params: info.params })
          return info.command === 'scope-import' ? { allow: false, reason: '搬移凍結中' } : undefined
        },
      },
    })
    const bundle = keyed([doc('config.yaml', CONFIG), doc('LANGUAGE.md', 'TeamStore: 團隊儲存\n')])
    const bytes = Buffer.byteLength(CONFIG) + Buffer.byteLength('TeamStore: 團隊儲存\n')

    const error = await rejection(host.importScope({ identity: ALICE, scope: 's1', bundle }))

    expect(error.code).toBe('refused')
    expect(error.message).toBe('搬移凍結中')
    expect(asked).toEqual([{ command: 'scope-import', params: { documents: 2, bytes } }])
    expect(port.calls.commit).toBe(0)

    // An export is asked about under its own name.
    await host.exportScope({ identity: ALICE, scope: 's1' })
    expect(asked[1]).toEqual({ command: 'scope-export', params: {} })
  })

  it('tells afterCommit about the import', async () => {
    const port = memoryPort()
    const watched: { command: string; scopeVersion: number; events: string[] }[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        afterCommit: (info) => {
          watched.push({ command: info.command, scopeVersion: info.scopeVersion, events: info.events.map((e) => e.name) })
        },
      },
    })

    const report = (await host.importScope({
      identity: ALICE,
      scope: 's1',
      bundle: keyed([doc('config.yaml', CONFIG)]),
    })) as { scopeVersion: number }
    await new Promise((resolve) => setTimeout(resolve, 20))

    expect(watched).toEqual([{ command: 'scope-import', scopeVersion: report.scopeVersion, events: ['scope-imported'] }])
  })

  it('rejects an identity or a scope that is empty before any store call', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    const bundle = keyed([doc('config.yaml', CONFIG)])
    // Each call starts only when it is awaited: a call that is already running
    // can reject while an earlier one is still awaited, before anything listens.
    const calls = [
      () => host.exportScope({ identity: { id: '', displayName: 'Alice' }, scope: 's1' }),
      () => host.exportScope({ identity: ALICE, scope: '' }),
      () => host.importScope({ identity: ALICE, scope: '', bundle }),
    ]
    for (const call of calls) expect((await rejection(call())).code).toBe('invalid_argv')
    expect(port.calls).toEqual({ list: 0, read: 0, commit: 0 })
  })

  it('does not block the event loop: a store that answers through timers still settles', async () => {
    const port = memoryPort({ async: true })
    const host = createHost({ store: port.store })
    let ticks = 0
    const ticker = setInterval(() => {
      ticks++
    }, 0)
    try {
      const report = await host.importScope({
        identity: ALICE,
        scope: 's1',
        bundle: keyed([doc('config.yaml', CONFIG), doc('specs/auth/spec.md', '# auth\n')]),
      })
      expect(report).toEqual({ scopeVersion: 1, documents: 2 })
      const bundle = (await host.exportScope({ identity: ALICE, scope: 's1' })) as Bundle
      expect(bundle.documents.map((d) => d.key)).toEqual(['config.yaml', 'specs/auth/spec.md'])
    } finally {
      clearInterval(ticker)
    }
    // Every store call above settled on a timer of this loop.
    expect(ticks).toBeGreaterThan(0)
  })
})

// node-sdk「匯出與匯入的失敗帶原因代碼」: the three codes of host.importScope,
// and the failures that have none.
describe('the reason of a failed host.importScope or host.exportScope', () => {
  it('names scope_not_empty for a scope that holds documents', async () => {
    const host = createHost({ store: memoryPort().store })
    await host.importScope({ identity: ALICE, scope: 's1', bundle: keyed([doc('config.yaml', CONFIG)]) })

    const error = await rejection(
      host.importScope({ identity: ALICE, scope: 's1', bundle: keyed([doc('config.yaml', CONFIG)]) }),
    )

    expect([error.code, error.reason]).toEqual(['refused', 'scope_not_empty'])
    expect(error.message).toBe('import (create-new): the target scope already holds documents')
  })

  it('names bundle_too_large for content over limits.importBytes', async () => {
    const host = createHost({ store: memoryPort().store, limits: { importBytes: 1024 } })
    const bundle = keyed([doc('config.yaml', 'x'.repeat(2000))])

    const error = await rejection(host.importScope({ identity: ALICE, scope: 's1', bundle }))

    expect([error.code, error.reason]).toEqual(['invalid_argv', 'bundle_too_large'])
    expect(error.message).toBe("the bundle's content is 2000 bytes, over the 1024-byte import limit")
  })

  it.each([
    [
      'a digest that is not the content’s',
      { key: 'config.yaml', content: CONFIG, digest: digestOf('other\n') },
      'bundle document "config.yaml": the digest is not the digest of the content',
    ],
    [
      'a key of notes/a.md',
      doc('notes/a.md', '# notes\n'),
      'bundle document "notes/a.md": the key follows none of the Engine\'s naming rules',
    ],
  ])('names invalid_bundle_document for %s', async (_, document, message) => {
    const host = createHost({ store: memoryPort().store })

    const error = await rejection(host.importScope({ identity: ALICE, scope: 's1', bundle: keyed([document]) }))

    expect([error.code, error.reason]).toEqual(['invalid_argv', 'invalid_bundle_document'])
    expect(error.message).toBe(message)
  })

  it('gives no reason when beforeCommand refuses the import', async () => {
    const host = createHost({
      store: memoryPort().store,
      hooks: { beforeCommand: (info) => (info.command === 'scope-import' ? { allow: false, reason: 'not allowed' } : undefined) },
    })

    const error = await rejection(host.importScope({ identity: ALICE, scope: 's1', bundle: keyed([doc('config.yaml', CONFIG)]) }))

    expect([error.code, error.message]).toEqual(['refused', 'not allowed'])
    expect('reason' in error).toBe(false)
  })

  it('gives no reason for a bundle of formatVersion 99', async () => {
    const host = createHost({ store: memoryPort().store })

    const error = await rejection(
      host.importScope({ identity: ALICE, scope: 's1', bundle: { formatVersion: 99, documents: [] } as never }),
    )

    expect(error.code).toBe('invalid_argv')
    expect('reason' in error).toBe(false)
  })

  it('gives no reason when the store’s list throws StoreUnavailableError during an export', async () => {
    const port = memoryPort()
    const store = {
      ...port.store,
      list: () => {
        throw new StoreUnavailableError('the store is down')
      },
    }
    const host = createHost({ store })

    const error = await rejection(host.exportScope({ identity: ALICE, scope: 's1' }))

    expect(error.code).toBe('unavailable')
    expect('reason' in error).toBe(false)
  })
})

describe('createHost limits.importBytes', () => {
  it('refuses an import body over the limit with 413 through handleRequest', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO, limits: { importBytes: 1024 } })

    const response = await host.handleRequest(post('import', new Uint8Array(4096).fill(0x20)), ALICE)

    expect(response?.status).toBe(413)
    expect(port.calls).toEqual({ list: 0, read: 0, commit: 0 })
  })

  it('applies the same limit to host.importScope, and none to an export', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, limits: { importBytes: 1024 } })
    const large = 'x'.repeat(2000)

    const error = await rejection(
      host.importScope({ identity: ALICE, scope: 's1', bundle: keyed([doc('specs/large/spec.md', large)]) }),
    )
    expect(error.code).toBe('invalid_argv')
    expect(error.message).toContain('1024')
    expect(port.calls.commit).toBe(0)

    port.seed('s2', { 'specs/large/spec.md': large })
    const bundle = (await host.exportScope({ identity: ALICE, scope: 's2' })) as Bundle
    expect(bundle.documents[0].content.length).toBe(2000)
  })

  it('answers the import and the export routes through handleRequest', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })
    const body = new TextEncoder().encode(JSON.stringify(keyed([doc('config.yaml', CONFIG)])))

    const imported = await host.handleRequest(post('import', body), ALICE)
    expect(imported?.status).toBe(200)
    expect(JSON.parse(new TextDecoder().decode(imported!.body))).toEqual({ scopeVersion: 1, documents: 1 })

    const exported = await host.handleRequest({ ...post('export', new Uint8Array()), method: 'GET', body: undefined }, ALICE)
    expect(exported?.status).toBe(200)
    const bundle = JSON.parse(new TextDecoder().decode(exported!.body)) as Bundle
    expect([bundle.formatVersion, bundle.scopeVersion, bundle.documents.length]).toEqual([2, 1, 1])
  })

  it('throws at once when importBytes is not a positive integer', () => {
    const store = memoryPort().store
    for (const importBytes of [0, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY, '1024', null]) {
      expect(() => createHost({ store, limits: { importBytes: importBytes as never } }), String(importBytes)).toThrow(
        /importBytes/,
      )
    }
    expect(() => createHost({ store, limits: { importByte: 1024 } as never })).toThrow(/importByte/)
    expect(() => createHost({ store, limits: 1024 as never })).toThrow(/limits/)
    // An array is no object of limits, whatever it holds.
    expect(() => createHost({ store, limits: [1024] as never })).toThrow(/limits must be an object/)
    expect(() => createHost({ store, limits: null as never })).toThrow(/limits must be an object/)
    // No limits, or none stated, is the default.
    expect(() => createHost({ store, limits: {} })).not.toThrow()
    expect(() => createHost({ store, limits: { importBytes: 1 } })).not.toThrow()
  })
})
