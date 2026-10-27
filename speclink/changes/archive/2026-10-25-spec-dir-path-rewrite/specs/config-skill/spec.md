## MODIFIED Requirements

### Requirement: 技能規定固定輸入來源與四條內容判準

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

渲染產出的 speclink-config 技能檔 SHALL 規定：掃描輸入限固定的結構性來源——workspace 清單與相依 manifest（含 Cargo workspace 成員與 workspace 相依、關鍵邊界相依、各 package 的相依清單）、README、docs 索引、既有 speclink/config.yaml、以及 speclink language show 的共用詞彙（若有）；SHALL NOT 全 repo 掃描原始碼。技能檔 SHALL 載明四條內容判準：(1) 已由政策開關或 schema 內建 instruction 自動注入的內容，以及品質關卡技能已承載的正典標準（如審查站的 smell baseline），context 與 rules 皆不得重述——引擎注入內容的判定 SHALL 以 speclink instructions <artifact> --json 取得的實際 payload 逐條反證，品質關卡正典的判定 SHALL 對照生成的品質關卡技能檔內容，皆不得憑印象；(2) 只對單一 artifact 咬合的內容歸 rules，不入 context；(3) 會過時的內容（版本號、計數、統計數字）不寫；(4) context 與 rules 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行皆核實——核實 SHALL 以靜態便宜手段進行（路徑查檔案系統、測試名以文字搜尋命中原始碼、npm script 查 package.json 宣告、CLI 子指令對照 --help 輸出），SHALL NOT 執行被引用的測試或建置指令；判準一的 speclink instructions <artifact> --json payload 探測不受此限。

技能檔 SHALL 載明刪除理由限定：一條既有 rule 只因不過四條判準、或使用者本人於政策詢問中明確撤回而被刪除，SHALL NOT 因「無法自固定輸入來源導出」而被刪除——使用者裁決後落地的 rule（如討論結論轉入者）因此不需任何標記即受保護，且落地裁決與撤回裁決同源。

技能檔 SHALL 載明 scope hint 的收窄語意：呼叫帶範圍提示時，判準一至三的全面重審收窄至範圍內的 artifacts；判準四的引用核實恆為全文件掃描；未帶範圍提示時維持全文件重審。

#### Scenario: 渲染產物含固定來源清單

- **WHEN** 檢視渲染產出的 speclink-config 技能檔
- **THEN** 技能檔載明固定輸入來源清單（manifest、README、docs 索引、既有 config.yaml、language show），並明示不做全 repo 原始碼掃描

#### Scenario: 渲染產物含四條判準與反證步驟

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準段落
- **THEN** 四條判準俱在，且判準一明定以 speclink instructions <artifact> --json 的 payload 逐條反證引擎注入內容、明定品質關卡技能承載的正典標準對照生成技能檔反證、判準四明定引用存在性以靜態手段核實

#### Scenario: 品質關卡正典不得重述進 rules

- **WHEN** 候選 rules 條目與品質關卡技能檔內嵌的正典標準同義（如要求避免特定 code smell）
- **THEN** 技能檔的判準指引該條目被淘汰，正典標準維持品質關卡技能檔單一落點

#### Scenario: 判準四核實不執行引用指令

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準四段落
- **THEN** 段落明定驗證引用以靜態手段（檔案系統、文字搜尋、package.json 宣告、--help 對照）進行、明文禁止執行被引用的測試或建置指令，並明示判準一的 payload 探測不在禁令範圍

#### Scenario: 使用者裁決型 rule 不因來源被刪

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於既有 rule 汰留的段落
- **THEN** 段落明定 rule 只因不過四條判準或使用者明確撤回而被刪，「無法自固定輸入來源導出」不構成刪除理由

#### Scenario: scope hint 收窄語意

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於範圍提示的段落
- **THEN** 段落明定範圍提示收窄判準一至三的重審至範圍內 artifacts、判準四恆為全文件掃描、未帶提示時全文件重審

### Requirement: 技能規定提醒的分流

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

渲染產出的 speclink-config 技能檔 SHALL 規定使用者提醒（使用者希望專案遵守的規矩）的處理：

- **來源**：提醒 SHALL 來自使用者——口述，或使用者提供的提醒清單（貼在對話中，或給一個檔案路徑由技能讀取）。提醒清單是使用者的輸入，不屬「固定輸入來源」的掃描範圍；技能 SHALL NOT 自行讀取任何工具的對話紀錄尋找提醒。
- **三題分流**：每條提醒 SHALL 依三題判斷去處——違反的證據在程式還是文件、審查員只看 patch 能不能判斷、太晚發現的代價大不大——分到四個去處之一：審查規則、某個 artifact 的產出規則（rules）、兩邊都放、或不收。
- **審查規則的出口只在選用時開放**：審查規則的模式為附加或覆蓋時，四個去處都開放；模式為不設定時，分到審查規則的提醒 SHALL NOT 寫入，SHALL 在最後報告列為「屬審查規則，未寫入」，分到兩邊都放的提醒只落地產出規則那一句。
- **兩邊都放的條件**：技能 SHALL 只在兩個階段各自有用時才兩邊放，其餘選一邊——產出規則每次產出該 artifact 都會注入。兩邊的兩句 SHALL 描述不同對象：產出規則要求計畫寫出什麼，審查規則檢查程式本身。
- **審查規則不寫 verify 的工作**：審查規則 SHALL NOT 寫成「程式要照 artifact 做」，那是 verify 關卡的工作。
- **產出規則的搬移建議**：模式為附加或覆蓋時，既有產出規則中某句其實在檢查程式本身，技能 SHALL 建議把該部分拆到審查規則；搬移 SHALL 經使用者確認才落地，被拒絕時原句保留（刪除理由限定照舊）。技能 SHALL NOT 記錄被拒絕的建議，下次執行再次提出屬預期行為。模式為不設定時，技能 SHALL NOT 提出搬移建議。
- **時機**：提醒依賴的行為尚未進程式（例如討論已定案、對應的變更尚未實作）時，技能 SHALL NOT 寫入該提醒，SHALL 告知使用者並在最後報告列出。需要查討論或變更的狀態時，技能 SHALL 用 speclink discuss list --json 或 speclink list --json，SHALL NOT 直接讀取 speclink/ 目錄。
- **各自的判準**：分到產出規則的提醒 SHALL 照四條內容判準檢查，分到審查規則的提醒 SHALL 照「技能規定審查規則的第六問與規則接收」的檢查。
- **報告**：最後報告 SHALL 逐條列出每條提醒的去處；不收或未寫入者 SHALL 附理由（不過判準、重述 smell、已有測試或 lint 守門、屬其他關卡、尚未進程式、屬審查規則未寫入）。

#### Scenario: 渲染產物含三題與四個去處

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）關於提醒分流的段落
- **THEN** 段落載明三題、四個去處、兩邊都放的條件、審查規則不寫 verify 的工作，以及模式為不設定時審查規則出口關閉

##### Example: 分流結果

| 提醒 | 判斷 | 模式為附加時的去處 | 模式為不設定時的去處 |
| ---- | ---- | ------------------ | -------------------- |
| 呼叫端不得用 className 覆蓋共用元件的顏色 | 證據在程式，只看 patch 就能判斷 | 審查規則 | 報告列為「屬審查規則，未寫入」 |
| design 要列出這次沿用的既有元件 | 證據在文件 | design 的產出規則 | design 的產出規則 |
| 共通行為要在共用層實作一次 | 計畫要寫出落點、程式要檢查，兩個階段各自有用 | 兩邊都放，兩句描述不同對象 | 只落地 design 的產出規則 |

#### Scenario: 從檔案讀入提醒清單

- **WHEN** 使用者執行技能時給一個提醒清單的檔案路徑
- **THEN** 技能讀取該檔，每條提醒各自分流，最後報告逐條列出去處；技能不讀取任何工具的對話紀錄

#### Scenario: 不設定時不提搬移建議

- **WHEN** 審查規則的模式為不設定，既有的 design 產出規則中有一句在檢查程式本身
- **THEN** 技能不提出搬移建議，該句維持原樣

#### Scenario: 搬移建議被拒絕後下次重提

- **WHEN** 模式為附加，使用者拒絕把某句產出規則拆到審查規則
- **THEN** 原句保留、不產生 diff；對同一 codebase 再次執行時建議再次提出，收斂驗收不受影響（建議不是 diff）

#### Scenario: 依賴未落地行為的提醒先不寫入

- **WHEN** 某條提醒依賴的行為由已定案的討論決定，但對應的變更尚未實作
- **THEN** 技能不寫入該提醒，告知使用者並在最後報告列出；查詢狀態只用 speclink discuss list --json 或 speclink list --json
