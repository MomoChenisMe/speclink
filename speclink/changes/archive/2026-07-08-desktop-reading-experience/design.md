## Context

桌面 app 的 markdown 渲染樣式為手工維護的 .markdown 區塊（apps/desktop/src/index.css），已踩到 Tailwind v4 preflight 重置 list-style 未被恢復的坑；字體排序使 Noto Sans TC 永遠輪不到；討論記錄 scaffold 的 HTML 註解以原文滲入渲染結果。本刀全落在前端（packages/ui 元件庫與 apps/desktop 殼），speclink-core／speclink-cli 兩個 crate 不動，CLI 輸出與回歸對照不受影響。相關者：以桌面 app 閱讀 SDD 文件的開發者／PO／PM。

## Goals / Non-Goals

**Goals**
- markdown 內容在抽屜與已封存檢視保留段落／列表／單換行結構，基準字級 16px。
- 介面文字在任何機器（含未安裝字體、離線）以 Noto Sans TC 呈現。
- raw HTML（含 scaffold 註解）不以原文呈現。
- 排版樣式在淺色與深色主題一致生效。

**Non-Goals**
- 任務工具列與勾選樂觀更新（desktop-task-interactions 刀）。
- 規格 nav 頁（desktop-specs-view 刀）。
- CLI 討論 scaffold 內容不動；Streamdown 與 github-markdown-css 不引入（討論已否決）。

## Decisions

### D1：typography 接管排版

排版交給 @tailwindcss/typography：index.css 以 @plugin 指令啟用插件；Markdown 元件的容器 class 由 .markdown 改為 prose 系（含 max-w-none 解除寬度上限），並保留一個自有掛鉤 class 供薄覆寫定錨。prose 出廠基準字級即 16px，同時滿足字級需求。
薄覆寫保留三件 Spectra 特調：teal code chip（inline code 的主色底），GFM 任務清單 checkbox 的懸掛縮排與完成刪除線，表格橫向捲動（prose 不處理 overflow）。
替代案：繼續手工維護 .markdown——list-style 坑即維護成本實證，否決；github-markdown-css——自帶配色系統與 oklch token 打架，否決。

### D2：prose token 映射

在樣式層一次性把 --tw-prose-body、--tw-prose-headings、--tw-prose-links、--tw-prose-bold、--tw-prose-code、--tw-prose-quotes、--tw-prose-hr、--tw-prose-th-borders 等變數指向既有設計 token（--foreground、--primary、--border、--muted-foreground）。token 本身已隨系統深色偏好翻轉，深色模式自動跟隨。
替代案：prose-invert——需要 .dark class 切換，本 app 深色走 media query 而非 class，不適用。

### D3：字體打包

apps/desktop 新增 @fontsource-variable/noto-sans-tc 依賴（可變字重版），於前端進入點單一 import——一個 woff2 檔族涵蓋 100–900 全字重（UI 用到的 400／500／600／700 皆為真字重），且僅出 woff2 單一格式。body font-family 改為 Noto Sans TC Variable 第一優先（Noto Sans TC、Segoe UI、system-ui 為後備）；等寬字體（Cascadia Code 系）維持不變。fontsource 的 CSS 以 unicode-range 切片宣告，Vite 打包後隨 app 分發（實測約 4.2 MB），離線可用。
替代案：只調 font-family 順序不打包——依賴使用者機器有無安裝，離線與新機不生效，討論已否決；靜態字重版 @fontsource/noto-sans-tc——每字重附 woff 舊格式備援（WebView2 永不取用的死重），三字重實測 15.5 MB 且 600 需以 700 替代，實作時量測後否決。

### D4：渲染端過濾與換行

Markdown 元件開 skipHtml（raw HTML 節點一律丟棄——本 app 從未渲染 raw HTML，原文顯示只是雜訊）並加 remark-breaks（單換行→硬換行）。討論記錄每輪 **Focus**／**Position** 各佔一行的格式依賴 remark-breaks 才有段落感。
替代案：改 CLI scaffold 移除註解——註解是 agent／CLI 的功能性錨點，否決；只濾註解不丟其他 raw HTML（自訂 remark plugin）——多養一個 plugin 換不來實益，否決。

### D5：任務字級對齊

TaskList 非 markdown 渲染（解析後自組 DOM），任務文字由 13px 改 16px、群組標題同步放大一級，與抽屜其他分頁視覺一致。
替代案：任務維持小字——同一抽屜兩種字級，違背 16px 統一訴求，否決。

## Implementation Contract

**可觀察行為**
1. 變更抽屜（提案／設計／規格分頁）、討論抽屜、已封存檢視渲染 markdown 時：無序清單顯示列表符號、有序清單顯示編號、段落間距可辨、來源單換行呈現為換行、基準字級 16px。
2. 討論過程分頁不出現 scaffold 註解原文（渲染結果無「<!--」序列）；openspec/ 下來源檔案位元不變。
3. body 的第一優先字體為 Noto Sans TC，字體資產包含在建置產物內（無網路請求）；inline code 與程式碼區塊維持等寬字體。
4. 任務分頁任務文字 16px；勾選、拖曳排序行為與現況相同（本刀不動互動邏輯）。
5. 淺色與深色主題下上述排版一致生效，內容色彩取自既有設計 token。

**驗收目標**
- packages/ui 測試：Markdown 渲染斷言——單換行產生換行、HTML 註解文字不出現、容器帶 prose 系 class；TaskList 字級 class 斷言。npm test -w packages/ui 全綠。
- apps/desktop 測試與建置：npm test -w apps/desktop 全綠；npm run build -w apps/desktop 成功且產物含字體資產。
- 真實視窗驗證（jsdom 測不出視覺）：release exe 開啟變更抽屜與討論抽屜截圖，確認列表符號、段落間距、16px、字形為 Noto Sans TC、註解消失。

**範圍邊界**
- In scope：packages/ui 的 Markdown 與 TaskList 元件、apps/desktop 的樣式進入點與字體載入、兩處 package.json 依賴。
- Out of scope：Rust 兩 crate、CLI 輸出、tasks 寫回與看板互動邏輯、討論 scaffold 內容。

## Risks / Trade-offs

- [remark-breaks 改變既有文件的渲染——來源若有「長行硬折」慣例會多出換行] → 專案 artifacts 皆 agent 產出、單行式書寫；實作時抽查數份既有變更與已封存文件對照渲染，個案差異屬可接受的呈現變化。
- [prose 的 ul/li 間距規則與 GFM checkbox 懸掛縮排疊加後錯位] → 薄覆寫定錨在自有掛鉤 class 上、特異性高於 prose 預設；jsdom 斷言＋真實視窗截圖雙重驗證。
- [安裝包體積增加約 5-10 MB] → 只 import 用到的字重；體積屬討論已接受的取捨。
- [回歸對照] → 純前端變更，CLI 人眼與 --json 輸出零接觸，parity／color 對照不受影響。
- [跨平台] → 字體打包後三平台字形一致，反而消除 Windows 微軟正黑／macOS 蘋方的呈現分歧。

## Migration Plan

無資料遷移。樣式與依賴變更隨一般建置發佈；回滾即還原 commit 重建。

## Open Questions

（無——關鍵取捨已在討論 desktop-reading-and-tasks-ux 定案）
