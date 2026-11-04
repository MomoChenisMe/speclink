// Herdr 面板的純邏輯（integrations/herdr/mods/speclink/model.js）：CLI 輸出怎麼變成
// 面板的列、每一列有哪些技能鈕、技能要送給誰。畫面與 Herdr API 不在這裡測。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  ALL_SKILLS,
  APP_ID,
  PICKER_SKILLS,
  attachPlan,
  agentChange,
  batchPlan,
  buildBoard,
  canOpenProject,
  changeActions,
  changeAgents,
  changeSkills,
  commandText,
  deliveryHint,
  discussionActions,
  discussionAgents,
  discussionSkills,
  inside,
  isUninitialized,
  joinAgents,
  groupedOrder,
  lastOpenError,
  logoOpenAttempts,
  newTabPlan,
  placePlan,
  planDelivery,
  sendArgs,
  commandLabels,
  hookLabels,
  promptLabels,
  skillLabels,
  skillTargets,
  statusCounts,
  statusSections,
  speclinkUrl,
  startChoices,
  waveToStart,
} from '../../integrations/herdr/mods/speclink/model.js';
import { TO_EN, TO_ZH, langFrom, setLang, tr } from '../../integrations/herdr/mods/speclink/i18n.js';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const REPO = '/work/repo';
const TREE = '/work/repo.worktrees/add-a';

const row = (over = {}) => ({
  name: 'add-a',
  stage: 'in-progress',
  wave: 1,
  blockedBy: [],
  completed: 1,
  total: 3,
  worktree: null,
  restale: false,
  ...over,
});
const agent = (over = {}) => ({
  address: 't1',
  status: 'done',
  key: 'claude',
  cwd: REPO,
  title: '',
  labels: {},
  tab: 'tab1',
  ...over,
});
const CLAUDE = [{ key: 'claude', run: ['claude'], installed: true }];
const ctx = (over = {}) => ({
  root: REPO,
  agents: [],
  focus: { tab: 'tab1', terminal: null },
  known: CLAUDE,
  ...over,
});

// ---- 面板資料 ----

test('buildBoard：階段、波次、順序取自 plan，任務數與 worktree 取自 list', () => {
  const board = buildBoard(
    {
      changes: [
        { name: 'add-b', completedTasks: 2, totalTasks: 4, status: 'in-progress' },
        {
          name: 'add-a',
          completedTasks: 0,
          totalTasks: 3,
          status: 'in-progress',
          worktree: { path: TREE, branch: 'speclink/add-a' },
          restaleFrom: ['talk'],
        },
      ],
    },
    {
      changes: [
        { name: 'add-a', wave: 1, stage: 'proposed', blockedBy: [] },
        { name: 'add-b', wave: 2, stage: 'in-progress', blockedBy: ['add-a'] },
      ],
      skipped: [],
      unpromotedCuts: [],
    },
    { discussions: [] },
  );
  assert.deepEqual(board.stages.proposed, [
    {
      name: 'add-a',
      stage: 'proposed',
      wave: 1,
      blockedBy: [],
      completed: 0,
      total: 3,
      codeRemaining: null,
      worktree: { path: TREE, branch: 'speclink/add-a' },
      restale: true,
    },
  ]);
  assert.deepEqual(board.stages['in-progress'][0].blockedBy, ['add-a']);
  assert.deepEqual(board.stages.ready, []);
});

test('isUninitialized：只認沒有專案的那句，設定檔壞掉仍算錯誤', () => {
  assert.equal(isUninitialized("Error: Not initialized. Run 'speclink init' to initialize."), true);
  assert.equal(isUninitialized('Error: failed to parse .speclink.yaml'), false);
});

test('buildBoard：metadata 壞掉的變更不會憑空消失', () => {
  const skipped = [{ change: 'bad', reason: 'parse error' }];
  const board = buildBoard({ changes: [] }, { changes: [], skipped, unpromotedCuts: [] }, { discussions: [] });
  assert.deepEqual(board.skipped, skipped);
});

test('buildBoard：討論依 status 分流，還有刀沒轉出的已轉出討論留在討論卡', () => {
  const board = buildBoard(
    { changes: [] },
    {
      changes: [],
      skipped: [],
      unpromotedCuts: [
        { discussion: 'in-flight', cut: 'cut 2', title: '', changes: ['add-b', 'add-c'], unpromoted: ['add-c'] },
      ],
    },
    {
      discussions: [
        { slug: 'talk', topic: '主題', status: 'open' },
        { slug: 'done-talk', topic: '定案', status: 'concluded' },
        { slug: 'in-flight', topic: '轉出一半', status: 'promoted' },
        { slug: 'all-out', topic: '全轉出', status: 'promoted' },
      ],
    },
  );
  assert.deepEqual(
    board.discussions.map((d) => [d.slug, d.cuts]),
    [
      ['talk', []],
      ['done-talk', []],
      ['in-flight', ['add-c']],
    ],
  );
  assert.deepEqual(
    board.promoted.map((d) => d.slug),
    ['all-out'],
  );
});

// ---- 技能表 ----

const names = (skills) => skills.map((s) => s.skill);
const primary = (skills) => skills.filter((s) => s.primary).map((s) => s.skill);

test('buildBoard：程式任務數取自 instructions，已就緒一定是 0，查不到是 null', () => {
  const board = buildBoard(
    { changes: [] },
    {
      changes: [
        { name: 'add-a', wave: 1, stage: 'in-progress', blockedBy: [] },
        { name: 'add-b', wave: 1, stage: 'in-progress', blockedBy: [] },
        { name: 'add-c', wave: 1, stage: 'ready', blockedBy: [] },
      ],
    },
    { discussions: [] },
    { 'add-a': 2 },
  );
  assert.deepEqual(board.stages['in-progress'].map((r) => r.codeRemaining), [2, null]);
  assert.equal(board.stages.ready[0].codeRemaining, 0);
});

test('changeSkills：提案中且沒被擋，建議 apply', () => {
  const skills = changeSkills(row({ stage: 'proposed' }));
  assert.deepEqual(names(skills), ['apply', 'analyze', 'ingest', 'drift', 'commit']);
  assert.deepEqual(primary(skills), ['apply']);
});

test('changeSkills：被擋時 apply 要等前一個變更，也不算建議', () => {
  const skills = changeSkills(row({ stage: 'proposed', blockedBy: ['add-z'] }));
  assert.deepEqual(skills.find((s) => s.skill === 'apply').waitingFor, ['add-z']);
  assert.deepEqual(primary(skills), []);
});

test('changeSkills：進行中建議 apply', () => {
  const skills = changeSkills(row());
  assert.deepEqual(names(skills), ['apply', 'ingest', 'drift', 'verify', 'quality', 'review', 'analyze', 'commit']);
  assert.deepEqual(primary(skills), ['apply']);
});

test('changeSkills：程式任務都做完、只剩手動任務時，建議 quality', () => {
  assert.deepEqual(primary(changeSkills(row({ codeRemaining: 0 }))), ['quality']);
  assert.deepEqual(primary(changeSkills(row({ codeRemaining: 1 }))), ['apply']);
});

test('changeSkills：有 worktree 時每個階段都能合併，也都不能封存', () => {
  const worktree = { path: TREE, branch: 'b' };
  for (const stage of ['proposed', 'in-progress', 'ready']) {
    const skills = names(changeSkills(row({ stage, worktree })));
    assert.ok(skills.includes('worktree-merge'), stage);
    assert.ok(!skills.includes('archive'), stage);
  }
  assert.ok(!names(changeSkills(row({ stage: 'ready' }))).includes('worktree-merge'), '沒有 worktree 就不能合併');
});

test('skillTargets：worktree-merge 列出有 worktree 的變更；quality、review 也列出進行中的變更', () => {
  const board = {
    stages: {
      proposed: [row({ name: 'add-new', stage: 'proposed' })],
      'in-progress': [row({ name: 'add-half', codeRemaining: 1 }), row({ name: 'add-tree', worktree: { path: TREE, branch: 'b' } })],
      ready: [row({ name: 'add-done', stage: 'ready' })],
    },
    discussions: [],
  };
  const targetNames = (skill) => skillTargets(skill, board).map((t) => t.name);
  assert.deepEqual(targetNames('worktree-merge'), ['add-tree']);
  assert.deepEqual(targetNames('quality'), ['add-half', 'add-tree', 'add-done']);
  assert.deepEqual(targetNames('review'), ['add-half', 'add-tree', 'add-done']);
});

test('quality、review：程式任務沒做完時不能開也不能放，手動任務不算', () => {
  const left = { kind: 'change', name: 'add-a', args: 'add-a', row: row({ codeRemaining: 2 }) };
  const done = { kind: 'change', name: 'add-a', args: 'add-a', row: row({ codeRemaining: 0 }) };
  const here = { address: 't1', tab: 'tb1' };
  const agents = [agent({ tab: 'tb1', address: 't1', key: 'claude', status: 'idle' })];
  for (const skill of ['quality', 'review']) {
    assert.equal(newTabPlan({ ...TAB, skill, target: left }).reason, '還有 2 個程式任務沒做完');
    assert.equal(newTabPlan({ ...TAB, skill, target: done }).kind, 'tab');
    assert.equal(placePlan({ skill, target: left, note: '', agents, focus: { tab: here.tab, terminal: here.address } }).reason, '還有 2 個程式任務沒做完');
    assert.equal(placePlan({ skill, target: done, note: '', agents, focus: { tab: here.tab, terminal: here.address } }).kind, 'place');
  }
  assert.equal(newTabPlan({ ...TAB, skill: 'verify', target: left }).kind, 'tab', 'verify 可以中途跑');
  assert.equal(newTabPlan({ ...TAB, skill: 'worktree-merge', target: left }).kind, 'tab', '合併不看任務');
});

test('開工：只剩手動任務時沒有開工鈕，因為沒有程式要寫', () => {
  assert.equal(startChoices(row({ codeRemaining: 0 }), ctx({ worktreePolicy: true })), null);
  assert.ok(startChoices(row({ codeRemaining: 1 }), ctx({ worktreePolicy: true })));
});

test('changeSkills：已就緒且沒有 worktree 可封存，建議 quality', () => {
  const skills = changeSkills(row({ stage: 'ready' }));
  assert.deepEqual(names(skills), ['quality', 'review', 'verify', 'archive', 'ingest', 'drift', 'commit']);
  assert.deepEqual(primary(skills), ['quality']);
});

test('changeSkills：已就緒且有 worktree 要先合併，不能封存', () => {
  const skills = changeSkills(row({ stage: 'ready', worktree: { path: TREE, branch: 'b' } }));
  assert.deepEqual(names(skills), ['quality', 'review', 'verify', 'ingest', 'drift', 'commit', 'worktree-merge']);
});

test('changeSkills：沒有階段限制的技能每個階段都有', () => {
  for (const stage of ['proposed', 'in-progress', 'ready']) {
    const skills = names(changeSkills(row({ stage })));
    for (const skill of ['ingest', 'drift', 'commit']) assert.ok(skills.includes(skill), `${stage} 要有 ${skill}`);
  }
});

test('commandLabels：只認會動到變更或討論的 speclink 指令，名稱的引號要拿掉', () => {
  const change = (name) => [`speclink.change=${name}`, 'speclink.discussion='];
  assert.deepEqual(commandLabels('speclink instructions apply --change "add-a" --json'), change('add-a'));
  assert.deepEqual(commandLabels('cd /x && speclink instructions apply --change add-a --json'), change('add-a'));
  assert.deepEqual(commandLabels('SPECLINK_WORKTREE=true speclink in-progress add add-a'), change('add-a'));
  assert.deepEqual(commandLabels('speclink new change "add-a" --schema spec-driven --agent claude --from-discussion talk'), change('add-a'));
  assert.deepEqual(commandLabels('speclink task done --change add-a tsk_1'), change('add-a'));
  assert.deepEqual(commandLabels('speclink review prepare "add-a" --base abc'), change('add-a'));
  assert.deepEqual(commandLabels('speclink verify prepare add-a'), change('add-a'));
  assert.deepEqual(commandLabels('speclink archive add-a -y'), change('add-a'));
  const talk = (slug) => ['speclink.change=', `speclink.discussion=${slug}`];
  assert.deepEqual(commandLabels('speclink discuss new "要不要改" --slug talk-a --kind x'), talk('talk-a'));
  assert.deepEqual(commandLabels("speclink discuss add-round talk-a --stdin <<'EOF'"), talk('talk-a'));
  assert.deepEqual(commandLabels('speclink discuss conclude talk-a --stdin'), talk('talk-a'));
});

test('commandLabels：查詢、提到和參數不是名稱時都不算', () => {
  for (const command of [
    'speclink show add-a',
    'speclink status --change add-a --json',
    'speclink artifact cat tasks --change add-a',
    'speclink plan --json',
    'speclink discuss show talk-a --json',
    'speclink review prepare --base abc',
    'git commit -m "speclink archive add-a"',
    'echo speclink archive add-a',
    "cat <<'EOF'\nspeclink archive add-a\nEOF",
    'grep speclink README.md',
    '',
  ]) {
    assert.equal(commandLabels(command), null, command);
  }
});

test('hookLabels：依事件決定看哪個欄位', () => {
  const apply = ['speclink.skill=speclink-apply', 'speclink.change=add-a', 'speclink.discussion='];
  assert.deepEqual(hookLabels({ hook_event_name: 'UserPromptSubmit', prompt: '/speclink-apply add-a' }), apply);
  assert.deepEqual(hookLabels({ hook_event_name: 'PostToolUse', tool_name: 'Skill', tool_input: { skill: 'speclink-apply', args: 'add-a' } }), apply);
  assert.deepEqual(hookLabels({ hook_event_name: 'PostToolUse', tool_name: 'Skill', tool_input: { skill: 'speclink-quality' } }), ['speclink.skill=speclink-quality', 'speclink.discussion=']);
  assert.equal(hookLabels({ hook_event_name: 'PostToolUse', tool_name: 'Skill', tool_input: { skill: 'frontend-design' } }), null, '別的技能不算');
  assert.deepEqual(hookLabels({ hook_event_name: 'PostToolUse', tool_name: 'Bash', tool_input: { command: 'speclink archive add-a' } }), ['speclink.change=add-a', 'speclink.discussion=']);
  assert.equal(hookLabels({ hook_event_name: 'PostToolUse', tool_name: 'Read', tool_input: { file_path: 'x' } }), null);
  const cleared = ['speclink.change=', 'speclink.discussion=', 'speclink.skill='];
  assert.deepEqual(hookLabels({ hook_event_name: 'SessionStart', source: 'clear' }), cleared);
  assert.deepEqual(hookLabels({ hook_event_name: 'SessionStart', source: 'resume' }), cleared);
  assert.equal(hookLabels({ hook_event_name: 'SessionStart', source: 'startup' }), null, 'hc run 開的 agent 一開始就有標籤，不能清');
  assert.equal(hookLabels({ hook_event_name: 'SessionStart', source: 'compact' }), null, '壓縮不換對話');
});

test('changeSkills：討論重新定案時建議改成 ingest', () => {
  assert.deepEqual(primary(changeSkills(row({ restale: true }))), ['ingest']);
  const ready = changeSkills(row({ stage: 'ready', restale: true }));
  assert.deepEqual(primary(ready), ['ingest']);
  assert.ok(names(ready).includes('ingest'));
});

test('discussionSkills：依討論狀態決定建議', () => {
  const d = (over) => ({ slug: 's', topic: 't', status: 'open', cuts: [], ...over });
  assert.deepEqual(discussionSkills(d()), [{ skill: 'discuss', args: 's', primary: true }]);
  assert.deepEqual(discussionSkills(d({ status: 'concluded' })), [
    { skill: 'propose', args: '--from-discussion s', primary: true },
    { skill: 'discuss', args: 's', primary: false },
  ]);
  assert.deepEqual(names(discussionSkills(d({ status: 'promoted', cuts: ['add-c'] }))), ['propose', 'discuss']);
  assert.deepEqual(discussionSkills(d({ status: 'promoted' })), [{ skill: 'discuss', args: 's', primary: false }]);
});

test('speclink 寫出的每個技能都有放處，表上也沒有已不存在的技能', () => {
  const written = readdirSync(path.join(ROOT, '.claude/skills'))
    .filter((name) => name.startsWith('speclink-'))
    .map((name) => name.slice('speclink-'.length))
    .sort();
  assert.deepEqual([...ALL_SKILLS].sort(), written);
});

// ---- 指令文字 ----

test('commandText：Claude Code、Copilot 用斜線，Codex 用錢字號', () => {
  assert.equal(commandText('claude', 'apply', 'add-a'), '/speclink-apply add-a');
  assert.equal(commandText('copilot', 'apply', 'add-a'), '/speclink-apply add-a');
  assert.equal(commandText('codex', 'apply', 'add-a'), '$speclink-apply add-a');
  assert.equal(commandText('claude', 'improve'), '/speclink-improve');
});

test('sendArgs：idle 用 --if idle；done 不帶旗標（--if idle 會拒絕 done）；其餘不送', () => {
  assert.deepEqual(sendArgs(agent({ status: 'idle' }), 'x'), ['send', '--if', 'idle', 't1', 'x']);
  assert.deepEqual(sendArgs(agent({ status: 'done' }), 'x'), ['send', 't1', 'x']);
  for (const status of ['working', 'blocked', 'error']) {
    assert.equal(sendArgs(agent({ status }), 'x'), null);
  }
});

// ---- 誰在做這個變更 ----

test('inside：同資料夾與子資料夾算在內，同前綴的兄弟資料夾不算', () => {
  assert.equal(inside(REPO, REPO), true);
  assert.equal(inside(`${REPO}/crates`, REPO), true);
  assert.equal(inside(TREE, REPO), false);
  assert.equal(inside(null, REPO), false);
});

test('changeAgents：依標籤、worktree 路徑、外掛回報的焦點的順序，同一個 agent 不重複', () => {
  const byFocus = agent({ address: 't3', labels: { 'speclink.focus': 'add-a' } });
  const byPath = agent({ address: 't2', cwd: TREE });
  const byLabel = agent({ address: 't1', labels: { 'speclink.change': 'add-a' } });
  const other = agent({ address: 't4', labels: { 'speclink.focus': 'add-z' } });
  const found = changeAgents(row({ worktree: { path: TREE, branch: 'b' } }), [byFocus, other, byPath, byLabel]);
  assert.deepEqual(
    found.map((a) => a.address),
    ['t1', 't2', 't3'],
  );
});

test('changeAgents：不看終端機標題，標題可能過時或只是提到變更', () => {
  const change = row({ name: 'add-a' });
  const titled = agent({ address: 't1', title: 'apply · add-a' });
  const none = agent({ address: 't2', title: 'apply · add-a', labels: { 'speclink.focus': '-' } });
  assert.deepEqual(changeAgents(change, [titled, none]), []);
});

test('skillLabels：做變更的技能記下變更，討論的技能記下討論，用不到的標籤要清掉', () => {
  assert.deepEqual(skillLabels('apply', 'add-a'), ['speclink.skill=speclink-apply', 'speclink.change=add-a', 'speclink.discussion=']);
  assert.deepEqual(skillLabels('quality'), ['speclink.skill=speclink-quality', 'speclink.discussion='], '沒指定就不動變更');
  assert.deepEqual(skillLabels('discuss', 'talk'), ['speclink.skill=speclink-discuss', 'speclink.change=', 'speclink.discussion=talk']);
  assert.deepEqual(skillLabels('propose'), ['speclink.skill=speclink-propose', 'speclink.change=', 'speclink.discussion=']);
  assert.deepEqual(skillLabels('improve', 'x'), ['speclink.skill=speclink-improve', 'speclink.change=', 'speclink.discussion=']);
});

test('promptLabels：只認開頭的 speclink 技能指令，名稱只看第一個參數', () => {
  const change = (skill, name) => [`speclink.skill=speclink-${skill}`, ...(name ? [`speclink.change=${name}`] : []), 'speclink.discussion='];
  assert.deepEqual(promptLabels('/speclink-apply add-a'), change('apply', 'add-a'));
  assert.deepEqual(promptLabels('  /speclink-ingest add-a 補充說明'), change('ingest', 'add-a'));
  assert.deepEqual(promptLabels('/speclink-quality'), change('quality'));
  assert.deepEqual(promptLabels('/speclink-apply 請接著做'), change('apply'), '中文不是變更名稱');
  assert.deepEqual(promptLabels('/speclink-apply --flag'), change('apply'));
  assert.equal(promptLabels('請跑 /speclink-apply add-a'), null, '句子裡提到不算');
  assert.equal(promptLabels('/speclink-unknown add-a'), null);
  assert.equal(promptLabels('/herdr-dispatch add-a'), null);
  assert.equal(promptLabels(''), null);
});

test('promptLabels：討論記下 slug，提案接續討論也算，slug 可以有中文', () => {
  const talk = (skill, slug) => [`speclink.skill=speclink-${skill}`, 'speclink.change=', `speclink.discussion=${slug}`];
  assert.deepEqual(promptLabels('/speclink-discuss agent-tool-sdk-layering'), talk('discuss', 'agent-tool-sdk-layering'));
  assert.deepEqual(promptLabels('/speclink-propose --from-discussion talk'), talk('propose', 'talk'));
  assert.deepEqual(promptLabels('/speclink-discuss 提示詞分開'), talk('discuss', '提示詞分開'));
  assert.deepEqual(promptLabels('/speclink-discuss'), talk('discuss', ''));
});

test('討論：標籤認得出這場討論的 agent，技能優先送給它，它在做事就等', () => {
  const d = { slug: 'talk', status: 'open', cuts: [] };
  const here = agent({ address: 't1', tab: 'tab1', cwd: REPO, status: 'idle' });
  const mine = agent({ address: 't2', tab: 'tab2', cwd: REPO, status: 'idle', labels: { 'speclink.discussion': 'talk' } });
  const other = agent({ address: 't3', labels: { 'speclink.discussion': 'other' } });
  assert.deepEqual(discussionAgents(d, [here, mine, other]).map((a) => a.address), ['t2']);
  const context = ctx({ agents: [here, mine], focus: { tab: 'tab1', terminal: 't1' } });
  const [sent] = discussionActions(d, context);
  assert.equal(sent.delivery.agent.address, 't2', '不是送給目前分頁');
  assert.equal(deliveryHint({ kind: 'send', where: sent.delivery.where }), '送給這場討論的 agent');
  assert.deepEqual(sent.delivery.labels, ['speclink.skill=speclink-discuss', 'speclink.change=', 'speclink.discussion=talk']);
  const [busy] = discussionActions(d, ctx({ agents: [{ ...mine, status: 'working' }], focus: { tab: 'tab1', terminal: 't1' } }));
  assert.equal(busy.delivery.kind, 'busy');
  assert.equal(deliveryHint(busy.delivery), '這場討論的 agent 正在做事');
});

test('討論：沒有 agent 時開新分頁，分頁名稱和標籤都寫出討論', () => {
  const d = { slug: 'talk', status: 'open', cuts: [] };
  const [fresh] = discussionActions(d, ctx({ agents: [] }));
  assert.equal(fresh.delivery.kind, 'new');
  assert.equal(fresh.delivery.tabName, 'discuss · talk');
  assert.deepEqual(fresh.delivery.labels, ['speclink.skill=speclink-discuss', 'speclink.change=', 'speclink.discussion=talk']);
  const plan = newTabPlan({ ...TAB, skill: 'discuss', target: { kind: 'discussion', name: 'talk', args: 'talk' } });
  assert.equal(plan.tabName, 'discuss · talk');
  assert.ok(plan.hc.includes('speclink.discussion=talk'));
  assert.ok(plan.hc.includes('speclink.skill=speclink-discuss'));
});

test('送達：送給現有 agent 時帶上標籤，apply 改走 worktree 時用實際的技能', () => {
  const here = agent({ address: 't1', tab: 'tab1', cwd: REPO, status: 'idle' });
  const d = planDelivery({ skill: 'quality', change: row(), args: 'add-a', ...ctx({ agents: [here], focus: { tab: 'tab1', terminal: 't1' } }) });
  assert.equal(d.kind, 'send');
  assert.deepEqual(d.labels, ['speclink.skill=speclink-quality', 'speclink.change=add-a', 'speclink.discussion=']);
  const moved = planDelivery({ skill: 'apply', change: row({ worktree: { path: TREE, branch: 'b' } }), args: 'add-a', ...ctx({ agents: [here], focus: { tab: 'tab1', terminal: 't1' } }) });
  assert.deepEqual(moved.labels, ['speclink.skill=speclink-apply-with-worktree', 'speclink.change=add-a', 'speclink.discussion=']);
});

test('放到輸入框：技能指令帶標籤，附加名稱不帶', () => {
  const agents = [agent({ tab: 'tb1', address: 't1', key: 'claude', status: 'idle' })];
  const focus = { tab: 'tb1', terminal: 't1' };
  const target = { kind: 'change', name: 'add-a', args: 'add-a', row: row() };
  assert.deepEqual(placePlan({ skill: 'apply', target, note: '', agents, focus }).labels, ['speclink.skill=speclink-apply', 'speclink.change=add-a', 'speclink.discussion=']);
  assert.deepEqual(placePlan({ skill: 'discuss', target: { kind: 'discussion', name: 'talk', args: 'talk' }, note: '', agents, focus }).labels, ['speclink.skill=speclink-discuss', 'speclink.change=', 'speclink.discussion=talk']);
  assert.deepEqual(placePlan({ skill: 'discuss', target: null, note: '主題', agents, focus }).labels, ['speclink.skill=speclink-discuss', 'speclink.change=', 'speclink.discussion=']);
  assert.equal(attachPlan({ name: 'add-a', agents, focus }).labels, undefined);
});

test('changeAgents：開工時貼了標籤的 agent 專做那個變更，焦點提到別的變更不算', () => {
  const change = row({ name: 'add-a' });
  const owned = agent({ address: 't1', labels: { 'speclink.change': 'add-b', 'speclink.focus': 'add-a' } });
  assert.deepEqual(changeAgents(change, [owned]), []);
});

test('送達：這個變更的 agent 在做事時，不會把技能送給只是提到它的討論分頁（實際發生過）', () => {
  const change = row({ name: 'add-a' });
  const apply = agent({ address: 't1', status: 'working', labels: { 'speclink.change': 'add-a' } });
  const talk = agent({ address: 't2', status: 'done', title: 'propose · add-a', labels: { 'speclink.skill': 'speclink-discuss' } });
  const d = planDelivery({ skill: 'apply', change, args: 'add-a', ...ctx({ agents: [apply, talk] }) });
  assert.equal(d.kind, 'busy');
  assert.equal(d.agent.address, 't1');
});

test('joinAgents：狀態用 core 的原始字（done 不會被 Herdr 改寫成 idle）', () => {
  const [a] = joinAgents(
    [{ address: 't1', status: 'idle', program: { key: 'claude', name: 'Claude Code' }, title: 'x', pane: 'p1', tab: 'tab1', tab_name: '1', space_name: 's' }],
    [{ address: 't1', status: 'done', labels: { a: 'b' } }],
    [{ pane: 'p1', terminal: 't1', cwd: REPO }],
  );
  assert.equal(a.status, 'done');
  assert.equal(a.view, 'idle');
  assert.equal(a.cwd, REPO);
  assert.equal(a.key, 'claude');
  assert.deepEqual(a.labels, { a: 'b' });
});

// ---- 送達 ----

const deliver = (over) => planDelivery({ skill: 'apply', change: row(), args: 'add-a', ...ctx(over) });

test('送達第一順位：正在做這個變更的 agent', () => {
  const mine = agent({ address: 't9', tab: 'other', labels: { 'speclink.change': 'add-a' } });
  const current = agent({ address: 't1' });
  const d = deliver({ agents: [current, mine], focus: { tab: 'tab1', terminal: 't1' } });
  assert.equal(d.kind, 'send');
  assert.equal(d.where, 'change');
  assert.equal(d.agent.address, 't9');
  assert.equal(d.text, '/speclink-apply add-a');
  assert.deepEqual(d.hc, ['send', 't9', '/speclink-apply add-a']);
});

test('送達第二順位：目前分頁的 agent，焦點所在的優先', () => {
  const left = agent({ address: 't1' });
  const right = agent({ address: 't2' });
  const d = deliver({ agents: [left, right], focus: { tab: 'tab1', terminal: 't2' } });
  assert.equal(d.where, 'tab');
  assert.equal(d.agent.address, 't2');
});

test('送達第三順位：都沒有就開新分頁，開在該去的資料夾', () => {
  const d = deliver({ agents: [agent({ tab: 'other' })] });
  assert.deepEqual(d, {
    kind: 'new',
    where: 'new',
    key: 'claude',
    folder: REPO,
    argv: ['claude'],
    text: '/speclink-apply add-a',
    tabName: 'apply · add-a',
    change: 'add-a',
    skill: 'apply',
    labels: ['speclink.skill=speclink-apply', 'speclink.change=add-a', 'speclink.discussion='],
  });
});

test('新分頁選最常用且已安裝的 agent；Codex 要帶 --no-daemon', () => {
  const known = [
    { key: 'pi', run: ['pi'], installed: true },
    { key: 'codex', run: ['codex'], installed: true },
    { key: 'claude', run: ['claude'], installed: true },
  ];
  const d = deliver({ known });
  assert.deepEqual(d.argv, ['codex', '--no-daemon']);
  assert.equal(d.text, '$speclink-apply add-a');
});

test('沒有可用的 agent 時不開分頁並說明原因', () => {
  assert.equal(deliver({ known: [{ key: 'pi', run: ['pi'], installed: true }] }).kind, 'none');
  assert.equal(deliver({ known: null }).kind, 'none');
});

test('agent 忙碌時不送，也不改送別人，並寫出原因', () => {
  const reasons = { working: '正在做事', blocked: '在等你回答', error: '出錯了', sending: '正在收指令' };
  for (const [status, reason] of Object.entries(reasons)) {
    const mine = agent({ address: 't9', status, labels: { 'speclink.change': 'add-a' } });
    const d = deliver({ agents: [mine, agent({ address: 't1' })] });
    assert.equal(d.kind, 'busy');
    assert.equal(d.reason, reason);
    assert.equal(d.agent.address, 't9');
  }
});

test('同一個變更有多個 agent 時，先送給有空的', () => {
  const busy = agent({ address: 't8', status: 'working', labels: { 'speclink.change': 'add-a' } });
  const free = agent({ address: 't9', labels: { 'speclink.change': 'add-a' } });
  assert.equal(deliver({ agents: [busy, free] }).agent.address, 't9');
});

test('沒有 speclink 技能的 agent 不當送達對象', () => {
  const d = deliver({ agents: [agent({ key: 'pi' })] });
  assert.equal(d.kind, 'new');
});

test('資料夾檢查：目前分頁的 agent 在別的 repo 就跳過', () => {
  const d = deliver({ agents: [agent({ cwd: '/work/other' })] });
  assert.equal(d.kind, 'new');
  assert.equal(d.folder, REPO);
});

const inTree = row({ worktree: { path: TREE, branch: 'speclink/add-a' } });
const treeCase = (over, skill = 'quality') => planDelivery({ skill, change: inTree, args: 'add-a', ...ctx(over) });

test('有 worktree 的變更：技能要在 worktree 裡跑，主資料夾的 agent 不算', () => {
  const d = treeCase({ agents: [agent()] });
  assert.equal(d.kind, 'new');
  assert.equal(d.folder, TREE);
});

test('有 worktree 的變更：worktree 裡的 agent 優先於標籤標在主資料夾的 agent', () => {
  const labelled = agent({ address: 't1', labels: { 'speclink.change': 'add-a' } });
  const there = agent({ address: 't2', cwd: TREE });
  assert.equal(treeCase({ agents: [labelled, there] }).agent.address, 't2');
});

test('有 worktree 的變更：標籤標明的 agent 即使資料夾還在主資料夾也算（agent 用 cd 進 worktree）', () => {
  const labeled = agent({ labels: { 'speclink.change': 'add-a' } });
  assert.equal(treeCase({ agents: [labeled] }).where, 'change');
});

test('worktree-merge 要在主資料夾跑，不送進 worktree', () => {
  const d = treeCase({ agents: [agent({ address: 't2', cwd: TREE })] }, 'worktree-merge');
  assert.equal(d.kind, 'new');
  assert.equal(d.folder, REPO);
});

test('apply：worktree 裡有 agent 就送 apply', () => {
  const d = treeCase({ agents: [agent({ cwd: TREE })] }, 'apply');
  assert.equal(d.text, '/speclink-apply add-a');
});

test('apply：worktree 裡沒有 agent，就從主資料夾用 apply-with-worktree 接續', () => {
  const current = treeCase({ agents: [agent()] }, 'apply');
  assert.equal(current.where, 'tab');
  assert.equal(current.text, '/speclink-apply-with-worktree add-a');
  const fresh = treeCase({}, 'apply');
  assert.equal(fresh.kind, 'new');
  assert.equal(fresh.folder, REPO);
  assert.equal(fresh.text, '/speclink-apply-with-worktree add-a');
  assert.equal(fresh.skill, 'apply-with-worktree');
});

test('apply：標籤標明但資料夾在主資料夾的 agent，用 apply-with-worktree 接續', () => {
  const labelled = agent({ labels: { 'speclink.change': 'add-a' } });
  const d = treeCase({ agents: [labelled] }, 'apply');
  assert.equal(d.where, 'change');
  assert.equal(d.text, '/speclink-apply-with-worktree add-a');
});

test('不帶變更的技能：送給目前分頁，沒有就開新分頁', () => {
  const sent = planDelivery({ skill: 'discuss', change: null, args: '要討論的主題', ...ctx({ agents: [agent()] }) });
  assert.equal(sent.text, '/speclink-discuss 要討論的主題');
  const fresh = planDelivery({ skill: 'improve', change: null, ...ctx() });
  assert.equal(fresh.kind, 'new');
  assert.equal(fresh.tabName, 'improve');
  assert.equal(fresh.change, null);
});

// ---- 列上的按鈕 ----

test('changeActions：被擋的 apply 不算送達，analyze 只在 Claude Code 出現', () => {
  const blocked = changeActions(row({ stage: 'proposed', blockedBy: ['add-z'] }), ctx());
  assert.equal(blocked.find((a) => a.skill === 'apply').delivery, null);
  const codex = changeActions(row({ stage: 'proposed' }), ctx({ agents: [agent({ key: 'codex' })] }));
  assert.deepEqual(names(codex), ['apply', 'ingest', 'drift', 'commit']);
  const claude = changeActions(row({ stage: 'proposed' }), ctx({ agents: [agent()] }));
  assert.deepEqual(names(claude), ['apply', 'analyze', 'ingest', 'drift', 'commit']);
});

test('discussionActions：帶討論 slug 送達', () => {
  const d = { slug: 'talk', topic: '主題', status: 'concluded', cuts: [] };
  const [propose] = discussionActions(d, ctx({ agents: [agent()] }));
  assert.equal(propose.delivery.text, '/speclink-propose --from-discussion talk');
});

test('deliveryHint：說明會送去哪裡，或為什麼不能送', () => {
  assert.equal(deliveryHint(deliver({ agents: [agent({ labels: { 'speclink.change': 'add-a' } })] })), '送給這個變更的 agent');
  assert.equal(deliveryHint(deliver({ agents: [agent()] })), '送給目前分頁的 agent');
  assert.equal(deliveryHint(deliver({})), '開新分頁');
  assert.equal(deliveryHint(deliver({ agents: [agent({ status: 'blocked' })] })), '目前分頁的 agent 在等你回答');
  assert.equal(
    deliveryHint(deliver({ agents: [agent({ status: 'working', labels: { 'speclink.change': 'add-a' } })] })),
    '這個變更的 agent 正在做事',
  );
  assert.equal(deliveryHint(deliver({ known: null })), '還在確認可用的 agent');
});

// ---- 技能跳窗 ----

test('技能表：前四組依工作流程，都有東西可以搭配；專案組不搭配', () => {
  assert.deepEqual([...new Set(PICKER_SKILLS.map((w) => w.group))], ['規劃', '實作', '品質', '收尾', '專案']);
  assert.ok(PICKER_SKILLS.every((w) => w.about));
  assert.ok(PICKER_SKILLS.every((w) => (w.group === '專案' ? w.pair === null : w.pair)));
  assert.deepEqual(
    PICKER_SKILLS.filter((w) => w.pair === '討論').map((w) => w.skill),
    ['discuss', 'propose'],
  );
});

const board = () =>
  buildBoard(
    {
      changes: [
        { name: 'add-p', completedTasks: 0, totalTasks: 2 },
        { name: 'add-i', completedTasks: 1, totalTasks: 2 },
        { name: 'add-r', completedTasks: 2, totalTasks: 2 },
        { name: 'add-w', completedTasks: 2, totalTasks: 2, worktree: { path: TREE } },
      ],
    },
    {
      changes: [
        { name: 'add-p', stage: 'proposed', wave: 1, blockedBy: [] },
        { name: 'add-i', stage: 'in-progress', wave: 1, blockedBy: [] },
        { name: 'add-r', stage: 'ready', wave: 1, blockedBy: [] },
        { name: 'add-w', stage: 'ready', wave: 1, blockedBy: [] },
      ],
    },
    {
      discussions: [
        { slug: 'talk-open', topic: '還在談', status: 'open' },
        { slug: 'talk-done', topic: '談完了', status: 'concluded' },
        { slug: 'talk-old', topic: '已轉出', status: 'promoted' },
      ],
    },
  );
const targetNames = (skill) => skillTargets(skill, board()).map((t) => t.name);

test('skillTargets：只列這個技能能搭配的變更，規則和面板的技能鈕相同', () => {
  assert.deepEqual(targetNames('apply'), ['add-p', 'add-i']);
  assert.deepEqual(targetNames('apply-with-worktree'), ['add-p', 'add-i']);
  assert.deepEqual(targetNames('archive'), ['add-r']);
  assert.deepEqual(targetNames('worktree-merge'), ['add-w']);
  assert.deepEqual(targetNames('verify'), ['add-i', 'add-r', 'add-w']);
});

test('skillTargets：討論帶對應的參數，已轉出完的不列', () => {
  assert.deepEqual(
    skillTargets('discuss', board()).map((t) => t.args),
    ['talk-open', 'talk-done'],
  );
  assert.deepEqual(
    skillTargets('propose', board()).map((t) => t.args),
    ['--from-discussion talk-done'],
  );
});

test('skillTargets：不搭配東西的技能沒有選項；表上的 pair 和實際列出的種類一致', () => {
  assert.deepEqual(targetNames('improve'), []);
  for (const w of PICKER_SKILLS) {
    const kinds = new Set(skillTargets(w.skill, board()).map((t) => t.kind));
    if (w.pair === null) assert.equal(kinds.size, 0, w.skill);
    if (w.pair === '變更') assert.ok(!kinds.has('discussion'), w.skill);
    if (w.pair === '討論') assert.ok(!kinds.has('change'), w.skill);
  }
});

const place = (over) => placePlan({ skill: 'apply', target: null, agents: [agent()], focus: { tab: 'tab1', terminal: 't1' }, ...over });

test('placePlan：放進目前分頁的 agent，結尾留空白好接著打字', () => {
  const plan = place({ skill: 'improve' });
  assert.equal(plan.kind, 'place');
  assert.equal(plan.agent.address, 't1');
  assert.equal(plan.text, '/speclink-improve ');
});

test('placePlan：什麼字都貼得進去，Codex 的錢字號和中文都可以', () => {
  assert.equal(place({ agents: [agent({ key: 'codex' })] }).text, '$speclink-apply ');
  assert.equal(place({ skill: 'discuss', note: '要談的主題', agents: [agent({ key: 'copilot' })] }).text, '/speclink-discuss 要談的主題 ');
  assert.equal(place({ skill: 'discuss', target: { args: '中文討論' } }).text, '/speclink-discuss 中文討論 ');
});

test('placePlan：控制字元換成空白，貼上時換行才不會變成送出', () => {
  assert.equal(place({ skill: 'discuss', note: 'a\nb\u001b[201~c' }).text, '/speclink-discuss a b [201~c ');
});

test('placePlan：帶搭配的變更或討論', () => {
  assert.equal(place({ target: { args: 'add-a' } }).text, '/speclink-apply add-a ');
  assert.equal(place({ skill: 'propose', target: { args: '--from-discussion s' } }).text, '/speclink-propose --from-discussion s ');
});

test('placePlan：焦點所在的 agent 優先，焦點不在 agent 時用同分頁的第一個', () => {
  const two = [agent({ address: 't1' }), agent({ address: 't2' })];
  assert.equal(place({ agents: two, focus: { tab: 'tab1', terminal: 't2' } }).agent.address, 't2');
  assert.equal(place({ agents: two, focus: { tab: 'tab1', terminal: 't9' } }).agent.address, 't1');
});

test('placePlan：agent 在做事也可以放，因為不會送出', () => {
  assert.equal(place({ agents: [agent({ status: 'working' })] }).kind, 'place');
});

test('placePlan：這些情況不放，並寫出原因', () => {
  assert.match(place({ agents: [] }).reason, /目前分頁沒有 agent/);
  assert.match(place({ agents: [agent({ tab: 'other' })] }).reason, /目前分頁沒有 agent/);
  assert.match(place({ agents: [agent({ key: 'shell' })] }).reason, /目前分頁沒有 agent/);
  assert.match(place({ agents: [agent({ status: 'blocked' })] }).reason, /在等你回答/);
  assert.match(place({ skill: 'analyze', agents: [agent({ key: 'copilot' })] }).reason, /只有 Claude Code/);
});

test('placePlan：補充說明接在搭配的東西後面', () => {
  assert.equal(place({ skill: 'discuss', target: { args: 's' }, note: 'next' }).text, '/speclink-discuss s next ');
});

const attach = (over) => attachPlan({ name: 'add-a', agents: [agent()], focus: { tab: 'tab1', terminal: 't1' }, ...over });

test('attachPlan：名稱前後留空白，agent 在做事也可以附加，中文名稱也可以', () => {
  assert.equal(attach().text, ' add-a ');
  assert.equal(attach({ agents: [agent({ status: 'working' })] }).kind, 'place');
  assert.equal(attach({ agents: [agent({ key: 'codex' })] }).kind, 'place');
  assert.equal(attach({ name: '中文討論' }).text, ' 中文討論 ');
});

test('attachPlan：沒有 agent 或在等回答時不放', () => {
  assert.equal(attach({ agents: [] }).reason, '目前分頁沒有 agent');
  assert.equal(attach({ agents: [agent({ status: 'blocked' })] }).reason, '目前分頁的 agent 在等你回答');
});

const TAB = { skill: 'improve', target: null, note: '', root: REPO, agents: [], known: CLAUDE, worktreePolicy: true, space: 'sp1', on: null };
const tab = (over) =>
  newTabPlan({ ...TAB, ...over });
const changeTarget = (over = {}) => ({ kind: 'change', name: 'add-a', args: 'add-a', row: row(over) });

test('newTabPlan：和 herdr-dispatch 相同，用 hc run 開在目前的 space，不接在誰底下', () => {
  const plan = tab({ skill: 'review', target: changeTarget() });
  assert.equal(plan.kind, 'tab');
  assert.equal(plan.tabName, 'review · add-a');
  assert.deepEqual(plan.hc, [
    'run', '--bg', '--json', '--no-parent',
    '--cwd', REPO,
    '--label', 'herdr.space=sp1',
    '--name', 'add-a',
    '--label', 'speclink.change=add-a',
    '--label', 'speclink.skill=speclink-review',
    'claude', '/speclink-review add-a',
  ]);
});

test('newTabPlan：不帶變更時不取名也不貼變更標籤，補充說明接在指令後面', () => {
  const plan = tab({ skill: 'discuss', note: '要談的主題' });
  assert.equal(plan.tabName, 'discuss');
  assert.ok(!plan.hc.includes('--name'));
  assert.equal(plan.hc.at(-1), '/speclink-discuss 要談的主題');
  assert.equal(tab({ skill: 'discuss', target: { kind: 'discussion', args: 's' }, note: '接著談' }).hc.at(-1), '/speclink-discuss s 接著談');
});

test('newTabPlan：有 worktree 的變更開在 worktree；apply 從主資料夾接續；另一台機器帶 --on', () => {
  const worktree = { worktree: { path: TREE } };
  assert.equal(tab({ skill: 'review', target: changeTarget(worktree) }).folder, TREE);
  const apply = tab({ skill: 'apply', target: changeTarget(worktree) });
  assert.equal(apply.folder, REPO);
  assert.equal(apply.hc.at(-1), '/speclink-apply-with-worktree add-a');
  assert.deepEqual(tab({ on: 'vps' }).hc.slice(4, 6), ['--on', 'vps']);
});

test('newTabPlan：Codex 用錢字號並帶 --no-daemon', () => {
  const plan = tab({ known: [{ key: 'codex', run: ['codex'], installed: true }] });
  assert.deepEqual(plan.hc.slice(-3), ['codex', '--no-daemon', '$speclink-improve']);
});

test('newTabPlan：這些情況不開，並寫出原因', () => {
  const mine = agent({ labels: { 'speclink.change': 'add-a' } });
  assert.equal(tab({ skill: 'review', target: changeTarget(), agents: [mine] }).reason, '已經有 agent 在做');
  assert.match(tab({ skill: 'apply', target: changeTarget({ blockedBy: ['add-b'] }) }).reason, /要等 add-b 落地/);
  assert.match(tab({ skill: 'analyze', known: [{ key: 'copilot', run: ['copilot'], installed: true }] }).reason, /只有 Claude Code/);
  assert.equal(tab({ known: null }).reason, '還在確認可用的 agent');
});

test('newTabPlan：沒開 worktree 流程時 apply-with-worktree 不開，因為技能會自己停下來', () => {
  assert.equal(tab({ skill: 'apply-with-worktree', worktreePolicy: false }).reason, '這個專案沒開 worktree 流程');
  assert.equal(tab({ skill: 'apply', worktreePolicy: false }).kind, 'tab');
});

const batch = (over) => batchPlan({ ...TAB, skill: 'quality', targets: [], ...over });
const named = (name, over = {}) => ({ kind: 'change', name, args: name, row: row({ name, ...over }) });

test('batchPlan：一個變更開一個分頁，互不相干', () => {
  const plan = batch({ targets: [named('add-a'), named('add-b')] });
  assert.equal(plan.kind, 'tabs');
  assert.deepEqual(plan.plans.map((p) => p.hc.at(-1)), ['/speclink-quality add-a', '/speclink-quality add-b']);
  assert.deepEqual(plan.plans.map((p) => p.tabName), ['quality · add-a', 'quality · add-b']);
  assert.equal(plan.text, '/speclink-quality add-a　/speclink-quality add-b');
});

test('batchPlan：沒有勾或勾的都不能開時不做事', () => {
  assert.equal(batch().reason, '勾選要開工的變更');
  const mine = agent({ labels: { 'speclink.change': 'add-a' } });
  assert.equal(batch({ targets: [named('add-a')], agents: [mine] }).reason, '勾選要開工的變更');
});

test('batchPlan：不能開的變更略過，其他照開', () => {
  const mine = agent({ labels: { 'speclink.change': 'add-a' } });
  const plan = batch({ targets: [named('add-a'), named('add-b')], agents: [mine] });
  assert.deepEqual(plan.plans.map((p) => p.change), ['add-b']);
});

test('batchPlan：兩個 apply 都在主資料夾就擋下；有 worktree 的接續 worktree，不算', () => {
  const two = [named('add-a'), named('add-b')];
  assert.match(batch({ skill: 'apply', targets: two }).reason, /一次只能跑一個 apply/);
  const one = [named('add-a'), named('add-b', { worktree: { path: TREE } })];
  assert.deepEqual(batch({ skill: 'apply', targets: one }).plans.map((p) => p.skill), ['apply', 'apply-with-worktree']);
  assert.equal(batch({ skill: 'apply-with-worktree', targets: two }).plans.length, 2);
});

// ---- 叫 Speclink 開頁面 ----

test('speclinkUrl：與 desktop-app「以 speclink 網址開啟詳情頁」的格式相同', () => {
  assert.equal(
    speclinkUrl('change', '/Users/dev/speclink', 'desktop-deep-link'),
    'speclink://change/desktop-deep-link?project=%2FUsers%2Fdev%2Fspeclink',
  );
  assert.equal(
    speclinkUrl('discussion', '/Users/dev/speclink', 'herdr-speclink-panel'),
    'speclink://discussion/herdr-speclink-panel?project=%2FUsers%2Fdev%2Fspeclink',
  );
  assert.equal(speclinkUrl('roadmap', '/Users/dev/speclink'), 'speclink://roadmap?project=%2FUsers%2Fdev%2Fspeclink');
});

test('speclinkUrl：名稱與路徑要百分比編碼', () => {
  assert.equal(speclinkUrl('change', '/Users/我 的/app', 'add a'), 'speclink://change/add%20a?project=%2FUsers%2F%E6%88%91%20%E7%9A%84%2Fapp');
});

test('speclinkUrl：專案網址不帶名稱，路徑要百分比編碼', () => {
  assert.equal(speclinkUrl('project', '/Users/dev/speclink'), 'speclink://project?project=%2FUsers%2Fdev%2Fspeclink');
  assert.equal(speclinkUrl('project', '/Users/我 的/app'), 'speclink://project?project=%2FUsers%2F%E6%88%91%20%E7%9A%84%2Fapp');
});

test('canOpenProject：讀得到專案，而且專案在這台機器上，才能用網址開', () => {
  assert.equal(canOpenProject({ hasBoard: true, on: null }), true);
  assert.equal(canOpenProject({ hasBoard: false, on: null }), false);
  assert.equal(canOpenProject({ hasBoard: true, on: 'build-box' }), false);
});

const LOGO_ROOT = '/Users/dev/speclink';
const BRING_UP = ['-b', APP_ID];

test('logoOpenAttempts：本機已初始化的專案先送網址，送不出去再退回只叫出 app', () => {
  assert.deepEqual(logoOpenAttempts({ root: LOGO_ROOT, hasBoard: true, on: null }), [
    ['speclink://project?project=%2FUsers%2Fdev%2Fspeclink'],
    BRING_UP,
  ]);
});

test('logoOpenAttempts：沒初始化、還在載入或讀不到專案時只叫出 app', () => {
  assert.deepEqual(logoOpenAttempts({ root: LOGO_ROOT, hasBoard: false, on: null }), [BRING_UP]);
  assert.deepEqual(logoOpenAttempts({ root: null, hasBoard: false, on: null }), [BRING_UP]);
});

test('logoOpenAttempts：別台機器的 space 只叫出 app，路徑在這台開不了', () => {
  assert.deepEqual(logoOpenAttempts({ root: LOGO_ROOT, hasBoard: true, on: 'build-box' }), [BRING_UP]);
});

// 假的 exec：依序回傳給定的結果，Error 代表 exec 本身失敗（例如系統沒有 open 指令）。
function fakeExec(...results) {
  const calls = [];
  const exec = async (command, args) => {
    calls.push([command, args]);
    const result = results.shift();
    if (result instanceof Error) throw result;
    return result;
  };
  return { calls, exec };
}

const URL_ARGS = ['speclink://project?project=%2Fa'];
const NOT_REGISTERED = { code: 1, stderr: 'No application knows how to open URL speclink://project?project=%2Fa\nmore' };

test('lastOpenError：第一組成功就停，沒有錯誤', async () => {
  const { calls, exec } = fakeExec({ code: 0, stderr: '' });
  assert.equal(await lastOpenError([URL_ARGS, BRING_UP], exec), null);
  assert.deepEqual(calls, [['open', URL_ARGS]]);
});

test('lastOpenError：沒有註冊網址的舊版 app 退回下一組，成功就沒有錯誤', async () => {
  const { calls, exec } = fakeExec(NOT_REGISTERED, { code: 0, stderr: '' });
  assert.equal(await lastOpenError([URL_ARGS, BRING_UP], exec), null);
  assert.deepEqual(calls, [['open', URL_ARGS], ['open', BRING_UP]]);
});

test('lastOpenError：全部失敗回傳最後一次有內容的錯誤', async () => {
  const { exec } = fakeExec(NOT_REGISTERED, { code: 1, stderr: 'Unable to find application' });
  assert.equal(await lastOpenError([URL_ARGS, BRING_UP], exec), 'Unable to find application');
});

test('lastOpenError：後一次沒有錯誤訊息時，保留前一次的訊息', async () => {
  const { exec } = fakeExec(NOT_REGISTERED, { code: 1, stderr: '' });
  assert.equal(await lastOpenError([URL_ARGS, BRING_UP], exec), NOT_REGISTERED.stderr);
});

test('lastOpenError：exec 拋例外算失敗，錯誤是例外的訊息', async () => {
  const { exec } = fakeExec(new Error('open: command not found'));
  assert.equal(await lastOpenError([BRING_UP], exec), 'open: command not found');
});

// ---- 開工 ----

const start = (over = {}, ctxOver = {}) => startChoices(row(over), ctx({ worktreePolicy: true, space: 'sp1', ...ctxOver }));
const planOf = (choices, label) => choices.find((c) => c.label === label).plan;

test('開工：開 worktree 政策、還沒有 worktree 時要問，兩種都開在目前的工作區和主資料夾', () => {
  const choices = start({ stage: 'proposed', completed: 0 });
  assert.deepEqual(choices.map((c) => c.label), ['用 worktree', '直接 apply']);
  const tree = planOf(choices, '用 worktree');
  assert.equal(tree.kind, 'tab');
  assert.equal(tree.folder, REPO);
  assert.equal(tree.text, '/speclink-apply-with-worktree add-a');
  assert.equal(tree.tabName, 'apply-with-worktree · add-a');
  assert.equal(tree.change, 'add-a');
  assert.ok(tree.hc.includes('herdr.space=sp1'), '開在目前的工作區，不開新工作區');
  assert.ok(tree.hc.includes('speclink.change=add-a'));
  assert.equal(planOf(choices, '直接 apply').text, '/speclink-apply add-a');
  assert.equal(planOf(choices, '直接 apply').folder, REPO);
});

const applyAgent = (change, over = {}) =>
  agent({ labels: { 'speclink.change': change, 'speclink.skill': 'speclink-apply' }, ...over });
const boardWith = (...rows) => ({ stages: { proposed: rows.filter((r) => r.stage === 'proposed'), 'in-progress': rows.filter((r) => r.stage === 'in-progress'), ready: rows.filter((r) => r.stage === 'ready') } });

test('開工：主資料夾的 apply 還沒做完時，直接 apply 不能選並寫出是哪個變更，worktree 還可以', () => {
  const board = boardWith(row({ name: 'add-z', stage: 'in-progress' }));
  // 分頁閒置也算：做到一半停下來，檔案還在主資料夾。
  const choices = start({}, { board, agents: [applyAgent('add-z', { status: 'idle' })] });
  assert.equal(planOf(choices, '直接 apply').reason, 'add-z 的 apply 還沒做完');
  assert.equal(planOf(choices, '用 worktree').kind, 'tab');
});

test('開工：apply 跑完了分頁還開著，不算佔用主資料夾', () => {
  // 已經封存，不在清單裡；這就是分頁留著、標籤還在的情況。
  const archived = start({}, { board: boardWith(row({ name: 'add-a', stage: 'proposed' })), agents: [applyAgent('add-z', { status: 'done' })] });
  assert.equal(planOf(archived, '直接 apply').kind, 'tab');
  // 任務都做完，接著在同一個分頁跑 quality。
  const ready = start({}, { board: boardWith(row({ name: 'add-z', stage: 'ready' })), agents: [applyAgent('add-z', { status: 'working' })] });
  assert.equal(planOf(ready, '直接 apply').kind, 'tab');
});

test('newTabPlan：apply-with-worktree 不受主資料夾的 apply 影響', () => {
  const board = boardWith(row({ name: 'add-z', stage: 'in-progress' }));
  assert.equal(tab({ skill: 'apply', board, agents: [applyAgent('add-z')] }).reason, 'add-z 的 apply 還沒做完');
  assert.equal(tab({ skill: 'apply-with-worktree', board, agents: [applyAgent('add-z')] }).kind, 'tab');
});

test('開工：未開 worktree 政策時不問，直接 apply', () => {
  const choices = start({}, { worktreePolicy: false });
  assert.deepEqual(choices.map((c) => c.label), ['開工']);
  assert.equal(choices[0].plan.text, '/speclink-apply add-a');
  assert.equal(choices[0].plan.folder, REPO);
});

test('開工：已有 worktree 時不問，從主資料夾接續', () => {
  const choices = start({ worktree: { path: TREE, branch: 'speclink/add-a' } });
  assert.deepEqual(choices.map((c) => c.label), ['開工']);
  assert.equal(choices[0].plan.folder, REPO);
  assert.equal(choices[0].plan.text, '/speclink-apply-with-worktree add-a');
});

test('開工：Codex 用錢字號並帶 --no-daemon', () => {
  const plan = planOf(start({}, { known: [{ key: 'codex', run: ['codex'], installed: true }] }), '用 worktree');
  assert.equal(plan.text, '$speclink-apply-with-worktree add-a');
  assert.ok(plan.hc.includes('--no-daemon'));
});

test('開工：沒有可用的 agent 時鈕還在，並說明原因', () => {
  assert.equal(planOf(start({}, { known: null }), '用 worktree').reason, '還在確認可用的 agent');
  assert.equal(planOf(start({}, { known: [] }), '直接 apply').kind, 'none');
});

test('開工：這些情況沒有開工鈕', () => {
  assert.equal(start({ stage: 'ready' }), null, '任務都做完了');
  assert.equal(start({ blockedBy: ['add-z'] }), null, '被擋住');
  assert.equal(start({ restale: true }), null, '討論重新定案，要先 ingest');
  assert.equal(start({}, { agents: [agent({ labels: { 'speclink.change': 'add-a' } })] }), null, '已有 agent 在做');
  assert.equal(start({}, { starting: new Set(['add-a']) }), null, '剛按過，agent 還沒開好');
});

const stages = (...rows) => ({
  proposed: rows.filter((r) => r.stage === 'proposed'),
  'in-progress': rows.filter((r) => r.stage === 'in-progress'),
  ready: rows.filter((r) => r.stage === 'ready'),
});
const waveCtx = (over = {}) => ctx({ worktreePolicy: true, ...over });

test('整波開工：開工最前面那一波能開工的變更，波次取自 plan', () => {
  const wave = waveToStart(
    stages(
      row({ name: 'add-a', stage: 'proposed' }),
      row({ name: 'add-b', stage: 'in-progress' }),
      row({ name: 'add-c', stage: 'proposed', wave: 2, blockedBy: ['add-a'] }),
    ),
    waveCtx(),
  );
  assert.equal(wave.wave, 1);
  assert.deepEqual(
    wave.plans.map((p) => p.text),
    ['/speclink-apply-with-worktree add-a', '/speclink-apply-with-worktree add-b'],
  );
  assert.ok(wave.plans.every((p) => p.kind === 'tab'));
});

test('整波開工：已有 agent、已就緒、被擋的變更不算在內', () => {
  const wave = waveToStart(
    stages(
      row({ name: 'add-a', stage: 'proposed' }),
      row({ name: 'add-b', stage: 'proposed' }),
      row({ name: 'add-c', stage: 'proposed' }),
      row({ name: 'add-d', stage: 'ready' }),
      row({ name: 'add-e', stage: 'proposed', wave: 2, blockedBy: ['add-a'] }),
    ),
    waveCtx({ agents: [agent({ labels: { 'speclink.change': 'add-c' } })] }),
  );
  assert.deepEqual(wave.plans.map((p) => p.change), ['add-a', 'add-b']);
});

test('整波開工：能開工的不到兩個時不提供，因為列上的開工鈕就夠了', () => {
  assert.equal(waveToStart(stages(row({ name: 'add-a', stage: 'proposed' })), waveCtx()), null);
  assert.equal(waveToStart(stages(), waveCtx()), null);
});

test('整波開工：未開 worktree 政策時不提供', () => {
  const two = stages(row({ name: 'add-a', stage: 'proposed' }), row({ name: 'add-b', stage: 'proposed' }));
  assert.equal(waveToStart(two, waveCtx({ worktreePolicy: false })), null);
});

test('整波開工：沒有可用的 agent 時不提供', () => {
  const two = stages(row({ name: 'add-a', stage: 'proposed' }), row({ name: 'add-b', stage: 'proposed' }));
  assert.equal(waveToStart(two, waveCtx({ known: [] })), null);
});

const seat = (over) => ({ address: 't1', labels: {}, cwd: '/p', title: '', ...over });
const boardOf = (...rows) => ({ stages: { proposed: rows, 'in-progress': [], ready: [] } });

test('agentChange：和面板同一套比對，標籤、worktree、外掛回報都算；查不到是 null', () => {
  const foo = { name: 'add-foo', worktree: null };
  const bar = { name: 'add-bar', worktree: { path: '/p/.wt/add-bar' } };
  const board = boardOf(foo, bar);
  assert.equal(agentChange(seat({ labels: { 'speclink.change': 'add-foo' } }), board), foo);
  assert.equal(agentChange(seat({ cwd: '/p/.wt/add-bar/src' }), board), bar);
  assert.equal(agentChange(seat({ labels: { 'speclink.focus': 'add-bar' } }), board), bar);
  assert.equal(agentChange(seat({ labels: { 'speclink.focus': '-' } }), board), null);
  assert.equal(agentChange(seat(), board), null);
});

test('groupedOrder：等你回答的放最前面，其他照 Spaces 順序，同一個 space 裡急的和新的在前面', () => {
  const rank = { blocked: 4, done: 3, working: 2, idle: 1 };
  const a = (address, space, status, at, seq = 0) => ({ address, space, status, at, seq });
  const list = [
    a('c-idle', 'c', 'idle', 9),
    a('b-idle-old', 'b', 'idle', 1),
    a('b-idle-new', 'b', 'idle', 2),
    a('b-working', 'b', 'working', 0),
    a('c-blocked', 'c', 'blocked', 0),
    a('gone', 'z', 'idle', 0),
    a('a-done', 'a', 'done', 0),
  ];
  const order = groupedOrder(list, [{ id: 'a' }, { id: 'b' }, { id: 'c' }], (x) => rank[x.status], (x) => x.at);
  assert.deepEqual(order.map((x) => x.address), ['c-blocked', 'a-done', 'b-working', 'b-idle-new', 'b-idle-old', 'c-idle', 'gone']);
});

test('statusSections：狀態相同的相鄰成一段，順序照排好的清單', () => {
  const rank = { done: 3, working: 2, idle: 1 };
  const members = ['done', 'working', 'idle', 'idle'].map((status, i) => ({ status, i }));
  const sections = statusSections(members, (a) => rank[a.status]);
  assert.deepEqual(sections.map((s) => [s.rank, s.members.map((a) => a.i)]), [[3, [0]], [2, [1]], [1, [2, 3]]]);
  assert.deepEqual(statusSections([], () => 1), []);
});

test('statusCounts：最急的在前面，沒有的不列', () => {
  const members = ['idle', 'working', 'idle', 'done'].map((status) => ({ status }));
  assert.deepEqual(statusCounts(members), [['done', 1], ['working', 1], ['idle', 2]]);
  assert.deepEqual(statusCounts([]), []);
});

test('langFrom：第一個語言算數，中文之外都用英文', () => {
  assert.equal(langFrom('(\n    "zh-Hant-TW"\n)\n'), 'zh');
  assert.equal(langFrom('(\n    "en-US",\n    "zh-Hant-TW"\n)\n'), 'en');
  assert.equal(langFrom('  zh_TW.UTF-8\n'), 'zh');
  assert.equal(langFrom('ja_JP.UTF-8'), 'en');
  assert.equal(langFrom(''), 'en');
});

test('tr：中文系統翻 Herdr 的英文，其他系統翻面板的中文，查不到用原文，佔位符照順序', () => {
  try {
    setLang('zh');
    assert.equal(tr('Needs you'), '需要你');
    assert.equal(tr('要等 {0} 落地', 'a'), '要等 a 落地');
    assert.equal(tr('apply'), 'apply');
    setLang('en');
    assert.equal(tr('Needs you'), 'Needs you');
    assert.equal(tr('要等 {0} 落地', 'a'), 'Waiting for a to land');
    assert.equal(tr('第 {0} 波可以一起開工：{1}', 2, 'x'), 'Wave 2 can start together: x');
    assert.equal(tr('apply'), 'apply');
    assert.equal(tr('$&{0}', 'x'), '$&x', '參數和原文裡的特殊字元不會被取代樣式吃掉');
  } finally {
    setLang('zh');
  }
});

test('placePlan：原因跟著語系', () => {
  try {
    setLang('en');
    const plan = placePlan({ skill: 'apply', target: null, note: '', agents: [], focus: {} });
    assert.equal(plan.reason, 'The current tab has no agent. You can open a new tab');
  } finally {
    setLang('zh');
  }
});

const HERDR_DIR = fileURLToPath(new URL('../../integrations/herdr/', import.meta.url));
const code = (file) =>
  readFileSync(path.join(HERDR_DIR, file), 'utf8')
    .split('\n')
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

test('語系守門：顯示的中文都有英文，也不用樣板字串繞過 tr', () => {
  const files = ['mods/speclink/main.js', 'mods/speclink/skill-picker.js', 'mods/speclink/shared.js', 'mods/speclink/model.js', 'spaces-badge/stage-badge.js', 'agents-list/speclink-rows.js'];
  for (const file of files) {
    const source = code(file).replace(/\s\/\/.*$/gm, '');
    for (const line of source.split('\n')) {
      assert.doesNotMatch(line.replace(/"[^"]*"/g, '""'), /`[^`]*\p{Script=Han}[^`]*`/u, `${file} 有中文樣板字串`);
    }
    for (const [, text] of source.matchAll(/"([^"\n]*)"/g)) {
      if (/\p{Script=Han}/u.test(text)) assert.ok(Object.hasOwn(TO_EN, text), `${file} 的「${text}」沒有英文`);
    }
  }
  for (const { group, about } of PICKER_SKILLS) {
    assert.ok(Object.hasOwn(TO_EN, group) && Object.hasOwn(TO_EN, about), `${group}／${about} 沒有英文`);
  }
});

test('語系守門：dock-panel 翻譯的英文都有中文', () => {
  const source = readFileSync(fileURLToPath(new URL('dock-panel.mjs', import.meta.url)), 'utf8');
  const keys = [...source.matchAll(/tr\([^"()]*"([^"]+)"(?: : "([^"]+)")?/g)].flatMap((m) => m.slice(1)).filter(Boolean);
  assert.ok(keys.length > 10);
  for (const key of keys) assert.ok(Object.hasOwn(TO_ZH, key), `「${key}」沒有中文`);
});
