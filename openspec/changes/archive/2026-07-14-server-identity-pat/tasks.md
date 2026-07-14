## 1. identity 儲存

- [x] 1.1 【紅】針對「identity 儲存獨立且版本守門」與生命週期行為寫測試：users/memberships/invitations/PATs/sessions 的建立與查驗、一次性邀請耗用、到期判定、撤銷時戳、last-used 更新；陌生 SQLite 檔拒開且位元不變；憑證欄位落庫皆為 hash。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXFHDCHT5DMGV5B3XA28C8KH -->
- [x] 1.2 【綠】實作 identity trait（crates/speclink-server/src/identity.rs）與 SQLite 實作（crates/speclink-server/src/identity_sqlite.rs，meta 表 schema version 1、守門原則沿用 sqlite-team-store）及 in-memory 測試變體；argon2 依賴僅進 speclink-server。1.1 全綠。 <!-- speclink-task:tsk_01KXFHDCHTXN7K5A4QEK9RJG22 -->

## 2. invite 子命令

- [x] 2.1 【紅→綠】針對「邀請一次性且到期失效」的建立側寫測試並實作：speclink-server binary 增 invite 子命令（--email、--display、--project 可重複、--admin、--expires-in-days 預設 7），以 --config 定位 identity 資料庫建立邀請並於 stdout 輸出一次性 URL；對已有 active user 或未過期邀請的 email 以非零 exit code 拒絕。既有 run 行為（--config --addr 啟動）不變。 <!-- speclink-task:tsk_01KXFHDCHT29RC7RQ6A338W4FF -->

## 3. Web 入口

- [x] 3.1 【紅→綠】invite 接受頁：GET 有效 token 呈現設密碼表單、POST 原子建立 active user（含指派 memberships）並耗用邀請；已用/過期/未知 token 回同一「邀請無效」頁（404），不區分原因。測試涵蓋「邀請走完即建立帳號」與「過期邀請不可用」情境。 <!-- speclink-task:tsk_01KXFHDCHTYK6JDJNBFXAARR1K -->
- [x] 3.2 【紅→綠】/login、/logout 與 session：argon2 驗證、session cookie（HttpOnly、Secure、SameSite=Strict）、變更型 POST 驗同源不符回 403、登入失敗統一訊息（不存在 email 與錯密碼回應逐位元相同）、登出撤銷 server 端 session 記錄且舊 cookie 視同未登入、未登入訪問 /account 導向 /login。 <!-- speclink-task:tsk_01KXFHDCHTYG7ZZ4Z2ZEP7BTAQ -->
- [x] 3.3 【紅→綠】/account 與 PAT 自助：頁面列 sessions 與 PAT（prefix、名稱、到期、last-used）；POST 建立 PAT（名稱、到期）回應頁顯示 spk_pat_ 明文恰一次，落庫僅 prefix+hash+metadata；POST 撤銷即時生效。測試涵蓋「明文只出現一次」與「撤銷即時生效」情境。 <!-- speclink-task:tsk_01KXFHDCHTMK7VDTC732H15FZG -->

## 4. bearer 接線與組態切換

- [x] 4.1 【紅】針對「bearer 驗證逐請求生效且分類明確」寫路由測試:identity 儲存無此 token 回 401；已撤銷/過期/user suspended 回 401 且與未知 token 回應不區分原因；有效 PAT 但非該 project member 回 403；有效 PAT 且具 membership 完成 /binding 且 actor 正確；成功請求後 last-used 更新。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXFHDCHTGVGAYF91M6BK77QC -->
- [x] 4.2 【綠】auth.rs 改查 identity 儲存（SHA-256 查表→撤銷/到期/active/membership 逐項檢查，無快取），4.1 全綠；config.rs 移除 tokens 段、新增 identity 段（sqlite 路徑或 memory），殘留 tokens 段啟動失敗且原因指出已由 identity 儲存取代，identity 段形狀不合啟動失敗。 <!-- speclink-task:tsk_01KXFHDCHTKTABZKM3QG4DEN0C -->
- [x] 4.3 遷移既有測試資產：tests/common 播種 helper 改為經 identity trait 建 user、membership 與 PAT；binding.rs、query/command/discussion 路由測試與 startup.rs 的組態改用 identity 段。驗收：cargo test -p speclink-server 既有測試全綠。 <!-- speclink-task:tsk_01KXFHDCHTJXSQD9751SK4PKFG -->

## 5. 端到端與回歸

- [x] 5.1 【紅→綠】e2e 全流程：啟動 tempdir SQLite（store＋identity）組態的真 server，以 invite 子命令建邀請→HTTP 走完接受頁設密碼→登入→建立 PAT→以該 PAT 明文配置真實 CLI 執行 remote 動詞流程（沿用既有 e2e 情境）→撤銷 PAT 後同一 CLI 呼叫得到 401 對映的現行錯誤訊息。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXFHDCHTBBSJW99CZSEA3QWM -->
- [x] 5.2 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff。驗收：全數通過。 <!-- speclink-task:tsk_01KXFHDCHTZWXWK9FH3NKW9ZY2 -->
