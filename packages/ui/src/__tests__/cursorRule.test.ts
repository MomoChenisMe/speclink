// @vitest-environment node
// 純檔案系統斷言：以 node 環境執行，import.meta.url 才是 file:// URL。
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";

import { REPO_ROOT, scanSources } from "./sourceScan";

// 可點元素的游標（spec desktop-app「可點元素的游標與 hover 回饋」；design D1／D2／D6）：
// 游標只在共用樣式的 --cursor-interaction 一處決定（預設手指、桌面覆寫為箭頭），
// 元件寫死手指或禁止游標就會讓桌面與網頁各自漂移，因此一律擋下。
const SCAN_ROOTS = ["packages/ui/src/components", "apps/desktop/src", "apps/server-web/src"];

/** 整字的 cursor-pointer／cursor-not-allowed（含 disabled:、peer-disabled: 等變體前綴）。 */
const HARDCODED_CURSOR = /(?<![\w-])cursor-(?:pointer|not-allowed)(?![\w-])/;

const readCss = (path: string) => readFileSync(`${REPO_ROOT}${path}`, "utf8");

describe("可點元素游標守門", () => {
  it("元件原始碼不寫死手指或禁止游標", () => {
    const violations = scanSources(SCAN_ROOTS, [".ts", ".tsx"])
      .filter(([, source]) => HARDCODED_CURSOR.test(source))
      .map(([file]) => file);
    expect(violations, "改用 cursor-(--cursor-interaction)；停用態用 cursor-default").toEqual([]);
  });

  it("共用樣式宣告游標變數的預設值為手指", () => {
    const css = readCss("packages/ui/src/theme.css");
    expect(css.includes("--cursor-interaction: pointer"), "theme.css 缺 --cursor-interaction: pointer").toBe(true);
  });

  it("共用樣式以 base layer 全域規則讓可用按鈕與 role=button 取用游標變數", () => {
    const base = [...readCss("packages/ui/src/theme.css").matchAll(/@layer base\s*\{([\s\S]*?)\n\}/g)]
      .map((m) => m[1])
      .find((block) => block.includes("cursor: var(--cursor-interaction)"));
    expect(base !== undefined, "theme.css 缺 @layer base 的 cursor: var(--cursor-interaction) 全域規則").toBe(true);
    expect(base!.includes("button:not(:disabled)"), "全域規則選擇器缺 button:not(:disabled)").toBe(true);
    expect(base!.includes('[role="button"]'), '全域規則選擇器缺 [role="button"]').toBe(true);
  });

  it("桌面根樣式把游標變數覆寫為箭頭", () => {
    const css = readCss("apps/desktop/src/index.css");
    expect(css.includes("--cursor-interaction: default"), "apps/desktop/src/index.css 缺 --cursor-interaction: default").toBe(
      true,
    );
  });

  it("網頁後台根樣式不覆寫游標變數", () => {
    const css = readCss("apps/server-web/src/index.css");
    expect(css.includes("--cursor-interaction"), "apps/server-web/src/index.css 不得覆寫 --cursor-interaction").toBe(false);
  });
});
