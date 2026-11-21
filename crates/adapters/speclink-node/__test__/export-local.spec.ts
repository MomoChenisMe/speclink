// node-sdk「本機引擎的 exportScope 方法」: engine.exportScope() makes a local
// project into a bundle by key, and host.importScope takes it as it is.
import { execFileSync, execSync } from 'node:child_process'
import {
  chmodSync,
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

// node-sdk「匯出與匯入的失敗帶原因代碼」: the six codes of engine.exportScope.
describe('the reason of a failed engine.exportScope', () => {
  // A folder with no permission still lists for root, and Windows has no such mode.
  const noPermission = process.platform === 'win32' || process.getuid?.() === 0

  it('names no_spec_folder for a root that holds no spec folder', async () => {
    const error = await rejection(fsEngine(folder({ 'notes/README.md': '# notes\n' })).exportScope())

    expect([error.code, error.reason]).toEqual(['error', 'no_spec_folder'])
    expect(error.message.startsWith('no spec folder:'), error.message).toBe(true)
    expect(['change' in error, 'path' in error]).toEqual([false, false])
  })

  it('names spec_folder_outside_project for a specDir of ../outside', async () => {
    const root = project({ 'changes/demo/.meta.yaml': META })
    const engine = createEngine({ store: { type: 'fs', root, specDir: '../outside' } })

    const error = await rejection(engine.exportScope())

    expect([error.code, error.reason]).toEqual(['error', 'spec_folder_outside_project'])
    expect(error.message).toBe(
      'spec folder outside the project: "../outside" — spec_dir must name a folder under the project root',
    )
  })

  it.skipIf(process.platform === 'win32')('names symlink_in_spec_folder for a change folder that is a link', async () => {
    const outside = folder({ 'demo/.meta.yaml': META, 'demo/proposal.md': PROPOSAL })
    const root = project({ 'specs/auth/spec.md': SPEC_AUTH })
    mkdirSync(join(root, 'speclink', 'changes'))
    symlinkSync(join(outside, 'demo'), join(root, 'speclink', 'changes', 'demo'))

    const error = await rejection(fsEngine(root).exportScope())

    expect([error.code, error.reason]).toEqual(['error', 'symlink_in_spec_folder'])
    expect(error.message).toBe(
      'symbolic link in the spec folder: speclink/changes/demo — replace it with the file itself or remove it',
    )
    expect(['change' in error, 'path' in error]).toEqual([false, false])
  })

  it.skipIf(process.platform === 'win32')('gives no change and no path for a spec that is a link', async () => {
    // Scenario「與變更無關的失敗沒有位置」.
    const outside = folder({ 'spec.md': SPEC_AUTH })
    const root = project({ 'changes/demo/.meta.yaml': META })
    mkdirSync(join(root, 'speclink', 'specs'))
    symlinkSync(outside, join(root, 'speclink', 'specs', 'linked'))

    const error = await rejection(fsEngine(root).exportScope())

    // Scenario「規格資料夾裡有符號連結」 too: the code and the start of the message.
    expect([error.code, error.reason]).toEqual(['error', 'symlink_in_spec_folder'])
    expect(error.message.startsWith('symbolic link in the spec folder:'), error.message).toBe(true)
    expect(['change' in error, 'path' in error]).toEqual([false, false])
  })

  it.skipIf(noPermission)('names spec_folder_unreadable for a folder with permission 000', async () => {
    const root = project({ 'changes/demo/.meta.yaml': META, 'changes/locked/.meta.yaml': META })
    const locked = join(root, 'speclink', 'changes', 'locked')
    chmodSync(locked, 0o000)
    try {
      const error = await rejection(fsEngine(root).exportScope())

      expect([error.code, error.reason]).toEqual(['error', 'spec_folder_unreadable'])
      expect(
        error.message.startsWith('cannot list speclink/changes/locked to check it for symbolic links:'),
        error.message,
      ).toBe(true)
    } finally {
      chmodSync(locked, 0o755)
    }
  })

  it('names invalid_change_metadata for a data file that does not parse', async () => {
    const root = project({ 'changes/demo/.meta.yaml': ': [', 'changes/demo/proposal.md': PROPOSAL })

    const error = await rejection(fsEngine(root).exportScope())

    expect([error.code, error.reason]).toEqual(['error', 'invalid_change_metadata'])
    expect(error.message).toContain('speclink/changes/demo/.meta.yaml')
    // Scenario「metadata 無法解析的匯出帶變更的位置」: the message is as it was.
    expect([error.change, error.path]).toEqual(['demo', 'speclink/changes/demo/.meta.yaml'])
    expect(error.message.startsWith('invalid speclink/changes/demo/.meta.yaml:'), error.message).toBe(true)
  })

  it('names change_schema_unusable, the change and its data file for a schema that is not found', async () => {
    // Scenario「產出流程找不到的匯出指出是哪一個變更」.
    const root = project({
      'changes/archive/2026-03-03-old-one/.meta.yaml': 'schema: nope\n',
      'changes/archive/2026-03-03-old-one/proposal.md': PROPOSAL,
    })

    const error = await rejection(fsEngine(root).exportScope())

    expect([error.code, error.reason, error.change, error.path]).toEqual([
      'error',
      'change_schema_unusable',
      '2026-03-03-old-one',
      'speclink/changes/archive/2026-03-03-old-one/.meta.yaml',
    ])
    expect(error.message).toBe(
      "change '2026-03-03-old-one': Schema not found: Schema 'nope' not found in project, user, or built-in locations",
    )
  })

  it('names change_schema_unusable for a schema whose definition does not load', async () => {
    const root = project({
      'schemas/broken/schema.yaml': ': [',
      'changes/demo/.meta.yaml': 'schema: broken\n',
      'changes/demo/proposal.md': PROPOSAL,
    })

    const error = await rejection(fsEngine(root).exportScope())

    expect([error.code, error.reason]).toEqual(['error', 'change_schema_unusable'])
    expect(error.message.startsWith("change 'demo': "), error.message).toBe(true)
  })

  // Example「位置屬性在哪些失敗出現」: the rows of a failure about one change.
  it.each([
    {
      files: { 'changes/demo/.meta.yaml': 'schema: nope\n' },
      reason: 'change_schema_unusable',
      change: 'demo',
      path: 'speclink/changes/demo/.meta.yaml',
    },
    {
      // Only the legacy file name is there: the path names it.
      files: { 'changes/archive/2026-03-03-old-one/.openspec.yaml': 'schema: nope\n' },
      reason: 'change_schema_unusable',
      change: '2026-03-03-old-one',
      path: 'speclink/changes/archive/2026-03-03-old-one/.openspec.yaml',
    },
    {
      files: { 'changes/demo/.meta.yaml': "reviewed_rounds: ''\n" },
      reason: 'invalid_change_metadata',
      change: 'demo',
      path: 'speclink/changes/demo/.meta.yaml',
    },
  ])('gives $reason with the change $change and the path $path', async ({ files, reason, change, path }) => {
    const error = await rejection(fsEngine(project(files)).exportScope())

    expect([error.code, error.reason, error.change, error.path]).toEqual(['error', reason, change, path])
  })

  it.each([
    // A name with a slash at its end, and a nested name as `path.join` gives it (backslashes on Windows).
    { specDir: 'speclink/', expected: 'speclink/changes/demo/.meta.yaml' },
    { specDir: join('docs', 'specs'), expected: 'docs/specs/changes/demo/.meta.yaml' },
  ])('gives the path with forward slashes for the specDir $specDir', async ({ specDir, expected }) => {
    const root = folder({ [`${expected}`]: 'schema: nope\n' })
    const engine = createEngine({ store: { type: 'fs', root, specDir } })

    const error = await rejection(engine.exportScope())

    expect([error.reason, error.change, error.path]).toEqual(['change_schema_unusable', 'demo', expected])
  })

  it('gives no change and no path for an import into a scope that holds documents', async () => {
    const root = project({ 'changes/demo/.meta.yaml': META, 'changes/demo/proposal.md': PROPOSAL })
    const bundle = await fsEngine(root).exportScope()
    const host = createHost({ store: memoryPort().store })
    await host.importScope({ identity: ALICE, scope: 's1', bundle })

    const error = await rejection(host.importScope({ identity: ALICE, scope: 's1', bundle }))

    expect([error.code, error.reason]).toEqual(['refused', 'scope_not_empty'])
    expect(['change' in error, 'path' in error]).toEqual([false, false])
  })
})

// change-lifecycle「change metadata 的 schema 空白時視同沒有設定」.
describe('a change whose schema is blank', () => {
  it('exports an archived change with a blank schema, its data file as it is', async () => {
    // Scenario「已封存變更的空白 schema 不擋匯出」: an older tool wrote these two lines.
    const blank = "schema: ''\ncreated: ''\n"
    const root = project({
      'changes/archive/2026-03-03-old-one/.meta.yaml': blank,
      'changes/archive/2026-03-03-old-one/proposal.md': PROPOSAL,
    })

    const bundle: ScopeBundle = await fsEngine(root).exportScope()

    expect(bundle.documents).toEqual([
      { key: 'archive/changes/2026-03-03-old-one/.meta.yaml', content: blank, digest: digestOf(blank) },
      { key: 'archive/changes/2026-03-03-old-one/proposal.md', content: PROPOSAL, digest: digestOf(PROPOSAL) },
    ])
  })
})

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
