## Why

worktree apply 收尾時，執行代理照唸 W3 逐字交棒句給使用者。這句話目前只列 `/speclink-review` ∥ `/speclink-verify` 兩個品質關卡入口，漏了 `/speclink-quality`（兩站合跑）。同一 repo 的其他三處字面——同檔 agent 端 Next steps、apply 完工模板、skill-routing 正典傘詞「品質站（review、verify 或 quality）」——都含 quality，唯獨使用者實際看到的這句沒有。quality 技能在 worktree 內可用（其出邊已有 worktree 分支），沒有排除理由。目標使用者是透過 AI 代理跑 SDD 的開發者，情境是 apply-with-worktree 完工後的品質關卡階段：入口清單不完整，兩站合跑的路徑被埋沒。

前一個變更 propose-apply-handoff-updates 的 Non-Goals 明寫不動此資產字面——當時只檢查了「可略過品質站」路徑，未檢查入口清單完整性。本變更源自討論 worktree-handoff-quality-mention 的結論。

落點修正（相對討論結論）：結論原指定在 skill-routing 新增交棒 scenario；propose 階段掃描發現 scenario 級字面要求已存在於 worktree-apply-skill 的「apply-with-worktree 技能的收尾指示」requirement，且其字面同樣只列兩站——正典釘子因此落在該 requirement，skill-routing 不動（其 Example row 用傘詞「品質站」，詞義已含 quality，不重複釘兩處）。結論「釘正典字面、防止回歸」的意圖不變。

## What Changes

- worktree 收尾段資產（crates/speclink-core/assets/skills/apply-worktree-post.md）的 W3 逐字交棒句：在 `/speclink-review`（工藝品質）∥ `/speclink-verify`（規格符合度）之後補列 `/speclink-quality`（兩站合跑）。同句其餘部分（Apply baseline 提醒、蓋章補提交、worktree-merge 收尾）字面不動。
- worktree-apply-skill 正典的「apply-with-worktree 技能的收尾指示」requirement：品質關卡括號自「review ∥ verify」展開為三入口（review ∥ verify，或 quality 兩站合跑）；scenario「內文含停點與正典順序交棒指示」的 THEN 同步要求交棒段明列三入口。scenario 名稱不變。
- 品質站 round 1 的入口名單同步（review 站 possible Shotgun Surgery 觀察的順手修）：apply 資產的 `[M]` 段（crates/speclink-core/assets/skills/apply.md）與 worktree-merge 資產的降級路徑句（crates/speclink-core/assets/skills/worktree-merge.md）同樣補列 quality 入口，三處入口名單字面同調。
- 資產內文異動走慣例三連動：speclink-core 的 ASSET_VERSION 版號遞增（crates/speclink-core/src/init.rs）、golden snapshot 更新（crates/speclink-core/tests/golden/claude-worktree.snapshot.md 為主要內文變動；claude、codex、neutral-cli、neutral-tool-call 四份的版號行同步再生，入口名單字面依 profile 收錄的技能落點跟動）、assets.lock 重生（crates/speclink-core/tests/golden/assets.lock）。
- 技能再生影響 claude 與 codex 兩個工具目標的 apply-with-worktree、apply、worktree-merge 技能檔；版號遞增會讓全部技能檔的版號行同步再生，內文異動僅上述三技能。

相容性影響：CLI 指令的人眼輸出與 `--json` shape 皆不變；golden 異動屬刻意變更——入口名單補列 quality 的字面依各 profile 收錄的技能落在對應 snapshot，隨本變更同批更新。既有使用者無遷移動作，跑 speclink update 即取得新字面。

## Non-Goals

- 不動 skill-routing 正典：其 apply-with-worktree Example row 的傘詞「品質站」已含 quality；scenario 級字面釘在 worktree-apply-skill，不在兩處重複維護同一字面。已否決討論結論原案「在 skill-routing 新增 scenario」，理由如上（落點修正，意圖不變）。
- 不動 quality 技能資產與 apply 本體完工模板——兩者字面已含三入口。
- 不動 W3 段的其他句子與 agent 端 Next steps——後者已含 quality。
- 不新增任何引擎指令、旗標或設定欄位。

## Capabilities

### New Capabilities

（無——掃描確認落點由既有 worktree-apply-skill 正典覆蓋；skill-routing 管出口交棒傘詞、quality-skill 管 quality 技能本體，皆無需新 capability。）

### Modified Capabilities

- `worktree-apply-skill`: 「apply-with-worktree 技能的收尾指示」requirement 的品質關卡入口清單自兩站展開為三入口，scenario 字面同步要求明列三入口。

## Impact

- Affected specs: worktree-apply-skill（修改）
- Affected code:
  - Modified: crates/speclink-core/assets/skills/apply-worktree-post.md、crates/speclink-core/assets/skills/apply.md、crates/speclink-core/assets/skills/worktree-merge.md、crates/speclink-core/src/init.rs、crates/speclink-core/tests/golden/assets.lock，及 5 份 golden（crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md——後四份以版號行再生為主）
  - 再生衍生物（不手改）: apply-with-worktree、apply、worktree-merge 三技能在 .claude/skills/ 與 .agents/skills/ 下的六份 SKILL.md（內文異動），及版號行同步再生的其餘技能檔
- Affected crates: speclink-core（資產與 golden 測試）
