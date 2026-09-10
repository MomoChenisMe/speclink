// 文件截圖場景腳本的把關與計畫（user-documentation spec「使用者文件以截圖呈現
// 實際介面」的腳本條款、design D1／D2）。
// 這支腳本唯一會碰使用者環境的動作是搬移 desktop 的狀態目錄，因此測試一律以
// 注入的假路徑與假偵測結果驗證——本檔案不得搬移、建立或刪除任何真實目錄。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';

import {
  DEMO_CHANGES,
  DEMO_DISCUSSION,
  SHOT_LIST,
  appIsRunning,
  backupPlanFor,
  dryRunReport,
  manifestPathIn,
  pathsFor,
  prepareBackup,
  restoreStepsFor,
  stateDirsFor,
} from './docs-screenshots.mjs';

const HOME = '/fake/home';
const TMP = '/fake/tmp';

/// 假的檔案系統操作：只記錄呼叫，永不動到真實路徑。
function spyOps({ existing = [] } = {}) {
  const calls = [];
  return {
    calls,
    mutations: () => calls.filter((c) => c.op !== 'exists'),
    exists: (target) => {
      calls.push({ op: 'exists', target });
      return existing.includes(target);
    },
    mkdir: (target) => calls.push({ op: 'mkdir', target }),
    rename: (from, to) => calls.push({ op: 'rename', from, to }),
    writeFile: (target, body) => calls.push({ op: 'writeFile', target, body }),
  };
}

test('狀態目錄推導：macOS 上兩個落點都掛在注入的 home 下，key 不重複', () => {
  const dirs = stateDirsFor({ home: HOME, platform: 'darwin' });
  assert.ok(dirs.length >= 2, '設定與 WebView 兩份狀態都要保護');
  for (const dir of dirs) {
    assert.ok(dir.path.startsWith(`${HOME}/`), `${dir.path} 必須落在注入的 home 下`);
    assert.ok(dir.label.length > 0, '每個落點要說明它裝的是什麼');
  }
  assert.equal(new Set(dirs.map((d) => d.key)).size, dirs.length, 'key 不可重複');
});

test('狀態目錄推導：涵蓋設定目錄與 WebView 資料目錄兩處', () => {
  const paths = stateDirsFor({ home: HOME, platform: 'darwin' }).map((d) => d.path);
  // 連線設定在 appConfigDir；workspace 分頁存在 WebView 的 localStorage，
  // 兩者不同層，只搬其中一個會讓截圖仍帶著使用者的真實 workspace 名稱。
  assert.ok(
    paths.some((p) => p.includes('Application Support')),
    '要涵蓋 appConfigDir（connections.json）',
  );
  assert.ok(paths.some((p) => p.includes('WebKit')), '要涵蓋 WebView 資料目錄（分頁 localStorage）');
});

test('狀態目錄推導：非 macOS 明確拒絕並點名平台', () => {
  assert.throws(() => stateDirsFor({ home: HOME, platform: 'linux' }), /linux/);
  assert.throws(() => stateDirsFor({ home: HOME, platform: 'win32' }), /win32/);
});

test('工作路徑推導：備份落在 home（可跨重開機救回）、示範 workspace 落在暫存區', () => {
  const { backupRoot, workspace } = pathsFor({ home: HOME, tmpdir: TMP });
  assert.ok(backupRoot.startsWith(`${HOME}/`), `備份路徑 ${backupRoot} 要在 home 下`);
  assert.ok(workspace.startsWith(`${TMP}/`), `示範 workspace ${workspace} 要在暫存區`);
  assert.notEqual(backupRoot, workspace);
});

test('備份計畫：每個來源有互不相同的落點，全部收在備份根目錄下', () => {
  const stateDirs = stateDirsFor({ home: HOME, platform: 'darwin' });
  const backupRoot = `${HOME}/.backup`;
  const plan = backupPlanFor({ stateDirs, backupRoot, exists: () => true });

  assert.equal(plan.length, stateDirs.length);
  assert.equal(new Set(plan.map((e) => e.backup)).size, plan.length, '落點不可互相覆蓋');
  for (const entry of plan) {
    assert.equal(path.dirname(entry.backup), backupRoot);
    assert.equal(entry.existed, true);
  }
});

test('備份計畫：existed 逐項反映來源是否真的存在', () => {
  const stateDirs = [
    { key: 'a', label: 'A', path: '/fake/a' },
    { key: 'b', label: 'B', path: '/fake/b' },
  ];
  const plan = backupPlanFor({
    stateDirs,
    backupRoot: '/fake/backup',
    exists: (target) => target === '/fake/a',
  });
  assert.deepEqual(
    plan.map((e) => [e.key, e.existed]),
    [
      ['a', true],
      ['b', false],
    ],
  );
});

test('還原步驟：原本存在的搬回，原本不存在的刪除而非搬回', () => {
  const steps = restoreStepsFor([
    { key: 'a', source: '/fake/a', backup: '/fake/backup/a', existed: true },
    { key: 'b', source: '/fake/b', backup: '/fake/backup/b', existed: false },
  ]);
  assert.deepEqual(
    steps.map((s) => [s.key, s.action]),
    [
      ['a', 'restore'],
      ['b', 'discard'],
    ],
  );
  // discard 這條不可帶著備份路徑走搬回分支——原本就沒有備份可搬。
  assert.equal(steps[1].backup, undefined);
});

test('app 偵測：pgrep 找到即執行中，沒找到即未執行', () => {
  assert.equal(appIsRunning({ status: 0 }), true);
  assert.equal(appIsRunning({ status: 1 }), false);
});

test('app 偵測：pgrep 本身失敗不可當成「沒在跑」', () => {
  assert.throws(() => appIsRunning({ status: 2 }), /pgrep|偵測/);
  assert.throws(() => appIsRunning({ error: new Error('ENOENT') }), /ENOENT|pgrep|偵測/);
  assert.throws(() => appIsRunning({ status: null, signal: 'SIGKILL' }), /SIGKILL|pgrep|偵測/);
});

test('備份把關：偵測到 app 執行中即以錯誤收場，且不搬移任何目錄', () => {
  const ops = spyOps();
  assert.throws(
    () =>
      prepareBackup({
        platform: 'darwin',
        home: HOME,
        backupRoot: `${HOME}/.backup`,
        isRunning: true,
        ops,
      }),
    /執行中|結束/,
  );
  assert.deepEqual(ops.mutations(), [], '拒絕的路徑上不得有任何建立或搬移');
});

test('備份把關：備份路徑已存在（上一次沒還原）即拒絕，不覆蓋既有備份', () => {
  const backupRoot = `${HOME}/.backup`;
  const ops = spyOps({ existing: [backupRoot] });
  assert.throws(
    () => prepareBackup({ platform: 'darwin', home: HOME, backupRoot, isRunning: false, ops }),
    (error) => {
      assert.match(error.message, /--restore/, '要告訴使用者怎麼救回上一次的備份');
      assert.ok(error.message.includes(backupRoot), '要點名備份路徑');
      return true;
    },
  );
  assert.deepEqual(ops.mutations(), [], '拒絕的路徑上不得有任何建立或搬移');
});

test('備份把關：非 macOS 即拒絕，不搬移任何目錄', () => {
  const ops = spyOps();
  assert.throws(
    () =>
      prepareBackup({
        platform: 'linux',
        home: HOME,
        backupRoot: `${HOME}/.backup`,
        isRunning: false,
        ops,
      }),
    /linux/,
  );
  assert.deepEqual(ops.mutations(), [], '拒絕的路徑上不得有任何建立或搬移');
});

test('備份執行：只搬移確實存在的來源，並寫下供 --restore 判讀的清單', () => {
  const backupRoot = `${HOME}/.backup`;
  const stateDirs = stateDirsFor({ home: HOME, platform: 'darwin' });
  const ops = spyOps({ existing: [stateDirs[0].path] });

  const plan = prepareBackup({
    platform: 'darwin',
    home: HOME,
    backupRoot,
    isRunning: false,
    ops,
  });

  const renames = ops.calls.filter((c) => c.op === 'rename');
  assert.deepEqual(renames, [{ op: 'rename', from: stateDirs[0].path, to: plan[0].backup }]);

  const manifest = ops.calls.find((c) => c.op === 'writeFile');
  assert.equal(manifest.target, manifestPathIn(backupRoot));
  assert.deepEqual(JSON.parse(manifest.body).entries, plan);
});

test('示範內容：提案中、進行中、已封存三態齊備，另有一則討論', () => {
  const states = DEMO_CHANGES.map((c) => c.state);
  for (const state of ['proposed', 'in-progress', 'archived']) {
    assert.ok(states.includes(state), `示範內容缺少 ${state} 的變更，看板會有空欄`);
  }
  assert.equal(DEMO_DISCUSSION.status, 'concluded');
  assert.ok(DEMO_DISCUSSION.rounds.length >= 1, '討論頁要有回合內容才不是空的');
  assert.ok(DEMO_DISCUSSION.conclusion.length > 0);
});

test('示範內容：每個變更都是 kebab-case 並帶齊四份 artifact', () => {
  for (const change of DEMO_CHANGES) {
    assert.match(change.name, /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/, `${change.name} 不是 kebab-case`);
    for (const key of ['proposal', 'design', 'tasks', 'spec']) {
      assert.ok(change.artifacts[key]?.trim().length > 0, `${change.name} 缺 ${key}`);
    }
    assert.match(change.artifacts.tasks, /- \[ \] /, '任務清單要有可勾選的項目');
  }
});

test('示範內容：已封存的變更帶 Purpose（archive 對新 capability 的硬需求）', () => {
  for (const change of DEMO_CHANGES.filter((c) => c.state === 'archived')) {
    assert.match(change.artifacts.spec, /^## Purpose$/m, `${change.name} 少了 Purpose，archive 會拒絕`);
  }
});

test('示範內容：進行中的變更只完成一部分任務，卡片才看得到進度', () => {
  for (const change of DEMO_CHANGES.filter((c) => c.state === 'in-progress')) {
    const total = change.artifacts.tasks.match(/^- \[ \] /gm)?.length ?? 0;
    assert.ok(change.completedTasks > 0, `${change.name} 應有已完成任務`);
    assert.ok(change.completedTasks < total, `${change.name} 不該全部完成（那是已封存的形狀）`);
  }
});

test('待拍清單：九張截圖，檔名與存放位置固定（design D2）', () => {
  assert.equal(SHOT_LIST.length, 9);
  assert.equal(new Set(SHOT_LIST.map((s) => s.file)).size, 9, '檔名不可重複');
  for (const shot of SHOT_LIST) {
    assert.match(shot.file, /^(desktop|server)-[a-z-]+\.png$/, `${shot.file} 命名不符`);
    assert.ok(shot.what.length > 0, `${shot.file} 要說明拍什麼`);
  }
});

test('dry-run 報告：印出備份來源與落點、示範內容與待拍清單，且不觸發任何搬移', () => {
  const backupRoot = `${HOME}/.backup`;
  const workspace = `${TMP}/workspace`;
  const stateDirs = stateDirsFor({ home: HOME, platform: 'darwin' });
  const plan = backupPlanFor({ stateDirs, backupRoot, exists: () => true });
  const report = dryRunReport({ plan, workspace }).join('\n');

  for (const entry of plan) {
    assert.ok(report.includes(entry.source), `報告要點名來源 ${entry.source}`);
    assert.ok(report.includes(entry.backup), `報告要點名落點 ${entry.backup}`);
  }
  assert.ok(report.includes(workspace), '報告要點名示範 workspace 路徑');
  for (const change of DEMO_CHANGES) {
    assert.ok(report.includes(change.name), `報告要列出示範變更 ${change.name}`);
  }
  assert.ok(report.includes(DEMO_DISCUSSION.topic), '報告要列出示範討論');
  for (const shot of SHOT_LIST) {
    assert.ok(report.includes(shot.file), `報告要列出待拍的 ${shot.file}`);
  }
});
