// 使用者可見繁中文案的正典詞彙守門（ui-copy-vocabulary spec、design D4）。
// 守門詞由 openspec/LANGUAGE.md 動態解析——新增詞條自動生效，這是「擋住新的越界」
// 而非「追討存量」：規格散文、程式碼註解與已封存內容都不在掃描面內。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = (relative) => readFileSync(path.join(ROOT, relative), 'utf8');

/// D4 界定的「使用者可見文案面」：使用者真的會看到這些字。
const SURFACE_FILES = [
  'apps/desktop/src/i18n/messages.ts',
  'apps/server-web/src/i18n/messages.ts',
  'README.md',
  'README.en.md',
];
const SURFACE_DIRS = ['crates/speclink-core/assets/skills', 'docs'];

/// 目錄下（含子目錄）全部 `.md`，回傳 ROOT 相對路徑並以正斜線表示。
function markdownsUnder(relativeDir) {
  const found = [];
  const walk = (relative) => {
    for (const entry of readdirSync(path.join(ROOT, relative), { withFileTypes: true })) {
      const child = `${relative}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else if (entry.name.endsWith('.md')) found.push(child);
    }
  };
  walk(relativeDir);
  return found.sort();
}

const surface = () => [...SURFACE_FILES, ...SURFACE_DIRS.flatMap(markdownsUnder)].sort();

/// LANGUAGE.md 各詞條 `avoid` 欄的守門詞。兩道過濾：
/// (1) 帶括號限定者依限定語境分流——限定為「使用者可見文案中」者，限定範圍正是
///     守門面本身，剝掉限定後納入守門；其他語境限定（`背景（此概念上）`、
///     `分頁（pagination 語意上）`）綁在比守門面更窄的語境，機械掃描必然誤命中，
///     比照 docs-parity.test.mjs 排除；
/// (2) 只取含中日韓文字者——ASCII 識別符（RichDetailDrawer、archivedDrawerBase）
///     與英文文案永遠不匹配，因此不需要任何識別符白名單。
const SURFACE_QUALIFIER = '使用者可見文案中';

function avoidTerms(languageMarkdown) {
  const terms = [];
  for (const line of languageMarkdown.split(/\r?\n/)) {
    const listed = /^- \*\*avoid\*\*:\s*(.+)$/.exec(line);
    if (!listed) continue;
    for (const raw of listed[1].split(/[、,]/)) {
      let term = raw.trim();
      const qualified = /^(.*?)[（(]([^（()）]*)[）)]$/.exec(term);
      if (qualified) {
        if (qualified[2] !== SURFACE_QUALIFIER) continue;
        term = qualified[1].trim();
      }
      if (!/[一-鿿]/.test(term)) continue;
      terms.push(term);
    }
  }
  return terms;
}

/// 單一檔案內容的命中清單。行號由 1 起算，LF 與 CRLF 同一套判定。
function scanText(relative, text, terms) {
  const hits = [];
  text.split(/\r?\n/).forEach((line, index) => {
    for (const term of terms) {
      if (line.includes(term)) hits.push(`${relative}:${index + 1} 「${term}」`);
    }
  });
  return hits;
}

const scanFiles = (files, terms) => files.flatMap((relative) => scanText(relative, read(relative), terms));

const TERMS = avoidTerms(read('openspec/LANGUAGE.md'));

test('詞彙守門：使用者可見文案面不出現 LANGUAGE.md 的 avoid 詞', () => {
  const hits = scanFiles(surface(), TERMS);
  assert.deepEqual(hits, [], `使用者可見文案面出現正典避免詞：\n${hits.join('\n')}`);
});

test('守門詞取自 LANGUAGE.md 的 avoid 欄，含本次收斂的兩個舊詞', () => {
  assert.ok(TERMS.includes('抽屜'), '「詳情面板」詞條的 avoid 未被解析出來');
  assert.ok(TERMS.includes('品質站'), '「品質關卡」詞條的 avoid 未被解析出來');
});

test('帶括號限定的 avoid 詞不入守門集（綁語境，機械掃描必誤命中）', () => {
  for (const contextual of ['背景', '分頁', '規則', '進行中']) {
    assert.ok(!TERMS.includes(contextual), `「${contextual}」是語境限定詞，不該進守門集`);
  }
});

test('限定語境即守門面（「使用者可見文案中」）的 avoid 詞剝掉限定後入守門集', () => {
  for (const surfaceScoped of ['覆審', '專案 ID', '子 change']) {
    assert.ok(TERMS.includes(surfaceScoped), `「${surfaceScoped}」的限定語境就是守門面，剝掉限定後該進守門集`);
  }
});

test('純 ASCII 的 avoid 詞不入守門集（只守中日韓詞彙）', () => {
  for (const ascii of ['promote', 'PAT', 'Store']) {
    assert.ok(!TERMS.includes(ascii), `「${ascii}」非中日韓詞彙，不該進守門集`);
  }
});

test('面內植入 avoid 詞時命中，並指出檔案、行號與命中的詞', () => {
  const planted = ['export const messages = {', '  "tour.navUsers.hint": "邀請與編輯都在右側抽屜完成",', '};'].join('\n');
  assert.deepEqual(scanText('apps/server-web/src/i18n/messages.ts', planted, TERMS), [
    'apps/server-web/src/i18n/messages.ts:2 「抽屜」',
  ]);
});

test('ASCII 識別符不觸發誤判', () => {
  const identifiers = ['import { RichDetailDrawer } from "./RichDetailDrawer";', 'const archivedDrawerBase = css`position: fixed`;', 'export type SpecDrawerProps = { open: boolean };'].join('\n');
  assert.deepEqual(scanText('apps/desktop/src/App.tsx', identifiers, TERMS), []);
});

test('英文文案不受詞彙約束', () => {
  const english = ['"store.reviewActionUnsupported": "this workspace cannot settle quality-station tickets",', '"tour.navUsers.hint": "invite and edit inside the right-hand drawer",'].join('\n');
  assert.deepEqual(scanText('apps/desktop/src/i18n/messages.ts', english, TERMS), []);
});

test('CRLF 與 LF 行尾的判定結果相同', () => {
  const lines = ['第一行乾淨', '第二行也乾淨', '第三行有抽屜'];
  const lf = scanText('docs/x.md', lines.join('\n'), TERMS);
  const crlf = scanText('docs/x.md', lines.join('\r\n'), TERMS);
  assert.deepEqual(crlf, lf);
  assert.deepEqual(lf, ['docs/x.md:3 「抽屜」']);
});

test('LANGUAGE.md 正典自身與 openspec/specs 的存量不在掃描面內', () => {
  const scanned = surface();
  assert.ok(!scanned.includes('openspec/LANGUAGE.md'), '正典自身入面會使 avoid 欄違規');
  assert.ok(
    !scanned.some((relative) => relative.startsWith('openspec/')),
    'openspec/ 底下的規格散文與封存內容不在掃描面內',
  );
  // 正典與規格散文都還帶著存量舊詞——它們被排除，不是剛好乾淨。
  assert.ok(scanText('openspec/LANGUAGE.md', read('openspec/LANGUAGE.md'), TERMS).length > 0);
  assert.ok(
    scanText('openspec/specs/desktop-app/spec.md', read('openspec/specs/desktop-app/spec.md'), TERMS).length > 0,
  );
});
