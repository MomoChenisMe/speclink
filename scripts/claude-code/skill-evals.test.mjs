// 技能評測套件的守門（skill-authoring 規格「技能評測套件」）。
// compareResults 以 inline 的 aggregate-result.json 驗證新舊比較的判定；compare 子指令以暫存
// 目錄裡的結果檔實際執行，驗證結束碼與輸出；最後一條掃過版本庫實況：`speclink update` 寫出的
// 每個 `.claude/skills/speclink-*` 技能，都要在 integrations/claude-code/skill-evals/evals/
// 至少有一題以 tags 標示 `skill:<短名>`。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { OUT_ROOT, RESULT_FILE, compareResults, evalSpawnOptions, parseRunArgs, runPaths } from './skill-evals.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const SCRIPT = path.join(ROOT, 'scripts/claude-code/skill-evals.mjs');
const EVALS_DIR = 'integrations/claude-code/skill-evals/evals';

/// aggregate-result.json 的最小形狀：題目名在 cases[].name、分數在 cases[].aggregates.score。
function aggregate(scores) {
  return { cases: Object.entries(scores).map(([name, score]) => ({ name, aggregates: { score } })) };
}

/// 在暫存目錄建立一個 repo 根目錄給 fn 用，結束後刪掉。
function withTempRoot(fn) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'skill-evals-test-'));
  try {
    return fn(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

/// 把結果文件寫成 <root>/<label>/aggregate-result.json，回傳該結果目錄。
function writeResult(root, label, doc) {
  const dir = path.join(root, label);
  mkdirSync(dir, { recursive: true });
  writeFileSync(path.join(dir, RESULT_FILE), typeof doc === 'string' ? doc : JSON.stringify(doc));
  return dir;
}

function compareCli(oldDir, newDir) {
  return spawnSync(process.execPath, [SCRIPT, 'compare', oldDir, newDir], { encoding: 'utf8' });
}

// 規格 Example「比較的判定」：一列一個 test。
for (const [name, oldScore, newScore, verdict, exitCode] of [
  ['propose-from-discussion', 0.67, 1, '進步', 0],
  ['apply-leaves-manual-task', 1, 1, '持平', 0],
  ['commit-gate-shows-plan', 1, 0.67, '退步', 1],
]) {
  test(`比較：Example「比較的判定」${name}（${verdict}）exit ${exitCode}`, () =>
    withTempRoot((root) => {
      const child = compareCli(writeResult(root, 'old', aggregate({ [name]: oldScore })), writeResult(root, 'new', aggregate({ [name]: newScore })));
      assert.equal(child.status, exitCode, child.stderr);
      assert.match(child.stdout, new RegExp(`${name}\\s+${oldScore.toFixed(2)}\\s+${newScore.toFixed(2)}`));
      assert.equal(child.stdout.includes(`退步：${name}`), verdict === '退步', child.stdout);
    }));
}

test('比較：Example 整張表——列出退步的題目，exit 1', () =>
  withTempRoot((root) => {
    const child = compareCli(
      writeResult(root, 'old', aggregate({ 'propose-from-discussion': 0.67, 'apply-leaves-manual-task': 1, 'commit-gate-shows-plan': 1 })),
      writeResult(root, 'new', aggregate({ 'propose-from-discussion': 1, 'apply-leaves-manual-task': 1, 'commit-gate-shows-plan': 0.67 })),
    );
    assert.equal(child.status, 1, child.stderr);
    assert.match(child.stdout, /退步：commit-gate-shows-plan（1\.00 → 0\.67）/);
    assert.ok(!child.stdout.includes('退步：propose-from-discussion'), child.stdout);
  }));

test('比較：每題新分數都不低於舊分數時 exit 0', () =>
  withTempRoot((root) => {
    const child = compareCli(writeResult(root, 'old', aggregate({ a: 0.5, b: 1 })), writeResult(root, 'new', aggregate({ a: 1, b: 1 })));
    assert.equal(child.status, 0, child.stderr);
    assert.match(child.stdout, /a\s+0\.50\s+1\.00\s+\+0\.50/);
  }));

test('比較：題目集合不同時在 stdout 列出缺題，exit 2', () =>
  withTempRoot((root) => {
    const child = compareCli(
      writeResult(root, 'old', aggregate({ 'trace-cites-archive': 1, 'drift-reports-only': 1 })),
      writeResult(root, 'new', aggregate({ 'drift-reports-only': 1 })),
    );
    assert.equal(child.status, 2, child.stderr);
    assert.match(child.stdout, /缺少的題目：trace-cites-archive/);
  }));

test('比較：結果檔不存在時 stderr 指名該檔，exit 3', () =>
  withTempRoot((root) => {
    const newDir = path.join(root, 'new');
    mkdirSync(newDir);
    const child = compareCli(writeResult(root, 'old', aggregate({ a: 1 })), newDir);
    assert.equal(child.status, 3);
    assert.ok(child.stderr.includes(path.join(newDir, RESULT_FILE)), child.stderr);
  }));

test('比較：結果檔不是 JSON 時 stderr 指名該檔，exit 3', () =>
  withTempRoot((root) => {
    const newDir = writeResult(root, 'new', 'not json');
    const child = compareCli(writeResult(root, 'old', aggregate({ a: 1 })), newDir);
    assert.equal(child.status, 3);
    assert.ok(child.stderr.includes(path.join(newDir, RESULT_FILE)), child.stderr);
  }));

test('比較：跑到花費上限的不完整結果不能比較——stderr 指名該檔，exit 3，stdout 不印比較', () =>
  withTempRoot((root) => {
    const skipped = aggregate({ a: 1 });
    skipped.cases[0].arms = { with: [{ score: 1, skippedPaidGraders: true }] };
    for (const doc of [{ ...aggregate({ a: 1 }), partial: true }, skipped]) {
      const oldDir = writeResult(root, 'old', aggregate({ a: 0 }));
      const newDir = writeResult(root, 'new', doc);
      const child = compareCli(oldDir, newDir);
      assert.equal(child.status, 3, JSON.stringify(doc));
      assert.ok(child.stderr.includes(path.join(newDir, RESULT_FILE)), child.stderr);
      assert.equal(child.stdout, '');
    }
  }));

test('比較：每題新分數都不低於舊分數時沒有退步', () => {
  const result = compareResults(aggregate({ a: 0.33, b: 1 }), aggregate({ a: 0.33, b: 1 }));
  assert.deepEqual(result.regressions, []);
  assert.deepEqual(result.missing, []);
  assert.deepEqual(
    result.rows.map((row) => row.delta),
    [0, 0],
  );
});

test('比較：差值是新分數減舊分數', () => {
  const [row] = compareResults(aggregate({ a: 0.5 }), aggregate({ a: 1 })).rows;
  assert.equal(row.delta, 0.5);
});

test('比較：題目只出現在一邊時列為缺題，並指出缺在哪一邊', () => {
  const result = compareResults(
    aggregate({ 'trace-cites-archive': 1, 'drift-reports-only': 1 }),
    aggregate({ 'drift-reports-only': 1, 'audit-finds-dangerous-default': 1 }),
  );
  assert.deepEqual(result.missing, [
    { name: 'audit-finds-dangerous-default', missingFrom: 'old' },
    { name: 'trace-cites-archive', missingFrom: 'new' },
  ]);
  assert.deepEqual(
    result.rows.map((row) => row.name),
    ['drift-reports-only'],
  );
});

test('run：缺參數時列出缺少的參數，不啟動評測', () => {
  const { errors } = parseRunArgs(['--speclink', 'target/debug/speclink', '--label', 'new-opus']);
  assert.ok(errors.some((error) => error.includes('--model')), errors.join('\n'));
  assert.ok(errors.some((error) => error.includes('--max-cost-usd')), errors.join('\n'));
});

/// 在暫存 repo 根目錄放一支假的執行檔，以及（選擇性）一份已存在的結果。
function fakeRoot(root, { existingLabel } = {}) {
  writeFileSync(path.join(root, 'speclink bin'), '');
  if (existingLabel) writeResult(path.join(root, OUT_ROOT), existingLabel, '{}');
  return root;
}

function runArgs(root, overrides = {}) {
  const values = { speclink: path.join(root, 'speclink bin'), model: 'claude-haiku-5-5', label: 'old-haiku', 'max-cost-usd': '20', ...overrides };
  return Object.entries(values).flatMap(([name, value]) => [`--${name}`, value]);
}

test('run：參數齊全且合法時沒有錯誤（執行檔路徑含空白也可以，腳本不經過 shell）', () =>
  withTempRoot((root) => {
    const { options, errors } = parseRunArgs(runArgs(fakeRoot(root)), root);
    assert.deepEqual(errors, []);
    assert.equal(options.runs, '3');
  }));

test('run：--label 只能是單一路徑段，含 / 或 .. 時拒絕（否則會寫出 target/skill-evals/ 之外）', () =>
  withTempRoot((root) => {
    for (const label of ['../escape', 'a/b', '..', '.hidden', 'a..b']) {
      const { errors } = parseRunArgs(runArgs(fakeRoot(root), { label }), root);
      assert.ok(errors.some((error) => error.includes('--label')), `${label}: ${errors.join('\n')}`);
    }
  }));

test('run：--max-cost-usd 必須是正數', () =>
  withTempRoot((root) => {
    for (const cost of ['abc', '0', '-5', 'Infinity', '20usd']) {
      const { errors } = parseRunArgs(runArgs(fakeRoot(root), { 'max-cost-usd': cost }), root);
      assert.ok(errors.some((error) => error.includes('--max-cost-usd')), `${cost}: ${errors.join('\n')}`);
    }
  }));

test('run：--runs 必須是 1 到 50 的整數', () =>
  withTempRoot((root) => {
    for (const runs of ['0', '1.5', '51', 'three']) {
      const { errors } = parseRunArgs([...runArgs(fakeRoot(root)), '--runs', runs], root);
      assert.ok(errors.some((error) => error.includes('--runs')), `${runs}: ${errors.join('\n')}`);
    }
  }));

test('run：--speclink 指向不存在的檔案時拒絕', () =>
  withTempRoot((root) => {
    const { errors } = parseRunArgs(runArgs(fakeRoot(root), { speclink: path.join(root, 'nope') }), root);
    assert.ok(errors.some((error) => error.includes('--speclink')), errors.join('\n'));
  }));

test('run：同名 label 已有結果時拒絕，不覆蓋付費跑出的結果', () =>
  withTempRoot((root) => {
    const { errors } = parseRunArgs(runArgs(fakeRoot(root, { existingLabel: 'old-haiku' })), root);
    assert.ok(errors.some((error) => error.includes('old-haiku')), errors.join('\n'));
  }));

test('run：任一 label 的暫存目錄都不會落在另一個 label 的結果目錄（重建暫存目錄不會刪到付費結果）', () => {
  const labels = ['old', 'plugin-old', 'bin-old', 'new-opus', 'staging'];
  const outDirs = labels.map((label) => runPaths(label, '/repo').outDir);
  const stagingDirs = labels.flatMap((label) => [runPaths(label, '/repo').binDir, runPaths(label, '/repo').pluginDir]);
  for (const staging of stagingDirs) {
    for (const out of outDirs) {
      assert.ok(staging !== out && !staging.startsWith(out + path.sep) && !out.startsWith(staging + path.sep), `${staging} vs ${out}`);
    }
  }
});

test('run：claude 子行程不讀 stdin、輸出不帶顏色，PATH 以受測執行檔的目錄開頭', () => {
  const options = evalSpawnOptions('/repo/bin', { PATH: '/usr/bin', HOME: '/home/user' });
  assert.equal(options.stdio[0], 'ignore');
  assert.equal(options.env.NO_COLOR, '1');
  assert.equal(options.env.PATH, `/repo/bin${path.delimiter}/usr/bin`);
  assert.equal(options.env.HOME, '/home/user');
});

/// prompt.md frontmatter 的 `tags: [skill:discuss, ...]`（題目一律寫成 flow 形式）。
function caseTags(promptMarkdown) {
  const frontmatter = promptMarkdown.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  const tags = frontmatter?.[1].match(/^tags:\s*\[([^\]]*)\]/m);
  return tags ? tags[1].split(',').map((tag) => tag.trim()) : [];
}

test('涵蓋度：每個 speclink 技能都至少有一題評測', () => {
  const skills = readdirSync(path.join(ROOT, '.claude/skills'))
    .filter((name) => name.startsWith('speclink-'))
    .map((name) => name.slice('speclink-'.length));
  const covered = new Set(
    readdirSync(path.join(ROOT, EVALS_DIR), { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .flatMap((entry) => caseTags(readFileSync(path.join(ROOT, EVALS_DIR, entry.name, 'prompt.md'), 'utf8'))),
  );
  const missing = skills.filter((name) => !covered.has(`skill:${name}`)).sort();
  assert.deepEqual(missing, [], `這些技能沒有評測題（${EVALS_DIR}/<題目>/prompt.md 的 tags 要含 skill:<短名>）：${missing.join(', ')}`);
});
