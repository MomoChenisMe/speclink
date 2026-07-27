// @vitest-environment node
// 純檔案系統斷言：以 node 環境執行，import.meta.url 才是 file:// URL。
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// spec 需求「共用設計系統維持高密度可存取體驗」的 Scenario「reduced motion 停用轉場」：
// 作業系統設定 `prefers-reduced-motion: reduce` 時 SPA 不執行任何轉場動畫。
//
// 這條只能在樣式層釘住：動畫散在 Tailwind 的 transition-colors／animate-* utility 裡，
// 逐處加 `motion-reduce:` 一定會漏（而且漏掉時完全沒有訊號）。改以一條全域守門，
// 並用這支測試確保它不會在某次 CSS 整理中被順手刪掉。
const css = () => readFileSync(fileURLToPath(new URL("../index.css", import.meta.url)), "utf8");

describe("reduced motion", () => {
  it("index.css 帶 prefers-reduced-motion 的全域守門", () => {
    const source = css();
    expect(source).toContain("@media (prefers-reduced-motion: reduce)");
    // 轉場與動畫都要壓掉：Sheet 用 animation，按鈕與導覽項用 transition。
    expect(source).toMatch(/animation-duration:\s*0\.01ms\s*!important/);
    expect(source).toMatch(/transition-duration:\s*0\.01ms\s*!important/);
    // 守門要涵蓋全部元素，逐處掛 motion-reduce: 一定會漏。
    expect(source).toMatch(/\*\s*,\s*\*::before\s*,\s*\*::after/);
  });
});
