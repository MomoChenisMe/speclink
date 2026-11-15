// Write-path verbs and the stdin parameter form (spec: 寫入動詞經 stdin 參數 /
// 錯誤以語義化例外傳遞).
import { beforeAll, describe, expect, it } from 'vitest'
import { execSync } from 'node:child_process'
import { existsSync } from 'node:fs'

import { cliBin, fsEngine, makeFsFixture, projectFiles, repoRoot } from './helpers'

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
}, 600_000)

describe('dispatch write path — stdin parameter form', () => {
  // spec node-sdk「寫入動詞經 stdin 參數」: the first row is the scenario's own
  // call (a proposal for change demo); the second writes into a change of the fixture.
  it.each([
    ['proposal', 'demo', '## Why\n\nA proposal written through the SDK, with enough words to pass the checks.\n'],
    ['design', 'alpha', '## Context\n\nDesign written through the SDK.\n'],
  ])('new artifact %s --change %s --stdin writes the content into the change folder', async (artifact, change, content) => {
    const root = makeFsFixture()
    const engine = fsEngine(root)
    await engine.dispatch(['new', 'change', 'demo'])
    const before = projectFiles(root)
    const result = (await engine.dispatch(['new', 'artifact', artifact, '--change', change, '--stdin'], {
      stdin: content,
    })) as { path: string }
    expect(result).toMatchObject({ artifact, change, status: 'created', validated: true })
    expect(result.path.endsWith(`${artifact}.md`)).toBe(true)
    // The artifact is the one file the call wrote.
    expect(projectFiles(root)).toEqual({ ...before, [`speclink/changes/${change}/${artifact}.md`]: content })
  })

  it('content validation failures reject with invalid_argv and the semantic CLI message', async () => {
    const root = makeFsFixture()
    const before = projectFiles(root)
    await expect(
      fsEngine(root).dispatch(['new', 'artifact', 'tasks', '--change', 'alpha', '--stdin', '--force'], {
        stdin: 'no checkboxes here\n',
      }),
    ).rejects.toMatchObject({
      message: 'Tasks must contain at least one checkbox (- [ ])',
      code: 'invalid_argv',
    })
    expect(projectFiles(root)).toEqual(before)
  })
})

describe('claim', () => {
  it('claim on the fs store fails loud like the CLI', async () => {
    await expect(fsEngine().dispatch(['claim', 'alpha'])).rejects.toThrow(/claim requires a remote store/)
  })
})
