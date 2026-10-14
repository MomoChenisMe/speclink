## MODIFIED Requirements

### Requirement: 修復迴圈的驗證門
<!-- BEFORE: 修正後一律跑 repo 慣例的全量建置與測試；未規定修正動到 artifact 時的處理 -->

使用者選擇「修正後重審」(或於 quality 時序中裁示修正)時，技能 SHALL 於修正完成後、派出下一輪 sub-agent 之前，執行建置並跑受影響範圍的測試，要求全數通過；未通過時 SHALL 先修復至全綠，不得帶著建置或測試失敗進入下一輪。受影響範圍 SHALL 依序判定：

1. 修正改到的檔案落在某個任務說明列出的路徑內時，跑該任務說明寫的驗證指令（任務原文取自 `speclink instructions apply --change <name> --json` 的 `tasks`）；
2. 任務沒涵蓋的檔案，依專案文件化的測試政策（CLAUDE.md、AGENTS.md 或 workflow config 的產出規則）選擇測試；
3. 專案未寫任何測試政策時，跑 repo 慣例的全量測試指令。

建置 SHALL 照跑，SHALL NOT 因範圍判定而略過。結果呈現 SHALL 列出本次跑了哪些測試指令、略過哪些範圍，並註明略過的範圍交由 CI 守門。

修正動到 artifact（規格、design、tasks）時，技能 SHALL 依三種情況處理：

- 只改程式碼使其符合規格：SHALL NOT 修改 artifact。
- 規格字面要修、意圖不變：主線直接修改 artifact，並在 `speclink validate <change>` 通過後才進下一輪；結果呈現 SHALL 列出改了哪幾段；下一輪的 sub-agent 簡報 SHALL 附上改過段落的原文——驗證 patch 不含 spec 目錄下的檔案，不附上則下一輪看不到修正。
- 意圖或範圍改變（修正需要新增任務，或改變某條 Requirement 的意圖）：技能 SHALL 以既有「範圍改變」出口結束本站（保留工單、不蓋章），並建議先執行 `/speclink-ingest` 再重跑；SHALL NOT 於迴圈內自行新增或退回任務。

#### Scenario: 修正引入的編譯錯誤被驗證門攔下

- **WHEN** 修正過程改壞另一呼叫端導致專案建置失敗
- **THEN** 技能於派出下一輪 sub-agent 之前發現並修復，下一輪開始時建置與受影響範圍的測試全綠

#### Scenario: 只跑任務寫的驗證指令

- **WHEN** 修正只改了任務 2.1 說明列出的檔案，該任務說明寫明以 `cargo test -p speclink-core` 驗證
- **THEN** 技能執行建置與 `cargo test -p speclink-core`，不執行 workspace 全量測試；結果呈現列出已跑的指令與略過的其餘 package，並註明交由 CI 守門

#### Scenario: 專案未寫測試政策時跑全量

- **WHEN** 修正改到的檔案不在任何任務說明的路徑內，且專案的 CLAUDE.md、AGENTS.md 與 workflow config 都沒有測試政策
- **THEN** 技能執行 repo 慣例的全量測試指令，與本變更前的行為相同

#### Scenario: 規格字面修正後驗證並附上段落

- **WHEN** 使用者裁示的修正把 delta 規格某個 Scenario 的 THEN 改寫成與程式碼一致、意圖不變
- **THEN** 技能修改該 delta 規格、`speclink validate` 通過後才進下一輪，結果呈現列出改過的段落，下一輪簡報附上該段原文

#### Scenario: 意圖改變時走範圍改變出口

- **WHEN** 某筆 finding 的修正需要在 tasks.md 新增一項任務
- **THEN** 技能不新增任務、不寫入新輪，以「範圍改變」結束本站並保留工單、不蓋章，建議先執行 /speclink-ingest 再重跑

## ADDED Requirements

### Requirement: 審查子代理簡報的查找紀律

技能派出的 Standards 與 Correctness 子代理簡報 SHALL 遵守下列查找紀律：

- 鎖定 patch 在 25 KB 以內時 SHALL 直接貼入簡報；超過時 SHALL 只給檔案路徑與 hunk 的行號範圍。
- 自動產生的檔案（golden snapshot、assets.lock、由 `speclink update` 再生的技能檔）SHALL 只給路徑，並指示子代理對它們最多搜尋一次。
- SHALL 指示子代理不得整份讀取大檔（以行號範圍讀取），不得做沒有指定目標的遞迴搜尋（只搜尋簡報列出的函式名、型別名與檔案）。
- discovery SHALL NOT 設工具呼叫次數上限；validation SHALL 指示工具呼叫上限 5 次。
- 簡報結尾 SHALL 指示「沒查完就直說，不要猜」；子代理回報未查完的部分 SHALL 原樣列入結果呈現，由使用者決定是否補查。
- finding 指向鎖定 patch 以外的呼叫端時，SHALL 錨定在 patch 內被呼叫的檔案，路徑 SHALL NOT 帶行號（配合 review-station「finding 路徑落在工單範圍內」）。
- 於 quality 時序中，簡報照本條組成，由編排方與驗證站的簡報於同一則訊息派出。

#### Scenario: 小差異直接貼入簡報

- **WHEN** discovery 的鎖定 patch 為 18 KB
- **THEN** 兩軸簡報都直接含該 patch 全文，並列出要查的函式名，不要求子代理自行讀取 patch 檔

#### Scenario: 大差異只給範圍

- **WHEN** discovery 的鎖定 patch 為 60 KB，其中 40 KB 是 golden snapshot
- **THEN** 簡報只給各檔路徑與 hunk 行號範圍，golden snapshot 只給路徑並註明最多搜尋一次

#### Scenario: 複驗輪限五次工具呼叫

- **WHEN** validation 輪派出子代理
- **THEN** 簡報載明工具呼叫上限 5 次與「沒查完就直說，不要猜」

#### Scenario: 未查完的部分照實呈現

- **WHEN** Correctness 子代理回報有兩個呼叫端未查完
- **THEN** 結果呈現原樣列出未查完的兩個呼叫端，不把它們當成已確認無問題

### Requirement: quality 時序轉交的 CRITICAL 確認

於 quality 時序中，編排方已於停下處逐條列出 CRITICAL finding 原文並取得使用者明確確認，且呼叫本站時明示「CRITICAL 已逐條確認」者，技能 SHALL 以該轉交作為「接受 CRITICAL 必修項的逐條確認」所要求的確認，SHALL NOT 再次詢問。轉交所列的 CRITICAL 少於本站本次要接受的 CRITICAL 時，技能 SHALL 只對未列的部分依該條逐條確認。單站直接呼叫時行為不變。

#### Scenario: 轉交後不再詢問

- **WHEN** quality 收尾補蓋呼叫本站並明示「CRITICAL 已逐條確認」，工單待接受的 CRITICAL 只有轉交所列的那一筆
- **THEN** 技能直接執行帶 `--accept` 的 review stamp，不再詢問使用者

#### Scenario: 轉交不完整時補問未列者

- **WHEN** 轉交只列一筆 CRITICAL，而工單待接受的 CRITICAL 有兩筆
- **THEN** 技能只對未列的那一筆列出原文並要求確認，確認後才蓋帶保留章
