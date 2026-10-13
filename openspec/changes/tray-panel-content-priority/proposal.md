## Why

在 desktop 追蹤 SDD 流程的開發者（透過 AI 代理跑 propose／apply 的開發者、PO、PM）會從 macOS 系統匣面板一眼確認「哪些變更在提案中、進行中、已就緒」，再決定要不要開工或封存。討論一多（例如 wadpilot 專案有 8 則討論），討論卡排在最上面、每列兩行（slug＋說明），佔掉可捲中段約八成；固定頁尾的四列動作區再佔約 130px。結果是打開面板只看得到「提案中」的標題，三個生命週期分區都要往下捲。另外，tray-panel-restyle 把面板補光層由約六成不透明改成約九成、卡片由近乎透明改成不透明白卡，macOS 毛玻璃只剩一成透出，使用者回報「毛玻璃好像消失了」。來源討論 `tray-panel-content-priority` 已定案。

## What Changes

- 面板可捲中段的順序改為：提案中 → 進行中 → 已就緒 → 討論。討論卡（含「還有 N 個…」溢出列與「已轉出」收合列）整張移到最後；三個階段維持看板左到右的順序，與原生選單「生命週期在前、討論在後」一致。
- 動作區塊卡由四列改為單列：左為「開啟 Speclink」文字鈕，右為「專案設定」「設定」「結束」三顆圖示鈕（帶語意化 label，指標停留時顯示名稱），「結束」與前兩顆拉開間距防誤按。四個動作的行為不變；中段約多出 90px。
- 新增內容卡收合：提案中、進行中、已就緒、討論四張卡各自可收合，整條分區標題為觸發區、計數之後有方向箭頭；收合後只剩「圖示＋名稱＋計數」一行，計數照常更新；預設全部展開；收合狀態存 app 本機、所有專案共用一份；收合中的卡有新項目時不自動展開。
- 找回毛玻璃：面板根層的補光層由約九成不透明調回約六成，白底卡片維持不透明；毛玻璃自面板外框與卡片之間的間距透出，卡內文字的亮度與對比不變。Rust 端的 vibrancy 材質（HudWindow）本來就正常，不動。
- 收合箭頭統一規則「收合時一律 ›、展開時箭頭指向展開出來的內容」：系統匣面板與看板討論欄底部的「已轉出 N」收合列，展開細列在該列上方，展開時箭頭改為朝上（現為朝下、與內容方向相反），收合時維持 ›；內容卡標題的收合箭頭（收合 ›、展開朝下，內容在標題下方）同一條規則。
- 專案 tab 條在專案很多時：「加入專案」固定在 tab 條右端、改為只有「＋」圖示的方塊（滑過顯示名稱），不隨 tab 捲走；有 tab 未顯示的那一側邊緣淡出提示可捲動（捲軸維持隱藏）；面板開啟與切換專案時自動捲到作用中的 tab。現況是 3 個專案加「加入專案」就塞滿 tab 條，第 4 個專案起「加入專案」被擠出、第 5 個起專案也被擠出，且看不出還有東西。
- 不變：高度上限 640、每區前 5 筆的溢出門檻、討論列兩行、非 macOS 原生選單。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 「面板樣式（macOS）」需求的區塊順序改為生命週期在前、動作區塊卡改為單列；新增「面板內容卡收合」需求；「討論列表」的已轉出收合列箭頭方向改為展開朝上。
- `desktop-app`: 「討論於看板第 0 欄兩級呈現」的欄底「已轉出 N」收合列箭頭方向改為展開朝上。

## Impact

- 目標使用者與情境：用 desktop 看板與系統匣追蹤變更生命週期的開發者／PO／PM，對應 propose 之後到 archive 之前「看狀態、決定下一個動詞」的時刻。
- Affected specs: `tray-status-menu`, `desktop-app`
- 影響的 app：`apps/desktop` 的系統匣面板前端，以及 `packages/ui` 的看板討論欄（只改「已轉出」收合列的箭頭旋轉方向）；speclink-core、speclink-cli、speclink-host 與 server 不動。
- 相容性影響：沒有 CLI 指令、`--json` 輸出、設定欄位或技能的變更。面板視窗的 localStorage 新增一個鍵記收合狀態；舊版本沒有這個鍵，首次開啟即為全部展開。
- Affected code:
  - New: apps/desktop/src/panel/sectionCollapse.ts, apps/desktop/src/panel/tabStrip.ts
  - Modified: apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/__tests__/trayPanel.test.tsx, packages/ui/src/components/DiscussionColumn.tsx, packages/ui/src/__tests__/discussionColumn.test.tsx
  - Removed: (none)
