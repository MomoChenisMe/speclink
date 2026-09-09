#!/usr/bin/env node
// 更新日誌的渲染腳本（release-notes spec「渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段」）。
// 真相只有 apps/desktop/src/release-notes/release-notes.json；CHANGELOG.md 與 Release
// 說明的更新內容片段都由這一份邏輯畫出，兩處格式不可能漂移。純 Node、無依賴。
//
// 用法：
//   --write            讀 JSON，寫出 repo 根 CHANGELOG.md
//   --version X.Y.Z    把該版片段印到 stdout（JSON 無此版 → exit 1，stderr 印頂端版號）
//   --check            重算並與 CHANGELOG.md 比對（不一致 → exit 1，stderr 印差異摘要）
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const NOTES_PATH = path.join(ROOT, 'apps', 'desktop', 'src', 'release-notes', 'release-notes.json');
const CHANGELOG_PATH = path.join(ROOT, 'CHANGELOG.md');
const USAGE = '用法：release-notes-render.mjs --write | --version X.Y.Z | --check';

/** 單一版本的 markdown 片段：`## X.Y.Z（YYYY-MM-DD）` 加各分組 `### 標題` 與 `- ` 條目；尾端一個換行。 */
export function renderEntry(entry) {
  const blocks = [`## ${entry.version}（${entry.date}）`];
  for (const section of entry.sections) {
    blocks.push(`### ${section.title}`);
    blocks.push(section.items.map((item) => `- ${item}`).join('\n'));
  }
  return `${blocks.join('\n\n')}\n`;
}

/** 完整 CHANGELOG.md：首行 `# 更新日誌`，其後依序接每版片段，條目間空一行，檔尾一個換行，一律 LF。 */
export function renderChangelog(entries) {
  return `# 更新日誌\n\n${entries.map(renderEntry).join('\n')}`;
}

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

const SECTION_TITLES = ['新功能', '修正', '改善'];
const VERSION_SHAPE = /^\d+\.\d+\.\d+$/;
const DATE_SHAPE = /^\d{4}-\d{2}-\d{2}$/;

/** 條目形狀守門（release-notes spec「更新日誌 JSON 為唯一真相」Scenario「條目形狀」）：
 * 不合即丟出說明哪裡錯的 Error；--check 因此也守住真 JSON 的形狀。 */
function assertShape(entries) {
  const bad = (where, why) => new Error(`release-notes.json 形狀錯誤：${where} ${why}`);
  if (!Array.isArray(entries)) throw bad('頂層', '必須是陣列');
  entries.forEach((entry, i) => {
    const at = `第 ${i + 1} 筆`;
    if (!VERSION_SHAPE.test(entry?.version ?? '')) throw bad(at, 'version 必須是 X.Y.Z');
    if (!DATE_SHAPE.test(entry?.date ?? '')) throw bad(at, 'date 必須是 YYYY-MM-DD');
    if (!Array.isArray(entry.sections)) throw bad(at, 'sections 必須是陣列');
    entry.sections.forEach((section) => {
      if (!SECTION_TITLES.includes(section?.title)) {
        throw bad(at, `title 只能是 ${SECTION_TITLES.join('／')}`);
      }
      const items = section.items;
      if (!Array.isArray(items) || items.length === 0 || items.some((s) => typeof s !== 'string' || !s)) {
        throw bad(at, `「${section.title}」的 items 必須是非空字串陣列`);
      }
    });
  });
}

/** 讀入全部條目（最新在前）並守門形狀。下載指南腳本也由此取條目，JSON 路徑只在這裡出現一次。 */
export function readReleaseNotes() {
  let parsed;
  try {
    parsed = JSON.parse(readFileSync(NOTES_PATH, 'utf8'));
  } catch (error) {
    throw new Error(`release-notes.json 讀取失敗：${error.message}`);
  }
  assertShape(parsed);
  return parsed;
}

/** 第一個不同的行（1-based）與兩邊該行內容——給人看的差異摘要，不是完整 diff。 */
function firstDifference(actual, expected) {
  const a = actual.split('\n');
  const e = expected.split('\n');
  const count = Math.max(a.length, e.length);
  for (let i = 0; i < count; i += 1) {
    if (a[i] !== e[i]) return { line: i + 1, actual: a[i] ?? '<檔尾>', expected: e[i] ?? '<檔尾>' };
  }
  return null;
}

function main(args) {
  const [command, argument] = args;
  if (command === '--write' && args.length === 1) {
    writeFileSync(CHANGELOG_PATH, renderChangelog(readReleaseNotes()));
    return;
  }
  if (command === '--version' && args.length === 2) {
    const entries = readReleaseNotes();
    const entry = entries.find((candidate) => candidate.version === argument);
    if (!entry) {
      fail(`release-notes.json 沒有 ${argument} 的條目（頂端版號為 ${entries[0]?.version ?? '無'}）`);
    }
    process.stdout.write(renderEntry(entry));
    return;
  }
  if (command === '--check' && args.length === 1) {
    // 先讀真相（JSON 壞掉要先說），再讀衍生物。
    const expected = renderChangelog(readReleaseNotes());
    let current;
    try {
      current = readFileSync(CHANGELOG_PATH, 'utf8');
    } catch {
      fail(`CHANGELOG.md 不存在——執行 node scripts/release-notes-render.mjs --write 產出`);
    }
    // Windows checkout 可能被 autocrlf 改寫成 CRLF；比對前正規化，寫出永遠 LF。
    const normalized = current.replace(/\r\n/g, '\n');
    const difference = firstDifference(normalized, expected);
    if (difference) {
      fail(
        [
          `CHANGELOG.md 與 release-notes.json 重算結果不一致（第 ${difference.line} 行）：`,
          `  CHANGELOG.md：${difference.actual}`,
          `  重算結果　　：${difference.expected}`,
          '執行 node scripts/release-notes-render.mjs --write 重新產出。',
        ].join('\n'),
      );
    }
    return;
  }
  fail(USAGE);
}

// 直接執行才跑 CLI；被 import（測試、release-notes.mjs）時只匯出函式（寫法同 repo 其他腳本）。
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    fail(error.message);
  }
}
