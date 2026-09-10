// 更新日誌兩支腳本的測試夾具：腳本由自身位置解析 repo 根（JSON 與 CHANGELOG.md 路徑固定），
// 測試把腳本複製進暫存目錄、自建同樣的目錄形狀——不碰真 repo 的 CHANGELOG.md。
// tmpdir 先 realpath：macOS 的 /var 是 /private/var 的 symlink，而 ESM 載入器會把
// import.meta.url 解到真實路徑，腳本的「直接執行」守門才對得上。
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, realpathSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

/**
 * 建一個暫存 repo：複製指定腳本到 <dir>/scripts/release、把 notes 寫進固定的 JSON 路徑
 * （notes 為字串時原樣寫入，供壞 JSON 情境）。回傳 dir、CHANGELOG.md 路徑與執行器。
 */
export function scratchRepo({ scripts, notes }) {
  const dir = mkdtempSync(path.join(realpathSync(tmpdir()), 'release-notes-'));
  mkdirSync(path.join(dir, 'scripts', 'release'), { recursive: true });
  mkdirSync(path.join(dir, 'apps', 'desktop', 'src', 'release-notes'), { recursive: true });
  for (const name of scripts) {
    copyFileSync(path.join(ROOT, 'scripts', 'release', name), path.join(dir, 'scripts', 'release', name));
  }
  writeFileSync(
    path.join(dir, 'apps', 'desktop', 'src', 'release-notes', 'release-notes.json'),
    typeof notes === 'string' ? notes : `${JSON.stringify(notes, null, 2)}\n`,
  );
  const run = (script, args) =>
    spawnSync(process.execPath, [path.join(dir, 'scripts', 'release', script), ...args], { encoding: 'utf8' });
  return { dir, run, changelog: path.join(dir, 'CHANGELOG.md') };
}
