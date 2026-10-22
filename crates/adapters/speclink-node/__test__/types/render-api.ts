// Resolve package.json's public `types` entry, not the generated binding.d.ts.
import { skills, type RenderOptions, type SkillInfo } from '../..'

const targets: RenderOptions['target'][] = ['claude', 'codex', 'copilot', 'antigravity', 'neutral']
const registry: SkillInfo[] = skills.list()
for (const target of targets) {
  const markdown: string = skills.render('apply', { target })
  const configured: string = skills.render('propose', {
    target, invocation: 'tool-call', specDir: 'specs-local', toolName: 'project-tool',
  })
}

// @ts-expect-error Unknown targets are not part of the public contract.
skills.render('apply', { target: 'vscode' })
// @ts-expect-error Invocation has a closed set of supported values.
skills.render('apply', { target: 'neutral', invocation: 'shell' })
// @ts-expect-error Store belongs to createEngine, not skill rendering.
skills.render('apply', { target: 'neutral', store: 'remote' })
// @ts-expect-error Skill rendering has no instruction-marker worktree axis.
skills.render('apply', { target: 'neutral', worktree: true })
// @ts-expect-error The retired instruction-file API is not exported.
import { instructions } from '../..'
