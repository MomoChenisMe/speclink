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
let copilotProject: string
let piCursorProject: string

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  initedProject = mkdtempSync(join(tmpdir(), 'speclink-node-init-'))
  execFileSync(cliBin, ['init', '.', '--tools', 'claude'], { cwd: initedProject })
  copilotProject = mkdtempSync(join(tmpdir(), 'speclink-node-init-copilot-'))
  execFileSync(cliBin, ['init', '.', '--tools', 'copilot'], { cwd: copilotProject })
  piCursorProject = mkdtempSync(join(tmpdir(), 'speclink-node-init-pi-cursor-'))
  execFileSync(cliBin, ['init', '.', '--tools', 'pi,cursor'], { cwd: piCursorProject })
}, 600_000)

afterAll(() => {
  if (initedProject) rmSync(initedProject, { recursive: true, force: true })
  if (copilotProject) rmSync(copilotProject, { recursive: true, force: true })
  if (piCursorProject) rmSync(piCursorProject, { recursive: true, force: true })
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
      specInCheckout: false,
    })
    expect(s).toContain('calling the speclink tool')
    expect(s).toContain('argv')
    expect(s).not.toContain('/speclink-')
    expect(s).not.toContain('$speclink-')
    // 本地規格路徑句：預設的規格資料夾是 speclink/（`.speclink/` 是工作資料夾，不算）。
    expect(s).not.toMatch(/(^|[^.\w/-])speclink\/(changes|specs|discussions|manual)/m)
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

  it('copilot × cli matches the SKILL.md the CLI init generates under .github/skills', () => {
    const generated = readFileSync(
      join(copilotProject, '.github', 'skills', 'speclink-apply', 'SKILL.md'),
      'utf8',
    )
    const rendered = skills.render('apply', { target: 'copilot' })
    expect(normalize(rendered)).toBe(normalize(generated))
    expect(rendered).toContain('/speclink-')
    expect(rendered).not.toContain('$speclink-')
  })

  it('pi × cli matches the SKILL.md the CLI init generates under .pi/skills', () => {
    const generated = readFileSync(
      join(piCursorProject, '.pi', 'skills', 'speclink-apply', 'SKILL.md'),
      'utf8',
    )
    const rendered = skills.render('apply', { target: 'pi' })
    expect(normalize(rendered)).toBe(normalize(generated))
    expect(rendered).toContain('/skill:speclink-')
    expect(rendered).not.toContain('$speclink-')
  })

  it('cursor × cli matches the SKILL.md the CLI init generates under .cursor/skills', () => {
    const generated = readFileSync(
      join(piCursorProject, '.cursor', 'skills', 'speclink-apply', 'SKILL.md'),
      'utf8',
    )
    const rendered = skills.render('apply', { target: 'cursor' })
    expect(normalize(rendered)).toBe(normalize(generated))
    expect(rendered).toContain('/speclink-')
    expect(rendered).not.toContain('$speclink-')
  })

  // spec node-sdk「渲染 API」：specInCheckout（規格資料夾是否在使用這份技能的 checkout）。
  const remoteSnapshot = () => {
    const text = normalize(
      readFileSync(
        join(repoRoot, 'crates/engine/speclink-core/tests/golden/neutral-tool-call-remote.snapshot.md'),
        'utf8',
      ),
    )
    const parts = text.split(/^=== (.*) ===\n/m)
    const sections = new Map<string, string>()
    for (let i = 1; i < parts.length; i += 2) sections.set(parts[i], parts[i + 1])
    return sections
  }

  it('specInCheckout false renders no spec-folder path and names hand-offs as skills', () => {
    const s = skills.render('archive', {
      target: 'neutral',
      invocation: 'tool-call',
      specInCheckout: false,
      specDir: 'openspec',
    })
    expect(s).not.toContain('openspec/')
    expect(s).toContain('`speclink-ingest <name>`')
    expect(s).not.toContain('`speclink ingest <name>`')
    // 動詞仍是動詞寫法。
    expect(s).toContain('speclink archive <name>')
  })

  it('specInCheckout false matches the remote neutral tool-call snapshot', () => {
    const section = remoteSnapshot().get('.wad/skills/speclink-archive/SKILL.md')
    expect(section).toBeDefined()
    const rendered = skills.render('archive', {
      target: 'neutral',
      invocation: 'tool-call',
      specInCheckout: false,
      toolName: 'wad-harness',
    })
    expect(normalize(rendered).trimEnd()).toBe(section!.trimEnd())
  })

  it('omitting specInCheckout equals true, and both match what the CLI generates locally', () => {
    const generated = readFileSync(
      join(initedProject, '.claude', 'skills', 'speclink-commit', 'SKILL.md'),
      'utf8',
    )
    const omitted = skills.render('commit', { target: 'claude' })
    const explicit = skills.render('commit', { target: 'claude', specInCheckout: true })
    expect(explicit).toBe(omitted)
    expect(normalize(omitted)).toBe(normalize(generated))
    // 兩個值渲染出不同的內容：這個選項不是擺設。
    expect(skills.render('commit', { target: 'claude', specInCheckout: false })).not.toBe(omitted)
  })

  it('a non-boolean specInCheckout fails loud instead of being read as true', () => {
    for (const bad of ['false', 0, {}, null]) {
      expect(() =>
        skills.render('commit', { target: 'claude', specInCheckout: bad as never }),
      ).toThrow()
    }
  })

  it('unknown target fails loud, listing every valid target', () => {
    expect(() => skills.render('apply', { target: 'github-copilot' as never })).toThrow(
      /'github-copilot' must be 'claude', 'codex', 'copilot', 'pi', 'cursor', or 'neutral'/,
    )
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
