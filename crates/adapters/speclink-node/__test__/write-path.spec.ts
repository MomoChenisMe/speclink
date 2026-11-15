// Write-path verbs and the stdin parameter form (spec: 寫入動詞經 stdin 參數 /
// 錯誤以語義化例外傳遞).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execSync } from 'node:child_process'
import { existsSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'

import { createEngine } from '../index.js'
import { cliBin, makeFsFixture, projectFiles, repoRoot } from './helpers'

let fsFixture: string

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  fsFixture = makeFsFixture()
}, 600_000)

afterAll(() => {
  if (fsFixture) rmSync(fsFixture, { recursive: true, force: true })
})

describe('dispatch write path — stdin parameter form', () => {
  // spec node-sdk「寫入動詞經 stdin 參數」
  it('new artifact --stdin writes the content into the change folder and resolves success', async () => {
    const root = makeFsFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    await engine.dispatch(['new', 'change', 'demo'])
    const before = projectFiles(root)
    const content = '## Why\n\nA proposal written through the SDK, with enough words to pass the checks.\n'
    const result = await engine.dispatch(['new', 'artifact', 'proposal', '--change', 'demo', '--stdin'], {
      stdin: content,
    })
    expect(result).toMatchObject({ artifact: 'proposal', change: 'demo', status: 'created' })
    // The proposal is the one file the call wrote.
    expect(projectFiles(root)).toEqual({ ...before, 'speclink/changes/demo/proposal.md': content })
  })

  it('new artifact --stdin on the fs form creates the file like the CLI', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fsFixture } })
    const content = '## Context\n\nDesign written through the SDK.\n'
    const result = (await engine.dispatch(
      ['new', 'artifact', 'design', '--change', 'alpha', '--stdin', '--json'],
      { stdin: content },
    )) as { artifact: string; change: string; path: string; status: string; validated: boolean }
    expect(result).toMatchObject({
      artifact: 'design',
      change: 'alpha',
      status: 'created',
      validated: true,
    })
    expect(result.path.endsWith('design.md')).toBe(true)
    const onDisk = readFileSync(join(fsFixture, 'speclink', 'changes', 'alpha', 'design.md'), 'utf8')
    expect(onDisk).toBe(content)
  })

  it('content validation failures reject with invalid_argv and the semantic CLI message', async () => {
    const root = makeFsFixture()
    const before = projectFiles(root)
    await expect(
      createEngine({ store: { type: 'fs', root } }).dispatch(
        ['new', 'artifact', 'tasks', '--change', 'alpha', '--stdin', '--force'],
        { stdin: 'no checkboxes here\n' },
      ),
    ).rejects.toMatchObject({
      message: 'Tasks must contain at least one checkbox (- [ ])',
      code: 'invalid_argv',
    })
    expect(projectFiles(root)).toEqual(before)
  })
})

describe('claim', () => {
  it('claim on the fs store fails loud like the CLI', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fsFixture } })
    await expect(engine.dispatch(['claim', 'alpha'])).rejects.toThrow(/claim requires a remote store/)
  })
})
