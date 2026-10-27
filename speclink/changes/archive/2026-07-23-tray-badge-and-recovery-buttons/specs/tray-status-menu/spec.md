## REMOVED Requirements

### Requirement: macOS 進行中數文字徽章
**Reason**: 數字徽章的更新在 macOS（Tahoe）上反覆凍結殘留——實機確診出孤兒 tray icon 凍住舊值（webview 重建時前一個 context 的 tray 無人清理），且以固定 id 清理孤兒後使用者實機仍觀察到數字不隨 workspace 切換更新。使用者裁定：系統匣不再顯示任何數字文字，錯誤類別整個移除。
**Migration**: 系統匣圖示 SHALL NOT 帶任何標題文字；進行中變更數由面板「進行中」分區計數（與看板同源）與看板欄計數承載。tray 仍以固定 id 建立並於初始化前移除同 id 既有 icon，防止孤兒 icon 殘留。
