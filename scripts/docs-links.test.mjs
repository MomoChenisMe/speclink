// 使用者文件的連結完整性（user-documentation spec「文件內部連結全部可解析」）。
// 純函式部分以字串驗證；最後一條掃過版本庫實況，讓斷鏈在 scripts 測試面就擋下來。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { brokenIn, docFilesIn, extractTargets, resolveTarget, scanDocs } from './docs-links.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('抽取：markdown 連結與圖片都算，行號指向出現的那一行', () => {
  const md = ['# T', '', 'see [guide](docs/guide.md)', '', '![shot](assets/a.png)'].join('\n');
  assert.deepEqual(extractTargets(md), [
    { target: 'docs/guide.md', line: 3 },
    { target: 'assets/a.png', line: 5 },
  ]);
});

test('抽取：HTML 的 src 與 href 也算（README 的品牌圖與語言切換就是這種寫法）', () => {
  const md = '<img src="docs/assets/brand/logo.png" alt="x" />\n<a href="README.en.md">English</a>\n';
  assert.deepEqual(extractTargets(md), [
    { target: 'docs/assets/brand/logo.png', line: 1 },
    { target: 'README.en.md', line: 2 },
  ]);
});

test('抽取：外部網址、mailto 與純錨點不在檢查範圍', () => {
  const md = [
    '[a](https://example.com)',
    '[b](http://example.com)',
    '[c](mailto:x@example.com)',
    '[d](#section)',
    '[e](docs/real.md)',
  ].join('\n');
  assert.deepEqual(extractTargets(md), [{ target: 'docs/real.md', line: 5 }]);
});

test('解析：以該檔所在目錄為基準，不是以版本庫根目錄', () => {
  assert.equal(
    resolveTarget({ fromFile: 'docs/workflow.zh-TW.md', target: 'getting-started.zh-TW.md', root: ROOT }),
    path.join(ROOT, 'docs/getting-started.zh-TW.md'),
  );
  assert.equal(
    resolveTarget({ fromFile: 'docs/workflow.zh-TW.md', target: '../README.md', root: ROOT }),
    path.join(ROOT, 'README.md'),
  );
});

test('解析：錨點與 query 先剝掉再看檔案存不存在', () => {
  const expected = path.join(ROOT, 'docs/workflow.md');
  assert.equal(resolveTarget({ fromFile: 'README.md', target: 'docs/workflow.md#apply', root: ROOT }), expected);
  assert.equal(resolveTarget({ fromFile: 'README.md', target: 'docs/workflow.md?plain=1', root: ROOT }), expected);
});

test('解析：百分號編碼還原成真實檔名', () => {
  assert.equal(
    resolveTarget({ fromFile: 'README.md', target: 'docs/a%20b.md', root: ROOT }),
    path.join(ROOT, 'docs/a b.md'),
  );
});

test('比對：只回報指不到檔案的那些，並帶上檔名、行號與原始寫法', () => {
  const md = ['[ok](good.md)', '[bad](missing.md)'].join('\n');
  const broken = brokenIn({
    file: 'docs/x.md',
    markdown: md,
    root: ROOT,
    exists: (target) => target === path.join(ROOT, 'docs/good.md'),
  });
  assert.equal(broken.length, 1);
  assert.deepEqual(
    { file: broken[0].file, line: broken[0].line, target: broken[0].target },
    { file: 'docs/x.md', line: 2, target: 'missing.md' },
  );
  assert.equal(broken[0].resolved, path.join(ROOT, 'docs/missing.md'));
});

test('比對：全部指得到時回空陣列', () => {
  assert.deepEqual(brokenIn({ file: 'docs/x.md', markdown: '[ok](good.md)', root: ROOT, exists: () => true }), []);
});

test('檢查面：兩份 README 與 docs 下每一份 markdown 都在掃描範圍內', () => {
  const files = docFilesIn(ROOT);
  assert.ok(files.includes('README.md'));
  assert.ok(files.includes('README.en.md'));
  assert.ok(files.includes('docs/workflow.zh-TW.md'));
  assert.ok(
    files.filter((f) => f.startsWith('docs/')).length >= 20,
    `docs 下的文件應全數納入，實得 ${files.filter((f) => f.startsWith('docs/')).length} 份`,
  );
  assert.ok(!files.some((f) => f.includes('node_modules')));
});

test('版本庫實況：使用者文件沒有任何斷鏈', () => {
  const broken = scanDocs(ROOT);
  assert.deepEqual(
    broken.map((b) => `${b.file}:${b.line} → ${b.target}`),
    [],
  );
});
