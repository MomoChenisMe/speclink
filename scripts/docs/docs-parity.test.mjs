// 中英對等與正典詞彙的驗收（user-documentation spec「中英文文件保持結構與事實對等」、
// design D5：同結構、同事實）。
// D5 的可機械檢查部分有兩塊：成對文件的 H2 章節序列與截圖引用集合須逐項相同；
// 繁體中文散文須用正典詞彙，引擎動詞只能出現在 code span 內。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const read = (relative) => readFileSync(path.join(ROOT, relative), 'utf8');

/// spec「中英文文件保持結構與事實對等」列出的全部成對文件。
/// remote-getting-started 與 development 的內容檢查另由 remote-docs.test.mjs 負責。
const PAIRS = [
  ['README.md', 'README.en.md'],
  ['docs/getting-started.zh-TW.md', 'docs/getting-started.md'],
  ['docs/workflow.zh-TW.md', 'docs/workflow.md'],
  ['docs/product-status.zh-TW.md', 'docs/product-status.md'],
  ['docs/roadmap.zh-TW.md', 'docs/roadmap.md'],
  ['docs/remote-getting-started.zh-TW.md', 'docs/remote-getting-started.md'],
  ['docs/development.zh-TW.md', 'docs/development.md'],
  ['docs/configuration.zh-TW.md', 'docs/configuration.md'],
  ['docs/verb-contract.zh-TW.md', 'docs/verb-contract.md'],
  ['docs/sdk-node.zh-TW.md', 'docs/sdk-node.md'],
  ['docs/server-deployment.zh-TW.md', 'docs/server-deployment.md'],
  ['docs/server-store-drivers.zh-TW.md', 'docs/server-store-drivers.md'],
  ['docs/server-backup.zh-TW.md', 'docs/server-backup.md'],
];

/// 兩語言的標題文字各寫各的，章節靠行首的 `<a id="..."></a>` 對齊；
/// 缺錨點的標題以原文出列，兩版必然不等而被點名。
const ANCHORED_H2 = /^## <a id="([^"]+)"><\/a>/;

const h2s = (markdown) =>
  markdown
    .split('\n')
    .filter((line) => line.startsWith('## '))
    .map((line) => ANCHORED_H2.exec(line)?.[1] ?? `（缺錨點）${line.slice(3).trim()}`);

const shots = (markdown) => markdown.match(/[\w./-]*assets\/screenshots\/[a-z-]+\.png/g) ?? [];

const cells = (line) =>
  line
    .trim()
    .replace(/^\||\|$/g, '')
    .split('|')
    .map((cell) => cell.trim());

/// 依 H2 錨點分組的表格資料列：表頭（緊接分隔列的那一列）與分隔列不算。
function tableRows(markdown) {
  const sections = new Map();
  for (const chunk of markdown.split(/^## /m).slice(1)) {
    const lines = chunk.split('\n').filter((line) => line.startsWith('|'));
    // 分隔列的每一格都只有 - 與對齊用的 :；只看開頭會把 `| --json |` 這種資料列也吃掉。
    const isSeparator = (line) => /^\|(\s*:?-+:?\s*\|)+\s*$/.test(line);
    const rows = lines.filter((line, index) => !isSeparator(line) && !isSeparator(lines[index + 1] ?? ''));
    sections.set(ANCHORED_H2.exec(`## ${chunk}`)?.[1], rows.map(cells));
  }
  return sections;
}

/// 狀態圖例寫成「Available（可用）」：由它推出中文狀態值對應的英文狀態值。
function statusMapOf(legendRows) {
  return new Map(
    legendRows.map(([label]) => {
      const match = /^(.+?)（(.+)）$/.exec(label);
      assert.ok(match, `狀態圖例「${label}」不是「English（中文）」的寫法`);
      return [match[2], match[1]];
    }),
  );
}

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
  'docs/server-deployment.zh-TW.md',
  'docs/server-store-drivers.zh-TW.md',
  'docs/server-backup.zh-TW.md',
];

for (const [zhPath, enPath] of PAIRS) {
  test(`中英對等：${zhPath} 與 ${enPath} 的 H2 錨點序列逐項相同`, () => {
    assert.deepEqual(h2s(read(zhPath)), h2s(read(enPath)));
  });

  test(`中英對等：${zhPath} 與 ${enPath} 的截圖引用集合與數量相同`, () => {
    const zh = shots(read(zhPath)).map((s) => s.split('/').pop());
    const en = shots(read(enPath)).map((s) => s.split('/').pop());
    assert.deepEqual(zh, en);
  });
}

test('表格列：第一格以 - 開頭的資料列不會被當成分隔列', () => {
  const md = ['## <a id="x"></a>X', '', '| 旗標 | 說明 |', '| --- | --- |', '| --json | 機器輸出 |', '| -y | 略過確認 |'].join('\n');
  assert.deepEqual(tableRows(md).get('x'), [
    ['--json', '機器輸出'],
    ['-y', '略過確認'],
  ]);
});

test('狀態圖例：多個字的狀態名也對得上，寫法不對時點名那一列', () => {
  assert.deepEqual(
    [...statusMapOf([['Available（可用）'], ['Not planned（不規劃）']])],
    [
      ['可用', 'Available'],
      ['不規劃', 'Not planned'],
    ],
  );
  assert.throws(() => statusMapOf([['Available']]), /Available/);
});

test('product-status 中英矩陣：每個章節的表格列數與欄數逐列相同', () => {
  const zh = tableRows(read('docs/product-status.zh-TW.md'));
  const en = tableRows(read('docs/product-status.md'));
  for (const [id, zhRows] of zh) {
    const enRows = en.get(id) ?? [];
    assert.equal(enRows.length, zhRows.length, `章節「${id}」的表格列數不同`);
    zhRows.forEach((row, index) => {
      assert.equal(enRows[index].length, row.length, `章節「${id}」第 ${index + 1} 列「${row[0]}」的欄數不同`);
    });
  }
});

test('product-status 能力清單：每列有證據、限制與下一步及查核日期，中英狀態與日期逐列對應', () => {
  const zh = tableRows(read('docs/product-status.zh-TW.md'));
  const en = tableRows(read('docs/product-status.md'));
  const statusOf = statusMapOf(zh.get('status-model'));
  const zhRows = zh.get('capabilities');
  const enRows = en.get('capabilities');
  assert.ok(zhRows.length > 0, '能力清單沒有任何列');
  const problems = [];
  zhRows.forEach(([name, status, , evidence, limits, checked], index) => {
    const [enName, enStatus, , enEvidence, enLimits, enChecked] = enRows[index];
    if (statusOf.get(status) !== enStatus) problems.push(`${name}：狀態「${status}」對不上英文版「${enStatus}」`);
    if (!evidence || !enEvidence) problems.push(`${name} / ${enName}：缺證據`);
    if (!limits || !enLimits) problems.push(`${name} / ${enName}：缺限制與下一步`);
    if (!/^\d{4}-\d{2}-\d{2}$/.test(checked ?? '')) problems.push(`${name}：查核日期「${checked}」不是 YYYY-MM-DD`);
    if (checked !== enChecked) problems.push(`${name}：查核日期與英文版「${enChecked}」不同`);
  });
  assert.deepEqual(problems, []);
});

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
