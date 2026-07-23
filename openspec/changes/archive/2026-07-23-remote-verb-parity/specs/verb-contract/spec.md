## MODIFIED Requirements

### Requirement: 動詞契約的涵蓋面與 payload 形狀

動詞契約 SHALL 涵蓋 RD 本地全流程：changes 列舉／讀取／建立／認領／歸檔／捨棄（discard，含 force 語意）、artifacts 讀取／寫入、tasks 勾選／取消勾選／搬移（1-based ordinal 定址與 before 側別）、validate 與 analyze 衍生查詢、instructions 計算、discussions 全動詞（列舉、建立、context、add-round、conclude、archive、promote）、政策讀取（workflow-config 有效值）、詞彙讀取（LANGUAGE 內容）、正典規格讀取、身分查驗。remote 模式下對應 CLI 指令的 stdout 輸出（人眼與 --json）形狀 SHALL 與 fs 模式一致（欄位 camelCase 同名）；validate 的聚合語意（無參數／--all／--changes）SHALL 由 client 以逐 change 端點呼叫組合，聚合輸出形狀與 fs 模式一致。契約的端點、payload 與錯誤形狀 SHALL 以 docs/verb-contract.md 為正典參考文件。

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
