## Context

semantic-color-system 與 verify-station-parity 落地後的使用者回饋批次。五個病根都已在討論 quality-skill-pause-and-ui-polish 定位：詳情抽屜狀態列單行塞入進度＋兩站章＋日期＋完整作者身分且不可壓縮，被抽屜的水平溢出裁切；主題化提示延遲不一致（詳情抽屜 0ms、卡片沿用元件庫預設 700ms）造成「tooltip 被移除」的感知；系統匣站章無 hover 前景色覆寫；截斷收尾兩制並存（卡片標題漸層淡出 vs 其餘全部省略號）；指令檔過期提示隨捲動離開可視區。全部屬前端呈現層——packages/ui（desktop 與 server-web 同源消費）與 apps/desktop，無 Rust／引擎／協定改動，desktop-core 與 Tauri 殼零改動。

## Goals / Non-Goals

**Goals:**

- 主題化提示單一共用延遲，desktop 與 server-web 同源生效
- 詳情抽屜狀態列任何資料組合單行不溢出，日期與完整識別收進提示（與出身列同構）
- 系統匣站章 hover 隨列改前景色
- 截斷收尾全系統統一省略號，CardNameRow 順勢簡化
- 指令檔過期提示捲動釘選、不透明底

**Non-Goals:**

- 不動兩站章紫色配色本身與「品質站蓋章配色與主色分離」約束
- 不動系統匣原生 title 提示機制
- 不動看板卡片其他解剖與 i18n 詞條
- 不含 quality skill 暫停制（另一變更 quality-skill-round-pause）

## Decisions

### D1 主題化提示延遲下沉共用元件

延遲預設（300ms）寫進 packages/ui/src/components/ui/tooltip.tsx 的共用元件層，所有消費端（看板卡片、詳情抽屜、規格清單、討論欄等）自然繼承；RichDetailDrawer.tsx 現行的 0ms 覆寫移除。元件庫既有的 skipDelay 行為保留——連續於多個觸發點間移動時第二個起立即顯示，密集區不因 300ms 而遲鈍。替代方案「各表面自訂延遲」否決：本次病根正是各處自訂造成的不一致。系統匣面板刻意用原生 title（既有 doc comment 記載的決定）不動。

### D2 狀態列章籤化與提示同構

詳情抽屜狀態列重排為：進度條＋完成百分比＋站章籤（圖示＋狀態詞，審查在前、驗證在後，i18n 沿用既有詞條）。蓋章日期與蓋章者完整識別（含 email）收進主題化提示——直接沿用出身列已驗證的同構 pattern（可視文字去 email、完整識別於提示）。狀態列容器補最小寬度壓縮（min-w-0）與單行約束，任何資料組合不再觸發抽屜的水平溢出裁切。替代方案「日期作者常駐換行堆疊」於討論否決（使用者選提示收納）。

### D3 系統匣章色隨列反白

TrayPanel.tsx 的 StationBadges 兩章補 group-hover 前景色覆寫（text-primary-foreground token），與同列名稱、任務數、進度條的既有 hover 處理一致；非 hover 維持紫色調不變，hover 期間站別由圖示形狀（徽章形／盾牌形）承辨。不引入任何原生色階字面，主題守門測試不受影響。「品質站蓋章配色與主色分離」的表面約束列舉看板卡片、詳情抽屜、已封存清單、已封存抽屜四處，不含系統匣——本決策不觸及該 requirement。

### D4 截斷統一省略號與 CardNameRow 簡化

CardNameRow.tsx 移除漸層遮罩常數與 ResizeObserver 寬度量測，標題改用 CSS 省略號截斷（truncate）；複製鈕維持同列尾隨。變更卡與討論卡共用此元件、一次生效。此為對 card-name-single-line-fade（2026-08-04）規格決定的刻意翻案，理由記於提案；淡出全系統僅此一處，統一後截斷語意單一。順帶簡化：量測邏輯與條件遮罩皆為淡出而存在，一併刪除。

### D5 過期提示捲動釘選

InstructionUpdatePrompt.tsx 根元素改 sticky 頂部釘選＋不透明底＋高於內容的層級；未捲動時版面與現行一致（原位、原間距）。其捲動容器是設定類視圖的主內容區（overflow-y-auto），sticky 直接以該容器為參考，不需改 App.tsx 結構。現行半透明底（bg-muted/40）在釘選後會透字，改為不透明底並於深淺主題各自審視。

### D6 省略號字形統一

（實機走查追加。）D4 讓截斷收尾統一為省略號後，浮現第二層不一致：省略號長什麼樣由該元素的字型決定——等寬把手（`font-mono` → SF Mono）畫出半形貼基線的省略號，一般中文文字（body → Noto Sans TC）畫出全形置中的省略號，同一張卡上兩種收尾並列。使用者裁定統一為拉丁半形貼底。

作法是在 packages/ui/src/theme.css 定義一個 `unicode-range: U+2026` 的 `@font-face`（family 名 `EllipsisLatin`，`src` 以 `local()` 列出各平台的拉丁字型：macOS 的 Helvetica Neue／Arial、Windows 的 Segoe UI、Linux 的 DejaVu Sans），插在 body 字型堆疊最前面。`unicode-range` 讓這層只接管省略號一個碼位，其餘字元照舊落到 Noto Sans TC——沒有任何一段文字換字型。`font-mono` 元素以 Tailwind 的 `--font-mono` 覆蓋 body 堆疊，本來就畫半形省略號，不受這層影響也不需要改。

替代方案「把等寬把手也換成 Noto Sans TC」於本次裁定否決：那會一併推翻「標題以等寬字型呈現」的把手語意，且讓省略號倒向全形置中，與裁定相反。替代方案「逐一在會截斷的元素加 class」否決：截斷處散在兩個 app 十餘個元件，逐處掛 class 是持續性的維護負債，而 `unicode-range` 一層就收斂。

已知副作用並接受：body 堆疊涵蓋 markdown 內文，中文段落裡作者自己輸入的「……」也會改用拉丁字形（偏低）。省略號在技術文件內文極少出現，不值得為它在 desktop 與 server-web 各補一份還原規則（`.markdown` 樣式目前只存在於 apps/desktop/src/index.css，補了反而讓兩端不同源）。`local()` 全數落空時整層 `@font-face` 無效、退回 Noto Sans TC，即維持現狀而非破圖。

## Implementation Contract

- **行為**：（1）任一主題化提示 hover 停留 300ms 顯示，卡片與抽屜一致；（2）開啟含兩站蓋章、蓋章者含 email 的變更詳情抽屜，狀態列顯示進度、百分比與兩枚章籤，停留章籤時提示顯示蓋章日期與完整識別，狀態列單行、抽屜無水平溢出裁切；（3）系統匣變更列 hover 反白時兩站章與同列元素同為前景色，離開回復紫色；（4）卡片與討論卡標題過長時以省略號收尾、不淡出、複製鈕同列，短標題完整顯示無省略號；（5）專案設定頁捲動時過期提示釘選頂部持續可見、底不透字；（6）同一畫面上等寬把手與中文文字的截斷省略號為同一字形（拉丁半形、貼基線），兩者文字本身的字型不變。
- **介面／資料形狀**：無協定、CLI、--json、i18n key 變更；元件 props 對外不變（僅內部樣式與結構）。
- **失敗模式**：無新增失敗路徑；提示、章籤與釘選皆為純呈現，資料缺席時依既有缺席規則整段缺席。省略號字形層以 `local()` 解析，全數落空時該層無效、退回既有字型，即維持現狀。
- **驗收**：packages/ui 與 apps/desktop 的 vitest 全綠（含新增 tooltipDelay 測試與改寫後的 cardNameRow、richDrawer、trayPanel、instructionUpdatePrompt、theme 測試）；實機走查六項行為（tauri dev 載入靜態 dist，先重建 desktop 前端）。
- **範圍界線**：in——packages/ui/src/components/ui/tooltip.tsx、RichDetailDrawer.tsx、CardNameRow.tsx、SourceDiscussionChip.tsx（其 0ms 覆寫同屬「個別介面不得自訂延遲」的糾察對象）、theme.css、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/components/InstructionUpdatePrompt.tsx 與對應測試（含隨 D2 改寫斷言的 reviewBadge、verifyBadge 測試）。out——Rust 各 crate、Tauri 殼、協定與 CLI、系統匣原生 title、i18n 詞條、看板卡片其他元素、任何一段文字的字型歸屬（等寬把手、markdown 程式碼、一般文字皆不換字型）。

## Risks / Trade-offs

- [詳情抽屜提示從 0ms 變 300ms 的體感差] → 討論已裁定一致優先；skipDelay 使連續 hover 即時，實機走查確認
- [cardNameRow 既有測試斷言淡出與量測，殘留斷言會誤導後續維護] → 紅測先行改寫，斷言省略號樣式且明確斷言遮罩標記缺席
- [sticky 底透字或層級遮擋] → 不透明底＋層級高於內容，深淺主題實機各查一次
- [主題守門測試（原生色階字面）誤觸] → D3 僅用既有 token 類別，不新增色階字面
- [省略號字形層在缺字型的機器上無效] → `local()` 依序列出三平台的常駐拉丁字型；全數落空只是退回現狀，theme 測試釘住宣告存在、實機走查確認字形
- [中文內文的「……」跟著改用拉丁字形] → 已於 D6 記載並接受；省略號在技術文件內文極少出現，不為它讓兩端樣式分家
- [回歸對照] 純前端變更，golden 與 CLI 測試不受影響；vitest 全綠與實機走查為驗收門檻
- [跨平台] 站章僅存在於 macOS 面板（原生選單不含站章，行為維持）；其餘表面為 web 技術，三平台一致
