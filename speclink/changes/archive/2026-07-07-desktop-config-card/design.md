## Context

設定頁(apps/desktop/src/views/SettingsView.tsx)現有「專案說明」多行文字區與「產出規則」逐項輸入框編輯器(上下移/刪除/新增按鈕),永遠處於可編輯狀態,各自帶儲存鈕。底層管線已完備:desktop-core 的設定讀取 payload 提供 context、rules、schemaArtifacts 與 parseError,寫入走雙重解析驗證並沿用 speclink-core 的 text→text 純函式(context 三態、rules 整份代換、保留字元自動加引號)。2026-07-07 討論「desktop-導覽與專案首頁重構」定案改為 Spectra 式唯讀優先卡片,但保持 Speclink 設計語言;產出規則改整份文字編輯。本變更為純前端重構——不動 Rust crate(speclink-core、speclink-cli、desktop-core 皆零改動),不涉 git 互動與 serde 結構變更。

## Goals / Non-Goals

**Goals:**

- 設定頁頂部「專案設定」卡:專案說明/產出規則分頁、唯讀優先、就地編輯(取消/儲存)。
- 產出規則整份文字編輯:每 schema 鍵一個多行文字區、一行一條規則,行序即寫入順序。
- 寫入語意與既有契約逐字元等價(trim、空行滌除、清空移除鍵、雙重驗證、自動加引號)。

**Non-Goals:**

- 不動 App 區與專案政策區(locale、spec_locale、tdd、audit)的表單形態。
- 不做獨立首頁視圖、統計卡;不動導覽(刀一 desktop-nav-reorder 的守備範圍)。
- 不改 desktop-core 讀寫橋接與 core 純函式的介面。

## Decisions

### D1 專案設定卡唯讀優先與就地編輯

卡片預設唯讀,右上單一編輯鈕進入編輯態(卡層級,兩分頁共享編輯態),按鈕列切為取消/儲存;取消還原讀取值且不觸發寫入,儲存collect兩分頁的變更一次寫入。config.yaml 解析失敗(parseError)時唯讀區顯示解析失敗說明並停用編輯鈕。替代方案:每分頁獨立編輯態——同檔兩處編輯態易產生「一頁儲存蓋掉另一頁未儲存」的混淆,捨棄;維持永遠可編輯的現狀——長文閱讀性差、與討論定案不符,捨棄。

### D2 產出規則整份文字編輯與行序語意

編輯態為活躍 schema 的每個 artifact 鍵渲染一個多行文字區(固定鍵、無自由鍵輸入),值為該鍵條目以換行串接;儲存時逐行修剪頭尾空白、空行滌除,行序即 Vec 順序,直接組成既有寫入 payload 的 rules 整份代換——語意與逐項編輯器完全等價,橋接層零改動。空文字區=該鍵移除;全部為空=rules 鍵移除(沿用既有語意)。替代方案:保留逐項輸入框編輯器——操作繁瑣被使用者否決;單一大文字區手寫鍵名分節——易格式錯且需新解析邏輯,捨棄。取捨:單行條目內無法包含換行——與現行逐項輸入框限制一致,無倒退。

### D3 專案說明 markdown 渲染與收合

唯讀態以 packages/ui 既有 Markdown 元件渲染 context 內容;超過固定高度時截斷並顯示「顯示更多」展開(純前端狀態,不持久化)。編輯態為 raw markdown 等寬多行文字區。未設定 context 時唯讀態顯示空狀態提示文字。替代方案:所見即所得編輯器——引入外部依賴且與 config.yaml 純文字本質不符,捨棄。

### D4 卡片文案與 i18n

分頁名沿用正典詞「專案說明」「產出規則」,卡標題「專案設定」與檔名標註 config.yaml;新增字串(編輯、取消、儲存、顯示更多、空狀態、解析失敗說明等)全數進 i18n 字典,zh-TW 與 en 鍵集合相等。替代方案:硬編 zh-TW——違反既有 i18n 需求,捨棄。

## Implementation Contract

- 行為:開啟設定頁即見頂部專案設定卡,唯讀呈現 config.yaml 的 context(markdown 渲染)與 rules(僅列有條目鍵);點編輯就地切換,儲存後檔案效果與舊逐項編輯器逐字元等價,取消後畫面還原且磁碟檔案不變。
- 介面/資料形狀:讀取沿用既有設定 payload 欄位(context、rules、schemaArtifacts、parseError,camelCase);寫入沿用既有寫入函式的 payload(context 三態、rules 整份代換)。前端新增的行↔條目轉換為 SettingsView 內部純函式:字串陣列以換行 join 呈現、split 後逐行 trim 並滌除空行還原為陣列。
- 失敗模式:寫入任一階段失敗時顯示既有單行錯誤(指明檔案與階段)、維持編輯態不遺失輸入;parseError 時編輯鈕停用並浮出解析失敗說明——不得靜默。
- 驗收:apps/desktop 的 vitest 全綠(涵蓋 MODIFIED 需求「設定頁編輯專案說明與產出規則」全部場景);真實視窗操作核對檔案效果(行序對調、清空移除鍵、保留字元條目、壞檔停用)。
- 範圍邊界:in——SettingsView.tsx、settingsView.test.tsx、messages.ts;out——App.tsx 與導覽、desktop-core/src 與 crates、政策區表單。

## Risks / Trade-offs

- [使用者在文字區貼入含前導空白的行,期待保留縮排] → 逐行 trim 為既有寫入契約,於分頁說明文字載明一行一條規則、頭尾空白不保留。
- [卡層級編輯態下,使用者只改一分頁但儲存寫入兩分頁資料] → 儲存一律以兩分頁當前值組 payload,與讀取值相同的部分寫回等值內容,檔案效果冪等;測試涵蓋「未動分頁原樣保留」。
- [Markdown 元件對非預期語法的渲染差異] → 唯讀渲染僅作呈現,儲存值以文字區原文為準,渲染異常不影響資料;真實視窗驗證含一份含標題與代碼片段的 context。
- [回歸對照] → 本變更零 Rust 改動,CLI 人眼與 --json 輸出不受影響;desktop 測試沿用既有測試基建,無跨平台新風險。
