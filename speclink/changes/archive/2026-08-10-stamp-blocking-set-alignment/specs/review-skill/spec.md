## MODIFIED Requirements

### Requirement: 審查後的迴圈與收尾

<!-- BEFORE: Discovery 有任何 findings 即詢問三選項；Bn 為空且無 accepted findings 才蓋章；SUGGESTION 也逼修或逼問 --accept -->

Discovery 呈現與 triage 後，有必修 findings 時技能 SHALL 沿既有三選項讓使用者選擇：修正後重審／接受現狀蓋章／先不蓋章；零 findings 或僅 SUGGESTION 級 findings 時 SHALL NOT 詢問——單站直接呼叫時記錄該輪後直接執行 review stamp（乾淨蓋章，SUGGESTION 級不擋章、無需批准）。修正 SHALL 一律由主線依專案 TDD 慣例執行，sub-agent 不得修改檔案；修正後 SHALL 先通過「修復迴圈的驗證門」，再開始 validation。於 quality 時序中（由 /speclink-quality 依序呼叫時），零 findings 或僅 SUGGESTION 的 discovery 與必修集合淨空的 validation 輪皆 SHALL NOT 當場蓋章，SHALL 改走既有「先不蓋章」離場，蓋章延至 quality 的收尾補蓋；惟編排方明示本次呼叫為收尾補蓋時，此禁蓋例外 SHALL NOT 適用——該呼叫中必修淨空的輪即蓋。quality 收尾補蓋 SHALL 區分必修淨空末輪的來源：外部守門失敗留下者沿既有路徑直接重試 stamp；quality 時序刻意留下者 SHALL 僅於收尾補蓋呼叫中蓋章——該呼叫先以 review scope 確認凍結點後內容未再移動，無移動即重試 stamp，有移動則於同一呼叫內先跑 validation 輪至必修淨空再蓋，SHALL NOT 對未驗證的移動直接補蓋；同一必修淨空末輪若由非收尾補蓋的呼叫進入，無論凍結點後有無移動皆 SHALL NOT 蓋章。單站直接呼叫時行為不變。

每輪 validation 後，技能 SHALL 以未接受的必修集合 Bn 與上輪 Bn-1 比較：

- Bn 為空且沒有 accepted 必修 findings 時 SHALL 執行 review stamp，結果為 passed clean——末輪殘留的 SUGGESTION 級 findings 不擋章、無需批准
- Bn 為空且仍有 accepted 必修 findings 時 SHALL 推薦使用者明示 review stamp --accept，結果為 passed with reservations
- Bn 非空且數量嚴格小於 Bn-1 時 SHALL 允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- Bn 數量大於或等於 Bn-1 時 SHALL 在記錄本輪後立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動工具不可用時 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪自動蓋章

- **WHEN** 單站直接呼叫且 discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 review stamp 並回報 passed clean

#### Scenario: 僅 SUGGESTION 首輪直接蓋章

- **WHEN** 單站直接呼叫且 discovery 僅記錄 SUGGESTION 級 findings、無任何必修
- **THEN** 技能記錄該 discovery round 後不發三選項詢問，直接執行 review stamp 並回報 passed clean，報告列出 SUGGESTION 清單

#### Scenario: quality 時序中乾淨首輪先不蓋章

- **WHEN** 於 quality 時序中 discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，以「先不蓋章」離場，工單與 host-local snapshot 保留，不執行 review stamp

#### Scenario: quality 時序中複驗淨空仍先不蓋章

- **WHEN** 於 quality 時序中（本次呼叫非收尾補蓋）validation 輪後必修集合為空且無 accepted 必修 findings
- **THEN** 技能記錄該輪後以「先不蓋章」離場，不執行 review stamp，蓋章延至 quality 的收尾補蓋

#### Scenario: 非收尾呼叫進入乾淨末輪不蓋章

- **WHEN** quality 時序中非收尾補蓋的呼叫進入已存在的必修淨空未蓋章末輪，且 review scope 顯示凍結點後無內容移動
- **THEN** 技能回報無新內容可判並結束，不執行 review stamp、不動工單

#### Scenario: quality 收尾補蓋前內容再移動則先驗後蓋

- **WHEN** quality 收尾補蓋時 review scope 顯示必修淨空末輪凍結點後仍有內容移動
- **THEN** 技能於同一呼叫內先執行 validation 輪，必修淨空後即執行 review stamp（收尾補蓋呼叫不受禁蓋例外攔截），不對未驗證的移動直接補蓋

#### Scenario: 有進展時允許再驗收

- **WHEN** 上輪有兩筆必修，validation 後剩一筆未解且沒有直接 regression
- **THEN** 技能記錄新輪並允許再次選擇修正，SHALL NOT 宣稱已通過

#### Scenario: 第一個無進展輪立即停止

- **WHEN** 上輪有一筆必修，validation 後同一筆仍未解
- **THEN** 技能記錄該輪後回報 failed，保留工單、不蓋章且不再派出 sub-agent

#### Scenario: 只剩保留事項

- **WHEN** validation 已解決所有必修但末輪仍帶 accepted 必修 findings
- **THEN** 技能推薦 review stamp --accept，不再為 accepted items 啟動 validation

#### Scenario: 先不蓋章離場

- **WHEN** 使用者在可選擇的節點選擇先不蓋章
- **THEN** 技能結束，工單與 host-local snapshot 保留，metadata 不變

### Requirement: 審查結果的裁量分類

<!-- BEFORE: 可裁含 possible-X 措辭的 WARNING；僅剩可裁項時推薦「接受現狀蓋章」並以 review stamp --accept 帶保留蓋章 -->

技能 SHALL 於兩軸結果並列呈現後、詢問使用者之前，對本輪每筆 finding 給出處置分類並隨報告一併呈現（不改動工單記錄格式）：**必修**——CRITICAL 級、Correctness 軸判定有現實觸發路徑的 bug（含 WARNING 級）、文件化 repo 標準的明確違反；**可裁**——"possible X" 措辭的 smell 判斷與其他 SUGGESTION 級事項，每筆附一行修繕成本與效益的裁量理由。可裁事項 SHALL 一律以 SUGGESTION 級記錄入單、SHALL NOT 以 WARNING 級入單——WARNING 保留給必修級判定；SUGGESTION 級不擋蓋章、不進入接受機制。三選項詢問 SHALL 僅於必修項存在時發出並帶明確推薦：推薦「修正後重審」並列出必修清單；僅剩可裁項時 SHALL NOT 詢問，依「審查後的迴圈與收尾」直接乾淨蓋章（單站直接呼叫）或先不蓋章離場（quality 時序中）。

#### Scenario: 有必修項時推薦修正

- **WHEN** 某輪 findings 含一筆 CRITICAL 與三筆 possible-X 措辭的 SUGGESTION
- **THEN** 呈現的分類為 1 筆必修、3 筆可裁，三選項詢問以「修正後重審」為推薦選項且附必修清單

<!-- REMOVED-SCENARIO: 僅剩可裁項時推薦接受 -->

#### Scenario: 僅剩可裁項時不詢問直接蓋章

- **WHEN** 某輪 findings 僅含 SUGGESTION 級可裁事項，無 CRITICAL、無現實路徑 bug、無文件化標準違反
- **THEN** 技能不發三選項詢問；單站直接呼叫時記錄該輪後直接執行 review stamp（乾淨蓋章），報告列出 SUGGESTION 清單

#### Scenario: 可裁事項一律以 SUGGESTION 入單

- **WHEN** Standards 軸對一段程式碼給出 possible Feature Envy 的 smell 判斷
- **THEN** 該筆以 SUGGESTION 級寫入工單，SHALL NOT 以 WARNING 級入單

### Requirement: 已接受事項的續輪前饋

<!-- BEFORE: 接受機制適用於任何 findings（含可裁 smell），蓋章一律走 review stamp --accept -->

接受機制 SHALL 僅適用於必修級 findings——SUGGESTION 級不擋章、不需接受，未修的 SUGGESTION 依既有 validation 規則以未解 finding 原文續帶、SHALL NOT 附 `(accepted)` 標記。已裁定接受而未修正的必修 findings，技能於續輪 SHALL 雙軌處置：(1) 續輪 sub-agent 的指示 SHALL 附上該清單並明令不得重報同一事項或其近似變體；(2) 續輪記錄 SHALL 由主線將這些事項原樣帶入該輪 findings 清單，並於行末附結構性標記 `(accepted)`（比照 severity 標籤維持英文、不隨 locale 翻譯），使末輪工單忠實反映殘留保留事項，蓋章走 `review stamp --accept`。跨 session 接手時，技能 SHALL 以末輪帶 `(accepted)` 標記的行重建不重報清單。

#### Scenario: 接受過的事項不再重報

- **WHEN** Round N 的一筆 WARNING 級現實路徑 bug 經使用者裁定接受後執行下一輪
- **THEN** 續輪 sub-agent 指示含該事項的不重報清單，且 Round N+1 的工單記錄由主線原樣帶入該筆事項並以 `(accepted)` 標記收尾

#### Scenario: 跨 session 重建不重報清單

- **WHEN** 另一 session 對末輪含 `(accepted)` 標記行的工單執行 `/speclink-review`
- **THEN** 該 session 的 sub-agent 指示以標記行重建不重報清單，標記事項不被重報
