---
topic: propose 與 apply 的收尾交棒補強
slug: propose-apply-handoff-updates
status: promoted
promoted_to: propose-apply-handoff-updates
created: 2026-08-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: propose 與 apply 的收尾交棒補強

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出兩項需求：(1) propose 技能收尾要加「盤點提案中的 change 並檢視執行順序」環節，worktree 政策開啟時要分「可平行／須依序」；(2) apply 技能完工後的建議缺「不跑品質站可直接 archive + commit」的提示。兩需求皆明確，無需 grill 階段，直接以假設清單開場。

Scout 結果：技能內文正典在 crates/speclink-core/assets/skills/（propose.md、apply.md、review.md、verify.md、quality.md、archive.md、commit.md、apply-worktree-post.md）；管收尾建議的 canon 是 skill-routing spec「出口交棒由技能結尾承載」及其 Example 交棒句邊集表。相關 specs：skill-routing、propose-skill、worktree-apply-skill、worktree-overlay、review-skill、verify-skill、quality-skill、archive-skill、commit-skill。worktree 政策開關為 openspec/config.yaml 的 worktree: true（含 SPECLINK_WORKTREE 覆寫層）。asset 改動的影響面：MARKER_VERSION／golden／assets.lock 三連動，speclink update 再生 32 份 SKILL.md。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-27)

**Focus**: 兩需求的落點與 canon 影響盤點，及使用者對需求 2 提出的三線接續流程
**Position**: 假設清單成立於 asset 正典；使用者回覆聚焦需求 2，提出品質站後一律向 archive 的流程設計：
- 需求 2 有一半是 asset 向 canon 補課：skill-routing 的 apply Scenario 已要求「或直接封存」，apply.md:321 字面未跟上
- 使用者流程：跑品質站（review／verify／quality）落章後一律接 archive，不交叉提醒另一站；不跑品質站則提示 archive+commit（一步）或 archive 再由 archive 導向 commit（兩步）
- 現況查證：quality.md 出邊已合此流程（archive；worktree 內→worktree-merge）；review.md／verify.md 出邊帶「另一站（若要）」交叉提醒，且被 skill-routing Example 表「review、verify｜落章｜另一站（若要）或 archive」釘住，改字面須帶 MODIFIED delta
- commit.md:118 已內建「Archive first, then commit together」子流程（含 Archived: yes 標記），是 archive+commit 一步到位的既有載具
- archive 技能現無 Next steps 段，canon 定位為流程終點、不被要求帶出邊；flow 3（archive 導向 commit）須動 archive-skill spec 與 skill-routing 該句
**Ruled out**: 需求 1 的引擎新指令（排序判準由 agent 讀檔即可，delta 目錄重疊為機械可查的硬信號）——暫定，待使用者確認
**Open**: 需求 1 的四項假設（觸發條件 ≥2 個提案中 change、硬軟排序判準、worktree 多 session 配方、skill-routing Example 表 propose row 跟改）尚未獲回覆；需求 2 的機制選擇（apply 出邊字面、archive 尾端提醒、兩者並用）；review／verify 出邊改寫時是否比照 quality 補 worktree 分支（站內落章→補提交＋worktree-merge）

### Round 2 — assumptions (2026-08-27)

**Focus**: 需求 2 機制組合與需求 1 全部假設的總確認
**Position**: 使用者以「OK 都對」全數確認，無任何修正：
- 需求 2 定案：review／verify 出邊改兩條（主 checkout 落章→archive；worktree 內落章→補提交蓋章 meta→worktree-merge），拿掉「另一站（若要）」交叉提醒；apply 出邊補「不跑品質站→直接 /speclink:archive，或 /speclink:commit 的『先封存再一起提交』一步到位」，完工模板同步，[M] 路徑不動；archive 尾端加收尾提交提醒（主機制，蓋住所有進 archive 的路）；quality.md 不動
- 需求 1 四項假設定案：提案中 change ≥2 才展開盤點；硬信號（delta capability 重疊→須依序）＋軟信號（讀 proposal／tasks 推測重疊或依賴）判序；worktree 政策開啟→分「可平行（各開 session 跑 apply-with-worktree，多 session 配方）／須依序」，關閉→單一建議順序；純技能文指示、不加引擎指令
- 兩需求收進同一個 change，共用 MARKER_VERSION／golden／assets.lock 三連動，避免平行版號對撞
**Open**: 無——進入結論

## Conclusion

**Decision**: 兩需求收進同一個 change，五個改動面：
1. propose.md 收尾加盤點環節：提案中 change ≥2 時以 speclink list --json 盤點，硬信號（delta capability 重疊→須依序，封存合併閘會拒絕後到者）＋軟信號（proposal／tasks 推測程式碼重疊或依賴→建議依序）判序；worktree 政策開啟（config.yaml worktree: true，含 SPECLINK_WORKTREE 覆寫）→分「可平行：各開一個 session 跑 apply-with-worktree（沿用多 session 配方）」與「須依序」兩組；政策關閉→給單一建議順序。純技能文指示，不加引擎指令。
2. apply.md 收尾：全部勾完的出邊補「不跑品質站→直接 /speclink:archive，或 /speclink:commit 的『先封存再一起提交』一步到位」；完工報告模板（Output On Completion）同步措辭；僅剩 [M] 的路徑不動。
3. review.md／verify.md 出邊改兩條：主 checkout 落章→archive；worktree 內落章→補提交蓋章 meta→worktree-merge；拿掉「另一站（若要）」交叉提醒。quality.md 不動（已合流程）。
4. archive.md 尾端加收尾提交提醒——單一位置蓋住所有進 archive 的路（apply 直達、review、verify、quality、worktree-merge 後）。
5. canon deltas：skill-routing spec MODIFIED（Example 交棒表 propose row 與 review/verify row、「archive 為流程終點不帶出邊」句改為得帶一條收尾提交提醒）；archive-skill spec 補敘明。asset 改動照例 MARKER_VERSION／golden／assets.lock 三連動，speclink update 再生全部 SKILL.md。
**Rationale**: 提交提醒放 archive 尾端一點蓋全路，只寫在 apply 出邊會漏掉走品質站的進路；archive+commit 一步到位沿用 commit.md 既有「Archive first, then commit together」子流程，不造新機制；排序硬信號用 delta 目錄重疊即機械可查，免引擎新 verb（YAGNI）。
**Rejected alternatives**: 排序判斷做成引擎新指令（範圍過大，agent 讀檔即可）；只在 apply 出邊寫 archive+commit 而不動 archive（品質站進路漏提醒）；review/verify 保留另一站交叉提醒（噪音，使用者裁定拿掉）；每次 propose 都展開盤點（單一 change 常見情境多噪音）。
**Deferred**: asset 字面措辭；propose 盤點環節的 requirement 歸屬（skill-routing 或 propose-skill spec 新增）——留待 propose 階段起草 delta 時定。
**Capture to**: proposal（轉出新 change）
**Next**: /speclink-propose --from-discussion propose-apply-handoff-updates
