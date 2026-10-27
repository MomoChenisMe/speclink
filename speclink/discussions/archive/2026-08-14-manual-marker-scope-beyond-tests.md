---
topic: 目前 [M] 定義是手動測試，希望放寬為任何需要人手動操作的任務，desktop UI 徽章也改成「手動」兩字避免誤會
slug: manual-marker-scope-beyond-tests
status: promoted
promoted_to: manual-marker-scope-beyond-tests
created: 2026-08-14
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 目前 [M] 定義是手動測試，希望放寬為任何需要人手動操作的任務，desktop UI 徽章也改成「手動」兩字避免誤會

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者發現 `[M]` 標記的現行定義（手動測試）太窄：任何 agent 無法代行、需要人動手的操作（例如去外部服務建帳號、放金鑰）都該標 `[M]`，且 desktop 任務列徽章「手動測試」四字會誤導使用者以為只限測試。

模式：assumptions——codebase scout 找到引擎解析（crates/speclink-core/src/tasks.rs、packages/ui/src/tasks.ts）、UI 文案（packages/ui/src/i18n.tsx）、六份 skill assets 與 spec（openspec/specs/manual-task-marker/spec.md、desktop-app/spec.md）等十餘個相關檔案，足以先列假設讓使用者修正。

相關 specs：manual-task-marker（全篇）、desktop-app（任務列的手動測試徽章、看板卡片的待手測標示）、client-protocol（manual 欄位）。相關詞條：LANGUAGE.md「手動測試」「待手測」（2026-08-11 討論 task-marker-ui-and-parallel-removal 定案）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-14)

**Focus**: `[M]` 語意放寬（手動測試 → 任何需人動手的操作）的波及面盤點——哪些層要動、哪些不動
**Position**: 這是純語意＋文案的變更，引擎程式碼零改動；五條假設經使用者確認大多成立：
- 引擎解析與守門不涉語意：解析器只剝前綴（packages/ui/src/tasks.ts stripMarkers、crates/speclink-core/src/tasks.rs），「寫碼任務全完成」預測子只看 manual 旗標（review.rs、verify.rs），位置 lint 只看字面位置（validate.rs）——沒有一處判斷「是不是測試」
- 中文正典詞「手動測試」改為「手動任務」，UI 任務列徽章改顯示「手動」兩字（packages/ui/src/i18n.tsx:87）
- 「待手測」章必須同步換詞以保同詞根（LANGUAGE.md 的 why 明寫兩處標示要讀成同一件事）；候選詞「待動手」
- 英文側同步："Manual test" → "Manual"、"Awaiting manual test" → "Awaiting manual"（i18n.tsx:251,252,285）；`[M]` 字母不動（本來就是 Manual 的 M）
- 波及面最大處是六份 skill assets（apply/ingest/propose/review/verify/quality）的 "manual-verification" 敘述——propose.md:266 的「Code tasks and automated tests never carry it」要改寫成「agent 做得到的都不帶」；改 asset 內文觸發 MARKER_VERSION／golden／assets.lock 三連動與 32 份 SKILL.md 再生
**Ruled out**: 加第二種標記區分「前置手動」與「驗收手動」——manual 欄位要從布林變值域，波及 client-protocol 與 desktop，語意放寬用不到；改 `[M]` 字母——既有 tasks.md 與封存區全部作廢，代價不成比例
**Open**: 「待手測」的替代詞（暫提「待動手」）定案；前置型手動任務（如去外部服務建帳號）要不要補起草或 apply 指引

### Round 2 — assumptions (2026-08-14)

**Focus**: 前置型手動任務的指引落點，與「待手測」替代詞定案
**Position**: 排序指引不加、apply 補「被擋即停」一句；章的替代詞經使用者裁定為「待手動」：
- 起草通則已有依賴排序（tasks.instruction.md:16「Order tasks by dependency」），`[M]` 專屬排序規則等於把通則抄一次
- apply.md:179「Skip it and move on」在新語意下有洞：前置型 `[M]`（如去外部服務建帳號）未做時，下游寫碼任務動不了，照字面跳過會在下游撞牆——補一句「寫碼任務依賴未勾的 `[M]` 時，停下來請使用者先做，不要繞過去」；該檔本案已在改動範圍，版號連動已觸發，邊際成本零
- 看板章由「待手測」改為「待手動」——使用者裁定：「待手動」才有「輪到你動手」的語感，且與徽章「手動」同詞根
**Ruled out**: 起草指引加 `[M]` 排序規則——通則已涵蓋，重複；「待動手」——使用者裁定語感不對
**Open**: none

## Conclusion

**Decision**: `[M]` 語意放寬為「任何 agent 無法代行、需要使用者親手操作的任務」，不限於測試。全案是純語意＋文案變更，引擎程式碼零改動。改動面：
- 中文正典詞：「手動測試」詞條改為「手動任務」；「待手測」詞條改為「待手動」（openspec/LANGUAGE.md）
- UI 文案：任務列徽章顯示「手動」兩字（packages/ui/src/i18n.tsx:87）；看板章「待手測」→「待手動」（i18n.tsx:53,54）；英文側 "Manual test" → "Manual"、"Awaiting manual test" → "Awaiting manual"（i18n.tsx:251,252,285）
- Spec 散文：manual-task-marker（Purpose 與各 requirement 的「手動測試」措辭）、desktop-app（「任務列的手動測試徽章」「看板卡片的待手測標示」兩條 requirement）
- 六份 skill assets：apply/ingest/propose/review/verify/quality 的 "manual-verification" 敘述放寬；propose.md:266「Code tasks and automated tests never carry it」改寫為「agent 做得到的都不帶」；apply.md:179 補一句「寫碼任務依賴未勾的 `[M]` 時，停下來請使用者先做」。asset 內文改動觸發 MARKER_VERSION／golden／assets.lock 三連動與 32 份 SKILL.md 再生
- `[M]` 字母不動（Manual 的 M，語意放寬後更貼）

**Rationale**: 引擎的解析、預測子、位置 lint 全不涉「是不是測試」的語意，放寬只發生在人讀的文字層；「待手動」與徽章「手動」同詞根，維持 LANGUAGE.md「兩處標示讀成同一件事」的原則。

**Rejected alternatives**: 加第二種標記區分前置／驗收手動（manual 欄位從布林變值域，波及 client-protocol 與 desktop，用不到）；改 `[M]` 字母（既有 tasks.md 與封存區全作廢）；起草指引加 `[M]` 排序規則（依賴排序通則已涵蓋）；章用「待動手」（使用者裁定語感不對，「待手動」才是「輪到你動手」）

**Deferred**: none

**Capture to**: proposal（新變更）＋ specs（manual-task-marker、desktop-app）＋ openspec/LANGUAGE.md（詞條改名，屬 vocabulary drift 的正式修訂）

**Next**: /speclink-propose --from-discussion manual-marker-scope-beyond-tests
