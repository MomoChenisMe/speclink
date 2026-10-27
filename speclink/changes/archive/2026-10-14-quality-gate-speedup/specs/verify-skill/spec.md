## ADDED Requirements

### Requirement: 驗證修復迴圈的驗證門

使用者選擇「修正後重驗」(或於 quality 時序中裁示修正)時，技能 SHALL 於修正完成後、派出下一輪檢查之前，執行建置並跑受影響範圍的測試，要求全數通過；未通過時 SHALL 先修復至全綠，不得帶著建置或測試失敗進入下一輪。受影響範圍 SHALL 依序判定：

1. 修正改到的檔案落在某個任務說明列出的路徑內時，跑該任務說明寫的驗證指令（任務原文取自 `speclink instructions apply --change <name> --json` 的 `tasks`）；
2. 任務沒涵蓋的檔案，依專案文件化的測試政策（CLAUDE.md、AGENTS.md 或 workflow config 的產出規則）選擇測試；
3. 專案未寫任何測試政策時，跑 repo 慣例的全量測試指令。

建置 SHALL 照跑，SHALL NOT 因範圍判定而略過。報告 SHALL 列出本次跑了哪些測試指令、略過哪些範圍，並註明略過的範圍交由 CI 守門。

修正動到 artifact（規格、design、tasks）時，技能 SHALL 依三種情況處理：

- 只改程式碼使其符合規格：SHALL NOT 修改 artifact。
- 規格字面要修、意圖不變：主線直接修改 artifact，並在 `speclink validate <change>` 通過後才進下一輪；報告 SHALL 列出改了哪幾段；下一輪的檢查簡報 SHALL 附上改過段落的原文，檢查據此判定原 finding 是否已解——驗證 patch 不含 spec 目錄下的檔案。
- 意圖或範圍改變（修正需要新增任務，或改變某條 Requirement 的意圖）：技能 SHALL 以既有「範圍改變」出口結束本站（保留工單、不蓋章），並建議先執行 `/speclink-ingest` 再重跑；SHALL NOT 於迴圈內自行新增或退回任務。

#### Scenario: 修正引入的測試失敗被驗證門攔下

- **WHEN** 修正讓同一 package 的另一個測試失敗
- **THEN** 技能於派出下一輪檢查之前發現並修復，下一輪開始時建置與受影響範圍的測試全綠

#### Scenario: 只跑任務寫的驗證指令

- **WHEN** 修正只改了任務 3.2 說明列出的前端檔案，該任務說明寫明以 `npm test -w packages/ui` 驗證
- **THEN** 技能執行建置與 `npm test -w packages/ui`，不執行全量測試；報告列出已跑的指令與略過的範圍，並註明交由 CI 守門

#### Scenario: 規格字面修正後下一輪看得到

- **WHEN** Round 1 的 WARNING 指出 delta 規格的 Scenario 與實作不符，使用者裁示改規格字面、意圖不變
- **THEN** 技能修改 delta 規格並通過 `speclink validate`，Round 2 的檢查簡報附上改過的 Scenario 原文，檢查據此判定該 WARNING 已解

#### Scenario: 意圖改變時走範圍改變出口

- **WHEN** 某筆 finding 的修正需要改變一條 Requirement 的意圖
- **THEN** 技能以「範圍改變」結束本站並保留工單、不蓋章，建議先執行 /speclink-ingest 再重跑，不在迴圈內改 tasks

### Requirement: 驗證子代理簡報的查找紀律與證據順序

技能派出的驗證檢查子代理簡報 SHALL 遵守下列查找紀律：

- 鎖定 patch 在 25 KB 以內時 SHALL 直接貼入簡報；超過時 SHALL 只給檔案路徑與 hunk 的行號範圍。
- 自動產生的檔案（golden snapshot、assets.lock、由 `speclink update` 再生的技能檔）SHALL 只給路徑，並指示子代理對它們最多搜尋一次。
- SHALL 指示子代理不得整份讀取大檔（以行號範圍讀取），不得做沒有指定目標的遞迴搜尋（只搜尋簡報列出的函式名、型別名與檔案）。
- discovery SHALL NOT 設工具呼叫次數上限；validation SHALL 指示工具呼叫上限 5 次。
- 簡報結尾 SHALL 指示「沒查完就直說，不要猜」；子代理回報未查完的部分 SHALL 原樣列入報告，由使用者決定是否補查。
- finding 指向鎖定 patch 以外的呼叫端時，SHALL 錨定在 patch 內被呼叫的檔案，路徑 SHALL NOT 帶行號（配合 verify-station「驗證 finding 路徑落在工單範圍內」）。
- 於 quality 時序中，簡報照本條組成，由編排方與審查站的簡報於同一則訊息派出。

證據順序：Completeness 與 Correctness 尋找每條 Requirement 的實作證據時，子代理 SHALL 先在鎖定 patch 內尋找；patch 內完全找不到某條 Requirement 的實作時（例如由變更前既有的程式碼滿足），才 SHALL 往 patch 以外搜尋，且只搜尋具體的函式名、型別名或檔名，SHALL NOT 以關鍵字搜尋整個 codebase。

#### Scenario: 證據先在差異內找

- **WHEN** delta 規格有 8 條 Requirement，其中 7 條的實作與測試都在鎖定 patch 內
- **THEN** 子代理只在 patch 內判定這 7 條，僅對剩下 1 條以具體函式名往 patch 外搜尋

#### Scenario: 由既有程式碼滿足的 Requirement

- **WHEN** 某條 Requirement 在鎖定 patch 內找不到實作，因為它由變更前既有的函式滿足
- **THEN** 子代理以該函式名往 patch 外搜尋並確認，報告註明證據位於 patch 外

#### Scenario: 複驗輪限五次工具呼叫

- **WHEN** validation 輪派出檢查子代理
- **THEN** 簡報載明工具呼叫上限 5 次、只附上輪未解 findings 與驗證 patch，以及「沒查完就直說，不要猜」

#### Scenario: 大差異只給範圍

- **WHEN** discovery 的鎖定 patch 為 60 KB
- **THEN** 簡報只給各檔路徑與 hunk 行號範圍，不直接貼入 patch 全文
