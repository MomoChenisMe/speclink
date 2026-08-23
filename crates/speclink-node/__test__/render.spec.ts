// Render API tests (spec: 渲染 API — 中性 tool-call 渲染 / 與 CLI 生成一致).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execFileSync, execSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { skills } from '../index.js'
import { cliBin, repoRoot } from './helpers'

const normalize = (s: string) => s.replace(/\r\n/g, '\n')

let initedProject: string

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  initedProject = mkdtempSync(join(tmpdir(), 'speclink-node-init-'))
  execFileSync(cliBin, ['init', '.', '--tools', 'claude'], { cwd: initedProject })
}, 600_000)

afterAll(() => {
  if (initedProject) rmSync(initedProject, { recursive: true, force: true })
})

describe('skills.list', () => {
  it('returns the skill registry (names and descriptions)', () => {
    const list = skills.list()
    const names = list.map((s: { name: string }) => s.name)
    expect(names).toContain('propose')
    expect(names).toContain('apply')
    expect(names).toContain('archive')
    const propose = list.find((s: { name: string }) => s.name === 'propose')
    // 入口路由由 description 承載：觸發情境句在前，產出在後。
    expect(propose.description).toMatch(/^Use when /)
    expect(propose.description).toContain('planning, proposing or designing')
  })
})

describe('skills.render — render matrix', () => {
  it('neutral × tool-call speaks tool-call wording without local prefixes/paths', () => {
    const s = skills.render('propose', {
      target: 'neutral',
      invocation: 'tool-call',
    })
    expect(s).toContain('calling the speclink tool')
    expect(s).toContain('argv')
    expect(s).not.toContain('/speclink-')
    expect(s).not.toContain('$speclink-')
    expect(s.toLowerCase()).not.toContain('plan mode')
    expect(s).not.toContain('{{')
  })

  it('claude × cli matches the SKILL.md the CLI init generates', () => {
    const generated = readFileSync(
      join(initedProject, '.claude', 'skills', 'speclink-apply', 'SKILL.md'),
      'utf8',
    )
    const rendered = skills.render('apply', { target: 'claude', invocation: 'cli' })
    expect(normalize(rendered)).toBe(normalize(generated))
  })

  it('unknown skill name fails loud', () => {
    expect(() => skills.render('nope', { target: 'claude' })).toThrow(/Unknown skill/)
  })

  it('init generates no instruction file — routing rides the skill descriptions', () => {
    expect(existsSync(join(initedProject, 'CLAUDE.md'))).toBe(false)
    const propose = skills.list().find((s: { name: string }) => s.name === 'propose')
    expect(propose.description.length).toBeGreaterThan(0)
  })
})
