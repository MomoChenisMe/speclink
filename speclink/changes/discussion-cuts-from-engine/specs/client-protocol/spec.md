## ADDED Requirements

### Requirement: 討論詳情回應的分刀 payload

protocol 的討論詳情回應（GET /discussions/{slug}）SHALL 在既有的 `info` 與 `content` 之外，於頂層增列 `cuts` 陣列，依刀在結論中的順序，每項含：`cut`（刀的識別籤原文）、`title`（範圍句，沒有時為空字串）、`note`（首行其餘的說明：範圍句取自粗體時為粗體之後去掉開頭冒號、外圍整組括號拿掉的文字，否則為第一個「——」之後的文字，都沒有時為空字串）、`line`（該刀列點去掉 `- ` 與頭尾空白的首行原文）四個字串，`changes`（變更名籤）與 `unpromoted`（變更名籤中不在該記錄 `promoted_to` 的名稱，依 `changes` 順序）兩個字串陣列，以及 `changesInBold`（變更名籤是否寫在粗體內）布林。欄位一律 camelCase。各欄的值 SHALL 由引擎自同一份記錄的結論與 `promoted_to` 計算，判定規則同 change-plan「plan 列出討論結論中尚未轉出的刀」；`changesInBold` 為 false 時 `unpromoted` SHALL 為空陣列。記錄沒有結論或結論沒有刀時 `cuts` SHALL 為空陣列。

typed client 反序列化時，`cuts` 缺席 SHALL 讀作空陣列，每項缺席的字串欄位 SHALL 讀作空字串、陣列欄位讀作空陣列、`changesInBold` 讀作 false，SHALL NOT 報錯；JSON Schema 匯出 SHALL 隨 Rust 型別更新。桌面 local 的討論記錄回應 SHALL 為 `{ content, cuts }`，`cuts` 每項與本需求同形同值；remote 的討論記錄回應 SHALL 自詳情回應逐欄搬運 `content` 與 `cuts`。CLI `speclink discuss show --json` 在 fs 與 remote 模式 SHALL NOT 包含 `cuts`，人眼與 `--json` 輸出 SHALL 與本需求引入前逐位元一致。

#### Scenario: 詳情回應帶分刀

- **WHEN** 討論 panel 的 promoted_to 為 `a`，結論的 Decision 有下表三條頂層列點，client 呼叫 GET /discussions/panel
- **THEN** 回應的 `cuts` 依序為三項，各欄如下表，每項的 `line` 為表中第一欄的原文；`info` 與 `content` 與本需求引入前相同

##### Example: 分刀各欄

| 列點首行（去掉 `- `） | cut | title | note | changes | changesInBold | unpromoted |
| --- | --- | --- | --- | --- | --- | --- |
| ``**cut 1 `a`**: 骨架——三種詳情頁改三欄`` | cut 1 | 骨架 | 三種詳情頁改三欄 | a | true | （空） |
| ``**cut 2 外殼**（拆兩個 change：`b` 外殼、`c` 選單）`` | cut 2 | 外殼 | ``拆兩個 change：`b` 外殼、`c` 選單`` | b、c | false | （空） |
| ``**cut 3 `d` `e`**: 收尾`` | cut 3 | 收尾 | （空字串） | d、e | true | d、e |

#### Scenario: 舊 server 缺 cuts 讀作空陣列

- **WHEN** server 的討論詳情回應只有 `info` 與 `content` 兩鍵
- **THEN** typed client 反序列化成功，`cuts` 為空陣列；桌面的討論記錄回應 `cuts` 為空陣列

#### Scenario: CLI 的 discuss show 不含 cuts

- **WHEN** 分別以 fs 模式與 remote 模式對同一份含刀的記錄執行 `speclink discuss show panel --json`
- **THEN** 兩次輸出都沒有 `cuts` 鍵，鍵集合彼此相同，且與本需求引入前逐位元一致

## MODIFIED Requirements

### Requirement: 討論搜尋回應 payload

<!-- BEFORE: 討論資訊 payload 與清單、詳情回應的欄位全部逐位元不變 -->

protocol SHALL 以 Rust 型別定義討論搜尋回應（JSON Schema 為匯出）：頂層 hits 陣列，每筆含與討論資訊 payload 相同的欄位（slug、topic、status、rounds、created、createdBy 選填、kind 選填、promotedTo、concluded 選填、path、archived）加 matches 陣列；每個 match 含 kind、where、text 三個字串欄位，kind 值為 topic、slug、ruled-out、decision、rejected、deferred 之一，where 值為 frontmatter、round-N 或 conclusion。欄位一律 camelCase。typed client SHALL 提供搜尋方法，以關鍵字清單組成空白分隔的 q 呼叫 GET /discussions/search，回應反序列化為 typed 型別，SHALL NOT 走 raw JSON 旁路。既有討論資訊 payload 與清單回應的欄位 SHALL 逐位元不變；詳情回應除「討論詳情回應的分刀 payload」增列的 `cuts` 外 SHALL 逐位元不變。

#### Scenario: typed client 讀取搜尋回應

- **WHEN** typed client 以關鍵字 golden 與 sse 呼叫搜尋方法，server 回一筆命中（matches 含 kind 為 deferred、where 為 conclusion 的項目）
- **THEN** 請求路徑為 /discussions/search 且 q 為 `golden sse`；回應反序列化成功，該筆的 slug、archived 與 matches 三欄位值與 JSON 一致，缺席的 createdBy 與 kind 為 None

#### Scenario: 選填欄位缺席時仍可反序列化

- **WHEN** server 回應的命中項目缺 createdBy、kind、concluded 三個鍵，promotedTo 為空陣列
- **THEN** typed client 反序列化成功，三個選填欄位為 None、promotedTo 為空清單，不報錯
