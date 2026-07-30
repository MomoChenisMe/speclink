## 1. Protocol wire 欄位

- [x] 1.1 撰寫 DTO 欄位測試：建立討論請求可帶選填 slug、變更清單摘要可帶選填 startedAt——序列化往返斷言 camelCase 欄位名（slug、startedAt）、None 不序列化、缺席時反序列化為預設（舊 payload 相容）。位置：crates/speclink-protocol/src/command.rs 與 crates/speclink-protocol/src/query.rs 的 #[cfg(test)] 模組。驗證：cargo test -p speclink-protocol 新測試紅燈（欄位尚不存在，編譯失敗即紅） <!-- speclink-task:tsk_01KYS6ADN643J9GSD12S6PN7Y5 -->
- [x] 1.2 實作兩 DTO 選填欄位（serde default、skip_serializing_if 慣例與同檔既有選填欄位一致）。驗證：cargo test -p speclink-protocol 全綠，既有 fixture 測試不變 <!-- speclink-task:tsk_01KYS6ADN6MNFVGG4KXM9GN2GJ -->

## 2. Server 端點

- [x] 2.1 撰寫討論寫入端點測試（spec 需求「討論寫入動詞端點補齊」）：POST /discussions 帶合法 slug 建檔（回應 slug 為覆寫值、topic 原文）、非法 slug 回語義化錯誤且 store 不落檔；DELETE /discussions/{slug} 對 0 輪討論刪除成功且 revision 前進、有輪無 force 拒絕且記錄保留、force=true 刪除成功、reader role 收 403；POST /discussions/{slug}/link 與 /seal 帶 change 名稱後 meta 鏈與 promoted 狀態成立、不存在者 404。位置：crates/speclink-server/tests/discussion_routes.rs。驗證：cargo test -p speclink-server discussion_routes 新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN6TT4Q8TWBPA4K4PDP -->
- [x] 2.2 實作討論寫入端點：create_discussion 轉傳 slug 至引擎（不再硬編 None）、新增 DELETE /discussions/{slug}（force query、editor 限定比照 change 刪除）、POST /discussions/{slug}/link 與 /seal 路由直通引擎命令。位置：crates/speclink-server/src/app.rs、crates/speclink-server/src/routes.rs。驗證：2.1 全綠 <!-- speclink-task:tsk_01KYS6ADN6ZP0N3APV7ZSA0FQ5 -->
- [x] 2.3 撰寫開工標記端點與清單欄位測試（spec 需求「變更開工標記端點」）：POST /changes/{name}/in-progress 首蓋後 meta 含 started_at 與呼叫者身分的 started_by、事件發布、revision 前進；重複與未知 change 皆 HTTP 200、零寫入、零事件、revision 不前進；GET /changes 清單中已開工 change 帶 startedAt（camelCase）、未開工不帶。位置：crates/speclink-server/tests/verb_api.rs、crates/speclink-server/tests/query_routes.rs。驗證：cargo test -p speclink-server 新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN6GQBA1TAX7ET0AQTV -->
- [x] 2.4 實作開工標記端點與清單欄位：新增 POST /changes/{name}/in-progress 路由直通引擎 InProgressAdd（引擎 outcome 未蓋章時不發事件）、變更清單摘要組裝時自 meta 帶出 startedAt。位置：crates/speclink-server/src/app.rs、crates/speclink-server/src/routes.rs。驗證：2.3 全綠 <!-- speclink-task:tsk_01KYS6ADN6X76CE2NKTNY3QN7P -->

## 3. Remote client

- [x] 3.1 撰寫 client 方法測試：new_discussion 攜帶 slug、討論 discard（force 參數成為 query）、link、seal、in-progress 五個方法對 mock server 發出正確路徑與 payload、錯誤回應映射為既有 RemoteError 語意（404 不 panic）。位置：crates/speclink-remote/tests/typed_client.rs。驗證：cargo test -p speclink-remote 新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN6XA42K94C5S5ENDWX -->
- [x] 3.2 實作 client 方法：new_discussion 增 slug 參數、新增 discard_discussion、link_discussion、seal_discussion、in_progress_add。位置：crates/speclink-remote/src/client.rs。驗證：3.1 全綠 <!-- speclink-task:tsk_01KYS6ADN6HQVJRDZVRQEBREHE -->

## 4. CLI remote 分支

- [x] 4.1 撰寫 discuss 四動詞 CLI 整合測試（雙沙盒；spec 需求「討論動詞於 remote 模式與本機同語意」）：remote 模式 speclink discuss new 帶 --slug 成功建檔且 stdout 與 --json 形狀與 fs 模式一致（--json 斷言 slug 欄位 camelCase）、非法 slug 非零 exit 且 stderr 說明、server 不落檔；discuss discard 的輪數 guard 與 --force 語意同 fs；discuss link 後 show --json 的 from_discussion 鏈成立；discuss seal 後 discuss list --json 反映 promoted。位置：crates/speclink-cli/tests/discuss_slug.rs、crates/speclink-cli/tests/remote_write_path.rs。驗證：cargo test -p speclink-cli 新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN698YB2MK3SG0AKCDD -->
- [x] 4.2 實作 discuss 四動詞 remote 分支：移除 --slug、discard、link、seal 四處 bail，接 client 方法，輸出走 fs 模式同一渲染路徑。位置：crates/speclink-cli/src/remote_commands.rs。驗證：4.1 全綠 <!-- speclink-task:tsk_01KYS6ADN6ZPCD3R6GW8AY7VEW -->
- [x] 4.3 撰寫 show 對照測試（spec 需求「動詞契約的涵蓋面與 payload 形狀」）：同一份 change 與規格內容分別置於 fs 沙盒與 remote server，speclink show 的人眼輸出與 --json 逐欄位一致（--json 斷言欄位 camelCase 同名）；remote 模式在本機 openspec 目錄缺席時仍成功（證明未讀本機 store）；不存在的 item 回語義化錯誤非零 exit。位置：crates/speclink-cli/tests/remote_verb_parity.rs。驗證：新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN6J9Q6WF0NPBDEJGMN -->
- [x] 4.4 實作 show 的 remote 分支：cmd_show 加 remote_ctx 分支，以既有讀 API（get_change、get_artifact、spec_document、兩份清單）組裝出與 fs 模式同形輸出，item 與 --type 判別序對齊引擎現行為。位置：crates/speclink-cli/src/commands.rs、crates/speclink-cli/src/remote_commands.rs。驗證：4.3 全綠 <!-- speclink-task:tsk_01KYS6ADN69HB9AHDXKPQSMKHB -->
- [x] 4.5 撰寫 in-progress 與 demo 的 remote 測試（spec 需求「in-progress 標記經 remote 通道寫入 server meta」）：remote 模式 speclink in-progress add 對存在 change 靜默 exit 0 且 server 端 meta 含 started_at 與 started_by、對不存在 change 靜默 exit 0 且 server 零寫入、fs 模式輸出逐位元不變；remote 模式 speclink demo 非零 exit 且 stderr 說明僅限本機、本機與 server 均無新 change。位置：crates/speclink-cli/tests/remote_write_path.rs。驗證：新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN6EBY7CGN3VNB1RJNB -->
- [x] 4.6 實作 in-progress 與 demo 的 remote 分支：cmd_in_progress 加 remote 路由至 client、cmd_demo 於 remote_ctx 存在時一行 bail。位置：crates/speclink-cli/src/commands.rs。驗證：4.5 全綠；cargo test -p speclink-cli 全綠（既有迴歸含 fs 模式輸出凍結） <!-- speclink-task:tsk_01KYS6ADN6F44VRCSQVAQX5FM9 -->

## 5. Desktop 消費端

- [x] 5.1 撰寫欄位推導與映射測試（spec 需求「看板欄位由生命週期標記驅動」）：src-tauri 的變更卡欄位推導對「帶 startedAt 且完成數 0」判為進行中（系統匣分區同步）；前端 remote 資料源將 wire 的 startedAt 映射進 ChangeItem.startedAt，changeStage 對該卡回 in-progress。位置：apps/desktop/src-tauri/tests/remote_data.rs、apps/desktop/src/__tests__/remoteDataSource.test.ts。驗證：cargo test -p speclink-desktop 與 npm test -w apps/desktop 新案例紅燈 <!-- speclink-task:tsk_01KYS6ADN6HM7CTGXHNYKRTWYW -->
- [x] 5.2 實作消費端：src-tauri remote 模組的 change_stage 增 started_at 判定並於 payload 映射補 startedAt 欄位；前端 remoteDataSource 映射補同名欄位（stage.ts 推導本身不動）。位置：apps/desktop/src-tauri/src/remote.rs、apps/desktop/src/adapter/remoteDataSource.ts。驗證：5.1 全綠 <!-- speclink-task:tsk_01KYS6ADN6M0EKYKSBZZ17KN20 -->

## 6. 文件與收尾

- [x] 6.1 更新正典契約文件：docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 增列 POST /discussions 的 slug 欄位、DELETE /discussions/{slug}、POST /discussions/{slug}/link、POST /discussions/{slug}/seal、POST /changes/{name}/in-progress 端點與 payload、錯誤形狀，及變更清單摘要的 startedAt 欄位。驗證：內容審視——五個端點與兩個欄位皆有條目，與 delta spec 場景一致 <!-- speclink-task:tsk_01KYS6ADN6JK0FFFEB21YK0451 -->
- [x] 6.2 全套驗證與 fs 迴歸確認：workspace cargo test 全綠、npm test -w packages/ui 與 npm test -w apps/desktop 全綠；確認 crates/speclink-core/tests/golden 未受影響（本變更不動引擎與 fs 模式輸出，golden 不重生）。驗證：上述指令全部通過且 git status 無 golden diff <!-- speclink-task:tsk_01KYS6ADN6C57NB8HJYQ06NY18 -->
