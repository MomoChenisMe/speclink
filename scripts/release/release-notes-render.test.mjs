// 更新日誌渲染腳本（release-notes spec「渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段」）：
// 兩種語言純函式的固定輸出，加 CLI 三用法的 exit code、--lang、CRLF 正規化與 JSON 雙語形狀守門。
// CLI 在暫存 repo 上跑（release-notes-scratch.mjs），不碰真 repo 的兩份更新日誌。
import assert from 'node:assert/strict';
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import test from 'node:test';

import { readReleaseNotes, renderChangelog, renderEntry } from './release-notes-render.mjs';
import { scratchRepo } from './release-notes-scratch.mjs';

const ENTRY_030 = {
  version: '0.3.0',
  date: '2026-09-10',
  sections: [
    {
      kind: 'features',
      items: [
        { en: 'Alpha', 'zh-TW': '甲' },
        { en: 'Beta', 'zh-TW': '乙' },
      ],
    },
    { kind: 'fixes', items: [{ en: 'Gamma', 'zh-TW': '丙' }] },
  ],
};
const ENTRY_020 = {
  version: '0.2.0',
  date: '2026-09-08',
  sections: [{ kind: 'improvements', items: [{ en: 'Delta', 'zh-TW': '丁' }] }],
};

const FRAGMENT_030_EN = `## 0.3.0 (2026-09-10)

### New features

- Alpha
- Beta

### Fixes

- Gamma
`;

const FRAGMENT_030_ZH = `## 0.3.0（2026-09-10）

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

test('renderEntry：依語言畫版號（日期）標題、分組三級標題與 - 條目，尾端單一換行', () => {
  assert.equal(renderEntry(ENTRY_030, 'en'), FRAGMENT_030_EN);
  assert.equal(renderEntry(ENTRY_030, 'zh-TW'), FRAGMENT_030_ZH);
  assert.equal(renderEntry(ENTRY_020, 'en'), '## 0.2.0 (2026-09-08)\n\n### Improvements\n\n- Delta\n');
  assert.equal(renderEntry(ENTRY_020, 'zh-TW'), '## 0.2.0（2026-09-08）\n\n### 改善\n\n- 丁\n');
});

test('renderChangelog en：首行 # Changelog、次行語言切換、條目依序、條目間空一行、檔尾單一換行、一律 LF', () => {
  const out = renderChangelog([ENTRY_030, ENTRY_020], 'en');
  assert.ok(
    out.startsWith('# Changelog\n**English** · [繁體中文](CHANGELOG.zh-TW.md)\n\n## 0.3.0 (2026-09-10)\n'),
    '首行、語言切換與第一筆',
  );
  assert.ok(out.indexOf('## 0.3.0') < out.indexOf('## 0.2.0'), '依 JSON 順序');
  assert.ok(out.includes('- Gamma\n\n## 0.2.0 (2026-09-08)\n\n### Improvements\n\n- Delta\n'), '條目間空一行');
  assert.ok(out.endsWith('- Delta\n') && !out.endsWith('\n\n'), '檔尾恰一個換行');
  assert.ok(!out.includes('\r'), '不得含 CR');
});

test('renderChangelog zh-TW：首行 # 更新日誌、次行語言切換、條目依序、條目間空一行、檔尾單一換行、一律 LF', () => {
  const out = renderChangelog([ENTRY_030, ENTRY_020], 'zh-TW');
  assert.ok(
    out.startsWith('# 更新日誌\n[English](CHANGELOG.md) · **繁體中文**\n\n## 0.3.0（2026-09-10）\n'),
    '首行、語言切換與第一筆',
  );
  assert.ok(out.indexOf('## 0.3.0') < out.indexOf('## 0.2.0'), '依 JSON 順序');
  assert.ok(out.includes('- 丙\n\n## 0.2.0（2026-09-08）\n\n### 改善\n\n- 丁\n'), '條目間空一行');
  assert.ok(out.endsWith('- 丁\n') && !out.endsWith('\n\n'), '檔尾恰一個換行');
  assert.ok(!out.includes('\r'), '不得含 CR');
});

test('--write 寫出 repo 根 CHANGELOG.md 與 CHANGELOG.zh-TW.md，內容各等於該語言的 renderChangelog，exit 0', () => {
  const { run, changelog, changelogZh } = renderRepo([ENTRY_030, ENTRY_020]);
  const result = run(['--write']);
  assert.equal(result.status, 0, result.stderr);
  const en = readFileSync(changelog, 'utf8');
  const zh = readFileSync(changelogZh, 'utf8');
  assert.equal(en, renderChangelog([ENTRY_030, ENTRY_020], 'en'));
  assert.equal(zh, renderChangelog([ENTRY_030, ENTRY_020], 'zh-TW'));
  assert.deepEqual(en.split('\n').slice(0, 2), ['# Changelog', '**English** · [繁體中文](CHANGELOG.zh-TW.md)']);
  assert.deepEqual(zh.split('\n').slice(0, 2), ['# 更新日誌', '[English](CHANGELOG.md) · **繁體中文**']);
});

test('--version 預設印英文片段，--lang zh-TW 印繁中片段，--lang en 印英文片段', () => {
  const { run } = renderRepo([ENTRY_030, ENTRY_020]);
  const byDefault = run(['--version', '0.3.0']);
  assert.equal(byDefault.status, 0, byDefault.stderr);
  assert.equal(byDefault.stdout, FRAGMENT_030_EN);

  const zh = run(['--version', '0.3.0', '--lang', 'zh-TW']);
  assert.equal(zh.status, 0, zh.stderr);
  assert.equal(zh.stdout, FRAGMENT_030_ZH);

  const en = run(['--version', '0.3.0', '--lang', 'en']);
  assert.equal(en.status, 0, en.stderr);
  assert.equal(en.stdout, FRAGMENT_030_EN);
});

test('--version 找不到版本 exit 1 且 stderr 含 JSON 頂端版號', () => {
  const { run } = renderRepo([ENTRY_030, ENTRY_020]);
  const miss = run(['--version', '0.9.9']);
  assert.equal(miss.status, 1);
  assert.equal(miss.stdout, '');
  assert.ok(miss.stderr.includes('0.9.9'), '點名找不到的版本');
  assert.ok(miss.stderr.includes('0.3.0'), '印出 JSON 頂端版號');
});

test('--lang 不是 en 或 zh-TW 時 exit 1、stdout 空、stderr 說明可用值', () => {
  const { run } = renderRepo([ENTRY_030]);
  for (const lang of ['fr', 'zh-tw', 'EN', '__proto__', '']) {
    const result = run(['--version', '0.3.0', '--lang', lang]);
    assert.equal(result.status, 1, `lang=${JSON.stringify(lang)}`);
    assert.equal(result.stdout, '');
    assert.ok(result.stderr.includes('zh-TW'), `stderr 說明可用值：${result.stderr}`);
  }
});

test('--check：兩份一致 exit 0', () => {
  const { run } = renderRepo([ENTRY_030, ENTRY_020]);
  assert.equal(run(['--write']).status, 0);
  const clean = run(['--check']);
  assert.equal(clean.status, 0, clean.stderr);
});

test('--check：手改 CHANGELOG.zh-TW.md 一字後 exit 1，stderr 點名該檔並印差異摘要', () => {
  const { run, changelogZh } = renderRepo([ENTRY_030, ENTRY_020]);
  assert.equal(run(['--write']).status, 0);
  writeFileSync(changelogZh, readFileSync(changelogZh, 'utf8').replace('- 乙', '- 戊'));
  const drifted = run(['--check']);
  assert.equal(drifted.status, 1);
  assert.match(drifted.stderr, /^CHANGELOG\.zh-TW\.md 與 release-notes\.json 重算結果不一致/m, '點名漂移的檔名');
  assert.doesNotMatch(drifted.stderr, /^CHANGELOG\.md 與/m, '沒漂移的檔不被點名');
  assert.ok(drifted.stderr.includes('- 戊') && drifted.stderr.includes('- 乙'), '差異摘要列出兩邊的行');
});

test('--check：手改 CHANGELOG.md 一字後 exit 1，stderr 點名該檔並印差異摘要', () => {
  const { run, changelog } = renderRepo([ENTRY_030, ENTRY_020]);
  assert.equal(run(['--write']).status, 0);
  writeFileSync(changelog, readFileSync(changelog, 'utf8').replace('- Beta', '- Epsilon'));
  const drifted = run(['--check']);
  assert.equal(drifted.status, 1);
  assert.match(drifted.stderr, /^CHANGELOG\.md 與 release-notes\.json 重算結果不一致/m, '點名漂移的檔名');
  assert.doesNotMatch(drifted.stderr, /^CHANGELOG\.zh-TW\.md 與/m, '沒漂移的檔不被點名');
  assert.ok(drifted.stderr.includes('- Epsilon') && drifted.stderr.includes('- Beta'), '差異摘要列出兩邊的行');
});

test('--check：兩份都以 CRLF 讀入時仍 exit 0', () => {
  const { run, changelog, changelogZh } = renderRepo([ENTRY_030, ENTRY_020]);
  assert.equal(run(['--write']).status, 0);
  for (const file of [changelog, changelogZh]) {
    writeFileSync(file, readFileSync(file, 'utf8').replace(/\n/g, '\r\n'));
  }
  const result = run(['--check']);
  assert.equal(result.status, 0, result.stderr);
});

test('--check：任一份不存在時 exit 1 並點名缺的檔', () => {
  const { run, changelog, changelogZh } = renderRepo([ENTRY_030]);
  assert.equal(run(['--write']).status, 0);
  rmSync(changelogZh);
  const missingZh = run(['--check']);
  assert.equal(missingZh.status, 1);
  assert.ok(missingZh.stderr.includes('CHANGELOG.zh-TW.md'), missingZh.stderr);

  rmSync(changelog);
  assert.equal(run(['--check']).status, 1);
});

test('無參數或未知參數 exit 1 並印用法', () => {
  const { run } = renderRepo([ENTRY_030]);
  for (const args of [[], ['--bogus'], ['--version'], ['--version', '0.3.0', '--lang'], ['--version', '0.3.0', '--bogus', 'en']]) {
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

const withSections = (sections) => [{ version: '0.3.0', date: '2026-09-10', sections }];

test('JSON 形狀不合 exit 1、stdout 空並說明哪裡錯', () => {
  const cases = [
    { notes: { version: '0.3.0' }, needle: '陣列' },
    { notes: [{ version: '0.3.0', date: '2026-09-10' }], needle: 'sections' },
    { notes: withSections([{ kind: 'other', items: [{ en: 'A', 'zh-TW': '甲' }] }]), needle: 'kind' },
    { notes: withSections([{ title: '修正', items: ['甲'] }]), needle: 'kind' },
    { notes: withSections([{ kind: 'fixes', items: [] }]), needle: 'items' },
    {
      notes: withSections([
        { kind: 'fixes', items: [{ en: 'A', 'zh-TW': '甲' }] },
        { kind: 'features', items: [{ en: 'B', 'zh-TW': '乙' }] },
      ]),
      needle: '順序',
    },
    {
      notes: withSections([
        { kind: 'fixes', items: [{ en: 'A', 'zh-TW': '甲' }] },
        { kind: 'fixes', items: [{ en: 'B', 'zh-TW': '乙' }] },
      ]),
      needle: '順序',
    },
    { notes: [{ version: 'v0.3.0', date: '2026-09-10', sections: [] }], needle: 'version' },
    { notes: [{ version: '0.3.0', date: '2026/09/10', sections: [] }], needle: 'date' },
  ];
  for (const { notes, needle } of cases) {
    const result = renderRepo(notes).run(['--version', '0.3.0']);
    assert.equal(result.status, 1, JSON.stringify(notes));
    assert.equal(result.stdout, '');
    assert.ok(result.stderr.includes(needle), `stderr 應提到 ${needle}：${result.stderr}`);
  }
});

test('條目缺一種語言、為空字串或仍是舊的字串形狀：三種用法都 exit 1，stderr 點名版號、分組、條目位置與缺的語言', () => {
  const cases = [
    { item: { 'zh-TW': '乙' }, lang: 'en' },
    { item: { en: 'Beta' }, lang: 'zh-TW' },
    { item: { en: 'Beta', 'zh-TW': '' }, lang: 'zh-TW' },
    { item: { en: '  ', 'zh-TW': '乙' }, lang: 'en' },
    { item: '乙', lang: 'en' },
  ];
  for (const { item, lang } of cases) {
    const notes = [ENTRY_030, { ...ENTRY_020, sections: [{ kind: 'improvements', items: [{ en: 'Delta', 'zh-TW': '丁' }, item] }] }];
    const repo = renderRepo(notes);
    for (const args of [['--write'], ['--version', '0.3.0'], ['--check']]) {
      const result = repo.run(args);
      assert.equal(result.status, 1, `${JSON.stringify(item)} args=${JSON.stringify(args)}`);
      assert.equal(result.stdout, '');
      for (const needle of ['0.2.0', 'improvements', '第 2 條', lang]) {
        assert.ok(result.stderr.includes(needle), `stderr 應提到 ${needle}：${result.stderr}`);
      }
    }
    assert.ok(!existsSync(repo.changelog) && !existsSync(repo.changelogZh), '形狀不合時不寫出任何檔案');
  }
});

test('真 repo 的 release-notes.json 通過形狀守門且最新在前', () => {
  const entries = readReleaseNotes();
  assert.ok(entries.length >= 1);
  assert.match(entries[0].version, /^\d+\.\d+\.\d+$/);
});
