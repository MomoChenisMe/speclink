import { spawnSync } from 'node:child_process'
import { createRequire } from 'node:module'
import { join } from 'node:path'
import { expect, it } from 'vitest'

const require = createRequire(import.meta.url)

it('compiles a consumer of the public package types, including Antigravity', () => {
  const result = spawnSync(process.execPath, [
    require.resolve('typescript/bin/tsc'),
    '--project', join(__dirname, 'types', 'tsconfig.json'),
  ], { encoding: 'utf8' })
  expect(result.error).toBeUndefined()
  expect(result.status, result.stdout + result.stderr).toBe(0)
})
