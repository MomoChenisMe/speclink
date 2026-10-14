## Context

`/speclink-config` 的技能原稿是 speclink-core 內嵌的 config 技能範本（crates/engine/speclink-core/assets/skills/config.md），經 `speclink init`／`speclink update` 渲染成 claude、codex、copilot 三種工具的技能檔。現行流程分五步：讀固定輸入來源 → 依四條判準起草 context 與 rules → 逐題詢問政策欄位 → 以 `--dry-run` 出 diff、核准後寫入 → 收斂驗收。

現況的缺口（討論 config-skill-rule-routing 第 1–6 輪）：

- 第 3 步每次執行都逐題問六題（locale、spec_locale、tdd、audit、第五問測試範圍、第六問 review.baseline），每題帶現值但問法與第一次相同；worktree 欄位可以由 `speclink workflow-config set worktree` 寫入，技能卻從未問過。
- 審查規則接收寫在第 3 步尾端：先問 baseline，再問要不要加規則，依措辭建議嚴重度；重複檢查只比對 12 條 code smell。
- 技能沒有「一條規矩該放審查規則還是產出規則」的判斷；使用者給一整份提醒清單時，技能沒有明寫可以收。
- speclink init 產生的預設 config 不含 review 區塊；沒有 review 區塊時引擎讀成 extend 加三個空清單（workflow-config 規格「config 的 review 區塊與嚴格解析」）。

render_golden 測試以字面片段釘住 config 技能的結構：第五問測試要求 `### The fifth question` 位於 `` - `audit` `` 之後、`## Step 4` 之前；判準測試以 `## Step 2`、`## Step 3`、各判準標題為邊界。skill_authoring 守門把 config.md 的大寫強調字上限登記為 4，現況剛好用滿（NEVER 1、NOT 2、STOP 1）。

## Goals / Non-Goals

**Goals:**

- 政策欄位改為一張確認表加一題確認，worktree 納入確認表。
- 審查規則改為選用：模式一題三選（不設定／附加／覆蓋），不設定時技能不碰 review 區塊。
- 使用者提醒（口述或提醒清單）經三題分流到審查規則、產出規則、兩邊或不收，並在報告逐條交代去處。
- 審查規則的嚴重度改以「有沒有正當例外」判斷；重複檢查多擋「已有測試或 lint 守門」與「屬其他關卡」兩種。
- 只改技能文字與其衍生物，引擎行為與 CLI 介面不變。

**Non-Goals:**

- 技能不自行讀取各工具的對話紀錄找規則（跨工具格式不同、量大、涉及隱私）。
- 嚴重度不加「看後果」第二個軸。
- 不記住使用者拒絕過的搬移建議（config.yaml 重寫會清掉註解，沒有地方存）。
- 提醒「尚未進程式」的判斷不做比對機制，只是一句判斷。
- 不改 workflow-config 動詞、config 欄位、審查站讀 review 區塊的方式、desktop 設定頁。
- 不改 skill_authoring 的強調字上限登記表。

## Decisions

### 技能分成六步，提醒分流獨立成一步

新結構：Step 1 讀固定輸入來源（不變）→ Step 2 四條判準（不變，只改跨步參照）→ Step 3 政策確認表 → Step 4 審查規則與提醒分流 → Step 5 出 diff 再寫入（原 Step 4）→ Step 6 收斂驗收（原 Step 5）。

- 為什麼獨立一步：政策確認是「使用者的答案」，提醒分流是「判斷使用者給的內容放哪裡」，兩者性質不同；擠在同一步會讓模型把分流當成政策題逐題問。
- Step 3 保留 `### The fifth question` 與 `### The sixth question` 兩個小節，分別說明確認表中這兩項被點名修改時怎麼問、答案怎麼落地；既有測試錨點因此大多不動。
- Step 2 中「review rules 由 Step 3 向使用者取得」與刪除理由段「the user's own answer in Step 3」的參照改指新步驟；判準四中「Step 4's `--dry-run` previews」改為 Step 5。
- 替代方案：維持五步、把分流塞進 Step 3 尾端——步驟標題「Ask for the policy fields」與內容不符，被否決。

### 政策確認表的七個項目與預設值標示

確認表逐列：`locale`（未設定＝英文）、`spec_locale`（未設定＝英文，`auto` 跟隨 locale）、`tdd`（false）、`audit`（false）、`worktree`（false）、測試範圍（無測試範圍 rule＝全量）、審查規則模式（無 review 區塊＝不設定）。未設定的列寫成「not set — defaults to X」，由執行技能的模型以使用者語言呈現。

- 只問一題「keep everything, or change some of them」；選 keep 不再提問、不改任何政策欄位。
- 選 change 時，「要改哪幾項」超過互動工具單題的選項上限就改純文字問；逐項問被點名的項目時沿用既有的 locale 代碼映射、第五問的落地路徑。
- 互動工具不可用時，整個確認以純文字進行，沿用 skill-authoring 規格的跨模型寫法（AskUserQuestion 要有純文字退路；守門 `missing_fallbacks` 會檢查）。
- worktree 寫入沿用 `speclink workflow-config set worktree <true|false>`：寫入會同步技能檔、有活躍 worktree 時關閉會被拒絕；技能只轉告拒絕訊息，不自行收尾 worktree（收尾屬 /speclink:worktree-merge）。

### 審查規則模式一題三選，不設定時審查規則出口關閉

第六問的三個選項：not set（預設）／`extend`／`replace`。

- **not set**：不寫 baseline、不新增規則。若 review 區塊已有規則，維持現狀，既有規則只照判準 3、4 檢查（會過時的內容、引用失效），移除仍要經 diff 確認。
- **extend／replace**：寫入 baseline（extend 由 CLI 寫成不帶 baseline 鍵），並接收規則。
- 提醒分流在三種模式下都會跑三題，用來決定產出規則；只有審查規則這個出口在 not set 時關閉——分到審查規則的提醒列進報告「review rule — not written」，分到兩邊都放的只落產出規則那一句。
- 為什麼分流照跑：結論要求「只適合當審查規則的提醒在報告列為屬審查規則，未寫入」，這需要先判斷；而產出規則與審查規則是否選用無關。
- 替代方案：not set 時完全不跑分流、提醒一律當產出規則候選——會把檢查程式的句子塞進產出規則，正是討論要解決的問題，被否決。

### 嚴重度以正當例外判斷，措辭降為提示

技能對每條要落入審查規則的規則問「有沒有正當例外」：沒有→`critical`；通常要修、偶有正當例外→`warning`；偏好→`suggestion`。措辭清單（「不得」「一律」「要」「宜」「偏好」等）保留為提示，明寫它不單獨決定分格，並舉「措辭為一律、但有正當例外→warning」的反例。

- 嚴重度名稱一律小寫並放在行內 code，避開大寫強調字守門。

### 兩種新的不收：已有守門、屬其他關卡

- **已有測試或 lint 守門**：用判準四同類的針對性靜態查核——在 lint 設定中找該規則、以文字搜尋找到釘住該行為的測試；不通讀原始碼。找得到就不收，報告附上守門的測試名或 lint 規則名。
- **屬其他關卡**：audit 開啟時安全類規矩歸 audit；「程式要照 artifact 實作」歸 verify。audit 以本次確認表確認後的值為準；audit 由關改開時，既有安全類審查規則在同一份 diff 中提出移除。
- 這兩條檢查同時適用既有與新收的審查規則，刪除理由限定照舊（不過檢查或使用者撤回）。

### 提醒清單是使用者輸入，不屬固定輸入來源

技能明寫提醒可以是口述、貼上的清單，或使用者給的檔案路徑（由技能讀取該檔）。Step 1 的「不掃原始碼、只讀固定輸入來源」不變：提醒清單是使用者交給技能的內容，不是技能自己去掃的來源。

- 「尚未進程式」的時機判斷需要查討論或變更狀態時，只用 `speclink discuss list --json`、`speclink list --json`，不直接讀 openspec/ 目錄——同時符合 verb-contract 守門，也讓技能在 remote 模式（規格存在 server）照樣成立，朝 storage 解耦的方向一致。

### 強調字上限維持 4

新文字不使用任何大寫強調字（MUST、NEVER、SHALL、ALWAYS、IMPORTANT、CRITICAL、STOP、NOT）；嚴重度名稱小寫。既有四個（Prerequisites 的 STOP、判準四的「Do NOT execute」、刪除理由的「is NOT a reason」、Step 5 的「NEVER write before the user approves」）保留——前兩個 NOT 被 render 測試以字面釘住。skill_authoring.rs 不改。

- 替代方案：調高上限——結論要求不調高，且新內容是判斷步驟，用跨模型寫法（說明理由）比加強調字有效。

### 技能版號升一個 minor，與 review-ticket-applied-rules 依序進行

改技能內文要三連動：`ASSET_VERSION` minor +1 → `UPDATE_GOLDEN=1` 再生六份快照 → `UPDATE_ASSETS_LOCK=1` 再生指紋。在途的 review-ticket-applied-rules 也改技能並升版號，兩者同時進行必在版號行、golden 與 assets.lock 對撞，所以依序進行：後做的那個在前者合併後的 main 上升版。版號數字以實作當下的 main 為準，本設計不寫死。

- 之後 `cargo build -p speclink-cli` 再執行 `./target/debug/speclink update`，再生本 repo 的受管技能檔；這些檔不會被 task done 記進 evidence，收尾以 `git status` 盤點。

### render 測試的改法

- 改寫 `config_skill_asks_the_test_scope_question`：第五問的結束邊界由 `## Step 4` 改為下一個小節或下一步標題；needle 改為「確認表帶出現值」的新字面，保留「Never infer it from the repo」「dependency manifests you already read in Step 1」「write no test-scope rule at all」「ruling to withdraw」等仍成立的片段；守則清單的錨點改為新守則字面。
- 新增三支測試，各對一條需求：政策確認表（七個項目含 worktree、一題確認、只問點名項目、純文字退路、worktree 拒絕轉告）、審查規則選用（三選一、not set 不碰 review 區塊、正當例外分格、兩種新的不收、audit 改開的移除）、提醒分流（三題、四個去處、兩邊都放的條件、不寫 verify 的工作、搬移建議只在選用時提出、提醒清單來源、不讀對話紀錄、時機判斷用 verb）。
- 測試沿用 `skill_for_both_tools` 對 claude 與 codex 兩種工具渲染後比對。

## Implementation Contract

**Behavior**：使用者執行 `/speclink-config` 時，看到的政策詢問是一張確認表加一題「全部沿用或改幾項」；審查規則是確認表中的一項，預設 not set；使用者給的提醒（口述、貼上或檔案）在報告中逐條交代去處與理由。沒有 review 區塊的專案，整輪執行不會新增 review 區塊，除非使用者選了 extend 或 replace。

**Interface**：只有渲染出的 speclink-config 技能檔內文改變。CLI 指令、旗標、stdin、exit code、`--json` 輸出、config.yaml 欄位與預設值全部不變。技能檔可觀察的結構：

- `## Step 3` 含七個政策項目（各帶未設定時的預設值）、單題確認、純文字退路、worktree 寫入與拒絕轉告，以及 `### The fifth question`、`### The sixth question` 兩小節。
- `## Step 4` 含提醒來源（口述、貼上、檔案路徑；不讀對話紀錄）、三題、四個去處、not set 時的出口關閉、兩邊都放的條件、不寫 verify 的工作、搬移建議與重提、時機判斷、嚴重度以正當例外判斷、兩種新的不收、audit 改開的移除。
- `## Step 5`（diff 與寫入）、`## Step 6`（收斂驗收，報告逐條列出提醒去處）。
- `## Guardrails` 更新：政策以一張表確認、審查規則選用且不從 repo 推導、提醒逐條交代去處。

**Failure modes**：技能檔內文不存在執行期失敗；舊版技能檔的專案照舊問法運作。worktree 關閉被 CLI 拒絕時，技能轉告訊息、config 不變。

**Acceptance criteria**：

- `cargo test -p speclink-core --test it` 全綠——含 render_golden（改寫的第五問測試與三支新測試）、skill_authoring（強調字仍為 4、無歷史字樣、AskUserQuestion 有純文字退路、技能引用走 render 記號）、golden 快照與 assets.lock。
- `speclink update` 後 `git status` 只多出版號行改變的受管技能檔與 speclink-config 內文改變；再跑一次 `speclink update`，`git status` 不再增加改動。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠（docs 改動的詞彙、中英對照、連結守門）。
- 人工：在一個專案實際執行 `/speclink-config`，確認確認表、三選一、提醒報告的行為。

**Scope boundaries**：

- 範圍內：config 技能原稿、`ASSET_VERSION`、六份 golden 快照、assets.lock、render_golden 的 config 測試、docs/configuration.md 與 docs/configuration.zh-TW.md 描述 config 技能的段落、本 repo 再生的受管技能檔。
- 範圍外：workflow-config 動詞與規格、review 技能與審查站、desktop、server、其他技能原稿的內文、skill_authoring 上限表。

## Risks / Trade-offs

- [golden 在髒樹再生會烙進無關改動] → 再生前以 `git status` 確認只有本 change 的原稿與版號改動；再生後 diff 只應出現 speclink-config 區段與版號行，再不帶開關重跑一次確認穩定。
- [與 review-ticket-applied-rules 的版號行對撞] → 依序進行；若仍需合併，照既定做法重生衍生物（版號取高於兩側的新值），不手挑衝突。
- [強調字守門紅燈] → 新文字不寫大寫強調字、嚴重度小寫；改完先跑 skill_authoring 測試。
- [模型照舊逐題詢問] → render 測試釘住單題確認的字面；人工任務實際執行一次技能確認。
- [被拒絕的搬移建議每次重提造成雜訊] → 接受；只出現在選了 extend 或 replace 的專案，且建議不產生 diff，不影響收斂驗收。
- [跨平台] → 只改 Markdown 原稿與快照；golden 比對沿用既有的換行處理，Windows CI 不受新的路徑或檔案系統行為影響。

## Migration Plan

- 本 repo：同一提交含原稿、版號、golden、assets.lock、docs 與 `speclink update` 再生的受管技能檔。
- 既有專案：升級 CLI 後執行 `speclink update` 取得新版技能檔；config.yaml 不需遷移。沒更新的專案沿用舊問法。
- 回退：還原原稿與版號後重跑三連動與 `speclink update`。

## Open Questions

(none)
