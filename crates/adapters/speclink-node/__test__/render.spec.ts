// Render API tests (spec: 渲染 API — 中性 tool-call 渲染 / 與 CLI 生成一致).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execFileSync, execSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { skills } from '../index.js'
import { cliBin, repoRoot } from './helpers'

const normalize = (s: string) => s.replace(/\r\n/g, '\n')

let initedProject: string
let copilotProject: string

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  initedProject = mkdtempSync(join(tmpdir(), 'speclink-node-init-'))
  execFileSync(cliBin, ['init', '.', '--tools', 'claude'], { cwd: initedProject })
  copilotProject = mkdtempSync(join(tmpdir(), 'speclink-node-init-copilot-'))
  execFileSync(cliBin, ['init', '.', '--tools', 'copilot'], { cwd: copilotProject })
}, 600_000)

afterAll(() => {
  if (initedProject) rmSync(initedProject, { recursive: true, force: true })
  if (copilotProject) rmSync(copilotProject, { recursive: true, force: true })
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
  it('neutral defaults match explicit cli, openspec and speclink options', () => {
    expect(skills.render('propose', { target: 'neutral' })).toBe(skills.render('propose', {
      target: 'neutral', invocation: 'cli', specDir: 'openspec', toolName: 'speclink',
    }))
  })

  it('neutral tool-call substitutes the custom specDir and toolName', () => {
    const rendered = skills.render('verify', {
      target: 'neutral', invocation: 'tool-call', specDir: 'specs-local', toolName: 'project-tool',
    })
    expect(rendered).toContain('calling the speclink tool')
    expect(rendered).toContain('--agent project-tool')
    expect(rendered).toContain('specs-local/changes/')
    expect(rendered).not.toContain('{{')
  })

  it('unknown invocation fails loud, listing the supported values', () => {
    expect(() => skills.render('apply', {
      target: 'neutral', invocation: 'shell' as never,
    })).toThrow("invocation 'shell' must be 'cli' or 'tool-call'")
  })

  it('copilot renders native syntax for every skill and matches CLI output', () => {
    const fixture = mkdtempSync(join(tmpdir(), 'speclink-node-copilot-'))
    try {
      execFileSync(cliBin, ['init', '.', '--tools', 'copilot'], { cwd: fixture })
      for (const { name } of skills.list()) {
        const copilot = skills.render(name, { target: 'copilot' })
        expect(copilot).not.toContain('$speclink-')
        expect(copilot).not.toContain('{{')
        const path = join(fixture, '.github/skills', `speclink-${name}`, 'SKILL.md')
        if (existsSync(path)) expect(normalize(copilot)).toBe(normalize(readFileSync(path, 'utf8')))
      }
    } finally {
      rmSync(fixture, { recursive: true, force: true })
    }
  })

  it('unknown targets list copilot and unknown skills still fail', () => {
    expect(() => skills.render('propose', { target: 'vscode' as never })).toThrow(/claude.*codex.*copilot.*antigravity.*neutral/)
    expect(() => skills.render('no-such-skill', { target: 'copilot' })).toThrow(/Unknown skill/)
  })
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

  it('unknown target fails loud, listing every valid target', () => {
    expect(() => skills.render('apply', { target: 'github-copilot' as never })).toThrow(
      /'github-copilot' must be 'claude', 'codex', 'copilot', 'antigravity', or 'neutral'/,
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

it('antigravity renders the same skills as init under its independent .agent directory',()=>{
 const fixture=mkdtempSync(join(tmpdir(),'speclink-node-antigravity-'));
 try {
  execFileSync(cliBin,['init','.','--tools','codex,antigravity'],{cwd:fixture});
  for(const {name} of skills.list()) {
   const path=join(fixture,'.agent','skills',`speclink-${name}`,'SKILL.md');
   if(existsSync(path))expect(normalize(skills.render(name,{target:'antigravity'}))).toBe(normalize(readFileSync(path,'utf8')));
  }
  expect(readFileSync(join(fixture,'.agents','skills','speclink-propose','SKILL.md'),'utf8')).toContain('$speclink-');
  expect(readFileSync(join(fixture,'.agent','skills','speclink-propose','SKILL.md'),'utf8')).toContain('/speclink-');
 } finally {rmSync(fixture,{recursive:true,force:true})}
});

it('index.d.ts accepts antigravity target under tsc typecheck', () => {
  const fixture = mkdtempSync(join(tmpdir(), 'speclink-node-types-'))
  try {
    const pkgRoot = join(repoRoot, 'crates', 'adapters', 'speclink-node')
    const tsFile = join(fixture, 'test.ts')
    writeFileSync(
      tsFile,
      `import { skills } from ${JSON.stringify(pkgRoot)}\nskills.render('apply', { target: 'antigravity' })\n`,
    )
    const tscBin = join(repoRoot, 'node_modules', 'typescript', 'bin', 'tsc')
    execFileSync(process.execPath, [
      tscBin,
      '--noEmit',
      '--strict',
      '--skipLibCheck',
      '--moduleResolution',
      'node',
      '--target',
      'ES2022',
      tsFile,
    ])
  } finally {
    rmSync(fixture, { recursive: true, force: true })
  }
})
