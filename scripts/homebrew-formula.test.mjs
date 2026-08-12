// Homebrew formula 產生器的單元測試（cli-distribution spec「Homebrew formula
// 產生器」，design D4）。以子行程執行 scripts/homebrew-formula.mjs：給定 tag 與
// 一份 SHA256SUMS.txt，驗證輸出的 formula 含四組平台 url＋sha256；缺任一平台條目
// 時必須以非零結束並點名該平台（checksum 每版都變，手抄必錯，錯了 brew 才會發現）。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts/homebrew-formula.mjs');

const TAG = 'v0.1.0';
const REPO = 'MomoChenisMe/speclink';

// 四個 brew 會用到的 target；Windows 的 msvc target 也在 SHA256SUMS.txt 裡，
// 但 formula 不引用它——fixture 保留該行，順便驗產生器不會誤把它算進來。
const TARGETS = {
  'aarch64-apple-darwin': 'a'.repeat(64),
  'x86_64-apple-darwin': 'b'.repeat(64),
  'aarch64-unknown-linux-gnu': 'c'.repeat(64),
  'x86_64-unknown-linux-gnu': 'd'.repeat(64),
};
const WINDOWS_LINE = `${'e'.repeat(64)}  speclink-${TAG}-x86_64-pc-windows-msvc.zip`;

/// 佈置一份 SHA256SUMS.txt。omit 列出要故意缺漏的 target。
function stageSums(t, { omit = [], tag = TAG } = {}) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'brew-formula-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));

  const lines = [];
  for (const [target, digest] of Object.entries(TARGETS)) {
    if (omit.includes(target)) continue;
    lines.push(`${digest}  speclink-${tag}-${target}.tar.gz`);
    // server 壓縮檔同樣在 Release 中；產生器必須只取 CLI 那支。
    lines.push(`${'f'.repeat(64)}  speclink-server-${tag}-${target}.tar.gz`);
  }
  lines.push(WINDOWS_LINE);
  lines.push(`${'0'.repeat(64)}  latest.json`);

  const file = path.join(dir, 'SHA256SUMS.txt');
  writeFileSync(file, `${lines.join('\n')}\n`);
  return file;
}

function runGenerator(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

test('輸出的 formula 含四組平台 url 與對應 sha256', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0, `產生器應成功結束\nstderr: ${result.stderr}`);
  const formula = result.stdout;

  assert.match(formula, /class Speclink < Formula/, 'formula 應宣告 Speclink class');
  assert.match(formula, /on_macos do/, 'formula 應含 on_macos 區塊');
  assert.match(formula, /on_linux do/, 'formula 應含 on_linux 區塊');

  for (const [target, digest] of Object.entries(TARGETS)) {
    const url = `https://github.com/${REPO}/releases/download/${TAG}/speclink-${TAG}-${target}.tar.gz`;
    assert.ok(formula.includes(url), `formula 應含 ${target} 的下載網址`);
    assert.ok(formula.includes(digest), `formula 應含 ${target} 的 sha256`);
  }
});

test('formula 綁定 arm64 與 intel 兩種架構分支', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0);
  // 每個 OS 區塊各有兩個架構分支，合計四組 url。
  const urlCount = (result.stdout.match(/url "https:\/\//g) || []).length;
  assert.equal(urlCount, 4, `formula 應恰有四組 url，實得 ${urlCount}`);
  assert.match(result.stdout, /Hardware::CPU\.arm\?/, 'formula 應以 CPU 架構分支選擇資產');
});

test('不把 speclink-server 的條目誤當成 CLI 資產', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0);
  assert.ok(!result.stdout.includes('speclink-server'), 'formula 不得引用 server 壓縮檔');
  assert.ok(!result.stdout.includes('f'.repeat(64)), 'formula 不得引用 server 的 sha256');
});

test('formula 不引用 Windows 資產', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0);
  assert.ok(!result.stdout.includes('windows-msvc'), 'brew formula 不應引用 Windows 資產');
});

test('formula 帶去掉 v 前綴的版本並安裝 speclink binary', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0);
  assert.match(result.stdout, /version "0\.1\.0"/, 'version 應為去掉 v 前綴的 tag');
  assert.match(result.stdout, /bin\.install "speclink"/, 'formula 應安裝 speclink binary');
  assert.match(result.stdout, /test do/, 'formula 應含 test 區塊供 brew audit 檢查');
});

// --- fail-closed ---

for (const target of Object.keys(TARGETS)) {
  test(`SHA256SUMS.txt 缺 ${target} 條目時以非零結束並點名該平台`, (t) => {
    const sums = stageSums(t, { omit: [target] });

    const result = runGenerator(['--tag', TAG, '--sums', sums]);

    assert.notEqual(result.status, 0, '缺條目應以非零結束');
    assert.match(result.stderr, new RegExp(target), `錯誤訊息應點名缺少的 ${target}`);
    assert.equal(result.stdout.trim(), '', '失敗時 stdout 不得有 formula 輸出');
  });
}

test('tag 與 SHA256SUMS.txt 內的版本不符時視為缺條目而失敗', (t) => {
  const sums = stageSums(t, { tag: 'v0.0.9' });

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.notEqual(result.status, 0, '版本不符應以非零結束');
  assert.equal(result.stdout.trim(), '', '失敗時 stdout 不得有 formula 輸出');
});

test('缺必要參數時以非零結束並說明用法', (t) => {
  const sums = stageSums(t);

  const noTag = runGenerator(['--sums', sums]);
  assert.notEqual(noTag.status, 0, '缺 --tag 應以非零結束');
  assert.match(noTag.stderr, /--tag/, '錯誤訊息應點名缺少的參數');

  const noSums = runGenerator(['--tag', TAG]);
  assert.notEqual(noSums.status, 0, '缺 --sums 應以非零結束');
  assert.match(noSums.stderr, /--sums/, '錯誤訊息應點名缺少的參數');
});

test('SHA256SUMS.txt 不存在時以非零結束並指出路徑', (t) => {
  const result = runGenerator(['--tag', TAG, '--sums', '/nonexistent/SHA256SUMS.txt']);

  assert.notEqual(result.status, 0, '檔案不存在應以非零結束');
  assert.match(result.stderr, /nonexistent/, '錯誤訊息應指出找不到的路徑');
});

// formula 是 Ruby——字串拼接產生的語法錯誤，不驗的話要到使用者 brew install
// 失敗才會現形。有 ruby 的環境就順手驗掉；沒有則跳過。
const hasRuby = spawnSync('ruby', ['--version']).status === 0;

test('產出的 formula 是合法 Ruby', { skip: !hasRuby && '本機無 ruby' }, (t) => {
  const sums = stageSums(t);
  const dir = mkdtempSync(path.join(os.tmpdir(), 'brew-formula-syntax-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));

  const result = runGenerator(['--tag', TAG, '--sums', sums]);
  assert.equal(result.status, 0, `產生器應成功結束\nstderr: ${result.stderr}`);

  const file = path.join(dir, 'speclink.rb');
  writeFileSync(file, result.stdout);
  const syntax = spawnSync('ruby', ['-c', file], { encoding: 'utf8' });

  assert.equal(syntax.status, 0, `formula 語法錯誤：${syntax.stderr}`);
});
