import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts', 'release-notes.mjs');

function run(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

// 下載指南的檔名必須與 release 管線的資產命名逐字一致（desktop-release spec
// 「指南檔名對齊版號與資產命名」）——這份清單就是那個命名契約。
const installersFor = (version) => [
  `Speclink_${version}_aarch64.dmg`,
  `Speclink_${version}_x64.dmg`,
  `Speclink_${version}_x64-setup.exe`,
  `Speclink_${version}_amd64.AppImage`,
  `Speclink_${version}_aarch64.AppImage`,
  `Speclink_${version}_amd64.deb`,
  `Speclink_${version}_arm64.deb`,
];

test('對照表列出三平台全部安裝檔且檔名含版號', () => {
  const result = run(['--tag', 'v0.1.0']);
  assert.equal(result.status, 0, result.stderr);
  for (const name of installersFor('0.1.0')) {
    assert.ok(result.stdout.includes(name), `缺安裝檔 ${name}`);
  }
});

test('版號替換跟著 tag 走', () => {
  const result = run(['--tag', 'v9.9.9']);
  assert.equal(result.status, 0, result.stderr);
  for (const name of installersFor('9.9.9')) {
    assert.ok(result.stdout.includes(name), `缺安裝檔 ${name}`);
  }
  assert.ok(!result.stdout.includes('0.1.0'), '不得殘留其他版號');
});

test('CLI 安裝指令與 README 教的同一套', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  assert.ok(
    stdout.includes(
      'curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh',
    ),
    '缺 sh 安裝一行',
  );
  assert.ok(
    stdout.includes(
      'irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex',
    ),
    '缺 PowerShell 安裝一行',
  );
  assert.ok(stdout.includes('brew install MomoChenisMe/tap/speclink'), '缺 brew 指令');
});

test('server 一行啟動節：npx 與 Docker image 指令（寫法比照 CLI 節）', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  assert.ok(stdout.includes('npx @speclink/server'), '缺 npx 一行啟動指令');
  assert.ok(stdout.includes('ghcr.io/momochenisme/speclink-server'), '缺 Docker image 一行');
});

test('更新機制檔案標註毋須手動下載', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  assert.ok(stdout.includes('.sig'), '缺 .sig 註記');
  assert.ok(stdout.includes('.app.tar.gz'), '缺 .app.tar.gz 註記');
  assert.ok(stdout.includes('latest.json'), '缺 latest.json 註記');
  assert.match(stdout, /自動更新/, '缺自動更新說明');
});

test('tag 格式不符即非零退出且 stdout 無輸出', () => {
  for (const bad of ['0.1.0', 'v1.2', 'vabc', 'v0.1.0-rc1', '']) {
    const result = run(bad === '' ? ['--tag'] : ['--tag', bad]);
    assert.notEqual(result.status, 0, `tag「${bad}」不應被接受`);
    assert.equal(result.stdout, '', `tag「${bad}」不得輸出內容`);
  }
});

test('缺 --tag 參數即非零退出', () => {
  const result = run([]);
  assert.notEqual(result.status, 0);
  assert.equal(result.stdout, '');
});
