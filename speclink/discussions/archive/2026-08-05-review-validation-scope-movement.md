---
topic: review 驗證範圍盲區的根治——以內容移動歸因取代 findings 點名
slug: review-validation-scope-movement
status: promoted
promoted_to: review-validation-scope-movement
created: 2026-08-05
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: review 驗證範圍盲區的根治——以內容移動歸因取代 findings 點名

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

review 站的驗證輪兩度撞上同型問題（記憶 review-validation-scope-gap，2026-08-04 golden 再生檔、2026-08-05 worktree-toggle-and-guards 手動附檔）：修復動到 findings 沒點名的鄰居檔時，第一輪漏出驗證面、下一輪凍結必卡 needsInput 死結（此形態無 candidateHash／hunk ids，只剩 stop 或 discard）。本討論裁定引擎面的根治方向。

模式：assumptions——已定位真碼 crates/speclink-host/src/change_diff.rs 的 resolve_validation_scope（行 629）、write_scope_snapshot（行 926）、Snapshot 結構（dirty_files_at_capture 僅存 PathHash，行 1083）。關鍵事實：快照存留至蓋章才清（行 739 註解），discovery 快照帶全候選檔 delta，重建「改之前內容」的資料已在磁碟上。

相關：change review-stamp-violet（進行中，review 站呈現面）；review skill 資產文本（needsInput 分支的處置指引會隨引擎行為改變）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-05)

**Focus**: 驗證輪的範圍計算基準要怎麼改，才能同時消滅「鄰居修復漏審」與「下一輪 needsInput 死結」
**Position**: 歸因基準從「findings 點名」改為「內容真的動了沒」，缺的 before 內容以快照鏈回走重建：
- 偵測：對上一輪快照的每個 dirty_files_at_capture 條目比對現況雜湊，動了才進驗證面（使用者裁定的基準）
- preserved 內（findings 檔）：走既有 remediation_segment，行為不變（change_diff.rs:687）
- preserved 外、但更早某輪快照收錄過：沿工單各輪 **Patch** 雜湊鏈回走，取最近一份收錄它的快照重建凍結後狀態（存的 text，或 base_commit＋delta 重放），diff(重建→現況) 以「adjacent 段」進驗證 patch——此即這次人工補救（git show＋git apply）的機械化，資料已在磁碟（快照存留至蓋章，discovery 快照帶全候選 delta）
- 任何輪都沒收錄過（discovery 時被使用者排除的檔）：註記＋放行（選項 A，使用者裁定）——排除本身是 discovery 的使用者處置，複驗不替已裁定的事再擋路；驗證輪的 needsInput 分支就此消滅
- patch 段帶出身標記（finding／adjacent／new），review skill 簡報對 adjacent 段要求評審確認歸屬——污染防護從「引擎拒絕凍結」移到「評審可見」
- 回走成本＝輪數（實務 2–4），無新增儲存
**Ruled out**: 對 never-captured 檔維持 needsInput（選項 B）——平行 session 改無關檔又會卡死複驗，正是要消滅的行為；只做雜湊移動偵測不重建 before——產不出可審的精確 diff（change_diff.rs:657 註解點名的正是缺 before）；每輪凍結都收錄全部髒檔內容——回走用既有快照即可，不必加寬儲存
**Open**: （無——本輪已定案）

## Conclusion

**Decision**: 驗證輪的範圍計算改以「內容移動」歸因：對上一輪快照 dirty_files_at_capture 的每個移動檔——preserved 內走既有 remediation_segment；preserved 外但更早輪快照收錄過的，沿工單 Patch 雜湊鏈回走、重建凍結後狀態、以 adjacent 段進驗證 patch；任何輪都沒收錄過的檔＝註記＋放行。patch 段帶 finding／adjacent／new 出身標記，review skill 簡報對 adjacent 段要求評審確認歸屬。驗證輪的 needsInput 分支消滅。
**Rationale**: 引擎只對 findings 點名檔存 before 內容（dirty_files_at_capture 僅 PathHash），鄰居修復第一輪漏審、第二輪死結（兩次實測）。重建所需資料本就在既有快照裡（存留至蓋章；discovery 帶全候選 delta），回走成本＝輪數。污染防護不丟棄，改由評審可見的 adjacent 標記承擔——fail-closed 縮回有使用者處置工具的 discovery。
**Rejected alternatives**: 對 never-captured 檔維持 needsInput（平行 session 改無關檔又卡死複驗）；只做雜湊偵測不重建 before（產不出可審 diff）；每輪收錄全部髒檔內容（既有快照已足，不加寬儲存）。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion review-validation-scope-movement
