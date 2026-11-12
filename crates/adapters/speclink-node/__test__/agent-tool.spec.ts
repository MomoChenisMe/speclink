// node-sdk「Agent 工具定義的匯出」: agentTool is the tool a host registers in
// its own Agent framework; its input goes to host.dispatch as it is.
import { describe, expect, it } from 'vitest'

import { agentTool, createHost } from '../index.js'
import { memoryPort } from './memory-port'

const ALICE = { id: 'u_1', displayName: 'Alice' }
const NEEDS_CHECKOUT = ['drift', 'review prepare', 'review scope', 'verify scope']

/** Each line of the verb list: `- <usage>: <help>`. */
function verbLines(): string[] {
  const [, list] = agentTool.description.split('\nVerbs:\n')
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
