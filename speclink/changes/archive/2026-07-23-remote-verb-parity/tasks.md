## 1. 行編輯語意遷入 core（等價重構）

- [x] 1.1 紅：於 crates/speclink-core/src/tasks.rs 的測試模組新增任務搬移單元測試（自 apps/desktop/core 既有 move 測試搬移案例：向上/向下方向推斷、明確 before 側別、跨群組邊界、重編號只改「數字.數字」前綴、越界拒絕、檔尾換行保留），並新增 Command::TaskMove 的 gateway 測試（design「決策 4：TaskMove 為新 Command 變體，行編輯語意遷入 speclink-core」；規格「任務搬移端點與重編號效果」的效果基準）——此時紅。 <!-- speclink-task:tsk_01KY6A5HKSA4P0RX951YCVC8A1 -->
- [x] 1.2 綠：把 move＋重編號行編輯邏輯自 apps/desktop/core/src/manage.rs 遷入 speclink-core（store-trait 函式），於 crates/speclink-core/src/command/mod.rs 新增 TaskMove 變體與 typed outcome；apps/desktop/core 的 move_task_at 改薄呼叫 core 同一函式。驗收：1.1 全綠，且桌面既有 move 測試（npm test -w apps/desktop 對應案例與 cargo test -p speclink-desktop-core）零修改全綠。 <!-- speclink-task:tsk_01KY6A5HKSJ5QP3KCYAHG9SH6P -->

## 2. server 三組端點

- [x] 2.1 紅：新增 crates/speclink-server/tests/verb_api.rs 整合測試（in-process server）：規格「validate 與 analyze 為唯讀衍生查詢端點」（結果與本地引擎對同內容一致、reader 可用、缺席 change 404、scope revision 不前進）；規格「DELETE change 為 discard 全語意」（未開工直刪＋SSE invalidate、已開工 force=false 拒絕零副作用且 reason 機器可判、force=true 成功、promote 來源討論 unlink）；規格「任務搬移端點與重編號效果」（跨群組搬移重編號、越界拒絕零副作用、成功發布 invalidate）；規格「寫入動詞 editor 限定」（reader 對 DELETE 與 move 收 403 reason 機器可判、capability 依 role 呈現）——此時紅。 <!-- speclink-task:tsk_01KY6A5HKSXMMS7DG2E6C5ST3D -->
- [x] 2.2 綠：speclink-protocol 新增 validate/analyze/discard/task-move DTOs；crates/speclink-server/src/app.rs 掛 GET /changes/{name}/validate、GET /changes/{name}/analyze、DELETE /changes/{name}、POST /changes/{name}/tasks/move；crates/speclink-server/src/routes.rs 四 handlers 全部經 verb::run 的 Command gateway、寫入動詞沿既有 role 檢查（design「決策 1：三組端點全部經既有 Command gateway，不另寫流程」「決策 3：DELETE 語意＝Discard 全語意，force 為端點參數，兩入口各保既有行為」「決策 5：寫入動詞 editor role 限定，唯讀動詞 reader 可用」）。驗收：2.1 全綠。 <!-- speclink-task:tsk_01KY6A5HKSGCCRYWW4RFVYSM7V -->

## 3. client 與 CLI 解鎖

- [x] 3.1 紅：CLI remote 測試（沿 crates/speclink-cli/tests/remote_write_path.rs 模式）：remote validate 無參數聚合的 --json 形狀與 fs 模式一致且任一 invalid 時 exit 非 0、remote analyze 輸出形狀一致、remote discard 無 --force 對已開工 change 的 guard 語義化訊息與 server 端零改動（規格「動詞契約的涵蓋面與 payload 形狀」）——此時紅。 <!-- speclink-task:tsk_01KY6A5HKSSS2XTRWZ3S74KDK2 -->
- [x] 3.2 綠：crates/speclink-remote/src/client.rs 新增 validate/analyze/discard/move_task 四方法；cmd_validate 與 cmd_analyze 開頭加 remote_ctx 分流、以逐 change 端點呼叫組合聚合語意、DTO 反序列化回本地型別走同一 render 函式；remote_discard 由 bail 改實作並把 guard 拒絕翻譯為與本地相同的「需要 --force」訊息（design「決策 2：validate 端點固定單 change；CLI 的聚合模式由 client 組合」「決策 6：CLI remote 分流沿 cmd_discard 既有模式，渲染共用本地路徑」）。驗收：3.1 全綠。 <!-- speclink-task:tsk_01KY6A5HKSSS0P0WH6EJHJ9AKT -->

## 4. 桌面解鎖

- [x] 4.1 紅：更新 apps/desktop/src/__tests__/remoteDataSource.test.ts——runVerb 的 validate/analyze、deleteChange（force=true）、moveTask 四方法斷言改為對 remote_* invoke 的參數映射（不再回拒絕錯誤）；Rust 側新增 RemoteCapabilities 斷言：editor 四欄全真、reader 的 deleteChange/moveTask 為假而 validate/analyze 為真（規格「capability 驅動停用且不偽造缺口」修訂後語意）——此時紅。 <!-- speclink-task:tsk_01KY6A5HKSFPF51EPTQKX9XVQJ -->
- [x] 4.2 綠：apps/desktop/src-tauri/src/remote.rs 新增四個指令與 RemoteCapabilities 依 role 翻真、apps/desktop/src-tauri/src/lib.rs 註冊、apps/desktop/src/adapter/remoteDataSource.ts 四方法由 unsupported 改 invoke 直達；UI 元件零改動（design「決策 7：capability 翻正即解鎖，UI 元件層零改動」）。驗收：4.1 全綠、npm test -w packages/ui 零修改全綠。 <!-- speclink-task:tsk_01KY6A5HKSR88NAPB8F7ZFFPGB -->

## 5. 契約文件與收尾驗證

- [x] 5.1 更新 docs/verb-contract.md 與 docs/verb-contract.zh-TW.md：新增四端點的 payload 與錯誤形狀、discard 的 force 語意、validate 聚合由 client 組合的規則（規格「動詞契約的涵蓋面與 payload 形狀」指定的正典參考文件）。驗收：文件含四端點條目且與 2.2 實作形狀一致。 <!-- speclink-task:tsk_01KY6A5HKSHHFMY5Z7N84B3QWK -->
- [ ] 5.2 GUI 鐵律手動驗證（remote-dev-harness：npm run dev 起 server＋desktop；操作前確認使用者未在使用螢幕）：editor 身分 remote 分頁——validate/analyze 按鈕可用且結果呈現、刪除變更（確認框→卡片消失→另一 client 數秒內反映）、任務拖排落位並重編號；看板卡片拖排維持停用附繁中說明；reader 身分——刪除與任務拖排停用、validate/analyze 可用。 <!-- speclink-task:tsk_01KY6A5HKSBJPTYF4JE148HHBC -->
- [x] 5.3 全量回歸：cargo test --workspace 與 npm test（workspaces）全綠；speclink validate remote-verb-parity 通過。 <!-- speclink-task:tsk_01KY6A5HKS6YRPQXGRKXX4XSC6 -->
