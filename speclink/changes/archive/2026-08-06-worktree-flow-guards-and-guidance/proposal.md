## Why

討論 worktree-archive-merge-order（2026-08-06）證實 worktree 流程存在一個資料遺失級的缺口與一段缺席的引導：在 linked worktree 內執行封存不會被任何機制擋下，但解封存備份寫進 worktree 的 gitignored `.speclink/snapshots/`，會隨 `git worktree remove` 一併蒸發；delta 也會套在分支點的過期正典上；主看板的 overlay 映射與 teardown 保護同源靜默失效。同時，品質站機制（Apply baseline、凍結面 sidecar）活在 worktree 的 `.speclink/`，兩站在 worktree 內執行才有完整機制，但 worktree-merge 技能的交棒文案現在指向相反方向（merge 後才跑品質站），且生成指令檔（CLAUDE.md／AGENTS.md）沒有任何 worktree 流程線——代理無從得知正典順序。

## What Changes

- **封存引擎防呆（唯一硬擋）**：封存動詞於 linked worktree 內執行時拒絕——判定條件為 workspace root 的 .git 是檔案（linked worktree 特徵，與 worktree overlay 的主副本判準同源）且當前分支具 speclink/ 前綴；拒絕時零檔案效果、非零 exit code，stderr 指路先以 worktree-merge 合回主分支再封存。git 不可用時 fail-open（沿 worktree discovery 慣例，無 git 的環境不得因此無法封存）。單筆與 bulk 封存同受此守門。
- **正典順序入生成指令檔**：worktree 政策開啟時，SPECLINK marker 區塊的 Workflow 段新增一條 worktree 流程線（apply-with-worktree ⇄ ingest → 品質站（建議於 worktree 內）→ worktree-merge → 主 checkout 封存），並新增一條指引 bullet 敘明「品質站建議在 worktree 內完成（Apply baseline 在場）；封存僅在主 checkout 執行」。政策關閉時 marker 內容維持現狀（既有兩行技能指引之外，新增內容同受政策閘控制）。
- **worktree-merge 技能交棒改向**：合併成功後的交棒提示自「續走品質站或封存」改為反映正典順序——品質站建議已於 worktree 內完成，完成（或使用者略過）則下一步為主 checkout 封存；未完成仍可於主 checkout 補跑，但屬降級路徑（主 checkout 無 Apply baseline）。
- **apply-with-worktree 技能收尾補品質站建議**：worktree 內提交完成後的交棒文字，自僅點名 worktree-merge 擴充為「建議先於 worktree 內執行品質站（review ∥ verify，兩站蓋章後補提交）再走 worktree-merge」；不合併、不移除 worktree 的停點不變。
- **archive 技能補 worktree 提示**：技能內文敘明封存於主 checkout 執行，worktree 內封存會被引擎拒絕並指路 worktree-merge。
- **生成物三連動**：以上 asset 與 init.rs 範本變更提升 MARKER_VERSION、再生 golden 快照（claude／claude-worktree／codex／neutral 各 target 與 assets.lock）、`speclink update` 刷新本 repo 的 CLAUDE.md／AGENTS.md 與技能檔——生成物不手改。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `change-lifecycle`: 新增「封存的 linked worktree 環境守門」需求——拒絕條件、零檔案效果、fail-open 邊界
- `workspace-tools`: 「marker 技能指引跟隨 worktree 政策」擴充——政策開啟時 Workflow 段含 worktree 流程線與品質站指引 bullet，關閉時不含
- `worktree-merge-skill`: 「worktree-merge 技能的收尾流程指示」的交棒步驟改向正典順序
- `worktree-apply-skill`: 「apply-with-worktree 技能的收尾指示」補品質站建議
- `archive-skill`: 技能敘述補「worktree 內不封存」提示

## Impact

- Affected specs: `change-lifecycle`、`workspace-tools`、`worktree-merge-skill`、`worktree-apply-skill`、`archive-skill`（皆修改）
- Affected code:
  - Modified: crates/speclink-core/src/archive.rs（封存防呆守門）、crates/speclink-core/src/command/mod.rs（runtime 於 `--mark-tasks-complete` 前置寫入前先守一次）、crates/speclink-core/src/init.rs（Workflow 段 worktree 流程線與指引 bullet、MARKER_VERSION 提升）、crates/speclink-core/assets/skills/worktree-merge.md（交棒改向）、crates/speclink-core/assets/skills/apply-worktree-post.md（收尾補品質站建議）、crates/speclink-core/assets/skills/archive.md（worktree 提示）、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/speclink-core/tests/golden/remote-claude.marker.md、crates/speclink-cli/tests/it/archive_readiness_gate.rs（防呆整合測試）、CLAUDE.md、AGENTS.md、全部技能檔（`.claude/skills/**`、`.agents/skills/**`——三個 worktree／archive 技能為內容變更，其餘隨 MARKER_VERSION 提升再生版號戳記；與 CLAUDE.md／AGENTS.md 同由 speclink update 產出）
  - New: crates/speclink-fs/tests/hostless_guard.rs（審查 Round 1 的重現測試：無 host workspace 的派發不得以行程 cwd 判定守門）
  - Removed: （無）
- 平行協調：quality-skill-canonicalization 與 verify-station-parity（皆進行中）同樣觸及 init.rs workflow 行與 golden——後落地者以先落地的正典為基準重整（討論結論 Deferred 明記）
