---
topic: improve 技能把專案結構整理列為其中一項摩擦訊號，並內建它自己的准入判準
slug: improve-layout-signal
status: promoted
created: 2026-09-10
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: improve-layout-signal
---

# Discussion: improve 技能把專案結構整理列為其中一項摩擦訊號，並內建它自己的准入判準

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者於 2026-09-10 在 improve-repo-layout 討論收尾後提出：使用者用 `/speclink-improve` 時，技能應該內建一套整理專案結構、資料夾結構的標準，讓任何專案的使用者都拿得到，不必像這次靠掃描者臨場改寫判準。使用者裁定：列為 improve 的其中一項（與既有五個摩擦訊號並列），不是獨立模式。

**現況**：技能正典 crates/speclink-core/assets/skills/improve.md（再生為 .claude/skills/speclink-improve/SKILL.md）只有一套判準——結構深化五訊號＋刪除測試（刪掉抽象後複雜度是否集中）。資料夾搬移不減少程式碼複雜度，照字面過不了刪除測試；improve-repo-layout 是在 Context 裡自行改寫准入判準（「第一次打開的人能否不靠 grep 預測功能住哪、哪些檔是一組」）才做下去，該改寫只活在那份記錄裡。先例 cli-verb-family-modules（improve，2026-08）已用同型判準通過。

**這次實證出來的六條做法**（improve-repo-layout Round 3–11）：准入測試改為讀者預測性；分組依據要有客觀來源（引用方向、架構文件分層、README 連結）；對外路徑不變（根層 re-export、舊路徑守門斷言）；硬限制先盤點（公開網址、tag 觸發 workflow、相對 include、path 相依）；測試鏡射搬移（含內嵌測試的行數門檻）；切刀由路徑相依決定、不用 worktree 平行。

**掃描者曾提出的顧慮**：讀者預測性測試與刪除測試是兩套不同的門，混成一項後一般掃描可能把純搬家當改進提出。使用者仍裁定列為一項；顧慮的解法留給本討論的 Round 1。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — interview (2026-09-10)

**Focus**: 「專案結構整理」以第六個摩擦訊號的形式進技能，它的准入判準怎麼寫才不會讓一般掃描把純搬家當改進。
**Position**: 第六訊號的字面：「**分組只靠檔名猜**：同一個概念的檔案散在平鋪目錄裡，讀者要靠前綴或字母序重拼分組；或目錄與架構文件的分層對不上。」它帶自己的准入測試，寫在刪除測試旁邊：「對第六訊號，刪除測試改問讀者預測測試——第一次打開這個目錄的人，能不能不靠搜尋就預測功能住哪、哪些檔案是一組；分組必須有客觀依據（引用方向、既有架構文件、對外連結），且對呼叫者不可見（路徑經 re-export 或守門斷言維持不變）。只是換位置擺、呼叫端要跟著改路徑的，不算。」這樣一般掃描仍以刪除測試守其餘五訊號，第六訊號用自己的門，兩套門不混。候選五欄位不變；Recommendation 的三強度不變。Step 5 的介面深度四項對第六訊號改問：(1) 分組邊界依據什麼客觀來源；(2) 對外路徑靠什麼維持不變；(3) 搬移前盤點了哪些硬限制（公開網址、tag 觸發 workflow、相對 include、path 相依）；(4) 讀者預測測試——刪掉分組回到平鋪，讀者失去什麼。測試與切刀兩條做法寫進 Step 5 的補述（測試鏡射搬移、切刀先動目錄本身、不用 worktree 平行）。
**Ruled out**: 獨立模式（`/speclink-improve --layout`）——使用者裁定列為一項；只在 Context 範例裡提一句——不進判準的字面，下一個掃描者仍會被刪除測試擋住；把讀者預測測試直接取代刪除測試——其餘五訊號會失去「集中 vs 搬移」的分辨力。
**Open**: 規格與技能檔的落點——improve-skill 規格補一條 requirement（技能檔 SHALL 列第六訊號與其准入測試、深度四問的對應版）；技能正典 improve.md 改動後 bump ASSET_VERSION、四份 golden 與 assets.lock 同批更新（記憶：三連動）。

## Conclusion

**Decision**: `/speclink-improve` 的摩擦訊號從五個增為六個，第六個是「分組只靠檔名猜」（同一概念的檔案散在平鋪目錄、讀者靠前綴或字母序重拼分組，或目錄與架構文件分層對不上）。第六訊號帶自己的准入測試，寫在刪除測試旁：讀者預測測試（第一次打開的人能否不靠搜尋預測功能住哪、哪些檔是一組）＋分組須有客觀依據（引用方向、既有架構文件、對外連結）＋對呼叫者不可見（re-export 或舊路徑守門斷言維持既有路徑）；只換位置、呼叫端要改路徑的不算。Step 5 的介面深度四問對第六訊號改為：分組邊界的客觀來源、對外路徑如何維持、搬移前盤點的硬限制（公開網址、tag 觸發 workflow、相對 include、path 相依）、回到平鋪讀者失去什麼；補述測試鏡射搬移與切刀規則（先動目錄本身、不用 worktree 平行）。其餘五訊號與刪除測試字面不變。落點：技能正典 crates/speclink-core/assets/skills/improve.md 改字面並 bump ASSET_VERSION、四份 golden 與 assets.lock 同批；規格 improve-skill 補一條 requirement 與 scenario（技能檔缺第六訊號或其准入測試時 validate 抓到）。
**Rationale**: 使用者要的是任何專案的使用者跑 improve 都拿得到這套標準，所以它必須進技能字面，不能留在單一討論記錄；列為一項而非獨立模式是使用者裁定。兩套門並列而非取代，是為了保住其餘五訊號「集中 vs 搬移」的分辨力。判準六條全部來自 improve-repo-layout 的實證（五個候選、11 輪），不是憑空設計。
**Rejected alternatives**: 獨立模式或旗標——使用者裁定列為一項；只在 Context 範例提一句——不進判準字面，下一個掃描者仍被刪除測試擋住；讀者預測測試取代刪除測試——其餘五訊號失去分辨力；只改 SKILL.md 不改正典 asset——`speclink update` 再生時會被蓋回。
**Deferred**: 第六訊號要不要也進 `/speclink-review` 的 Standards 軸（審查時提醒新檔落點）——另案；技能文字是否附一個最小範例（如 identity_sqlite.rs 的前綴編碼）——propose 期依技能檔篇幅決定。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion improve-layout-signal
