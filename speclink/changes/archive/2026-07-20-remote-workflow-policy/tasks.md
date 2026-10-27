## 1. identity role 模型（TDD）

- [x] 1.1 紅（規格「membership role 最小模型」；design「決策 3：role 最小模型與傳播」）：identity 測試——membership 設定帶 role 且 audit 記錄含新 role 值、invitation 建立的 membership 固定 editor、sqlite 既有資料庫升版後全部 membership 為 editor（migration 測試）。cargo test -p speclink-server 確認新案例全紅。 <!-- speclink-task:tsk_01KXZ63BRX2PD3EFSJSF0EHQ8J -->
- [x] 1.2 綠：crates/speclink-server/src/identity.rs 的 trait 與 memory 實作、crates/speclink-server/src/identity_sqlite.rs 的 schema 升版（memberships 加 role TEXT NOT NULL DEFAULT 'editor'，依既有 migration 機制）、crates/speclink-server/src/admin.rs 的 membership 表單加 role 選擇。1.1 全綠。 <!-- speclink-task:tsk_01KXZ63BRXF0Q7RNS73NRSWA93 -->

## 2. config 讀寫端點（TDD）

- [x] 2.1 紅（規格「config 內容與 revision 隨讀取下發」「policy 寫入一律 CAS 且經引擎驗證」「role 經 capabilities 傳播且 server 為最終防線」；design「決策 1：config 讀寫走文件原文＋scope revision」「決策 2：PUT 的兩道防線與錯誤語意」）：新增 crates/speclink-server/tests/policy_write.rs——GET 回 content＋revision 且與 ETag 同值、reader 亦可讀；PUT 成功後新內容新 revision；過期 expectedRevision 回 revision_conflict 無副作用；壞 YAML 回 invalid_config 不落盤；reader 直呼 403 且文件未變；缺 expectedRevision 拒收；binding 回應 policy_write 隨 role 翻轉。確認全紅。 <!-- speclink-task:tsk_01KXZ63BRX2KT8ZB0WGVC1RP3Y -->
- [x] 2.2 綠：crates/speclink-server/src/routes.rs 的 config 讀取擴充與 PUT handler（順序：role→引擎驗證→CAS）、crates/speclink-server/src/app.rs 掛路由、crates/speclink-server/src/auth.rs 的 Capabilities 加 policy_write、crates/speclink-protocol/src/query.rs 與 crates/speclink-protocol/src/binding.rs 的 DTO 欄位。2.1 全綠。 <!-- speclink-task:tsk_01KXZ63BRXXT3HMERAVBZ0XSX8 -->
- [x] 2.3 crates/speclink-remote/src/client.rs 加 config 全量讀取與 put_config 方法，對 in-process server 補方法級斷言。cargo test -p speclink-remote 全綠。 <!-- speclink-task:tsk_01KXZ63BRXJWJ6CC7RV35XWX01 -->

## 3. desktop 文字層 seam 與遠端編輯器

- [x] 3.1（design「決策 4：desktop-core settings 的文字層 seam」）：apps/desktop/core/src/settings.rs 抽出 from-text 解析與 targeted-key 文字改寫函式，root 路徑函式改為薄包裝；新增文字層直測（各欄位解析、未觸及鍵原樣保留、設回預設移除鍵——與既有 root 路徑測試同斷言）。既有 settings 測試全綠（本地行為凍結）。 <!-- speclink-task:tsk_01KXZ63BRX302FDATBQZ23VF88 -->
- [x] 3.2（規格 desktop-config「設定頁圖形化讀寫兩層設定」修訂的遠端 Workflow 簽；design「決策 5：remote 專案設定頁＝單一 Workflow 簽」）：remote settings provider 由 stub 改實作（apps/desktop/src-tauri/src/remote.rs 以 /config content 過文字層 seam、寫回組裝全文帶 expectedRevision；apps/desktop/src/adapter/workspace.ts 與 apps/desktop/src/session.ts 接線）；apps/desktop/src/views/ProjectSettingsView.tsx 的 remote 分支改單一 Workflow 簽三卡、簽首等寬字顯示 revision、reader 唯讀存檔停用附繁中說明。apps/desktop/src/__tests__/projectSettingsView.test.tsx 覆蓋 editor 可存、reader 唯讀、revision 顯示。全綠。 <!-- speclink-task:tsk_01KXZ63BRXB0CSGRABGXRMKQMT -->
- [x] 3.3（design「決策 6：409 對照流程——informed resubmit，非 force overwrite」；規格 desktop-config 修訂的衝突情境）：儲存收到 revision_conflict 時輸入原樣保留、衝突面板逐欄位對照（重新讀取最新 content/revision）、兩出口（以 server 版重載｜以最新 revision 重新提交）、對照期間再前進即再次 409 遞迴成立；無任何未經對照的覆寫路徑（斷言不存在該 UI）。vitest 覆蓋輸入保留、對照內容、兩出口與遞迴 409。全綠。 <!-- speclink-task:tsk_01KXZ63BRX14CE4PQZKBHED8PA -->

## 4. 驗收

- [x] 4.1 GUI 鐵律雙帳號手動全鏈（design Implementation Contract；操作前確認使用者未在使用螢幕）：npm run dev → admin 於 /admin 設一 editor 一 reader → editor 於 remote 分頁 Workflow 簽改政策存檔、以 CLI 對同 scope 取 instructions 確認輸出反映新政策 → 兩視窗並發編輯實走 409 對照與重新提交 → reader 帳號見唯讀與角色說明 → 本地分頁專案設定頁行為與前一版一致。 <!-- speclink-task:tsk_01KXZ63BRXWSN8WY379RPDQYFD -->
- [x] 4.2 回歸：cargo test --workspace、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠（重建前關閉執行中 exe）；CLI 輸出凍結不受影響（本刀不動 CLI 命令面）。 <!-- speclink-task:tsk_01KXZ63BRX5NMAXNAD0Z95V6GV -->
