// Shared test helpers: the fs fixture project, CLI invocation, and an
// in-memory JS Store implementing the full Store interface (async methods)
// with the same content as the fs fixture, so bridge output can be compared
// to CLI output field by field.
import { execFileSync } from 'node:child_process'
import { existsSync, mkdtempSync, mkdirSync, realpathSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

export const repoRoot = join(__dirname, '..', '..', '..')
// 本機 cargo test 產 debug CLI；CI 的 Node SDK workflow 只建 release——擇一存在者。
const binName = process.platform === 'win32' ? 'speclink.exe' : 'speclink'
const debugBin = join(repoRoot, 'target', 'debug', binName)
export const cliBin = existsSync(debugBin) ? debugBin : join(repoRoot, 'target', 'release', binName)

/** Run the CLI in the fixture project and parse its --json stdout. */
export function cliJson(cwd: string, args: string[]): unknown {
  const out = execFileSync(cliBin, args, { cwd, encoding: 'utf8' })
  return JSON.parse(out)
}

export const PROPOSAL_ALPHA =
  '## Why\n\nAdd rate limiting to the public API so abusive clients cannot exhaust capacity.\n\n## What Changes\n\n- add limiter\n'
export const TASKS_ALPHA =
  '## 1. Work\n\n- [x] 1.1 First task\n- [ ] 1.2 Second task\n- [ ] 1.3 Third task\n'
export const DELTA_ALPHA =
  '## ADDED Requirements\n\n### Requirement: Quota enforcement\nThe API SHALL reject clients over quota.\n\n#### Scenario: over quota\n- **WHEN** a client exceeds its quota\n- **THEN** the request is rejected\n'
export const PROPOSAL_BETA = '## Why\n\nSmall cleanup change with every task already complete.\n'
export const TASKS_BETA = '## 1. Work\n\n- [x] 1.1 Only task\n'
export const SPEC_USER_AUTH =
  '## Purpose\nAuthentication.\n\n### Requirement: Login\nUsers SHALL be able to log in.\n\n#### Scenario: valid login\n- **WHEN** credentials are valid\n- **THEN** a session is created\n'

/** A fixture project with two changes (task progress + proposal summaries) and one spec. */
export function makeFsFixture(): string {
  // realpath: macOS 的 tmpdir 是 /var → /private/var 的 symlink；CLI 以 getcwd 回報
  // 實體路徑，engine 則沿用傳入字串——不先解析，兩邊的 path 欄位無法逐位元比對。
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-node-fixture-')))
  const changes = join(root, 'openspec', 'changes')

  const alpha = join(changes, 'alpha')
  mkdirSync(alpha, { recursive: true })
  writeFileSync(
    join(alpha, '.openspec.yaml'),
    'schema: spec-driven\ncreated: "2026-07-01"\ncreated_by: tester\n',
  )
  writeFileSync(join(alpha, 'proposal.md'), PROPOSAL_ALPHA)
  writeFileSync(join(alpha, 'tasks.md'), TASKS_ALPHA)
  const alphaSpecDir = join(alpha, 'specs', 'api-quota')
  mkdirSync(alphaSpecDir, { recursive: true })
  writeFileSync(join(alphaSpecDir, 'spec.md'), DELTA_ALPHA)

  const beta = join(changes, 'beta')
  mkdirSync(beta, { recursive: true })
  writeFileSync(
    join(beta, '.openspec.yaml'),
    'schema: spec-driven\ncreated: "2026-07-02"\ncreated_by: tester\n',
  )
  writeFileSync(join(beta, 'proposal.md'), PROPOSAL_BETA)
  writeFileSync(join(beta, 'tasks.md'), TASKS_BETA)

  const specDir = join(root, 'openspec', 'specs', 'user-auth')
  mkdirSync(specDir, { recursive: true })
  writeFileSync(join(specDir, 'spec.md'), SPEC_USER_AUTH)
  return root
}

export interface MemoryProject {
  changes: Record<
    string,
    {
      meta?: { schema?: string; created?: string; createdBy?: string }
      artifacts: Record<string, string>
      updatedAtSecs?: number
    }
  >
  specs: Record<string, string>
  language?: string
  workflowConfig?: string
}

/** The memory-store twin of `makeFsFixture()`. */
export function fixtureProject(): MemoryProject {
  return {
    changes: {
      alpha: {
        meta: { schema: 'spec-driven', created: '2026-07-01', createdBy: 'tester' },
        artifacts: {
          // raw metadata 文件與 meta 物件同步——蓋章走 readChangeMeta 讀原文，
          // 缺了它就測不出「既有欄位被整份洗掉」。
          '.openspec.yaml': 'schema: spec-driven\ncreated: "2026-07-01"\ncreated_by: tester\n',
          'proposal.md': PROPOSAL_ALPHA,
          'tasks.md': TASKS_ALPHA,
          'specs/api-quota/spec.md': DELTA_ALPHA,
        },
        updatedAtSecs: 100,
      },
      beta: {
        meta: { schema: 'spec-driven', created: '2026-07-02', createdBy: 'tester' },
        artifacts: {
          '.openspec.yaml': 'schema: spec-driven\ncreated: "2026-07-02"\ncreated_by: tester\n',
          'proposal.md': PROPOSAL_BETA,
          'tasks.md': TASKS_BETA,
        },
        updatedAtSecs: 200,
      },
    },
    specs: { 'user-auth': SPEC_USER_AUTH },
  }
}

/**
 * A complete in-memory Store: every method of the Store interface, all async
 * (returning Promises), backed by a MemoryProject. Discussions and the archive
 * are held in maps so write-path verbs work too.
 */
export function memoryStore(project: MemoryProject) {
  const changes = project.changes
  const specs = project.specs
  const archived = new Map<string, string>()
  const liveDiscussions = new Map<string, string>()
  const archivedDiscussions = new Map<string, string>()

  const changeObj = (name: string) => ({
    name,
    dir: `changes/${name}`,
    meta: changes[name].meta ?? {},
  })
  const discussionDoc = (slug: string, text: string, isArchived: boolean) => ({
    slug,
    text,
    path: isArchived ? `discussions/archive/${slug}.md` : `discussions/${slug}.md`,
    archived: isArchived,
  })

  return {
    // --- changes ---
    async listChanges() {
      return Object.keys(changes).sort().map(changeObj)
    },
    async findChange(name: string) {
      return changes[name] ? changeObj(name) : null
    },
    async changeExists(name: string) {
      return Boolean(changes[name])
    },
    async createChange(name: string, metaText: string) {
      if (changes[name]) throw new Error(`change '${name}' already exists`)
      changes[name] = { artifacts: { '.openspec.yaml': metaText } }
      return `changes/${name}`
    },
    async updatedAtSecs(name: string) {
      return changes[name]?.updatedAtSecs ?? 0
    },
    // 原始 metadata 文件（選填的宿主方法）——蓋章動詞經這一對讀寫 .openspec.yaml。
    async readChangeMeta(name: string) {
      return changes[name]?.artifacts['.openspec.yaml'] ?? null
    },
    async writeChangeMeta(name: string, content: string) {
      if (!changes[name]) throw new Error(`change '${name}' not found`)
      changes[name].artifacts['.openspec.yaml'] = content
    },
    // --- artifacts ---
    async readArtifact(change: string, artifact: string) {
      return changes[change]?.artifacts[artifact] ?? null
    },
    async writeArtifact(change: string, artifact: string, content: string) {
      if (!changes[change]) throw new Error(`change '${change}' not found`)
      changes[change].artifacts[artifact] = content
      return `changes/${change}/${artifact}`
    },
    async artifactExists(change: string, artifact: string) {
      return changes[change]?.artifacts[artifact] !== undefined
    },
    async deleteArtifact(change: string, artifact: string) {
      delete changes[change]?.artifacts[artifact]
    },
    // --- delta specs ---
    async deltaCapabilities(change: string) {
      const caps = Object.keys(changes[change]?.artifacts ?? {})
        .map((p) => /^specs\/([^/]+)\/spec\.md$/.exec(p)?.[1])
        .filter((c): c is string => Boolean(c))
      return [...new Set(caps)].sort()
    },
    async hasCapabilityDirs(change: string) {
      return Object.keys(changes[change]?.artifacts ?? {}).some((p) => p.startsWith('specs/'))
    },
    // --- canonical specs ---
    async listCanonicalCapabilities() {
      return Object.keys(specs)
    },
    async canonicalSpecExists(cap: string) {
      return specs[cap] !== undefined
    },
    async readCanonicalSpec(cap: string) {
      return specs[cap] ?? null
    },
    async writeCanonicalSpec(cap: string, content: string) {
      specs[cap] = content
    },
    async canonicalSpecPath(cap: string) {
      return `specs/${cap}/spec.md`
    },
    // --- archive ---
    async archivedChangeExists(datedName: string) {
      return archived.has(datedName)
    },
    async archiveChange(name: string, datedName: string) {
      if (!changes[name]) throw new Error(`change '${name}' not found`)
      archived.set(datedName, JSON.stringify(changes[name]))
      delete changes[name]
    },
    async readArchivedMeta(datedName: string) {
      const raw = archived.get(datedName)
      if (!raw) return null
      return (JSON.parse(raw).artifacts ?? {})['.openspec.yaml'] ?? null
    },
    async writeArchivedMeta(datedName: string, content: string) {
      const raw = archived.get(datedName)
      if (!raw) throw new Error(`archived change '${datedName}' not found`)
      const parsed = JSON.parse(raw)
      parsed.artifacts['.openspec.yaml'] = content
      archived.set(datedName, JSON.stringify(parsed))
    },
    // --- discussions ---
    async liveDiscussionExists(slug: string) {
      return liveDiscussions.has(slug)
    },
    async archivedDiscussionExists(slug: string) {
      return [...archivedDiscussions.keys()].some((k) => k.endsWith(`-${slug}`) || k === slug)
    },
    async liveDiscussionPath(slug: string) {
      return `discussions/${slug}.md`
    },
    async readLiveDiscussion(slug: string) {
      return liveDiscussions.get(slug) ?? null
    },
    async writeLiveDiscussion(slug: string, content: string) {
      liveDiscussions.set(slug, content)
      return `discussions/${slug}.md`
    },
    async deleteLiveDiscussion(slug: string) {
      if (!liveDiscussions.delete(slug)) throw new Error(`discussion '${slug}' not found`)
    },
    async readDiscussion(slug: string) {
      const live = liveDiscussions.get(slug)
      if (live !== undefined) return discussionDoc(slug, live, false)
      const key = [...archivedDiscussions.keys()]
        .filter((k) => k.endsWith(`-${slug}`) || k === slug)
        .sort()
        .pop()
      if (key === undefined) return null
      return discussionDoc(slug, archivedDiscussions.get(key)!, true)
    },
    async listLiveDiscussions() {
      return [...liveDiscussions.entries()].map(([slug, text]) => discussionDoc(slug, text, false))
    },
    async listArchivedDiscussions() {
      return [...archivedDiscussions.entries()]
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([key, text]) => discussionDoc(key, text, true))
    },
    async archiveDiscussion(slug: string, created: string) {
      const live = liveDiscussions.get(slug)
      if (live === undefined) return null
      const stored = `${created}-${slug}`
      archivedDiscussions.set(stored, live)
      liveDiscussions.delete(slug)
      return stored
    },
    // --- workflow config / shared vocabulary ---
    async readWorkflowConfig() {
      return project.workflowConfig ?? null
    },
    async readLanguage() {
      return project.language ?? null
    },
  }
}
