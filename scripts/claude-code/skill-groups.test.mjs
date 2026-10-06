// Claude Code mod 技能分頁表的同步守門（integrations/claude-code/speclink-skills）。
// mod 的分頁表寫死在 hooks/groups.ts；表上沒有的技能雖然會落到「其他」不會消失，
// 但新技能該放哪一頁要有人決定。這裡拿表對照 repo 裡 `speclink update` 寫出的
// `.claude/skills/speclink-*`：新技能沒進表、或表上留著已移除的名字，都在這裡紅。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const GROUPS_FILE = 'integrations/claude-code/speclink-skills/hooks/groups.ts';

/// groups.ts 的 `skills: [...]` 陣列裡的短名；合併鈕 `a+b` 拆成各自的技能。
function tabledSkills(source) {
  const names = [...source.matchAll(/skills:\s*\[([^\]]*)\]/g)].flatMap((block) =>
    [...block[1].matchAll(/'([^']+)'/g)].map((name) => name[1]),
  );
  return new Set(names.flatMap((name) => name.split('+')));
}

const tabled = tabledSkills(readFileSync(path.join(ROOT, GROUPS_FILE), 'utf8'));
const written = readdirSync(path.join(ROOT, '.claude/skills'))
  .filter((name) => name.startsWith('speclink-'))
  .map((name) => name.slice('speclink-'.length))
  .sort();

test('解析：讀得到分頁表，合併鈕拆成各自的技能', () => {
  const sample = "{ id: 'ship', skills: ['archive', 'archive+commit'] },\n{ id: 'x', skills: [\n 'trace',\n] }";
  assert.deepEqual([...tabledSkills(sample)].sort(), ['archive', 'commit', 'trace']);
  assert.ok(tabled.size > 0, `${GROUPS_FILE} 解析不到任何技能`);
});

test('speclink update 寫出的每個技能都在 mod 的分頁表上', () => {
  const missing = written.filter((name) => !tabled.has(name));
  assert.deepEqual(missing, [], `請把這些技能放進 ${GROUPS_FILE} 的某個分頁：${missing.join(', ')}`);
});

test('mod 的分頁表沒有已不存在的技能', () => {
  const stale = [...tabled].filter((name) => !written.includes(name)).sort();
  assert.deepEqual(stale, [], `${GROUPS_FILE} 裡這些技能已不存在：${stale.join(', ')}`);
});
