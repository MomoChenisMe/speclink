// @vitest-environment node
// 純檔案系統斷言：以 node 環境執行，import.meta.url 才是 file:// URL
// （jsdom 環境會把它換成 http location，導致 fileURLToPath 失敗）。
import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
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

/** Tailwind 內建色階（bg-amber-500 之類）不經 theme，排除。 */
const PALETTE =
  /^(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-\d{2,3}$/;
const BUILTIN = new Set(["black", "white", "transparent", "current", "inherit"]);

// 狀態語意色守門（spec desktop-app「原生色階守門」場景）：原生語意色階字面只能
// 出現在集中常數檔，其餘元件一律經常數表或設計 token 取色。掃描範圍涵蓋共用套件
// 與兩個 app 的元件原始碼——漂移多半發生在 app 側，只掃 packages/ui 擋不住。
const REPO_ROOT = fileURLToPath(new URL("../../../../", import.meta.url));

/** 掃描根（相對 repo root）。 */
const SCAN_ROOTS = ["packages/ui/src", "apps/desktop/src", "apps/server-web/src"];

/**
 * 白名單：集中常數檔——語意色表、審查樣式表、delta 徽章表、生命週期表。
 * 三紅分工（destructive／rose／red）就是靠這三張表各自持有色階字面來維持。
 * stage.ts 現況只用 primary，列入為防未來階梯改用原生色階時無處可放。
 */
const TONE_SOURCES = new Set([
  "packages/ui/src/tone.ts",
  "packages/ui/src/components/reviewStyle.tsx",
  "packages/ui/src/components/DeltaBadges.tsx",
  "packages/ui/src/stage.ts",
]);

/**
 * 語意色階字面。錨定 Tailwind class 型式（utility 前綴＋色名＋階）以免撞到
 * 一般字串（例如 "to-do-red-1" 這種非 class 文字不會有 utility 前綴）。
 * 中性色階（slate/gray/zinc/neutral/stone）不在此列——中性用 token 表達，
 * 但既有中性字面不是本規則的糾察對象。
 */
const SEMANTIC_SCALE =
  /\b(?:text|bg|border|ring|from|to)-(?:sky|amber|emerald|rose|red|teal|green|violet|purple|orange|yellow|fuchsia)-\d{2,3}\b/g;

// 註解裡提到某個 class 名稱（例如解釋為什麼不用它）不該被當成用到了它。
const stripComments = (source: string) =>
  source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/[^\n]*/g, "");

/** 掃描範圍內的 .ts/.tsx 原始碼（排除 __tests__ 與 dist）。 */
const scannedSources = () => {
  const files: string[] = [];
  const walk = (path: string) => {
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      if (entry.name === "__tests__" || entry.name === "dist" || entry.name === "node_modules")
        continue;
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else if (entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) files.push(child);
    }
  };
  for (const root of SCAN_ROOTS) walk(`${REPO_ROOT}${root}`);
  return files.map(
    (f) => [f.slice(REPO_ROOT.length), stripComments(readFileSync(f, "utf8"))] as const,
  );
};

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

describe("狀態語意色守門", () => {
  it("原生語意色階字面只出現在集中常數檔", () => {
    const violations: string[] = [];
    for (const [file, source] of scannedSources()) {
      if (TONE_SOURCES.has(file)) continue;
      for (const match of source.matchAll(SEMANTIC_SCALE)) {
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
