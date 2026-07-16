## REMOVED Requirements

### Requirement: 系統匣樣式偏好
**Reason**: tray-copy-and-panel-mode 的實測裁決（2026-07-16）確定 macOS 固定採用面板樣式——偏好是兩樣式並存試驗期的 A/B 把手，裁決後移除；系統匣互動樣式改由平台決定（見 tray-status-menu「系統匣圖示與原生選單」）。
**Migration**: 無使用者動作：macOS 升級後系統匣直接為面板、非 macOS 維持原生選單；app 本機殘留的舊偏好鍵不再讀取（無害殘留）。面板建立失敗的單行錯誤改於設定頁本機設定簽以獨立警示行浮出。
