## Why

收尾交棒句是透過 AI 代理跑 SDD 的開發者在各工作流階段的導航。現行字面有五個缺口：propose 完成後不盤點已提案變更的執行順序，worktree 政策開啟時無法判斷哪些可平行；apply 完工後「不跑品質站可直接封存」的路徑不明顯——skill-routing 正典已要求「或直接封存」，資產字面未跟上；review 與 verify 落章後交叉推銷另一站形成噪音，且缺 worktree 分支，代理可能建議被引擎拒絕的封存；archive 收尾無提交提醒，走品質站進入封存的使用者會漏掉提交；discuss 結論後並列轉出與提案兩個入口，角色分工不清。本變更源自討論 propose-apply-handoff-updates 的結論與其後追加確認的第三需求。

## What Changes

- propose 技能資產（crates/speclink-core/assets/skills/propose.md）收尾新增盤點環節：提案中（未開工）變更 ≥2 個時，以 list 動詞的 JSON 輸出盤點全部提案中變更並判定執行順序。硬信號為 delta capability 重疊——兩個變更的 delta 目錄含同一 capability 即須依序，因為封存合併閘會拒絕後到者。軟信號為讀 proposal 與 tasks 推測的程式碼重疊或依賴。worktree 政策開啟（openspec/config.yaml 的 worktree 欄位，含 SPECLINK_WORKTREE 環境覆寫）時分兩組：「可平行——各開一個 session 走 apply-with-worktree，沿用既有多 session 配方」與「須依序」；政策關閉時給單一建議順序。純技能文指示，不新增引擎指令。
- apply 技能資產在全部勾完的出邊補明確的跳過品質站路徑：直接走 archive 技能，或走 commit 技能既有的「Archive first, then commit together」子流程一步到位；完工報告模板（Output On Completion）同步措辭。僅剩 [M] 手動任務的路徑不變。
- review 與 verify 技能資產的落章出邊改為兩條：主 checkout 落章走 archive；worktree 內落章先補提交蓋章的 meta 異動、再走 worktree-merge。移除「另一站（若要）」交叉提醒。quality 技能資產不動——現行已符合此流程。
- archive 技能資產尾端新增收尾提交提醒，單一位置涵蓋所有進入封存的路徑（apply 直達、review、verify、quality、worktree-merge 之後）。
- discuss 技能資產的結論邊改為單推 propose 的 --from-discussion 入口；promote 教學保留於中途轉出段。理由：已結論時 promote 只預填 Why、產物仍須跑 propose，等於一步拆兩步。
- openspec/LANGUAGE.md「轉為變更」詞條定義微調：promote 的主場改為未結論的中途轉出，定義不再限定「已結論的討論」。
- 對應 canon deltas：skill-routing（Example 交棒表與 archive 終點句）、propose-skill（新增盤點環節 requirement）、archive-skill（新增收尾提交提醒敘明）、discuss-skill（新增結論後交棒路由敘明）。

## Non-Goals

- 不新增任何引擎指令、旗標或 JSON 欄位；排序判斷完全由代理讀檔完成。已否決「做成引擎新指令」：範圍過大且非必要。
- 不改 quality 與 commit 技能資產的內文；archive+commit 一步到位沿用 commit 技能既有子流程，不造新機制。
- 不把提交提醒只寫在 apply 出邊。已否決：走品質站進入封存的路徑會漏提醒。
- 不保留 review／verify 的「另一站（若要）」交叉提醒。已否決：使用者裁定為噪音。
- 不在每次 propose 都展開盤點。已否決：單一變更的常見情境徒增噪音。
- 不動 worktree 收尾段資產（crates/speclink-core/assets/skills/apply-worktree-post.md）的字面：其 W3 交棒句已含略過品質站的路徑。

## Capabilities

### New Capabilities

（無——step 3 掃描確認全部落點皆有既有 spec 覆蓋：skill-routing 管出口交棒、propose-skill／archive-skill／discuss-skill 管各自技能內文。）

### Modified Capabilities

- `skill-routing`: 「出口交棒由技能結尾承載」requirement 更新——Example 交棒表的 propose row 加盤點環節、review/verify row 改為落章直達 archive（worktree 內走 worktree-merge）、新增 archive 導向 commit 的 row；「archive 為流程終點不帶出邊」句改為 archive 得帶一條收尾提交提醒。
- `propose-skill`: 新增收尾盤點環節 requirement——觸發條件（提案中變更 ≥2）、排序硬軟信號、worktree 政策分流。
- `archive-skill`: 新增收尾提交提醒 requirement——技能檔尾端敘明封存完成後提醒提交。
- `discuss-skill`: 新增結論後交棒路由 requirement——結論邊單推 propose 的 --from-discussion 入口，promote 留給中途轉出。

## Impact

- Affected specs: skill-routing、propose-skill、archive-skill、discuss-skill（皆為修改，無新 capability）。
- Affected code:
  - Modified:
    - crates/speclink-core/assets/skills/propose.md
    - crates/speclink-core/assets/skills/apply.md
    - crates/speclink-core/assets/skills/review.md
    - crates/speclink-core/assets/skills/verify.md
    - crates/speclink-core/assets/skills/archive.md
    - crates/speclink-core/assets/skills/discuss.md
    - crates/speclink-core/src/init.rs（ASSET_VERSION 由 v1.21.0 遞增）
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/speclink-core/tests/golden/assets.lock
    - openspec/LANGUAGE.md
  - New:（無）
  - Removed:（無）
- 影響 crate：僅 speclink-core（內嵌技能資產、版號常數與 golden 測試）。目標使用者為透過 AI 代理跑 SDD 的開發者；影響 claude 與 codex 兩工具生成的技能：propose、apply、apply-with-worktree（apply 本體同源組合，位元隨之變動）、review、verify、archive、discuss。
- CLI 指令：無新增或變更；人眼輸出與 JSON shape 皆不變。
- 相容性影響：技能檔生成內容改變屬刻意變更，golden 快照與 assets.lock 同批更新並以本提案記載；技能檔 frontmatter 版號隨 ASSET_VERSION 遞增，既有安裝以 update 動詞再生即可，無遷移動作。再生的各工具技能檔不列入本變更 evidence，收尾以 git status 盤點。
- 設定欄位：無新增；沿用既有 worktree 政策欄位。
