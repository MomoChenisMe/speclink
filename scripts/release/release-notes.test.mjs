import assert from 'node:assert/strict';
import test from 'node:test';

import { scratchRepo } from './release-notes-scratch.mjs';

// 指南之後要接該版的更新日誌片段（release-notes.json 為真相）；斷言不隨真 JSON 的內容變動。
const FIXTURE_NOTES = [
  { version: '0.2.0', date: '2026-09-08', sections: [{ kind: 'features', items: [{ en: 'Beta', 'zh-TW': '乙' }] }] },
  { version: '0.1.0', date: '2026-08-01', sections: [{ kind: 'fixes', items: [{ en: 'Alpha', 'zh-TW': '甲' }] }] },
];
const repo = scratchRepo({ scripts: ['release-notes.mjs', 'release-notes-render.mjs'], notes: FIXTURE_NOTES });
const run = (args) => repo.run('release-notes.mjs', args);

// 下載指南的檔名必須與 release 管線的資產命名逐字一致（desktop-release spec
// 「指南檔名對齊版號與資產命名」；release-assets-trim 後的四個安裝檔）——這份清單
// 就是那個命名契約（含 Windows portable ZIP）。
const installersFor = (version) => [
  `Speclink_${version}_universal.dmg`,
  `Speclink_${version}_x64-setup.exe`,
  `Speclink_${version}_x64-portable.zip`,
  `Speclink_${version}_amd64.AppImage`,
  `Speclink_${version}_aarch64.AppImage`,
];

const SUMMARY = '<summary>繁體中文</summary>';

/// 把輸出拆成英文段（`<details>` 之前）與繁中段（`<summary>` 與 `</details>` 之間），並斷言收合區塊的骨架。
function sections(stdout) {
  const open = stdout.indexOf(`\n\n<details>\n${SUMMARY}\n\n`);
  assert.ok(open > 0, '英文段之後接 <details><summary>繁體中文</summary>，前後各空一行');
  assert.ok(stdout.endsWith('\n\n</details>\n'), '收合區塊以空行與 </details> 收尾，位於輸出最後');
  assert.equal(stdout.split('<details>').length, 2, '只有一個收合區塊');
  const zhStart = stdout.indexOf(SUMMARY) + SUMMARY.length;
  return { en: stdout.slice(0, open), zh: stdout.slice(zhStart, stdout.lastIndexOf('</details>')) };
}

const NAME_PATTERN = /Speclink_[\w.-]+|ghcr\.io\/[\w./-]+:[\w.]+/g;

test('輸出依序為英文指南、英文片段、收合的「繁體中文」區塊（繁中指南、繁中片段）', () => {
  const result = run(['--tag', 'v0.2.0']);
  assert.equal(result.status, 0, result.stderr);
  const { en, zh } = sections(result.stdout);
  assert.ok(result.stdout.startsWith('## 📦 Which file should I download?\n'), '英文指南在最前面');
  assert.ok(en.indexOf('\n---\n') < en.indexOf('## 0.2.0 (2026-09-08)'), '英文指南（--- 分隔線）之後接英文片段');
  assert.ok(en.includes('## 0.2.0 (2026-09-08)\n\n### New features\n\n- Beta'), '英文片段含版號標題與英文條目');
  assert.ok(zh.startsWith('\n\n## 📦 我該下載哪個檔案？\n'), '收合區塊以繁中指南開頭');
  assert.ok(zh.indexOf('\n---\n') < zh.indexOf('## 0.2.0（2026-09-08）'), '繁中指南（--- 分隔線）之後接繁中片段');
  assert.ok(zh.includes('## 0.2.0（2026-09-08）\n\n### 新功能\n\n- 乙'), '繁中片段含版號標題與繁中條目');
  assert.ok(!en.includes('乙') && !zh.includes('Beta'), '兩段的條目各用自己的語言');
});

test('兩種語言的對照表都列出安裝檔與 Windows 免安裝 ZIP 且檔名含版號；macOS 一列註明兩種晶片同一檔', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  const { en, zh } = sections(stdout);
  for (const [lang, text] of Object.entries({ en, zh })) {
    for (const name of installersFor('0.1.0')) {
      assert.ok(text.includes(name), `${lang} 缺安裝檔 ${name}`);
    }
    assert.match(text, /Apple Silicon[^\n]*Intel[^\n]*Speclink_0\.1\.0_universal\.dmg/, `${lang} 的 macOS 一列應註明兩種晶片同一檔`);
  }
});

test('兩段的檔名與 Docker image 逐字相同', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  const { en, zh } = sections(stdout);
  const enNames = en.match(NAME_PATTERN);
  assert.ok(enNames.length >= 5, `英文段應含四個安裝檔與 Docker image：${enNames}`);
  assert.deepEqual(zh.match(NAME_PATTERN), enNames);
});

test('對照表為 Linux 伺服器／無圖形介面另列一列，指向 CLI 一行安裝而非任何檔案', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  const { en, zh } = sections(stdout);
  const rows = {
    en: en.split('\n').find((line) => /^\|[^|]*(server|graphical)/i.test(line)),
    zh: zh.split('\n').find((line) => /^\|[^|]*(伺服器|無圖形)/.test(line)),
  };
  for (const [lang, row] of Object.entries(rows)) {
    assert.ok(row, `${lang} 缺 Linux 伺服器／無圖形介面那一列`);
    assert.ok(!/Speclink_0\.1\.0/.test(row), `${lang} 該列不得指向任何安裝檔`);
    assert.match(row, /CLI/, `${lang} 該列應導向 CLI 一行安裝`);
  }
});

test('退場的檔案不再出現：.sig、.deb、校驗碼檔、CLI 壓縮檔、PowerShell 腳本', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  for (const gone of ['.sig', '.deb', 'SHA256SUMS', 'install.ps1', 'speclink-v0.1.0-']) {
    assert.ok(!stdout.includes(gone), `指南不得再提及 ${gone}`);
  }
  // .app.tar.gz 仍在（自動更新用）；其他 .tar.gz 都不該出現。
  assert.ok(!stdout.replace(/\.app\.tar\.gz/g, '').includes('.tar.gz'), '除 .app.tar.gz 外不得提及 .tar.gz');
});

test('版號替換跟著 tag 走', () => {
  const result = run(['--tag', 'v0.2.0']);
  assert.equal(result.status, 0, result.stderr);
  const { en, zh } = sections(result.stdout);
  for (const name of installersFor('0.2.0')) {
    assert.ok(en.includes(name) && zh.includes(name), `缺安裝檔 ${name}`);
  }
  assert.ok(!result.stdout.includes('0.1.0'), '不得殘留其他版號');
});

test('兩種語言都有 CLI 安裝指令三條（npm、安裝腳本、Homebrew），與 README 教的同一套', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  for (const [lang, text] of Object.entries(sections(stdout))) {
    assert.ok(text.includes('npm i -g @speclink/cli'), `${lang} 缺 npm 全域安裝一行`);
    assert.ok(
      text.includes('curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh'),
      `${lang} 缺 sh 安裝一行`,
    );
    assert.ok(text.includes('brew install MomoChenisMe/tap/speclink'), `${lang} 缺 brew 指令`);
  }
});

test('兩種語言都有 server 一行啟動節：npx 與 Docker image 指令（寫法比照 CLI 節）', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  for (const [lang, text] of Object.entries(sections(stdout))) {
    assert.ok(text.includes('npx @speclink/server'), `${lang} 缺 npx 一行啟動指令`);
    assert.ok(text.includes('ghcr.io/mosutw/speclink-server:0.1.0'), `${lang} 缺 Docker image 一行`);
  }
});

test('兩種語言都標註更新機制檔案（.app.tar.gz 與 latest.json）毋須手動下載', () => {
  const { stdout } = run(['--tag', 'v0.1.0']);
  const { en, zh } = sections(stdout);
  for (const [lang, text] of Object.entries({ en, zh })) {
    assert.ok(text.includes('.app.tar.gz'), `${lang} 缺 .app.tar.gz 註記`);
    assert.ok(text.includes('latest.json'), `${lang} 缺 latest.json 註記`);
  }
  assert.match(en, /automatic update/i, '英文段缺自動更新說明');
  assert.match(zh, /自動更新/, '繁中段缺自動更新說明');
});

test('只含 tag 那一版的片段', () => {
  const { stdout } = run(['--tag', 'v0.2.0']);
  assert.ok(!stdout.includes('## 0.1.0'), '不含其他版本');
  assert.ok(!stdout.includes('Alpha') && !stdout.includes('甲'), '不含其他版本的條目');
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

test('JSON 沒有該版（tag 仍符 vX.Y.Z）即非零退出、stderr 說明缺該版、stdout 無輸出', () => {
  const result = run(['--tag', 'v9.9.9']);
  assert.equal(result.status, 1);
  assert.equal(result.stdout, '');
  assert.ok(result.stderr.includes('9.9.9'), 'stderr 點名缺的版本');
});
