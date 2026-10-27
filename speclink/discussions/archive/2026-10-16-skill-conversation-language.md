---
topic: issue #2：skill 的對話語言與文件語言該怎麼綁定（locale 是否擴大成對話語言、哪些 skill 的文件要跟 locale）
slug: skill-conversation-language
status: promoted
created: 2026-10-16
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: skill-conversation-language
---

# Discussion: issue #2：skill 的對話語言與文件語言該怎麼綁定（locale 是否擴大成對話語言、哪些 skill 的文件要跟 locale）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

GitHub issue #2（Wonsle 回報，2026-10-16）：專案設 `locale: tw`、Claude Code 也設繁中，但 /speclink-apply、/speclink-quality、/speclink-worktree-merge 等長流程中 Agent 常改用英文（10 天約 95 個對話出現 8 次）。issue 建議讓 `locale` 也決定 Agent 在每個 skill 的對話語言，並在每個 skill 開頭貼同一段語言規則、回報範例註明依 locale、子代理 brief 帶語言規則。需求清楚，不需要 grill 階段。
查證：19 個產品 skill 中 10 個完全沒提到 locale（apply、apply-with-worktree、quality、worktree-merge、archive、commit、audit、discuss、improve、trace）；issue 的表格漏列 analyze（它和 drift 一樣只綁報告語言）；apply、apply-with-worktree、worktree-merge 已呼叫含 `locale` 欄位的 `instructions apply --json` 卻沒使用；`skills.rs` 已有 `concat!` 共用片段機制（`station-shared.md`）。相關正式規格：skill-authoring、workflow-config、discuss-skill、improve-skill、config-skill、archive-skill、discussion-docs；目前沒有在途 change。
Prior discussions: review-ticket-locale

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-16)

**Focus**: issue 對現況的描述是否屬實，決策空間有哪些節點
**Position**: issue 的現況描述大致正確，症狀屬實；決策樹如下：
- A locale 是否擴大成對話語言（issue 的主張）
- B 不擴大的話，對話語言漂移怎麼處理
- C 哪些 skill 寫的文件要跟 locale
- 事實：10 個 skill 沒提 locale 屬實；表格漏 analyze；有綁定的 skill 多半只管文件／報告，review 是例外（回合呈現與提問也綁 locale，見 review-ticket-locale）
- 事實：apply、apply-with-worktree、worktree-merge 已拿到含 `locale` 的 payload 但沒用；`skills.rs` 的 `concat!`（`station-shared.md` 前例）可做單一來源的共用片段
**Open**: A locale 是否擴大成對話語言；B 對話語言漂移的處理方式；C 文件語言的綁定範圍

### Round 2 — interview (2026-10-16)

**Focus**: locale 是否應擴大成 Agent 的對話語言
**Position**: 不擴大；locale 維持「AI 產出文件的語言」，對話語言是個人偏好、由各工具自己的語言設定決定。
- 定義證據：init 範本 `# Language for AI-generated artifacts`（`crates/engine/speclink-core/src/workspace/init.rs`）、`docs/configuration.md` 的 "The language of AI-generated artifacts"、desktop-config 規格「引擎產出 artifacts 的語言政策」且與 UI 語言分開
- locale 寫在 `openspec/config.yaml`、進版控，是團隊設定；對話語言屬個人
- review 把 locale 用在回合呈現與提問，是因為那些是審查結果的呈現（產出的一部分），不構成「對話語言」先例
**Ruled out**: locale 同時決定對話語言——團隊設 `locale: en` 統一文件語言時，中文母語成員會被強制用英文對話，比現況更糟；在 AGENTS.md／CLAUDE.md 補語言規則——一句話敵不過 skill 內幾百行英文，且要各專案自補；Stop hook 檢查中文比例——事後補救，Codex、Copilot 各要一套
**Open**: B 對話語言漂移的處理方式；C 文件語言的綁定範圍

### Round 3 — interview (2026-10-16)

**Focus**: 不動 locale 的前提下，對話語言漂移怎麼處理
**Position**: 微調 skill 規則：新增一份共用的對話語言段落，接在每個產品 skill 內文最前面。
- 規則三點：跟使用者對話時用使用者的語言；skill 用英文寫、回報範例只示範格式，文字要翻譯，指令／路徑／程式碼／嚴重度籤等固定 token 維持原樣；某步驟規定文件或報告語言（例如經 locale）時以該步驟為準
- 草稿：`**Conversation language**: write to the user in the language they use with you — progress updates, questions, and summaries. This skill is written in English, and its example messages show structure, not language: translate their prose, and keep commands, file paths, code, and fixed tokens (severity labels, status words) as they are. When a step sets the language of an artifact or a report (for example through locale), that step decides for that output.`
- 落點：新增 `assets/skills/` 共用檔，經 `skills.rs` 的 `concat!` 接進每個產品 skill（`station-shared.md` 前例），單一來源
- 段落不用 MUST／NOT 等大寫強調字，新檔在 `EMPHASIS_CAPS` 登記上限 0，不擠其他原稿的上限
- 子代理 brief 不改：子代理不直接對使用者說話
- 連動成本：ASSET_VERSION、golden、assets.lock 三連動
**Ruled out**: 在 19 個 skill 各貼一份相同段落（issue 的建議）——多份複本會漂移，已有 concat 單一來源機制；locale 沒設定時對話強制英文——沒設 locale 的中文使用者會變差
**Open**: C 文件語言的綁定範圍

### Round 4 — interview (2026-10-16)

**Focus**: 10 個沒綁 locale 的 skill 中，哪些寫的文件要跟 locale
**Position**: discuss、improve、config、audit 四個 skill 綁 locale；apply 與 archive 改既有檔案時沿用原檔語言；其餘不改。
- discuss、improve：討論記錄進版控給團隊看，且是 proposal（已綁 locale）的來源；內文跟 locale，固定標記（`**Focus**:`、`**Ruled out**:`、`**Decision**:` 等，`discuss search` 靠它們找決定行）維持英文，slug 照舊英文，topic 照 discussion-docs 規格維持使用者原文
- config：起草的 `context`、`rules` 文字跟 locale；review 規則由使用者口述，照原話寫
- audit：獨立模式的報告跟 locale，與 analyze、drift 的 Report language 一致（使用者裁定納入）
- 讀設定：discuss、improve、config、audit 不綁 change，用 `speclink workflow-config show --json` 的 `locale`；此指令不套用 SPECLINK_LOCALE 環境變數，manual、baseline 已有同樣前例，接受
- apply（含 apply-with-worktree）、archive：只改既有檔案，「沿用原檔語言」放進共用段落一句話即可（`When you edit a document that already exists, keep the language it is written in.`），不必查 spec_locale（apply payload 沒有 specLocale）
- 不改：commit（摘要取自 proposal 的 Why，已是 locale 語言）、worktree-merge（不寫文件）、quality（工單由 review／verify 寫，已綁定）、trace（對話中回答，屬對話語言）
- 行為改變：locale 沒設定＝英文，沒設 locale 的使用者討論記錄會從跟對話語言變成英文；與 proposal 現況一致，回覆 issue 時要講明
**Ruled out**: discuss 記錄跟對話語言（現況）——進版控的團隊文件與下游 proposal 語言不一致；apply／archive 改檔時查 spec_locale——要動引擎 payload，沿用原檔語言更簡單且不會在同一檔混語言
**Open**: 規則最終措辭留 propose 定稿；是否一個 change 收完

## Conclusion

**Decision**: 不讓 locale 擴大成對話語言；改以一份共用的對話語言段落處理 Agent 中途改講英文，並把 discuss、improve、config、audit 的產出綁定 locale，一個 change 收完。
- locale 維持「AI 產出文件的語言」（團隊設定、進版控）；對話語言是個人偏好，由各工具自己的語言設定決定
- 共用段落：新增一份 `assets/skills/` 共用檔，經 `skills.rs` 的 `concat!` 接在每個產品 skill 內文最前面（`station-shared.md` 前例），單一來源
- 段落內容：跟使用者對話時用使用者的語言；skill 用英文寫、回報範例只示範格式，文字要翻譯，指令／路徑／程式碼／嚴重度籤等固定 token 維持原樣；某步驟規定文件或報告語言（例如經 locale）時以該步驟為準；修改既有文件時沿用該文件原本的語言
- 段落不用 MUST／NOT 等大寫強調字，新檔在 `EMPHASIS_CAPS` 登記上限 0；最終措辭於 propose 定稿
- discuss、improve：討論記錄內文跟 locale；`**Focus**:`、`**Ruled out**:`、`**Decision**:` 等固定標記維持英文，slug 維持英文，topic 照 discussion-docs 規格維持使用者原文
- config：起草的 `context`、`rules` 跟 locale；review 規則照使用者原話寫
- audit：獨立模式的報告跟 locale，與 analyze、drift 的 Report language 一致；嚴重度等固定 token 維持英文
- 上述四個 skill 以 `speclink workflow-config show --json` 讀 `locale`（不套用 SPECLINK_LOCALE 環境變數，比照 manual、baseline 前例）
- apply（含 apply-with-worktree）、archive 靠共用段落的「沿用原檔語言」處理既有檔案的修改，不另查 spec_locale
- 不改：commit、worktree-merge、quality、trace；子代理 brief 不改（子代理不直接對使用者說話，review／verify 報告本來就綁 locale）
- 行為改變：locale 沒設定＝英文，沒設 locale 的專案討論記錄由跟對話語言改為英文，與 proposal 現況一致
- 連動：ASSET_VERSION、golden、assets.lock 三連動；完成後回報並關閉 GitHub issue #2
**Rationale**: 關鍵取捨是「單一設定來源」對上「團隊設定與個人偏好分離」。issue 希望 locale 一個欄位管到底，但 locale 進版控、屬團隊；讓它決定對話語言，會讓文件統一用英文的團隊強迫成員用英文對話。症狀的根源是 skill 內大量英文指令與範例把 Agent 帶走，所以修 skill 寫法即可，不必改設定語意；文件側則補齊仍未綁 locale 的四個 skill，讓「locale＝產出文件語言」在所有 skill 一致成立。
**Rejected alternatives**:
- locale 同時決定對話語言 — 團隊設 `locale: en` 時中文母語成員被強制用英文對話，比現況更糟
- 在 19 個 skill 各貼一份相同段落 — 多份複本會漂移，已有 concat 單一來源機制
- locale 沒設定時對話強制英文 — 沒設 locale 的中文使用者會變差
- AGENTS.md／CLAUDE.md 補語言規則 — 一句話敵不過 skill 內幾百行英文，且要各專案自補
- Stop hook 檢查中文比例 — 事後補救，各工具要各做一套
- discuss 記錄維持跟對話語言 — 進版控的團隊文件與下游 proposal 語言不一致
- apply／archive 改檔時查 spec_locale — 要動引擎 payload，沿用原檔語言更簡單且不會在同一檔混語言
**Deferred**: none
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion skill-conversation-language
