## 1. protocol 欄位擴充（契約先行）

- [ ] 1.1 依規格「封存回應的完整結果欄位」「工單回應的原文欄位」擴充 crates/speclink-protocol/src/command.rs 的 ArchiveResponse（datedName 選填、ArchivedSpec 四項計數預設 0、snapshotCreated 選填、archivedDiscussions 預設空、evidenceRecorded 選填）與 ReviewTicketResponse（content 選填）。先寫紅測試再實作：空物件與既有形狀 JSON 反序列化成功、新欄位取預設缺席值、schema 匯出含新欄位（camelCase 命名斷言）。驗證：`cargo test -p speclink-protocol` 全綠 <!-- speclink-task:tsk_01KZE8B03VZQPZTZZ1RBA0BAAQ -->

## 2. server 端點回填

- [ ] 2.1 依規格「archive 與工單讀取端點回填完整結果」讓 crates/speclink-server/src/routes.rs 的封存端點自引擎封存結果回填全部新欄位、review 與 verify 工單讀取端點回填 content 為工單原文全文；既有欄位、狀態碼與 404 語意不變。先寫紅整合測試：封存改動規格時回應含 datedName 與四項計數；封存帶來源討論且無證據時 archivedDiscussions 與 evidenceRecorded=false；工單在場時 content 等於原文；工單缺席維持 404。驗證：`cargo test -p speclink-server --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03V9EJC6RE17R1PTTS2 -->

## 3. remote client 讀取

- [ ] 3.1 crates/speclink-remote/src/client.rs 的 archive 與 station_ticket 方法隨 DTO 擴充讀取新欄位（typed client 不走 raw JSON 旁路）。先寫紅測試：typed_client 測試面斷言新欄位可讀、缺席時為預設值。驗證：`cargo test -p speclink-remote --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03VB52ARRYWFT8T29R5 -->

## 4. CLI 渲染收斂（A 類：wire 已載夠）

- [ ] 4.1 依規格「動詞人眼輸出的兩模式同形」做 `speclink list` 渲染收斂：把本機人眼渲染抽為單一函式（吃 core 清單型別），remote 路徑經欄位轉接餵同一支；remote 輸出開始渲染 invalid 標記（漂移修正，兩模式文本逐位元一致，worktree 標示維持 remote 恆缺席的明文分歧）。先更新 crates/speclink-cli/tests/it/remote_verb_parity.rs 對照為本機文本（先紅），實作至綠；確認本機既有對照零改動。驗證：`cargo test -p speclink-cli --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03VE1FZA37S10GQJAW6 -->
- [ ] 4.2 [P] discuss 全家（new／list／show／context／add-round／conclude／archive／promote／discard／link／seal）成功訊息與清單渲染收斂為每子指令單一函式，crates/speclink-cli/src/remote_commands.rs 對應重複文本刪除；兩模式文本逐位元一致。驗證：`cargo test -p speclink-cli --test it` 全綠（discuss 相關對照更新在同批） <!-- speclink-task:tsk_01KZE8B03VWS703YEQ7AHFS19Y -->
- [ ] 4.3 [P] `task done`／`task undone`／`in-progress remove`／`discard` 的成功行與 --json 組裝收斂為單一函式；--json 欄位集合與 camelCase 命名零變更（payload 結構斷言）。驗證：`cargo test -p speclink-cli --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03VQKFSDQ92VJRVG4WH -->
- [ ] 4.4 [P] review 與 verify 的 add-round／stamp／discard 成功行收斂；依規格「動詞 --json 輸出形狀凍結」把 `review show --json`／`verify show --json` 的 remote 路徑改走 ticket_json 同一組裝（wire 轉回 core 工單型別後組裝），斷言 content 不出現在任何 --json payload、欄位集合兩模式一致。驗證：`cargo test -p speclink-cli --test it` 全綠（含 crates/speclink-cli/tests/it/no_raw_wire_json.rs 守門） <!-- speclink-task:tsk_01KZE8B03VSCE0S9Q2J7EFYAPN -->

## 5. CLI 渲染對齊（B 類：wire 新欄位）

- [ ] 5.1 `speclink archive` 的 remote 路徑：wire 回應轉回 core 封存結果型別、餵本機同一支渲染（完整輸出：封存目的地、規格計數、封存討論、零證據 stderr 提示）；datedName 哨兵缺席時整體退回既有 remote 簡短輸出、不混合渲染；evidenceRecorded 缺席不印提示。先寫紅測試：新 server fixture 兩模式文本一致、舊 server fixture 退化輸出、exit code 皆 0。驗證：`cargo test -p speclink-cli --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03VM0VDDQWV9F0M8FW4 -->
- [ ] 5.2 `review show`／`verify show` 人眼路徑：content 在場時印工單原文全文（與本機同文本）、缺席時退回既有結構化摘要；wire 的 phase token 轉回引擎型別時未知 token 明確報錯（fail loud，訊息指出 token）。先寫紅測試覆蓋三態：原文在場、原文缺席、未知 phase token。驗證：`cargo test -p speclink-cli --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03VPHXMRXDA253XZMHE -->

## 6. 收尾

- [ ] 6.1 殘留盤點與跨面驗證：人工確認 crates/speclink-cli/src/remote_commands.rs 不再存在與本機重複的人眼渲染文本（守門 bail 與 design D5 分歧清單四項除外），本機模式人眼與 --json 輸出零變更（既有本機對照零改動佐證）；跨面收尾驗證依 change-scoped-test-policy 逐 crate 執行：`cargo test -p speclink-protocol`、`cargo test -p speclink-server --test it`、`cargo test -p speclink-remote --test it`、`cargo test -p speclink-cli --test it` 全綠 <!-- speclink-task:tsk_01KZE8B03V5S07900PFMT0XNN5 -->
