// @vitest-environment node
// 純檔案系統斷言：以 node 環境執行，import.meta.url 才是 file:// URL。
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { scanSources } from "./sourceScan";

// 共用元件唯一來源（spec desktop-app「共用元件唯一來源」Scenario「同名本地定義被守門擋下」；
// design D9）：@speclink/ui 匯出的名稱，桌面 app 與 server web 不得在原始碼頂層再定義一份——
// 散落的同名元件正是「換皮時每一份都要再改一次」的來源。
const SCAN_ROOTS = ["apps/desktop/src", "apps/server-web/src"];

/** index.ts 匯出的值識別字（`export { A, B as C } from`、`export function A`、`export const A`；略過 type）。 */
function exportedNames(): Set<string> {
  const index = readFileSync(fileURLToPath(new URL("../index.ts", import.meta.url)), "utf8");
  const names = new Set<string>();
  for (const block of index.matchAll(/export\s*\{([^}]*)\}\s*from/g)) {
    for (const raw of block[1].split(",")) {
      const entry = raw.replace(/\/\/[^\n]*/g, "").trim();
      if (!entry || entry.startsWith("type ")) continue;
      names.add(entry.split(/\s+as\s+/).pop()!.trim());
    }
  }
  for (const m of index.matchAll(/^export\s+(?:async\s+)?(?:function|const)\s+([A-Za-z_$][\w$]*)/gm)) {
    names.add(m[1]);
  }
  return names;
}

/** 頂層（行首、不縮排）的函式或常數定義名。 */
const TOP_LEVEL_DEFINITION =
  /^(?:export\s+)?(?:(?:async\s+)?function\s+([A-Za-z_$][\w$]*)\s*[<(]|const\s+([A-Za-z_$][\w$]*)\s*[:=])/gm;

describe("共用元件唯一來源守門", () => {
  it("收得到 @speclink/ui 的匯出名（守門本身不空轉）", () => {
    const names = exportedNames();
    for (const name of ["CopyButton", "EmptyState", "SectionHeader", "ConfirmDialog", "Wordmark"]) {
      expect(names.has(name), name).toBe(true);
    }
    expect(names.has("ConfirmDialogProps")).toBe(false);
  });

  it("桌面 app 與 server web 頂層不定義與 @speclink/ui 匯出同名的函式或常數", () => {
    const names = exportedNames();
    const violations: string[] = [];
    for (const [file, source] of scanSources(SCAN_ROOTS, [".ts", ".tsx"])) {
      for (const m of source.matchAll(TOP_LEVEL_DEFINITION)) {
        const name = m[1] ?? m[2];
        if (names.has(name)) violations.push(`${file}: ${name}`);
      }
    }
    expect(violations).toEqual([]);
  });
});
