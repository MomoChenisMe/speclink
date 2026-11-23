// node-sdk「Agent 工具定義的匯出」: agentTool is the tool a host registers in
// its own Agent framework; its input goes to host.dispatch as it is.
import { describe, expect, it } from 'vitest'

import { agentTool, agentToolDescription, createHost } from '../index.js'
import { memoryPort } from './memory-port'

const ALICE = { id: 'u_1', displayName: 'Alice' }
const NEEDS_CHECKOUT = ['drift', 'review prepare', 'review scope', 'verify scope']

/** The usage of each line of the verb list: `- <usage>: <help>`. */
function verbLines(description: string = agentTool.description): string[] {
  const [, list] = description.split('\nVerbs:\n')
  return list
    .split('\n')
    .filter((line) => line.startsWith('- '))
    .map((line) => line.slice(2).split(': ')[0])
}

/** The verb words of a usage: the words before the first `[` or `<`. */
function words(usage: string): string {
  const out: string[] = []
  for (const word of usage.split(' ')) {
    if (word.startsWith('[') || word.startsWith('<')) break
    out.push(word)
  }
  return out.join(' ')
}

/**
 * The usage with a value for each required argument, the optional ones left
 * out. Required options go last: an option that takes several values
 * (`--on <OTHER>...`) would take the positionals after it.
 */
function minimalArgv(usage: string): string[] {
  // A required group `<--before <OTHER>|--after <OTHER>>` takes its first choice.
  const tokens = usage.replace(/<(--[\w-]+ <[A-Z_]+>)(?:\|--[\w-]+ <[A-Z_]+>)+>/g, '$1').split(' ')
  const [verb, positionals, options]: string[][] = [[], [], []]
  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i]
    if (token.startsWith('[')) continue
    if (token.startsWith('--')) {
      options.push(token)
      if (tokens[i + 1]?.startsWith('<')) {
        options.push('demo')
        i++
      }
    } else if (token.startsWith('<')) {
      positionals.push('demo')
    } else {
      verb.push(token)
    }
  }
  return [...verb, ...positionals, ...options]
}

describe('agentTool', () => {
  it('takes argv and an optional stdin, nothing else', () => {
    expect(agentTool.name).toBe('speclink')
    const schema = agentTool.inputSchema as Record<string, any>
    expect(schema.type).toBe('object')
    expect(Object.keys(schema.properties)).toEqual(['argv', 'stdin'])
    expect(schema.properties.argv).toMatchObject({ type: 'array', items: { type: 'string' } })
    expect(schema.properties.stdin).toMatchObject({ type: 'string' })
    expect(schema.required).toEqual(['argv'])
    expect(schema.additionalProperties).toBe(false)
    expect(agentTool.description).toContain('--stdin')
    expect(agentTool.description).toContain('--json')
  })

  // node-sdk「Agent 工具的說明指出 path 是儲存的鍵」
  it('tells the model that a path is a key in the store, and names the verbs that read', () => {
    const [usage] = agentTool.description.split('\nVerbs:\n')
    expect(usage).toContain(
      'A "path" in a result is the document\'s key in the store, not a file you can open. To read a document, use "artifact cat", "show" or "discuss show".',
    )
    const listed = verbLines().map(words)
    for (const verb of ['artifact cat', 'show', 'discuss show']) expect(listed).toContain(verb)
  })

  it('lists only verbs host.dispatch can parse, and none that needs a checkout', async () => {
    const usages = verbLines()
    const listed = usages.map(words)
    for (const verb of NEEDS_CHECKOUT) expect(listed).not.toContain(verb)
    expect(listed).toContain('discuss add-round')
    expect(listed).toContain('workflow-config set')
    expect(listed).toContain('change rank')
    expect(listed).toContain('change landed')
    expect(listed).toContain('evidence show')
    expect(listed).toContain('trace')
    expect(usages.filter((usage) => / stamp /.test(usage)).every((usage) => usage.includes('--stdin'))).toBe(true)

    const host = createHost({ store: memoryPort().store })
    for (const usage of usages) {
      const argv = minimalArgv(usage)
      try {
        await host.dispatch({ identity: ALICE, scope: 's1', argv, stdin: undefined })
      } catch (e) {
        // A missing change or a bad value is the engine's answer; a usage
        // error or a verb that needs a checkout would mean the list is wrong.
        const message = (e as Error).message
        expect(message, argv.join(' ')).not.toMatch(/Usage:|checkout/)
      }
    }
  })

  it('passes the tool input straight to host.dispatch', async () => {
    const host = createHost({ store: memoryPort().store })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['discuss', 'new', 'Host kit', '--slug', 'host-kit'] })
    const input = { argv: ['discuss', 'list', '--json'] }
    const result = (await host.dispatch({ identity: ALICE, scope: 's1', ...input })) as {
      discussions: { slug: string }[]
    }
    expect(Array.isArray(result.discussions)).toBe(true)
    expect(result.discussions.map((d) => d.slug)).toEqual(['host-kit'])
  })
})

// node-sdk「Agent 工具說明的動詞篩選」
describe('agentToolDescription', () => {
  const verbsOf = (description: string) => verbLines(description).map(words)
  const fullVerbs = () => verbsOf(agentTool.description)

  it('equals agentTool.description without options or with an empty exclude', () => {
    expect(agentToolDescription()).toBe(agentTool.description)
    expect(agentToolDescription({ exclude: [] })).toBe(agentTool.description)
  })

  it('leaves out the top-level archive and keeps discuss archive', () => {
    const description = agentToolDescription({ exclude: ['archive'] })
    const verbs = verbsOf(description)
    expect(verbs).not.toContain('archive')
    expect(verbs).toContain('discuss archive')
    expect(verbs).toEqual(fullVerbs().filter((verb) => verb !== 'archive'))
    expect(description.split('\nVerbs:\n')[0]).toBe(agentTool.description.split('\nVerbs:\n')[0])
  })

  it('leaves out every verb under review and verify', () => {
    const verbs = verbsOf(agentToolDescription({ exclude: ['review', 'verify'] }))
    for (const verb of ['review show', 'review add-round', 'review stamp', 'review discard']) {
      expect(fullVerbs()).toContain(verb)
    }
    expect(verbs.filter((verb) => /^(review|verify)( |$)/.test(verb))).toEqual([])
    expect(verbs).toContain('discuss show')
  })

  it('leaves out a two-word verb and keeps its siblings and the top-level verb', () => {
    const verbs = verbsOf(agentToolDescription({ exclude: ['discuss archive'] }))
    expect(verbs).not.toContain('discuss archive')
    expect(verbs).toContain('discuss list')
    expect(verbs).toContain('archive')
  })

  it('keeps only verbs host.dispatch can parse', async () => {
    const usages = verbLines(agentToolDescription({ exclude: ['archive', 'review'] }))
    expect(usages.map(words)).toEqual(
      fullVerbs().filter((verb) => verb !== 'archive' && verb !== 'review' && !verb.startsWith('review ')),
    )
    const host = createHost({ store: memoryPort().store })
    for (const usage of usages) {
      const argv = minimalArgv(usage)
      try {
        await host.dispatch({ identity: ALICE, scope: 's1', argv, stdin: undefined })
      } catch (e) {
        expect((e as Error).message, argv.join(' ')).not.toMatch(/Usage:|checkout/)
      }
    }
  })

  it('refuses options it cannot use, naming what is wrong', () => {
    expect(() => agentToolDescription({ exclude: 'archive' } as never)).toThrow()
    expect(() => agentToolDescription({ exclude: ['archiv'] })).toThrow(/archiv/)
    expect(() => agentToolDescription({ exclude: ['drift'] })).toThrow(/drift/)
    expect(() => agentToolDescription({ include: ['list'] } as never)).toThrow(/include/)
  })

  it('refuses options that are not a plain object, instead of listing every verb', () => {
    class Settings {
      exclude = ['archive']
    }
    for (const bad of [new Map([['exclude', ['archive']]]), new Date(), new Settings(), ['archive'], null, 'archive']) {
      expect(() => agentToolDescription(bad as never)).toThrow(/options must be a plain object/)
    }
  })

  it('does not change agentTool', () => {
    const before = JSON.stringify(agentTool)
    agentToolDescription({ exclude: ['archive', 'review'] })
    expect(JSON.stringify(agentTool)).toBe(before)
    expect(Object.keys(agentTool).sort()).toEqual(['description', 'inputSchema', 'name'])
    expect(agentTool.name).toBe('speclink')
    expect(Object.isFrozen(agentTool)).toBe(true)
  })
})
