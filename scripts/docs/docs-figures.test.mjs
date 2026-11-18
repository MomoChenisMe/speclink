// 文件圖解的守門（user-documentation spec「圖解以自帶樣式的 SVG 呈現」、design D7／D9）。
// GitHub 會把 SVG 當圖片載入：script、foreignObject 與外部資源都不會生效，沒有背景的圖在
// 深色主題會看不清楚。這裡把這幾件事擋在 scripts 測試面，另外確認兩個語言各有同一組圖、
// 每張圖都有同語言的文件引用。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { docFilesIn, extractTargets, resolveTarget } from './docs-links.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

const FIGURE_DIR = 'docs/assets/figures';
const FIGURE_LOCALES = ['en', 'zh-TW'];
/// spec 點名的五張圖。清單寫死：少一張或多一張都要回來改規格。
const FIGURES = [
  'embedded-shape.svg',
  'sdk-contents.svg',
  'shared-host-kit.svg',
  'skill-path-vs-verb.svg',
  'verb-four-steps.svg',
];

/// 語言資料夾裡的 SVG 檔名。自己排序，不依賴作業系統列目錄的順序；資料夾不存在時回空陣列，
/// 由斷言點名缺了哪幾張。
function figuresIn(locale) {
  const dir = path.join(ROOT, FIGURE_DIR, locale);
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((name) => name.endsWith('.svg'))
    .sort();
}

const attr = (tag, name) => new RegExp(`\\s${name}="([^"]*)"`).exec(tag)?.[1];

/// 第一個繪圖元素：略過註解、`<title>`、`<desc>`、`<defs>` 與 `<style>`——它們不畫東西。
function firstDrawnTag(svg) {
  const body = svg
    .slice(svg.indexOf('>', svg.indexOf('<svg')) + 1)
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/<(title|desc|defs|style)\b[\s\S]*?<\/\1>/g, '');
  return /<[a-zA-Z][^>]*>/.exec(body)?.[0] ?? '';
}

/// 一張圖違反的規則；空陣列表示合格。
function svgProblems(svg) {
  const problems = [];
  const root = /<svg\b[^>]*>/.exec(svg)?.[0];
  if (!root) return ['不是 SVG'];

  if (/<script\b/i.test(svg)) problems.push('含 script');
  if (/<foreignObject\b/i.test(svg)) problems.push('含 foreignObject');
  // 外部資源：指向檔案或網址的 href（只准 `#id` 的內部參照）、url(...) 裡不是 `#id` 的、
  // @import 與 <image>。xmlns 的網址是命名空間，不是資源。
  if (/\b(?:xlink:)?href="(?!#)[^"]*"/i.test(svg)) problems.push('含外部的 href');
  if (/url\(\s*['"]?(?!#)/i.test(svg)) problems.push('含外部的 url()');
  if (/@import/i.test(svg)) problems.push('含 @import');
  if (/<image\b/i.test(svg)) problems.push('含 image');

  if (!attr(root, 'aria-label')) problems.push('缺 aria-label');
  if (attr(root, 'role') !== 'img') problems.push('缺 role="img"');

  const viewBox = attr(root, 'viewBox')?.trim().split(/\s+/);
  if (!viewBox || viewBox.length !== 4) {
    problems.push('缺 viewBox');
    return problems;
  }
  const [, , width, height] = viewBox;
  const first = firstDrawnTag(svg);
  const covers = (value, full) => value === full || value === '100%';
  const atOrigin = (value) => value === undefined || value === '0';
  const fill = attr(first, 'fill');
  if (
    !first.startsWith('<rect') ||
    !covers(attr(first, 'width'), width) ||
    !covers(attr(first, 'height'), height) ||
    !atOrigin(attr(first, 'x')) ||
    !atOrigin(attr(first, 'y')) ||
    !fill ||
    fill === 'none' ||
    fill === 'transparent'
  ) {
    problems.push('第一個繪圖元素不是鋪滿的背景矩形');
  }
  return problems;
}

const GOOD = [
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 300" role="img" aria-label="示意">',
  '<title>示意</title>',
  '<defs><marker id="arrow"><path d="M0 0L6 3L0 6z"/></marker></defs>',
  '<style>text { font-family: sans-serif; }</style>',
  '<rect width="760" height="300" fill="#ffffff"/>',
  '<line x1="0" y1="0" x2="10" y2="10" marker-end="url(#arrow)"/>',
  '</svg>',
].join('\n');

test('檢查：合格的圖沒有任何問題（內部的 url(#id) 與 xmlns 不算外部資源）', () => {
  assert.deepEqual(svgProblems(GOOD), []);
});

test('檢查：script、foreignObject 與外部資源各自被點名', () => {
  const planted = (extra) => GOOD.replace('</svg>', `${extra}</svg>`);
  assert.deepEqual(svgProblems(planted('<script>alert(1)</script>')), ['含 script']);
  assert.deepEqual(svgProblems(planted('<foreignObject><p>x</p></foreignObject>')), ['含 foreignObject']);
  assert.deepEqual(svgProblems(planted('<use href="other.svg#a"/>')), ['含外部的 href']);
  assert.deepEqual(svgProblems(planted('<a xlink:href="https://example.com"><text>x</text></a>')), ['含外部的 href']);
  assert.deepEqual(svgProblems(planted('<rect fill="url(https://example.com/p.svg#x)"/>')), ['含外部的 url()']);
  assert.deepEqual(svgProblems(planted('<image width="1" height="1"/>')), ['含 image']);
  assert.deepEqual(
    svgProblems(GOOD.replace('<style>', '<style>@import "fonts.css";')),
    ['含 @import'],
  );
});

test('檢查：沒有背景、背景沒鋪滿或透明時被點名', () => {
  const background = '<rect width="760" height="300" fill="#ffffff"/>';
  const problem = ['第一個繪圖元素不是鋪滿的背景矩形'];
  assert.deepEqual(svgProblems(GOOD.replace(background, '')), problem);
  assert.deepEqual(svgProblems(GOOD.replace(background, '<rect width="700" height="300" fill="#ffffff"/>')), problem);
  assert.deepEqual(svgProblems(GOOD.replace(background, '<rect width="760" height="300" fill="none"/>')), problem);
  assert.deepEqual(svgProblems(GOOD.replace(background, '<rect x="10" width="760" height="300" fill="#fff"/>')), problem);
  assert.deepEqual(svgProblems(GOOD.replace(background, '<rect width="100%" height="100%" fill="#fff"/>')), []);
});

test('檢查：缺 aria-label 或 role 時被點名', () => {
  assert.deepEqual(svgProblems(GOOD.replace(' aria-label="示意"', '')), ['缺 aria-label']);
  assert.deepEqual(svgProblems(GOOD.replace(' role="img"', '')), ['缺 role="img"']);
});

for (const locale of FIGURE_LOCALES) {
  test(`版本庫實況：${FIGURE_DIR}/${locale}/ 剛好有規格列的五張圖`, () => {
    assert.deepEqual(figuresIn(locale), FIGURES);
  });
}

test('版本庫實況：每張圖都自帶背景與說明，沒有 script、foreignObject 與外部資源', () => {
  const problems = FIGURE_LOCALES.flatMap((locale) =>
    figuresIn(locale).flatMap((name) => {
      const relative = `${FIGURE_DIR}/${locale}/${name}`;
      return svgProblems(readFileSync(path.join(ROOT, relative), 'utf8')).map((problem) => `${relative}：${problem}`);
    }),
  );
  assert.deepEqual(problems, []);
});

/// 文件的語言看檔名：`.zh-TW.md` 是繁中，其餘是英文。
const docLocale = (file) => (file.endsWith('.zh-TW.md') ? 'zh-TW' : 'en');

test('版本庫實況：每張圖在每個語言都被同語言的 README 或 docs/ 文件引用', () => {
  const referenced = new Set(
    docFilesIn(ROOT).flatMap((file) =>
      extractTargets(readFileSync(path.join(ROOT, file), 'utf8')).map(
        ({ target }) => `${docLocale(file)}:${resolveTarget({ fromFile: file, target, root: ROOT })}`,
      ),
    ),
  );
  const orphans = FIGURE_LOCALES.flatMap((locale) =>
    FIGURES.map((name) => `${locale}/${name}`).filter(
      (relative) => !referenced.has(`${locale}:${path.join(ROOT, FIGURE_DIR, relative)}`),
    ),
  );
  assert.deepEqual(orphans, [], `沒有同語言文件引用的圖：${orphans.join('、')}`);
});
