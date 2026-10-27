<!-- 開發依專案 TDD 慣例:每個介面面先寫紅燈測試,再實作轉綠。 -->

## 1. Popover 原語與相依

- [x] 1.1 packages/ui/package.json 增列 @radix-ui/react-popover 並安裝——契約:@speclink/ui 內 import 該套件可解析、與既有 radix 家族同版本線;驗證:npm test -w @speclink/ui 全綠(套件無獨立 build script,型別檢查由 apps/desktop 的 vite build 涵蓋)。 <!-- speclink-task:tsk_01KZ5BN3V6JJ5ZY459MMF78FKA -->
- [x] 1.2 新增 packages/ui/src/components/ui/popover.tsx(shadcn Popover 原語;浮層底色比照 packages/ui/src/components/ui/select.tsx 的主題註解用 bg-card,主題無 --popover 變數)——契約:Popover 支援點擊開啟、Esc 與點擊外部關閉;驗證:任務 2.1 的溢出浮層測試以其為載體轉綠(先行以 build 通過為暫準)。 <!-- speclink-task:tsk_01KZ5BN3V63R9W8DN4XFEW0BZV -->

## 2. 來源討論列共用元件(slug 籤+溢出浮層)

- [x] 2.1 [測試先行] 於 packages/ui/src/__tests__/richDrawer.test.tsx 新增斷言,錨定規格「變更的來源討論多值呈現」、「抽屜標頭標記受寬度約束且抽屜不產生水平捲軸」與「討論抽屜檢視與轉出變更」(同源 change 互跳場景的 slug 籤呈現):籤面文字為討論 slug 且不含 topic 全文;主題化提示含 slug 與 topic;多筆來源討論時僅首籤(出身)直接渲染、「+N」籤呈現溢出數;點 +N 浮層列出其餘討論(slug 主行+topic 副行);點浮層項觸發 onOpenDiscussion 且浮層關閉;單筆來源討論無 +N 籤——契約:新案例紅燈(現實作 topic 直出);驗證:npm test -w @speclink/ui 顯示上述案例失敗、其餘既有案例不受影響。 <!-- speclink-task:tsk_01KZ5BN3V6W6W47KE1XM04X8MD -->
- [x] 2.2 SourceDiscussionChip props 改為 slug+topic+onClick(slug 直出、等寬字型、max-w-[140px] 截斷、shadcn Tooltip 呈現 slug 與 topic),同檔新增共用來源討論列元件(前綴標籤+首籤+「+N」Popover 浮層;「同源」籤比照同規則)——契約:任務 2.1 全數轉綠;驗證:npm test -w @speclink/ui 全綠。 <!-- speclink-task:tsk_01KZ5BN3V6NCSXGEC41NTC5KR7 -->

## 3. RichDetailDrawer 標頭四層重構

- [x] 3.1 [測試先行] richDrawer.test.tsx 增列規格「變更詳情抽屜標頭的四層結構」與「詳情抽屜的審查資訊列」的斷言:標頭可視文字無任務計數字樣;審查資訊(data-review-row)與進度條同列;建立者與開工者的 email 不出現於可視文字、Tooltip 保完整識別;4 筆來源討論+開工+同源的 fixture 下出身列容器為不折行單行——契約:新案例紅燈;驗證:npm test -w @speclink/ui。 <!-- speclink-task:tsk_01KZ5BN3V6XZZHK0K2A1TN4WJ9 -->
- [x] 3.2 packages/ui/src/components/RichDetailDrawer.tsx 標頭重構為四層:標題列(維持)/狀態列(進度條+百分比+審查章,none 時僅進度)/出身列(單行 flex 不折行+overflow 裁切兜底:頭像+名字+產生工具+相對時間+開工日期+來自籤列+同源籤列,經任務 2.2 共用元件)/動作列(維持);移除「N/N 任務」span 與 email 直出——契約:任務 3.1 轉綠且既有 richDrawer 案例全數通過;驗證:npm test -w @speclink/ui 全綠。 <!-- speclink-task:tsk_01KZ5BN3V69VF4MMMEPCQS3DHD -->

## 4. ArchivedDrawer 比照同構

- [x] 4.1 [測試先行] packages/ui/src/__tests__/archivedDrawer.test.tsx 增列斷言:封存變更抽屜的來源討論標記 slug 直出、提示含 slug 與 topic、多筆時 +N 浮層列出其餘且點擊觸發跳轉回呼——契約:新案例紅燈;驗證:npm test -w @speclink/ui。 <!-- speclink-task:tsk_01KZ5BN3V6TQG5TMY3S3MQ80BR -->
- [x] 4.2 packages/ui/src/components/ArchivedDrawer.tsx 改用任務 2.2 的共用來源討論列元件,呈現與變更詳情抽屜同構——契約:任務 4.1 轉綠、既有 archivedDrawer 案例不破;驗證:npm test -w @speclink/ui 全綠。 <!-- speclink-task:tsk_01KZ5BN3V61B66S88VD1Y0PHQ6 -->

## 5. 文案與 LANGUAGE.md

- [x] 5.1 packages/ui/src/i18n.tsx:rdrawer.fromDiscussion 由「來自討論:」改「來自」、rdrawer.siblings 由「同源:」改「同源」,英文對應 From/Siblings;新增 +N 籤的 aria-label key(zh/en,語意為「其餘 N 份」)——契約:出身列前綴縮短、+N 籤具可存取名稱;驗證:npm test -w @speclink/ui 以 i18n 文字斷言通過。 <!-- speclink-task:tsk_01KZ5BN3V6MQ0D1R2256PX3MTV -->
- [x] 5.2 openspec/LANGUAGE.md「slug 直出」明文例外的適用範圍清單增列:變更詳情抽屜與已封存抽屜的來源討論籤及其溢出浮層(註記範圍擴充:change-drawer-header-redesign,2026-08-04)——契約:speclink language show 輸出含新增範圍;驗證:內容審閱該行存在且沿用既有例外的行文格式。 <!-- speclink-task:tsk_01KZ5BN3V6AR521M8PEH6FE3YN -->

## 6. 整體驗證

- [x] 6.1 全套驗證與手動確認:npm test -w @speclink/ui 全綠、npm test -w apps/desktop 與 npm run build -w apps/desktop(vite 打包)通過;建置桌面 app 後手動開啟 verify-station-parity(4 筆來源討論)與 evidence-home-and-trace-slim(2 筆+開工+審查)的詳情抽屜及對應封存抽屜,確認 design.md Implementation Contract 驗收條件全數成立(四層結構、單行出身列、+N 浮層跳轉、無水平捲軸)——驗證:逐條核對 Implementation Contract 的觀察行為清單。 <!-- speclink-task:tsk_01KZ5BN3V6GKBAX13A0RRZE4Y3 -->
