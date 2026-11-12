// node-sdk「host.handleRequest 與 createHost 的入口設定」: the host kit's HTTP
// door for a host's one route, over a JS store port.
import { describe, expect, it } from 'vitest'

import { createHost } from '../index.js'
import { rejection } from './helpers'
import { memoryPort } from './memory-port'

const API = '/api/speclink/v1'
const ALICE = { id: 'u_42', displayName: 'Alice' }
const META = 'schema: spec-driven\n'
const TASKS = '## 1. Work\n\n- [ ] 1.1 First task\n'

/** Project alpha with one repo, web, on scope s1, writable. */
const ONE_REPO = [
  { key: 'alpha', name: 'Alpha', repos: [{ key: 'web', name: 'Web', scope: 's1', canWrite: true, canWritePolicy: true }] },
]

type Request = { method: string; path: string; query?: string; headers: Record<string, string>; body?: Uint8Array }

function request(method: string, path: string, body?: unknown): Request {
  return {
    method,
    path,
    headers: { 'x-speclink-api-version': '1', 'content-type': 'application/json' },
    body: body === undefined ? undefined : new TextEncoder().encode(JSON.stringify(body)),
  }
}

function alpha(tail: string) {
  return `${API}/projects/alpha/${tail}`
}

function json(response: { body: Uint8Array } | null): any {
  expect(response).not.toBeNull()
  return JSON.parse(new TextDecoder().decode(response!.body))
}

describe('host.handleRequest', () => {
  it('serves a handshake, a create and a list over a JS store, committing once', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })

    const handshake = await host.handleRequest(request('GET', alpha('binding')), ALICE)
    const created = await host.handleRequest(request('POST', alpha('changes'), { name: 'demo' }), ALICE)
    const listed = await host.handleRequest(request('GET', alpha('changes')), ALICE)

    expect([handshake?.status, created?.status, listed?.status]).toEqual([200, 200, 200])
    expect(json(handshake).actor).toEqual({ id: 'u_42', name: 'Alice' })
    expect(json(handshake).capabilities.events.polling).toEqual({ url: '/sync-state', etag: true })
    expect(json(created).name).toBe('demo')
    expect(created?.headers.etag).toMatch(/^"\d+"$/)
    expect(created?.headers['content-type']).toBe('application/json')
    expect(json(listed).changes.map((c: { name: string }) => c.name)).toEqual(['demo'])
    expect(created?.body).toBeInstanceOf(Uint8Array)
    expect(port.calls.commit).toBe(1)
  })

  it('answers polling with 304 while the scope is unchanged, and 200 after a commit', async () => {
    const host = createHost({ store: memoryPort().store, scopes: () => ONE_REPO })
    const listed = await host.handleRequest(request('GET', alpha('changes')), ALICE)
    const etag = listed!.headers.etag
    const poll = () => {
      const req = request('GET', alpha('sync-state'))
      req.headers['if-none-match'] = etag
      return host.handleRequest(req, ALICE)
    }

    const unchanged = await poll()
    expect(unchanged?.status).toBe(304)
    expect(unchanged?.headers.etag).toBe(etag)
    expect(unchanged?.body.length).toBe(0)

    await host.handleRequest(request('POST', alpha('changes'), { name: 'demo' }), ALICE)
    const changed = await poll()
    expect(changed?.status).toBe(200)
    expect(changed?.headers.etag).not.toBe(etag)
  })

  it('answers a null identity with 401', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })

    const response = await host.handleRequest(request('GET', alpha('changes')), null)

    expect(response?.status).toBe(401)
    expect(json(response).reason).toBe('permission_denied')
    expect(port.calls.list + port.calls.read + port.calls.commit).toBe(0)
  })

  it('resolves to null for a path that is not the door’s', async () => {
    const host = createHost({ store: memoryPort().store, scopes: () => ONE_REPO })
    expect(await host.handleRequest(request('GET', '/healthz'), ALICE)).toBeNull()
  })

  it('shares the hooks with host.dispatch, with the same message', async () => {
    const port = memoryPort()
    port.seed('s1', { 'changes/demo/.meta.yaml': META, 'changes/demo/tasks.md': TASKS })
    const host = createHost({
      store: port.store,
      scopes: () => ONE_REPO,
      hooks: {
        beforeCommand: (info: { command: string }) =>
          info.command === 'archive' ? { allow: false, reason: '程式碼還沒合併' } : undefined,
      },
    })

    const viaTool = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['archive', 'demo'] }))
    const viaDoor = await host.handleRequest(request('POST', alpha('changes/demo/archive')), ALICE)

    expect(viaTool.code).toBe('refused')
    expect(viaDoor?.status).toBe(409)
    expect(json(viaDoor).reason).toBe('refused')
    expect(json(viaDoor).message).toBe(viaTool.message)
    expect(viaTool.message).toBe('程式碼還沒合併')
  })

  it('waits for an async scopes function', async () => {
    const host = createHost({
      store: memoryPort().store,
      scopes: async () => {
        await new Promise((resolve) => setTimeout(resolve, 20))
        return ONE_REPO
      },
    })

    const response = await host.handleRequest(request('GET', alpha('changes')), ALICE)

    expect(response?.status).toBe(200)
    expect(json(response).changes).toEqual([])
  })

  it('answers 503 when scopes throws', async () => {
    const host = createHost({
      store: memoryPort().store,
      scopes: () => {
        throw new Error('db at 10.0.0.7 refused the connection')
      },
    })

    const response = await host.handleRequest(request('GET', alpha('changes')), ALICE)

    expect(response?.status).toBe(503)
    expect(json(response).reason).toBe('unavailable')
    expect(json(response).message).not.toContain('10.0.0.7')
  })

  it('answers the login-method declaration without an identity', async () => {
    const host = createHost({
      store: memoryPort().store,
      authMethods: { methods: ['browser'], browser: { issuer: 'https://id.example.com', clientId: 'speclink' } },
    })

    const response = await host.handleRequest({ method: 'GET', path: `${API}/auth-methods`, headers: {} }, null)

    expect(response?.status).toBe(200)
    expect(json(response)).toEqual({
      methods: ['browser'],
      browser: { issuer: 'https://id.example.com', clientId: 'speclink', scopes: [] },
    })
  })

  it('answers a method it does not take with 405', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })

    const response = await host.handleRequest(request('PATCH', alpha('changes')), ALICE)

    expect(response?.status).toBe(405)
    expect(port.calls.list + port.calls.read + port.calls.commit).toBe(0)
  })

  it('declares the push streams the host gives in the handshake', async () => {
    const host = createHost({
      store: memoryPort().store,
      scopes: () => ONE_REPO,
      pushTransports: [{ type: 'sse', url: 'https://host.example/events', resume: true }],
    })

    const handshake = await host.handleRequest(request('GET', alpha('binding')), ALICE)

    expect(json(handshake).capabilities.events.transports).toEqual([
      { type: 'sse', url: 'https://host.example/events', resume: true },
    ])
  })

  it('rejects with internal when scopes answers in another shape', async () => {
    const host = createHost({
      store: memoryPort().store,
      // canWritePolicy is missing: a switch has no default.
      scopes: () => [{ key: 'alpha', name: 'Alpha', repos: [{ key: 'web', name: 'Web', scope: 's1', canWrite: true }] }],
    })

    const error = await rejection(host.handleRequest(request('GET', alpha('changes')), ALICE))

    expect(error.code).toBe('internal')
    expect(error.message).toContain('canWritePolicy')
  })

  it('throws at createHost for a declaration of another shape', () => {
    expect(() => createHost({ store: memoryPort().store, authMethods: { method: ['token'] } as never })).toThrow(
      /authMethods/,
    )
    expect(() => createHost({ store: memoryPort().store, scopes: 'alpha' as never })).toThrow(/scopes must be a function/)
  })
})
