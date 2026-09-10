// 更新日誌渲染腳本（release-notes spec「渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段」）：
// 純函式的固定輸出，加 CLI 三用法的 exit code、CRLF 正規化與 JSON 形狀守門。
// CLI 在暫存 repo 上跑（release-notes-scratch.mjs），不碰真 repo 的 CHANGELOG.md。
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import test from 'node:test';

import { readReleaseNotes, renderChangelog, renderEntry } from './release-notes-render.mjs';
import { scratchRepo } from './release-notes-scratch.mjs';

const ENTRY_030 = {
  version: '0.3.0',
  date: '2026-09-10',
  sections: [
    { title: '新功能', items: ['甲', '乙'] },
    { title: '修正', items: ['丙'] },
  ],
};
const ENTRY_020 = {
  version: '0.2.0',
  date: '2026-09-08',
  sections: [{ title: '改善', items: ['丁'] }],
};

const FRAGMENT_030 = `## 0.3.0（2026-09-10）

### 新功能

- 甲
- 乙

### 修正

- 丙
`;

/// 只放渲染腳本的暫存 repo；run 直接綁定該腳本。
function renderRepo(notes) {
  const repo = scratchRepo({ scripts: ['release-notes-render.mjs'], notes });
  return { ...repo, run: (args) => repo.run('release-notes-render.mjs', args) };
}

test('renderEntry：版號（日期）標題、分組三級標題、- 條目，尾端單一換行', () => {
  assert.equal(renderEntry(ENTRY_030), FRAGMENT_030);
});

test('renderChangelog：首行 # 更新日誌、條目依序、條目間空一行、檔尾單一換行、一律 LF', () => {
  const out = renderChangelog([ENTRY_030, ENTRY_020]);
  assert.ok(out.startsWith('# 更新日誌\n\n## 0.3.0（2026-09-10）\n'), '首行與第一筆');
  assert.ok(out.indexOf('## 0.3.0') < out.indexOf('## 0.2.0'), '依 JSON 順序');
  assert.ok(out.includes('- 丙\n\n## 0.2.0（2026-09-08）\n\n### 改善\n\n- 丁\n'), '條目間空一行');
  assert.ok(out.endsWith('- 丁\n') && !out.endsWith('\n\n'), '檔尾恰一個換行');
  assert.ok(!out.includes('\r'), '不得含 CR');
});

test('--write 寫出 repo 根 CHANGELOG.md 且內容等於 renderChangelog，exit 0', () => {
  const { run, changelog } = renderRepo([ENTRY_030, ENTRY_020]);
  const result = run(['--write']);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(changelog, 'utf8'), renderChangelog([ENTRY_030, ENTRY_020]));
});

test('--version 印出該版片段；找不到版本 exit 1 且 stderr 含 JSON 頂端版號', () => {
  const { run } = renderRepo([ENTRY_030, ENTRY_020]);
  const hit = run(['--version', '0.3.0']);
  assert.equal(hit.status, 0, hit.stderr);
  assert.equal(hit.stdout, FRAGMENT_030);

  const miss = run(['--version', '0.9.9']);
  assert.equal(miss.status, 1);
  assert.equal(miss.stdout, '');
  assert.ok(miss.stderr.includes('0.9.9'), '點名找不到的版本');
  assert.ok(miss.stderr.includes('0.3.0'), '印出 JSON 頂端版號');
});

test('--check：一致 exit 0；手改一字後 exit 1 且 stderr 含差異摘要', () => {
  const { run, changelog } = renderRepo([ENTRY_030, ENTRY_020]);
  assert.equal(run(['--write']).status, 0);
  const clean = run(['--check']);
  assert.equal(clean.status, 0, clean.stderr);

  writeFileSync(changelog, readFileSync(changelog, 'utf8').replace('- 乙', '- 戊'));
  const drifted = run(['--check']);
  assert.equal(drifted.status, 1);
  assert.ok(drifted.stderr.includes('不一致'), '說明不一致');
  assert.ok(drifted.stderr.includes('- 戊') && drifted.stderr.includes('- 乙'), '差異摘要列出兩邊的行');
});

test('--check：CHANGELOG.md 以 CRLF 讀入時仍 exit 0', () => {
  const { run, changelog } = renderRepo([ENTRY_030, ENTRY_020]);
  assert.equal(run(['--write']).status, 0);
  writeFileSync(changelog, readFileSync(changelog, 'utf8').replace(/\n/g, '\r\n'));
  const result = run(['--check']);
  assert.equal(result.status, 0, result.stderr);
});

test('--check：CHANGELOG.md 不存在時 exit 1', () => {
  const { run } = renderRepo([ENTRY_030]);
  assert.equal(run(['--check']).status, 1);
});

test('無參數或未知參數 exit 1 並印用法', () => {
  const { run } = renderRepo([ENTRY_030]);
  for (const args of [[], ['--bogus'], ['--version']]) {
    const result = run(args);
    assert.equal(result.status, 1, `args=${JSON.stringify(args)}`);
    assert.ok(result.stderr.includes('用法'), 'stderr 印用法');
  }
});

test('JSON 語法錯誤：三種用法都 exit 1、stdout 空、stderr 點名 release-notes.json', () => {
  const { run } = renderRepo('[{"version": "0.3.0",');
  for (const args of [['--write'], ['--version', '0.3.0'], ['--check']]) {
    const result = run(args);
    assert.equal(result.status, 1, `args=${JSON.stringify(args)}`);
    assert.equal(result.stdout, '');
    assert.ok(result.stderr.includes('release-notes.json'), 'stderr 點名檔案');
  }
});

test('JSON 形狀不合（非陣列、缺 sections、title 三值之外、items 空）exit 1 並說明哪裡錯', () => {
  const cases = [
    { notes: { version: '0.3.0' }, needle: '陣列' },
    { notes: [{ version: '0.3.0', date: '2026-09-10' }], needle: 'sections' },
    { notes: [{ version: '0.3.0', date: '2026-09-10', sections: [{ title: '其他', items: ['甲'] }] }], needle: 'title' },
    { notes: [{ version: '0.3.0', date: '2026-09-10', sections: [{ title: '修正', items: [] }] }], needle: 'items' },
    { notes: [{ version: 'v0.3.0', date: '2026-09-10', sections: [{ title: '修正', items: ['甲'] }] }], needle: 'version' },
    { notes: [{ version: '0.3.0', date: '2026/09/10', sections: [{ title: '修正', items: ['甲'] }] }], needle: 'date' },
  ];
  for (const { notes, needle } of cases) {
    const result = renderRepo(notes).run(['--version', '0.3.0']);
    assert.equal(result.status, 1, JSON.stringify(notes));
    assert.equal(result.stdout, '');
    assert.ok(result.stderr.includes(needle), `stderr 應提到 ${needle}：${result.stderr}`);
  }
});

test('真 repo 的 release-notes.json 通過形狀守門且最新在前', () => {
  const entries = readReleaseNotes();
  assert.ok(entries.length >= 1);
  assert.match(entries[0].version, /^\d+\.\d+\.\d+$/);
});
