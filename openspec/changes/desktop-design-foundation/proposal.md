## Summary

把桌面 app（apps/desktop）、Server Web（apps/server-web）與共用元件庫（packages/ui）的視覺地基換成新風格：色彩統整成「色值只在 theme.css、對照只在 TS 表、元件只寫 token class」兩層，原語改為白底中性灰、細框、圓角分層、浮層才有陰影的扁平外觀，markdown 渲染換成 Streamdown，重複定義的共用元件與品牌資產（使用者已重畫的「接合 S」標記，SVG 原稿在 docs/assets/brand/svg）收回 packages/ui，並以守門測試釘死。這是討論 desktop-ui-redesign 的 cut 1；畫面版面在這一刀之後仍是舊的，顏色與元件已經是新的。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境是桌面 app 全部頁面（看板、抽屜、規格、手冊、設定）與 Server Web 主控台。三個問題：

- 色彩散在三層：`packages/ui/src/theme.css` 的 16 個基底 token、`tone.ts`／`stage.ts`／`reviewStyle.tsx`／`verifyStyle.tsx`／`improveStyle.tsx`／`DeltaBadges.tsx` 以原生色階成對寫（`violet-600 dark:violet-400`）、再加元件上的填滿色與陰影。換一個色相要改三處，深色要寫兩遍，守門靠白名單。
- 同一個元件各寫各的：`CopyButton` 定義 4 次、`EmptyState` 2 次、`SectionHeader` 2 次、確認框以 `AlertDialogContent` 四件組手拼 13 次（App.tsx）＋ 3 次（ProjectSettingsView）＋ 12 次（server-web 四頁）；換皮時每一份都要再改一次。
- markdown 渲染沒有程式碼上色與複製鈕，`.markdown` 覆寫 11 條靠 prose 變數對照 token 維持。

後面四刀（外殼、看板與抽屜、設定模式、系統匣）都站在這一刀的 token、原語與守門上，所以先做。

## Proposed Solution

1. **色彩兩層**：`theme.css` 新增語意 token（`status-progress`、`status-success`、`status-warning`、`stamp`、`improve`）、生命週期 token（`stage-discussion`、`stage-proposed`、`stage-in-progress`、`stage-ready`）與面 token（`sidebar`），淺色深色各寫一次並映射成 Tailwind utility；深色基底去掉藍色調改純中性；圓角改用 Tailwind 預設四階（md 6px、lg 8px、xl 12px、2xl 16px），移除自訂 `--radius`。TS 常數表縮成「狀態→token class」對照；集中常數檔以外不再有任何原生語意色階字面。看板欄頂條、欄計數徽章、卡片波次章、系統匣生命週期分區與討論分區的圖示與計數全部改取生命週期 token——這一刀就把四個色相落到所有消費者，後面的刀只改形狀。`--primary` 淺色、深色各以品牌固定色（青綠 `#167873`、淺青綠 `#4bb9b3`）換算的 oklch 值校準一次，讓介面主色與 Logo 肉眼一致；品牌固定色本身留在 docs/assets/brand 的 README，不進 token。
2. **原語扁平化**：17 個原語改成新外觀（按鈕 6px 圓角與新變體、卡片 16px 圓角無陰影（容器內小卡 12px 變體）、徽章全圓、分頁改卡片標頭式底線（分頁列是內容卡的頂部）、輸入與下拉 h-8 細框無陰影、浮層才有陰影、對話框 12px 圓角與 30% 中性遮罩）；抽屜（Sheet）遮罩與寬度留給 cut 3。
3. **共用元件收口**：新增 `ConfirmDialog`、`CopyButton`、`EmptyState`、`SectionHeader`、`BrandMark`、`Wordmark` 六個元件，所有重複定義改 import；Logo 資產集中到 packages/ui 一處：共用元件庫收三個 SVG（裸標記、橫式鎖版淺色、橫式鎖版深色），`BrandMark` 把標記以 currentColor 內嵌（淺色呈主色 teal、深色呈淺青綠，跟 `text-primary` 走），`Wordmark` 以 img 呈現橫式鎖版並依系統偏好切深色版；桌面 app 與 server web 的 PNG 副本刪除，`apps/desktop/public/logo-mark.png` 只留給 favicon。本刀的既有頂欄與 server web 殼只把舊 PNG 換成新元件、高度不變；裸標記放圖示列頂、鎖版放空狀態與「關於」的擺放隨 cut 2 落地。沒有消費者的原語（dropdown-menu、switch、search-input、status-pill、kbd、page-header、settings-card／row）不在這一刀建立，各自隨第一個消費者所在的刀落地，避免無人使用的程式碼。
4. **守門**：`theme.test.ts` 改為全掃描面無白名單的原生色階禁令，token 快照換新值；新增「apps/* 不得定義與 @speclink/ui 匯出同名的元件」守門測試；`openspec/config.yaml` 的 design 產出規則加一條「優先用 @speclink/ui 既有元件、呼叫端不覆蓋變體的顏色／圓角／陰影」。
5. **Markdown 換 Streamdown**：static 模式，選配 `@streamdown/code` 做 Shiki 上色（淺色 github-light、深色 github-dark），內建複製鈕接 lucide 圖示與 i18n；搬四個既有行為（GitHub Alert、單換行＝換行、raw HTML 不進畫面、行寬上限 96ch）；移除 `@tailwindcss/typography` 與 prose 變數對照。

不新增或變更任何 CLI 指令、技能或 Agent 指令。設定欄位：`openspec/config.yaml` 的 `rules.design` 多一條規則字串（無預設值概念，純文字清單）。相容性：`--json` 與人眼 CLI 輸出不變；前端測試的 class 斷言與 theme 快照會更新。

## Non-Goals

- 不改版面：側欄、頂列、標題列、看板欄形狀、抽屜骨架、設定頁、系統匣面板的結構與尺寸都留給 cut 2–5。
- 不建立尚無消費者的原語。
- 不重畫 Logo：新標記、原生 App 圖示（`tauri icon` 產物）與系統匣單色圖示（`trayIcon.ts` 的 base64 常數）已由使用者在本刀外完成；本刀只集中介面資產與渲染點。也不搬動品牌在畫面上的位置（圖示列頂、空狀態、關於），那是 cut 2 的外殼範圍。
- 不動 Sheet 原語（詳情頁取代抽屜後於 cut 3b 退場）。
- 不改 Tailwind 中性色階的用法：掃描面現在零處使用中性色階字面，禁令一併涵蓋它們不需要任何改寫。
- 不動 Rust 端與 CLI。

## Alternatives Considered

- 維持三層只改色值——深色成對寫法與白名單守門繼續存在，換色仍要改三處。
- 語意色全搬進 TS 表——深色仍寫兩遍，server-web 與系統匣沒有共用來源。
- react-markdown 加 `@shikijs/rehype`——使用者裁定要 Streamdown 整套程式碼區塊介面。
- 一次把 11 個原語全建好——沒有消費者的原語是死碼，等不到第一個消費者前無法驗證形狀。

## Impact

- Affected specs: `desktop-app`（MODIFIED「介面狀態語意色分層」——生命週期每階一色相、計數徽章隨分區色、守門改全掃描面無白名單；ADDED「共用元件唯一來源」）
- Affected code:
  - Modified: `packages/ui/src/theme.css`、`packages/ui/src/tone.ts`、`packages/ui/src/stage.ts`、`packages/ui/src/components/reviewStyle.tsx`、`packages/ui/src/components/verifyStyle.tsx`、`packages/ui/src/components/improveStyle.tsx`、`packages/ui/src/components/DeltaBadges.tsx`、`packages/ui/src/components/Markdown.tsx`、`packages/ui/src/components/DiscussionColumn.tsx`、`packages/ui/src/components/KanbanBoard.tsx`、`packages/ui/src/components/ArchivedList.tsx`、`packages/ui/src/components/DocumentTree.tsx`、`packages/ui/src/components/ui/button.tsx`、`packages/ui/src/components/ui/card.tsx`、`packages/ui/src/components/ui/badge.tsx`、`packages/ui/src/components/ui/tabs.tsx`、`packages/ui/src/components/ui/tooltip.tsx`、`packages/ui/src/components/ui/input.tsx`、`packages/ui/src/components/ui/textarea.tsx`、`packages/ui/src/components/ui/select.tsx`、`packages/ui/src/components/ui/popover.tsx`、`packages/ui/src/components/ui/alert-dialog.tsx`、`packages/ui/src/components/ui/sonner.tsx`、`packages/ui/src/components/ui/checkbox.tsx`、`packages/ui/src/i18n.tsx`、`packages/ui/src/index.ts`、`packages/ui/package.json`、`packages/ui/src/__tests__/theme.test.ts`、`packages/ui/src/__tests__/markdownAlerts.test.tsx`、`apps/desktop/src/index.css`、`apps/desktop/src/App.tsx`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/components/connectionLogin.tsx`、`apps/desktop/src/panel/TrayPanel.tsx`、`apps/desktop/package.json`、`apps/server-web/src/index.css`、`apps/server-web/src/layouts/ConsoleLayout.tsx`、`apps/server-web/src/layouts/FocusLayout.tsx`、`apps/server-web/src/pages/admin/UsersPage.tsx`、`apps/server-web/src/pages/admin/SystemPage.tsx`、`apps/server-web/src/pages/admin/CredentialsPage.tsx`、`apps/server-web/src/pages/AccountPage.tsx`、`openspec/config.yaml`、`docs/assets/brand/README.md`、`docs/assets/brand/README.en.md`、`.gitignore`、受 class 斷言影響的既有 vitest 檔
  - New: `packages/ui/src/components/ui/confirm-dialog.tsx`、`packages/ui/src/components/CopyButton.tsx`、`packages/ui/src/components/EmptyState.tsx`、`packages/ui/src/components/SectionHeader.tsx`、`packages/ui/src/components/Brand.tsx`、`packages/ui/src/assets/logo-mark.svg`、`packages/ui/src/assets/logo-horizontal.svg`、`packages/ui/src/assets/logo-horizontal-dark.svg`、`packages/ui/src/vite-env.d.ts`、`packages/ui/src/__tests__/brand.test.tsx`、`packages/ui/src/__tests__/uiSingleSource.test.ts`
  - Removed: `apps/server-web/src/components/CopyButton.tsx`、`apps/server-web/src/components/EmptyState.tsx`、`apps/server-web/src/components/Wordmark.tsx`、`apps/server-web/src/assets/logo-mark.png`、`apps/server-web/src/assets/speclink-wordmark.png`、`apps/server-web/src/assets/speclink-wordmark-dark.png`、`apps/desktop/public/speclink-wordmark.png`、`apps/desktop/public/speclink-wordmark-dark.png`（`apps/desktop/public/logo-mark.png` 仍供 index.html 的 favicon 使用，保留）
