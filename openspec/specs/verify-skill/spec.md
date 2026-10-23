# verify-skill Specification

## Purpose

/speclink-verify 技能的內容與行為：把規格符合度的檢查結果落成驗證工單、續輪只驗收前一輪的修正而不重跑全面檢查，以及收尾迴圈。本 capability 保證驗證每一輪都往收斂走而不是漫無邊際重來，續輪冒出重大晚發問題時有安全退出。

## Requirements

### Requirement: 驗證技能的工單落地

verify 技能 SHALL 於檢查段(fork)結束時依寫碼任務完成度分流:寫碼任務全數完成時(含僅餘 `[M]` 手動測試任務未勾的情形)先取得 frozen verify scope,再以 `verify add-round` 將相同 phase、patch hash、Scope 與 findings 寫入驗證工單並於報告中告知,僅餘 `[M]` 任務時 SHALL 一併點名尚餘的手動測試任務;寫碼任務未全數完成時維持對話報告(進度盤點),不呼叫 `verify scope`、不落工單。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。Completeness/Correctness/Coherence 三維度的檢查內容與 CRITICAL/WARNING/SUGGESTION 分級 SHALL 不因本需求改變。

#### Scenario: 成品驗證落 structured 工單

- **WHEN** 對任務全數完成的 change 執行 verify 技能,resolved scope 為 discovery 且檢查產出 findings
- **THEN** 檢查段執行 `verify add-round` 成功,Round 1 記錄相同的 Phase/Patch/Scope,報告說明工單已建立與輪次

#### Scenario: 僅餘手動任務走成品驗證

- **WHEN** 對寫碼任務全勾、一個 `[M]` 任務未勾的 change 執行 verify 技能
- **THEN** 檢查段照常取得 frozen scope 並以 `verify add-round` 落工單,報告點名尚餘的手動測試任務

#### Scenario: 中途盤點不落工單

- **WHEN** 對寫碼任務 3/5 的 change 執行 verify 技能
- **THEN** 技能輸出對話報告(含未完成任務),不執行 `verify scope` 或 `verify add-round`,change 目錄無 `verify.md`

#### Scenario: 技能模板生成

- **WHEN** 執行 `speclink update`
- **THEN** claude 與 codex 的 verify 技能檔更新為含 frozen scope、structured 工單與有限續輪流程的版本,且與 golden 對照一致


<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: 驗證續輪只驗收修正

任務全數完成且無工單時，技能 SHALL 執行唯一一次 discovery：讀取全部 change artifacts，並以 frozen change patch 及判斷直接影響所需的呼叫端／測試作程式碼證據，完整執行 Completeness／Correctness／Coherence。已有 structured 工單時 SHALL 執行 validation，只將上輪未解 findings、accepted 清單、remediation patch 與必要脈絡交給 fork；逐筆判定原 finding 已解／未解，並只回報 remediation patch 直接引入的 regression。

未解 finding SHALL 以原文寫入新輪；已解 finding SHALL 從新輪移除。Validation SHALL NOT 重新掃描整份 change、finding 所在整檔或未修改區域，也 SHALL NOT 新增未修改區域的 smell、SUGGESTION 或既存問題。legacy 工單缺少可對應的 phase／patch snapshot 時 SHALL fail closed，保留工單並等待使用者明示 discard 後重新 discovery，不得以 touched 整檔替代。

#### Scenario: 首輪完整驗證 frozen change

- **WHEN** change 有 12 項 requirements，而 frozen discovery patch 只含 touched 檔案的三個 hunks
- **THEN** fork 對 12 項 requirements 執行完整三維度驗證，程式碼 discovery 面限定於該三個 hunks及必要脈絡，不掃描任意 workspace 既存問題

#### Scenario: 續輪只驗收未解 finding 與修正 patch

- **WHEN** Round 1 有兩筆未解 findings，remediation patch 只修改其中一檔並新增一個呼叫端
- **THEN** validation 只判定兩筆原 findings 與該 patch 的直接 regression，不重新執行整份 change discovery

#### Scenario: 未解 finding 保留原文

- **WHEN** validation 判定一筆上輪 finding 仍未解
- **THEN** 新 round 以原 severity、path 與 text 寫回該 finding，不以改寫文字假裝 blocking set 已縮小

#### Scenario: legacy 工單缺 snapshot 時 fail closed

- **WHEN** 工單有 findings但 lastRound.patchHash 為 null，且 Host 沒有可對應 snapshot
- **THEN** 技能保留工單、說明無法精確重建 remediation delta，等待使用者明示 discard 後重新 discovery，不重驗 touched 整檔


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證收尾迴圈

技能 SHALL 於主線接手 fork 報告後進行 remediation triage；修正一律回主線依專案 TDD 慣例執行，fork 不得修改任何檔案。triage 的阻斷分界 SHALL 落在嚴重度：必修＝CRITICAL／WARNING 級；可裁事項 SHALL 一律以 SUGGESTION 級記錄、SHALL NOT 以 WARNING 級入單，且 SHALL NOT 進入接受機制——SUGGESTION 級 findings 不擋蓋章、無需任何人批准。

Discovery 有必修 findings 時 SHALL 詢問三選項——修正後重驗／接受現狀蓋章（`verify stamp --accept`）／先不蓋章結束；零 findings 或僅 SUGGESTION 級 findings 時，單站直接呼叫 SHALL 記錄該 discovery round 並由主線執行 `verify stamp`（乾淨蓋章）；於 quality 時序中（由 /speclink-quality 依序呼叫時）SHALL 記錄該 discovery round 後改以「先不蓋章」結束，且必修集合淨空的 validation 輪亦 SHALL 以「先不蓋章」結束，蓋章一律延至 quality 的收尾補蓋；惟編排方明示本次呼叫為收尾補蓋時，此禁蓋例外 SHALL NOT 適用——該呼叫中必修淨空的輪即蓋。收尾補蓋呼叫 SHALL 於入口即分流：末輪必修淨空且凍結點後內容無移動時，跳過檢查 pass 直接執行 `verify stamp`、不另落空輪；有移動時於同一呼叫內先跑 validation 輪至必修淨空再蓋，SHALL NOT 對未驗證的移動直接補蓋。

令 Bn 為第 n 輪 triage 後「未接受且要求修正」的必修集合。每輪 validation 寫入工單後 SHALL 依下列規則收尾：

- Bn 為空且沒有 accepted 必修 findings：執行 `verify stamp`，結果為 passed clean——末輪殘留的 SUGGESTION 級 findings 不擋章、無需批准
- Bn 為空且仍有 accepted 必修 findings：等待使用者明示 `verify stamp --accept`，結果為 passed with reservations
- `0 < |Bn| < |Bn-1|`：允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- `|Bn| >= |Bn-1|`：立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動詢問於不支援選單工具的環境 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪蓋章

- **WHEN** 單站直接呼叫且 discovery 的三維度皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 `verify stamp` 並回報 passed clean

#### Scenario: 僅 SUGGESTION 首輪直接蓋章

- **WHEN** 單站直接呼叫且 discovery 僅記錄 SUGGESTION 級 findings、無任何必修
- **THEN** 技能記錄該 discovery round 後不發三選項詢問，直接執行 `verify stamp` 並回報 passed clean，報告列出 SUGGESTION 清單

#### Scenario: quality 時序中乾淨首輪先不蓋章

- **WHEN** 於 quality 時序中 discovery 的三維度皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，以「先不蓋章」結束，驗證工單與 verify snapshot 保留，不執行 `verify stamp`

#### Scenario: quality 時序中複驗淨空仍先不蓋章

- **WHEN** 於 quality 時序中（本次呼叫非收尾補蓋）validation 輪後必修集合為空且無 accepted 必修 findings
- **THEN** 技能記錄該輪後以「先不蓋章」結束，不執行 `verify stamp`，蓋章延至 quality 的收尾補蓋

#### Scenario: quality 收尾補蓋於入口分流

- **WHEN** 收尾補蓋呼叫進入 verify 站且末輪必修淨空、凍結點後內容無移動
- **THEN** 技能跳過檢查 pass 直接執行 `verify stamp`，不另落空輪

#### Scenario: quality 收尾補蓋前內容再移動則先驗後蓋

- **WHEN** quality 收尾補蓋時凍結點後仍有內容移動
- **THEN** 技能於同一呼叫內先執行 validation 輪，必修淨空後即執行 `verify stamp`（收尾補蓋呼叫不受禁蓋例外攔截），不對未驗證的移動直接補蓋

#### Scenario: 有進展時允許再驗收

- **WHEN** 上輪有兩筆必修，validation 後剩一筆未解且沒有直接 regression
- **THEN** 技能記錄新輪並允許再次選擇修正，且不得宣稱已通過

#### Scenario: 第一個無進展輪立即停止

- **WHEN** 上輪有一筆必修，validation 後同一筆仍未解
- **THEN** 技能記錄該輪後回報 failed，保留工單、不蓋章且不再啟動 validation

#### Scenario: 只剩保留事項

- **WHEN** validation 已解決所有必修但末輪仍帶 accepted 必修 findings
- **THEN** 技能等待使用者明示 `verify stamp --accept`，不再為 accepted findings 啟動 validation

#### Scenario: 先不蓋章離場

- **WHEN** 使用者於可選擇節點選擇「先不蓋章」
- **THEN** 技能結束，驗證工單與 verify snapshot 保留，metadata 不變


<!-- @trace
source: stamp-blocking-set-alignment
updated: 2026-08-10
-->

---
### Requirement: 驗證續輪重大晚發問題的安全退出

Validation 偶然看見與 remediation patch 無關的新問題時，技能 SHALL NOT 將它加入目前 findings 或重新開啟 discovery。只有問題同時具有現實觸發路徑、重現方式／失敗測試／明確 invariant 破壞之一，且影響安全、資料損失或錯誤行為時，技能 SHALL 以 scope changed／failed 結束本站，保留原工單且不蓋章，並建議另開 discovery 或衍生 change。證據不足或不達門檻的事項 SHALL 僅列為後續提示，不阻斷目前 validation。

#### Scenario: 無關 smell 不加入續輪

- **WHEN** validation 期間注意到未修改鄰檔的 possible Duplicated Code
- **THEN** 該事項不寫入目前 round、不改變 blocking set，也不觸發新的 discovery

#### Scenario: 有證據的資料損失問題終止本站

- **WHEN** validation 期間發現與 remediation patch 無關但有失敗測試可重現的資料損失
- **THEN** 技能回報 scope changed／failed、保留工單且不蓋章，建議另開 discovery 或衍生 change

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證修復迴圈的驗證門

使用者選擇「修正後重驗」(或於 quality 時序中裁示修正)時，技能 SHALL 於修正完成後、派出下一輪檢查之前，執行建置並跑受影響範圍的測試，要求全數通過；未通過時 SHALL 先修復至全綠，不得帶著建置或測試失敗進入下一輪。受影響範圍 SHALL 依序判定：

1. 修正改到的檔案落在某個任務說明列出的路徑內時，跑該任務說明寫的驗證指令（任務原文取自 `speclink instructions apply --change <name> --json` 的 `tasks`）；
2. 任務沒涵蓋的檔案，依專案文件化的測試政策（CLAUDE.md、AGENTS.md 或 workflow config 的產出規則）選擇測試；
3. 專案未寫任何測試政策時，跑 repo 慣例的完整測試指令。

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
- **THEN** 技能執行建置與 `npm test -w packages/ui`，不執行完整測試；報告列出已跑的指令與略過的範圍，並註明交由 CI 守門

#### Scenario: 規格字面修正後下一輪看得到

- **WHEN** Round 1 的 WARNING 指出 delta 規格的 Scenario 與實作不符，使用者裁示改規格字面、意圖不變
- **THEN** 技能修改 delta 規格並通過 `speclink validate`，Round 2 的檢查簡報附上改過的 Scenario 原文，檢查據此判定該 WARNING 已解

#### Scenario: 意圖改變時走範圍改變出口

- **WHEN** 某筆 finding 的修正需要改變一條 Requirement 的意圖
- **THEN** 技能以「範圍改變」結束本站並保留工單、不蓋章，建議先執行 /speclink-ingest 再重跑，不在迴圈內改 tasks

<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->

---
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

<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->

---
### Requirement: Verify 報告中的獨立初步安全資訊

驗證技能 SHALL 在安全政策啟用時，讀取當前目標的離線初步安全審查結果，於報告獨立列已選標準／版本、執行結果、疑似風險、未驗證與本機報告連結；缺報告、資料或目標 SHALL 明寫未驗證與原因，不自行連網下載、不視為通過。基本掃描需要執行時 SHALL 使用同一內建離線規則命令，無AI仍可使用；需要深度審查時 SHALL 明確交棒speclink-security-review，驗證技能不以基本掃描代替該skill。技能 SHALL 不把外部標準文件當指令，不上傳原始碼。

安全資訊 SHALL 不寫成驗證工單的CRITICAL／WARNING或新章，required_for_acceptance=false時不因初步FINDING／NOT_VERIFIED阻擋驗收或Archive；enabled且required_for_acceptance=true時 SHALL 讀取共用安全驗收判定，缺報告、過期、必要未驗證或未處理中高風險時不宣稱通過，不用一般verify --accept豁免；SHALL 保留原本規格合規判斷、TDD／audit開關、既有工單／stale章守門與人工處置。停用且無歷史報告時 SHALL 維持原輸出，不額外掃描；停用後已有歷史報告可供手動查看，但不得冒充本輪新結果。

#### Scenario: 初步疑似風險不改驗證工單

- **WHEN** 未勾選納入安全驗收且原變更符合其規格，離線安全報告有疑似SQL拼接或未驗證ASVS要求
- **THEN** 驗證報告獨立揭露資訊及PASS限制，verify工單與章不因這些初步結果多出必修項，沒有新增封存阻擋

#### Scenario: 缺資料或停用政策

- **WHEN** 啟用但所需資料缺失，或完全未啟用且無歷史報告
- **THEN** 前者安全區塊標示未驗證並指向資料管理，後者既有Verify行為及輸出維持，兩者都不自動下載

#### Scenario: 既有守門仍有效

- **WHEN** 變更另有未結Verify工單或既有過期章
- **THEN** 原有封存守門與處置仍生效，不以新的安全報告通過來繞過

#### Scenario: 必要安全條件未滿

- **WHEN** required模式下驗證其他規格皆符合，但安全報告缺失、過期、必要未驗證或有未處理HIGH
- **THEN** 技能回報安全驗收未通過及原因，指向審查或風險處置，不自行接受、不連網補件、不宣稱驗收完成


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 驗證完成紀錄的技能交棒

verify技能 SHALL 回報蓋章完成紀錄位於提案checks/verify，提供verify history或Desktop查詢入口；保存原輪次、當時規格檢查及接受情形，不覆寫歷史。保存失敗 SHALL 回報未完成，不刪復原資料；Claude／Codex／Copilot及自訂工具 SHALL 使用同一原稿生成，與資產版本及golden同步。安全風險接受 SHALL 綁當前finding與理由，skill不得代使用者自動接受。

#### Scenario: 成功與保存失敗的回報

- **WHEN** verify蓋章成功，或蓋章因保存歷史失敗
- **THEN** 前者提供可查完整紀錄的recordId，後者明示未完成且不宣稱已驗證，沒有刪除唯一紀錄

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->