## Why

審查站目前把每一輪都當成新的探索：修正後會重新掃描整份 finding 檔案，因而持續產生新的 CRITICAL 與 SUGGESTION，沒有可預期終點。透過 AI 代理執行 SDD 的開發者需要在 apply 完成、archive 之前，對固定 change diff 做一次完整審查，後續只驗收修正且能明確通過或未通過。

## What Changes

- 把審查 Round 1 定義為唯一 discovery pass：針對凍結的 change hunks，平行執行 Standards 與 Correctness，保留兩軸原報告；spec 合規仍由驗證站負責
- 把 Round 2+ 定義為 remediation validation：只判定上輪未解事項是否修復，以及修正 patch 是否直接引入回歸，不重新探索未修改區域
- 以「必修集合嚴格縮小」作為自動續跑條件；第一個無進展輪立即以未通過停止、保留工單且不蓋章。只剩可裁事項時可由使用者明示接受並帶保留蓋章
- 新增 Host 端 change-diff resolver：以 touchedFiles 限定候選路徑，從可信 Git fixed point 比對至目前 worktree，涵蓋 staged、unstaged、rename、delete 與 touched untracked additions；解析並凍結 patch hash、before／after hashes 與 old/new hunk ranges
- Apply 技能在首次標記 in-progress 前呼叫新增的 review prepare 動詞，將 baseCommit、dirtyFilesAtStart、capturedAt 寫入 host-local baseline；baseline 與 review snapshots 不進 touched 記錄，也不進 TeamStore
- 新增 speclink review scope 子指令：可帶 --base、--candidate-hash 與可重複的 --include-hunk；自動可歸屬時凍結 scope，歧義時非零結束並列出候選 patch／hunk ID，只有使用者提供可信 base、以 candidate hash 錨定選取 hunks，或改用隔離 worktree 後才可繼續。輸入不使用 stdin
- 審查工單的新輪次帶 phase 與 patch hash；review show --json 增列 nullable 的 phase、patchHash，既有無欄位工單仍可讀，但缺少可重建 snapshot 的續輪不得假裝精確驗收
- 與修正 patch 無關的新事項不加入續輪；只有具現實觸發與重現／失敗測試／明確 invariant 證據的安全、資料損失或錯誤行為，才讓本站以範圍改變／未通過結束並另開 discovery
- claude 與 codex 的 apply／review 技能模板及 golden 同步更新

相容性影響：

- touched v1／v2 JSON、commit、archive、drift、verify evidence 與 speclink list --json 皆維持原形
- review add-round 的既有 stdin 文法仍可讀；新技能產生的輪次才附 phase／patch hash
- review show --json 為刻意的 additive shape change，protocol、remote client、server route 與 CLI 測試同批更新；人眼工單新增 phase／patch 行
- review prepare 與 review scope 為新增子指令；歧義、缺 Git baseline、candidate hash 漂移或 snapshot 缺失皆為非零 exit，且不得建立已解 scope
- 不新增 openspec/config.yaml、.speclink.yaml 或環境變數設定

## Non-Goals

- 不把 touchedFiles 改成 hunk store，不在檔名後附行數，也不要求既有 touched 消費者理解 patch
- 不建立 capture-apply-change-provenance，不攔截每一次 edit，不提供無 Git、跨 Host 或 workspace 前進後重播原始 Apply patch 的完整 provenance
- 不把驗證三維度塞回審查站；verify-station-parity 另以 ingest 採用相同首輪／續輪收斂契約
- 不改審查章、看板、抽屜、系統匣或 archive 的使用者介面
- 不以固定最大輪數、finding 總數或「模型再也找不到問題」作為通過條件

## Capabilities

### New Capabilities

- change-diff-scope: Apply baseline、Git-backed change hunk 解析、歧義 fail-closed、review snapshot 與 old/new hunk ranges 的 Host 契約

### Modified Capabilities

- review-skill: 首輪 Matt-compatible discovery、續輪 remediation validation、無進展終止與重大晚發問題逃生口
- review-station: 工單輪次與 frozen patch 的 phase／patch-hash 綁定，以及既有工單相容行為

## Impact

- Affected specs: change-diff-scope（新增）、review-skill（修改）、review-station（修改）
- Affected code:
  - New: crates/speclink-host/src/change_diff.rs
  - Modified: crates/speclink-host/src/lib.rs、crates/speclink-core/src/workspace.rs、crates/speclink-core/src/review.rs、crates/speclink-core/src/skills.rs、crates/speclink-core/assets/skills/apply.md、crates/speclink-core/assets/skills/review.md、crates/speclink-core/tests/golden/、crates/speclink-cli/src/main.rs、crates/speclink-cli/src/commands.rs、crates/speclink-cli/src/remote_commands.rs、crates/speclink-cli/tests/it/review_verbs.rs、crates/speclink-cli/tests/it/remote_verb_parity.rs、crates/speclink-protocol/src/command.rs、crates/speclink-remote/src/client.rs、crates/speclink-remote/tests/it/typed_client.rs、crates/speclink-server/src/routes.rs、crates/speclink-server/tests/it/review_api.rs
  - Removed: 無
