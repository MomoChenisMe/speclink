// Render API tests (spec: 渲染 API — 中性 tool-call 渲染 / 與 CLI 生成一致).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execFileSync, execSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { skills, instructions } from '../index.js'
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
    expect(propose.description).toBe('Create a change proposal with all required artifacts')
  })
})

describe('skills.render — render matrix', () => {
  it('neutral × tool-call × remote speaks tool-call wording without local prefixes/paths', () => {
    const s = skills.render('propose', {
      target: 'neutral',
      invocation: 'tool-call',
      store: 'remote',
    })
    expect(s).toContain('calling the speclink tool')
    expect(s).toContain('argv')
    expect(s).not.toContain('/speclink-')
    expect(s).not.toContain('$speclink-')
    expect(s.toLowerCase()).not.toContain('plan mode')
    expect(s).not.toContain('{{')
    // 不含本地規格路徑句 (the fs-marker sentence steering at local spec paths)
    expect(s).not.toContain('Specs live in')
  })

  it('claude × cli × fs matches the SKILL.md the CLI init generates', () => {
    const generated = readFileSync(
      join(initedProject, '.claude', 'skills', 'speclink-apply', 'SKILL.md'),
      'utf8',
    )
    const rendered = skills.render('apply', { target: 'claude', invocation: 'cli', store: 'fs' })
    expect(normalize(rendered)).toBe(normalize(generated))
  })

  it('unknown skill name fails loud', () => {
    expect(() => skills.render('nope', { target: 'claude' })).toThrow(/Unknown skill/)
  })
})

describe('instructions.render — render matrix', () => {
  it('neutral × tool-call × remote names the tool-call invocation and no local paths', () => {
    const block = instructions.render({ target: 'neutral', invocation: 'tool-call', store: 'remote' })
    expect(block).toContain('calling the speclink tool')
    expect(block).toContain('argv')
    expect(block).not.toContain('openspec/specs')
    expect(block).not.toContain('openspec/changes')
    expect(block).toContain('SPECLINK:START')
  })

  it('claude × fs matches the CLAUDE.md marker block the CLI init generates', () => {
    const generated = readFileSync(join(initedProject, 'CLAUDE.md'), 'utf8')
    const rendered = instructions.render({ target: 'claude', invocation: 'cli', store: 'fs' })
    expect(normalize(generated)).toBe(normalize(rendered))
  })

  it('the worktree axis toggles exactly the two worktree skill lines', () => {
    const on = instructions.render({ target: 'claude', store: 'fs', worktree: true })
    const off = instructions.render({ target: 'claude', store: 'fs', worktree: false })
    expect(on).toContain('apply-with-worktree')
    expect(on).toContain('worktree-merge')
    expect(off).not.toContain('apply-with-worktree')
    expect(off).not.toContain('worktree-merge')
    const offLines = new Set(normalize(off).split('\n'))
    const added = normalize(on)
      .split('\n')
      .filter((line) => !offLines.has(line))
    expect(added).toHaveLength(2)
  })

  it('an omitted worktree option renders the policy-off block', () => {
    const omitted = instructions.render({ target: 'claude', store: 'fs' })
    const explicit = instructions.render({ target: 'claude', store: 'fs', worktree: false })
    expect(normalize(omitted)).toBe(normalize(explicit))
  })
})
