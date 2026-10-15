## MODIFIED Requirements

### Requirement: 詳情頁的工單分頁

change 詳情頁 SHALL 於左欄「文件」節點「排程」之後提供兩個條件式節點：清單項 `reviewStatus` 為 `inReview` 時 SHALL 出現「審查」節點（tw「審查」、en「Review」），`verifyStatus` 為 `inVerify` 時 SHALL 出現「驗證」節點（tw「驗證」、en「Verify」），審查節點在前；兩者皆非時 SHALL NOT 渲染這兩個節點，文件節點維持提案／設計／任務／規格／排程。工單文件的閱讀卡 SHALL 為該站工單的結構化呈現：籤槽為「第 N 輪」（N 為末輪序號）與末輪階段詞籤（`phase` 為 `discovery` 顯示 tw「首輪」en「First pass」、`validation` 顯示 tw「複驗」en「Re-check」、`null` 不出階段詞籤）；數字磚依序為「輪」（輪數）、「CRITICAL（末輪）」、「WARNING（末輪）」、「SUGGESTION（末輪）」（末輪三級各自的條數，標籤維持英文）；末輪 CRITICAL 與 WARNING 皆為 0 時 SHALL 有 tip 提示框「可以蓋章」，內文為「末輪只剩 N 條 SUGGESTION，不擋章」（N 為 0 時「末輪無發現」）並點名以 CLI 或 agent 跑該站的 stamp 指令蓋章；否則提示框缺席；內文 SHALL NOT 另畫段標題列。輪次依序號升序逐輪一張卡（與排程文件同一種邊框卡片），末輪預設展開、其餘預設收合；卡頭為 16px 粗體的「第 N 輪」帶本頁目錄錨點、階段詞、「範圍 N 檔」、findings 條數與各嚴重度的計數籤（為零的不出）；展開後卡內先列「套用規則」列（僅該輪 `rules` 非 null 時，見下段），再列範圍路徑（等寬，原樣顯示、不做平台轉換），再列每條 finding：嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、軸向籤（描述以「Correctness」「Standards」「Coherence」加冒號開頭時取出該詞成籤，描述去掉前綴；無前綴時無籤、描述原樣）、路徑（等寬字）與描述原文（不翻譯、不截斷）；描述以結構 token `(accepted)` 結尾時 SHALL 去掉 token 並附「已接受」籤，token 出現於行中時 SHALL 原樣顯示；零 findings 的輪 SHALL 顯示「本輪無發現」。工單 SHALL 於節點出現時載入，並隨外部檔案變更重新載入；載入三態依「詳情頁文件載入以 skeleton 呈現」；載入完成而無工單（不存在、遠端 404、讀取或解析失敗）時 SHALL 顯示空態「工單尚未抵達或已被刪除」而非錯誤，籤槽、數字磚與提示框缺席。工單資料 SHALL 為 `{ rounds: [{ index, phase, patchHash, rules, scope, findings: [{ severity, path, text }] }], reviewBaselineSmells: string[] }`：`rounds` 的欄位名與型別與 CLI `speclink review show --json` 的 `rounds` 相同（`rules` 為 `{ baseline, critical, warning, suggestion }` 或 null）；`reviewBaselineSmells` 為 desktop 內建引擎提供的預設 smell 名稱（依審查技能範本順序），兩站的工單資料都帶此欄位；本機（含 worktree 覆蓋層）與遠端、活變更與已封存 SHALL 回同一形狀。遠端活變更 SHALL 以 `GET /changes/{name}/review`／`verify` 取得結構化輪次，SHALL NOT 放寬 `artifact cat` 的 artifact 名白名單。看板刷新後對應狀態翻為非進行中時，該節點 SHALL 消失；使用者正停在該文件時 SHALL 切回「提案」。工單文件 SHALL 為唯讀，SHALL NOT 提供蓋章、放棄或追加輪的操作。

「套用規則」列：輪次 `rules` 非 null 時，該輪卡展開後 SHALL 先渲染此列（tw「套用規則」、en「Rules applied」），再列範圍路徑；`rules` 為 null（舊工單、驗證站工單、複驗輪）時 SHALL NOT 渲染此列。列頭 SHALL 為帶 `aria-expanded` 的展開切換按鈕，與輪卡卡頭同一種 chevron 樣式，預設收合。收合時列頭依 baseline 與專案規則條數呈現模式文字與條數籤：條數籤以嚴重度色章呈現「CRITICAL 2」形式、條數為零的嚴重度不出；沒有專案規則時 SHALL NOT 顯示任何條數籤；baseline 為 replace 且三個清單皆空時，列 SHALL 以琥珀底（WARNING 語意色）呈現。展開後 SHALL 依序列出：baseline 為 extend 時，SUGGESTION 色章、「預設 N 條 code smell」與 N 個 smell 名稱（等寬字，取自 `reviewBaselineSmells`）；baseline 為 replace 時，一行「沒有套用預設 N 條 code smell」；接著每個非空的嚴重度一組——嚴重度色章、「專案規則 N 條」與規則原文清單（不翻譯、不截斷）；末尾一行註記「記錄首輪當時的設定；之後改了審查規則，這裡不會跟著變。」N 為 `reviewBaselineSmells` 的長度，SHALL NOT 寫死。smell 名稱清單 SHALL 與設定頁審查規則頁的名稱清單為同一個共用元件。

#### Scenario: 審查工單分頁出現並結構化呈現

- **WHEN** 開啟 `reviewStatus: "inReview"`、`verifyStatus: "none"` 的變更詳情頁，工單含 Round 1（`phase: "discovery"`，scope 3 檔，findings 兩條 WARNING）與 Round 2（`phase: "validation"`，scope 3 檔，findings 一條 WARNING 描述結尾 `(accepted)`、一條 SUGGESTION）
- **THEN** 左欄文件節點為提案／設計／任務／規格／排程／審查，無「驗證」節點；點「審查」後閱讀卡標題「審查」、籤槽「第 2 輪」「複驗」，數字磚「輪 2」「CRITICAL（末輪）0」「WARNING（末輪）1」「SUGGESTION（末輪）1」，無「可以蓋章」提示框；第 2 輪卡展開列兩條 finding，WARNING 那條附「已接受」籤且描述不含 `(accepted)`；第 1 輪卡收合、卡頭顯示「第 1 輪 · 首輪 · 範圍 3 檔 · 2 條」與「WARNING 2」計數籤

##### Example: 階段詞與 accepted token

| round.phase | 顯示階段詞（tw） | finding.text | 顯示描述 | 已接受籤 |
| ----------- | ---------------- | ------------ | -------- | -------- |
| discovery | 首輪 | Correctness: 未處理空清單 | 未處理空清單（軸向籤 Correctness） | 無 |
| validation | 複驗 | Standards: 命名不一致 (accepted) | 命名不一致（軸向籤 Standards） | 有 |
| null | （不顯示） | Standards: (accepted) 語意不清 | (accepted) 語意不清（軸向籤 Standards） | 無 |

#### Scenario: 兩站皆進行中出現兩個分頁

- **WHEN** 開啟 `reviewStatus: "inReview"` 且 `verifyStatus: "inVerify"` 的變更詳情頁
- **THEN** 左欄文件節點於「排程」後依序為「審查」「驗證」，各自載入對應站的工單，兩份文件內容互不混入

#### Scenario: 無工單時分頁缺席

- **WHEN** 開啟 `reviewStatus: "reviewed"`、`verifyStatus: "none"` 的變更詳情頁
- **THEN** 文件節點維持提案／設計／任務／規格／排程五個，無「審查」與「驗證」節點，且未發出工單讀取請求

#### Scenario: 工單被刪時分頁退場

- **WHEN** 使用者停在 `inReview` 變更的「審查」文件，其後工單被蓋章刪除、看板刷新後該變更 `reviewStatus` 翻為 `reviewed`
- **THEN** 「審查」節點自左欄消失，詳情頁切回「提案」文件，「狀態」列「品質關卡」列的章籤改為「已審查」

#### Scenario: 遠端工單 404 顯示空態

- **WHEN** 於 remote 資料源開啟 `inReview` 變更詳情頁並點「審查」節點，server 對 `GET /changes/{name}/review` 回 404
- **THEN** 節點仍在，skeleton 消失後顯示「工單尚未抵達或已被刪除」，無錯誤彈窗，籤槽與數字磚缺席

#### Scenario: 工單追加輪後分頁更新

- **WHEN** 「審查」文件開啟中，外部以 `speclink review add-round` 追加 Round 3
- **THEN** 檔案變更觸發重新載入，籤槽改為「第 3 輪」，數字磚「輪 3」，第 3 輪卡展開、第 2 輪與第 1 輪收合

#### Scenario: 可蓋章提示框與軸向籤

- **WHEN** 開啟 `inReview` 變更的「審查」文件，末輪為 Round 2（`validation`）只有一條 SUGGESTION，描述為「Standards: possible Duplicated Code。讀寫加 try/catch 的寫法…」
- **THEN** tip 提示框標題「可以蓋章」、內文「末輪只剩 1 條 SUGGESTION，不擋章」並點名 `speclink review stamp`；該條 finding 顯示 SUGGESTION 色章、「Standards」軸向籤、等寬路徑與去掉前綴的描述；末輪 CRITICAL 或 WARNING 任一非零時提示框缺席

#### Scenario: 沒有專案規則時套用規則列只寫預設清單

- **WHEN** 開啟 `inReview` 變更的「審查」文件，Round 1（`discovery`）的 `rules` 為 `{ baseline: "extend", critical: [], warning: [], suggestion: [] }`，`reviewBaselineSmells` 有 12 個名稱，展開第 1 輪卡
- **THEN** 範圍路徑之前有一列收合的「套用規則」，列頭文字為「預設 12 條 code smell」，沒有任何條數籤；點列頭展開後顯示 SUGGESTION 色章、「預設 12 條 code smell」、12 個 smell 名稱與末尾註記，沒有任何「專案規則」組

#### Scenario: 套用規則列的四種呈現

- **WHEN** 開啟 `inReview` 變更的「審查」文件，Round 1 的 `rules` 分別為下表四種，`reviewBaselineSmells` 有 12 個名稱
- **THEN** 收合時列頭文字、條數籤與底色如下表；展開後 extend 列出 12 個名稱、replace 寫「沒有套用預設 12 條 code smell」，再列非空嚴重度的規則原文組與末尾註記

##### Example: 列頭呈現

| rules | 列頭文字（tw） | 列頭文字（en） | 條數籤 | 底色 |
| ----- | -------------- | -------------- | ------ | ---- |
| extend，三清單皆空 | 預設 12 條 code smell | 12 baseline code smells | 無 | 一般 |
| extend，critical 2、warning 1、suggestion 0 | 預設 12 條＋專案規則 | 12 baseline smells + project rules | CRITICAL 2、WARNING 1 | 一般 |
| replace，suggestion 1 | 只用專案規則（沒有套用預設 12 條） | Project rules only (baseline not applied) | SUGGESTION 1 | 一般 |
| replace，三清單皆空 | 整份換掉且沒有專案規則：只依 repo 文件審查 | Baseline replaced, no project rules: judged against repo documents only | 無 | 琥珀 |

#### Scenario: 展開後列出專案規則原文

- **WHEN** Round 1 的 `rules` 為 `{ baseline: "extend", critical: ["所有 SQL 一律用參數化查詢，不得用字串拼接", "不得把金鑰或密碼寫死在程式碼裡"], warning: ["新增的公開 API 要有對應的測試"], suggestion: [] }`，使用者展開「套用規則」列
- **THEN** 依序顯示 12 個 smell 名稱、CRITICAL 色章與「專案規則 2 條」及兩條原文、WARNING 色章與「專案規則 1 條」及一條原文，沒有 SUGGESTION 組，末尾為「記錄首輪當時的設定；之後改了審查規則，這裡不會跟著變。」

#### Scenario: 沒有規則記錄時不顯示套用規則列

- **WHEN** 開啟「審查」文件，工單所有輪的 `rules` 皆為 null（審查技能升版前寫的工單）；或開啟「驗證」文件
- **THEN** 任何輪卡展開後都直接從範圍路徑開始，沒有「套用規則」列
