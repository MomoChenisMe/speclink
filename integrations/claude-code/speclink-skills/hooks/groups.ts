// 技能分頁表，依工作流程排序。表上沒有的技能一律落到 `other`，不會消失；
// `a+b` 是一顆合併鈕，填入 `/speclink-a + /speclink-b`。
//
// scripts/claude-code/skill-groups.test.mjs 拿這張表對照 repo 裡 `speclink update`
// 寫出的技能：新技能要明確放進某個分頁，改名或移除的技能不得留在表上。
export type GroupId = 'plan' | 'build' | 'quality' | 'ship' | 'other'

export const GROUPS: { id: GroupId; skills: string[] }[] = [
  { id: 'plan', skills: ['discuss', 'improve', 'propose', 'ingest'] },
  { id: 'build', skills: ['apply', 'apply-with-worktree'] },
  { id: 'quality', skills: ['quality', 'review', 'verify'] },
  { id: 'ship', skills: ['archive', 'archive+commit', 'worktree-merge'] },
  {
    id: 'other',
    skills: ['commit', 'analyze', 'drift', 'audit', 'trace', 'manual', 'baseline', 'config'],
  },
]
