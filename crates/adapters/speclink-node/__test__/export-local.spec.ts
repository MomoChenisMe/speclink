// node-sdk「本機引擎的 exportScope 方法」: engine.exportScope() makes a local
// project into a bundle by key, and host.importScope takes it as it is.
import { execFileSync, execSync } from 'node:child_process'
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import type { ScopeBundle } from '../index'
import { createEngine, createHost } from '../index.js'
import { cliBin, digestOf, fileTimes, fsEngine, projectFiles, rejection, repoRoot } from './helpers'
import { byBytes, memoryPort } from './memory-port'

const ALICE = { id: 'u_42', displayName: 'Alice' }

const META = 'schema: spec-driven\ncreated: "2026-07-01"\n'
const PROPOSAL = '## Why\n\nDemo proposal.\n'
const TASKS = '## 1. Work\n\n- [ ] 1.1 First task\n'
const SPEC_AUTH =
  '## Purpose\nAuthentication.\n\n### Requirement: Login\nUsers SHALL be able to log in.\n\n#### Scenario: valid login\n- **WHEN** credentials are valid\n- **THEN** a session is created\n'
const TOPIC =
  '---\ntopic: Topic\nslug: topic\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n'
const ARCHIVED_META = 'schema: spec-driven\ncreated: "2026-06-01"\n'
const ARCHIVED_EVIDENCE =
  '{"version":2,"change":"old","entries":[{"taskId":"tsk_1","taskDesc":"1.1 Old task","touchedFiles":["src/old.rs"],"recordedAt":"2026-06-02T10:00:00Z"}]}\n'
const DELTA_AUTH =
  '## ADDED Requirements\n\n### Requirement: Logout\nUsers SHALL be able to log out.\n\n#### Scenario: logout\n- **WHEN** a user logs out\n- **THEN** the session ends\n'
const PAST =
  '---\ntopic: Past\nslug: past\nstatus: concluded\ncreated: 2026-06-01\n---\n\nArchived discussion.\n'

const roots: string[] = []
afterAll(() => {
  for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true, maxRetries: 5 })
})

/** A new folder that holds `files` and `dirs`, each a path from the folder itself. */
function folder(files: Record<string, string>, dirs: string[] = []): string {
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-export-local-')))
  roots.push(root)
  for (const dir of dirs) mkdirSync(join(root, dir), { recursive: true })
  for (const [path, content] of Object.entries(files)) {
    mkdirSync(join(root, path, '..'), { recursive: true })
    writeFileSync(join(root, path), content)
  }
  return root
}

/** A new local project: each key of `files` is a path under its spec folder. */
const project = (files: Record<string, string>) =>
  folder(Object.fromEntries(Object.entries(files).map(([path, content]) => [`speclink/${path}`, content])))


/** The bundle of the project at `root`, imported into scope s1 of a new host on a JS memory store. */
async function importedFrom(root: string) {
  const bundle = await fsEngine(root).exportScope()
  const host = createHost({ store: memoryPort().store })
  await host.importScope({ identity: ALICE, scope: 's1', bundle })
  return { bundle, viaHost: (argv: string[]) => host.dispatch({ identity: ALICE, scope: 's1', argv }) }
}

/**
 * A listing without the fields each side has its own way: an item's `path`
 * (a file path on the local side, a store key in a host) and its
 * `modifiedAt` (a file time on the local side, the store's own in a host).
 */
const withoutLocalFields = (listed: unknown) =>
  Object.fromEntries(
    Object.entries(listed as Record<string, unknown>).map(([name, value]) => [
      name,
      Array.isArray(value)
        ? value.map(({ path: _path, modifiedAt: _time, ...item }: Record<string, unknown>) => item)
        : value,
    ]),
  )

describe('engine.exportScope', () => {
  it('gives a bundle by key that a host imports, and the host reads what the engine reads', async () => {
    const root = project({
      'changes/demo/.meta.yaml': META,
      'changes/demo/proposal.md': PROPOSAL,
      'changes/demo/tasks.md': TASKS,
      'specs/auth/spec.md': SPEC_AUTH,
      'discussions/topic.md': TOPIC,
    })
    const engine = fsEngine(root)

    const { bundle, viaHost } = await importedFrom(root)

    expect(bundle.formatVersion).toBe(2)
    expect(bundle).not.toHaveProperty('scopeVersion')
    const keys = bundle.documents.map((d) => d.key)
    expect(keys).toEqual([...keys].sort(byBytes))
    expect(new Set(keys).size).toBe(keys.length)
    expect(keys).toEqual([
      'changes/demo/.meta.yaml',
      'changes/demo/proposal.md',
      'changes/demo/tasks.md',
      'discussions/topic.md',
      'specs/auth/spec.md',
    ])
    for (const document of bundle.documents) {
      expect(Object.keys(document).sort()).toEqual(['content', 'digest', 'key'])
      expect(document.digest).toBe(digestOf(document.content))
    }

    for (const argv of [
      ['list', '--json'],
      ['discuss', 'list', '--json'],
    ]) {
      const [host, local] = [await viaHost(argv), await engine.dispatch(argv)]
      expect(withoutLocalFields(host), argv.join(' ')).toEqual(withoutLocalFields(local))
    }
    const show = ['show', 'auth', '--item-type', 'spec']
    expect(await viaHost(show)).toEqual(await engine.dispatch(show))
    const listed = (await viaHost(['list', '--json'])) as { changes: { name: string }[] }
    expect(listed.changes.map((c) => c.name)).toEqual(['demo'])
  })

  it('names each document by its key, in the shape of the spec example', async () => {
    // spec Example「匯出的包的形狀」: one change demo with a proposal and its metadata.
    const root = project({ 'changes/demo/.meta.yaml': META, 'changes/demo/proposal.md': PROPOSAL })

    const bundle: ScopeBundle = await fsEngine(root).exportScope()

    expect(bundle).toEqual({
      formatVersion: 2,
      documents: [
        { key: 'changes/demo/.meta.yaml', content: META, digest: digestOf(META) },
        { key: 'changes/demo/proposal.md', content: PROPOSAL, digest: digestOf(PROPOSAL) },
      ],
    })
  })

  it('reads the project and writes nothing', async () => {
    const root = project({
      'changes/demo/.meta.yaml': META,
      'changes/demo/proposal.md': PROPOSAL,
      'specs/auth/spec.md': SPEC_AUTH,
      'discussions/topic.md': TOPIC,
    })
    const [files, times] = [projectFiles(root), fileTimes(root)]

    await fsEngine(root).exportScope()

    expect(projectFiles(root)).toEqual(files)
    expect(fileTimes(root)).toEqual(times)
  })

  it('rejects with an Error that names the broken metadata file, and gives no bundle', async () => {
    const root = project({
      'changes/good/.meta.yaml': META,
      'changes/good/proposal.md': PROPOSAL,
      'changes/broken/.meta.yaml': 'schema: [unterminated\n',
      'changes/broken/proposal.md': PROPOSAL,
    })

    const error = await rejection(fsEngine(root).exportScope())

    expect(error.code).toBe('error')
    expect(error.message).toContain('speclink/changes/broken/.meta.yaml')
  })

  // Making a symbolic link on Windows needs a privilege a test run may not have.
  it.skipIf(process.platform === 'win32')(
    'rejects when the spec folder holds a symbolic link, names it, and shows nothing it points at',
    async () => {
      const outside = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-export-outside-')))
      roots.push(outside)
      writeFileSync(join(outside, 'secret.md'), 'TOP SECRET\n')
      const root = project({ 'changes/demo/.meta.yaml': META, 'changes/demo/proposal.md': PROPOSAL })
      mkdirSync(join(root, 'speclink', 'specs', 'leak'), { recursive: true })
      symlinkSync(join(outside, 'secret.md'), join(root, 'speclink', 'specs', 'leak', 'spec.md'))

      const error = await rejection(fsEngine(root).exportScope())

      expect(error.code).toBe('error')
      expect(error.message).toContain('speclink/specs/leak/spec.md')
      expect(error.message).not.toContain('TOP SECRET')
    },
  )

  it('rejects when spec_dir names a folder outside the project, names it, and reads nothing there', async () => {
    // spec「規格資料夾指到專案外讓匯出失敗」.
    const base = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-export-spec-dir-')))
    roots.push(base)
    mkdirSync(join(base, 'other', 'speclink', 'specs', 'private'), { recursive: true })
    writeFileSync(join(base, 'other', 'speclink', 'specs', 'private', 'spec.md'), 'OTHER PROJECT SECRET\n')
    const root = join(base, 'project')
    mkdirSync(root)
    writeFileSync(join(root, '.speclink.yaml'), 'spec_dir: ../other/speclink\n')

    const error = await rejection(fsEngine(root).exportScope())

    expect(error.code).toBe('error')
    expect(error.message).toContain('"../other/speclink"')
    expect(error.message).not.toContain('OTHER PROJECT SECRET')
    // The name check comes first: a bad name is never reported as a missing spec folder.
    expect(error.message.startsWith('spec folder outside the project:')).toBe(true)
    expect(error.message).not.toContain('no spec folder:')
  })

  // spec「沒有規格資料夾的 root 讓匯出失敗」and its example table.
  describe('a root that holds no spec folder', () => {
    it('rejects a root with no speclink/, names the folder but not the root, and changes nothing', async () => {
      const root = folder({ 'downloaded/README.md': '# Put in the wrong place\n' })
      const [files, times] = [projectFiles(root), fileTimes(root)]

      const error = await rejection(fsEngine(root).exportScope())

      expect(error.code).toBe('error')
      expect(error.message.startsWith('no spec folder: "speclink"'), error.message).toBe(true)
      expect(error.message).not.toContain(root)
      expect(projectFiles(root)).toEqual(files)
      expect(fileTimes(root)).toEqual(times)
    })

    it.each([
      ['a speclink/ that holds only a README.md', { 'speclink/README.md': '# Not specs\n' }, 'speclink'],
      ['a spec_dir that names no folder', { '.speclink.yaml': 'spec_dir: docs/specs\n' }, 'docs/specs'],
    ])('rejects %s', async (_, files, name) => {
      const error = await rejection(fsEngine(folder(files)).exportScope())

      expect(error.code).toBe('error')
      expect(error.message.startsWith(`no spec folder: "${name}"`), error.message).toBe(true)
    })

    it('exports a spec folder that holds only an empty specs/, with no documents', async () => {
      const bundle = await fsEngine(folder({}, ['speclink/specs'])).exportScope()

      expect(bundle.formatVersion).toBe(2)
      expect(bundle.documents).toEqual([])
    })

    it('exports a spec folder that was downloaded alone, named by specDir', async () => {
      // spec「只下載規格資料夾時以 specDir 指定名稱」.
      const root = folder({
        'specs-folder/specs/auth/spec.md': SPEC_AUTH,
        'specs-folder/changes/demo/.meta.yaml': META,
        'specs-folder/changes/demo/proposal.md': PROPOSAL,
      })

      const bundle = await createEngine({ store: { type: 'fs', root, specDir: 'specs-folder' } }).exportScope()

      expect(bundle.documents.map((d) => d.key)).toEqual([
        'changes/demo/.meta.yaml',
        'changes/demo/proposal.md',
        'specs/auth/spec.md',
      ])
    })
  })

  describe('a project that was just initialized', () => {
    beforeAll(() => {
      if (!existsSync(cliBin)) execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
    }, 600_000)

    it('exports the documents init wrote and nothing else', async () => {
      const root = project({})
      execFileSync(cliBin, ['init', '.', '--tools', 'claude'], { cwd: root, encoding: 'utf8' })

      const bundle = await fsEngine(root).exportScope()

      expect(bundle.formatVersion).toBe(2)
      expect(bundle.documents.map((d) => d.key)).toEqual(['config.yaml'])
    })
  })

  // verb-contract「單筆變更的詳細輸出」Scenario「三種執行方式輸出相同」: the local engine and a
  // host that imported the same project give the same stations with --detail.
  it('gives the same show --json --detail through engine.dispatch and through the host', async () => {
    const stamped =
      `${META}reviewed_at: 2026-11-02\nreviewed_tasks_total: 1\nreviewed_rounds: 2\n` +
      'reviewed_scope:\n  - path: src/b.rs\n    hash: bb01\n  - path: src/a.rs\n    hash: aa01\n' +
      'verified_at: 2026-11-03\nverified_tasks_total: 1\nverified_rounds: 1\n' +
      'verified_scope:\n  - path: src/a.rs\n    hash: aa02\n'
    const root = project({ 'changes/demo/.meta.yaml': stamped, 'changes/demo/proposal.md': PROPOSAL })
    const engine = fsEngine(root)
    const { viaHost } = await importedFrom(root)
    type Shown = { review?: { scope?: unknown }; verify?: { scope?: unknown } }
    const argv = ['show', 'demo', '--json', '--detail']

    const local = (await engine.dispatch(argv)) as Shown
    const hosted = (await viaHost(argv)) as Shown

    expect(hosted.review).toEqual(local.review)
    expect(hosted.verify).toEqual(local.verify)
    expect(local.review?.scope).toEqual([
      { path: 'src/b.rs', hash: 'bb01' },
      { path: 'src/a.rs', hash: 'aa01' },
    ])
    expect(local.verify?.scope).toEqual([{ path: 'src/a.rs', hash: 'aa02' }])
    expect((await rejection(engine.dispatch(['show', 'demo', '--detail']))).code).toBe('invalid_argv')
  })

  it('takes the archived changes and discussions too, and the host reads them', async () => {
    const root = project({
      'specs/auth/spec.md': SPEC_AUTH,
      'changes/archive/2026-06-01-old/.meta.yaml': ARCHIVED_META,
      'changes/archive/2026-06-01-old/proposal.md': PROPOSAL,
      'changes/archive/2026-06-01-old/.evidence.json': ARCHIVED_EVIDENCE,
      'changes/archive/2026-06-01-old/specs/auth/spec.md': DELTA_AUTH,
      'discussions/archive/2026-06-01-past.md': PAST,
    })
    const engine = fsEngine(root)

    const { bundle, viaHost } = await importedFrom(root)

    expect(bundle.documents.map((d) => d.key)).toEqual([
      'archive/changes/2026-06-01-old/.evidence.json',
      'archive/changes/2026-06-01-old/.meta.yaml',
      'archive/changes/2026-06-01-old/proposal.md',
      'archive/changes/2026-06-01-old/specs/auth/spec.md',
      // An archived discussion is stored under its slug.
      'archive/discussions/past.md',
      'specs/auth/spec.md',
    ])
    const archived = ['discuss', 'list', '--archived', '--json']
    expect(withoutLocalFields(await viaHost(archived))).toEqual(withoutLocalFields(await engine.dispatch(archived)))
    for (const argv of [
      ['evidence', 'show', 'old', '--json'],
      ['trace', 'auth', '--json'],
    ]) {
      expect(await viaHost(argv), argv.join(' ')).toEqual(await engine.dispatch(argv))
    }
    const past = (await viaHost(['discuss', 'list', '--archived', '--json'])) as { discussions: { slug: string }[] }
    expect(past.discussions.map((d) => d.slug)).toEqual(['past'])
    expect(await viaHost(['evidence', 'show', 'old', '--json'])).toMatchObject({ archivedAs: '2026-06-01-old' })
  })
})
