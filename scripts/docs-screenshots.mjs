#!/usr/bin/env node
// 文件截圖的場景佈置（user-documentation spec「使用者文件以截圖呈現實際介面」、
// design D1／D2）。本腳本不按快門——版面與時機交給人判斷，腳本只負責兩件人做
// 容易出錯的事：把使用者的 desktop 狀態安全地移開再放回，以及造出一個內容固定
// 的示範 workspace。
//
// 用法：
//   node scripts/docs-screenshots.mjs --dry-run   只印出會搬什麼、會造什麼，不動任何檔案
//   node scripts/docs-screenshots.mjs --setup     備份 → 佈置 → 等你拍完按 Enter → 還原
//   node scripts/docs-screenshots.mjs --restore   單獨還原（--setup 中途當掉時的救援入口）
//
// desktop 沒有資料目錄隔離：分頁與連線都存在使用者層級的狀態目錄，dev 與安裝版
// 共用。要拍到乾淨的全新狀態只能整個移開再放回，因此還原在 Ctrl+C 下也必須執行。
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { checkoutCliPath } from './cli.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/// tauri.conf.json 的 identifier；macOS 的兩處狀態落點都以它命名。
export const APP_IDENTIFIER = 'app.speclink.desktop';

/// 拍攝期間必須整個移開的使用者狀態。兩處分屬不同層，缺一不可：
/// - appConfigDir：connections.json（Server 連線清單）
/// - WebView 資料目錄：localStorage（speclink.projectTabs＝workspace 分頁、介面偏好）
///   分頁不在 appConfigDir——那裡的 tabs.json 是七月的殘留、現行程式碼已不再讀寫，
///   只搬 appConfigDir 的話截圖仍會帶著使用者真實的 workspace 名稱。
export function stateDirsFor({ home, platform }) {
  if (platform !== 'darwin') {
    throw new Error(`截圖場景腳本目前只支援 macOS，目前平台為 ${platform}`);
  }
  return [
    {
      key: 'app-config',
      label: '設定目錄（Server 連線清單）',
      path: path.posix.join(home, 'Library', 'Application Support', APP_IDENTIFIER),
    },
    {
      key: 'webview-data',
      label: 'WebView 資料目錄（workspace 分頁、介面偏好）',
      path: path.posix.join(home, 'Library', 'WebKit', APP_IDENTIFIER),
    },
  ];
}

/// 腳本自建的兩個工作路徑。備份刻意放在 home 而非暫存區——暫存區可能被系統清掉
/// 或在重開機後消失，而備份沒還原就等於使用者的分頁與連線設定消失。
export function pathsFor({ home, tmpdir }) {
  return {
    backupRoot: path.posix.join(home, '.speclink-docs-screenshots-backup'),
    workspace: path.posix.join(tmpdir, 'speclink-docs-screenshots-workspace'),
  };
}

/// 備份清單的落點（--restore 靠它知道哪些目錄「原本就不存在」）。
export function manifestPathIn(backupRoot) {
  return path.posix.join(backupRoot, 'manifest.json');
}

/// 每個狀態目錄對應一個以 key 命名的備份落點——兩處的目錄名相同，用 basename
/// 會互相覆蓋。
export function backupPlanFor({ stateDirs, backupRoot, exists }) {
  return stateDirs.map((dir) => ({
    key: dir.key,
    label: dir.label,
    source: dir.path,
    backup: path.posix.join(backupRoot, dir.key),
    existed: exists(dir.path),
  }));
}

/// 還原步驟：拍攝期間 app 會自行重建狀態目錄，所以兩種情況都要先刪掉現場的目錄，
/// 差別只在後面搬不搬得回來。原本不存在的不帶備份路徑——沒有東西可搬回。
export function restoreStepsFor(entries) {
  return entries.map((entry) =>
    entry.existed
      ? { key: entry.key, source: entry.source, backup: entry.backup, action: 'restore' }
      : { key: entry.key, source: entry.source, action: 'discard' },
  );
}

/// pgrep 的 exit code 語意：0＝有符合、1＝沒有符合、其餘＝pgrep 本身失敗。
/// 失敗絕不可當成「沒在跑」——那會讓備份在 app 執行中開始，還原結果被 app 退出時
/// 寫回的記憶體狀態覆蓋掉。
export function appIsRunning(result) {
  if (result.error) {
    throw new Error(`無法確認 desktop app 是否執行中（pgrep：${result.error.message}）`);
  }
  if (result.status === 0) return true;
  if (result.status === 1) return false;
  const how = result.status === null ? `signal ${result.signal}` : `exit ${result.status}`;
  throw new Error(`無法確認 desktop app 是否執行中（pgrep 以 ${how} 結束）`);
}

/// 備份：拒絕條件全部在任何搬移之前判定，被拒絕的路徑上一個檔案都不會動。
export function prepareBackup({ platform, home, backupRoot, isRunning, ops }) {
  const stateDirs = stateDirsFor({ home, platform });

  if (isRunning) {
    throw new Error(
      'Speclink desktop 正在執行——請先完全結束 app 再重跑（不代為結束 app：你可能正在裡面工作）',
    );
  }

  // 備份路徑還在，代表上一次拍攝沒有還原完成。覆蓋它就等於把使用者真正的設定
  // 換成拍攝期間的空白狀態，這是本腳本最不能犯的錯。
  if (ops.exists(backupRoot)) {
    throw new Error(
      `備份路徑已存在：${backupRoot}\n上一次拍攝沒有還原完成。先執行 --restore 把它搬回原位，再重跑 --setup。`,
    );
  }

  const plan = backupPlanFor({ stateDirs, backupRoot, exists: ops.exists });
  ops.mkdir(backupRoot);
  for (const entry of plan) {
    if (entry.existed) ops.rename(entry.source, entry.backup);
  }
  ops.writeFile(manifestPathIn(backupRoot), `${JSON.stringify({ entries: plan }, null, 2)}\n`);
  return plan;
}

/// 待拍清單（design D2）。中英兩版共用同一組圖片檔。
export const SHOT_LIST = [
  { file: 'desktop-board.png', what: '變更看板三欄與卡片' },
  { file: 'desktop-change-drawer.png', what: '變更詳情抽屜（含任務與 artifacts）' },
  { file: 'desktop-spec.png', what: '規格檢視' },
  { file: 'desktop-discussion.png', what: '討論記錄檢視' },
  { file: 'desktop-archived.png', what: '已封存檢視' },
  { file: 'desktop-settings.png', what: '設定（含 Server 連線）' },
  { file: 'server-setup.png', what: 'server 首次 /setup 畫面' },
  { file: 'server-overview.png', what: '後台總覽' },
  { file: 'server-members.png', what: '成員與 PAT 管理' },
];

/// 截圖存放位置（相對 repo root）。
export const SHOT_DIR = 'docs/assets/screenshots';

const TEAM_INVITES = {
  name: 'add-team-invites',
  state: 'proposed',
  capability: 'team-invites',
  description: 'Let team leads invite members by email',
  completedTasks: 0,
  artifacts: {
    proposal: `## Why

Adding someone to a shared store means asking a maintainer to run a CLI command. Team leads cannot grow their own team, so onboarding stalls on whoever holds the terminal.

## What Changes

- Send an invitation email carrying a single-use join link
- Let the lead pick the member role before the invitation goes out
- Expire unaccepted invitations so stale links cannot be redeemed

## Capabilities

### New Capabilities

- \`team-invites\`: Invite members by email and let them join without a maintainer

### Modified Capabilities

（None）

## Impact

- **Code**: server membership routes, settings UI
- **Behavior**: an invited member reaches the store without touching the CLI
`,
    design: `## Context

Membership is written directly by maintainers today. The store already knows every role, so the missing piece is a way to hand out a scoped, expiring token that a newcomer can redeem themselves.

## Goals / Non-Goals

**Goals:**
- One invitation link redeems exactly once
- The role is fixed when the invitation is issued, not when it is accepted
- An unaccepted invitation stops working on its own

**Non-Goals:**
- Bulk import from an external directory
- Self-service signup without an invitation

## Decisions

### Single-use tokens over shareable links

A shareable link cannot tell two people apart, so the audit trail loses who joined with which grant. Single-use tokens keep one row per member.

## Risks / Trade-offs

- A lost invitation email needs a reissue rather than a resend; accepted as the cost of single use.
`,
    tasks: `## 1. Invitation lifecycle

- [ ] 1.1 Issue an invitation with a fixed role and an expiry
- [ ] 1.2 Redeem an invitation exactly once
- [ ] 1.3 Reject expired and already-redeemed tokens

## 2. Surfaces

- [ ] 2.1 Add the invite form to the members screen
- [ ] 2.2 Show pending invitations with their expiry
`,
    spec: `## Purpose

Invite members to a shared store by email so a team lead can grow the team without a maintainer running CLI commands, with every grant traceable to a single redeemed invitation.

## ADDED Requirements

### Requirement: Invitation issuing

The system SHALL issue an invitation bound to one email address, one role and one expiry moment.

#### Scenario: Role is fixed at issue time
- **WHEN** a team lead issues an invitation with the editor role
- **THEN** the redeemed membership SHALL carry the editor role regardless of when it is accepted

### Requirement: Single redemption

The system SHALL allow each invitation to be redeemed at most once.

#### Scenario: Second redemption is refused
- **WHEN** an already-redeemed invitation link is opened again
- **THEN** the system SHALL refuse the redemption and SHALL NOT create a second membership
`,
  },
};

const SEARCH_RANKING = {
  name: 'improve-search-ranking',
  state: 'in-progress',
  capability: 'search-ranking',
  description: 'Rank search results by relevance instead of file order',
  completedTasks: 2,
  artifacts: {
    proposal: `## Why

Search walks files in directory order, so the match you want is rarely the first one you see. On a store with a few hundred specs, people scroll instead of read.

## What Changes

- Score matches by field weight, with titles above body text
- Break ties by recency so live work floats above archived work
- Show why a result matched

## Capabilities

### New Capabilities

- \`search-ranking\`: Order search results by relevance rather than by file order

### Modified Capabilities

（None）

## Impact

- **Code**: search index, results list
- **Behavior**: the first result is the one people actually open
`,
    design: `## Context

The index already stores every field separately, so ranking needs no new data — only a scoring pass between the match and the result list.

## Goals / Non-Goals

**Goals:**
- A title match outranks a body match for the same term
- Two equally scored results order by most recently touched
- The reason for a match is visible in the result row

**Non-Goals:**
- Fuzzy or phonetic matching
- Learning from click behaviour

## Decisions

### Static field weights over a learned model

Weights are readable and arguable in review; a learned model would need click data the product does not collect.

## Risks / Trade-offs

- Weights need retuning when new fields join the index; the ranking test suite pins the current expectations.
`,
    tasks: `## 1. Scoring

- [ ] 1.1 Score matches per field with static weights
- [ ] 1.2 Break score ties by last-touched time
- [ ] 1.3 Pin the ordering with a ranking test suite

## 2. Result list

- [ ] 2.1 Show the matched field on each result row
- [ ] 2.2 Keep keyboard navigation on the ranked order
`,
    spec: `## Purpose

Order search results by how well they match rather than by where the file sits on disk, so the most relevant spec, change or discussion is the first one a reader sees.

## ADDED Requirements

### Requirement: Relevance ordering

The system SHALL order search results by a relevance score computed from per-field weights.

#### Scenario: Title match outranks body match
- **WHEN** one document matches a term in its title and another matches the same term in its body
- **THEN** the title match SHALL appear above the body match

### Requirement: Stable tie-breaking

The system SHALL order equally scored results by the most recently touched document first.

#### Scenario: Equal scores order by recency
- **WHEN** two results carry the same relevance score
- **THEN** the more recently touched document SHALL appear first
`,
  },
};

const DARK_MODE = {
  name: 'add-dark-mode',
  state: 'archived',
  capability: 'dark-mode',
  description: 'Follow the system appearance and offer a manual override',
  completedTasks: null,
  artifacts: {
    proposal: `## Why

The app is bright white at every hour. People working in a dark room either squint or leave the app off screen.

## What Changes

- Follow the operating system appearance by default
- Offer a manual light or dark override in settings
- Keep the chosen appearance across restarts

## Capabilities

### New Capabilities

- \`dark-mode\`: Follow the system appearance with a manual override

### Modified Capabilities

（None）

## Impact

- **Code**: theme tokens, settings screen
- **Behavior**: the window matches the system appearance on first launch
`,
    design: `## Context

Colours are already expressed as tokens, so a second palette is a token swap rather than a restyle of every component.

## Goals / Non-Goals

**Goals:**
- First launch matches the system appearance with no configuration
- A manual override wins over the system setting until it is cleared
- Switching appearance does not reload the window

**Non-Goals:**
- User-authored themes
- Per-workspace appearance

## Decisions

### Token swap over a parallel stylesheet

Two stylesheets drift apart the moment a component ships a one-off colour; one token set keeps both appearances honest.

## Risks / Trade-offs

- Contrast has to be checked twice, once per appearance; covered by the token contrast test.
`,
    tasks: `## 1. Palette

- [ ] 1.1 Add the dark values to the colour token set
- [ ] 1.2 Check contrast in both appearances

## 2. Selection

- [ ] 2.1 Follow the system appearance on first launch
- [ ] 2.2 Persist the manual override across restarts
`,
    spec: `## Purpose

Render the desktop app in the appearance the reader expects: matching the operating system by default and honouring a manual light or dark choice, with the choice surviving a restart.

## ADDED Requirements

### Requirement: System appearance by default

The system SHALL adopt the operating system appearance when no manual override is set.

#### Scenario: First launch in a dark system
- **WHEN** the app launches with no stored appearance preference on a system set to dark
- **THEN** the window SHALL render in the dark appearance

### Requirement: Manual override wins

The system SHALL honour a manual appearance choice over the operating system setting until the choice is cleared.

#### Scenario: Override survives a restart
- **WHEN** a reader picks the light appearance and restarts the app
- **THEN** the window SHALL render in the light appearance even if the system is set to dark
`,
  },
};

/// 示範變更：三種看板欄位各一，內容全部由本腳本產生，不引用任何真實資料。
export const DEMO_CHANGES = [TEAM_INVITES, SEARCH_RANKING, DARK_MODE];

/// 示範討論：已結論，讓討論頁不是空的。
export const DEMO_DISCUSSION = {
  slug: 'background-exports',
  topic: 'Should large exports run in the background?',
  status: 'concluded',
  context: 'Exporting a large store freezes the window until it finishes, so people assume the app has crashed and force-quit it.\n',
  rounds: [
    'Option A: run the export as a background job and report progress in a toast.\nOption B: keep it blocking but refuse exports above a size cap.\n\nOption B is cheaper, but the cap has to be guessed and the people it blocks are exactly the ones who need the export most.\n',
  ],
  conclusion:
    'Go with Option A. A size cap only moves the pain onto the largest stores, and the progress toast is reusable for the import path later.\n',
};

/// 示範用的 git 身分：示範 workspace 內的 .openspec.yaml 會記下 created_by，
/// 沿用使用者的全域 git 設定就會把真名與 email 拍進截圖裡。
const DEMO_IDENTITY = { name: 'Speclink Demo', email: 'demo@speclink.invalid' };

export function dryRunReport({ plan, workspace }) {
  const lines = ['[dry-run] 不會搬移、建立或刪除任何檔案。', '', '將備份的狀態目錄：'];
  for (const entry of plan) {
    const note = entry.existed ? '' : '（目前不存在，還原階段改為刪除拍攝期間產生的目錄）';
    lines.push(`  - ${entry.label}${note}`);
    lines.push(`      來源：${entry.source}`);
    lines.push(`      落點：${entry.backup}`);
  }
  lines.push('', `將建立的示範 workspace：${workspace}`, '', '示範內容：');
  for (const change of DEMO_CHANGES) {
    lines.push(`  - [${STATE_LABEL[change.state]}] ${change.name} — ${change.description}`);
  }
  lines.push(`  - [已結論的討論] ${DEMO_DISCUSSION.topic}`);
  lines.push('', `待拍清單（存入 ${SHOT_DIR}/）：`);
  for (const shot of SHOT_LIST) {
    lines.push(`  - ${shot.file} — ${shot.what}`);
  }
  return lines;
}

const STATE_LABEL = { proposed: '提案中', 'in-progress': '進行中', archived: '已封存' };

// ---------------------------------------------------------------------------
// 以下為實際執行路徑（測試以注入的假 ops 覆蓋上方的純函式，不進入這一段）
// ---------------------------------------------------------------------------

const realOps = {
  exists: existsSync,
  mkdir: (target) => mkdirSync(target, { recursive: true }),
  rename: renameSync,
  writeFile: (target, body) => writeFileSync(target, body),
};

function detectAppRunning() {
  // 涵蓋安裝版與 dev 兩種執行形態：兩者的執行檔名相同，只有路徑不同。
  return appIsRunning(spawnSync('pgrep', ['-x', 'speclink-desktop']));
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', ...options });
  if (result.error) {
    throw new Error(`${command} ${args.join(' ')} 無法執行：${result.error.message}`);
  }
  if (result.status !== 0) {
    const how = result.status === null ? `signal ${result.signal}` : result.status;
    throw new Error(`${command} ${args.join(' ')} 以非零結束（${how}）`);
  }
}

function cli(workspace, args, options = {}) {
  const binary = checkoutCliPath();
  if (!existsSync(binary)) {
    throw new Error(`找不到 checkout 的 CLI：${binary}——先執行 cargo build -p speclink-cli`);
  }
  run(binary, args, { cwd: workspace, ...options });
}

function cliJson(workspace, args) {
  const binary = checkoutCliPath();
  const result = spawnSync(binary, args, { cwd: workspace, encoding: 'utf8' });
  if (result.error || result.status !== 0) {
    throw new Error(`speclink ${args.join(' ')} 失敗：${result.error?.message ?? result.stderr}`);
  }
  return JSON.parse(result.stdout);
}

/// 造出示範 workspace。全新目錄——既有內容一律先清掉，避免上一次拍攝的殘留混進畫面。
/// 對外開放是為了能單獨驗證示範內容，不必連帶搬動使用者的狀態目錄。
export function createDemoWorkspace(workspace) {
  rmSync(workspace, { recursive: true, force: true });
  mkdirSync(workspace, { recursive: true });

  run('git', ['init', '-q', '.'], { cwd: workspace });
  // local 設定蓋過全域：created_by 會被寫進每個 .openspec.yaml。
  run('git', ['config', 'user.name', DEMO_IDENTITY.name], { cwd: workspace });
  run('git', ['config', 'user.email', DEMO_IDENTITY.email], { cwd: workspace });

  cli(workspace, ['init', '--tools', 'claude']);

  for (const change of DEMO_CHANGES) {
    cli(workspace, ['new', 'change', change.name, '--description', change.description]);
    writeChangeArtifacts(workspace, change);

    if (change.state === 'proposed') continue;

    cli(workspace, ['in-progress', 'add', change.name]);
    const ids = cliJson(workspace, ['instructions', 'apply', '--change', change.name, '--json']).tasks.map(
      (task) => task.id,
    );
    const done = change.completedTasks ?? ids.length;
    for (const id of ids.slice(0, done)) {
      cli(workspace, ['task', 'done', '--change', change.name, id]);
    }
    if (change.state === 'archived') cli(workspace, ['archive', change.name]);
  }

  const { slug, topic, context, rounds, conclusion } = DEMO_DISCUSSION;
  cli(workspace, ['discuss', 'new', topic, '--slug', slug]);
  cli(workspace, ['discuss', 'context', slug], { input: context, stdio: ['pipe', 'inherit', 'inherit'] });
  for (const round of rounds) {
    cli(workspace, ['discuss', 'add-round', slug], { input: round, stdio: ['pipe', 'inherit', 'inherit'] });
  }
  cli(workspace, ['discuss', 'conclude', slug], {
    input: conclusion,
    stdio: ['pipe', 'inherit', 'inherit'],
  });
}

function writeChangeArtifacts(workspace, change) {
  const dir = path.join(workspace, 'openspec', 'changes', change.name);
  writeFileSync(path.join(dir, 'proposal.md'), change.artifacts.proposal);
  writeFileSync(path.join(dir, 'design.md'), change.artifacts.design);
  writeFileSync(path.join(dir, 'tasks.md'), change.artifacts.tasks);
  const specDir = path.join(dir, 'specs', change.capability);
  mkdirSync(specDir, { recursive: true });
  writeFileSync(path.join(specDir, 'spec.md'), change.artifacts.spec);
}

/// 還原：先刪掉現場的目錄，再把備份搬回。任一步失敗都印出備份路徑——使用者至少
/// 知道自己的設定還完整地躺在哪裡。
function restoreAll({ backupRoot, workspace }) {
  const manifest = manifestPathIn(backupRoot);
  if (!existsSync(manifest)) {
    console.log(`沒有待還原的備份（${manifest} 不存在）。`);
    rmSync(workspace, { recursive: true, force: true });
    return;
  }

  const { entries } = JSON.parse(readFileSync(manifest, 'utf8'));
  for (const step of restoreStepsFor(entries)) {
    try {
      rmSync(step.source, { recursive: true, force: true });
      if (step.action === 'restore') renameSync(step.backup, step.source);
    } catch (error) {
      throw new Error(
        `還原 ${step.source} 失敗：${error.message}\n備份完整保留在 ${backupRoot}，手動把 ${step.backup ?? '（原本不存在）'} 搬回 ${step.source} 即可。`,
      );
    }
  }

  rmSync(manifestPathIn(backupRoot), { force: true });
  rmSync(backupRoot, { recursive: true, force: true });
  rmSync(workspace, { recursive: true, force: true });
  console.log('✓ 已還原原本的 desktop 狀態，示範 workspace 已清除。');
}

function printHandoff(workspace) {
  console.log('');
  console.log('─'.repeat(72));
  console.log('狀態已備份，示範 workspace 已就緒：');
  console.log(`  ${workspace}`);
  console.log('');
  console.log('接著請你手動完成：');
  console.log('  1. 開啟 Speclink desktop（此時它是全新狀態，沒有任何分頁與連線）');
  console.log('  2. 把上面這個路徑加入為 workspace');
  console.log(`  3. 逐張擷取並存入 ${SHOT_DIR}/：`);
  for (const shot of SHOT_LIST.filter((s) => s.file.startsWith('desktop-'))) {
    console.log(`       ${shot.file} — ${shot.what}`);
  }
  console.log('     （server 三張另外依 docs/development.zh-TW.md 啟動 server 擷取）');
  console.log('  4. 拍完先完全結束 desktop app，再回到這裡按 Enter 還原');
  console.log('');
  console.log('中途 Ctrl+C 也會還原；若腳本整個當掉，改跑 --restore。');
  console.log('─'.repeat(72));
  console.log('');
}

async function waitForEnter() {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  try {
    await new Promise((resolve) => rl.question('拍完並關閉 app 後按 Enter 還原…', resolve));
  } finally {
    rl.close();
  }
}

async function main(argv) {
  const paths = pathsFor({ home: os.homedir(), tmpdir: os.tmpdir() });

  if (argv.includes('--restore')) {
    restoreAll(paths);
    return;
  }

  if (argv.includes('--dry-run')) {
    const stateDirs = stateDirsFor({ home: os.homedir(), platform: process.platform });
    const plan = backupPlanFor({ stateDirs, backupRoot: paths.backupRoot, exists: existsSync });
    console.log(dryRunReport({ plan, workspace: paths.workspace }).join('\n'));
    return;
  }

  if (!argv.includes('--setup')) {
    throw new Error('用法：node scripts/docs-screenshots.mjs [--setup | --restore | --dry-run]');
  }

  prepareBackup({
    platform: process.platform,
    home: os.homedir(),
    backupRoot: paths.backupRoot,
    isRunning: detectAppRunning(),
    ops: realOps,
  });

  // 備份已經落地：從這一刻起，任何離開路徑都必須還原。
  let restored = false;
  const restoreOnce = () => {
    if (restored) return;
    restored = true;
    try {
      restoreAll(paths);
    } catch (error) {
      console.error(`docs-screenshots: ${error.message}`);
      process.exitCode = 1;
    }
  };
  for (const signal of ['SIGINT', 'SIGTERM']) {
    process.on(signal, () => {
      console.log(`\n收到 ${signal}，還原中…`);
      restoreOnce();
      process.exit(1);
    });
  }

  try {
    createDemoWorkspace(paths.workspace);
    printHandoff(paths.workspace);
    await waitForEnter();
  } finally {
    restoreOnce();
  }
}

// node --test 匯入本模組時只取函式，不執行任何搬移。
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`docs-screenshots: ${error.message}`);
    process.exit(1);
  });
}

export { ROOT };
