// Herdr 面板的純邏輯（integrations/herdr/mods/speclink/model.js）：CLI 輸出怎麼變成
// 面板的列、每一列有哪些技能鈕、技能要送給誰。畫面與 Herdr API 不在這裡測。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  ALL_SKILLS,
  PICKER_SKILLS,
  attachPlan,
  batchPlan,
  buildBoard,
  changeActions,
  changeAgents,
  changeSkills,
  commandText,
  deliveryHint,
  discussionActions,
  discussionSkills,
  inside,
  isUninitialized,
  joinAgents,
  newTabPlan,
  placePlan,
  planDelivery,
  sendArgs,
  skillTargets,
  speclinkUrl,
  startAction,
  titleChange,
  waveToStart,
} from '../../integrations/herdr/mods/speclink/model.js';

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

test('changeSkills：提案中且沒被擋，建議 apply', () => {
  const skills = changeSkills(row({ stage: 'proposed' }));
  assert.deepEqual(names(skills), ['apply', 'analyze', 'ingest']);
  assert.deepEqual(primary(skills), ['apply']);
});

test('changeSkills：被擋時 apply 要等前一個變更，也不算建議', () => {
  const skills = changeSkills(row({ stage: 'proposed', blockedBy: ['add-z'] }));
  assert.deepEqual(skills.find((s) => s.skill === 'apply').waitingFor, ['add-z']);
  assert.deepEqual(primary(skills), []);
});

test('changeSkills：進行中建議 apply', () => {
  const skills = changeSkills(row());
  assert.deepEqual(names(skills), ['apply', 'ingest', 'drift', 'verify']);
  assert.deepEqual(primary(skills), ['apply']);
});

test('changeSkills：已就緒且沒有 worktree 可封存，建議 quality', () => {
  const skills = changeSkills(row({ stage: 'ready' }));
  assert.deepEqual(names(skills), ['quality', 'review', 'verify', 'archive', 'commit']);
  assert.deepEqual(primary(skills), ['quality']);
});

test('changeSkills：已就緒且有 worktree 要先合併，不能封存', () => {
  const skills = changeSkills(row({ stage: 'ready', worktree: { path: TREE, branch: 'b' } }));
  assert.deepEqual(names(skills), ['quality', 'review', 'verify', 'commit', 'worktree-merge']);
});

test('changeSkills：討論重新定案時建議改成 ingest，沒有 ingest 的階段也會加上', () => {
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

test('titleChange：讀 Claude Code 外掛設的標題，步驟與變更之間是「 · 」', () => {
  assert.equal(titleChange('apply · add-a'), 'add-a');
  assert.equal(titleChange('⠐ apply · add-a'), 'add-a');
  assert.equal(titleChange('archive · add-a ✓'), 'add-a');
  assert.equal(titleChange('Claude Code'), null);
  assert.equal(titleChange(''), null);
});

test('inside：同資料夾與子資料夾算在內，同前綴的兄弟資料夾不算', () => {
  assert.equal(inside(REPO, REPO), true);
  assert.equal(inside(`${REPO}/crates`, REPO), true);
  assert.equal(inside(TREE, REPO), false);
  assert.equal(inside(null, REPO), false);
});

test('changeAgents：依標籤、worktree 路徑、終端機標題的順序，同一個 agent 不重複', () => {
  const byTitle = agent({ address: 't3', title: 'apply · add-a' });
  const byPath = agent({ address: 't2', cwd: TREE });
  const byLabel = agent({ address: 't1', labels: { 'speclink.change': 'add-a' }, title: 'apply · add-a' });
  const other = agent({ address: 't4', title: 'apply · add-z' });
  const found = changeAgents(row({ worktree: { path: TREE, branch: 'b' } }), [byTitle, other, byPath, byLabel]);
  assert.deepEqual(
    found.map((a) => a.address),
    ['t1', 't2', 't3'],
  );
});

test('changeAgents：討論或提案的標題只是提到變更，不算在做', () => {
  const change = row({ name: 'add-a' });
  const talk = agent({ address: 't1', title: 'discuss · add-a' });
  const propose = agent({ address: 't2', title: 'propose · add-a' });
  const work = agent({ address: 't3', title: 'apply · add-a' });
  assert.deepEqual(changeAgents(change, [talk, propose, work]).map((a) => a.address), ['t3']);
});

test('changeAgents：外掛回報的焦點優先於標題，回報「沒有變更」時也不看標題', () => {
  const change = row({ name: 'add-a' });
  const moved = agent({ address: 't1', title: 'apply · add-a', labels: { 'speclink.focus': 'add-b' } });
  const none = agent({ address: 't2', title: 'apply · add-a', labels: { 'speclink.focus': '-' } });
  const here = agent({ address: 't3', title: 'propose · add-b', labels: { 'speclink.focus': 'add-a' } });
  assert.deepEqual(changeAgents(change, [moved, none, here]).map((a) => a.address), ['t3']);
});

test('changeAgents：開工時貼了標籤的 agent 專做那個變更，焦點提到別的變更不算', () => {
  const change = row({ name: 'add-a' });
  const owned = agent({ address: 't1', labels: { 'speclink.change': 'add-b', 'speclink.focus': 'add-a' } });
  assert.deepEqual(changeAgents(change, [owned]), []);
});

test('送達：這個變更的 agent 在做事時，不會把技能送給只是提到它的討論分頁（實際發生過）', () => {
  const change = row({ name: 'add-a' });
  const apply = agent({ address: 't1', status: 'working', title: 'apply · add-a', labels: { 'speclink.change': 'add-a' } });
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

test('有 worktree 的變更：標籤或標題標明的 agent 即使資料夾還在主資料夾也算（agent 用 cd 進 worktree）', () => {
  const titled = agent({ title: 'apply-with-worktree · add-a' });
  assert.equal(treeCase({ agents: [titled] }).where, 'change');
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
  assert.deepEqual(names(codex), ['apply', 'ingest']);
  const claude = changeActions(row({ stage: 'proposed' }), ctx({ agents: [agent()] }));
  assert.deepEqual(names(claude), ['apply', 'analyze', 'ingest']);
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
  const running = agent({ labels: { 'speclink.skill': 'speclink-apply' } });
  assert.equal(tab({ skill: 'apply', agents: [running] }).reason, '主資料夾已經有 apply 在跑');
  assert.equal(tab({ skill: 'apply-with-worktree', agents: [running] }).kind, 'tab');
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

// ---- 開工 ----

const start = (over = {}, ctxOver = {}) => startAction(row(over), ctx({ worktreePolicy: true, ...ctxOver }));

test('開工：開 worktree 政策時開新 space，跑 apply-with-worktree，開在主資料夾', () => {
  const { delivery: d } = start({ stage: 'proposed', completed: 0 });
  assert.equal(d.kind, 'space');
  assert.equal(d.folder, REPO);
  assert.equal(d.text, '/speclink-apply-with-worktree add-a');
  assert.equal(d.tabName, 'apply-with-worktree · add-a');
  assert.equal(d.change, 'add-a');
  assert.equal(d.skill, 'apply-with-worktree');
  assert.deepEqual(d.argv, ['claude']);
});

test('開工：未開 worktree 政策時改開新分頁，跑 apply', () => {
  const { delivery: d } = start({}, { worktreePolicy: false });
  assert.equal(d.kind, 'new');
  assert.equal(d.folder, REPO);
  assert.equal(d.text, '/speclink-apply add-a');
});

test('開工：已有 worktree 但沒有 agent 時也從主資料夾接續', () => {
  const { delivery: d } = start({ worktree: { path: TREE, branch: 'speclink/add-a' } });
  assert.equal(d.kind, 'space');
  assert.equal(d.folder, REPO);
  assert.equal(d.text, '/speclink-apply-with-worktree add-a');
});

test('開工：Codex 用錢字號並帶 --no-daemon', () => {
  const { delivery: d } = start({}, { known: [{ key: 'codex', run: ['codex'], installed: true }] });
  assert.equal(d.text, '$speclink-apply-with-worktree add-a');
  assert.deepEqual(d.argv, ['codex', '--no-daemon']);
});

test('開工：沒有可用的 agent 時鈕還在，並說明原因', () => {
  assert.equal(deliveryHint(start({}, { known: null }).delivery), '還在確認可用的 agent');
  assert.equal(start({}, { known: [] }).delivery.kind, 'none');
});

test('開工：這些情況沒有開工鈕', () => {
  assert.equal(start({ stage: 'ready' }), null, '任務都做完了');
  assert.equal(start({ blockedBy: ['add-z'] }), null, '被擋住');
  assert.equal(start({ restale: true }), null, '討論重新定案，要先 ingest');
  assert.equal(start({}, { agents: [agent({ labels: { 'speclink.change': 'add-a' } })] }), null, '已有 agent 在做');
  assert.equal(start({}, { starting: new Set(['add-a']) }), null, '剛按過，agent 還沒開好');
});

test('deliveryHint：開新 space', () => {
  assert.equal(deliveryHint(start().delivery), '開新 space');
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
    wave.deliveries.map((d) => d.text),
    ['/speclink-apply-with-worktree add-a', '/speclink-apply-with-worktree add-b'],
  );
  assert.ok(wave.deliveries.every((d) => d.kind === 'space'));
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
  assert.deepEqual(wave.deliveries.map((d) => d.change), ['add-a', 'add-b']);
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
