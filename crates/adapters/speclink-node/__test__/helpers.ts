// Shared test helpers: the fs fixture project and CLI invocation, so engine
// output can be compared to CLI output field by field.
import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join, relative, sep } from 'node:path'
import { afterAll, expect } from 'vitest'

import { createEngine } from '../index.js'

export const repoRoot = join(__dirname, '..', '..', '..', '..')
// 本機 cargo test 產 debug CLI；CI 的 Node SDK workflow 只建 release——擇一存在者。
const binName = process.platform === 'win32' ? 'speclink.exe' : 'speclink'
const debugBin = join(repoRoot, 'target', 'debug', binName)
export const cliBin = existsSync(debugBin) ? debugBin : join(repoRoot, 'target', 'release', binName)

/** Run the CLI in the fixture project and parse its --json stdout. */
export function cliJson(cwd: string, args: string[]): unknown {
  const out = execFileSync(cliBin, args, { cwd, encoding: 'utf8' })
  return JSON.parse(out)
}

/** The Error a promise rejects with; fails the test when it resolves. */
export async function rejection(promise: Promise<unknown>): Promise<Error & { code?: string; reason?: string }> {
  let thrown: (Error & { code?: string; reason?: string }) | undefined
  try {
    await promise
  } catch (e) {
    thrown = e as Error & { code?: string; reason?: string }
  }
  expect(thrown).toBeInstanceOf(Error)
  return thrown!
}

/** Every file under `root`, as path (forward slashes) → what `read` gives for it. */
function eachFile<T>(root: string, read: (file: string) => T): Record<string, T> {
  const files: Record<string, T> = {}
  const walk = (dir: string) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name)
      if (entry.isDirectory()) walk(full)
      else files[relative(root, full).split(sep).join('/')] = read(full)
    }
  }
  walk(root)
  return files
}

/**
 * Every file under `root`, as path (forward slashes) → content: compare two of
 * them to see what a call wrote, or that it wrote nothing.
 */
export const projectFiles = (root: string) => eachFile(root, (file) => readFileSync(file, 'utf8'))

/** Every file under `root`, as path (forward slashes) → modification time. */
export const fileTimes = (root: string) => eachFile(root, (file) => statSync(file).mtimeMs)

/** The digest of a bundle document: `sha256:` and the hex SHA-256 of its content (UTF-8). */
export const digestOf = (content: string) => `sha256:${createHash('sha256').update(content, 'utf8').digest('hex')}`

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

/** A workflow config that is not YAML. */
export const BAD_CONFIG = ': not yaml : [\n'

// Every fixture project a test file made is removed when the file is done.
const fixtureRoots: string[] = []
afterAll(() => {
  // maxRetries: Windows can still hold a file of a project for a moment.
  for (const root of fixtureRoots.splice(0)) rmSync(root, { recursive: true, force: true, maxRetries: 5 })
})

/** A fixture project with two changes (task progress + proposal summaries) and one spec. */
export function makeFsFixture(): string {
  // realpath: macOS 的 tmpdir 是 /var → /private/var 的 symlink；CLI 以 getcwd 回報
  // 實體路徑，engine 則沿用傳入字串——不先解析，兩邊的 path 欄位無法逐位元比對。
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-node-fixture-')))
  fixtureRoots.push(root)
  const changes = join(root, 'speclink', 'changes')

  const alpha = join(changes, 'alpha')
  mkdirSync(alpha, { recursive: true })
  writeFileSync(
    join(alpha, '.meta.yaml'),
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
    join(beta, '.meta.yaml'),
    'schema: spec-driven\ncreated: "2026-07-02"\ncreated_by: tester\n',
  )
  writeFileSync(join(beta, 'proposal.md'), PROPOSAL_BETA)
  writeFileSync(join(beta, 'tasks.md'), TASKS_BETA)

  const specDir = join(root, 'speclink', 'specs', 'user-auth')
  mkdirSync(specDir, { recursive: true })
  writeFileSync(join(specDir, 'spec.md'), SPEC_USER_AUTH)
  return root
}

/** An engine over a local project folder; a new fixture project when no root is given. */
export const fsEngine = (root: string = makeFsFixture(), actor?: string) =>
  createEngine({ store: { type: 'fs', root }, actor })

// A fixture project brings its own git identity: without one the developer's
// global config is used, and the assertions cannot be repeated.
export const GIT_NAME = 'Fixture Dev'
export const GIT_EMAIL = 'fixture@example.com'
export const GIT_IDENTITY = `${GIT_NAME} <${GIT_EMAIL}>`

/** Make `root` a git repo whose identity is the fixture's. */
export function initGitIdentity(root: string): void {
  execFileSync('git', ['init', '-q'], { cwd: root })
  execFileSync('git', ['config', 'user.name', GIT_NAME], { cwd: root })
  execFileSync('git', ['config', 'user.email', GIT_EMAIL], { cwd: root })
}
