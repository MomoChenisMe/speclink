---
topic: 將 grill me（烤問）skill 併入 speclink-discuss 的評估與作法
slug: grill-mode-in-discuss
status: promoted
promoted_to: discuss-decision-tree-interview
created: 2026-07-30
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 將 grill me（烤問）skill 併入 speclink-discuss 的評估與作法

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者看到 mattpocock/skills 的 grilling skill（「grill me」——沿決策樹逐題拷問使用者直到 shared understanding），想評估是否併入 speclink-discuss、如何併入才符合 discuss 的目的。模式選 assumptions：找到三處 skill 實例（crates/speclink-core/assets/skills/discuss.md、.claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md）與引擎端 discuss.rs／main.rs，脈絡足夠。無既存相關開放討論。關鍵引擎事實：add-round 的 --mode 是自由字串無驗證（main.rs:683、discuss.rs:275），新增 grill 模式值零引擎成本。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-30)

**Focus**: grilling 四條規則與 discuss 現況逐條比對，決定併入策略
**Position**: 值得加，但不整包照抄——普適規則併入，relentless 做成 opt-in：
- grilling 僅四條規則：一次一題（discuss 已有）、事實自己查／決策問人、每題附建議答案、relentless 決策樹遍歷直到 shared understanding
- 「事實自己查、決策才問人」「每題附建議答案」兩條併入 interview mode（How to Discuss 區）——使用者已確認
- relentless 決策樹與 discuss 的收斂閥（one nudge maximum）方向相反，擬做成掛在 Mode switching 區的觸發式強度開關（待裁定）
- 引擎零改動：add-round 的 --mode 是自由字串（main.rs:683、discuss.rs:275）
- 落點：三處 skill 檔同步＋乾淨樹再生 render golden（CLAUDE.md 既知規則）
- grill 與 assumptions 模式審視方向互補：assumptions＝使用者修正 agent 的假設；grill＝agent 拷問使用者的計畫
**Ruled out**: 整包照抄 grilling 成預設行為——relentless 與「Don't overwhelm」「one nudge maximum」在同一份 skill 內直接衝突
**Open**: one nudge maximum 與決策樹遍歷的行為差異（使用者本輪提問，下輪回答）；grill 輪記錄用 --mode grill 還是沿用 interview；觸發詞是否要中文版、叫什麼；假設 2／4／5 待使用者確認

### Round 2 — assumptions (2026-07-30)

**Focus**: 決策樹要做成 opt-in 模式，還是直接取代 interview 的預設提問策略（使用者提議後者）
**Position**: 直接取代、不加模式——關鍵是把 grilling 拆成兩半，只拿樹、不拿「不准停」：
- 決策樹＝提問順序與覆蓋結構；relentless＝停止條件。兩者可分離，grilling 原文把它們捆在一起
- 樹當預設：discuss 本來就是收斂導向，依賴排序比現行「挑最重要的先問」更有原則（下游問題的形狀取決於上游答案，先問下游會被翻盤）
- 記錄結構天然吻合：每輪一個 Focus＝解掉一個節點；Open ledger＝還沒走到的分支；結論「resolve or defer every remaining one」＝樹走完或明示留白
- 停止條件維持使用者主導：one nudge maximum 與 Deferred 續留——樹改變的是覆蓋與順序，不是步調控制權
- assumptions mode 不動：脈絡夠時等於「預填好答案的樹」一次攤開，修正後走進受影響分支
- 不加模式的紅利：無第三種狀態、無觸發詞、無輪標籤問題，skill 自身複雜度不升
**Ruled out**: opt-in grill 模式（前輪假設 2）——樹紀律對所有討論都有益，不該鎖在觸發詞後面，且加第三種模式狀態徒增 skill 複雜度；relentless 停止條件——discuss 的契約是「明示 Deferred 地收斂」，不是「不全解完不放行」；前輪兩個開放問題（--mode grill、中文觸發詞）隨不加模式一併作廢
**Open**: one-nudge 續留與 assumptions mode 保留不動的最終裁定；落地方式（三處 skill 檔＋golden 走 promote 成 change）

### Round 3 — assumptions (2026-07-30)

**Focus**: 換成決策樹後，討論文件格式是否需要調整？跨 session 延續與演變追蹤是否保留？
**Position**: 格式零調整——骨架、輪模板、append-only 規則、結論格式全部不動；改變的只是輪內容的產生方式：
- 引擎只管骨架（Context／Rounds／Conclusion、### Round N 標題，discuss.rs:194-293）；輪內文本來就是 skill 端的自由 markdown，欄位名（Focus／Position／Ruled out／Open）沿用，新舊記錄讀起來一致、無需遷移
- 樹的落點：首輪 Position 攤開初始決策空間（可含 ASCII 樹）；之後每輪解一個節點；中途發現的新分支記在該輪 Open——維持 append-only，不回改早期輪
- Open ledger 從「鬆散的未解清單」升級為「精確的樹前緣」：跨 session 續談直接從上輪 Open 接手要走的分支，延續性反而更強
- 演變追蹤照舊：翻案開新輪並記明原因、Ruled out 記淘汰理由；樹讓「這輪解哪個節點、為何此序」更可讀
- propose --from-discussion 讀的 Conclusion 結構未動，下游不受影響
**Open**: none——所有分支已走完（one-nudge 續留與 assumptions 保留已獲使用者本輪「OK可以」確認）

### Round 4 — assumptions (2026-07-30)

**Focus**: 調整後 discuss 是否會依 codebase 實況回答？（使用者觀察：現行偏「簡單看過就回答」）
**Position**: 會——這是兩條新規則的直接後果；調查模式從「開場一次淺掃」變為「沿樹逐節點查」：
- 現況診斷（使用者體感準確）：Step 2 scout 明文時間盒（few seconds、≤5 files），用途僅是選模式；interview 提問無結構性查證要求，「Ground in reality」只是軟指示
- 「事實自查、決策問人」→ 每個節點解決前先分診事實／決策；事實類自己 Grep／Read，不問人、也不得憑印象答
- 「每題附建議答案」→ 建議須有依據，等於把 assumptions 模式的強制 Evidence 擴展到 interview 每一題
- 開場 scout 維持淺是對的設計：深讀花在確定會走到的分支上，被剪掉的分支不預讀
- 本輪新增裁定：落地時 skill 文字須把「建議答案附 Evidence（檔案路徑／查證結果）」寫成硬規則，避免退化回軟指示——結論隨之更新
**Open**: none

## Conclusion

**Decision**: speclink-discuss 的 interview 模式預設提問策略全面換成決策樹遍歷——開場攤決策空間、依依賴序一次一題、每題附建議答案且**附 Evidence（檔案路徑或查證結果；assumptions 模式的 Evidence 慣例擴展為 interview 硬規則）**、事實自查決策問人（每節點先分診事實／決策，事實類沿樹逐節點查證，不憑印象答）；不新增 grill 模式或觸發詞；relentless 停止條件不採用，one nudge maximum 與 Deferred 續留；assumptions mode 保留不動（脈絡足夠時＝預填好答案的樹一次攤開）；開場 scout 維持淺（僅負責選模式）；討論文件格式零調整。
**Rationale**: grilling 可拆為「決策樹（覆蓋＋依賴排序）」與「relentless（停止條件）」兩件獨立的事。樹紀律對所有討論都有益，且與既有 Socratic ledger 天然吻合（每輪一 Focus＝解一節點、Open＝樹前緣、結論的 resolve-or-defer＝樹走完或明示留白），不該鎖在觸發詞後面；停止條件維持使用者主導，才符合 discuss「明示 Deferred 地收斂」的契約。逐節點查證＋每題 Evidence 同時修掉現行「簡單看過就回答」的缺口——深讀花在確定會走到的分支上，而非開場全讀。
**Rejected alternatives**: 整包照抄 grilling 成預設——relentless 與「Don't overwhelm」「one nudge maximum」直接衝突；opt-in grill 第三模式——徒增 skill 自身複雜度，並衍生 --mode grill 輪標籤、中文觸發詞等問題，樹紀律不該是選配；relentless 停止條件——「不全解完不放行」違背 discuss 收斂契約；加深開場 scout——會預讀後來被剪掉的分支，查證深度應跟著樹走。
**Deferred**: none
**Capture to**: 三處 skill 檔（crates/speclink-core/assets/skills/discuss.md、.claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md）＋乾淨樹再生 render golden——經變更（change）落地
**Next**: speclink discuss promote grill-mode-in-discuss --name discuss-decision-tree-interview
