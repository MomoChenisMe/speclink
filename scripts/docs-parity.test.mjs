// 中英對等與正典詞彙的驗收（user-documentation spec「中英文文件保持結構與事實對等」、
// design D5：同結構、同事實）。
// D5 的可機械檢查部分有兩塊：成對文件的 H2 章節序列與截圖引用集合須逐項相同；
// 繁體中文散文須用正典詞彙，引擎動詞只能出現在 code span 內。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = (relative) => readFileSync(path.join(ROOT, relative), 'utf8');

/// D5 點名的五組成對文件。單語的 server 三份不在本要求範圍。
const PAIRS = [
  ['README.md', 'README.en.md'],
  ['docs/getting-started.zh-TW.md', 'docs/getting-started.md'],
  ['docs/workflow.zh-TW.md', 'docs/workflow.md'],
  ['docs/product-status.zh-TW.md', 'docs/product-status.md'],
  ['docs/roadmap.zh-TW.md', 'docs/roadmap.md'],
];

const h2s = (markdown) =>
  markdown
    .split('\n')
    .filter((line) => line.startsWith('## '))
    .map((line) => line.slice(3).trim());

const shots = (markdown) => markdown.match(/[\w./-]*assets\/screenshots\/[a-z-]+\.png/g) ?? [];

/// 移除 fenced block 與行內 code span——引擎動詞、CLI 命令與欄位名留在 code span
/// 內是正典允許的，只有散文裡的裸用才算違規。
function prose(markdown) {
  return markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`\n]*`/g, '');
}

/// 正典詞彙的避免詞中，語意不依賴上下文、可直接機械判定的那些。
/// （`進行中`、`分頁`、`context` 等避免詞綁在其他正典詞的語境上，看板欄名與
///  一般用語都會誤命中，不納入機械檢查。）
const AVOID = ['促轉', '已促轉', '再促轉', '促轉分頁', '歸檔', '撤回開工', '取消開工'];

/// spec 的場景點名的三個詞：繁中散文須寫「轉為變更」「已轉出變更」「封存」，
/// 英文動詞只准出現在 CLI／欄位／code span 或必要的引擎動詞對照中。
/// 對照（站別章節、表格的動詞欄、雙語標題）是明文允許的，因此只掃純散文行。
const PROSE_FORBIDDEN = ['promote', 'promoted'];

const ZH_DOCS = [
  'README.md',
  'docs/getting-started.zh-TW.md',
  'docs/workflow.zh-TW.md',
  'docs/product-status.zh-TW.md',
  'docs/roadmap.zh-TW.md',
  'docs/remote-getting-started.zh-TW.md',
  'docs/configuration.zh-TW.md',
  'docs/verb-contract.zh-TW.md',
  'docs/sdk-node.zh-TW.md',
  'docs/development.zh-TW.md',
];

for (const [zhPath, enPath] of PAIRS) {
  test(`中英對等：${zhPath} 與 ${enPath} 的 H2 章節序列逐項相同`, () => {
    assert.deepEqual(h2s(read(zhPath)), h2s(read(enPath)));
  });

  test(`中英對等：${zhPath} 與 ${enPath} 的截圖引用集合與數量相同`, () => {
    const zh = shots(read(zhPath)).map((s) => s.split('/').pop());
    const en = shots(read(enPath)).map((s) => s.split('/').pop());
    assert.deepEqual(zh, en);
  });
}

test('正典詞彙：繁中文件不使用避免詞', () => {
  const hits = [];
  for (const relative of ZH_DOCS) {
    prose(read(relative))
      .split('\n')
      .forEach((line, index) => {
        for (const word of AVOID) {
          if (line.includes(word)) hits.push(`${relative}:${index + 1} 「${word}」`);
        }
      });
  }
  assert.deepEqual(hits, []);
});

test('`promote` 系列不裸用於繁中散文（散文用「轉為變更」「已轉出變更」）', () => {
  const hits = [];
  for (const relative of ZH_DOCS) {
    prose(read(relative))
      .split('\n')
      .forEach((line, index) => {
        // 標題與表格列是明文允許的「引擎動詞對照」，只掃純散文行。
        if (line.startsWith('#') || line.trim().startsWith('|')) return;
        if (!/[一-鿿]/.test(line)) return;
        for (const verb of PROSE_FORBIDDEN) {
          if (new RegExp(`(?<![\\w-/])${verb}(?![\\w-])`, 'i').test(line)) {
            hits.push(`${relative}:${index + 1} 「${verb}」 → ${line.trim().slice(0, 60)}`);
          }
        }
      });
  }
  assert.deepEqual(hits, []);
});

test('繁中散文使用正典詞彙「轉為變更」「已轉出變更」「封存」', () => {
  const workflow = read('docs/workflow.zh-TW.md');
  for (const term of ['轉為變更', '已轉出變更', '封存']) {
    assert.ok(workflow.includes(term), `工作流正典缺少正典詞彙「${term}」`);
  }
});
