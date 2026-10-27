---
topic: merge worktree 引擎與技能的疑問：worktree 內封存再 merge 的順序、防呆與指令檔指引
slug: worktree-archive-merge-order
status: promoted
promoted_to: worktree-flow-guards-and-guidance
created: 2026-08-06
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: merge worktree 引擎與技能的疑問：worktree 內封存再 merge 的順序、防呆與指令檔指引

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 worktree 流程收尾階段（看板上 instruction-downgrade-guard 已 done 且掛著 worktree）提出對 merge worktree 引擎＋技能的疑問，第一個是「在 worktree 中直接封存再 merge 可以嗎、會不會壞掉」。模式：assumptions——相關原始碼充足（crates/speclink-host/src/worktree.rs 的 overlay／teardown 機制、crates/speclink-core/src/archive.rs 的封存引擎、.claude/skills/speclink-worktree-merge/SKILL.md）。相關 change：instruction-downgrade-guard（done、worktree 掛載中）。相關既有討論：worktree-parallel-apply（LANGUAGE.md 記載 worktree 直出裁定）。討論範圍隨第二則訊息擴大到：流程順序正典化、archive 防呆落點、指令檔生成引擎的 worktree 段落指引。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-06)

**Focus**: 在 worktree 內直接 `speclink archive` 再 merge 回主分支，是否安全？
**Position**: git 層面不會炸（封存＝tracked 檔案搬移＋修改，可乾淨 merge），引擎也不擋（archive.rs 無任何 worktree 判斷），但有三個真實的坑，故正確順序是先 merge 再於主 checkout 封存：
- 坑 1／解封存備份蒸發：archive 把套用前正典規格備份寫進 `.speclink/snapshots/`（archive.rs:474-483，for unarchive support），`.speclink/` 是 gitignored——備份只存在 worktree 磁碟上，`git worktree remove` 時（ignored 檔不擋移除）一併消失。
- 坑 2／delta 套在分支點的舊規格上：worktree 的 `openspec/specs/` 停在開分支當下；主分支若有其他 change 先封存過同一正典規格，輕則 merge 時規格檔衝突要人工解，重則 git 無衝突但語意錯誤。先 merge 再封存則 delta 永遠套在最新正典，整類問題不存在。
- 坑 3／過渡期主看板靜默降級：overlay 映射條件之一是「worktree 內該 change 目錄可讀」（worktree.rs:150）；worktree 內一封存目錄即搬走→映射掉落→主看板顯示過期主副本（似進行中、[worktree] 標示消失），且 teardown_blockers 同源失效——worktree 未收、關閉政策的擋門先消失。
- 次要：archive 的討論自動封存判斷（still_referenced）只看眼前副本，看不到主分支後來新 link 的兄弟 change，可能提早封存討論；在 worktree 先封存也放棄了對合併後結果跑品質站的機會。
**Ruled out**: 「在 worktree 內封存再 merge」作為支援流程——git 不會壞但上述坑無一有防護；merge 技能第 6 步 handoff 已明示 merge 後才走品質站／封存。
**Open**: 引擎要不要對「在 worktree 內 archive」加防呆？品質站（review／verify）在 worktree 內執行的定位？指令檔生成（CLAUDE.md／AGENTS.md）要不要寫入 worktree 流程順序指引？

### Round 2 — assumptions (2026-08-06)

**Focus**: 品質站（review／verify）應在 worktree 內完成、兩章過了才准 merge，還是 merge 後在主 repo 跑？
**Position**: 使用者主張（兩站在 worktree 內完成是基本要求）成立，且程式碼證據顯示它比上一輪「merge 後跑站」的建議更自洽——修正正典順序為：worktree 內 apply → review ∥ verify → 蓋兩章 → commit → merge；封存仍只在主 checkout：
- 工單即刪已證實：蓋章與刪工單是同一原子動作（review.rs write_stamp 先 delete review.md 再寫 meta 章，中斷寧退回未審查；verify-station-parity 提案明寫 verify stamp＝蓋章＋刪工單）——蓋完章的 worktree 沒有工單殘留，merge 帶回的只有 meta 裡的章（tracked）。
- 章跨 merge 的行為：雙錨（任務錨＋內容指紋錨）不因 parity 改變；ff 或不重疊 rebase 落地時 scope 檔內容逐位元組相同 → 章維持 Fresh；重疊 rebase 改寫落地內容 → 章轉「其後有變動」——這是正確訊號（審過的不是落地的），非缺陷。
- sidecar 自洽：Apply baseline 於 apply 起點寫入 worktree 的 .speclink/review-scopes/<change>/baseline.json（change_diff.rs prepare）；站在 worktree 跑時 baseline 在場、機制完整。反之 merge 後在主 repo 跑站，主 repo 無 baseline（apply 在 worktree 跑的）→ scope 解析降級（"no Apply baseline was captured"）。
- stamp／discard 後 host-local review snapshots 即清（baseline 保留）——worktree 移除時無有價值資料損失。
**Ruled out**: 上一輪「品質站留到 merge 後在主 checkout 跑」的建議——那才是降級路徑（主 repo 缺 Apply baseline）；「verify-station-parity 會解掉章跨 merge 變黃」的假設——parity 不改雙錨語意，但該行為本來就只在「落地內容≠審過內容」時觸發，是要保留的正確警示。
**Open**: worktree-merge 技能的「兩章俱在」preflight 強度（硬擋 vs 提示）？verify-station-parity 未落地的過渡期只有審查章，preflight 檢查什麼？波及面：merge 技能第 6 步 handoff 文案、生成指令檔 worktree_lines、archive 技能文案、archive 引擎防呆（上輪已定方向）。

### Round 3 — assumptions (2026-08-06)

**Focus**: 蓋章編排的另一解（quality-skill-canonicalization）如何接上 worktree 流程，以及引導強度的最終裁定。
**Position**: 收斂為「引導不強制、決策權在使用者」：
- quality-skill-canonicalization 與 worktree 流程是互補而非替代：其時序（兩站檢查先不蓋章 → findings 統一修正 → 各自複驗 → 兩章接連蓋 → 封存）放進 worktree 恰是收尾動作的正典編排——兩章在最末接連落下，蓋後零編輯窗口，也避開跨站互相打黃（cross-station-staleness）；它不改變章跨 merge 的指紋語意（該語意本就正確）。其尾步「封存」在 worktree 內會被新的 archive 防呆擋下、正確彈回 merge——協同運作。
- merge 前蓋章不作硬擋：workflow 正典本就把兩站標為可選（review? ∥ verify?，見生成 CLAUDE.md 的 Workflow 行）；不需要站的規格存在，使用者也可先 merge 再回主 repo 補跑兩站（接受 Apply baseline 缺席的降級）。
- 引導落在生成指令檔（政策開啟時）：init.rs 的 worktree_lines 之外，Workflow 段補一條 worktree 流程線（例：worktree: apply-with-worktree ⇄ ingest → (review? ∥ verify?) → worktree-merge → archive（主 checkout））＋一條「品質站建議在 worktree 內完成（Apply baseline 在場）；封存僅在主 checkout」的 bullet；worktree-merge 技能第 6 步 handoff 與 apply-with-worktree 收尾 handoff 同步改向。三連動成本（MARKER_VERSION／golden／assets.lock）為既知機械成本。
**Ruled out**: worktree-merge 的「兩章俱在」硬性 preflight（上輪 Open）——兩站在 workflow 正典中即為可選，硬擋違反其語意；唯一硬擋保留給 archive 引擎防呆（資料遺失級）。
**Open**: 無——進結論。與 quality-skill-canonicalization／verify-station-parity 在 init.rs 與 golden 的落地順序屬機械協調，記入結論 Deferred。

## Conclusion

**Decision**: worktree 流程的正典順序定為「worktree 內 apply → (review? ∥ verify?，建議於 worktree 內完成；兩站都跑走 quality 編排) → commit → worktree-merge → 封存一律在主 checkout」，以「引導不強制」落地：(1) archive 引擎加唯一硬防呆——workspace root 的 .git 是檔案（linked worktree 特徵）且分支為 speclink/* 時拒絕封存並指路 worktree-merge；(2) worktree-merge 不加蓋章 preflight；(3) 生成指令檔（CLAUDE.md／AGENTS.md）於 worktree 政策開啟時注入 worktree 流程線與品質站指引 bullet，worktree-merge 與 apply-with-worktree 的 handoff 文案同步改向，archive 技能補「worktree 內不封存」提示。
**Rationale**: 品質站機制（Apply baseline、凍結面 sidecar）活在 worktree 的 .speclink/，站在 worktree 內跑才有完整機制，merge 後在主 repo 補跑是降級路徑（無 baseline）；蓋章即原子刪工單、章隨 meta 合回主分支，跨 merge 只在「落地內容≠審過內容」時轉黃——正確警示應保留。封存反向：snapshot 備份 gitignored 隨 worktree 移除蒸發（資料遺失級）、delta 須套最新正典、overlay 映射與 teardown 保護同源失效——三坑皆指向主 checkout。兩站在 workflow 正典中本為可選，故引導用文件、硬擋只留給資料遺失級的封存。
**Rejected alternatives**: worktree 內封存再 merge（三坑，資料遺失級）；「merge 後才跑品質站」作為建議預設（baseline 缺席的降級路徑，僅保留為使用者自選）；worktree-merge 的兩章硬性 preflight（站可選，硬擋違反 workflow 語意）；防呆延伸至品質站動詞（無資料遺失後果，過度綁手）。
**Deferred**: 與 quality-skill-canonicalization、verify-station-parity 在 init.rs workflow 行與 golden 的落地順序協調（機械衝突，後落地者重整）；主 repo 補跑站時 baseline 缺席警告是否要指路 worktree 流程（未議）；verify-station-parity 落地前 verify 無章的過渡期不影響引導文字（流程線只寫站名不涉章）。
**Capture to**: proposal（promote 成新 change）
**Next**: /speclink-propose --from-discussion worktree-archive-merge-order（或 speclink discuss promote worktree-archive-merge-order --name worktree-flow-guards-and-guidance）
