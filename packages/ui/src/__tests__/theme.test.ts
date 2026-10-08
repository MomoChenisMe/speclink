// @vitest-environment node
// 純檔案系統斷言：以 node 環境執行，import.meta.url 才是 file:// URL
// （jsdom 環境會把它換成 http location，導致 fileURLToPath 失敗）。
import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { scanSources } from "./sourceScan";

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

// canonical token 值（desktop-design-foundation D1／D3）：色值只在 theme.css，
// 逐一釘死，任何值漂移即回歸失敗。原生色階值逐字取自 tailwindcss/theme.css。
const LIGHT_TOKENS = [
  "--background: oklch(1 0 0);",
  "--foreground: oklch(0.145 0 0);",
  "--card: oklch(1 0 0);",
  "--card-foreground: oklch(0.145 0 0);",
  "--primary: oklch(0.52 0.085 190);",
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
  "--sidebar: oklch(0.985 0 0);",
  "--status-progress: oklch(58.8% 0.158 241.966);",
  "--status-success: oklch(59.6% 0.145 163.225);",
  "--status-warning: oklch(66.6% 0.179 58.318);",
  "--stamp: oklch(54.1% 0.281 293.009);",
  "--improve: oklch(51.1% 0.262 276.966);",
  "--stage-discussion: oklch(59.1% 0.293 322.896);",
  "--stage-proposed: var(--primary);",
  "--stage-in-progress: var(--status-progress);",
  "--stage-ready: var(--status-success);",
];

const DARK_TOKENS = [
  "--background: oklch(0.145 0 0);",
  "--card: oklch(0.205 0 0);",
  "--secondary: oklch(0.269 0 0);",
  "--muted: oklch(0.269 0 0);",
  "--primary: oklch(0.72 0.1 190);",
  "--primary-foreground: oklch(0.16 0.02 210);",
  "--accent: oklch(0.3 0.045 192);",
  "--destructive: oklch(0.62 0.2 25);",
  "--border: oklch(1 0 0 / 12%);",
  "--input: oklch(1 0 0 / 15%);",
  "--ring: oklch(0.64 0.1 192);",
  "--sidebar: oklch(0.185 0 0);",
  "--status-progress: oklch(74.6% 0.16 232.661);",
  "--status-success: oklch(76.5% 0.177 163.223);",
  "--status-warning: oklch(76.9% 0.188 70.08);",
  "--stamp: oklch(70.2% 0.183 293.541);",
  "--improve: oklch(67.3% 0.182 276.935);",
  "--stage-discussion: oklch(74% 0.238 322.16);",
];

/** 新增 token 都要映射成 Tailwind utility（bg-status-progress、text-stamp…）。 */
const NEW_TOKENS = [
  "sidebar",
  "status-progress",
  "status-success",
  "status-warning",
  "stamp",
  "improve",
  "stage-discussion",
  "stage-proposed",
  "stage-in-progress",
  "stage-ready",
];

const THEME_MAP = [
  "--color-primary: var(--primary);",
  "--color-background: var(--background);",
  "--color-ring: var(--ring);",
  ...NEW_TOKENS.map((name) => `--color-${name}: var(--${name});`),
];

// 元件只能用這份 theme 真的映射出來的語意色。用了沒定義的 token（例如上游 shadcn 的
// `bg-popover`——這個 theme 沒有 `--popover`）Tailwind 會產出一條解析不到值的宣告，
// 元件靜默變透明：不會有編譯錯誤、不會有測試紅燈，只有肉眼看得到。
const componentSources = () => {
  const dir = fileURLToPath(new URL("..", import.meta.url));
  const files: string[] = [];
  const walk = (path: string) => {
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      if (entry.name === "__tests__") continue;
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else if (entry.name.endsWith(".tsx")) files.push(child);
    }
  };
  walk(dir);
  return files.map((f) => [f, stripComments(readFileSync(f, "utf8"))] as const);
};

/** Tailwind 內建色名（逐字取自 tailwindcss/theme.css 的 --color-*-500）。 */
const PALETTE_NAMES =
  "red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|slate|gray|zinc|neutral|stone|mauve|olive|mist|taupe";
/** Tailwind 內建色階（bg-amber-500 之類）不經 theme，排除。 */
const PALETTE = new RegExp(`^(?:${PALETTE_NAMES})-\\d{2,3}$`);
const BUILTIN = new Set(["black", "white", "transparent", "current", "inherit"]);

// 原生色階守門（spec desktop-app「原生色階守門無白名單」場景）：色值只在 theme.css，
// 集中常數檔只保存「狀態→token class」對照，元件只寫 token class——三個掃描根的
// 任何原始碼都不得出現原生色階字面，沒有白名單。漂移多半發生在 app 側，只掃
// packages/ui 擋不住。
/** 掃描根（相對 repo root）。 */
const SCAN_ROOTS = ["packages/ui/src", "apps/desktop/src", "apps/server-web/src"];

/**
 * 原生色階字面：任意 utility 前綴（text-、fill-、border-t-、shadow-…）＋全部色名＋
 * Tailwind 色階數字。前綴不列舉——列舉的前綴清單本身就是白名單的缺口。
 */
const PALETTE_SCALE = new RegExp(`\\b[a-z][a-z-]*-(?:${PALETTE_NAMES})-(?:50|[1-9]00|950)\\b`, "g");

// 註解裡提到某個 class 名稱（例如解釋為什麼不用它）不該被當成用到了它。
const stripComments = (source: string) =>
  source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/[^\n]*/g, "");

/** 掃描範圍內的 .ts／.tsx／.css 原始碼（去註解）。 */
const scannedSources = () =>
  scanSources(SCAN_ROOTS, [".ts", ".tsx", ".css"]).map(([file, source]) => [file, stripComments(source)] as const);

describe("共用 semantic theme 抽取", () => {
  it("元件用到的 bg-* 語意色都在 theme.css 有對應 token", () => {
    const mapped = new Set([...themeCss().matchAll(/--color-([a-z0-9-]+):/g)].map((m) => m[1]));
    const unmapped: string[] = [];
    for (const [file, source] of componentSources()) {
      for (const match of source.matchAll(/\bbg-([a-z][a-z0-9-]*)/g)) {
        const name = match[1];
        if (mapped.has(name) || BUILTIN.has(name) || PALETTE.test(name)) continue;
        unmapped.push(`${file.split("/src/")[1]}: bg-${name}`);
      }
    }
    expect(unmapped).toEqual([]);
  });
});

describe("原生色階守門", () => {
  it("比對式涵蓋 Tailwind 全部色名與任意 utility 前綴，不誤中 token class", () => {
    const hits = (source: string) => [...source.matchAll(PALETTE_SCALE)].map((m) => m[0]);
    const caught = [
      "text-blue-600",
      "bg-cyan-50",
      "border-lime-950",
      "ring-pink-400",
      "fill-sky-500",
      "stroke-amber-600",
      "outline-rose-300",
      "shadow-cyan-500/50",
      "divide-slate-200",
      "via-violet-400",
      "border-t-amber-500",
      "hover:bg-emerald-100",
      "decoration-mauve-700",
      "accent-olive-600",
      "caret-mist-500",
      "placeholder-taupe-400",
    ];
    for (const cls of caught) expect(hits(cls), cls).toEqual([cls.replace(/^hover:/, "").replace(/\/\d+$/, "")]);
    for (const cls of ["text-status-progress", "bg-stage-ready", "border-t-stage-discussion", "text-primary-foreground"]) {
      expect(hits(cls), cls).toEqual([]);
    }
  });

  it("三個掃描根的原始碼都不含原生色階字面（無白名單）", () => {
    const violations: string[] = [];
    for (const [file, source] of scannedSources()) {
      for (const match of source.matchAll(PALETTE_SCALE)) {
        violations.push(`${file}: ${match[0]}`);
      }
    }
    expect(violations).toEqual([]);
  });
});

// spec desktop-app「截斷省略號的統一字形」（design D6）：省略號長什麼樣由該處字型
// 決定——等寬把手畫半形貼基線、中文文字畫全形置中，同一畫面兩種收尾。以限定
// U+2026 的字型層統一，其餘字元照原字型解析（不換任何一段文字的字型）。
describe("截斷省略號的統一字形", () => {
  const ELLIPSIS_FALLBACKS = ["Helvetica Neue", "Arial", "Segoe UI", "DejaVu Sans"];
  /** 既有 body 堆疊——省略號層插在最前，其後必須原封不動。 */
  const EXISTING_STACK =
    '"Noto Sans TC Variable", "Noto Sans TC", "Segoe UI", system-ui, -apple-system, sans-serif';

  it("theme.css 宣告只接管 U+2026 的拉丁字型層", () => {
    const face = themeCss().match(/@font-face\s*\{[^}]*EllipsisLatin[^}]*\}/)?.[0];
    expect(face).toBeTruthy();
    // 限定單一碼位：沒有 unicode-range 這層會接管整段文字，把中文換成拉丁字型。
    expect(face).toContain("unicode-range: U+2026;");
    // 三平台各有一個常駐拉丁字型；全數落空時整層無效、退回既有字型（不破圖）。
    for (const font of ELLIPSIS_FALLBACKS) {
      expect(face).toContain(`local("${font}")`);
    }
  });

  it("body 字型堆疊以省略號層為首，其後既有順序不變", () => {
    const family = themeCss().match(/body\s*\{[^}]*font-family:\s*([^;]+);/)?.[1]?.trim();
    expect(family).toBeTruthy();
    expect(family!.startsWith('"EllipsisLatin"')).toBe(true);
    expect(family).toContain(EXISTING_STACK);
  });
});

describe("共用 semantic theme 抽取（既有）", () => {
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
    // 深色基底純中性（D3）：不再帶色相 260 的藍調。
    expect(css).not.toContain("0.008 260");
  });

  it("theme.css 保留 @theme inline 對 Tailwind utility 的映射", () => {
    const css = themeCss();
    expect(css).toContain("@theme inline");
    for (const mapping of THEME_MAP) {
      expect(css).toContain(mapping);
    }
    // 圓角改用 Tailwind 預設四階（D2）：不再自訂 --radius 與其衍生覆寫。
    expect(css).not.toMatch(/--radius/);
  });

  it("theme.css 讓 Tailwind 掃描 Streamdown 的 class（D10，兩個 app 經 @import 共用）", () => {
    expect(themeCss()).toContain('@source "../../../node_modules/streamdown/dist/*.js";');
  });

  it("theme.css 保留 Noto Sans TC body 字型堆疊", () => {
    const css = themeCss();
    expect(css).toContain('"Noto Sans TC Variable"');
    expect(css).toContain("font-family:");
  });

  it("Desktop index.css 的 markdown 覆寫只留字級行高、code chip、del 與任務清單（D10）", () => {
    const css = desktopCss();
    // 排版交給 Streamdown：typography 外掛與 prose 變數對照整段移除。
    expect(css).not.toContain("@tailwindcss/typography");
    expect(css).not.toContain("--tw-prose-");
    const base = css.match(/\.markdown\s*\{([^}]*)\}/)?.[1] ?? "";
    expect(base).toContain("font-size: 1rem;");
    expect(base).toContain("line-height: 1.7;");
    // Streamdown 自帶表格容器與程式碼框，這幾條不再需要。
    expect(css).not.toMatch(/\.markdown table\s*\{/);
    expect(css).not.toMatch(/\.markdown th\s*\{/);
    expect(css).not.toMatch(/\.markdown pre\s*\{/);
    expect(css).not.toContain("code::before");
    expect(css).toMatch(/\.markdown code:not\(pre code\)\s*\{/);
    expect(css).toMatch(/\.markdown del\s*\{/);
    expect(css).toContain('.markdown li:has(> input[type="checkbox"])');
  });

  it("Desktop index.css 改為 import 共用 theme，不再自行內嵌 token（單一真相源）", () => {
    const css = desktopCss();
    // 引用共用 theme.css（相對 workspace 路徑，與既有 @source 慣例一致）。
    expect(css).toMatch(/@import\s+["'][^"']*packages\/ui\/src\/theme\.css["']/);
    // 不得再就地宣告青綠主色 token——否則兩處各自維護會漂移。
    expect(css).not.toContain("--primary: oklch(0.52 0.085 190);");
  });
});
