# Desktop 串接驗證

- 新增 RED 案例先確認：指定提案的基本掃描仍落裝置資料；含 canonical key 的進度錯用報告 key；前端 report 呼叫缺少原始 project/change。修正後提案報告、裝置工作識別與原始目標分別維持正確範圍。
- Desktop core 整包原為 350 passed / 1 failed；失敗由新增兩個平行測試撞到既有全行程一次一掃限制，測試加入共用鎖後 security:: 的 8 個定向案例全通過（含取消／未知工作、A/B 隔離、完整紀錄蓋章後與封存 dated 名稱查詢、沒有 checkout 的 Remote KB job 不連網）。產品的單一基本掃描限制維持。
- 新增檢查歷史、提案安全報告 Markdown／view、本機知識庫 job 與當次 Remote 來源核對。報告讀取／匯出／複製／定位都攜帶原 target/change，不以目前 UI 頁面猜範圍。
- 本機指定提案掃描存提案 checks；Remote 基本掃描維持本機草稿／快取，畫面明示不可視作遠端已發佈或驗收。
- Desktop adapter 3 tests、SecurityReviewPanel 4 tests 通過；新增「準備本機知識庫」只走獨立 KB 工作，不啟動 Agent／基本掃描／Remote prepare。
- Tauri cargo check 通過；Tauri 只處理 IPC、既有連線解析與系統開檔，資料判定沿 desktop-core / Host。
