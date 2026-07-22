## 1. 劇本基建

- [x] 1.1（design「決策 2：單一連續劇本、兩 server 常駐」）：apps/desktop/src-tauri/tests/common/mod.rs 擴充 helpers——雙 in-process server（獨立 tempdir sqlite、隨機埠、可個別停起）、setup 出帳號與 scope 的播種、真實 CLI binary 驅動（於指定資料夾以 env 指向 server 執行 speclink 命令並收 stdout）、device family 撤銷操作。驗證：helpers 各自的最小自測（起雙 server、CLI 對其一 list 成功）。 <!-- speclink-task:tsk_01KY178KD7BNWY8WCTVPSVHKFF -->
- [x] 1.2（design「決策 4：失敗現場慣例沿 phase2」；規格「失敗現場可讀且 CI 必跑」）：情境名前綴斷言巨集與失敗時傾印兩 server 輸出尾段＋connection 狀態的慣例落地。驗證：故意失敗一次確認輸出形狀後改回。 <!-- speclink-task:tsk_01KY178KD7XS2JZBS3YTJRTGYY -->

## 2. 六幕劇本（規格「五情境單一連續劇本」）

- [x] 2.1 第一、二幕（規格「五情境單一連續劇本」；design「決策 1：desktop 資料面為劇本主角、CLI 為 RD 分身」）：雙 server 起機與 setup → PM 無 checkout——登入（in-memory credential store）、handshake 開 session、清單/文件讀取、任務勾選寫入、reader 帳號的 capability 停用斷言。apps/desktop/src-tauri/tests/phase3_chain.rs 內全綠。 <!-- speclink-task:tsk_01KY178KD7J61Z2ES6RZW4HH0M -->
- [x] 2.2 第三幕 RD 有 checkout：tempdir git repo 寫 marker → CLI 於該資料夾 remote 模式 new change＋寫 artifacts＋task done → PM 的 session 資料面數秒內經失效提示重查得到更新（覆蓋規格情境「RD 寫入即時反映至 PM 資料面」）。全綠。 <!-- speclink-task:tsk_01KY178KD7Z4YVFK5A0JME5KSM -->
- [x] 2.3（規格「五情境單一連續劇本」之「多 tab 依 locator scope 共用與隔離事件流」；design「決策 2：單一連續劇本、兩 server 常駐」）第四、五幕：多 server——第二連線開 session、憑證逐 origin 隔離斷言、事件互不串流；多 tab——同 server 的 PM／RD 兩 scope 各維持一條 SSE（總數 2），同 RD locator 的第二個 session 只增加參照計數且不另開流，任一 scope 寫入只分發對應 locator。驗證：apps/desktop/src-tauri/tests/phase3_chain.rs 全綠。 <!-- speclink-task:tsk_01KY178KD77KZ9QH3QFD83HVS1 -->
- [x] 2.4 第六幕失聯恢復：殺第一 server → offline 廣播與寫入即拒、第二 server 全程正常（覆蓋規格情境「失聯幕不波及另一 server」）→ 期間以 CLI 對第二 server 寫入 → 重啟第一 server 自動收斂含期間變更 → 撤 device family → needs-reauth → 重登入原地恢復。全綠。 <!-- speclink-task:tsk_01KY178KD7MSYJMGA6TP1G8HPA -->

## 3. gate 錨定與 CI

- [x] 3.1（規格「gate 條目逐一錨定於劇本斷言」；design「決策 3：gate 條目逐一錨定」）：六條 gate 的命名斷言補齊——三形態 session 並存（local＋remote spec-only＋remote+checkout 同時存在）、locator key 身分、capability 停用、registry 與分頁持久化序列化掃描無 credential、Polling＋ETag 收斂、stale 只讀且恢復後 server 查無離線寫入。全綠。 <!-- speclink-task:tsk_01KY178KD7YHTYMNG3PM7Q3Q71 -->
- [ ] 3.2 .github/workflows/ci.yml 劇本獨立 job（與單元測試分開）、失敗上傳 server 輸出 artifact。驗證：CI 綠。 <!-- speclink-task:tsk_01KY178KD7PY3Q45C7CF3AYVE7 -->

## 4. 驗收

- [x] 4.1（規格「GUI 面以手動鐵律清單對照」；操作前確認使用者未在使用螢幕）：remote-dev-harness 手動清單五情境走查——PM spec-only 全流程、RD checkout 綁定與 CLI 互通即時反映、雙 server 分頁並存與狀態圖示、多 tab 切換、殺 server／重啟／撤 family 的橫幅與復活；security find-generic-password 確認 Keychain entry、開發者工具檢視 localStorage 無 credential。逐項記錄結果。 <!-- speclink-task:tsk_01KY178KD7YPTTNX6W77XQNEGA -->
- [x] 4.2 回歸：cargo test --workspace（含新劇本）、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠；與 local-remote-migration 平行時 tests/common/mod.rs 共檔依提交衛生合流。 <!-- speclink-task:tsk_01KY178KD7Y82MS830BNF0EJA0 -->
