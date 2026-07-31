---
topic: 變更從進行中退回提案中:機制與防呆
slug: revert-in-progress-to-proposed
status: promoted
promoted_to: revert-in-progress-to-proposed
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 變更從進行中退回提案中:機制與防呆

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因:使用者不小心對錯的變更開工(誤觸使卡片進入「進行中」),但看板沒有回頭路——跨欄拖曳是 spec 明文 no-op,一旦進入進行中就無法退回提案中。希望加一條受防呆保護的退回路徑,本地與 remote 模式都要有對應守門。

模式:假設模式——codebase scout 找到 6 處直接相關原始碼(packages/ui/src/boardDnd.ts、packages/ui/src/stage.ts、crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/tasks.rs、crates/speclink-host/src/gate.rs、crates/speclink-server/src/routes.rs),足以形成假設。

關鍵既有事實:「進行中」是派生狀態(started_at 戳記或已勾 task > 0,stage.ts:45);取消勾選是純狀態翻轉、不動 touched 與 started 戳記(tasks.rs:344);touched 記錄目前只有 discard 會刪(discard.rs:83),沒有使用者可觸及的清除動詞;生命週期 gate 是 forward-only 單一裁決點但尚未接線 enforcement(gate.rs)。

相關變更:無同題變更在途;discard --force 已有「started work = started_at 或已勾 task」的守門定義先例(speclink-cli/src/main.rs:224)。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: 退回動詞的機制,以及有工作痕跡時防呆的形狀——機械清理還是硬擋?
**Position**: 退回=新引擎動詞移除 meta 的 started_* 戳記,僅在「零工作痕跡」時合法;有痕跡時硬擋,不提供機械清理。
- 機制:新引擎動詞(InProgressRemove)與 add 同居 inprogress.rs,外科手術式移除 started_at/started_by/started_with 三行(沿 read→改→write、不重序列化慣例)。
- 「進行中」是派生的(stage.ts:45):已勾 task > 0 時就算移除戳記,派生仍是進行中——「勾了 task 不能退」是語意必要條件,不只是防呆。
- 防呆守在引擎 command 層,本地與 remote 自然共用;守門條件沿 discard --force 的「started work」定義:已勾 task > 0 或 TouchedRecord 非空 → 拒絕。
- 使用者裁定(本輪關鍵修正):有 touched/已勾 task 時**不提供 --force 機械清理**——touched 指向的檔案可能混有其他 change 的內容,不靠判斷(LLM 或人工)機械地清記錄+還原檔案有誤刪風險;desktop 不該給這種假乾淨的出路。
- 有利性質:desktop 勾錯又取消的常見誤觸不會留下 touched(取消勾選不記 touched,command.rs:75),所以誤觸場景幾乎都落在「零痕跡、可退回」;touched 守門只會擋真的做過工的變更。
**Ruled out**: 另存 "reverted" 標記(與派生規則打架,多一個狀態來源);防呆只守 UI 層(CLI/agent 可繞過,remote 要另寫一套);--force 一併清 touched(機械清理無法分辨檔案中非本 change 的內容——使用者裁定風險不可接受)。
**Open**: 觸發介面用明確動作還是開放跨欄拖曳(boardDnd.ts:35 的 spec pin 要不要改);被擋下時的提示文案要給什麼出路;remote 模式是否需要額外守門(他人開工的變更能否退回、併發寫入)。

### Round 2 — assumptions (2026-07-31)

**Focus**: 觸發介面——desktop 開拖曳或按鈕,還是狀態變更一律走 LLM+skill+CLI?
**Position**: 看板拖曳維持只管排序,狀態變更(含退回)一律 CLI 動詞、由 LLM+skill 驅動,desktop 本期不加任何狀態變更介面。
- CLI 的 in-progress 子命令群已存在且只有 add(main.rs:644-650),補對稱的 remove 是最自然形狀。
- 開工本來就是 LLM+apply skill 呼叫 CLI 蓋戳,退回走同管道是鏡像;守門擋下時需要判斷,而判斷引擎(LLM)就在 CLI 側——本地模式一切皆檔案,帶判斷修 touched json 的門天然開著。
- 事實修正:desktop 勾選框與 CLI 共用 complete()(tasks.rs:265-268),首勾即蓋 started_at,且工作樹髒時會把髒檔記進 touched(tasks.rs:302)——desktop 誤觸不保證零痕跡。
- 已裁定的不對稱:desktop 能推進(勾選框蓋戳)卻不能退回,誤觸後需請 agent 處理——可接受;引擎動詞先行,日後補桌面按鈕成本低。
- 「已就緒」欄全由派生(task 全勾)決定,取消勾選自動退欄;唯一有黏性的轉換是 started_at,一個 remove 動詞涵蓋全部。
**Ruled out**: 開放「進行中→提案中」跨欄拖曳(要改 boardDnd.ts:35 的 spec pin、定義其餘跨欄組合、設計守門擋下的回彈互動——低頻修正動作不值得);desktop 退回按鈕(本期不做,非永久排除——引擎動詞使日後補上成本低)。
**Open**: 無——轉入結論。

### Round 3 — assumptions (2026-07-31)

**Focus**: 翻案——desktop 到底要不要退回按鈕?(第 2 輪裁定「本期不做」,本輪使用者推翻)
**Position**: 要做:「退回提案中」按鈕進 desktop,卡片與抽屜都放,樣式沿討論卡既有的「封存」按鈕;守門仍只在引擎裁決一次。
- 按鈕只出現在「進行中」的變更卡與其抽屜;點擊直接呼叫引擎動詞(InProgressRemove),不在 UI 層預判守門條件——list payload 沒有 touched 狀態,預判會做出第二個裁決點,違反單一裁決原則。
- 被擋下時 desktop 以對話框呈現引擎回傳的證據(N 個已勾 task、touched 的 M 個檔案清單)與出路說明(取消勾選可自行處理;touched 需請 agent 判斷)。
- 第 2 輪「desktop 彈它自己無法處理的錯誤不如不做」的顧慮,由「錯誤對話框誠實引導去找 agent」吸收——使用者裁定按鈕的即時性價值高於此顧慮;引擎動詞先行的設計本來就讓補按鈕成本低。
- 順帶範圍(使用者指出的既有不對稱):討論卡有「封存」按鈕但討論抽屜沒有——抽屜補上同動作,卡片與抽屜的動作面板對稱。
- 按鈕文案循詞彙原則「動詞直說結果」:「退回提案中」與看板欄名直接呼應;納入 LANGUAGE.md 詞條由 propose 階段定。
**Ruled out**: desktop 不做按鈕、狀態變更一律走 LLM+CLI(第 2 輪結論——使用者翻案:誤觸修正要有免開口的即時出路);UI 層預判守門條件後停用按鈕(需要 touched 狀態進 list payload,擴 payload 又生第二裁決點,不值得)。
**Open**: 無——更新結論。

## Conclusion

**Decision**: 新增引擎動詞 InProgressRemove(CLI:`speclink in-progress remove <name>`),外科手術式移除 meta 的 started_at/started_by/started_with 三行,僅在零工作痕跡(已勾 task == 0 且 TouchedRecord 為空)時成功;有痕跡時硬擋並列出證據(N 個已勾 task、touched 記錄 M 個檔案),不提供 --force 或機械清理。desktop 加「退回提案中」按鈕:只出現在「進行中」的變更卡與其抽屜(樣式沿討論卡的「封存」按鈕),點擊直接呼叫引擎動詞、不在 UI 預判守門(單一裁決點);被擋下時以對話框呈現引擎回傳的證據與出路(取消勾選可自行處理;touched 需請 agent 判斷)。看板拖曳維持只管排序(spec pin「跨欄拖曳不改變變更階段」不動)。Remote 端加對稱路由(與 POST /changes/{name}/in-progress 成鏡像),守門在引擎 command 層共用,本地與遠端行為一致。退回定位為 gate 外的修正動詞(與 discard 同類),不動六站轉換表。skills(apply 等)補「開錯工怎麼退」的指引。順帶範圍:討論抽屜補「封存」按鈕,與討論卡對稱(既有不對稱修復)。
**Rationale**: 「進行中」是派生狀態(started_at 或已勾 task,stage.ts:45),唯一有黏性的是 started_at 戳記,一個 remove 動詞即涵蓋;touched 只知檔案不知行,檔案可能混有其他 change 的內容,機械清理有誤刪風險——守門擋「無判斷的機械路徑」,把需要判斷的清理留給 agent 或人;守門在引擎層裁決一次,CLI、desktop 按鈕、remote 路由三個入口共用同一裁決,行為一字不差。desktop 按鈕(第 3 輪翻案納入)給誤觸修正一條免開口的即時出路,被擋下時對話框誠實引導去找 agent。
**Rejected alternatives**: 開放「進行中→提案中」跨欄拖曳(要改 spec pin、定義其餘跨欄組合語意、設計守門擋下的回彈互動,低頻修正不值得);desktop 完全不做按鈕、狀態變更一律走 LLM+CLI(第 2 輪一度採納,第 3 輪使用者翻案——誤觸修正要有免開口的即時出路);UI 層預判守門後停用按鈕(touched 狀態要進 list payload,擴 payload 又生第二裁決點);--force 機械清理 touched(無法分辨檔案中非本 change 的內容,風險不可接受);另存 reverted 標記(與派生規則打架,多一個狀態來源);防呆只守 UI 層(CLI/agent 可繞過,remote 要另寫一套)。
**Deferred**: remote 模式 touched/evidence 被污染時的修復路徑(remote 不能手改檔案,本期只保證守門一致);守門對話框與錯誤訊息的精確文案、remote 路由的 HTTP 形狀、「退回提案中」是否立 LANGUAGE.md 詞條(propose 階段定)。
**Capture to**: proposal(新變更)
**Next**: /speclink-propose --from-discussion revert-in-progress-to-proposed
