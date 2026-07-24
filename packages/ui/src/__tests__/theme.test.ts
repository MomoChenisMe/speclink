// @vitest-environment node
// 純檔案系統斷言：以 node 環境執行，import.meta.url 才是 file:// URL
// （jsdom 環境會把它換成 http location，導致 fileURLToPath 失敗）。
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// 共用 semantic theme（D1/D7）：青綠 token 從 apps/desktop/src/index.css 抽到
// packages/ui/src/theme.css，Desktop 與 Server Web 各自 import 同一份，證明
// 「抽取只改所有權、不改值」——Desktop 可觀察 token 不得漂移。
const themeCss = () =>
  readFileSync(fileURLToPath(new URL("../theme.css", import.meta.url)), "utf8");
const desktopCss = () =>
  readFileSync(
    fileURLToPath(new URL("../../../../apps/desktop/src/index.css", import.meta.url)),
    "utf8",
  );

// 抽取前 Desktop index.css 內的 canonical token 值（web-service-navigation-redesign
// 明訂本 change 不改色，只改所有權）。逐一釘死，任何值漂移即回歸失敗。
const LIGHT_TOKENS = [
  "--radius: 0.625rem;",
  "--background: oklch(1 0 0);",
  "--foreground: oklch(0.145 0 0);",
  "--card: oklch(1 0 0);",
  "--card-foreground: oklch(0.145 0 0);",
  "--primary: oklch(0.52 0.1 192);",
  "--primary-foreground: oklch(0.985 0 0);",
  "--secondary: oklch(0.97 0 0);",
  "--secondary-foreground: oklch(0.205 0 0);",
  "--muted: oklch(0.97 0 0);",
  "--muted-foreground: oklch(0.556 0 0);",
  "--accent: oklch(0.94 0.035 192);",
  "--accent-foreground: oklch(0.205 0 0);",
  "--destructive: oklch(0.577 0.245 27.325);",
  "--border: oklch(0.922 0 0);",
  "--input: oklch(0.922 0 0);",
  "--ring: oklch(0.62 0.1 192);",
];

const DARK_TOKENS = [
  "--background: oklch(0.155 0.008 260);",
  "--card: oklch(0.205 0.008 260);",
  "--primary: oklch(0.74 0.11 192);",
  "--primary-foreground: oklch(0.16 0.02 210);",
  "--accent: oklch(0.3 0.045 192);",
  "--destructive: oklch(0.62 0.2 25);",
  "--border: oklch(1 0 0 / 12%);",
  "--input: oklch(1 0 0 / 15%);",
  "--ring: oklch(0.64 0.1 192);",
];

const THEME_MAP = [
  "--color-primary: var(--primary);",
  "--color-background: var(--background);",
  "--color-ring: var(--ring);",
  "--radius-md: calc(var(--radius) - 2px);",
];

describe("共用 semantic theme 抽取", () => {
  it("packages/ui/src/theme.css 保留全部 canonical light token 值", () => {
    const css = themeCss();
    for (const token of LIGHT_TOKENS) {
      expect(css).toContain(token);
    }
  });

  it("theme.css 保留 dark 模式 token 與系統偏好查詢", () => {
    const css = themeCss();
    expect(css).toContain("@media (prefers-color-scheme: dark)");
    for (const token of DARK_TOKENS) {
      expect(css).toContain(token);
    }
  });

  it("theme.css 保留 @theme inline 對 Tailwind utility 的映射", () => {
    const css = themeCss();
    expect(css).toContain("@theme inline");
    for (const mapping of THEME_MAP) {
      expect(css).toContain(mapping);
    }
  });

  it("theme.css 保留 Noto Sans TC body 字型堆疊", () => {
    const css = themeCss();
    expect(css).toContain('"Noto Sans TC Variable"');
    expect(css).toContain("font-family:");
  });

  it("Desktop index.css 改為 import 共用 theme，不再自行內嵌 token（單一真相源）", () => {
    const css = desktopCss();
    // 引用共用 theme.css（相對 workspace 路徑，與既有 @source 慣例一致）。
    expect(css).toMatch(/@import\s+["'][^"']*packages\/ui\/src\/theme\.css["']/);
    // 不得再就地宣告青綠主色 token——否則兩處各自維護會漂移。
    expect(css).not.toContain("--primary: oklch(0.52 0.1 192);");
  });
});
