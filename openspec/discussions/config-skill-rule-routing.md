---
topic: config 技能的規則分流：審查規則與產出規則的界定、重複檢查、嚴重度判斷，以及相關 CLI 缺口
slug: config-skill-rule-routing
status: concluded
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
hold: true
---

# Discussion: config 技能的規則分流：審查規則與產出規則的界定、重複檢查、嚴重度判斷，以及相關 CLI 缺口

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者在兩個專案（speclink 與另一個專案）實際整理審查規則與產出規則——先從近一個月的對話找出使用者反覆提醒的程式規矩，再逐條判斷放進 openspec/config.yaml 的哪一處。過程中 config 技能的現行判準不夠用：技能把來源分死（審查規則只收口述、產出規則從 repo 結構推導），沒有「一條提醒該放審查規則還是產出規則」的分流；審查規則的重複檢查只比對 12 條 smell；嚴重度依措辭建議，實際整理時反而是先定嚴重度再回頭改措辭。另發現兩個 CLI 缺口：技能檔比引擎舊時寫入審查規則會靜默失效（另一個專案寫入時技能檔仍是舊版，CLI 只印成功訊息），以及 workflow-config 的 --dry-run 差異把沒改的行列成刪除再加回。

需求明確（六個要點各有具體目標），未經 grill 階段，直接以假設清單開場。

相關規格：config-skill、workflow-config、review-skill；在途 delta：review-rules-config: config-skill, review-rules-config: workflow-config, review-rules-config: review-skill, quality-gate-speedup: review-skill；相關變更：review-rules-config、quality-gate-speedup。
Prior discussions: custom-review-rules, config-skill-rule-alignment, quality-gate-speed, desktop-ui-redesign, plan-overlap-granularity, desktop-window-and-settings-polish, config-context-與-rules-gui-編輯, desktop-導覽與專案首頁重構

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: config 技能處理使用者規則提醒的六個要點（R1–R6）各自怎麼定
**Position**: 六條假設使用者全部確認，作為後續的既定方向。
- R1 分流：技能對每條提醒先問三題——違反證據在程式還是文件、審查員只看 patch 能否判斷、太晚發現代價大不大——分成審查規則、某 artifact 的產出規則、兩邊都放（兩句描述不同對象：產出規則要求計畫寫出什麼，審查規則檢查程式本身）、或不收；審查規則不得寫成「程式要照 artifact 做」（那是 verify）；檢查既有產出規則時，若某句其實在檢查程式，建議拆一半到審查規則，因刪除理由限於不過判準或使用者撤回，搬移須使用者確認（實例：speclink 4c33325e 的 className 覆蓋句、另一個專案的動畫時間常數句）
- R2 重複檢查：在「extend 時淘汰重述 12 條 smell」之外再加兩種不收——已有測試或 lint 守門的（以靜態搜尋測試與 lint 設定核實；實例 packages/ui/src/__tests__/cursorRule.test.ts），與屬於他站的（安全類在 audit 開啟時歸 audit、對照 artifact 歸 verify）；與 custom-review-rules「lint 已強制的規則審查本來就略過」同一理由
- R3 嚴重度：沿用 custom-review-rules 已定的分格意義（critical＝違反必錯無例外、warning＝通常要修偶有正當例外、suggestion＝偏好），技能改以「有沒有正當例外」判斷，措辭降為提示；與在途 delta「技能規定審查規則的第六問與規則接收」的依語氣建議牴觸，舊理由（措辭對應簡單）不再成立——兩個專案實際整理時都是先定嚴重度再改措辭配合；「看後果」不另加，既有定義已足
- R4 時機：規則依賴的行為尚未進程式（討論已定案但 change 未落地）時先不寫入並告知使用者，只加一句判斷，不做比對機制（實例：另一個專案的「模型讀的提示詞用英文」）
- R5 CLI：(a) workflow-config review 寫入成功後，技能檔比引擎舊時多印一行提示跑 speclink update（過期探測已在 crates/engine/speclink-core/src/workspace/init.rs），屬刻意輸出變更要記相容性影響；(b) crates/adapters/speclink-cli/src/verbs/config.rs 的 unified_diff 只去頭尾相同行，改成逐行比對只標真正改動；兩項為引擎改動，與技能文字分刀
- R6 來源：技能明寫可收使用者提供的提醒清單（貼上或檔案路徑），每條走 R1 分流；canon 的「口述」本已涵蓋，config-skill-rule-alignment 已定使用者裁決型規則為正當來源
**Ruled out**: 技能自行讀取各工具的對話紀錄找規則——跨工具格式不同、量大（另一個專案 2.1 GB）、涉及隱私；嚴重度改看「後果」——既有的「有無正當例外」定義已足以判斷，多一個軸只會讓判準打架
**Open**: 與在途 change 及未轉出 cut 的相依與順序（使用者新提）；分刀方式（技能文字一刀、CLI 一刀是否成立）

### Round 2 — interview (2026-10-14)

**Focus**: 與在途 change、未轉出 cut 的相依，以及分刀與順序
**Position**: 分兩刀，排在 review-rules-config 封存之後；CLI 刀先做，技能文字刀排在 quality-gate-speedup 封存之後、與 custom-review-rules 第 2 刀不同時進行。
- 唯一實質相依是 review-rules-config：R1、R2、R3、R6 要改寫的需求「技能規定審查規則的第六問與規則接收」只在它的 delta、尚未進正典，R3 改的正是它新增的依語氣建議；檔案亦重疊（crates/engine/speclink-core/assets/skills/config.md、crates/adapters/speclink-cli/src/verbs/config.rs，後者平行 session 正在修品質站發現）——propose 也等它封存後再做，直接改寫正典而不建立在未封存 delta 上
- quality-gate-speedup 與 custom-review-rules 第 2 刀（review-ticket-applied-rules）內容不重疊（都不碰 config 技能、workflow-config、config.rs；R3 不改三清單的意義，第 2 刀照記規則原文不受影響），但都會升 ASSET_VERSION、改 golden 與 assets.lock，與技能文字刀同時做必在版號行對撞
- CLI 刀（R5）不升技能版號，review-rules-config 封存後即可開工；它修的是規則靜默失效，價值最高；desktop 本已顯示技能檔過期提示，缺口只在 CLI
- server-web-list-pages、server-web-desktop-alignment 第 3 刀、agent-tool-sdk-layering 無關
- 兩刀 → conclude 帶 --hold，第二刀轉出帶 --last
**Ruled out**: 技能文字刀與 quality-gate-speedup 或第 2 刀平行 apply——內容雖不衝突，版號行與 golden 必對撞，合併時要重生衍生物，不值得；現在就 propose 並以 review-rules-config 的 delta 為底——多一層「封存須排在它之後」的協調，而它已接近封存
**Open**: 這些調整對整體 SDD 架構與提示詞的影響（使用者新提）

### Round 3 — interview (2026-10-14)

**Focus**: 審查規則是否由 config 技能主動加入（使用者糾正第 1 輪 R1）
**Position**: 改變第 1 輪 R1——審查規則是選用的，技能不主動加入；先問使用者要不要設定、附加還是覆蓋 12 條 smell，選了才分流。
- 改變的原因：使用者指出不是每個人都會用審查規則，預設 config 不應主動加；第 1 輪 R1 的「提醒自動分流進審查規則」與「建議把產出規則中檢查程式的句子搬到審查規則」都等於主動加入
- 現行正典本就是問：在途 delta「技能規定審查規則的第六問與規則接收」帶現值問 baseline、再問是否新增規則；speclink init 產生的預設 config 不含 review 區塊（crates/engine/speclink-core/src/workspace/init.rs 無 review）——問題出在第 1 輪 R1 的寫法，不在正典
- 新形狀：第六問與「要不要加規則」合併為一題三選——不設定（預設；不動 review 區塊、不分流到審查規則、不建議搬移，只適合當審查規則的提醒在報告列為「屬審查規則，未寫入」）／附加（extend：12 條照舊再加使用者規則，之後才走 R1 分流與 R2、R3 判斷）／覆蓋（replace：只用使用者規則，清單可為空＝只依 repo 文件，為 custom-review-rules 認可的正當選擇）
- 已有審查規則時題目帶現值，選不設定＝維持現狀，既有規則仍照判準 3、4 檢查引用是否失效
- 連帶效果：搬移建議只出現在選了附加或覆蓋的專案，「拒絕的搬移建議每次重提」的範圍隨之縮小
**Ruled out**: 提醒一律自動分流到審查規則——沒打算用審查站的人會被塞進審查規則；搬移建議不分是否選用都提出——同上，且在不用審查規則的專案只會是每次重提的雜訊
**Open**: 三選一題的形狀待使用者確認；第 3 節兩個取捨待確認——「兩邊都放」只在兩個階段都各自有用時才用、接受被拒絕的搬移建議會重提

### Round 4 — interview (2026-10-14)

**Focus**: 三選一題的形狀，以及本次調整是否影響 custom-review-rules 第 2 刀（review-ticket-applied-rules）
**Position**: 三選一題形狀使用者確認；第 2 刀的範圍與資料格式不用改，只有 desktop 審查分頁「套用規則」列在沒有專案規則時的呈現要調整。
- 「不設定」不產生新的設定狀態：沒有 review 區塊＝引擎讀到 extend 加三個空清單，與現況相同；工單照記 `**Rules**: extend` 且正確——那次審查確實套用了 12 條 smell
- R1、R2、R3 只在 config 技能執行時作用，決定規則寫進哪裡，不改審查時讀到的資料；R3 不改三清單的意義，工單記的嚴重度不受影響；R5 屬 CLI，與工單無關
- 給第 2 刀的輸入：大多數專案沒有專案規則，原設計會在每張工單的「套用規則」列顯示 extend 與三個 0 計數；改為沒有專案規則時只寫「預設 12 條 code smell」、不顯示 0 計數，有專案規則時維持原設計（模式＋三計數、可展開），replace 且無規則時維持琥珀提示
- 這條輸入記在本討論的結論與記憶中；第 2 刀以 `--from-discussion custom-review-rules` 轉出，不會自動讀到本討論
**Ruled out**: 在 custom-review-rules 追加輪次並重寫結論——它已以 --hold 結論，重寫會把在途的 review-rules-config 標為待重新反映，要逐一處理，不划算
**Open**: 第 3 節兩個取捨待確認——「兩邊都放」只在兩個階段都各自有用時才用、接受被拒絕的搬移建議會重提；政策欄位已設定時的問法（使用者新提）

### Round 5 — interview (2026-10-14)

**Focus**: 政策欄位已設定時，再次執行 config 技能的問法
**Position**: 改為先一次確認現值、要改的才逐項問（B），並把 worktree 納入確認表；歸入技能文字刀。
- 現況：技能每次以六題逐題詢問（locale、spec_locale、tdd、audit、第五問測試範圍、第六問審查規則），每題帶現值但問法與首次設定相同；正典「技能規定 diff 先行與收斂驗收」寫「政策四欄 SHALL 逐項詢問」，第五問需求同寫逐項詢問，目的為「是使用者的答案、技能不推斷」
- 新問法：先列一張表顯示所有政策欄位的現值（未設定者標「未設定，預設 X」），只問一題「全部沿用，還是要改其中幾項」；選要改才逐項問被點名的欄位；首次執行同樣攤開每個值，只是改為一題確認
- 不違反「不推斷」：「全部沿用」是使用者明確給的答案；收斂驗收不受影響（沿用不產生 diff）
- 選項多於互動工具單題上限時，「要改哪幾項」改以純文字詢問；技能文字維持跨工具寫法（互動工具不可用時以純文字詢問）
- 缺口：config.yaml 的 worktree 欄位可由 workflow-config set 寫入，但 config 技能全文未提及 worktree，從未被詢問；它與 tdd、audit 同性質，納入確認表
- 規格面：config-skill 的「逐項詢問」字面改為「一次確認現值、要改的逐項詢問」，政策欄位清單加入 worktree
**Ruled out**: 每題只把措辭改成「要沿用嗎」（A）——題數仍是六題，使用者在意的是重複作答，只換措辭解決不了
**Open**: 第 3 節兩個取捨待確認——「兩邊都放」只在兩個階段都各自有用時才用、接受被拒絕的搬移建議會重提

### Round 6 — interview (2026-10-14)

**Focus**: 選用審查規則的專案裡，「兩邊都放」的條件與被拒絕搬移建議的處理
**Position**: 兩個取捨都照建議定案。
- 「兩邊都放」只在兩個階段各自有用時才用，其餘選一邊：產出規則每次產出該 artifact 都會注入，放太多會讓提示詞變長
- 接受被拒絕的搬移建議每次執行都會重提：config.yaml 寫入會清掉註解，沒有地方記住拒絕；收斂驗收不受影響（建議不是 diff）；第 3 輪的選用設計已把範圍縮到選了附加或覆蓋的專案
**Ruled out**: 記住使用者拒絕過的建議——需要來源註記，config-skill-rule-alignment 已否決（config.yaml 重寫會清掉註解，標記活不過下次落地）
**Open**: none

## Conclusion

**Decision**: 調整 config 技能處理審查規則、產出規則與政策欄位的方式，並補兩個 CLI 缺口；分兩刀，都排在 review-rules-config 封存之後。
- 審查規則改為選用：第六問與「要不要加規則」合併為一題三選——不設定（預設；不動 review 區塊、提醒不分流到審查規則、不建議搬移，只適合當審查規則的提醒在報告列為「屬審查規則，未寫入」）、附加（extend：12 條 smell 照舊再加使用者規則）、覆蓋（replace：只用使用者規則，清單可為空＝只依 repo 文件）；已有審查規則時帶現值，選不設定＝維持現狀，既有規則仍照判準 3、4 檢查
- 規則分流（選了附加或覆蓋才進行）：每條提醒先問三題——違反證據在程式還是文件、審查員只看 patch 能否判斷、太晚發現代價大不大——分成審查規則、某 artifact 的產出規則、兩邊都放、或不收
  - 兩邊都放時兩句描述不同對象：產出規則要求計畫寫出什麼，審查規則檢查程式本身；只在兩個階段各自有用時才兩邊放（產出規則每次產出都會注入）
  - 審查規則不得寫成「程式要照 artifact 做」，那是 verify 的工作
  - 既有產出規則中其實在檢查程式的句子，建議拆一半到審查規則；搬移須使用者確認（刪除理由限於不過判準或使用者撤回）；被拒絕的建議下次執行會重提，接受
- 重複檢查再加兩種不收：已有測試或 lint 守門的（以判準 4 同類的針對性靜態查核核實，不掃原始碼）；屬於他站的（安全類在 audit 開啟時歸 audit、對照 artifact 歸 verify）；audit 開關改變時，既有安全類審查規則的去留跟著變，經 diff 確認
- 嚴重度沿用 custom-review-rules 已定的分格意義（critical＝違反必錯無例外、warning＝通常要修偶有正當例外、suggestion＝偏好），技能改以「有沒有正當例外」判斷，措辭降為提示
- 時機：規則依賴的行為尚未進程式時先不寫入並告知使用者，只加一句判斷，不做比對機制
- 來源：技能明寫可收使用者提供的提醒清單（貼上或檔案路徑），每條走三選一與分流
- 政策欄位問法：先列一張表顯示所有政策欄位現值（未設定者標「未設定，預設 X」），只問「全部沿用，還是要改其中幾項」，選要改才逐項問；政策欄位清單納入 worktree（現行技能全文未提及，從未被詢問）
- 技能原稿的寫作限制：config.md 強調字已用滿上限 4（NEVER 1、NOT 2、STOP 1），新文字不用大寫強調字、嚴重度名稱寫小寫，否則須刻意調高上限；判斷步驟用跨模型寫法；技能引用用 /speclink:<名稱> 記號；正文不寫日期與 design 編號；需要查討論狀態時用 speclink discuss list --json，不直接讀 openspec/
- 給 custom-review-rules 第 2 刀（review-ticket-applied-rules）的輸入：範圍與資料格式不變；desktop 審查分頁「套用規則」列在沒有專案規則時只寫「預設 12 條 code smell」、不顯示三個 0 計數，有專案規則時維持模式＋三計數可展開，replace 且無規則時維持琥珀提示
- **cut 1 `workflow-config-write-feedback`**: CLI 寫入回饋——技能檔過期提示與逐行差異預覽；review-rules-config 封存後即可開工
  - workflow-config review 寫入成功後，本機技能檔比引擎舊時多印一行提示執行 speclink update（過期探測沿用 crates/engine/speclink-core/src/workspace/init.rs）；remote 模式照查本機技能檔，本機沒有技能檔時不提示
  - crates/adapters/speclink-cli/src/verbs/config.rs 的 unified_diff 由「去頭尾相同行」改為逐行比對，只標真正改動
  - 相容性：review 寫入的人眼輸出在技能檔過期時多一行；所有 workflow-config 子指令的 --dry-run 輸出隨差異演算法改變，CLI 測試同批更新；預覽與實際寫入一致不受影響；不升技能版號
  - 規格：workflow-config
- **cut 2 `config-skill-rule-routing`**: config 技能文字——審查規則選用三選一、規則分流、重複檢查、嚴重度判斷、時機、提醒清單來源、政策欄位一次確認與 worktree
  - 排在 quality-gate-speedup 封存之後；與 custom-review-rules 第 2 刀不同時進行（三者都升 ASSET_VERSION、改 golden 與 assets.lock）
  - 規格：config-skill（改寫「技能規定審查規則的第六問與規則接收」、「技能規定 diff 先行與收斂驗收」的逐項詢問字面、第五問的逐項詢問字面）
  - 引擎：ASSET_VERSION 升一版、golden 與 assets.lock 同批更新；既有專案執行 speclink update 取得新版
**Rationale**: 規則要放在真正會被檢查的地方才有作用——檢查程式的句子放在產出規則，審查站不保證讀得到；但審查規則是選用的，技能只能在使用者選了之後才分流，不能替使用者加；嚴重度的意義早已定好，問題只在技能用措辭代替它；CLI 的靜默失效比技能文字更急，且不升版號、不與其他刀衝突，所以先做。
**Rejected alternatives**:
- 技能自行讀取各工具的對話紀錄找規則 — 跨工具格式不同、量大、涉及隱私
- 嚴重度改看「後果」 — 既有「有無正當例外」的定義已足，多一個軸會讓判準打架
- 提醒一律自動分流到審查規則 — 沒打算用審查站的人會被塞進審查規則
- 搬移建議不分是否選用都提出 — 同上，且在不用審查規則的專案只是每次重提的雜訊
- 技能文字刀與 quality-gate-speedup 或第 2 刀平行 apply — 內容不衝突，但版號行與 golden 必對撞
- 現在就 propose 並以 review-rules-config 的 delta 為底 — 多一層封存順序協調，而它已接近封存
- 在 custom-review-rules 追加輪次並重寫結論 — 會把在途的 review-rules-config 標為待重新反映
- 政策問法只把措辭改成「要沿用嗎」 — 題數仍是六題，重複作答沒有減少
- 記住使用者拒絕過的搬移建議 — 需要來源註記，config-skill-rule-alignment 已否決
**Deferred**: none
**Capture to**: proposal | design | spec | tasks
**Next**: review-rules-config 封存後 /speclink-propose --from-discussion config-skill-rule-routing（先轉出 cut 1 workflow-config-write-feedback；cut 2 轉出時帶 --last）
