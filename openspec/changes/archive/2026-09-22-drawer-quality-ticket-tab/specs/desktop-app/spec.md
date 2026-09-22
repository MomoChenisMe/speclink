## ADDED Requirements

### Requirement: 詳情抽屜的工單分頁

change 詳情抽屜 SHALL 於「排程」分頁之後提供兩個條件式分頁：清單項 `reviewStatus` 為 `inReview` 時 SHALL 出現「審查」分頁（tw「審查」、en「Review」），`verifyStatus` 為 `inVerify` 時 SHALL 出現「驗證」分頁（tw「驗證」、en「Verify」），審查分頁在前；兩者皆非時 SHALL NOT 渲染這兩個分頁，分頁列維持提案／設計／任務／規格／排程。分頁內容 SHALL 為該站工單的結構化呈現：段標題含站名、「第 N 輪」（N 為末輪序號）、末輪階段詞與末輪三級計數（CRITICAL／WARNING／SUGGESTION 各自的條數，標籤維持英文）；輪次依序號升序逐輪可收合，末輪預設展開、其餘預設收合；每輪標題含「第 N 輪」、階段詞（`phase` 為 `discovery` 顯示 tw「首輪」en「First pass」、`validation` 顯示 tw「複驗」en「Re-check」、`null` 不顯示階段詞）、「範圍 N 檔」（展開後列出 scope 路徑，原樣顯示、不做平台轉換）與 findings 條數；每條 finding 一列，含嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、路徑（等寬字）與描述原文（不翻譯、不截斷）；描述以結構 token `(accepted)` 結尾時 SHALL 去掉 token 並附「已接受」籤，token 出現於行中時 SHALL 原樣顯示；零 findings 的輪 SHALL 顯示「本輪無發現」。工單 SHALL 於分頁出現時載入，並隨外部檔案變更重新載入；載入三態依「抽屜文件載入以 skeleton 呈現」；載入完成而無工單（不存在、遠端 404、讀取或解析失敗）時 SHALL 顯示分頁空態「工單尚未抵達或已被刪除」而非錯誤。工單資料 SHALL 為 `{ rounds: [{ index, phase, patchHash, scope, findings: [{ severity, path, text }] }] }`，欄位名與型別與 CLI `speclink review show --json` 的 `rounds` 相同；本機（含 worktree 覆蓋層）與遠端、活變更與已封存 SHALL 回同一形狀。遠端活變更 SHALL 以 `GET /changes/{name}/review`／`verify` 取得結構化輪次，SHALL NOT 放寬 `artifact cat` 的 artifact 名白名單。看板刷新後對應狀態翻為非進行中時，該分頁 SHALL 消失；使用者正停在該分頁時 SHALL 切回「提案」分頁。分頁 SHALL 為唯讀，SHALL NOT 提供蓋章、放棄或追加輪的操作。

#### Scenario: 審查工單分頁出現並結構化呈現

- **WHEN** 開啟 `reviewStatus: "inReview"`、`verifyStatus: "none"` 的變更抽屜，工單含 Round 1（`phase: "discovery"`，scope 3 檔，findings 兩條 WARNING）與 Round 2（`phase: "validation"`，scope 3 檔，findings 一條 WARNING 描述結尾 `(accepted)`、一條 SUGGESTION）
- **THEN** 分頁列為提案／設計／任務／規格／排程／審查，無「驗證」分頁；切到「審查」分頁後段標題顯示「審查 · 第 2 輪（複驗）CRITICAL 0 · WARNING 1 · SUGGESTION 1」；第 2 輪展開列兩條 finding，WARNING 那條附「已接受」籤且描述不含 `(accepted)`；第 1 輪收合、標題顯示「第 1 輪 · 首輪 · 範圍 3 檔 · 2 條」

##### Example: 階段詞與 accepted token

| round.phase | 顯示階段詞（tw） | finding.text | 顯示描述 | 已接受籤 |
| ----------- | ---------------- | ------------ | -------- | -------- |
| discovery | 首輪 | Correctness: 未處理空清單 | Correctness: 未處理空清單 | 無 |
| validation | 複驗 | Standards: 命名不一致 (accepted) | Standards: 命名不一致 | 有 |
| null | （不顯示） | Standards: (accepted) 語意不清 | Standards: (accepted) 語意不清 | 無 |

#### Scenario: 兩站皆進行中出現兩個分頁

- **WHEN** 開啟 `reviewStatus: "inReview"` 且 `verifyStatus: "inVerify"` 的變更抽屜
- **THEN** 分頁列於「排程」後依序為「審查」「驗證」，各自載入對應站的工單，兩分頁內容互不混入

#### Scenario: 無工單時分頁缺席

- **WHEN** 開啟 `reviewStatus: "reviewed"`、`verifyStatus: "none"` 的變更抽屜
- **THEN** 分頁列維持提案／設計／任務／規格／排程五個，無「審查」與「驗證」分頁，且未發出工單讀取請求

#### Scenario: 工單被刪時分頁退場

- **WHEN** 使用者停在 `inReview` 變更的「審查」分頁，其後工單被蓋章刪除、看板刷新後該變更 `reviewStatus` 翻為 `reviewed`
- **THEN** 「審查」分頁自分頁列消失，抽屜切回「提案」分頁，狀態列章籤改為「已審查」

#### Scenario: 遠端工單 404 顯示空態

- **WHEN** 於 remote 資料源開啟 `inReview` 變更抽屜並切到「審查」分頁，server 對 `GET /changes/{name}/review` 回 404
- **THEN** 分頁仍在，skeleton 消失後顯示「工單尚未抵達或已被刪除」，無錯誤彈窗

#### Scenario: 工單追加輪後分頁更新

- **WHEN** 「審查」分頁開啟中，外部以 `speclink review add-round` 追加 Round 3
- **THEN** 檔案變更觸發重新載入，段標題改為「第 3 輪」，第 3 輪展開、第 2 輪與第 1 輪收合

### Requirement: 已封存抽屜的工單分頁

已封存變更抽屜 SHALL 於「規格」分頁之後提供兩個條件式分頁：封存清單項 `reviewStatus` 為 `reviewedNotPassed` 時 SHALL 出現「審查」分頁，`verifyStatus` 為 `verifiedNotPassed` 時 SHALL 出現「驗證」分頁，審查在前；其他狀態（`none`、`reviewed`、`verified`）SHALL NOT 渲染對應分頁。分頁內容、載入三態、空態文案與工單資料形狀 SHALL 與「詳情抽屜的工單分頁」相同，資料來自封存目錄內的 `review.md`／`verify.md`（本機讀封存目錄；遠端以 `GET /archived/{datedName}/artifacts/review.md` 或 `verify.md` 取原文後解析）。分頁 SHALL 為唯讀。

#### Scenario: 曾審查未通過的封存抽屜顯示審查分頁

- **WHEN** 開啟 `reviewStatus: "reviewedNotPassed"`、`verifyStatus: "verified"` 的封存變更抽屜
- **THEN** 分頁列為提案／設計／任務／規格／審查，無「驗證」分頁；「審查」分頁以結構化方式呈現封存目錄內 review.md 的各輪，末輪展開

#### Scenario: 蓋章封存的抽屜無工單分頁

- **WHEN** 開啟 `reviewStatus: "reviewed"`、`verifyStatus: "none"` 的封存變更抽屜
- **THEN** 分頁列維持提案／設計／任務／規格四個，未發出工單讀取請求

#### Scenario: 遠端封存工單原文解析

- **WHEN** 於 remote 資料源開啟 `verifiedNotPassed` 的封存變更抽屜並切到「驗證」分頁，server 對 `GET /archived/{datedName}/artifacts/verify.md` 回傳工單原文
- **THEN** 分頁呈現與本機相同形狀的結構化輪次（`rounds[].index`、`phase`、`patchHash`、`scope`、`findings[].severity`／`path`／`text`）
