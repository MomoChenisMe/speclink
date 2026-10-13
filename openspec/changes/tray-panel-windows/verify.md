# Verify — tray-panel-windows

## Round 1

**Phase**: discovery
**Patch**: sha256:3a56ee0d8c6c4fa89e2335258e2d7741a82cecf7ea7ade1690f31784af981d3f
**Scope**: apps/desktop/core/src/lib.rs, apps/desktop/core/src/tray_panel.rs, apps/desktop/src-tauri/Cargo.toml, apps/desktop/src-tauri/capabilities/tray-panel.json, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/panel.rs, apps/desktop/src/__tests__/platform.test.ts, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/__tests__/themeBoot.test.ts, apps/desktop/src/__tests__/tray.test.ts, apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/panel/main.tsx, apps/desktop/src/store.ts, apps/desktop/src/tray.ts
- [WARNING] apps/desktop/src-tauri/src/panel.rs — Correctness: 工藝站已記的「同步 command 建窗死結」，會讓「Windows 主鍵點擊圖示於上方彈出面板」和「Windows 面板建立失敗退回原生選單」在第一次開啟時實際上不成立：程式卡住、不會回錯，所以 panelFallback 也不會觸發。修好後要用 4.2 在真機再驗一次。
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Correctness: 「工作列在頂端時面板改於下方」靠的是 positioner 2.3.3 的 TrayCenter 自動翻轉（套件原始碼 ext.rs:290-293）。它的判斷是 `y < 0`，看的是整個虛擬桌面的絕對座標，不是圖示所在螢幕的邊界。副螢幕放在主螢幕上方或下方時會選錯邊，跟 D3「上方放不下改放下方」的意思有落差。建議 place() 改用 work area 上緣判斷，或在 4.2 明列這個多螢幕情境。
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Correctness: delta spec 的「Windows 次要鍵點擊圖示顯示原生選單」寫「面板未因此開啟或收合」。但面板開著時右鍵點圖示，面板會先失焦，Windows 分支的 Focused(false) 一定會把它收起來，這跟同一條需求的「失焦自動收合」互相衝突。建議把 scenario 的 WHEN 限定為「面板未開啟」，或把 THEN 改成「右鍵本身不開閉面板」。
- [SUGGESTION] apps/desktop/core/src/tray_panel.rs — Correctness: Example「水平位置限制」的輸入是圖示中心 x（1000／1860／100），測試用的卻是換算後的左緣（840／1700／-60）。結果等價，但「中心換算成左緣」這一步在 positioner 裡，沒有測試涵蓋。建議在測試名稱或註解標出對應的 Example 列。
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Correctness: 以下 scenario 都沒有自動測試：上方彈出貼齊、頂端改下方、高度改變後仍貼齊、失焦收合、Windows 11 系統圓角與陰影、關閉不閃現的事件順序。Windows 分支本機也編不了，只能靠 [M] 4.1／4.2，所以降為 SUGGESTION。工作樹裡 4.1／4.2 是 session 外打的勾，封存前要確認真的跑過。

## Round 2

**Phase**: validation
**Patch**: sha256:4f1822e6d3728b8e48d950fef3c267000a8008925676855c3f04c2aa687ea8eb
**Scope**: apps/desktop/core/src/tray_panel.rs, apps/desktop/src-tauri/Cargo.toml, apps/desktop/src-tauri/capabilities/tray-panel-macos.json, apps/desktop/src-tauri/capabilities/tray-panel.json, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/panel.rs, apps/desktop/src/__tests__/tray.test.ts, apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/tray.ts
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Correctness: 以下 scenario 都沒有自動測試：上方彈出貼齊、頂端改下方、高度改變後仍貼齊、失焦收合、Windows 11 系統圓角與陰影、關閉不閃現的事件順序。Windows 分支本機也編不了，只能靠 [M] 4.1／4.2，所以降為 SUGGESTION。工作樹裡 4.1／4.2 是 session 外打的勾，封存前要確認真的跑過。
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Coherence: 問題實際在 openspec/changes/tray-panel-windows/tasks.md 的 1.2／2.1／2.2。這三條仍寫 positioner 擴及 Windows、用 TrayCenter 加水平限制定位、匯出水平限制函式與門檻常數、能力宣告 permissions 不變，都跟修正後的實作矛盾。建議改寫這三條描述，行尾的 ID 保留不動。

## Round 3

**Phase**: validation
**Patch**: sha256:105b2dfbaf486d517523afbfa592b8ca6f61238a6271277c751ec79ce18cd14b
**Scope**: apps/desktop/src-tauri/src/panel.rs
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Correctness: 以下 scenario 都沒有自動測試：上方彈出貼齊、頂端改下方、高度改變後仍貼齊、失焦收合、Windows 11 系統圓角與陰影、關閉不閃現的事件順序。Windows 分支本機也編不了，只能靠 [M] 4.1／4.2，所以降為 SUGGESTION。工作樹裡 4.1／4.2 是 session 外打的勾，封存前要確認真的跑過。
