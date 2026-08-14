#!/usr/bin/env node
// 使用者文件的連結完整性檢查（user-documentation spec「文件內部連結全部可解析」）。
// 掃出兩份 README 與 docs/ 下每一份 markdown 的相對連結與圖片路徑，逐一確認檔案
// 存在；任一斷鏈即以非零結束並點名該路徑。
//
// 用法：node scripts/docs-links.mjs
// 也由 scripts/docs-links.test.mjs 帶進 scripts 測試面，改文件時就會擋下來。
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/// 檢查面：入口的兩份 README，加上 docs/ 底下全部 markdown。
const README_FILES = ['README.md', 'README.en.md'];
const DOCS_DIR = 'docs';

/// markdown 的 [text](target) 與 ![alt](target)。
const MARKDOWN_LINK = /!?\[[^\]]*\]\(([^)\s]+)/g;
/// README 的品牌圖與語言切換寫成 HTML，同樣要檢查。
const HTML_ATTR = /(?:src|href)="([^"]+)"/g;
/// 不在檢查範圍：外部網址、mailto、以及只指向本檔章節的錨點。
const EXTERNAL = /^(?:[a-z][a-z0-9+.-]*:|\/\/|#)/i;

/// 逐行抽取本地目標，附上行號——報斷鏈時要能直接跳到出事的那一行。
export function extractTargets(markdown) {
  const found = [];
  markdown.split('\n').forEach((text, index) => {
    for (const pattern of [MARKDOWN_LINK, HTML_ATTR]) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(text)) !== null) {
        const target = match[1];
        if (!EXTERNAL.test(target)) found.push({ target, line: index + 1 });
      }
    }
  });
  return found;
}

/// 相對路徑以「該檔所在目錄」為基準；錨點與 query 不是檔名的一部分，先剝掉。
/// 空格等字元在 markdown 裡以百分號編碼出現，比對檔案前要還原。
export function resolveTarget({ fromFile, target, root }) {
  const bare = target.split('#')[0].split('?')[0];
  let decoded = bare;
  try {
    decoded = decodeURIComponent(bare);
  } catch {
    // 不是合法的百分號編碼就照字面看待——那本來就是檔名的一部分。
  }
  return path.resolve(root, path.dirname(fromFile), decoded);
}

export function brokenIn({ file, markdown, root, exists = existsSync }) {
  return extractTargets(markdown)
    .map(({ target, line }) => ({ file, line, target, resolved: resolveTarget({ fromFile: file, target, root }) }))
    .filter((entry) => !exists(entry.resolved));
}

/// 檢查面清單，路徑相對版本庫根目錄。
export function docFilesIn(root) {
  const docs = readdirSync(path.join(root, DOCS_DIR), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.md'))
    .map((entry) => path.relative(root, path.join(entry.parentPath ?? entry.path, entry.name)))
    .map((relative) => relative.split(path.sep).join('/'))
    .sort();
  return [...README_FILES, ...docs];
}

export function scanDocs(root) {
  return docFilesIn(root).flatMap((file) =>
    brokenIn({ file, markdown: readFileSync(path.join(root, file), 'utf8'), root }),
  );
}

function main() {
  const broken = scanDocs(ROOT);
  const files = docFilesIn(ROOT);

  if (broken.length === 0) {
    console.log(`✓ ${files.length} 份使用者文件的相對連結與圖片路徑全部指得到檔案。`);
    return;
  }

  console.error(`✗ ${broken.length} 條斷鏈：`);
  for (const entry of broken) {
    console.error(`  ${entry.file}:${entry.line} → ${entry.target}`);
    console.error(`      指向不存在的 ${path.relative(ROOT, entry.resolved)}`);
  }
  process.exit(1);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main();
}
