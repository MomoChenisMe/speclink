## 1. protocol 與 remote client（D3）

- [ ] 1.1 紅燈→綠燈：「plan 回應 payload」與「依賴寫入請求與回應」——在 crates/protocol/speclink-protocol/src/query.rs 新增 PlanResponse、PlanWave、PlanChangeEntry、PlanOverlap、PlanSkipped，在 command.rs 新增 SetDependsRequest、SetDependsResponse（camelCase、陣列 `#[serde(default)]`），先寫單元測試斷言規格的兩段 JSON 反序列化與缺 overlaps 鍵容忍、請求序列化為 `{"on":[…],"remove":false}`，再實作型別。驗證：`cargo test -p speclink-protocol` 全綠。 <!-- speclink-task:tsk_01M2J2FCGVRWYX08RVCHBF9G4J -->
- [ ] 1.2 紅燈→綠燈：crates/protocol/speclink-remote/src/client.rs 新增 `plan()`（GET /plan）與 `set_depends(name, on, remove)`（POST /changes/{name}/depends）；convert.rs 新增 `plan_report(PlanResponse) -> speclink_core::plan::PlanReport` 往返測試；error reason registry 加入 depends-self、depends-target-invalid、dependency-cycle 並測試 409 可機器判別。驗證：`cargo test -p speclink-remote` 全綠。 <!-- speclink-task:tsk_01M2J2FCGV598NN6MMGPMVK48G -->

## 2. server 端點（D1、D2、D8）

- [ ] 2.1 紅燈：新增 crates/host/speclink-server/tests/plan_api.rs 覆蓋「plan 唯讀衍生查詢端點」四個 scenario（reader 可讀且附 ETag、rank 來自 board resource、壞 board 內容視為空圖、成環 409 dependency-cycle）與「變更依賴寫入端點」四個 scenario（editor 200 與 revision 前進、reader 403、三種 409 零寫入、冪等 200 零事件）。驗證：`cargo test -p speclink-server --test plan_api` 先紅。 <!-- speclink-task:tsk_01M2J2FCGVBVQHY4XCWEG5CE21 -->
- [ ] 2.2 綠燈：第一刀的 `Command::Plan` 加 `ranks: Option<BTreeMap<String, String>>`（local 呼叫端傳 None）；app.rs 註冊 `GET /plan` 與 `POST /changes/{name}/depends`；routes.rs 的 plan handler 先以桌面同款寬鬆解析讀 board resource 的 changes 段（對應「board resource 為 scope 單文件且 server 不解析」的 plan 端點寬鬆讀取例外：壞內容視為空圖、不回寫）再經 `verb::run` 執行 `Command::Plan { ranks: Some(..) }`，depends handler 直通 `Command::ChangeDepends`、editor 限定、404／409 映射；speclink-host 的 bridge.rs 與 commit.rs 讓 `ChangeDependsChanged` 事件 commit 並 invalidate。驗證：2.1 全綠；`cargo test -p speclink-host` 全綠。 <!-- speclink-task:tsk_01M2J2FCGV57KP70WFBAZPNFAJ -->

## 3. CLI 兩動詞升為 Dual（D4）

- [ ] 3.1 紅燈：新增 crates/adapters/speclink-cli/tests/it/remote_plan.rs（沿 remote_verb_parity 的 capturing mock server）覆蓋「plan 與 change depends 的 remote 臂」：plan --json 鍵與 fs 模式一致且未讀本機 openspec/、成環 409 → 非零 exit 與 stderr `dependency cycle:`、change depends 發出正確 body 且 stdout 為 `✓ add-b depends on: add-a`；mode_dispatch.rs 移除兩動詞的 FsOnly 拒絕案例。驗證：`cargo test -p speclink-cli --test it remote_plan` 先紅。 <!-- speclink-task:tsk_01M2J2FCGV632Q4WRZGKVG491A -->
- [ ] 3.2 綠燈：main.rs 把 `plan` 與 `change` 改以 `dual` 宣告（對應「模式分岔的單點宣告」的 Dual 清單），verbs/plan.rs 加 remote 臂（client.plan → convert → 共用渲染；client.set_depends → 共用成功行），刪除兩句 FsOnly 拒絕常數。驗證：3.1 全綠；`cargo test -p speclink-cli --test it mode_dispatch` 與 `remote_verb_parity` 全綠。 <!-- speclink-task:tsk_01M2J2FCGVJT16S929HRFC54VD -->

## 4. 桌面 remote 路徑（D5、D6、D7）

- [ ] 4.1 紅燈：apps/desktop/src-tauri/tests/remote_data.rs 補「remote 變更清單的排程欄位」與「remote 排序 overlay 與本地語意同構」案例：list 以 plan 順序排列變更且每項含四欄、planError null；plan 409 時退回 overlay、四欄缺席、planError 為訊息；plan 404 時退回且 planError null；討論仍 overlay。驗證：`cargo test -p speclink-desktop --test remote_data` 先紅（需先備妥 sidecar 與 server-web dist）。 <!-- speclink-task:tsk_01M2J2FCGV2DYCPG7NZ9ADAS36 -->
- [ ] 4.2 綠燈：remote.rs 的 `list_changes` 於 list 與 board doc 後呼叫 `client.plan()`，以 `RemoteChangeItem`（flatten ChangeSummary 加四個 skip_serializing_if 欄位）與頂層 planError 回傳；404／連線失敗 → 退回且 planError null。驗證：4.1 全綠。 <!-- speclink-task:tsk_01M2J2FCGVZN9FHMK5HVK3ZDHR -->
- [ ] 4.3 紅燈→綠燈：`reorder_via` 在產生新全文前以快照的 plan dependsOn 圖與拖放後同階段序列呼叫 `speclink_core::plan::violations`，違反即回 RemoteError 且不呼叫 PUT；測試「remote 拖排違反依賴不發 PUT」與「無 plan（舊 server）跳過檢查照常 PUT」。驗證：`cargo test -p speclink-desktop --test remote_data` 全綠。 <!-- speclink-task:tsk_01M2J2FCGVHD8NY2ABZFQDA17H -->
- [ ] 4.4 紅燈→綠燈：新增 Tauri command `remote_set_change_depends` 單行委派 remote.rs 的 `set_depends`；apps/desktop/src/adapter/remoteDataSource.ts 的 `setDepends` 呼叫該 command 並讀 `planError`；session.ts 的 remote capability `setDepends` 與 reorderCard 同源（editor true、reader false、offline mask 同列）。驗證：apps/desktop/src/__tests__/remoteDataSource.test.ts 斷言 invoke 參數與 planError 傳遞、remoteCapabilities.test.tsx 斷言 role 兩值；`npm test -w apps/desktop` 全綠。 <!-- speclink-task:tsk_01M2J2FCGVWXWHESJ0QDHZ3YR5 -->

## 5. 收尾

- [ ] 5.1 全面回歸：`cargo test -p speclink-protocol`、`cargo test -p speclink-remote`、`cargo test -p speclink-server --test plan_api`、`cargo test -p speclink-cli --test it`、`cargo test -p speclink-desktop --test remote_data`、`npm test -w apps/desktop` 全綠；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠。 <!-- speclink-task:tsk_01M2J2FCGVHJVRMDFR05ECWJZG -->
- [ ] [M] 5.2 以本機 server（npm run dev）開 remote 分頁：看板變更順序與 local 分頁對同一內容一致、卡片有波次章、editor 可在排程分頁新增前置且 SSE 刷新後順序更新、拖到宣告前置之前得到單行錯誤、以 reader 登入時排程分頁無編輯控制項。驗證：人工核對上述五項。 <!-- speclink-task:tsk_01M2J2FCGVGS3FYBPRPRRQ4XJX -->
