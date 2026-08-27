## MODIFIED Requirements

### Requirement: 動詞契約的涵蓋面與 payload 形狀

動詞契約 SHALL 涵蓋 RD 本地全流程：changes 列舉／讀取／建立／認領／歸檔／捨棄（discard，含 force 語意）／開工標記（in-progress add）、artifacts 讀取／寫入、tasks 勾選／取消勾選／搬移（1-based ordinal 定址與 before 側別）、validate 與 analyze 衍生查詢、instructions 計算、discussions 全動詞（列舉、建立含 slug 覆寫、context、add-round、conclude、archive、promote、discard 含 force 語意、link、seal）、政策讀取（workflow-config 有效值）、詞彙讀取（LANGUAGE 內容）、正典規格讀取、身分查驗。remote 模式下對應 CLI 指令的 stdout 輸出（人眼與 --json）形狀 SHALL 與 fs 模式一致（欄位 camelCase 同名）；validate 的聚合語意（無參數／--all／--changes）SHALL 由 client 以逐 change 端點呼叫組合，聚合輸出形狀與 fs 模式一致；show SHALL 由 client 以讀取端點組合出與 fs 模式同形輸出，SHALL NOT 於 remote 模式讀取本機 store。本質本機動詞（demo、trace）於 remote 模式 SHALL 明確拒絕（非零 exit code、stderr 說明僅限本機模式），SHALL NOT 靜默作用於本機 store，SHALL NOT 發出任何 server 請求；trace 的拒絕訊息 SHALL 說明溯源鏈自本地 openspec 樹組裝。契約的端點、payload 與錯誤形狀 SHALL 以 docs/verb-contract.md 為正典參考文件。

list --json 的 change 條目得含本機觀察面的可空欄位 worktree（物件：path 字串與 branch 字串）——該欄位僅於 fs 模式的主 checkout、worktree 政策開啟且映射成立時出現，缺席時 SHALL NOT 序列化；remote 模式的 list 條目 SHALL 恆缺席此欄位。形狀一致性 SHALL 以「可空且缺席不序列化」維持：無 worktree 情境下兩模式的 list 輸出逐欄位一致。

#### Scenario: remote 列舉輸出形狀與 fs 一致

- **WHEN** 於 remote 模式執行 speclink list --json，server 回傳兩個 change
- **THEN** stdout 的 JSON 欄位名與 fs 模式的 speclink list --json 完全一致，exit code 為 0

#### Scenario: instructions 由 server 計算

- **WHEN** 於 remote 模式執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 含 context、rules、template、locale 等欄位（值來自 server 端政策），欄位名與 fs 模式一致

#### Scenario: remote validate 輸出形狀與 fs 一致

- **WHEN** 於 remote 模式執行 speclink validate --json（無參數聚合）
- **THEN** stdout 為與 fs 模式同形的逐 change results JSON（欄位名一致），有任一 invalid 時 exit code 非 0

#### Scenario: remote discard 的 guard 語意與本地一致

- **WHEN** 於 remote 模式對已勾選任務的 change 執行 speclink discard（無 --force）
- **THEN** exit code 非 0，stderr 語義化訊息說明已開工需 --force（與 fs 模式同語意），server 上該 change 完整保留

#### Scenario: remote 建立討論帶 slug 覆寫

- **WHEN** 於 remote 模式以中文主題執行 speclink discuss new 並帶 --slug board-search-bar
- **THEN** server 端以 board-search-bar 建立記錄，stdout 與 --json 的 slug 欄位形狀與 fs 模式一致；非法 slug 值時 exit code 非 0、stderr 說明原因、server 端不落檔

#### Scenario: remote show 輸出與 fs 一致

- **WHEN** 同一份 change 內容分別存在於 fs 專案與 remote server，兩模式各執行 speclink show 該 change
- **THEN** 兩者的人眼輸出與 --json 逐欄位一致；remote 模式的結果來自 server 資料，未讀取本機 store

#### Scenario: remote in-progress add 蓋章於 server

- **WHEN** 於 remote 模式對 server 上存在的 change 執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端該 change 的 meta 含 started_at 與 started_by（server 認證身分）

#### Scenario: demo 於 remote 明確拒絕

- **WHEN** 於 remote 模式執行 speclink demo
- **THEN** exit code 非 0，stderr 說明 demo 僅限本機模式，本機與 server 均未新增任何 change

#### Scenario: trace 於 remote 明確拒絕

- **WHEN** 於 remote 模式執行 speclink trace 某 capability
- **THEN** exit code 非 0，stderr 說明 trace 僅限本機模式且溯源鏈自本地 openspec 樹組裝，未發出任何 server 請求

#### Scenario: remote list 恆無 worktree 欄位

- **WHEN** 於 remote 模式執行 speclink list --json，而本機恰有合乎慣例的 linked worktree
- **THEN** 所有條目均無 worktree 欄位，欄位名與 fs 模式無 worktree 情境完全一致
