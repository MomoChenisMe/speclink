---
topic: Apply provenance 是否應全面記錄 change hunks，或以 touched files 搭配 Git 比對為主
slug: apply-provenance-scope
status: promoted
promoted_to: converge-review-remediation-rounds, verify-station-parity
created: 2026-08-03
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: Apply provenance 是否應全面記錄 change hunks，或以 touched files 搭配 Git 比對為主

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

前一份 code-review-convergence-boundary 討論先暫定另立 capture-apply-change-provenance，以完整 ChangeDelta 提供 old/new hunk ranges；後續影響盤點發現 touchedFiles 已被 task evidence、archive、commit、drift、remote protocol、server 與 UI 多層消費，而全面 hunk provenance 仍無法單獨判定 dirty worktree 中同檔既存修改或多個 active changes 的意圖歸屬，因此需在正式轉為變更前重新裁定範圍。本討論採 assumptions 模式：已檢視 crates/speclink-core/src/tasks.rs、crates/speclink-core/assets/skills/review.md、crates/speclink-host、crates/speclink-protocol 與 crates/speclink-server/src/routes.rs 等 3 個以上來源，並確認既有 change verify-station-parity 仍在進行中。使用者確認四項前提全數成立：主要目標是讓審查準確定界並收斂；大多數本地審查有 Git checkout；同檔跨 change 重疊屬例外且可 fail closed（安全地拒絕猜測）；commit、archive、drift 維持檔案層級。目標是裁定全面 Apply hunk provenance 與 touchedFiles＋Git 比對的成本效益、介面落點及後續 capture；本輪不實作。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-03)

**Focus**: 為了讓審查精確鎖定 change hunks，是否值得把 touched 全面改造成 Apply provenance
**Position**: 使用者確認背景中的四項前提全數成立，採 touchedFiles＋Git 為主、provenance 僅補特殊環境的混合方案：
- touchedFiles 維持 change 級候選檔案歸屬，不改成 hunk store，也不讓 commit、archive、drift 等既有消費者理解行號或 patch
- Apply 只需留下輕量 fixed-point metadata：至少 baseCommit、dirtyFilesAtStart 與 startedAt；不在每次修改時攔截或累積完整 patch
- 審查 Round 1 由 Host 端 change-diff resolver 以 touched paths 限定 Git 差異，使用 git diff --find-renames <fixed-point> -- <touched-paths> 比對 fixed point 到目前 worktree；不得沿用 git diff <base>...HEAD，因其只比較 commits，會漏掉尚未提交的 Apply 修改
- Git 未追蹤且屬 touched 的檔案視為整檔新增；改名、刪除與多段修改保留原生 patch 語意。解析後凍結 patch hash、before/after content hashes，以及每個文字 hunk 的 oldStart、oldLines、newStart、newLines，供首輪 discovery 與後續是否失效判定
- dirtyFilesAtStart 已含 touched path，或多個 active changes 對同一路徑重疊時，resolver 不猜測意圖並 fail closed；要求明示 hunk 排除、改給可信 fixed point，或改用隔離 worktree
- Round 2+ 的修正差異由審查／驗證站在修正前後各凍結小型 snapshot，僅用於 remediation validation，不回寫成全面 Apply provenance
- seam 應是 Host 端 change-diff resolver：有 Git workspace 時使用 Git adapter，本地與遠端 Host 共用相同輸出契約；只有無 Git、跨 Host／跨 session 仍要求精確重播時，才新增可選 provenance adapter。刪掉可選 adapter 只會失去這些特殊情境，Git-backed 審查仍成立，因此現階段不值得全面重構核心 touched
- 若日後將 remote／無 Git／跨 session 精確重播列為硬性產品需求，另立的 provenance 契約仍 SHALL 包含 old/new hunk ranges、before/after hashes 與可重建 patch 或 snapshot reference；本討論不弱化前一份討論對該契約的精確度要求
**Ruled out**: 全面把 touched 改成逐 hunk provenance——跨 core、host、protocol、server、desktop、UI 與 skills，成本高且仍無法從混合 dirty diff 推出意圖；在 touched file 字串後加行數——無法穩定表示插入、刪除、改名與多 hunk；直接使用 <base>...HEAD——會漏掉未提交 worktree；把 touchedFiles 當成精確 ownership——它只能先縮小候選檔案
**Open**: 無產品決策未解；可選 provenance 的儲存與傳輸格式只在 remote／無 Git／跨 session 精確重播成為硬需求時另案設計

## Conclusion

**Decision**: 不全面把 touched 改造成 change-hunk provenance；採「檔案歸屬＋Git 解差異＋必要時才有 provenance adapter」：
- touchedFiles 保持版本相容的 change 級候選檔案清單，commit、archive、drift、evidence 與 UI 等既有消費者維持檔案層級
- Apply 僅補 baseCommit、dirtyFilesAtStart、startedAt 等輕量 baseline metadata，不攔截每一次 edit，也不在 touched entries 內儲存 patch
- Host 端建立 change-diff resolver。有 Git workspace 時，以 touched paths 限定 fixed point 到目前 worktree 的 diff，包含 staged／unstaged，另將 touched 且未追蹤的檔案納為整檔新增；使用 rename-aware 比對，不使用會漏掉 worktree 的 <base>...HEAD
- resolver 輸出在審查 Round 1 凍結：patch hash、before/after hashes，以及文字 hunks 的 oldStart、oldLines、newStart、newLines。審查與驗證針對同一 frozen scope 執行首輪 discovery；Round 2+ 另凍結修正前後 snapshot，僅驗收既有 findings 與修正直接引入的回歸
- touched path 在 Apply 開始前已髒，或與另一 active change 重疊時，視為 ambiguous 並 fail closed；使用者須提供可信 fixed point、明示排除 hunks 或使用隔離 worktree
- 無 Git、跨 Host，或原 workspace 已前進後仍要求重播原始 Apply 差異時，才考慮可選 provenance adapter。若未來成為硬性需求，其契約 SHALL 保留 old/new hunk ranges、before/after hashes 與可重建 patch 或 snapshot reference
- 本決策收窄 code-review-convergence-boundary 結論中「立即另立 capture-apply-change-provenance」的落點：目前不建立該全面 provenance change，其精確資料契約只作未來特殊情境的最低要求
**Rationale**: 審查真正需要的是一份可凍結、可重現且限於 change 的 patch，不是讓所有 touched 消費者都理解 hunks。Git-backed Host resolver 能以較小改動取得主要效益，並把最難的 base selection、未追蹤／改名處理、歧義偵測與 patch freezing 放在真正看得見 workspace 的介面。全面 provenance 橫跨 core、host、protocol、remote、server、desktop、UI、skills 與測試，卻仍不能憑混合 diff 自動推導意圖；刪除可選 provenance adapter 後一般本地及有 Git 的遠端 Host 審查仍可運作，證明它目前不應成為核心 touched 契約。
**Rejected alternatives**:
- touched 全面改為 hunks：成本與連動面過大，且未解決同檔既存修改或跨 change 重疊的歸屬問題
- touched file 後附行數：座標易漂移，無法完整表示插入、刪除、改名、binary 與多段 hunk
- 只靠 touched file 重審整檔：仍會讓首輪超出實際 change hunks，增加新 findings 與無限循環風險
- 使用 git diff <base>...HEAD：比較 commit graph，不包含尚未提交的 Apply worktree
- 現在就為所有 remote／無 Git／跨 session 情境建立完整 provenance：尚無硬性產品需求，不能通過刪除測試
**Deferred**: 無阻擋目前設計的產品決策。只有未來明定無 Git、跨 Host 或 workspace 前進後仍須精確重播原始 Apply patch 時，才另案決定 provenance 的版本、儲存、傳輸、binary 表示與保留期限；不得弱化已定的 old/new ranges、hash anchors 與可重建性。
**Capture to**:
- 新 change converge-review-remediation-rounds：proposal／design／review-skill spec／tasks 納入輕量 Apply baseline、Host change-diff resolver、frozen Round 1 hunks、歧義 fail-closed 與後續 remediation snapshots
- 既有 change verify-station-parity：discussion link 後以 ingest 納入共用 frozen scope、續輪 snapshot 與 fail-closed 行為，不新增另一套 provenance
- 不建立 capture-apply-change-provenance；若 deferred 情境日後成為硬需求，再從本討論另轉出獨立 change
**Next**: 待使用者要求正式化時，先把本討論轉為 converge-review-remediation-rounds，再 link 至 verify-station-parity 並用 speclink-ingest 更新既有 artifacts；兩者完成後才進入 apply。
