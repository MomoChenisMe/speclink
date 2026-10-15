#!/usr/bin/env node
// 更新日誌的渲染腳本（release-notes spec「渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段」）。
// 真相只有 apps/desktop/src/release-notes/release-notes.json（每個條目同時存 en 與 zh-TW）；
// 英文 CHANGELOG.md、繁中 CHANGELOG.zh-TW.md 與 Release 說明的兩種語言片段都由這一份邏輯
// 畫出，格式不可能漂移。純 Node、無依賴。
//
// 用法：
//   --write                            讀 JSON，寫出 repo 根 CHANGELOG.md 與 CHANGELOG.zh-TW.md
//   --version X.Y.Z [--lang en|zh-TW]  把該版該語言的片段印到 stdout，預設 en
//                                      （JSON 無此版 → exit 1，stderr 印頂端版號）
//   --check                            重算兩份並各自與現有檔比對（任一份不一致 → exit 1，
//                                      stderr 點名檔名並印差異摘要）
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const NOTES_PATH = path.join(ROOT, 'apps', 'desktop', 'src', 'release-notes', 'release-notes.json');
const USAGE = '用法：release-notes-render.mjs --write | --version X.Y.Z [--lang en|zh-TW] | --check';

/** 分組只有這三種，同一筆內也依此順序排列；標題不存在 JSON，由下方各語言決定。 */
const KINDS = ['features', 'fixes', 'improvements'];

/** 每種語言的檔名、檔頭（標題＋語言切換列）、版號標題寫法與分組標題。 */
const LANGS = {
  en: {
    file: 'CHANGELOG.md',
    header: '# Changelog\n**English** · [繁體中文](CHANGELOG.zh-TW.md)',
    heading: (entry) => `## ${entry.version} (${entry.date})`,
    kinds: { features: 'New features', fixes: 'Fixes', improvements: 'Improvements' },
  },
  'zh-TW': {
    file: 'CHANGELOG.zh-TW.md',
    header: '# 更新日誌\n[English](CHANGELOG.md) · **繁體中文**',
    heading: (entry) => `## ${entry.version}（${entry.date}）`,
    kinds: { features: '新功能', fixes: '修正', improvements: '改善' },
  },
};

/** 單一版本某語言的 markdown 片段：版號（日期）標題加各分組 `### 標題` 與 `- ` 條目；尾端一個換行。 */
export function renderEntry(entry, lang) {
  const { heading, kinds } = LANGS[lang];
  const blocks = [heading(entry)];
  for (const section of entry.sections) {
    blocks.push(`### ${kinds[section.kind]}`);
    blocks.push(section.items.map((item) => `- ${item[lang]}`).join('\n'));
  }
  return `${blocks.join('\n\n')}\n`;
}

/** 某語言的完整更新日誌：標題、次行語言切換，其後依序接每版片段，條目間空一行，檔尾一個換行，一律 LF。 */
export function renderChangelog(entries, lang) {
  return `${LANGS[lang].header}\n\n${entries.map((entry) => renderEntry(entry, lang)).join('\n')}`;
}

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

const VERSION_SHAPE = /^\d+\.\d+\.\d+$/;
const DATE_SHAPE = /^\d{4}-\d{2}-\d{2}$/;

/** 條目形狀守門（release-notes spec「更新日誌 JSON 為唯一真相」）：不合即丟出點名版號與位置的
 * Error；--check 因此也守住真 JSON 的形狀。 */
function assertShape(entries) {
  const bad = (where, why) => new Error(`release-notes.json 形狀錯誤：${where} ${why}`);
  if (!Array.isArray(entries)) throw bad('頂層', '必須是陣列');
  entries.forEach((entry, i) => {
    const at = `第 ${i + 1} 筆（${entry?.version}）`;
    if (!VERSION_SHAPE.test(entry?.version ?? '')) throw bad(at, 'version 必須是 X.Y.Z');
    if (!DATE_SHAPE.test(entry?.date ?? '')) throw bad(at, 'date 必須是 YYYY-MM-DD');
    if (!Array.isArray(entry.sections)) throw bad(at, 'sections 必須是陣列');
    let previous = -1;
    entry.sections.forEach((section) => {
      const order = KINDS.indexOf(section?.kind);
      if (order < 0) throw bad(at, `kind 只能是 ${KINDS.join('／')}`);
      if (order <= previous) throw bad(at, `分組須依 ${KINDS.join('、')} 的順序排列且不得重複`);
      previous = order;
      const items = section.items;
      if (!Array.isArray(items) || items.length === 0) throw bad(at, `${section.kind} 的 items 必須是非空陣列`);
      items.forEach((item, j) => {
        for (const lang of Object.keys(LANGS)) {
          const text = item?.[lang];
          if (typeof text !== 'string' || !text.trim()) {
            throw bad(at, `${section.kind} 第 ${j + 1} 條的 ${lang} 必須是非空字串`);
          }
        }
      });
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

/** 一份更新日誌與重算結果比對；一致回 null，否則回點名該檔的說明。 */
function checkFile(file, expected) {
  let current;
  try {
    current = readFileSync(path.join(ROOT, file), 'utf8');
  } catch {
    return `${file} 不存在。`;
  }
  // Windows checkout 可能被 autocrlf 改寫成 CRLF；比對前正規化，寫出永遠 LF。
  const difference = firstDifference(current.replace(/\r\n/g, '\n'), expected);
  if (!difference) return null;
  return [
    `${file} 與 release-notes.json 重算結果不一致（第 ${difference.line} 行）：`,
    `  ${file}：${difference.actual}`,
    `  重算結果：${difference.expected}`,
  ].join('\n');
}

function main(args) {
  const [command, argument] = args;
  if (command === '--write' && args.length === 1) {
    // 兩份都先算好才寫：形狀錯誤時一份都不寫。
    const entries = readReleaseNotes();
    const outputs = Object.entries(LANGS).map(([lang, { file }]) => [file, renderChangelog(entries, lang)]);
    for (const [file, content] of outputs) writeFileSync(path.join(ROOT, file), content);
    return;
  }
  if (command === '--version' && (args.length === 2 || (args.length === 4 && args[2] === '--lang'))) {
    const lang = args.length === 4 ? args[3] : 'en';
    if (!Object.hasOwn(LANGS, lang)) fail(`--lang 只能是 ${Object.keys(LANGS).join(' 或 ')}（收到「${lang}」）`);
    const entries = readReleaseNotes();
    const entry = entries.find((candidate) => candidate.version === argument);
    if (!entry) {
      fail(`release-notes.json 沒有 ${argument} 的條目（頂端版號為 ${entries[0]?.version ?? '無'}）`);
    }
    process.stdout.write(renderEntry(entry, lang));
    return;
  }
  if (command === '--check' && args.length === 1) {
    // 先讀真相（JSON 壞掉要先說），再逐份比對衍生物；兩份都漂移時一次點名兩份。
    const entries = readReleaseNotes();
    const problems = Object.entries(LANGS)
      .map(([lang, { file }]) => checkFile(file, renderChangelog(entries, lang)))
      .filter(Boolean);
    if (problems.length > 0) {
      fail(
        [
          ...problems,
          `執行 node scripts/release/release-notes-render.mjs --write 重新產出 ${Object.values(LANGS)
            .map(({ file }) => file)
            .join(' 與 ')}。`,
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
