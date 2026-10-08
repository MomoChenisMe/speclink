## Context

三個前端面共用一份設計系統：`packages/ui`（React 19＋Tailwind v4＋shadcn 式 Radix 原語＋lucide＋sonner）被 `apps/desktop` 與 `apps/server-web` 以原始碼方式引用（`@speclink/ui` 的 main 指向 `src/index.ts`，兩個 app 的 `index.css` 都 `@import` 同一份 `packages/ui/src/theme.css` 並 `@source` 其原始碼）。色彩現況分三層：

| 層 | 位置 | 現況 |
| --- | --- | --- |
| 基底 token | `packages/ui/src/theme.css` | 16 個 oklch token，淺色一套、深色一套（`prefers-color-scheme`），深色底帶色相 260 的藍調；單一圓角 `--radius: 0.625rem` 衍生 lg／md／sm |
| 語意色 | `tone.ts`、`stage.ts`、`reviewStyle.tsx`、`verifyStyle.tsx`、`improveStyle.tsx`、`DeltaBadges.tsx` | 以原生色階成對寫（`text-violet-600 dark:text-violet-400`）；`stage.ts` 用 teal 深淺三階表達生命週期 |
| 元件 class | 17 個原語與約 33 個領域元件 | 卡片 `shadow-sm`、分頁底線式、側欄選中整塊填 `bg-primary` 等 |

守門在 `packages/ui/src/__tests__/theme.test.ts`：逐值寫死 token、掃描三個 src 根禁止白名單以外出現原生語意色階、檢查 `bg-*` 都有 token。重複元件：`CopyButton` 4 份、`EmptyState` 2 份、`SectionHeader` 2 份、確認框以 `AlertDialogContent` 四件組手拼 28 處。markdown 由 `react-markdown`＋`remark-gfm`＋`remark-breaks`＋自寫 GitHub Alert 轉換渲染，排版靠 `@tailwindcss/typography` 的 prose 與 `apps/desktop/src/index.css` 的 11 條 `.markdown` 覆寫。Logo 是 PNG，渲染於 App.tsx 內嵌 img 與 server-web `Wordmark.tsx`，資產重複三處；使用者已重畫成「接合 S」標記，SVG 原稿在 `docs/assets/brand/svg/`（文字已轉路徑），原生 App 圖示與系統匣單色圖示也已重生。

這是討論 desktop-ui-redesign 的 cut 1（地基），cut 2–5 都站在它的 token、原語與守門上。純前端，不動 Rust 端、CLI 與引擎。

## Goals / Non-Goals

**Goals:**

- 色值只在 theme.css、對照只在 TS 表、元件只寫 token class；深色不必成對寫；換色只改一行。
- 生命週期四個色相（討論 fuchsia、提案中 teal、進行中 sky、已就緒 emerald）這一刀落到所有消費者：看板欄頂條、欄計數徽章、卡片波次章、系統匣生命週期分區與討論分區的圖示與計數。
- 17 個原語改成白底中性灰、細框、圓角四階、浮層才有陰影的外觀；兩個 app 自動吃到。
- 重複元件收口到 packages/ui，並以守門測試擋住再度散落。
- markdown 渲染換 Streamdown，既有四個行為不變。

**Non-Goals:**

- 版面（側欄、頂列、標題列、看板欄形狀、抽屜骨架、設定頁、系統匣面板結構）——cut 2–5。
- 尚無消費者的原語（dropdown-menu、switch、search-input、status-pill、kbd、page-header、settings-card／row）——隨第一個消費者所在的刀建立。
- Sheet 原語——詳情頁取代抽屜後於 cut 3b 退場，本刀不動它。
- Logo 圖形本身——已在本刀外完成；品牌在畫面上的位置（圖示列頂的裸標記、空狀態與「關於」的鎖版）屬 cut 2 的外殼。
- Rust 端、CLI、`--json`、引擎。

## Decisions

### D1 色值只在 theme.css，TS 表只剩對照

`theme.css` 的 `:root` 與深色區塊各新增下列 token，`@theme inline` 逐一映射為 `--color-<name>`，讓 `bg-status-progress`、`text-stamp`、`bg-stage-ready/10` 這類 utility 直接可用（透明度修飾與現有 `bg-primary/8` 同機制）：

| token | 淺色 | 深色 | 語意 |
| --- | --- | --- | --- |
| `--sidebar` | neutral-50 `oklch(0.985 0 0)` | `oklch(0.185 0 0)` | 專案欄底（cut 2 消費） |
| `--status-progress` | sky-600 | sky-400 | 進行中 |
| `--status-success` | emerald-600 | emerald-400 | 成功 |
| `--status-warning` | amber-600 | amber-500 | 警示 |
| `--stamp` | violet-600 | violet-400 | 品質站蓋章專屬 |
| `--improve` | indigo-600 | indigo-400 | 改進標示（既有色相，只是搬成 token） |
| `--stage-discussion` | fuchsia-600 | fuchsia-400 | 看板討論欄、系統匣討論分區 |
| `--stage-proposed` | `var(--primary)` | `var(--primary)` | 提案中 |
| `--stage-in-progress` | `var(--status-progress)` | 同 | 進行中 |
| `--stage-ready` | `var(--status-success)` | 同 | 已就緒 |

錯誤／危險沿用既有 `--destructive`，不另設 `status-danger`（一個意思一個 token）。原生色階的 oklch 值逐字取自 `node_modules/tailwindcss/theme.css` 的對應色階，不手調。TS 表改為：`SEMANTIC_TONE`＝`text-status-progress`／`text-status-success`／`text-status-warning`／`text-destructive`，`SEMANTIC_SURFACE`＝`border-<token>/40 bg-<token>/10`；`REVIEW_TONE.reviewed`／`VERIFY_TONE.verified`＝`text-stamp`，其餘三態取 status token；`IMPROVE_TONE`＝`text-improve`、`IMPROVE_CHIP_TONE`＝`bg-improve/10`；`DELTA_COLORS`＝added→success、modified→warning、removed→destructive、renamed→progress。替代方案「語意色全搬進 TS 表」與「維持三層只改值」在討論中否決。

`--primary` 同時校準到品牌固定色：淺色 `oklch(0.52 0.085 190)`（青綠 `#167873` 換算為 oklch(0.519 0.084 189.5)）、深色 `oklch(0.72 0.1 190)`（淺青綠 `#4bb9b3` 換算為 oklch(0.721 0.1 190.2)）；`--primary-foreground` 不動。品牌固定色的十六進位值只存在於 `docs/assets/brand` 的 README 與 SVG 原稿，不進 theme.css；兩者分開管理，用這一次校準讓肉眼一致。

### D2 圓角改用 Tailwind 預設四階

移除 `--radius` 與 `@theme inline` 的三行圓角覆寫；`rounded-md`＝6px（按鈕、tooltip）、`rounded-lg`＝8px（清單列、分頁、選單項、輸入框）、`rounded-xl`＝12px（選單面板、對話框、popover、toast）、`rounded-2xl`＝16px（卡片）、`rounded-full`（徽章、搜尋框、圓形圖示鈕）。既有 `rounded-md`／`rounded-lg` 用法的像素值跟著變，這是刻意的。替代方案「自訂四個 radius token」多一層對照卻與 Tailwind 預設值相同，不值。

### D3 深色基底去藍調

深色 `--background`＝`oklch(0.145 0 0)`、`--card`＝`oklch(0.205 0 0)`、`--secondary`／`--muted`＝`oklch(0.269 0 0)`（neutral-800），與淺色對稱；`--border`／`--input` 維持白色 12%／15%，`--primary`、`--accent`、`--ring`、`--destructive` 深色值不變。

### D4 生命週期與討論的色相由 stage.ts 單一來源供給

`STAGE_BADGE`／`STAGE_BAR`／`STAGE_ICON` 改取 `stage-*` token（例：`ready`＝`bg-stage-ready text-primary-foreground`／`bg-stage-ready`／`text-stage-ready`；`proposed`、`in-progress` 以 `/10` 淡底＋實色字為徽章、實色為條與圖示）；新增 `DISCUSSION_TONE`＝`{ icon: "text-stage-discussion", badge: "bg-stage-discussion/10 text-stage-discussion" }` 供看板討論欄標頭（`DiscussionColumn.tsx`）與系統匣討論分區（`TrayPanel.tsx` 的 `iconCls`／`badgeCls`）取用；「已轉出」分區與收合列維持中性。欄計數徽章從此隨所在分區色，不再一律中性。

### D5 原語扁平化的逐一規格

| 原語 | 新外觀 |
| --- | --- |
| button | 圓角 `rounded-md`；變體 default（`bg-primary` 白字）、secondary（`bg-primary/10 text-primary`）、outline（白底細框）、ghost、subtle（圖示鈕，hover 才 `bg-foreground/5`）、link、destructive（實心紅）、danger（`bg-destructive/10 text-destructive`）；尺寸 default h-9、sm h-7、toolbar h-8、icon 32／28／24px；焦點 `focus-visible:ring-2 ring-ring` |
| card | `rounded-2xl border bg-card`，無陰影；`size="nested"` 變體為 `rounded-xl`（容器內的小卡：看板卡片、對話框內的清單卡）；`interactive` 變體提供 hover（框變深＋底微灰）；呼叫端不得以 className 覆蓋圓角與陰影 |
| badge | `rounded-full px-1.5 py-px text-[11px]`；變體 secondary（`bg-muted`）、outline、default（`bg-primary`） |
| tabs | 卡片標頭式底線：List `flex w-full items-center px-2 bg-card border border-border rounded-t-2xl`（作為內容卡的頂部；內容卡以 `rounded-t-none border-t-0` 接在下方），Trigger `h-10 px-3.5 text-sm font-medium text-muted-foreground border-b-2 border-transparent -mb-px`，作用中 `text-primary border-primary`、無底色無陰影；計數徽章緊接在文字後；List 右端保留一個 `actions` slot 放頁級動作（如「開啟 ▾」） |
| input／textarea | `h-8 rounded-lg border-input bg-background`，無 `shadow-sm`；聚焦 `border-ring ring-2 ring-ring/30` |
| select | Trigger 與 input 同高同框；Content `min-w-56 rounded-xl p-1.5 shadow-lg shadow-black/5`；Item `rounded-lg px-2.5 py-1.5 text-[13px]`，hover `bg-muted`、選取為勾號不填主色；分隔線 `mx-2`；外殼刀的 dropdown-menu 沿用同一組 class |
| popover | `rounded-xl p-3 shadow-md` |
| tooltip | `bg-card text-foreground border shadow-md text-xs rounded-md`；延遲維持 300ms（spec「主題化提示統一延遲」） |
| alert-dialog | 遮罩 `bg-foreground/30`；Content `w-[28rem] rounded-xl p-5 shadow-lg`；Title 15px medium、Description 13px 灰字；Cancel 為 ghost |
| sonner | toast `rounded-xl shadow-lg`；位置與時長不變 |
| checkbox | `rounded-[4px]`，其餘不變（spec「表單控制項與按鈕以主題化元件呈現」） |
| skeleton、table、label、portal-container、sheet | 不變（sheet 留給 cut 3） |

既有測試裡針對舊 class 的斷言（`ui.test.tsx`、`select.test.tsx`、`skeleton*.test.tsx`、`kanban.test.tsx`、各抽屜測試、desktop 的 `App.test.tsx`、`trayPanel.test.tsx` 等）隨之更新；行為斷言不動。

### D6 只建有消費者的共用元件

新增六個：`ConfirmDialog`（原語層 `components/ui/confirm-dialog.tsx`）、`CopyButton`、`EmptyState`、`SectionHeader`、`BrandMark`、`Wordmark`（領域層）。其餘原語等第一個消費者。理由：沒有消費者就無法驗證形狀，且與專案「不為一次性使用建抽象」的慣例一致。

### D7 ConfirmDialog 的形狀

```ts
interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: ReactNode;
  confirmLabel: string;
  cancelLabel?: string;        // 預設 i18n common.cancel
  destructive?: boolean;       // 確認鈕用 destructive 變體
  busy?: boolean;              // 確認鈕 disabled＋取消鈕 disabled
  onConfirm: () => void | Promise<void>;
  children?: ReactNode;        // 標題與說明之後、按鈕之前的額外內容（如工具多選）
}
```

兩鍵式確認框全部改用它：App.tsx 的初始化、啟用、封存、封存討論、退回提案中、刪除等；`ProjectSettingsView.tsx` 的三處；server-web `UsersPage`／`SystemPage`／`CredentialsPage`／`AccountPage`。三選項以上的對話框（`ReviewArchiveDialog`、`RevertBlockedDialog`、`WorkspaceChooser`、`MigrationDialog`、`RemoteConflictDialog`、`ReleaseNotesDialog`）維持各自的 `AlertDialog` 組裝。

### D8 CopyButton／EmptyState／SectionHeader／Brand 的形狀

- `CopyButton({ value, label, size?: "sm" | "icon", onCopy?, className? })`：預設走 `navigator.clipboard.writeText`，`onCopy` 可覆寫（系統匣面板改接 Tauri clipboard plugin）；複製成功後圖示換勾號 1.5 秒（spec 豁免項「複製成功勾號為主色」維持）。
- `EmptyState({ icon, title, description?, action? })`：置中直欄，圖示 40px、標題 2xl 一般字重、描述 13px 灰字、動作為 default 按鈕。
- `SectionHeader({ icon?, label, count?, countClassName?, action?, className? })`：`text-xs font-semibold text-muted-foreground` 一列，計數為 badge。
- `Brand.tsx` 匯出 `BrandMark({ size?: number, className? })` 與 `Wordmark({ className? })`。資產放 `packages/ui/src/assets/` 三個 SVG：`logo-mark.svg`（自 `docs/assets/brand/svg/speclink-logo-mark.svg` 複製，唯一改動是 `fill="#167873"` 改成 `fill="currentColor"`）、`logo-horizontal.svg` 與 `logo-horizontal-dark.svg`（逐字複製）；`packages/ui/src/vite-env.d.ts` 加 `/// <reference types="vite/client" />` 讓 `.svg` 與 `?raw` 匯入有型別。換 Logo 只換這三個檔。
  - `BrandMark`：以 `import mark from "../assets/logo-mark.svg?raw"` 取得 SVG 字串（建置期常數，不是使用者內容），渲染 `<span role="img" aria-label="Speclink" className={cn("inline-block text-primary [&>svg]:h-full [&>svg]:w-auto", className)} style={{ height: size }} dangerouslySetInnerHTML={{ __html: mark }} />`；`size` 預設 26。顏色跟 `text-primary` 走：淺色主色 teal、深色淺青綠，與 D1 校準後的 `--primary` 同值。不用 app 圖示的「青綠底板＋白 S」——它與作用中專案方塊同形。本刀沒有畫面消費者（圖示列頂在 cut 2），仍於本刀建立：它與三個資產、`Wordmark` 同屬一個品牌模組，討論結論把資產收口整個指派給本刀；形狀由 `brand.test.tsx` 驗證。這是 D6 的唯一例外。
  - `Wordmark`：`<picture>` 帶 `<source media="(prefers-color-scheme: dark)" srcSet={horizontalDark} />` 與 `<img src={horizontal} alt="Speclink" className={cn("w-auto", className)} />`，高度由呼叫端給：本刀的桌面頂欄、server-web `ConsoleLayout` 與 `FocusLayout` 傳 `h-5`（與現狀同高）；cut 2 在零專案空狀態與「關於 Speclink」改 `h-10`（40px），其他地方不再出現字標。
  - 消費端：`apps/desktop/src/App.tsx` 頂欄的 `logo-mark.png` img 與字標 `<picture>` 整組換成 `<Wordmark className="h-5" />`；server-web 本地 `Wordmark.tsx` 刪除，`ConsoleLayout.tsx`、`FocusLayout.tsx` 改自 `@speclink/ui` 引用；`apps/server-web/src/assets/` 三張 PNG 與 `apps/desktop/public/` 兩張字標 PNG 刪除，`apps/desktop/public/logo-mark.png` 留給 `index.html` favicon。server-web 的 CSP 已允許 `img-src 'self' data:`，SVG 經 Vite 內嵌或輸出雜湊檔皆可。系統匣面板不放品牌標記（選單列圖示已是識別）。
  - 文件：`docs/assets/brand/README.md` 與 `README.en.md` 的「App 使用位置」改指向 `packages/ui/src/assets/` 與 `Brand.tsx`；`.gitignore` 加 `docs/assets/brand/*.zip`（品牌套件壓縮檔不進 repo）。

### D9 守門

- `theme.test.ts`：token 快照換成 D1／D3 的全部值；原生色階禁令改為三個掃描根全面適用、`TONE_SOURCES` 白名單刪除；色階正規式加入中性色階（slate／gray／zinc／neutral／stone，掃描面現為零處使用）；`bg-*` 對應 token 檢查涵蓋新 token。
- 新增 `packages/ui/src/__tests__/uiSingleSource.test.ts`：讀 `packages/ui/src/index.ts` 收集匯出的識別字（`export { A, B } from`、`export function A`、`export const A`），掃描 `apps/desktop/src` 與 `apps/server-web/src`（排除 `__tests__`）的 `.ts`／`.tsx`，任一檔案頂層出現 `function <Name>(` 或 `const <Name> =` 且 `<Name>` 在匯出集合內即失敗，訊息列出檔案與名稱。
- `openspec/config.yaml` 的 `rules.design` 新增一條：「前端共用元件優先用 @speclink/ui 既有原語與領域元件；呼叫端不得以 className 覆蓋變體的顏色、圓角與陰影；apps/* 不得定義與 @speclink/ui 匯出同名的元件」。

### D10 Streamdown 的接法

- `packages/ui/package.json` 加 `streamdown`（^2.6.0）與 `@streamdown/code`；移除 `react-markdown`；`remark-gfm` 由 Streamdown 自帶（移除直接相依）、`remark-breaks` 保留。`apps/desktop/package.json` 移除 `@tailwindcss/typography`。
- `Markdown.tsx`：`<div className="markdown max-w-[96ch]"><Streamdown mode="static" remarkPlugins={[...Object.values(defaultRemarkPlugins), alerts, remarkBreaks]} rehypePlugins={rehypeWithAlertSchema} plugins={{ code }} controls={{ code: { copy: true, download: false }, table: false, mermaid: false }} translations={{ copyCode: t("markdown.copyCode"), copied: t("markdown.copied") }} icons={{ CopyIcon: Copy, CheckIcon: Check }} linkSafety={{ enabled: false }} shikiTheme={["github-light", "github-dark"]}>{content}</Streamdown></div>`。`rehypeWithAlertSchema` 以 `defaultRehypePlugins` 為底，把 sanitize schema 擴充成允許 `div` 帶 `markdown-alert*`、語意 surface 與 title 的 class（Streamdown 傳自訂 rehypePlugins 後不再自動擴充 schema）；`remarkGithubAlerts` 不變。raw HTML 由 Streamdown 的 rehype-raw＋sanitize 承擔：HTML 註解不進畫面、code fence 內原文照常。
- `packages/ui/src/theme.css` 加 `@source "../../../node_modules/streamdown/dist/*.js";`（兩個 app 經 `@import` 共用，不各自加）。
- `apps/desktop/src/index.css`：移除 `@plugin "@tailwindcss/typography"` 與整段 `--tw-prose-*` 對照、`code::before/after`、`pre` 邊框、`table` display／overflow、`th` 底色（Streamdown 自帶表格容器與程式碼框）；保留並改寫 `.markdown` 為 `font-size: 1rem; line-height: 1.7`、行內 code chip（teal 底）、`del` 灰字、GFM 任務清單四條規則。
- 深色：Streamdown 以 `dark:` 變體切 Shiki 深色，Tailwind v4 的 `dark:` 預設跟 `prefers-color-scheme`，與 theme.css 同步，不加 `@custom-variant`。
- i18n：`packages/ui/src/i18n.tsx` 兩語系各加 `markdown.copyCode`（複製程式碼／Copy code）與 `markdown.copied`（已複製／Copied）。

替代方案「react-markdown 加 @shikijs/rehype」在討論中否決。

## Implementation Contract

**Behavior**

- 淺色與深色主題下，所有狀態色、生命週期色、蓋章色、改進色由 theme.css 的 token 決定；改 theme.css 一個值，看板、抽屜、系統匣面板、server-web 同步變色，不需改任何 TS 或 TSX。
- 看板三個生命週期欄的頂條、標頭圖示、計數徽章與卡片波次章分別呈現 teal／sky／emerald；討論欄標頭圖示與計數呈現 fuchsia；系統匣面板的生命週期分區與討論分區同色；「已轉出」維持中性。
- 原語外觀依 D5；兩個 app 不加任何覆寫即得到相同外觀。
- 任一兩鍵式確認框的標題、說明、確認、取消文案與 disabled 行為與改版前一致。
- 複製鈕、空狀態、區段標題、品牌標記在兩個 app 與系統匣面板只有一份實作。
- 品牌標記在淺色呈主色 teal、深色呈淺青綠，與 `--primary` 同值；字標為橫式鎖版，系統偏好深色時自動取深色版；系統匣面板不出現品牌標記。
- markdown：清單符號與編號、段落間距、單換行＝換行、16px 基準字級、行寬 96ch 置中、寬表格橫捲、HTML 註解不出現、code fence 內 HTML 原文照常、GitHub Alert 四型提示框與 i18n 標籤，全部與改版前一致；新增程式碼區塊語法上色（淺色 github-light、深色 github-dark）與複製鈕。

**Interface / data shape**

- 新 token 名與 utility 名：`sidebar`、`status-progress`、`status-success`、`status-warning`、`stamp`、`improve`、`stage-discussion`、`stage-proposed`、`stage-in-progress`、`stage-ready`。
- `@speclink/ui` 新匯出：`ConfirmDialog`、`CopyButton`、`EmptyState`、`SectionHeader`、`BrandMark({ size?: number = 26, className? })`、`Wordmark({ className? })`、`DISCUSSION_TONE`；移除匯出：無。
- `Button` 變體集合：default、secondary、outline、ghost、subtle、link、destructive、danger；尺寸：default、sm、toolbar、icon、icon-sm、icon-xs。
- i18n 新鍵：`markdown.copyCode`、`markdown.copied`（兩語系鍵集合維持相等）。
- `openspec/config.yaml`：`rules.design` 多一條字串。

**Failure modes**

- 守門測試失敗即指名檔案與 class／名稱；不在執行期做任何檢查。
- Streamdown 的 Shiki 語言未載入時，程式碼區塊以無上色的等寬文字呈現，不拋錯。
- 剪貼簿寫入失敗：`CopyButton` 不顯示勾號、不彈錯誤（與系統匣既有行為一致）。

**Acceptance criteria**

- `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 全綠；`theme.test.ts` 新快照、`uiSingleSource.test.ts`、`markdownAlerts.test.tsx`、`stage.test.ts` 更新後通過。
- `grep -rE '(text|bg|border|ring|from|to)-(sky|amber|emerald|rose|red|teal|green|violet|purple|orange|yellow|fuchsia|indigo|slate|gray|zinc|neutral|stone)-[0-9]{2,3}'` 在三個 src 根（排除 `__tests__`）零命中。
- `apps/desktop`、`apps/server-web` 的 `vite build` 通過。
- `grep -rn 'logo-mark\|wordmark' apps/ --include='*.ts' --include='*.tsx' --include='*.html'` 只命中 `apps/desktop/index.html` 的 favicon；`apps/server-web/src/assets/` 不含任何品牌 PNG。
- 手動：desktop 看板在淺色與深色各看一次，四欄色相正確、卡片無陰影、分頁為卡片標頭式底線；開一份含程式碼區塊的規格，上色與複製鈕可用；server-web 總覽頁外觀同步。

**Scope boundaries**

- In：`packages/ui/src`（theme 含 `--primary` 校準、常數表、17 個原語、六個新元件、三個品牌 SVG、Markdown、i18n、index、守門測試）、兩個 app 的 `index.css`、重複元件的消費端改寫、`package.json` 相依、`openspec/config.yaml` 一條規則、品牌 README 的「App 使用位置」、`.gitignore` 一行、受影響測試。
- Out：任何版面與尺寸、Sheet 遮罩、系統匣面板結構、Rust 端、CLI、Logo 圖形（已在本刀外完成）、品牌在畫面上的位置（圖示列頂、空狀態、關於＝cut 2）、原生 App 圖示與系統匣圖示（已重生）。

## Risks / Trade-offs

- [圓角改預設值讓 61 處 `rounded-md` 從 8px 變 6px、31 處 `rounded-lg` 從 10px 變 8px] → 刻意；視覺由 [M] 任務在淺深兩色各看一次。
- [Streamdown 的 sanitize 可能剝掉 GitHub Alert 的 class] → D10 擴充 schema；`markdownAlerts.test.tsx` 逐案驗證輸出含 `markdown-alert-<type>` class。
- [Streamdown 自帶元素樣式與 `.markdown` 規則重疊] → 覆寫只留四類（字級行高、code chip、del、任務清單），其餘刪除；markdown 結構測試守住清單符號、編號、16px。
- [theme.test 無白名單後，任何新原生色階立即紅燈] → 這是目的；訊息列出檔案與 class，改成 token class 即可。
- [`BrandMark` 以 `dangerouslySetInnerHTML` 內嵌 SVG] → 字串來自 `?raw` 匯入的建置期常數，不經任何執行期輸入；`brand.test.tsx` 斷言渲染結果含 `fill="currentColor"` 的 `<svg>`。
- [`--primary` 校準改變所有主色消費者的色值] → 與現值差距在 chroma 0.015、hue 2° 內，肉眼接近；theme 快照更新即可。
- [uiSingleSource 以正規式掃頂層定義，可能漏掉巢狀定義] → 守門只針對「散落的同名元件」這一類，巢狀區域函式不是重複實作來源；接受。
- [回歸對照] → 不動 CLI、golden、`--json`；前端三個 workspace 的測試是基準。
- [跨平台] → 純 CSS／TS；Shiki 在 WebView2（Windows）與 WebKitGTK（Linux）無原生相依。
- [server-web 的 `Field`／`DataList`／`ListToolbar`／`DetailSheet` 仍留在 server-web] → 它們只有一個消費者且不與 ui 匯出撞名，不是本刀的散落問題。

## Migration Plan

無資料遷移。`npm install` 更新相依；部署隨桌面 app 與 server 一般發版；回滾即還原本刀的 commit。

## Open Questions

無。
