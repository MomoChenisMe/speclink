---
topic: 手冊「可能過期」同日平手：@trace updated 與 generated 改比到時間
slug: manual-stale-time-granularity
status: promoted
promoted_to: manual-stale-time-granularity
created: 2026-09-07
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 手冊「可能過期」同日平手：@trace updated 與 generated 改比到時間

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者於 2026-09-05 23:17 封存 discuss-search-recall、23:31 跑 /speclink-manual 重生手冊後，desktop 手冊頁的「認識資料」「討論」兩頁仍標「可能過期」。查明為設計限制：manual-pages 契約「過期判定基準」只比到日、同日視為過期；頁 generated 與來源規格 @trace updated 都是 2026-09-05，平手即標記。使用者問「改比到時間可以嗎」，需求已具可驗證目標（同日先封存後生成不再誤標；生成後同日封存仍要標），不需 grill 階段，直接列假設。
相關規格：manual-pages（判定基準契約）、desktop-manual-page（標示）、manual-skill（generated 寫入）、verify-evidence（@trace updated 定義）、archive-skill（只說兩欄）。相關程式碼：crates/speclink-core/src/archive.rs trace_block／util::today、apps/desktop/core/src/manual.rs 日期 regex、packages/ui/src/i18n.tsx manual.stale。
Prior discussions: manual-generation-skill（已封存；定案過期報告為「updated 晚於 generated」，後於 manual-skill 變更收緊為同日也算）

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-07)

**Focus**: 同日平手誤標能否改比到時間；時間要從哪來、波及哪些讀寫端
**Position**: 可以，但時間必須兩邊都有——@trace updated 與 generated 都改寫成 RFC 3339 帶偏移量，判定改三段式；使用者確認六項假設全對。
- 寫入端：archive.rs trace_block 改用帶時間時戳，只影響未來封存；現有 568 行純日期不回改
- 讀取端只有一處：apps/desktop/core/src/manual.rs 的 updated regex；trace.ts 與 trace.rs 只讀 source，desktop-app 規格明寫規格卡不顯示 updated
- 判定三段式：兩邊都有時間→規格時間晚於頁面時間才過期；任一邊只有日→維持同日也算；此規則寫進 manual-pages 契約，生成端與讀取端同基準
- generated 由 manual 技能寫入帶時間；SKILL.md 是 asset，要 bump ASSET_VERSION＋golden＋assets.lock
- 要改的規格四份：manual-pages、desktop-manual-page、manual-skill、verify-evidence（updated 定義從「封存日期」放寬）
- 時區：RFC 3339 帶偏移量，比較時換成同一瞬間；跨機器封存與生成是常態
**Ruled out**: git commit 時間（兩端都要跑 git、未 commit 無時間、違反同基準）；只比日但改成「晚於才算」（生成後同日封存永遠漏判，契約明文拒絕）；維持現狀（封存完立刻生手冊是常態工作流，平手每次都發生）
**Open**: 比到時間後 UI 文案「可能過期」要不要改（LANGUAGE.md 已有此詞條）；第一輪執行時純日期舊資料的假警報如何收斂

### Round 2 — interview (2026-09-07)

**Focus**: 比到時間後，UI 文案「可能過期」要不要改成「已過期」
**Position**: 保留「可能過期」，只更新 LANGUAGE.md 詞條的 definition 為三段式規則；使用者同意。
- 「可能」的來源是判定粗：比對只證明來源動過，不證明頁面內容真的變了（LANGUAGE.md 詞條 why 已寫明）；時間只消掉先後疑問，消不掉這層
- 任一邊只有日期時退回「同日也算」，標記仍是猜測；同一標記兩種誠實程度，不宜叫「已過期」
- 改名要動 i18n 兩語、兩份規格條文、技能摘要用詞，成本與收益不成比例
- 舊純日期規格的假警報：下一個日曆日重生一次即收斂，與現行方式相同，不需額外處理（事實，非決定）
**Ruled out**: 「已過期」（過度宣稱，引擎不知內容是否變）；「規格已更新」（誠實但少了「所以手冊可能不準」的推論，讀者要自己接）
**Open**: 無

## Conclusion

**Decision**: 手冊過期判定改比到時間。@trace updated（archive 寫入）與手冊頁 generated（manual 技能寫入）都改寫成 RFC 3339 帶時區偏移量（例 `2026-09-05T23:17:28+08:00`），只影響未來寫入、現有純日期不回改。判定三段式寫進 manual-pages 契約：兩邊都有時間→規格時間晚於頁面時間才過期；任一邊只有日期→維持「不早於（同日也算）」；sources 空、generated 缺席或格式不對、規格不存在→不標記。UI 文案保留「可能過期」，LANGUAGE.md 詞條只更新 definition。
**Rationale**: 「封存完立刻生手冊」是常態工作流，同日平手每次都發生，不是邊角；時間必須兩邊都有才能分先後，所以是 @trace 格式契約的放寬，不是 desktop 小修。讀取端只有 apps/desktop/core/src/manual.rs 一處 regex，寫入端只有 archive.rs trace_block，波及可控。
**Rejected alternatives**: git commit 時間（兩端都要跑 git、未 commit 無時間、違反生成端與讀取端同基準）；只比日但改「晚於才算」（生成後同日封存永遠漏判，契約明文拒絕）；維持現狀（平手是每次工作流的常態）；文案改「已過期」（引擎不知內容是否真的變）；文案改「規格已更新」（少了「手冊可能不準」的推論）。
**Example**: 頁 generated `2026-09-05T23:31:00+08:00`，來源規格最新 updated `2026-09-05T23:17:28+08:00` → 無標記；updated `2026-09-05T23:40:00+08:00` → 可能過期；updated `2026-09-05`（純日期）→ 可能過期（退回同日規則）；頁 generated `2026-09-05`（純日期）而 updated `2026-09-05T23:17:28+08:00` → 可能過期（退回同日規則）。
**Deferred**: none
**Capture to**: proposal（變更範圍）、spec（manual-pages 判定基準、desktop-manual-page 判定表、manual-skill generated 格式、verify-evidence updated 定義）、LANGUAGE.md（「可能過期」definition 更新）
**Next**: /speclink-propose --from-discussion manual-stale-time-granularity
