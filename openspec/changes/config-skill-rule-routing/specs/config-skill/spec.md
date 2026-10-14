## MODIFIED Requirements

### Requirement: 技能規定 diff 先行與收斂驗收

<!-- BEFORE: 政策四欄（locale、spec_locale、tdd、audit）逐項詢問使用者 -->

渲染產出的 speclink-config 技能檔 SHALL 規定執行流程：整理結果 SHALL 先以 workflow-config 動詞的 --dry-run 產出 unified diff 呈現給使用者，經使用者確認後才執行實際寫入；SHALL NOT 未經確認直接寫入。政策欄位 SHALL 依「技能規定政策欄位一次確認現值」的問法由使用者作答、不由技能推斷。技能檔 SHALL 載明收斂驗收：對同一未變動的 codebase 連續執行兩次，第二次產出的 diff SHALL 為空——第二次仍有 diff 即為判準執行不當，SHALL 回查判準而非落檔。

#### Scenario: 渲染產物規定 dry-run 先行

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的執行流程段落
- **THEN** 流程明定先以 --dry-run 產 diff、經使用者確認後才寫入，且政策欄位以一次確認現值的問法詢問、不由技能推斷

#### Scenario: 渲染產物載明收斂驗收

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的驗收段落
- **THEN** 載明同一未變動 codebase 連跑兩次、第二次 diff 為空的驗收條件，及第二次仍有 diff 時回查判準的處置

### Requirement: 技能規定任務驗證測試範圍的第五問

<!-- BEFORE: 第五問與政策四欄同樣逐項詢問；已有測試範圍 rule 時提問帶現值 -->

渲染產出的 speclink-config 技能檔 SHALL 於政策詢問流程納入第五問：任務清單（tasks）的驗證步驟要包含全量測試，或只跑受影響面的測試——與其餘政策欄位同性質，SHALL 列入「技能規定政策欄位一次確認現值」的確認表、由使用者作答，SHALL NOT 自 repo 推斷；現行文件已有測試範圍相關 rule 時，確認表 SHALL 以該 rule 帶出現值，無既有 rule 時 SHALL 標為未設定、預設全量。使用者沿用現值時，技能 SHALL NOT 改動測試範圍 rule。使用者點名修改此項並答「只跑受影響面」時，技能檔 SHALL 指引自已讀取的 dependency manifests 組出該專案客製的對應規則（按專案的組件型態對應其測試指令）寫入 rules 的 tasks 段，並沿既有 dry-run 核准流程落地；答「全量」時 SHALL NOT 寫入任何測試範圍規則——現行文件已有測試範圍 rule 時，該答案即為使用者對其之明確撤回，技能檔 SHALL 指引沿同一 dry-run 核准流程移除之，無既有 rule 時現行文件維持原樣。

#### Scenario: 渲染產物含第五問

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）的政策詢問段落
- **THEN** 段落載明任務驗證測試範圍的第五問、與其餘政策欄位同樣列入確認表且不得推斷、以及現行文件已有測試範圍 rule 時以該 rule 帶出現值

#### Scenario: 答受影響面時組出客製規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於第五問後續處理的段落
- **THEN** 段落明定自已讀取的 dependency manifests 組出專案客製的測試對應規則、寫入 rules 的 tasks 段、經 dry-run 核准後落地

#### Scenario: 答全量時不寫規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於第五問後續處理的段落
- **THEN** 段落明定使用者選擇全量時不寫入任何測試範圍規則；現行文件已有測試範圍 rule 時視為明確撤回、經 dry-run 核准移除該 rule，無既有 rule 時現行文件維持原樣

#### Scenario: 沿用現值不動測試範圍規則

- **WHEN** 現行文件的 tasks 段已有測試範圍 rule，使用者在確認表選全部沿用
- **THEN** 技能不再詢問第五問，該 rule 原樣保留，rules 的 dry-run diff 不含測試範圍 rule 的改動

### Requirement: 技能規定審查規則的第六問與規則接收

<!-- BEFORE: 第六問逐項詢問 review.baseline、另問是否新增規則；口述規則依語氣建議分格；只淘汰重述 smell 的規則 -->

<!-- REMOVED-SCENARIO: 依語氣建議分格 -->

渲染產出的 speclink-config 技能檔 SHALL 規定技能也管理 config 的 review 區塊，並載明審查規則是選用的、以及以下行為：

- **第六問：審查規則的模式，一題三選**：技能 SHALL 把 review.baseline 與「要不要設定專案審查規則」合併為一題三選，列入「技能規定政策欄位一次確認現值」的確認表，由使用者作答，SHALL NOT 自 repo 推斷：
  - **不設定**（預設）：技能 SHALL NOT 改動 review 區塊（不寫 baseline、不新增規則）；提醒 SHALL NOT 分流到審查規則，技能 SHALL NOT 建議把產出規則的句子搬到審查規則；只適合當審查規則的提醒 SHALL 在最後報告列為「屬審查規則，未寫入」。
  - **附加**（extend）：審查站照舊檢查內建的 12 條 code smell，再加專案審查規則。
  - **覆蓋**（replace）：審查站只檢查專案審查規則；三個清單都為空時，Standards 軸只依 repo 文件判斷，這是正當的選擇。
  - **現值**：config 沒有 review 區塊時，確認表 SHALL 顯示為未設定、預設不設定；已有 review 區塊時 SHALL 帶出現值（模式與三個清單各自的條數）。已有審查規則而使用者選不設定或沿用現值時，技能 SHALL 維持 review 區塊現狀——既有規則不因此被移除（移除只經使用者明確撤回），且仍照下方「既有與新收審查規則的檢查」中會過時內容與引用核實兩項檢查。
- **不自 repo 推導審查規則**：技能 SHALL NOT 從固定輸入來源或 repo 文件（CLAUDE.md、AGENTS.md、CONTRIBUTING、lint 設定等）整理出審查規則——審查站的 Standards 軸本身會讀這些文件，複製一份會成為第二份正典。審查規則 SHALL 只來自使用者口述或使用者提供的提醒清單，並經「技能規定提醒的分流」決定去處。
- **規則接收與嚴重度**：模式為附加或覆蓋時，技能 SHALL 詢問使用者是否要新增或修改審查規則。每條要落入審查規則的規則，技能 SHALL 以「有沒有正當例外」建議清單——違反一定錯、沒有正當例外者建議 critical；通常要修、偶有正當例外者建議 warning；屬偏好者建議 suggestion。規則的措辭（如「不得」「一律」「要」「宜」「偏好」）SHALL 只作提示，SHALL NOT 單獨決定分格；分格 SHALL 由使用者確認或改分格後才落地。
- **既有與新收審查規則的檢查**：每條既有與新收的審查規則 SHALL 照以下檢查：
  - baseline 為 extend（含未設定）時，與生成的 speclink-review 技能檔內嵌 smell baseline 同義的規則 SHALL 被淘汰；baseline 為 replace 時不做此項檢查。
  - 已有測試或 lint 守門的規矩 SHALL 不收：核實 SHALL 以判準四同類的針對性靜態查核進行（在 lint 設定中查該規則、以文字搜尋找到釘住該行為的測試），SHALL NOT 為此通讀原始碼；查不到守門時照常收。
  - 屬於其他關卡的規矩 SHALL 不收：audit 為開啟時，安全類規矩歸 audit 關卡；要求程式照 artifact 實作的規矩歸 verify 關卡。audit 的開關以本次確認表確認後的值為準；audit 由關改開時，既有的安全類審查規則 SHALL 經 dry-run diff 提出移除，使用者確認後才落地。
  - 會過時的內容（版本號、計數、統計數字）不寫；引用的路徑、測試名、指令 SHALL 以靜態手段核實存在。
  - 審查規則的刪除理由限定與 rules 相同：只因不過上列檢查或使用者明確撤回而刪。
- **落地**：審查規則與 baseline 的寫入 SHALL 經 speclink workflow-config review 子指令的 --dry-run 產出 diff、經使用者確認後才實際寫入，與其餘寫入同一核准流程；收斂驗收（同一未變動 codebase 第二次執行 diff 為空）同樣涵蓋 review 區塊。

#### Scenario: 渲染產物含第六問

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）的政策詢問段落
- **THEN** 段落載明審查規則的模式為不設定、附加、覆蓋三選一，列入確認表並帶現值，不得自 repo 推斷，且不設定為預設

#### Scenario: 不自 repo 推導審查規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於審查規則的段落
- **THEN** 段落明定技能不從固定輸入來源或 repo 文件整理審查規則，只接收使用者口述或使用者提供的提醒清單

#### Scenario: 選不設定時不動 review 區塊

- **WHEN** config 沒有 review 區塊，使用者在確認表沿用「不設定」，並提供一條只適合當審查規則的提醒
- **THEN** dry-run diff 不含 review 區塊的任何改動；最後報告把該提醒列為「屬審查規則，未寫入」

#### Scenario: 已有規則時選不設定維持現狀

- **WHEN** review 區塊已有兩條 warning 規則，其中一條引用的測試名在 repo 中已搜尋不到，使用者在確認表選不設定
- **THEN** 引用有效的那條規則保留；引用失效的那條經 dry-run diff 提出移除，使用者確認後才寫入

#### Scenario: 依正當例外建議分格

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於規則接收的段落
- **THEN** 段落載明以有沒有正當例外建議 critical、warning、suggestion，措辭只作提示，且分格由使用者確認後才落地

##### Example: 分格建議

| 使用者的規則 | 有沒有正當例外 | 技能建議的清單 |
| ------------ | -------------- | -------------- |
| 公開函式在錯誤路徑不得 panic，一律回傳錯誤 | 沒有 | critical |
| 元件一律使用設計 token 的顏色 | 有：圖表配色需要原始色碼 | warning（措辭雖為「一律」） |
| 新增的公開 API 要有對應的測試 | 有：純 re-export 不需要 | warning |
| 錯誤訊息偏好寫成完整句子 | 屬偏好 | suggestion |

#### Scenario: extend 時淘汰重述 smell 的規則

- **WHEN** baseline 為 extend，使用者口述「變數名稱要能說明用途」（與 smell baseline 的 Mysterious Name 同義）
- **THEN** 技能檔的判準指引該規則被淘汰並向使用者說明理由；baseline 為 replace 時同一條規則可以落地

#### Scenario: 已有測試或 lint 守門的規矩不收

- **WHEN** 模式為附加，使用者提供的規矩已有測試釘住（該測試名以文字搜尋命中測試檔）
- **THEN** 該規矩不寫入審查規則，最後報告以該測試名作為不收的理由

#### Scenario: 屬於其他關卡的規矩不收

- **WHEN** 模式為附加，使用者提供的規矩屬安全類或要求程式照 artifact 實作
- **THEN** 技能依 audit 的開關與規矩性質判定去處，判為其他關卡者不寫入審查規則並說明歸屬

##### Example: 其他關卡的判定

| 使用者的規矩 | audit | 結果 |
| ------------ | ----- | ---- |
| 不得把金鑰或密碼寫死在程式碼裡 | 開啟 | 不收，歸 audit |
| 不得把金鑰或密碼寫死在程式碼裡 | 關閉 | 照常判斷，可以收為審查規則 |
| 程式要照 design 的模組切分實作 | 開啟或關閉 | 不收，歸 verify |

#### Scenario: audit 改開時移除既有安全類規則經 diff

- **WHEN** audit 原為關閉，review.critical 有一條安全類規則，使用者在確認表把 audit 改為開啟
- **THEN** 技能產出的 dry-run diff 同時含 audit 改為 true 與該條規則的移除，使用者確認後才寫入

#### Scenario: 寫入經 dry-run 核准

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的寫入段落
- **THEN** 段落列出 speclink workflow-config review 子指令的 --dry-run 用法，並明定經使用者確認後才不帶 --dry-run 重跑

## ADDED Requirements

### Requirement: 技能規定政策欄位一次確認現值

渲染產出的 speclink-config 技能檔 SHALL 規定政策欄位的問法：

- **確認表**：技能 SHALL 先列一張確認表，逐列顯示每個政策項目的現值——locale、spec_locale、tdd、audit、worktree、任務驗證的測試範圍（第五問）、審查規則的模式（第六問）。未設定的項目 SHALL 標為「未設定，預設 X」：locale 與 spec_locale 預設英文、tdd、audit 與 worktree 預設 false、測試範圍預設全量、審查規則預設不設定。
- **一題確認**：技能 SHALL 只問一題「全部沿用，還是要改其中幾項」。使用者選全部沿用時，技能 SHALL NOT 再逐項詢問，SHALL NOT 改動任何政策欄位；選要改時，技能 SHALL 只逐項詢問使用者點名的項目，每題帶出該項現值。第一次執行（config 尚無任何政策欄位）SHALL 同樣攤開確認表、以一題確認。
- **不推斷**：「全部沿用」是使用者明確給的答案；技能 SHALL NOT 自 repo 推斷任何政策值（例如有測試目錄不代表 tdd 為 true）。
- **工具限制**：互動提問工具的單題選項上限容不下所有項目時，「要改哪幾項」SHALL 改以純文字詢問；互動提問工具不可用時，整個確認 SHALL 以純文字進行。
- **worktree 的寫入**：worktree 的改動 SHALL 經 speclink workflow-config set worktree <true|false> 的 --dry-run 核准流程落地。該寫入會同步技能檔，且有活躍 worktree 時關閉會被 CLI 拒絕；技能 SHALL 把拒絕訊息轉告使用者，SHALL NOT 自行收尾或移除 worktree。

#### Scenario: 渲染產物含確認表與單題確認

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）的政策詢問段落
- **THEN** 段落列出七個政策項目（含 worktree）、未設定者標示預設值、只問一題全部沿用或改其中幾項、選要改才逐項詢問被點名的項目

##### Example: 確認表的顯示

| 項目 | config 現況 | 確認表顯示 |
| ---- | ----------- | ---------- |
| locale | tw | tw |
| spec_locale | 未設定 | 未設定，預設英文 |
| tdd | true | true |
| audit | 未設定 | 未設定，預設 false |
| worktree | 未設定 | 未設定，預設 false |
| 測試範圍 | rules 無測試範圍 rule | 未設定，預設全量 |
| 審查規則 | 無 review 區塊 | 未設定，預設不設定（只用內建 12 條 code smell） |

#### Scenario: 全部沿用不產生政策 diff

- **WHEN** 使用者在確認表選全部沿用
- **THEN** 技能不再詢問任何政策項目，政策欄位與 review 區塊的 dry-run diff 皆不因政策問答而產生改動

#### Scenario: 只問被點名的項目

- **WHEN** 使用者回答要改 tdd 與 worktree
- **THEN** 技能只逐項詢問 tdd 與 worktree 兩題，每題帶出現值；其餘五項維持現值、不再詢問

#### Scenario: 項目多於單題上限時改純文字

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於「要改哪幾項」的段落
- **THEN** 段落明定項目多於互動提問工具單題的選項上限時改以純文字詢問，互動提問工具不可用時整個確認以純文字進行

#### Scenario: 關閉 worktree 被拒時轉告使用者

- **WHEN** 使用者把 worktree 改為 false，而專案有活躍的 linked worktree
- **THEN** speclink workflow-config set worktree false 以非零 exit code 拒絕、config 位元組不變；技能把 CLI 列出的活躍 worktree 轉告使用者，不自行收尾或移除 worktree

### Requirement: 技能規定提醒的分流

渲染產出的 speclink-config 技能檔 SHALL 規定使用者提醒（使用者希望專案遵守的規矩）的處理：

- **來源**：提醒 SHALL 來自使用者——口述，或使用者提供的提醒清單（貼在對話中，或給一個檔案路徑由技能讀取）。提醒清單是使用者的輸入，不屬「固定輸入來源」的掃描範圍；技能 SHALL NOT 自行讀取任何工具的對話紀錄尋找提醒。
- **三題分流**：每條提醒 SHALL 依三題判斷去處——違反的證據在程式還是文件、審查員只看 patch 能不能判斷、太晚發現的代價大不大——分到四個去處之一：審查規則、某個 artifact 的產出規則（rules）、兩邊都放、或不收。
- **審查規則的出口只在選用時開放**：審查規則的模式為附加或覆蓋時，四個去處都開放；模式為不設定時，分到審查規則的提醒 SHALL NOT 寫入，SHALL 在最後報告列為「屬審查規則，未寫入」，分到兩邊都放的提醒只落地產出規則那一句。
- **兩邊都放的條件**：技能 SHALL 只在兩個階段各自有用時才兩邊放，其餘選一邊——產出規則每次產出該 artifact 都會注入。兩邊的兩句 SHALL 描述不同對象：產出規則要求計畫寫出什麼，審查規則檢查程式本身。
- **審查規則不寫 verify 的工作**：審查規則 SHALL NOT 寫成「程式要照 artifact 做」，那是 verify 關卡的工作。
- **產出規則的搬移建議**：模式為附加或覆蓋時，既有產出規則中某句其實在檢查程式本身，技能 SHALL 建議把該部分拆到審查規則；搬移 SHALL 經使用者確認才落地，被拒絕時原句保留（刪除理由限定照舊）。技能 SHALL NOT 記錄被拒絕的建議，下次執行再次提出屬預期行為。模式為不設定時，技能 SHALL NOT 提出搬移建議。
- **時機**：提醒依賴的行為尚未進程式（例如討論已定案、對應的變更尚未實作）時，技能 SHALL NOT 寫入該提醒，SHALL 告知使用者並在最後報告列出。需要查討論或變更的狀態時，技能 SHALL 用 speclink discuss list --json 或 speclink list --json，SHALL NOT 直接讀取 openspec/ 目錄。
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
