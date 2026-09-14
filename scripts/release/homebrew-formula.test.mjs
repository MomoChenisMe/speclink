// Homebrew formula 產生器的單元測試（cli-distribution spec「Homebrew formula
// 產生器」，design D4／release-assets-trim D5）。以子行程執行 scripts/release/homebrew-formula.mjs：
// 給定 tag 與一份 CLI npm 發布產出的 tgz 校驗清單，驗證輸出的 formula 含四組指向
// registry.npmjs.org 的 url＋sha256；缺任一平台條目時必須以非零結束並點名該平台
// （checksum 每版都變，手抄必錯，錯了 brew 才會發現）。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const script = path.join(root, 'scripts/release/homebrew-formula.mjs');

const TAG = 'v0.5.0';
const VERSION = '0.5.0';

// 四個 brew 會用到的平台子套件（os-cpu）；win32 的 tgz 也在校驗清單裡，但 formula
// 不引用它——fixture 保留該行，順便驗產生器不會誤把它算進來。校驗清單的檔名是
// npm pack 的產物名：@speclink/cli-darwin-arm64 → speclink-cli-darwin-arm64-<版本>.tgz。
const PLATFORMS = {
  'darwin-arm64': 'a'.repeat(64),
  'darwin-x64': 'b'.repeat(64),
  'linux-arm64': 'c'.repeat(64),
  'linux-x64': 'd'.repeat(64),
};
const packName = (platform, version) => `speclink-cli-${platform}-${version}.tgz`;
const registryUrl = (platform, version) =>
  `https://registry.npmjs.org/@speclink/cli-${platform}/-/cli-${platform}-${version}.tgz`;

/// 佈置一份校驗清單（sha256sum 的輸出格式：digest、兩空白、檔名）。omit 列出要故意缺漏的平台。
function stageSums(t, { omit = [], version = VERSION } = {}) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'brew-formula-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));

  const lines = [];
  for (const [platform, digest] of Object.entries(PLATFORMS)) {
    if (omit.includes(platform)) continue;
    lines.push(`${digest}  ${packName(platform, version)}`);
  }
  lines.push(`${'e'.repeat(64)}  ${packName('win32-x64', version)}`);
  // 主套件的 tgz 也在清單裡；formula 只引用平台子套件。
  lines.push(`${'f'.repeat(64)}  speclink-cli-${version}.tgz`);

  const file = path.join(dir, 'cli-npm-sums.txt');
  writeFileSync(file, `${lines.join('\n')}\n`);
  return file;
}

function runGenerator(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

test('輸出的 formula 含四組指向 registry.npmjs.org 的 url 與對應 sha256', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0, `產生器應成功結束\nstderr: ${result.stderr}`);
  const formula = result.stdout;

  assert.match(formula, /class Speclink < Formula/, 'formula 應宣告 Speclink class');
  assert.match(formula, /on_macos do/, 'formula 應含 on_macos 區塊');
  assert.match(formula, /on_linux do/, 'formula 應含 on_linux 區塊');

  for (const [platform, digest] of Object.entries(PLATFORMS)) {
    assert.ok(formula.includes(`url "${registryUrl(platform, VERSION)}"`), `formula 應含 ${platform} 的 registry 下載網址`);
    assert.ok(formula.includes(`sha256 "${digest}"`), `formula 應含 ${platform} 的 sha256`);
  }
  assert.ok(!formula.includes('github.com/MomoChenisMe/speclink/releases'), 'formula 不得再指向 GitHub Release 資產');
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

test('formula 不引用 win32 子套件與主套件的 tgz', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0);
  assert.ok(!result.stdout.includes('win32'), 'brew formula 不應引用 Windows 資產');
  assert.ok(!result.stdout.includes('e'.repeat(64)), 'formula 不得引用 win32 的 sha256');
  assert.ok(!result.stdout.includes('f'.repeat(64)), 'formula 不得引用主套件的 sha256');
});

test('formula 帶去掉 v 前綴的版本並安裝 speclink binary', (t) => {
  const sums = stageSums(t);

  const result = runGenerator(['--tag', TAG, '--sums', sums]);

  assert.equal(result.status, 0);
  assert.match(result.stdout, /version "0\.5\.0"/, 'version 應為去掉 v 前綴的 tag');
  assert.match(result.stdout, /bin\.install "speclink"/, 'formula 應安裝 speclink binary');
  assert.match(result.stdout, /test do/, 'formula 應含 test 區塊供 brew audit 檢查');
});

// --- fail-closed ---

for (const platform of Object.keys(PLATFORMS)) {
  test(`校驗清單缺 ${platform} 條目時以非零結束並點名該平台`, (t) => {
    const sums = stageSums(t, { omit: [platform] });

    const result = runGenerator(['--tag', TAG, '--sums', sums]);

    assert.notEqual(result.status, 0, '缺條目應以非零結束');
    assert.match(result.stderr, new RegExp(platform), `錯誤訊息應點名缺少的 ${platform}`);
    assert.equal(result.stdout.trim(), '', '失敗時 stdout 不得有 formula 輸出');
  });
}

test('tag 與校驗清單內的版本不符時視為缺條目而失敗', (t) => {
  const sums = stageSums(t, { version: '0.4.9' });

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

test('校驗清單不存在時以非零結束並指出路徑', (t) => {
  const result = runGenerator(['--tag', TAG, '--sums', '/nonexistent/cli-npm-sums.txt']);

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
