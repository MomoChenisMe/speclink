## Context

桌面 app（apps/desktop，Tauri WKWebView）與網頁後台（apps/server-web）共用 packages/ui 的元件與 packages/ui/src/theme.css；兩個 app 的根樣式（各自的 index.css）先 `@import "tailwindcss"` 再匯入 theme.css。Tailwind v4（4.3.2）的 preflight 不再替按鈕設手指，所以目前所有 `Button` 與原生 `<button>` 都是箭頭；只有元件自己寫了 `cursor-pointer` 的地方是手指（15 處元件、開關原語、網頁後台 3 處）。停用態則有 `cursor-not-allowed`（元件庫原語 6 個、開啟專案畫面 3 處）與箭頭兩種寫法。

參考對象 Codex app（`com.openai.codex`，Electron）的打包樣式：可點元素一律 `cursor: var(--cursor-interaction, pointer)`，桌面版以 `[data-codex-window-type=electron] body { --cursor-interaction: default }` 改成箭頭。

前提：可點元素的 hover 底色已在 2026-10-12 驗收時統一（前景色 5%；任務列唯讀不給 hover），隨 desktop-discussion-doc-views 的驗收修正提交。本設計只處理游標與停用態，不再調 hover 深淺。

## Goals / Non-Goals

**Goals:**

- 桌面 app（主視窗與系統匣面板）所有可點元素為箭頭；網頁後台所有可點元素為手指；Markdown 正文超連結兩邊都是手指。
- 游標只在一處決定：共用樣式的一個變數；宿主只改變數值。
- 新寫的按鈕不用記得加游標；寫死手指或禁止游標的元件被測試擋下。
- 停用與唯讀一律箭頭、無 hover 底色。

**Non-Goals:**

- 不改 hover 底色深淺、不改拖曳把手（抓取游標）、文字輸入（文字游標）與調整尺寸的游標。
- 不做使用者可切換的游標偏好。
- 不改系統匣原生選單（非 webview）。
- 不替網頁後台補 hover 底色的可見度規則（網頁以手指表示可點）。

## Decisions

### D1 游標變數集中設定

共用樣式 theme.css 的 `:root` token 區新增 `--cursor-interaction: pointer;`。桌面 app 的根樣式 apps/desktop/src/index.css 在匯入 theme.css 之後宣告 `:root { --cursor-interaction: default; }`（同選擇器、後宣告者勝）。系統匣面板的入口也匯入同一份 index.css，自動得到箭頭。網頁後台的 index.css 不宣告此變數，沿用預設手指。

替代方案：桌面版以 `* { cursor: default !important }` 一刀切——會蓋掉文字輸入的文字游標與拖曳把手的抓取游標；不採用。以 `data-host` 屬性切換——要在兩個 app 的根元素各掛屬性，比直接在各自根樣式改變數多一層；不採用。

### D2 全域規則套用原生按鈕與 role="button"

theme.css 新增：

```css
@layer base {
  button:not(:disabled):not([aria-disabled="true"]),
  [role="button"]:not([aria-disabled="true"]) {
    cursor: var(--cursor-interaction);
  }
}
```

放在 base layer：元件上的 utility（`cursor-default`、`cursor-grab`）屬 utilities layer，仍可覆寫；停用的元素不命中，維持瀏覽器預設箭頭。`Button`、`NavItem`、分頁、文件樹節點、勾選框（Radix 以 button 實作）、名稱籤（Markdown 外掛產生的 `span[role="button"]`）都因此得到變數游標，不需逐一加 class。

第三方元件寫死的游標 class 屬 utilities layer，會蓋過這條規則：Markdown 內 Streamdown 的程式碼複製鈕寫死手指與停用禁止游標。Markdown 容器以任意變體（`[&_button]:cursor-(--cursor-interaction)`、`[&_button:disabled]:cursor-default`）蓋回共用設定——同屬 utilities layer、選擇器權重較高；程式碼區塊的無外框圖示鈕另補圓角與 hover 淡底（`[&_[data-streamdown=code-block-actions]_button]:rounded-md`、`[&_[data-streamdown=code-block-actions]_button:enabled]:hover:bg-foreground/5`）；圖片下載鈕自帶底色，只蓋回游標、保留它的 hover 底色。D6 守門掃不到套件內的 class，由 Markdown 測試釘住容器變體（品質站第 1 輪補入）。

### D3 非按鈕的可點元素取用變數 class

現有每一處 `cursor-pointer` 一律機械式換成 Tailwind v4 的變數寫法 `cursor-(--cursor-interaction)`（產生 `cursor: var(--cursor-interaction)`）。不逐一判斷該元素是否已被 D2 涵蓋：對 `<button>` 與 `role="button"` 元素重複但無害，對 `<label>`（任務列）、無 role 的 `<div>`（看板卡片 Card、網頁後台的可選取卡片與表格列）則是必要。名稱籤容器的任意變體 `[&_.name-chip]:cursor-pointer` 同樣換成 `[&_.name-chip]:cursor-(--cursor-interaction)`。

涵蓋的位置（以元素描述）：設計決策卡頭、手冊頁出處籤、背景內文名稱籤容器、任務列 label（可寫時）、共用清單列、看板討論卡與收合討論列、討論過程輪次卡頭、看板變更卡、規格頁溯源關聯兩顆籤、工單輪卡頭、需求收合列兩處、遠端復原的技術細節鈕、開關原語、網頁後台清單元件的可選取卡片與表格列、網頁後台登錄頁的可選取卡片。

### D4 停用與唯讀為箭頭且無 hover

- 元件庫原語的 `disabled:cursor-not-allowed`（勾選框、輸入框、多行輸入、下拉觸發鈕、開關）與標籤的 `peer-disabled:cursor-not-allowed` 改成 `cursor-default` 對應變體。
- 開啟專案畫面：選擇卡（ChoiceCard）與單選列（RadioRow）停用時改 `cursor-default`，hover 底色與框線只在未停用時套用；最近開啟列的外層在列停用（開啟進行中 `busy`）時不套 hover 底色，其內按鈕的 `disabled:cursor-not-allowed` 改 `disabled:cursor-default`；remote marker 與本機 openspec 並存時的衝突對話框（RemoteConflictDialog），三張選擇卡在處理中（`busy`）一起停用，hover 底色與框線只在未處理中套用（apply 期掃描停用元素時補入；游標本來就不命中 D2 的全域規則，維持箭頭）。選擇卡、單選列與衝突對話框選擇卡都是原生 `<button>`，以 Tailwind 的 `enabled:hover:` 變體表達「只在可用時 hover」；最近開啟列的外層是 `div`，維持依 `busy` 的條件 class。
- 遠端復原的技術細節鈕（無外框）補上前景色 5% 的 hover 淡底：proposal 前提把它列為已統一，實際只有文字變色（品質站第 1 輪補入）。
- `Button` 原語已有 `disabled:pointer-events-none`（無 hover、游標取下層的箭頭），不需改。共用清單列停用分支已是 `cursor-default` 且無 hover，不需改。

### D5 選單項與下拉選項取用變數

下拉選單項目的共用 class（menuItemClass）與下拉選項（SelectItem）的 `cursor-default` 改成 `cursor-(--cursor-interaction)`：網頁是手指、桌面維持箭頭。下拉選單的上下捲動鈕（hover 即捲動、非點擊）維持 `cursor-default`。

### D6 游標守門測試

新增 packages/ui/src/__tests__/cursorRule.test.ts（`// @vitest-environment node`，沿用 sourceScan 的 `scanSources`）：

1. 掃描 `packages/ui/src/components`、`apps/desktop/src`、`apps/server-web/src` 的 `.ts`／`.tsx`（scanSources 已略過 `__tests__`、`dist`、`node_modules`），任何檔案內出現整字 `cursor-pointer` 或 `cursor-not-allowed`（含 `disabled:`、`peer-disabled:` 等變體前綴）即失敗，訊息列出全部違規檔的 repo 相對路徑。
2. theme.css 須含 `--cursor-interaction: pointer` 與 D2 的全域規則（`cursor: var(--cursor-interaction)` 且選擇器含 `button:not(:disabled)` 與 `[role="button"]`）。
3. apps/desktop/src/index.css 須含 `--cursor-interaction: default`；apps/server-web/src/index.css 不得含 `--cursor-interaction`。

各條失敗訊息分別指出缺少的是哪一項。

### D7 tray-status-menu 游標可觀察條件

只改規格文字：「游標樣式隨元素改變」改為「hover 樣式且游標為箭頭」。面板與主視窗共用 index.css，D1 已讓兩者為箭頭，無程式改動。

## Implementation Contract

**Behavior**

- 桌面 app：滑過任一可點元素（含按鈕、清單列、卡片、收合卡頭、籤、分頁、文件樹、導覽列、選單項）游標為箭頭；hover 底色照舊出現。Markdown 正文超連結為手指。停用與唯讀元素為箭頭、無 hover 底色。
- 網頁後台：同一批可點元素為手指；停用與唯讀為箭頭、無 hover 底色。
- 文字輸入、拖曳把手、調整尺寸的游標不變。

**Interface / data shape**

- CSS 變數：`--cursor-interaction`，值 `pointer`（theme.css `:root` 預設）或 `default`（桌面 index.css 覆寫）。
- 元件 class：`cursor-(--cursor-interaction)`；停用為 `cursor-default`（含 `disabled:`／`peer-disabled:` 變體）。
- 全域規則選擇器：`button:not(:disabled):not([aria-disabled="true"])`、`[role="button"]:not([aria-disabled="true"])`，位於 `@layer base`。

**Failure modes**

- 元件寫死 `cursor-pointer`／`cursor-not-allowed` → cursorRule 守門失敗並列出檔案。
- 變數預設值、全域規則或桌面覆寫缺漏 → cursorRule 守門失敗並指出缺哪一項。
- 非按鈕的可點元素漏加變數 class → 桌面無差（本來就是箭頭）、網頁該處為箭頭；守門抓不到，由 D3 的位置清單與手動驗收補位。

**Acceptance criteria**

- `npm test -w packages/ui`（含新守門與更新後的 listRow／taskList／deltaSpecView／specDetailPage／ticketView／roundsDoc／designDoc 斷言：原本檢查 `cursor-pointer` 的改檢查 `cursor-(--cursor-interaction)`）全綠。
- `npm test -w apps/desktop`、`npm test -w apps/server-web` 全綠；`npm run build -w apps/desktop` 與 `npm run build -w apps/server-web` 成功，且建置出的 CSS 含 `cursor:var(--cursor-interaction)`。
- 手動（macOS 桌面）：輪次卡頭、規格清單列、工具列按鈕、文件樹節點、專案欄導覽列、「開啟 ▾」選單項皆為箭頭且有 hover 底色；Markdown 超連結為手指；已封存變更任務列與停用選擇卡為箭頭且無 hover。手動（瀏覽器網頁後台）：建立按鈕、可選取列、下拉選單項為手指；停用按鈕為箭頭。

**Scope boundaries**

- In scope：theme.css 變數與全域規則、桌面 index.css 覆寫、D3 列出的位置、D4 的停用態、D5 的選單項、D6 守門、三份規格差異。
- Out of scope：hover 深淺、拖曳／文字／調整尺寸游標、系統匣原生選單、網頁後台的 hover 可見度、任何 CLI／引擎／Rust 程式。

## Risks / Trade-offs

- [Tailwind 未產生 `cursor-(--cursor-interaction)` 的 CSS] → Tailwind 4.3.2 支援括號變數寫法；驗收時建置並在 dist CSS 搜尋 `cursor:var(--cursor-interaction)`。
- [全域規則被 preflight 或匯入順序蓋掉] → theme.css 在 `@import "tailwindcss"` 之後匯入，同屬 base layer 時後者勝；守門只能檢查文字存在，實際效果靠手動驗收。
- [無 role 的可點元素漏加] → 只影響網頁（該處為箭頭），桌面不受影響；D3 列出現有全部位置，apply 時逐一核對。
- [使用者習慣上次加的手指] → proposal 已標明相容性影響：桌面版 15 處改回箭頭、hover 保留。
- [回歸對照與跨平台] → 不動 CLI 與 golden；Windows／Linux 桌面同樣為箭頭，與兩平台原生按鈕慣例一致；WKWebView 與 Chromium 都支援 CSS 變數游標。

## Migration Plan

無資料遷移。回退＝還原本變更的樣式與 class 修改；規格差異隨封存併入，回退需另開變更。

## Open Questions

（none）
