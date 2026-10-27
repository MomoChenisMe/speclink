---
topic: 手冊「可能過期」再現：desktop-app 單一 capability 餵多頁，判定粒度與只換時戳規則
slug: manual-stale-capability-granularity
status: promoted
created: 2026-09-11
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: manual-stale-requirement-anchor
---

# Discussion: 手冊「可能過期」再現：desktop-app 單一 capability 餵多頁，判定粒度與只換時戳規則

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報：manual-stale-time-granularity 已於 2026-09-07 封存後，桌面 app 手冊側欄仍有三頁（desktop-overview、desktop-browse、desktop-quality）標「可能過期」，問這次是什麼問題。不需 grill：目標可驗證（三頁為何亮燈、怎麼消）。
探勘結果：三頁 sources 皆只有 desktop-app、generated 皆為純日期 2026-09-05；desktop-app 規格（3324 行、71 個 @trace、73 條 Requirement）在 09-09（release-changelog-whats-new）與 09-10（discussion-hold-until-release）兩次封存後帶有新時戳；09-11 那輪手冊只重生了受影響的 desktop-board／desktop-update／tray，三頁內文與這兩次改動無關、依技能「逐位元相同即未動」規則未寫入，generated 停在 09-05。桌面端判定（apps/desktop/core/src/manual.rs）依契約以 capability 為單位、任一 @trace updated 晚於頁即標，判定本身正確。
相關規格：manual-pages（過期判定基準、frontmatter sources 定義）、desktop-manual-page（側欄標記）；相關已封存變更：manual-stale-time-granularity、manual-generation-skill。
Prior discussions: manual-stale-time-granularity, manual-generation-skill

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-11)

**Focus**: 三頁再度標「可能過期」的原因是什麼——時間判定沒修好，還是別的？
**Position**: 判定沒錯，錯在粒度：capability 級的過期判定配上「內文不變就不寫」的技能規則，讓餵自超大規格的頁永遠亮燈。
- 三頁 sources 只有 desktop-app，generated 停在純日期 2026-09-05（openspec/manual/desktop-{overview,browse,quality}.md）
- desktop-app 規格在 09-09 與 09-10 兩次封存後帶 RFC 3339 時戳，晚於 09-05，依 manual-pages 契約「任一 @trace updated 晚於頁即過期」標記正確（apps/desktop/core/src/manual.rs:159）
- 那兩次封存改的是看板討論欄 hold 標示與更新日誌彈窗，三頁內文 grep 不到相關詞；09-11 手冊 commit 816a2903 只重生受影響的 desktop-board／desktop-update／tray
- 技能 SKILL.md 第 74 行「重生後逐位元相同即未動、不寫」，所以三頁 generated 沒推進；但 09-07 那輪 commit 328f5a59 曾對 apply／data-layout／discuss 三頁只換時戳（各差兩行），兩輪行為不一致，技能文字沒寫這條
- desktop-app 一份規格餵 5 頁手冊（about 除外 4 頁），任何動到桌面 app 的封存都讓所有頁亮燈
- 決策樹：A 技能改規則「逐位元相同→只推進 generated」；B 判定粒度降到 Requirement（sources 記到段落）；C 拆 desktop-app 規格
**Ruled out**: 時間判定本身有 bug（manual.rs 分段判定與契約一致、測試 stale_compares_at_second_precision 覆蓋）；C 拆規格（大工程、與手冊無關的封存流程一併受影響，不為手冊亮燈做）
**Open**: A 與 B 是否疊加、疊加時的先後；B 的錨定鍵與錨定失效時的判定；A 是否需改 manual-pages 契約「未被重生的頁逐位元不變」的措辭

### Round 2 — assumptions (2026-09-11)

**Focus**: A 與 B 能否疊加，以及 B 的改動面是否都落在手冊範圍內？
**Position**: 能疊加且互補，B 的改動面只有三處，全部是手冊功能自己的檔案。
- B（讀取端粒度）消掉「改的段落與本頁無關」的假警報；A（生成端只換時戳）補「段落改了但手冊字面不變」的殘餘，兩層互不干擾
- @trace 本來就一條 Requirement 一個（trace.rs:23、requirement_sources），Requirement 名是引擎合併 delta 的既有鍵，可直接當錨定鍵
- B 動：manual-pages 契約（sources 項格式、錨定判定）、apps/desktop/core/src/manual.rs（第 280 行只收單一路徑段→拆 #；第 159 行整份掃→只取該 Requirement 段內時戳）、manual 技能 asset（生成時寫錨定）
- B 不動：archive 寫 @trace 的引擎、trace.rs、桌面前端（grep 不到任何讀 sources 的地方）、server、既有 41 頁（無井號＝整份規格語意，不回填）
- 錨定寫法建議 `desktop-app#<Requirement 名>`；錨定找不到（改名／刪除）視為過期，寧多亮不失聯；頁尾出處行維持只列 capability 名
- 兩案都要改 manual 技能 asset，各自立 change 就各燒一次 ASSET_VERSION／golden／assets.lock／37 份 SKILL.md 再生；合為一個 change 只燒一次
**Ruled out**: 兩個 cut 分開走（前一輪的建議）——B 面積只有三檔、且與 A 共用同一筆 asset 版號稅，分開做多付一次稅沒換到隔離價值
**Open**: 使用者是否接受 A+B 合為一個 change；desktop-manual-page 規格是否需要 delta（它引用契約，可能零改動）

## Conclusion

**Decision**: A+B 合為一個變更。A：manual 技能改規則——重生後內文逐位元相同的頁只推進 `generated`（把 09-07 那輪的實際做法寫進技能文字），manual-pages 契約「未被重生的頁逐位元不變」措辭對應放寬為「內文不變、時戳得推進」。B：`sources` 項可帶 `#<Requirement 名>` 錨定（例 `desktop-app#看板與任務`），過期判定只取該 Requirement 段落內的 `@trace updated`；無井號維持整份規格語意，既有 41 頁不回填；錨定找不到（Requirement 改名或刪除）視為過期；頁尾出處行維持只列 capability 名。改動面：manual-pages 契約、apps/desktop/core/src/manual.rs（拆井號＋分段取時戳）、manual 技能 asset；desktop-manual-page 規格引用契約，propose 時確認是否零 delta。
**Rationale**: 三頁亮燈的根因是 capability 級判定配上「內文不變就不寫」——desktop-app 一份規格餵 4 頁，任何桌面封存都讓全部亮燈且永遠消不掉。B 在讀取端消掉段落無關的假警報，A 在生成端清掉段落有關但字面不變的殘餘，兩層互補。兩案都要改技能 asset，合一個 change 只燒一次 ASSET_VERSION／golden／assets.lock／SKILL.md 再生的版號稅。
**Rejected alternatives**: 只做 A（每次封存都要跑手冊清燈，治標）；只做 B（段落改措辭但手冊字面不變時燈消不掉）；拆 desktop-app 規格為多個 capability（大工程，與手冊無關的封存流程一併受影響）；A、B 分兩個 cut（B 只三檔，分開多付一次版號稅沒換到隔離價值）；錨定失效時靜默退回整份規格（失聯不可見，寧多亮一次燈）。
**Deferred**: desktop-manual-page 規格是否需要 delta（propose 時對照契約引用句決定）；既有 41 頁何時換成錨定寫法（各頁下次重生時自然換，不另立工作）。
**Capture to**: proposal（範圍）、design（A 的時戳推進規則與 B 的錨定語法／失效判定）、spec（manual-pages delta）、tasks
**Next**: /speclink-propose --from-discussion manual-stale-capability-granularity
