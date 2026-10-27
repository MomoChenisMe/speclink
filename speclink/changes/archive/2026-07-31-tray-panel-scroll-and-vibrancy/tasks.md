## 1. 選單模型與動作接線（專案設定入口）

- [x] 1.1 撰寫 tray 選單模型測試：buildTrayModel 的動作區依序為 open、project-settings、settings、quit 四項（滿足「系統匣圖示與原生選單」動作區四項——macOS 面板與非 macOS 原生選單同一模型）。檔案：apps/desktop/src/__tests__/tray.test.ts。驗證：npm test -w apps/desktop 先紅（新斷言失敗、既有測試不受影響）。 <!-- speclink-task:tsk_01KYVP535WFQV40731QX5HT8EA -->
- [x] 1.2 實作模型層與接線：apps/desktop/src/tray.ts——TrayMenuItem 聯集新增 project-settings 動作項、buildTrayModel 動作區於 open 之後插入該項（文案沿用既有 app.navProjectSettings 詞條）、toOptions 新增 case 以 openIn 語意喚起主視窗並 setBoardView("project-settings")、TrayStoreApi.setBoardView 型別自 "settings" 放寬為含 "project-settings" 的聯集、tray-panel-action 事件 handler 新增 open-project-settings 分支。行為：原生選單點「專案設定」喚起主視窗並切至作用中專案的專案設定頁、recovery／stale 不特判（滿足「開啟視窗與結束動作」）。驗證：1.1 測試轉綠。 <!-- speclink-task:tsk_01KYVP535W3KCWYC7T30C5C105 -->
- [x] 1.3 面板動作區列：apps/desktop/src/panel/TrayPanel.tsx 動作區於「開啟 Speclink」與「設定」之間新增「專案設定」列（SlidersHorizontal 圖示沿用主視窗側欄同款、onOpenProjectSettings 回呼）；apps/desktop/src/panel/main.tsx 接線發出 open-project-settings 面板事件。行為：面板動作區四列依序為開啟 Speclink、專案設定、設定、結束，點擊「專案設定」發出對應事件。驗證：apps/desktop/src/__tests__/trayPanel.test.tsx 新增斷言（四列順序與點擊事件 payload），npm test -w apps/desktop 綠。 <!-- speclink-task:tsk_01KYVP535WHHWTDWVH9Y2ZX851 -->

## 2. 三段式版面與捲動範圍

- [x] 2.1 撰寫面板三段式版面測試：root 之下分固定頁首（專案 tab 條＋分割線）、可捲中段（討論／已轉出／生命週期分區，或復原卡／stale 條）、固定頁尾（分割線＋動作區四列）三個容器；中段容器帶縱向捲動樣式（overflow-y auto 類名）、頁首頁尾不帶（滿足「面板樣式（macOS）」三段式版面與分割線歸屬）。檔案：apps/desktop/src/__tests__/trayPanel.test.tsx。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KYVP535WXR647275G3HS6NH7 -->
- [x] 2.2 實作三段式版面：apps/desktop/src/panel/TrayPanel.tsx 把單一 flex column 拆為頁首／中段／頁尾——中段掛 flex-1 min-h-0 overflow-y-auto，recovery／stale 分支歸入中段、頁首頁尾結構不分支；主色漸層 wash 移為視窗固定（不隨內容捲動）；apps/desktop/panel.html 的 body 捲動關閉（overflow hidden）、overscroll-behavior:none 移入中段容器。行為：內容超過上限高度時僅中段捲動、tab 條與動作區常駐可見。驗證：2.1 測試轉綠。

  實測修正（4.2 驗證發現）：原假設「維持 WebKit 預設 overlay 捲軸」不成立——index.css 的全域 `::-webkit-scrollbar` 樣式使 WebKit 改用常駐且佔寬的自訂捲軸，系統偏好 AppleShowScrollBars=Always 亦強制常駐。已改為中段隱藏原生捲軸（scrollbar-width:none）＋ ScrollIndicator 自繪指示條（捲動時浮現、閒置 800ms 淡出、疊在內容上不佔寬）；vitest.setup.ts 補 ResizeObserver stub（jsdom 環境缺口）。 <!-- speclink-task:tsk_01KYVP535WQQH4MTM05QXW1951 -->
- [x] 2.3 高度自適應量測基準改寫：apps/desktop/src/panel/main.tsx 的 fit 量測由「#root 實高」改為「頁首實高＋中段自然內容高（scrollHeight）＋頁尾實高」，上限 640 不變。行為：內容未超限時面板貼合內容（無多餘捲動與空白）、超限時視窗停在上限並由中段內部捲動——避免視窗高與 root 高互相回饋的循環。驗證：npm run build -w apps/desktop 型別檢查通過；貼合與超限行為列入 4.2 真實視窗清單逐項確認。 <!-- speclink-task:tsk_01KYVP535WSM1BFDNJTHW2DJEA -->

## 3. 毛玻璃補光層

- [x] 3.1 補光底層：apps/desktop/src/panel/TrayPanel.tsx 面板 root 於毛玻璃之上、內容之下鋪主題背景色半透明補光層（bg-background 低透明度 token，隨 prefers-color-scheme 深淺自動）。行為：深色背景下面板亮度錨定主題背景色、不明顯偏暗；淺色背景下毛玻璃仍可辨（滿足「面板樣式（macOS）」補光要求）。驗證：trayPanel.test.tsx 斷言 root 補光層類名存在，npm test -w apps/desktop 綠。 <!-- speclink-task:tsk_01KYVP535WB33KJYFDKMWKVM16 -->
- [x] 3.2 真實視窗調參：於深色（全螢幕深色 IDE）與淺色兩種背景下逐檔實測補光濃度（55%–70% 區間），判準——深色背景下亮度明顯上提、淺色背景下毛玻璃仍可辨；定案值寫回 3.1 的 class。驗證：兩種背景截圖對比確認判準成立（操作前確認使用者未在使用螢幕）。 <!-- speclink-task:tsk_01KYVP535WZGVWRSA92P9Z4FGE -->

## 4. 驗證

- [x] 4.1 全套前端測試與建置：npm test -w apps/desktop 全綠、npm run build -w apps/desktop 通過（無 Rust 端改動、無 CLI 輸出影響——不涉 golden 與 --json 契約）。 <!-- speclink-task:tsk_01KYVP535WNGVSEWACVNMQM5N5 -->
- [x] 4.2 macOS 真實視窗驗證清單（GUI 改動不得只靠 jsdom；操作前確認使用者未在使用螢幕）：(1) 內容超過上限時 tab 條與動作區固定可見、僅中段捲動、捲軸浮動式不常駐；(2) 內容未超限時面板貼合、無多餘捲動與空白；(3) 深色背景下面板不明顯偏暗、淺色背景下毛玻璃可辨；(4) 動作區依序「開啟 Speclink」「專案設定」「設定」「結束」，點「專案設定」喚起主視窗並切至專案設定頁；(5) recovery 排列（復原卡）下頁首頁尾仍固定、復原卡在中段。 <!-- speclink-task:tsk_01KYVP535WFAT8SHTA4M2410PH -->
