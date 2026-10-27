## MODIFIED Requirements

### Requirement: Standards 軸的 smell baseline

<!-- BEFORE: smell baseline 一律帶入 Standards 軸，無條件套用 -->

技能文件的 Standards sub-agent 指示 SHALL 逐字內嵌 smell baseline 正典原文（來源：Matt Pocock code-review skill，MIT；正典文字見 design「D7a」）：引言（"On top of whatever the repo documents…"）、兩條約束規則（The repo overrides——repo 文件化標準勝出、被其背書者壓下不報；Always a judgement call——smells 一律為 "possible X" 標籤式啟發、永不作 hard violation，且 tooling 已強制的規則跳過不報）、"(Refactoring, ch.3)" 出處，以及 12 條 smells 逐項（what it is → how to fix）。12 個專有名詞 SHALL 原文不動：Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest。生成之技能檔 SHALL 含一行出處註記。

smell baseline 的套用 SHALL 依 config 的 review.baseline 決定：extend（含未設定）時，Standards 軸的 sub-agent 簡報 SHALL 帶入 baseline；replace 時，簡報 SHALL NOT 帶入 baseline，12 條 smell 不檢查、不入單。技能檔內文 SHALL 照舊逐字內嵌 baseline，不因設定而改變生成內容。12 個專有名詞的清單 SHALL 由引擎單一提供給呈現端（如 desktop 設定頁），且 SHALL 以測試釘住該清單與技能檔內嵌的 12 條逐名一致。

#### Scenario: 生成技能檔含完整 baseline

- **WHEN** `speclink update` 生成 speclink-review 技能檔（claude 與 codex）
- **THEN** 兩份檔內皆含 12 條 smells 專有名詞逐項、兩條約束規則與 "(Refactoring, ch.3)" 出處，內容與 design「D7a」正典文字逐字一致

##### Example: 專有名詞逐一在場

- **GIVEN** 生成的 speclink-review 技能檔
- **WHEN** 逐一檢索 Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest
- **THEN** 12 個名稱皆命中，且各項皆帶「→」修法句

#### Scenario: replace 時簡報不帶 baseline

- **WHEN** 檢視生成的 speclink-review 技能檔關於 Standards 軸簡報的段落
- **THEN** 段落明定 review.baseline 為 replace 時簡報不帶 smell baseline、12 條不檢查不入單；為 extend 或未設定時照舊帶入

#### Scenario: 引擎提供的 12 條清單與技能檔一致

- **WHEN** 執行 speclink-core 的測試
- **THEN** 測試確認引擎提供的 12 個名稱與生成的 speclink-review 技能檔內嵌的 12 條逐名一致，任一端改名或增刪即失敗

### Requirement: 審查結果的裁量分類

<!-- BEFORE: 必修只含 CRITICAL、現實路徑的 Correctness bug、文件化 repo 標準的明確違反 -->

技能 SHALL 於兩軸結果並列呈現後、詢問使用者之前，對本輪每筆 finding 給出處置分類並隨報告一併呈現（不改動工單記錄格式）：**必修**——CRITICAL 級、Correctness 軸判定有現實觸發路徑的 bug（含 WARNING 級）、文件化 repo 標準的明確違反、專案審查規則 critical 或 warning 清單的違反；**可裁**——"possible X" 措辭的 smell 判斷、專案審查規則 suggestion 清單的違反與其他 SUGGESTION 級事項，每筆附一行修繕成本與效益的裁量理由。可裁事項 SHALL 一律以 SUGGESTION 級記錄入單、SHALL NOT 以 WARNING 級入單——WARNING 保留給必修級判定；SUGGESTION 級不擋蓋章、不進入接受機制。三選項詢問 SHALL 僅於必修項存在時發出並帶明確推薦：推薦「修正後重審」並列出必修清單；僅剩可裁項時 SHALL NOT 詢問，依「審查後的迴圈與收尾」直接乾淨蓋章（單站直接呼叫）或先不蓋章離場（quality 時序中）。

#### Scenario: 有必修項時推薦修正

- **WHEN** 某輪 findings 含一筆 CRITICAL 與三筆 possible-X 措辭的 SUGGESTION
- **THEN** 呈現的分類為 1 筆必修、3 筆可裁，三選項詢問以「修正後重審」為推薦選項且附必修清單

#### Scenario: 僅剩可裁項時不詢問直接蓋章

- **WHEN** 某輪 findings 僅含 SUGGESTION 級可裁事項，無 CRITICAL、無現實路徑 bug、無文件化標準違反
- **THEN** 技能不發三選項詢問；單站直接呼叫時記錄該輪後直接執行 review stamp（乾淨蓋章），報告列出 SUGGESTION 清單

#### Scenario: 可裁事項一律以 SUGGESTION 入單

- **WHEN** Standards 軸對一段程式碼給出 possible Feature Envy 的 smell 判斷
- **THEN** 該筆以 SUGGESTION 級寫入工單，SHALL NOT 以 WARNING 級入單

#### Scenario: 專案規則 warning 清單的違反列為必修

- **WHEN** config 的 review.warning 含「新增的公開 API 要有對應的測試」，Standards 軸判定本輪新增的公開 API 沒有測試
- **THEN** 該筆以 WARNING 級入單並分類為必修，觸發三選項詢問；同輪 review.suggestion 清單的違反以 SUGGESTION 級入單並分類為可裁

## ADDED Requirements

### Requirement: 專案審查規則的讀取與套用

技能 SHALL 於開審前以 speclink workflow-config show --json 讀取 review 物件（baseline 與 critical、warning、suggestion 三個清單），SHALL NOT 直接讀取 config 檔；remote 模式經同一動詞取得 server 端設定。讀取失敗（例如 config 無法解析）時技能 SHALL 原樣回報錯誤並停止，SHALL NOT 在缺少專案規則的情況下開審。

Discovery 時 Standards 軸的 sub-agent 簡報 SHALL 依嚴重度分組帶入三個清單的規則原文；Correctness 軸簡報 SHALL NOT 帶入。違反專案規則的 finding SHALL 以該規則所在清單決定嚴重度（critical→CRITICAL、warning→WARNING、suggestion→SUGGESTION），sub-agent 與主線 SHALL NOT 另行升降級；finding 描述 SHALL 原文引用被違反的規則，描述其餘部分依「審查產出的語言綁定」使用解析後的 locale。專案規則與 repo 文件化標準衝突時，專案規則 SHALL 勝出。Validation 輪 SHALL NOT 因專案規則產生新 finding——續輪只判前輪 finding 是否已解與 remediation patch 的直接 regression，與既有語意相同。三個清單皆空且 baseline 為 extend 時，審查行為 SHALL 與未設定 review 時完全相同。

#### Scenario: 違反 critical 清單的規則

- **WHEN** config 的 review.critical 含「所有 SQL 一律用參數化查詢，不得用字串拼接」，frozen patch 中有一段以字串拼接組出 SQL
- **THEN** Standards 軸回報一筆 CRITICAL finding，描述原文引用該規則；主線照該嚴重度入單並分類為必修

##### Example: 工單中的 finding 行（locale 為 tw）

- **GIVEN** review.critical 含「所有 SQL 一律用參數化查詢，不得用字串拼接」，locale 為 tw
- **WHEN** crates/x/src/db.rs 第 42 行以 format! 拼接 WHERE 子句
- **THEN** 工單記為 `- [CRITICAL] crates/x/src/db.rs — Standards: 違反專案審查規則「所有 SQL 一律用參數化查詢，不得用字串拼接」：第 42 行以 format! 拼接 WHERE 子句`

#### Scenario: 規則只進 Standards 軸

- **WHEN** 檢視生成的 speclink-review 技能檔關於 discovery 簡報的段落
- **THEN** 段落明定 Standards 簡報依嚴重度分組帶入專案規則原文、Correctness 簡報不帶入、嚴重度由規則所在清單決定且不得另行升降級

#### Scenario: 讀不到設定即停止

- **WHEN** openspec/config.yaml 的 review 區塊含未知子鍵，執行 speclink-review
- **THEN** 技能原樣回報 workflow-config show 的錯誤並停止，不呼叫 review scope、不派出 sub-agent、不寫工單

#### Scenario: 續輪不因規則新增 finding

- **WHEN** Round 1 後使用者在 review.critical 新增一條規則，接著跑 validation 輪
- **THEN** validation 只判定前輪 finding 是否已解與 remediation patch 的直接 regression，不依新規則回報新 finding

### Requirement: 接受 CRITICAL 必修項的逐條確認

使用者於任何接受途徑（修正後重審時選擇不修某些必修項而以 `(accepted)` 帶入、選擇接受現狀蓋章、或依推薦明示 review stamp --accept）所接受的必修項含 CRITICAL 級時，技能 SHALL 先逐條列出這些 CRITICAL finding 的原文，並以互動工具要求使用者明確確認一次（互動工具不可用時以純文字詢問並等待回覆）；確認後才記錄 `(accepted)` 或執行 --accept 蓋章。使用者不確認時 SHALL 回到原本的選擇節點，SHALL NOT 記錄 `(accepted)`、SHALL NOT 蓋章。所接受的必修項只含 WARNING 級時，流程 SHALL 與現行相同、不追加確認。本需求不改變引擎的蓋章守門：CRITICAL 與 WARNING 照舊都擋乾淨蓋章、都可經 --accept 放行。

#### Scenario: 接受項含 CRITICAL 時追加確認

- **WHEN** 某輪必修項為一筆 CRITICAL 與一筆 WARNING，使用者選擇接受現狀蓋章
- **THEN** 技能先列出該筆 CRITICAL 原文並詢問是否確定接受；使用者確認後才執行 review stamp --accept

#### Scenario: 不確認則回到選擇

- **WHEN** 技能列出待接受的 CRITICAL 並詢問確認，使用者回答不接受
- **THEN** 技能回到三選項節點，不記錄 `(accepted)`、不執行 review stamp，工單不變

#### Scenario: 只接受 WARNING 時不追加確認

- **WHEN** 某輪必修項只有兩筆 WARNING，使用者選擇接受現狀蓋章
- **THEN** 技能照現行流程執行 review stamp --accept，不追加確認

### Requirement: 審查呈現標明套用的規則

技能於並列呈現兩軸結果時，SHALL 在開頭先印一行，寫明本次 Standards 軸套用的預設清單模式與三個專案規則清單的條數；baseline 為 replace 且三個清單皆空時，該行 SHALL 寫明 Standards 軸只依 repo 文件審查。該行 SHALL 依「審查產出的語言綁定」以解析後的 locale 撰寫（tw、ja、en；未設定時為英文），extend、replace 與 CRITICAL、WARNING、SUGGESTION 等 token 維持英文。該行只出現在呈現中，SHALL NOT 寫入工單。

#### Scenario: extend 加專案規則

- **WHEN** review 為 baseline 未設定、critical 兩條、warning 一條、suggestion 一條，locale 為 tw，執行 speclink-review 的 discovery
- **THEN** 呈現開頭一行寫明預設 12 條（extend）加上專案規則 CRITICAL 2、WARNING 1、SUGGESTION 1；工單不含此行

##### Example: 各 locale 的開頭行

| locale | review 設定 | 開頭行 |
| ------ | ----------- | ------ |
| tw | extend、2／1／1 | 審查規則：預設 12 條（extend）＋專案規則 CRITICAL 2、WARNING 1、SUGGESTION 1 |
| ja | extend、清單皆空 | 審査ルール：既定の 12 項目（extend）、プロジェクトルールなし |
| en | replace、清單皆空 | Review rules: replace, no project rules — the Standards axis judges against repo documents only |
| 未設定 | replace、critical 1 | Review rules: replace, project rules CRITICAL 1, WARNING 0, SUGGESTION 0 |

#### Scenario: replace 且無規則時寫明只依 repo 文件

- **WHEN** review 為 baseline replace、三個清單皆空，執行 speclink-review
- **THEN** 呈現開頭一行寫明未套用預設 12 條、沒有專案規則、Standards 軸只依 repo 文件審查
