// 守門測試共用的原始碼掃描（theme.test.ts 原生色階守門、uiSingleSource.test.ts 同名定義守門）。
// 以 import.meta.url 定位 repo 根：呼叫端須以 node 環境執行（jsdom 會把它換成 http location）。
import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

/** repo 根目錄（結尾帶 /）。 */
export const REPO_ROOT = fileURLToPath(new URL("../../../../", import.meta.url));

const SKIPPED_DIRS = new Set(["__tests__", "dist", "node_modules"]);

/** 掃描根（相對 repo 根）下副檔名符合的原始碼，回傳 [repo 相對路徑, 內容]；略過測試、建置產物與相依。 */
export function scanSources(roots: readonly string[], extensions: readonly string[]): [string, string][] {
  const files: string[] = [];
  const walk = (path: string) => {
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      if (SKIPPED_DIRS.has(entry.name)) continue;
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else if (extensions.some((ext) => entry.name.endsWith(ext))) files.push(child);
    }
  };
  for (const root of roots) walk(`${REPO_ROOT}${root}`);
  return files.map((f) => [f.slice(REPO_ROOT.length), readFileSync(f, "utf8")]);
}
