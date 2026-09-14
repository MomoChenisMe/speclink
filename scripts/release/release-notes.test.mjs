import assert from 'node:assert/strict';
import test from 'node:test';

import { scratchRepo } from './release-notes-scratch.mjs';

// 指南之後要接該版的更新日誌片段（release-notes.json 為真相）；斷言不隨真 JSON 的內容變動。
const FIXTURE_NOTES = [
  { version: '0.2.0', date: '2026-09-08', sections: [{ title: '新功能', items: ['乙'] }] },
  { version: '0.1.0', date: '2026-08-01', sections: [{ title: '修正', items: ['甲'] }] },
];
const repo = scratchRepo({ scripts: ['release-notes.mjs', 'release-notes-render.mjs'], notes: FIXTURE_NOTES });
const run = (args) => repo.run('release-notes.mjs', args);

// 下載指南的檔名必須與 release 管線的資產命名逐字一致（desktop-release spec
// 「指南檔名對齊版號與資產命名」；release-assets-trim 後的四個安裝檔）——這份清單
// 就是那個命名契約。
const installersFor = (version) => [
  `Speclink_${version}_universal.dmg`,
  `Speclink_${version}_x64-setup.exe`,
  `Speclink_${version}_amd64.AppImage`,
  `Speclink_${version}_aarch64.AppImage`,
];

test('對照表列出四個安裝檔且檔名含版號；macOS 一列註明兩種晶片同一檔', () => {
  const result = run(['--tag', 'v0.1.0']);
  assert.equal(result.status, 0, result.stderr);
  for (const name of installersFor('0.1.0')) {
    assert.ok(result.stdout.includes(name), `缺安裝檔 ${name}`);
  }
  assert.match(result.stdout, /Apple Silicon[^\n]*Intel[^\n]*Speclink_0\.1\.0_universal\.dmg/, 'macOS 一列應註明兩種晶片同一檔');
});

test('對照表為 Linux 伺服器／無圖形介面另列一列，指向 CLI 一行安裝而非任何檔案', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  const row = stdout.split('\n').find((line) => /^\|[^|]*(伺服器|無圖形)/.test(line));
  assert.ok(row, '缺 Linux 伺服器／無圖形介面那一列');
  assert.ok(!/Speclink_0\.1\.0/.test(row), '該列不得指向任何安裝檔');
  assert.match(row, /CLI/, '該列應導向 CLI 一行安裝');
});

test('退場的檔案不再出現：.sig、.deb、校驗碼檔、CLI 壓縮檔、PowerShell 腳本', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  for (const gone of ['.sig', '.deb', 'SHA256SUMS', 'install.ps1', '.zip', 'speclink-v0.1.0-']) {
    assert.ok(!stdout.includes(gone), `指南不得再提及 ${gone}`);
  }
  // .app.tar.gz 仍在（自動更新用）；其他 .tar.gz 都不該出現。
  assert.ok(!stdout.replace(/\.app\.tar\.gz/g, '').includes('.tar.gz'), '除 .app.tar.gz 外不得提及 .tar.gz');
});

test('版號替換跟著 tag 走', () => {
  const result = run(['--tag', 'v0.2.0']);
  assert.equal(result.status, 0, result.stderr);
  for (const name of installersFor('0.2.0')) {
    assert.ok(result.stdout.includes(name), `缺安裝檔 ${name}`);
  }
  assert.ok(!result.stdout.includes('0.1.0'), '不得殘留其他版號');
});

test('CLI 安裝指令三條（npm、安裝腳本、Homebrew）與 README 教的同一套', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  assert.ok(stdout.includes('npm i -g @speclink/cli'), '缺 npm 全域安裝一行');
  assert.ok(
    stdout.includes(
      'curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh',
    ),
    '缺 sh 安裝一行',
  );
  assert.ok(stdout.includes('brew install MomoChenisMe/tap/speclink'), '缺 brew 指令');
});

test('server 一行啟動節：npx 與 Docker image 指令（寫法比照 CLI 節）', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  assert.ok(stdout.includes('npx @speclink/server'), '缺 npx 一行啟動指令');
  assert.ok(stdout.includes('ghcr.io/momochenisme/speclink-server'), '缺 Docker image 一行');
});

test('更新機制檔案（.app.tar.gz 與 latest.json）標註毋須手動下載', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
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

test('指南（--- 分隔線）之後接該版的更新日誌片段', () => {
  const result = run(['--tag', 'v0.2.0']);
  assert.equal(result.status, 0, result.stderr);
  const divider = result.stdout.indexOf('\n---\n');
  assert.ok(divider >= 0, '指南以 --- 分隔線結尾');
  const tail = result.stdout.slice(divider);
  assert.ok(tail.includes('## 0.2.0（2026-09-08）'), '分隔線之後含該版標題');
  assert.ok(tail.includes('- 乙'), '分隔線之後含該版條目');
  assert.ok(!result.stdout.includes('## 0.1.0'), '不含其他版本');
});

test('JSON 沒有該版（tag 仍符 vX.Y.Z）即非零退出、stderr 說明缺該版、stdout 無輸出', () => {
  const result = run(['--tag', 'v9.9.9']);
  assert.equal(result.status, 1);
  assert.equal(result.stdout, '');
  assert.ok(result.stderr.includes('9.9.9'), 'stderr 點名缺的版本');
});
