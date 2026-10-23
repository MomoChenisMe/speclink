# 設定與歷史文件樹

- 設定 RED：4 新案例因缺 checkbox / 保留設定失敗，原 104 案例通過。GREEN 後 projectSettingsView + workspace 共 193 tests 通過；驗收預設 false、三項保留 true、停用保值、取消零寫入、reader 唯讀、CAS 對照／重送及僅送已修改葉節點均有覆蓋。
- Rust 設定合併測試通過：security 與 check_records 先全部解析與驗證再寫入；保留未知鍵及註解，錯型別不改原檔。Tauri cargo check 及 TypeScript noEmit 通過。
- 歷史文件樹 RED：3 新案例缺節點，原 142 案例通過。GREEN 後 active / archived 共 146 tests 通過，另加鍵盤 Enter 選取安全紀錄，changeDetailPage 定向 107 tests 通過。
- 同一提案離線／查詢失敗保留先前已載入的唯讀紀錄並顯示錯誤；更換 adapter / 提案立即隔離，遲到資料不能覆蓋另一專案。401 顯示重新登入訊息。Markdown 沿不執行 HTML 的既有 renderer。
- Desktop adapter + securityReview 共 5 tests 通過；三類清單使用同一 checks/index 的 Host 查詢，讀取只帶提案、類別與 recordId，不傳任意顯示路徑。Remote reader 的安全歷史 HTTP 測試通過，無 checkout、零寫入。
- 完整紀錄只由 index 列出；未保存或索引缺失不重跑檢查。畫面與設定均區分完整報告與必要安全驗收摘要。
