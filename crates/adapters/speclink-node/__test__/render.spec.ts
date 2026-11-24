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

/** The golden snapshot of the remote neutral tool-call skill files, by file path. */
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
    const section = remoteSnapshot().get('.xyz/skills/speclink-archive/SKILL.md')
    expect(section).toBeDefined()
    const rendered = skills.render('archive', {
      target: 'neutral',
      invocation: 'tool-call',
      specInCheckout: false,
      toolName: 'xyz-harness',
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

// spec node-sdk「渲染 API 的宿主限制選項」.
describe('skills.render — harness limits', () => {
  const HOST_SKILLS = ['discuss', 'trace', 'propose', 'ingest', 'analyze', 'config']

  /** The rendered text cut around its `## Harness limits` section: before, the section, after. */
  function aroundLimits(rendered: string): [string, string, string] {
    const start = rendered.indexOf('## Harness limits\n')
    expect(start, rendered).toBeGreaterThanOrEqual(0)
    const end = rendered.indexOf('\n---\n\n', start) + '\n---\n\n'.length
    return [rendered.slice(0, start), rendered.slice(start, end), rendered.slice(end)]
  }

  /** The items of a `## Harness limits` section. */
  const items = (section: string) => section.split('\n').filter((line) => line.startsWith('- '))

  it('declares the skills the host provides, after the invocation preamble', () => {
    const base = { target: 'neutral', invocation: 'tool-call', specInCheckout: false } as const
    const [before, section, after] = aroundLimits(skills.render('propose', { ...base, skills: HOST_SKILLS }))
    expect(before).toContain('## Invocation')
    expect(before.endsWith('---\n\n')).toBe(true)
    for (const name of HOST_SKILLS) expect(section).toContain(`\`speclink-${name}\``)
    expect(section).not.toContain('speclink-apply')
    expect(section).toContain('do not load it, and do not tell the user to run it')
    expect(items(section)).toHaveLength(1)
    expect(before + after).toBe(skills.render('propose', base))
  })

  it('declares no checkout, and renders the body of specInCheckout: false', () => {
    const base = { target: 'neutral', invocation: 'tool-call' } as const
    const [before, section, after] = aroundLimits(skills.render('discuss', { ...base, checkout: false }))
    expect(items(section)).toHaveLength(1)
    const [item] = items(section)
    expect(item).toContain('no checkout of the code and no git history')
    expect(item).toContain('Skip every step of this document that reads code files')
    expect(item).toContain('runs a git command')
    expect(before + after).toBe(skills.render('discuss', { ...base, specInCheckout: false }))
    // discuss 有規格資料夾的條件區塊：兩份內文不同，所以上一行驗到了連帶的 specInCheckout。
    expect(before + after).not.toBe(skills.render('discuss', base))
  })

  it('puts the skills item first when both options are given', () => {
    const rendered = skills.render('trace', {
      target: 'neutral',
      invocation: 'tool-call',
      checkout: false,
      skills: ['discuss', 'trace'],
    })
    const list = items(aroundLimits(rendered)[1])
    expect(list).toHaveLength(2)
    expect(list[0]).toContain('provides only these skills: `speclink-discuss`, `speclink-trace`.')
    expect(list[1]).toContain('no checkout of the code and no git history')
  })

  it('renders the same text with checkout: true as without the two options', () => {
    const base = { target: 'neutral', invocation: 'tool-call', specInCheckout: false } as const
    const omitted = skills.render('propose', base)
    expect(skills.render('propose', { ...base, checkout: true })).toBe(omitted)
    expect(omitted).not.toContain('## Harness limits')
    // The golden snapshot is the output from before the two options.
    const section = remoteSnapshot().get('.xyz/skills/speclink-propose/SKILL.md')
    expect(section).toBeDefined()
    const rendered = skills.render('propose', { ...base, toolName: 'xyz-harness', checkout: true })
    expect(normalize(rendered).trimEnd()).toBe(section!.trimEnd())
  })

  it('works for the claude target, naming each skill the Claude way', () => {
    const [before, section, after] = aroundLimits(
      skills.render('propose', { target: 'claude', skills: ['propose', 'discuss'] }),
    )
    expect(section).toContain('only these skills: `/speclink-discuss`, `/speclink-propose`.')
    expect(before + after).toBe(skills.render('propose', { target: 'claude' }))
  })

  const NO_GIT_ITEM =
    '- This harness has a copy of the code that you can read and search, but it has no git history ' +
    'and cannot run git. Skip every step of this document that runs a git command. When a step names ' +
    'another way to get the same fact, use that way. In your answer, say which steps you skipped for this reason.'
  const NO_CHECKOUT_ITEM =
    '- This harness has no checkout of the code and no git history. Skip every step of this document ' +
    'that reads code files, searches the code, or runs a git command. Check facts only with speclink ' +
    'verbs. In your answer, say which steps you skipped for this reason.'

  it('declares code that can be read but no git, and keeps the rest of trace', () => {
    const base = { target: 'neutral', invocation: 'tool-call', specInCheckout: false } as const
    const [before, section, after] = aroundLimits(skills.render('trace', { ...base, gitHistory: false }))
    expect(items(section)).toEqual([NO_GIT_ITEM])
    expect(before + after).toBe(skills.render('trace', base))
  })

  it('puts the skills item before the no-git item', () => {
    const rendered = skills.render('trace', {
      target: 'neutral',
      invocation: 'tool-call',
      specInCheckout: false,
      gitHistory: false,
      skills: ['discuss', 'trace'],
    })
    const list = items(aroundLimits(rendered)[1])
    expect(list).toHaveLength(2)
    expect(list[0]).toContain('provides only these skills: `speclink-discuss`, `speclink-trace`.')
    expect(list[1]).toBe(NO_GIT_ITEM)
  })

  it('follows checkout: false with gitHistory, and renders what checkout: false rendered before', () => {
    const base = { target: 'neutral', invocation: 'tool-call', checkout: false } as const
    const omitted = skills.render('discuss', base)
    expect(skills.render('discuss', { ...base, gitHistory: false })).toBe(omitted)
    const [before, section, after] = aroundLimits(omitted)
    // The output from before gitHistory: the one no-checkout item, and the body of specInCheckout: false.
    expect(section).toBe(`## Harness limits\n\n${NO_CHECKOUT_ITEM}\n\n---\n\n`)
    expect(before + after).toBe(skills.render('discuss', { target: 'neutral', invocation: 'tool-call', specInCheckout: false }))
  })

  it('renders the same text with gitHistory: true as without it', () => {
    const base = { target: 'neutral', invocation: 'tool-call', specInCheckout: false } as const
    const omitted = skills.render('trace', base)
    expect(skills.render('trace', { ...base, gitHistory: true })).toBe(omitted)
    expect(skills.render('trace', { ...base, checkout: true, gitHistory: true })).toBe(omitted)
    expect(omitted).not.toContain('## Harness limits')
  })

  it('leaves the specInCheckout default alone', () => {
    const omitted = skills.render('discuss', { target: 'neutral', gitHistory: false })
    expect(skills.render('discuss', { target: 'neutral', specInCheckout: true, gitHistory: false })).toBe(omitted)
    // discuss 有規格資料夾的條件區塊：兩份內文不同，所以上一行驗到了 specInCheckout 的預設。
    expect(omitted).not.toBe(skills.render('discuss', { target: 'neutral', specInCheckout: false, gitHistory: false }))
  })

  // Example「各種不合法的值」：每一列都拋出錯誤，沒有一列回傳字串。
  it.each([
    [{ skills: 'propose' }, undefined],
    [{ skills: [] }, /empty/],
    [{ skills: ['propose', 'aply'] }, /aply/],
    [{ skills: ['discuss'] }, /propose/],
    [{ skills: ['propose', 7] }, undefined],
    [{ checkout: 'false' }, undefined],
    [{ checkout: null }, undefined],
    [{ checkout: false, specInCheckout: true }, /checkout: false.*specInCheckout: true/],
    [{ gitHistory: 'false' }, /bool.*gitHistory/],
    [{ gitHistory: null }, /bool.*gitHistory/],
    [{ checkout: false, gitHistory: true }, /checkout: false.*gitHistory: true/],
  ])('refuses %o', (bad, message) => {
    expect(() => skills.render('propose', { target: 'neutral', ...bad } as never)).toThrow(message)
  })

  it('refuses a gitHistory that only looks true or false, instead of reading it as a boolean', () => {
    for (const bad of [0, 1, 'true', {}]) {
      expect(() => skills.render('propose', { target: 'neutral', gitHistory: bad } as never)).toThrow(/bool.*gitHistory/)
    }
  })

  it('refuses a skills, checkout or gitHistory key that holds undefined, instead of reading it as left out', () => {
    const render = (extra: object) => () => skills.render('propose', { target: 'neutral', ...extra } as never)
    expect(render({ skills: undefined })).toThrow(/skills must be an array.*undefined/)
    expect(render({ checkout: undefined })).toThrow(/checkout must be true or false.*undefined/)
    expect(render({ gitHistory: undefined })).toThrow(TypeError)
    expect(render({ gitHistory: undefined })).toThrow(/gitHistory must be true or false.*undefined/)
  })

  it('tells a caller who writes the rendered name to leave the prefix out', () => {
    expect(() => skills.render('propose', { target: 'neutral', skills: ['speclink-propose'] })).toThrow(
      /'speclink-propose' is not a skill name.*without the speclink- prefix/,
    )
  })
})

// spec node-sdk「渲染 API 拒絕不認得的選項鍵」.
describe('skills.render — option keys', () => {
  const KEYS = ['target', 'invocation', 'specInCheckout', 'specDir', 'toolName', 'skills', 'checkout', 'gitHistory']

  /** The message `skills.render` throws for these options. */
  const thrown = (options: object) => {
    try {
      skills.render('propose', options as never)
    } catch (e) {
      return (e as Error).message
    }
    throw new Error('skills.render did not throw')
  }

  it('refuses a misspelled key, naming it and every valid key', () => {
    const message = thrown({ target: 'neutral', skils: ['propose'] })
    expect(message).toContain('skils')
    for (const key of KEYS) expect(message).toContain(key)
  })

  it('names gitHistory among the valid keys when gitHistory is misspelled', () => {
    const message = thrown({ target: 'neutral', gitHistroy: false })
    expect(message).toContain('gitHistroy')
    expect(message).toMatch(/the options are .*\bgitHistory\b/)
  })

  it('renders as before when every key is valid', () => {
    const options = {
      target: 'neutral',
      invocation: 'tool-call',
      specInCheckout: false,
      specDir: 'speclink',
      toolName: 'xyz-harness',
    } as const
    const section = remoteSnapshot().get('.xyz/skills/speclink-propose/SKILL.md')
    expect(normalize(skills.render('propose', options)).trimEnd()).toBe(section!.trimEnd())
    // The scenario's own values: `specDir` and `toolName` hold their defaults.
    const base = { target: 'neutral', invocation: 'tool-call', specInCheckout: false } as const
    expect(skills.render('propose', { ...base, specDir: 'speclink', toolName: 'speclink' })).toBe(
      skills.render('propose', base),
    )
  })
})
