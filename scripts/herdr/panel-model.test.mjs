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
  PICKER_ROWS,
  PICKER_SKILLS,
  PICKER_VARIANTS,
  SKILL_GROUPS,
  customPlacePlan,
  customTabPlan,
  defaultAgent,
  hasSpeclinkSkills,
  missingSkillsPlan,
  onlyClaude,
  pickerGroups,
  pickerStep,
  pluginFolders,
  skillEntries,
  skillFolders,
  skillRows,
  skillUse,
  usableAgents,
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
  parseStations,
  placePlan,
  planDelivery,
  rowChoices,
  sendArgs,
  commandLabels,
  hookLabels,
  promptLabels,
  skillLabels,
  skillTargets,
  statusCounts,
  statusSections,
  speclinkUrl,
  stampedPaths,
  startChoices,
  stationStates,
  stationViews,
  waveToStart,
} from '../../integrations/herdr/mods/speclink/model.js';
import { TO_EN, TO_ZH, autoTr, langFrom, setLang, tr, translateHerdr } from '../../integrations/herdr/mods/speclink/i18n.js';

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
      stations: { review: 'none', verify: 'none' },
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

test('changeSkills：提案中可以選 worktree，建議仍是 apply', () => {
  const skills = changeSkills(row({ stage: 'proposed' }), true);
  assert.deepEqual(names(skills), ['apply', 'apply-with-worktree', 'analyze', 'ingest', 'drift', 'commit']);
  assert.deepEqual(primary(skills), ['apply']);
});

test('changeSkills：不能選 worktree 的情況不多這一顆', () => {
  const absent = (over, policy = true) =>
    assert.ok(!names(changeSkills(row({ stage: 'proposed', ...over }), policy)).includes('apply-with-worktree'), JSON.stringify(over));
  absent({}, false); // 工作流設定沒開
  absent({ worktree: { path: TREE, branch: 'b' } }); // 已有 worktree，apply 鈕會接續它
  absent({ blockedBy: ['add-z'] }); // 被擋住
  absent({ restale: true }); // 討論重新定案，要先 ingest
  absent({ codeRemaining: 0 }); // 只剩手動任務
  absent({ stage: 'in-progress' }); // 已經開始做，地方已經決定了
  absent({ stage: 'ready' });
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
  assert.deepEqual(names(skills), ['quality', 'review', 'verify', 'archive', 'archive+commit', 'ingest', 'drift', 'commit']);
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

test('commandLabels：speclink 帶路徑或先存進變數也認得', () => {
  const talk = (slug) => ['speclink.change=', `speclink.discussion=${slug}`];
  const change = (name) => [`speclink.change=${name}`, 'speclink.discussion='];
  assert.deepEqual(commandLabels("cd /x; ~/.local/bin/speclink discuss add-round talk-a --mode assumptions --stdin <<'EOF'"), talk('talk-a'));
  assert.deepEqual(commandLabels('"$HOME/.local/bin/speclink" archive add-a -y'), change('add-a'));
  assert.deepEqual(commandLabels('./target/debug/speclink task done --change add-a tsk_1'), change('add-a'));
  assert.deepEqual(commandLabels('S=~/.local/bin/speclink; $S discuss new "要不要改" --slug talk-a 2>&1 | tail -3'), talk('talk-a'));
  assert.deepEqual(commandLabels('cd /x && S=~/.local/bin/speclink; ${S} discuss conclude talk-a --stdin'), talk('talk-a'));
  // 變數不是 speclink、或 speclink 只是別的指令的參數時不算。
  assert.equal(commandLabels('T=/usr/bin/tar; $T archive add-a'), null);
  assert.equal(commandLabels('cat ~/.local/bin/speclink archive add-a'), null);
  assert.equal(commandLabels('S=~/.local/bin/speclink; echo $S discuss conclude talk-a'), null);
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
    [{ address: 't1', status: 'done', name: 'add-a', labels: { a: 'b' } }],
    [{ pane: 'p1', terminal: 't1', cwd: REPO }],
  );
  assert.equal(a.status, 'done');
  assert.equal(a.name, 'add-a');
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
    { key: 'opencode', run: ['opencode'], installed: true },
    { key: 'codex', run: ['codex'], installed: true },
    { key: 'claude', run: ['claude'], installed: true },
  ];
  const d = deliver({ known });
  assert.deepEqual(d.argv, ['codex', '--no-daemon']);
  assert.equal(d.text, '$speclink-apply add-a');
});

test('沒有可用的 agent 時不開分頁並說明原因', () => {
  assert.equal(deliver({ known: [{ key: 'opencode', run: ['opencode'], installed: true }] }).kind, 'none');
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
  const d = deliver({ agents: [agent({ key: 'opencode' })] });
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

test('changeActions：提案時的 agent 停著時，apply-with-worktree 送給它', () => {
  const proposing = agent({ address: 't7', labels: { 'speclink.change': 'add-a' } });
  const actions = changeActions(row({ stage: 'proposed' }), ctx({ worktreePolicy: true, agents: [proposing] }));
  const { delivery } = actions.find((a) => a.skill === 'apply-with-worktree');
  assert.equal(delivery.kind, 'send');
  assert.equal(delivery.agent.address, 't7');
  assert.equal(delivery.text, '/speclink-apply-with-worktree add-a');
  // agent 還在忙時，和 apply 一樣不能按
  const busy = changeActions(row({ stage: 'proposed' }), ctx({ worktreePolicy: true, agents: [{ ...proposing, status: 'working' }] }));
  assert.equal(busy.find((a) => a.skill === 'apply-with-worktree').delivery.kind, 'busy');
  // 工作流設定沒開時沒有這一顆
  const off = changeActions(row({ stage: 'proposed' }), ctx({ agents: [proposing] }));
  assert.ok(!names(off).includes('apply-with-worktree'));
});

test('discussionActions：帶討論 slug 送達', () => {
  const d = { slug: 'talk', topic: '主題', status: 'concluded', cuts: [] };
  const [propose] = discussionActions(d, ctx({ agents: [agent()] }));
  assert.equal(propose.delivery.text, '/speclink-propose --from-discussion talk');
});

test('目前分頁：英文的標籤大寫開頭，接在句子裡時另有一句', () => {
  const busy = { kind: 'busy', reason: '正在做事' };
  assert.equal(deliveryHint(busy), '目前分頁的 agent 正在做事');
  setLang('en');
  try {
    assert.equal(tr('目前分頁'), 'Current tab');
    assert.match(deliveryHint(busy), /^Agent on the current tab: /);
  } finally {
    setLang('zh');
  }
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

test('newTabPlan：變更的 agent 在忙才擋，做完停著的不算', () => {
  const on = (status) => agent({ status, labels: { 'speclink.change': 'add-a' } });
  const merge = (status) => tab({ skill: 'worktree-merge', target: changeTarget(), agents: [on(status)] });
  for (const status of ['working', 'blocked', 'error']) assert.equal(merge(status).reason, '已經有 agent 在做', status);
  for (const status of ['idle', 'done']) assert.equal(merge(status).kind, 'tab', status);
  // 一個在忙一個停著，還是擋
  const both = tab({ skill: 'worktree-merge', target: changeTarget(), agents: [on('idle'), on('working')] });
  assert.equal(both.reason, '已經有 agent 在做');
});

test('newTabPlan：主資料夾的 apply 還沒做完時，agent 停著也不能再開一個', () => {
  const row = changeTarget().row;
  const board = { stages: { proposed: [], 'in-progress': [{ ...row, stage: 'in-progress' }], ready: [] } };
  const idle = agent({ status: 'idle', labels: { 'speclink.change': 'add-a', 'speclink.skill': 'speclink-apply' } });
  assert.equal(tab({ skill: 'apply', target: changeTarget(), agents: [idle], board }).reason, 'add-a 的 apply 還沒做完');
});

test('newTabPlan：名稱被別的終端機佔用時不取名，其他照舊', () => {
  const held = agent({ status: 'idle', name: 'add-a', labels: { 'speclink.change': 'add-a' } });
  const plan = tab({ skill: 'worktree-merge', target: changeTarget(), agents: [held] });
  assert.ok(!plan.hc.includes('--name'));
  assert.ok(plan.hc.includes('speclink.change=add-a'));
  assert.ok(plan.hc.includes('speclink.skill=speclink-worktree-merge'));
  // 標籤已經換到別的變更，名稱還佔著
  const moved = agent({ status: 'idle', name: 'add-a', labels: { 'speclink.change': 'add-z' } });
  assert.ok(!tab({ skill: 'review', target: changeTarget(), agents: [moved] }).hc.includes('--name'));
  // 名稱沒人用就照舊取名，hc 的名稱最多 32 個字
  const free = tab({ skill: 'review', target: changeTarget(), agents: [agent({ name: 'other' })] });
  assert.equal(free.hc[free.hc.indexOf('--name') + 1], 'add-a');
  const long = 'a'.repeat(33);
  const cut = agent({ status: 'idle', name: 'a'.repeat(32), labels: {} });
  const target = changeTarget({ name: long });
  assert.ok(!tab({ skill: 'review', target: { ...target, name: long, args: long }, agents: [cut] }).hc.includes('--name'));
});

test('batchPlan：做完停著的 agent 不會讓那一列被略過', () => {
  const target = (name) => ({ kind: 'change', name, args: name, row: row({ name, stage: 'ready' }) });
  const idle = agent({ status: 'idle', labels: { 'speclink.change': 'add-a' } });
  const plan = batchPlan({ ...ctx({ agents: [idle] }), skill: 'review', targets: [target('add-a'), target('add-b')], space: 's1', on: null });
  assert.equal(plan.kind, 'tabs');
  assert.equal(plan.plans.length, 2);
});

test('newTabPlan：Codex 用錢字號並帶 --no-daemon', () => {
  const plan = tab({ known: [{ key: 'codex', run: ['codex'], installed: true }] });
  assert.deepEqual(plan.hc.slice(-3), ['codex', '--no-daemon', '$speclink-improve']);
});

test('newTabPlan：這些情況不開，並寫出原因', () => {
  const mine = agent({ status: 'working', labels: { 'speclink.change': 'add-a' } });
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
  const mine = agent({ status: 'working', labels: { 'speclink.change': 'add-a' } });
  assert.equal(batch({ targets: [named('add-a')], agents: [mine] }).reason, '勾選要開工的變更');
});

test('batchPlan：不能開的變更略過，其他照開', () => {
  const mine = agent({ status: 'working', labels: { 'speclink.change': 'add-a' } });
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

// ---- 品質關卡的章 ----

const META = `schema: spec-driven
created: 2026-11-04T15:00:22+08:00
reviewed_at: 2026-11-04T17:09:58+08:00
reviewed_by: M <m@example.com>
reviewed_tasks_total: 12
reviewed_rounds: 3
reviewed_scope:
  - path: src/a.js
    hash: aaa
  - path: "src/b c.js"
    hash: bbb
verified_at: 2026-11-04T17:09:58+08:00
verified_tasks_total: 12
verified_scope:
  - path: src/a.js
    hash: aaa
`;
const STAMP_R = { total: 12, scope: [{ path: 'src/a.js', hash: 'aaa' }, { path: 'src/b c.js', hash: 'bbb' }] };
const STAMP_V = { total: 12, scope: [{ path: 'src/a.js', hash: 'aaa' }] };

test('parseStations：讀 .meta.yaml 的章和工單', () => {
  assert.deepEqual(parseStations(META), {
    review: { ticket: false, stamp: STAMP_R },
    verify: { ticket: false, stamp: STAMP_V },
  });
  assert.deepEqual(parseStations(`${META}\nopen_ticket: verify\n`).verify.ticket, true);
  assert.deepEqual(parseStations('schema: spec-driven\n'), {
    review: { ticket: false, stamp: null },
    verify: { ticket: false, stamp: null },
  });
  assert.equal(parseStations('').review.stamp, null);
});

test('parseStations：欄位不全不算章（引擎的 is_stamped）', () => {
  const noScope = 'reviewed_at: x\nreviewed_tasks_total: 3\n';
  const noTotal = 'reviewed_at: x\nreviewed_scope:\n  - path: a\n    hash: b\n';
  const noTime = 'reviewed_tasks_total: 3\nreviewed_scope:\n  - path: a\n    hash: b\n';
  for (const meta of [noScope, noTotal, noTime]) assert.equal(parseStations(meta).review.stamp, null);
});

test('stampedPaths：要查的檔案不重複', () => {
  assert.deepEqual(stampedPaths(parseStations(META)), ['src/a.js', 'src/b c.js']);
  assert.deepEqual(stampedPaths(parseStations('')), []);
});

test('stationStates：判斷和引擎的封存守門相同', () => {
  const fresh = { 'src/a.js': 'aaa', 'src/b c.js': 'bbb' };
  const facts = (over = {}) => ({ ...parseStations(META), hashes: fresh, ...over });
  const base = { total: 12, codeRemaining: 0 };
  assert.deepEqual(stationStates(facts(), base), { review: 'stamped', verify: 'stamped' });
  // 任務數變了
  assert.deepEqual(stationStates(facts(), { ...base, total: 13 }), { review: 'stale', verify: 'stale' });
  // 程式任務又多了沒做完的
  assert.deepEqual(stationStates(facts(), { ...base, codeRemaining: 1 }), { review: 'stale', verify: 'stale' });
  // 只有範圍內的檔案算：b 只在 review 的範圍內
  assert.deepEqual(stationStates(facts({ hashes: { ...fresh, 'src/b c.js': 'xxx' } }), base), { review: 'stale', verify: 'stamped' });
  // 檔案不見了
  assert.deepEqual(stationStates(facts({ hashes: { 'src/a.js': 'aaa' } }), base), { review: 'stale', verify: 'stamped' });
  // 查不到雜湊時只看任務，和引擎沒有工作樹時相同
  assert.deepEqual(stationStates(facts({ hashes: null }), base), { review: 'stamped', verify: 'stamped' });
  // 查不到程式任務數不算過期
  assert.deepEqual(stationStates(facts(), { total: 12, codeRemaining: null }), { review: 'stamped', verify: 'stamped' });
  // 工單開著就不看章
  const open = parseStations(`${META}\nopen_ticket: review\n`);
  assert.deepEqual(stationStates({ ...open, hashes: { 'src/a.js': 'xxx' } }, base), { review: 'open', verify: 'stale' });
  // 沒有資料
  assert.deepEqual(stationStates(undefined, base), { review: 'none', verify: 'none' });
});

test('buildBoard：列上帶兩站的狀態', () => {
  const list = { changes: [{ name: 'a', completedTasks: 12, totalTasks: 12 }, { name: 'b', completedTasks: 0, totalTasks: 12 }] };
  const plan = {
    changes: [
      { name: 'a', wave: 1, stage: 'ready', blockedBy: [] },
      { name: 'b', wave: 1, stage: 'proposed', blockedBy: [] },
    ],
  };
  const hashes = { 'src/a.js': 'aaa', 'src/b c.js': 'bbb' };
  const board = buildBoard(list, plan, { discussions: [] }, {}, { a: { ...parseStations(META), hashes } });
  assert.deepEqual(board.stages.ready[0].stations, { review: 'stamped', verify: 'stamped' });
  assert.deepEqual(board.stages.proposed[0].stations, { review: 'none', verify: 'none' });
  // 技能跳窗不查章，也不會壞
  assert.deepEqual(buildBoard(list, plan, { discussions: [] }).stages.ready[0].stations, { review: 'none', verify: 'none' });
});

test('stationViews：agent 正在跑的站顯示 running，沒有狀態的站不畫', () => {
  const on = (skill, status = 'working') => agent({ status, labels: { 'speclink.change': 'add-a', 'speclink.skill': `speclink-${skill}` } });
  const stations = (review, verify) => row({ stage: 'ready', stations: { review, verify } });
  const states = (r, agents = []) => stationViews(r, agents).map((v) => [v.station, v.state]);
  assert.deepEqual(states(stations('none', 'none')), []);
  assert.deepEqual(states(row({ stage: 'ready' })), [], '沒有查過章的列');
  assert.deepEqual(states(stations('stamped', 'open')), [['review', 'stamped'], ['verify', 'open']]);
  // quality 跑兩站，還沒有工單時也要顯示
  assert.deepEqual(states(stations('none', 'none'), [on('quality')]), [['review', 'running'], ['verify', 'running']]);
  assert.deepEqual(states(stations('none', 'none'), [on('review')]), [['review', 'running']]);
  assert.deepEqual(states(stations('none', 'none'), [on('verify')]), [['verify', 'running']]);
  // 蓋好的站就算蓋好了，另一站還在跑
  assert.deepEqual(states(stations('stamped', 'open'), [on('quality')]), [['review', 'stamped'], ['verify', 'running']]);
  // 重跑過期的章
  assert.deepEqual(states(stations('stale', 'stale'), [on('quality')]), [['review', 'running'], ['verify', 'running']]);
  // agent 停下來、做別的事、或做別的變更，都不算在跑
  assert.deepEqual(states(stations('open', 'none'), [on('quality', 'blocked')]), [['review', 'open']]);
  assert.deepEqual(states(stations('none', 'none'), [on('apply')]), []);
  const other = agent({ status: 'working', labels: { 'speclink.change': 'add-z', 'speclink.skill': 'speclink-quality' } });
  assert.deepEqual(states(stations('none', 'none'), [other]), []);
});

test('changeSkills：兩站都蓋章後，建議封存加提交', () => {
  const primary = (over) => changeSkills(row({ stage: 'ready', ...over })).find((s) => s.primary)?.skill;
  const both = { review: 'stamped', verify: 'stamped' };
  assert.equal(primary({}), 'quality');
  assert.equal(primary({ stations: both }), 'archive+commit');
  // 只蓋了一站、章過期、工單開著，都還要跑
  for (const verify of ['none', 'stale', 'open']) assert.equal(primary({ stations: { review: 'stamped', verify } }), 'quality');
  // 有 worktree 要先合併，封存不能選
  const merged = changeSkills(row({ stage: 'ready', stations: both, worktree: { path: TREE, branch: 'b' } }));
  assert.equal(merged.find((s) => s.primary).skill, 'worktree-merge');
  assert.ok(!merged.some((s) => s.skill.startsWith('archive')));
  // 討論重新定案優先
  assert.equal(primary({ stations: both, restale: true }), 'ingest');
  // 蓋了章但手動任務還沒做：不用再跑一次 quality，也還不能封存
  const manual = changeSkills(row({ stage: 'in-progress', codeRemaining: 0, stations: both }));
  assert.equal(manual.find((s) => s.primary).skill, 'apply');
  assert.ok(!manual.some((s) => s.skill.startsWith('archive')));
  assert.equal(changeSkills(row({ stage: 'in-progress', codeRemaining: 0 })).find((s) => s.primary).skill, 'quality');
});

test('rowChoices：建議的那一顆放外面，其他的按 ⋯ 才展開', () => {
  const quality = { skill: 'quality', primary: true };
  const review = { skill: 'review', primary: false };
  const merge = { skill: 'worktree-merge', primary: false };
  assert.deepEqual(rowChoices([review, quality, merge]), { next: quality, beside: [], others: [review, merge] });
  // apply 被擋時沒有建議，列上放第一顆
  assert.deepEqual(rowChoices([review, merge]), { next: review, beside: [], others: [merge] });
  // 討論中只有一個技能，沒有東西可以展開
  assert.deepEqual(rowChoices([quality]), { next: quality, beside: [], others: [] });
});

test('rowChoices：apply-with-worktree 放 apply 旁邊，有開工鈕時收進 ⋯', () => {
  const apply = { skill: 'apply', primary: true };
  const tree = { skill: 'apply-with-worktree', primary: false };
  const drift = { skill: 'drift', primary: false };
  assert.deepEqual(rowChoices([apply, tree, drift]), { next: apply, beside: [tree], others: [drift] });
  // 開工已經能選 worktree，列上不用重複
  assert.deepEqual(rowChoices([apply, tree, drift], [{ label: '用 worktree' }]), { next: apply, beside: [], others: [tree, drift] });
});

test('rowChoices：做完 apply 的 worktree 變更，建議合併，展開後還能跑 quality', () => {
  const actions = changeActions(row({ stage: 'ready', worktree: { path: TREE, branch: 'b' } }), ctx());
  const { next, others } = rowChoices(actions);
  assert.equal(next.skill, 'worktree-merge');
  assert.ok(others.some((a) => a.skill === 'quality'));
  assert.ok(others.every((a) => a.skill !== 'worktree-merge'));
});

test('changeSkills：有 worktree 時，程式任務做完就建議合併，不管有沒有蓋章', () => {
  const tree = { path: TREE, branch: 'b' };
  const primary = (over) => changeSkills(row({ worktree: tree, ...over })).find((s) => s.primary)?.skill;
  assert.equal(primary({ stage: 'ready' }), 'worktree-merge');
  for (const verify of ['none', 'stale', 'open', 'stamped']) {
    assert.equal(primary({ stage: 'ready', stations: { review: 'stamped', verify } }), 'worktree-merge', verify);
  }
  // 只剩手動任務
  assert.equal(primary({ stage: 'in-progress', codeRemaining: 0 }), 'worktree-merge');
  // 程式還沒做完、查不到進度、還沒開工，都留在 apply
  assert.equal(primary({ stage: 'in-progress', codeRemaining: 2 }), 'apply');
  assert.equal(primary({ stage: 'in-progress', codeRemaining: null }), 'apply');
  assert.equal(primary({ stage: 'proposed' }), 'apply');
  // 討論重新定案優先
  assert.equal(primary({ stage: 'ready', restale: true }), 'ingest');
  // 沒有 worktree 不受影響
  assert.equal(changeSkills(row({ stage: 'ready' })).find((s) => s.primary).skill, 'quality');
  assert.equal(changeSkills(row({ stage: 'in-progress', codeRemaining: 0 })).find((s) => s.primary).skill, 'quality');
});

// ---- archive+commit ----

const readyRow = (over = {}) => row({ stage: 'ready', stations: { review: 'stamped', verify: 'stamped' }, ...over });

test('archive+commit：指令和 Claude Code mod 相同，兩個都帶前綴', () => {
  assert.equal(commandText('claude', 'archive+commit', 'add-a'), '/speclink-archive + /speclink-commit add-a');
  assert.equal(commandText('codex', 'archive+commit', 'add-a'), '$speclink-archive + $speclink-commit add-a');
  assert.equal(commandText('claude', 'archive+commit'), '/speclink-archive + /speclink-commit');
  assert.equal(commandText('claude', 'archive', 'add-a'), '/speclink-archive add-a');
});

test('archive+commit：跳窗放在封存後面，對象和封存相同', () => {
  const names = PICKER_SKILLS.filter((w) => w.group === '收尾').map((w) => w.skill);
  assert.deepEqual(names, ['archive', 'archive+commit', 'worktree-merge', 'commit']);
  const board = {
    stages: { proposed: [], 'in-progress': [], ready: [readyRow(), readyRow({ name: 'add-t', worktree: { path: TREE, branch: 'b' } })] },
    discussions: [],
  };
  assert.deepEqual(skillTargets('archive+commit', board).map((t) => t.name), skillTargets('archive', board).map((t) => t.name));
  assert.deepEqual(skillTargets('archive+commit', board).map((t) => t.name), ['add-a']);
});

test('archive+commit：面板的鈕送到主資料夾的 agent，標籤算 archive', () => {
  const idle = agent({ labels: { 'speclink.change': 'add-a' } });
  const send = planDelivery({ ...ctx({ agents: [idle] }), skill: 'archive+commit', change: readyRow(), args: 'add-a' });
  assert.equal(send.kind, 'send');
  assert.equal(send.text, '/speclink-archive + /speclink-commit add-a');
  assert.deepEqual(send.labels, ['speclink.skill=speclink-archive', 'speclink.change=add-a', 'speclink.discussion=']);
  const fresh = planDelivery({ ...ctx(), focus: { tab: 'other', terminal: null }, skill: 'archive+commit', change: readyRow(), args: 'add-a' });
  assert.equal(fresh.kind, 'new');
  assert.equal(fresh.folder, REPO);
  assert.equal(fresh.tabName, 'archive+commit · add-a');
});

test('archive+commit：跳窗放到輸入框和開新分頁', () => {
  const target = { kind: 'change', name: 'add-a', args: 'add-a', row: readyRow() };
  const placed = placePlan({ skill: 'archive+commit', target, note: '', agents: [agent()], focus: { tab: 'tab1', terminal: 't1' } });
  assert.equal(placed.text, '/speclink-archive + /speclink-commit add-a ');
  assert.deepEqual(placed.labels, ['speclink.skill=speclink-archive', 'speclink.change=add-a', 'speclink.discussion=']);
  const tab = newTabPlan({ ...ctx(), skill: 'archive+commit', target, note: '', space: 's1', on: null });
  assert.equal(tab.kind, 'tab');
  assert.ok(tab.hc.includes('speclink.skill=speclink-archive'));
  assert.equal(tab.hc.at(-1), '/speclink-archive + /speclink-commit add-a');
});

test('archive+commit：一次只開一個，同時提交會互相影響', () => {
  const targets = ['add-a', 'add-b'].map((name) => ({ kind: 'change', name, args: name, row: readyRow({ name }) }));
  const plan = batchPlan({ ...ctx(), skill: 'archive+commit', targets, space: 's1', on: null });
  assert.equal(plan.kind, 'none');
  assert.equal(batchPlan({ ...ctx(), skill: 'archive', targets, space: 's1', on: null }).kind, 'tabs');
});

test('promptLabels：認得疊加的指令', () => {
  const labels = ['speclink.skill=speclink-archive', 'speclink.change=add-a', 'speclink.discussion='];
  assert.deepEqual(promptLabels('/speclink-archive + /speclink-commit add-a'), labels);
  assert.deepEqual(hookLabels({ tool_name: 'Skill', tool_input: { skill: 'speclink-archive', args: '+ /speclink-commit add-a' } }), labels);
  assert.deepEqual(promptLabels('/speclink-archive + /speclink-commit'), ['speclink.skill=speclink-archive', 'speclink.discussion=']);
});

// ---- 內建畫面的翻譯 ----

test('autoTr：中文系統查對照表，英文系統維持原文', () => {
  try {
    setLang('zh');
    assert.equal(autoTr('Settings'), '設定');
    assert.equal(autoTr('Commit'), '提交');
    assert.equal(autoTr('LAYOUT'), '版面');
    setLang('en');
    assert.equal(autoTr('Settings'), 'Settings');
    assert.equal(autoTr('7 agents'), '7 agents');
  } finally {
    setLang('zh');
  }
});

test('autoTr：帶變數的句子要整句符合才翻', () => {
  assert.equal(autoTr('7 agents'), '7 個 agent');
  assert.equal(autoTr('· 1 needs you'), '· 1 個需要你');
  assert.equal(autoTr('⌘⇧P  commands'), '⌘⇧P  指令');
  assert.equal(autoTr('12 files'), '12 個檔案');
  assert.equal(autoTr('Theme: graphite'), '主題：graphite');
  assert.equal(autoTr('Default (Geist)'), '預設（Geist）');
  assert.equal(autoTr('Up to date. Checked at 09:30.'), '已是最新版本。檢查時間 09:30。');
  assert.equal(autoTr('0.1.13 is ready. Restart to switch now; it also switches when you quit.'), '0.1.13 已經準備好。重新啟動就會切換，結束 Herdr 時也會切換。');
  // 使用者取的名稱不算
  for (const name of ['two agents', 'speclink', 'add-team-invites', 'my files', 'main']) assert.equal(autoTr(name), name);
});

test('translateHerdr：畫面上的字串都查對照表，其他東西原樣傳下去', () => {
  class Fake {
    child(c) {
      this.got = c;
      return this;
    }
    children(items) {
      this.got = [...items];
      return this;
    }
    tooltip(t) {
      this.got = t;
      return this;
    }
  }
  const inputs = { new: (options) => options };
  const ui = { div: () => new Fake(), InputState: inputs };
  translateHerdr(ui);
  const element = new Fake();
  assert.equal(element.child('Settings').got, '設定');
  assert.equal(element.child('add-team-invites').got, 'add-team-invites');
  assert.equal(element.child(3).got, 3);
  const inner = new Fake();
  assert.equal(element.child(inner).got, inner);
  assert.deepEqual(element.children(['Look', 5, 'x']).got, ['外觀', 5, 'x']);
  assert.equal(element.tooltip('Rename').got, '重新命名');
  assert.deepEqual(inputs.new({ placeholder: 'Commit message', value: 'a' }), { placeholder: '提交訊息', value: 'a' });
  assert.equal(inputs.new(), undefined);
  // 再裝一次（mod 重新載入）不會包兩層
  const once = Fake.prototype.child;
  translateHerdr(ui);
  assert.equal(Fake.prototype.child, once);
  assert.equal(new Fake().child('Settings').got, '設定');
});

test('dock-panel：檔案樹的名稱不會被翻，改不到時不擋停靠', () => {
  const source = readFileSync(fileURLToPath(new URL('dock-panel.mjs', import.meta.url)), 'utf8');
  assert.match(source, /as: 'speclink-files'.*optional: UNPROTECTED/);
  for (const name of ['speclink-status', 'speclink-chrome']) assert.ok(!source.includes(name), name);
});

test('dock-panel：捲動頁籤列不切換頁籤，改不到時不擋停靠', () => {
  const source = readFileSync(fileURLToPath(new URL('dock-panel.mjs', import.meta.url)), 'utf8');
  assert.match(source, /const TABS_EDITS = \[\['[^']*on_scroll_wheel[^']*tab\.prev[^']*', ''\]\];/);
  assert.match(source, /as: 'speclink-tabs', edits: TABS_EDITS.*optional: '/);
});

const HERDR_DIR = fileURLToPath(new URL('../../integrations/herdr/', import.meta.url));
const code = (file) =>
  readFileSync(path.join(HERDR_DIR, file), 'utf8')
    .split('\n')
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

test('語系守門：顯示的中文都有英文，也不用樣板字串繞過 tr', () => {
  const files = ['mods/speclink/main.js', 'mods/speclink/skill-picker.js', 'mods/speclink/my-skills.js', 'mods/speclink/shared.js', 'mods/speclink/model.js', 'spaces-badge/stage-badge.js', 'agents-list/speclink-rows.js'];
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

// ---- Codex 和 Pi ----

const ALL_AGENTS = [
  { key: 'claude', name: 'Claude Code', run: ['claude'], installed: true },
  { key: 'codex', name: 'Codex', run: ['codex'], installed: true },
  { key: 'pi', name: 'Pi', run: ['pi'], installed: true },
  { key: 'opencode', name: 'OpenCode', run: ['opencode'], installed: true },
  { key: 'copilot', name: 'Copilot', run: ['copilot'], installed: false },
];

test('叫用寫法：每個 agent 用自己的前綴，疊加的技能也一樣', () => {
  assert.equal(commandText('claude', 'apply', 'a'), '/speclink-apply a');
  assert.equal(commandText('codex', 'apply', 'a'), '$speclink-apply a');
  assert.equal(commandText('pi', 'apply', 'a'), '/skill:speclink-apply a');
  assert.equal(commandText('pi', 'archive+commit', 'a'), '/skill:speclink-archive + /skill:speclink-commit a');
});

test('Pi 是送達對象；開新分頁時直接帶指令', () => {
  const d = planDelivery({ ...ctx({ agents: [agent({ key: 'pi' })] }), skill: 'apply', args: 'add-a' });
  assert.equal(d.kind, 'send');
  assert.equal(d.text, '/skill:speclink-apply add-a');
  const fresh = planDelivery({ ...ctx({ known: ALL_AGENTS, preferred: 'pi' }), skill: 'apply', args: 'add-a' });
  assert.equal(fresh.kind, 'new');
  assert.deepEqual(fresh.argv, ['pi']);
  assert.equal(fresh.text, '/skill:speclink-apply add-a');
});

test('可以選的 agent：要裝好、要有 speclink 技能', () => {
  assert.deepEqual(usableAgents(ALL_AGENTS).map((a) => a.key), ['claude', 'codex', 'pi']);
  assert.deepEqual(usableAgents(null), []);
});

test('開新分頁用選的 agent；選的沒裝就用最常用的', () => {
  const plan = (preferred) => newTabPlan({ ...ctx({ known: ALL_AGENTS, preferred, space: 's1' }), skill: 'discuss', target: null, note: 'x' });
  assert.deepEqual(plan('codex').hc.slice(-3), ['codex', '--no-daemon', '$speclink-discuss x']);
  assert.deepEqual(plan('pi').hc.slice(-2), ['pi', '/skill:speclink-discuss x']);
  assert.deepEqual(plan('copilot').hc.slice(-2), ['claude', '/speclink-discuss x']);
});

test('跳窗預設給目前分頁的 agent，再來是上次選的，最後是最常用的', () => {
  const pick = (over) => defaultAgent({ known: ALL_AGENTS, agents: [], focus: { tab: 'tab1', terminal: null }, preferred: null, ...over });
  assert.equal(pick({}), 'claude');
  assert.equal(pick({ preferred: 'pi' }), 'pi');
  assert.equal(pick({ preferred: 'pi', agents: [agent({ key: 'codex' })] }), 'codex');
  assert.equal(pick({ preferred: 'copilot' }), 'claude', '選的已經沒裝');
  assert.equal(defaultAgent({ known: [], agents: [], focus: {}, preferred: 'pi' }), null);
});

test('放到輸入框：選的 agent 和目前分頁不同時不放', () => {
  const base = { skill: 'discuss', target: null, note: '', agents: [agent({ program: { name: 'Claude Code' } })], focus: { tab: 'tab1', terminal: 't1' } };
  assert.equal(placePlan({ ...base, agentKey: 'claude' }).kind, 'place');
  assert.equal(placePlan(base).kind, 'place', '沒有選時照舊');
  const other = placePlan({ ...base, agentKey: 'pi' });
  assert.equal(other.kind, 'none');
  assert.match(other.reason, /Claude Code/);
});

test('analyze 只有 Claude Code 有', () => {
  assert.equal(onlyClaude('analyze', 'claude'), false);
  assert.equal(onlyClaude('analyze', 'pi'), true);
  assert.equal(onlyClaude('apply', 'pi'), false);
});

test('專案沒有這個 agent 的技能時，說明要怎麼裝；Pi 和 Codex 共用同一份', () => {
  assert.equal(missingSkillsPlan('Pi', 'pi').kind, 'none');
  assert.match(missingSkillsPlan('Pi', 'pi').reason, /Pi.*--tools codex/);
  assert.match(missingSkillsPlan('Claude Code', 'claude').reason, /--tools claude/);
});

test('hook：認得三種叫用寫法', () => {
  for (const prefix of ['/', '$', '/skill:']) {
    assert.deepEqual(promptLabels(`${prefix}speclink-apply add-a`), skillLabels('apply', 'add-a'), prefix);
    assert.deepEqual(promptLabels(`${prefix}speclink-archive + ${prefix}speclink-commit add-a`), skillLabels('archive', 'add-a'), prefix);
    assert.deepEqual(promptLabels(`${prefix}speclink-propose --from-discussion d`), skillLabels('propose', 'd'), prefix);
  }
  assert.equal(promptLabels('$other add-a'), null);
  assert.equal(promptLabels('/skill:other add-a'), null);
});

// ---- 跳窗的 Speclink 頁 ----

test('合成一列：每個做法都是真的技能，清單少掉其他做法', () => {
  const names = new Set(PICKER_SKILLS.map((w) => w.skill));
  for (const [row, choices] of Object.entries(PICKER_VARIANTS)) {
    assert.equal(choices[0].skill, row, '第一個是預設，也是那一列的名字');
    for (const { skill } of choices) assert.ok(names.has(skill), skill);
  }
  assert.equal(PICKER_ROWS.length, 16);
  for (const hidden of ['apply-with-worktree', 'review', 'verify', 'archive+commit']) {
    assert.ok(!PICKER_ROWS.some((w) => w.skill === hidden), hidden);
  }
});

test('清單只列現在能做的：沒有變更時只剩規劃；有變更時藏掉沒得搭配的；專案預設收合', () => {
  const empty = pickerGroups(() => 0);
  assert.deepEqual(empty.map((g) => [g.group, g.state]), [['規劃', 'open'], ['實作', 'closed'], ['品質', 'closed'], ['收尾', 'closed'], ['專案', 'closed']]);
  assert.deepEqual(empty[0].rows.map((w) => w.skill), ['discuss', 'propose']);
  const counts = { ingest: 5, apply: 5, analyze: 5, drift: 5, quality: 3, 'worktree-merge': 3, commit: 5 };
  const busy = pickerGroups((skill) => counts[skill] ?? 0);
  assert.deepEqual(busy.flatMap((g) => g.rows.map((w) => w.skill)), ['discuss', 'propose', 'ingest', 'apply', 'analyze', 'drift', 'quality', 'worktree-merge', 'commit']);
  assert.equal(busy.find((g) => g.group === '專案').state, 'closed');
});

test('點過的組照使用者的：all 全列，closed 收起來', () => {
  const groups = pickerGroups(() => 0, { 專案: 'all', 規劃: 'closed' });
  assert.equal(groups.find((g) => g.group === '專案').rows.length, 6);
  assert.equal(groups.find((g) => g.group === '規劃').rows.length, 0);
});

test('↑↓ 走得到收合的組，走進去就展開；頭尾相接', () => {
  const groups = pickerGroups(() => 0);
  assert.deepEqual(pickerStep(groups, 'discuss', 1), { skill: 'propose', open: null });
  assert.deepEqual(pickerStep(groups, 'propose', 1), { skill: 'apply', open: '實作' });
  assert.deepEqual(pickerStep(groups, 'discuss', -1), { skill: 'config', open: '專案' });
  assert.deepEqual(pickerStep(groups, 'not-shown', 1), { skill: 'discuss', open: null });
});

// ---- 我的技能 ----

const skill = (name, source = 'project', about = `${name} 的說明`) => ({ name, source, path: `.claude/skills/${name}`, about, use: skillUse(name) });

test('用途：實際的技能名稱都分得對，對不到的是其他', () => {
  const expected = {
    'remotion-render': 'video',
    'motion-graphics': 'anim',
    'gsap-core': 'anim',
    'threejs-shaders': '3d',
    'momo-social-publish': 'content',
    'momo-short-video-publish': 'video',
    'frontend-design:frontend-design': 'design',
    'ui-ux-pro-max': 'design',
    'conventional-commit': 'ship',
    release: 'ship',
    'tdd-workflow': 'dev',
    'herdr-dispatch': 'tool',
    'skill-creator:skill-creator': 'tool',
    'terminal-inserts': 'other',
  };
  for (const [name, use] of Object.entries(expected)) assert.equal(skillUse(name), use, name);
  const ids = new Set(SKILL_GROUPS.map((g) => g.id));
  for (const use of Object.values(expected)) assert.ok(ids.has(use), use);
});

test('每個 agent 讀自己的資料夾', () => {
  const dirs = (key) => skillFolders(key).map((f) => f.dir);
  assert.deepEqual(dirs('claude'), ['.claude/skills', '~/.claude/skills']);
  assert.ok(dirs('codex').includes('~/.codex/skills'));
  assert.ok(dirs('pi').includes('.agents/skills') && dirs('pi').includes('~/.pi/agent/skills'));
  assert.deepEqual(dirs('opencode'), []);
});

test('外掛：只算已啟用的；專案範圍只算這個專案；不掃快取裡的舊版', () => {
  const installed = {
    plugins: {
      'a@m': [{ scope: 'user', installPath: '/c/a/2' }],
      'b@m': [{ scope: 'project', projectPath: '/work/repo', installPath: '/c/b/1' }],
      'c@m': [{ scope: 'project', projectPath: '/work/other', installPath: '/c/c/1' }],
      'd@m': [{ scope: 'user', installPath: '/c/d/1' }],
    },
  };
  const settings = [{ enabledPlugins: { 'a@m': true, 'c@m': true, 'd@m': true } }, { enabledPlugins: { 'b@m': true, 'd@m': false } }, null];
  assert.deepEqual(pluginFolders(installed, settings, '/work/repo'), [
    { source: 'plugin', prefix: 'a:', dir: '/c/a/2/skills' },
    { source: 'plugin', prefix: 'b:', dir: '/c/b/1/skills' },
  ]);
  assert.deepEqual(pluginFolders(null, [null], '/work/repo'), []);
});

test('清單：不列 Speclink 和不能手動叫用的，同名只留第一個，說明整理過', () => {
  const found = [
    { source: 'project', name: 'speclink-apply', description: 'x', invocable: '', path: 'p' },
    { source: 'project', name: 'shadcn', description: 'x', invocable: 'false', path: 'p' },
    { source: 'project', name: 'release', description: '"引號"', invocable: '', path: '.claude/skills/release' },
    { source: 'personal', name: 'release', description: '同名', invocable: '', path: '~/.claude/skills/release' },
    { source: 'plugin', name: 'frontend-design:frontend-design', description: '> 摺疊 的值', invocable: '', path: '/c/x' },
    { source: 'personal', name: 'long', description: 'a'.repeat(300), invocable: '', path: 'p' },
  ];
  const list = skillEntries(found);
  assert.deepEqual(list.map((x) => x.name), ['frontend-design:frontend-design', 'long', 'release']);
  assert.equal(list[2].about, '引號');
  assert.equal(list[2].source, 'project');
  assert.equal(list[0].about, '摺疊 的值');
  assert.equal(list[0].path, 'frontend-design', '外掛顯示外掛的名稱');
  assert.equal(list[1].about.length, 201);
  assert.equal(hasSpeclinkSkills(found), true);
  assert.equal(hasSpeclinkSkills(found.slice(1)), false);
});

test('列：依用途分組；三個以上同開頭收成系列；整組同系列時不多一層', () => {
  const skills = ['gsap-a', 'gsap-b', 'gsap-c', 'motion-graphics', 'threejs-a', 'threejs-b', 'threejs-c', 'release'].map((n) => skill(n));
  const rows = skillRows({ skills });
  assert.deepEqual(rows.map((r) => r.key), ['g:ship', 's:release', 'g:anim', 'f:anim/gsap', 's:motion-graphics', 'g:3d', 's:threejs-a', 's:threejs-b', 's:threejs-c']);
  assert.deepEqual(rows.find((r) => r.kind === 'family'), { kind: 'family', key: 'f:anim/gsap', id: 'anim/gsap', name: 'gsap', count: 3, open: false, hint: 'a、b、c' });
  const opened = skillRows({ skills, familyOpen: { 'anim/gsap': true } });
  assert.ok(opened.find((r) => r.key === 's:gsap-a').nested);
  assert.equal(skillRows({ skills, groupOpen: { ship: false } }).some((r) => r.key === 's:release'), false);
});

test('列：一組太長就預設收合，點過就照使用者的', () => {
  const skills = Array.from({ length: 13 }, (_, i) => skill(`threejs-${i}`));
  assert.deepEqual(skillRows({ skills }).map((r) => r.key), ['g:3d']);
  assert.equal(skillRows({ skills, groupOpen: { '3d': true } }).length, 14);
});

test('列：有搜尋字時全部攤開，名稱和說明都比對', () => {
  const skills = [...['gsap-a', 'gsap-b', 'gsap-c', 'motion-graphics'].map((n) => skill(n)), skill('release', 'project', '發版流程')];
  assert.deepEqual(skillRows({ skills, query: 'gsap-B' }).map((r) => r.key), ['g:anim', 's:gsap-b']);
  assert.deepEqual(skillRows({ skills, query: '發版' }).map((r) => r.key), ['g:ship', 's:release']);
  assert.deepEqual(skillRows({ skills, query: 'zzz' }), []);
});

test('我的技能放到輸入框：用目前分頁的 agent 的寫法，不送出，也不貼 speclink 標籤', () => {
  const base = { name: 'release', note: '', focus: { tab: 'tab1', terminal: 't1' } };
  const plan = customPlacePlan({ ...base, agents: [agent({ key: 'pi' })], agentKey: 'pi' });
  assert.equal(plan.text, '/skill:release ');
  assert.equal(plan.labels, undefined);
  assert.equal(customPlacePlan({ ...base, agents: [agent({ key: 'codex' })], agentKey: 'codex', note: 'x\ny' }).text, '$release x y ');
  assert.equal(customPlacePlan({ ...base, agents: [], agentKey: 'claude' }).kind, 'none');
  assert.equal(customPlacePlan({ ...base, agents: [agent()], agentKey: 'pi' }).kind, 'none');
});

test('我的技能開新分頁：用選的 agent，分頁取技能的名字', () => {
  const base = { name: 'release', note: 'v1', root: REPO, known: ALL_AGENTS, space: 's1', on: null };
  const claude = customTabPlan({ ...base, agentKey: 'claude' });
  assert.equal(claude.kind, 'tab');
  assert.equal(claude.tabName, 'release');
  assert.deepEqual(claude.hc, ['run', '--bg', '--json', '--no-parent', '--cwd', REPO, '--label', 'herdr.space=s1', 'claude', '/release v1']);
  assert.deepEqual(customTabPlan({ ...base, agentKey: 'codex' }).hc.slice(-3), ['codex', '--no-daemon', '$release v1']);
  assert.deepEqual(customTabPlan({ ...base, agentKey: 'pi', on: 'vps' }).hc.slice(4, 6), ['--on', 'vps']);
  assert.equal(customTabPlan({ ...base, known: null, agentKey: 'claude' }).kind, 'none');
  assert.equal(customTabPlan({ ...base, known: [], agentKey: 'claude' }).kind, 'none');
});
