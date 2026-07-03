---
topic: discuss lifecycle: document created on start, manual cleanup when abandoned
slug: discuss-lifecycle-document-created-on-start-manual-cleanup-when-abandoned
status: concluded
created: 2026-07-03
---

# Discussion: discuss lifecycle: document created on start, manual cleanup when abandoned

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

**Prompt**: 使用者指出 `/speclink-discuss` 的體驗問題 — skill 開場即呼叫 `speclink discuss new`，文件立即落地 `openspec/discussions/<slug>.md`；若後來發現根本不需要這個 discussion，唯一的清理方式是手動刪檔（或把空殼記錄塞進 archive）。

**Mode**: assumptions — 找到 3+ 相關原始碼：`crates/speclink-core/src/discuss.rs`（完整 lifecycle 實作）、`crates/speclink-cli/src/commands.rs:1597`（DiscussCommands 子命令表）、`crates/speclink-core/assets/skills/discuss.md` 與 `.claude/skills/speclink-discuss/SKILL.md`（規定開場即建檔的流程）。

**Related**: 無 open changes；一筆已 concluded 的 discussion（sdd-engine-as-sdk-...）。現有 discuss 子命令：new / list / show / context / add-round / conclude / archive / promote — 沒有任何「放棄」動詞。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-03)

**Focus**: 問題根源在哪一層（skill 流程 vs CLI），缺的是什麼機制？
**Position**: 初步判斷 — eager creation 是 skill 開場流程規定的（SKILL.md「At the start... create a new record」），但真正的缺口是 CLI 沒有「放棄」動詞：lifecycle 只有 new/context/add-round/conclude/archive/promote，abandoned 的 open discussion 只能手動 rm。傾向以 `discuss discard` 命令為主要解法、skill 建檔時機延後為輔；lazy creation（CLI 層延遲寫檔）初步認為成本高於收益，因 slug 由 topic 決定性推導，new 不寫檔近乎 no-op，而存在檢查、跨 session resume、list 可見性都要重做。
**Open**: 使用者的 abandon 情境多發生在哪個階段（開場即棄 vs 聊了幾輪才棄）？discard 對 rounds>0 的記錄是否該加 --force 保護？archive 能否直接充當放棄的歸宿？

### Round 2 — assumptions (2026-07-03)

**Focus**: 純探索型討論（「只是想討論實作的可能性」）是否也該建檔？
**Position**: 使用者確認 round 1 全部假設成立（discard 為主、skill 延後建檔為輔、rounds>0 需 --force、archive 不當垃圾桶、不做 CLI 層 lazy creation）。新情境分析：可行性探討若產出「裁決」（可行 / 不可行 / 成本太高先不做），記錄正是最有價值的部分 — 防止日後重新翻案；若探討完只是「理解了問題，沒有任何裁決」，它本質是 ask-shaped 而非 discuss-shaped，打從一開始就不該建檔，該由 skill 的分流指引處理。建檔時機可從假設 4 的「寫 context 時」再延後到「第一個實質 round」。
**Ruled out**: write-on-conclude（到結論才寫檔）作為預設 — 它讓誤觸零成本，但犧牲跨 session 的 durable thread，而那正是 speclink discuss 相對 spectra ephemeral discuss 存在的核心理由。
**Open**: 探索型討論的典型結局是「裁決」還是「純理解」？（決定需不需要一條明確的不建檔路徑）discard 是真刪還是移到 trash？

### Round 3 — assumptions (2026-07-03)

**Focus**: 探索型討論的典型結局是裁決還是純理解？（最後一個 open question）
**Position**: 使用者回答「兩種情況都有可能」。這確立了設計約束：機制不得要求使用者在開場預知結局，兩個方向的收尾成本都必須極低。裁決型 → 既有 conclude + archive/promote 出口已足夠；純理解型 → 靠 discard 一個命令收拾；誤觸/速答型 → 靠延後建檔（第一個實質 round 才落地）根本不產生檔案。三者組合完整覆蓋，無需 ephemeral 模式。
**Ruled out**: 開場 `--ephemeral` 旗標 — 要求預知結局，與「聊到一半才知道是哪種」的實際情況矛盾。discard 做軟刪除（trash）— discussions 受 git 管理，已 commit 的可復原，未 commit 的本來就是被判定無價值的內容，YAGNI。
**Open**: 無 — 全部收斂。

## Conclusion

**Decision**: 以三件事的組合解決「開場即建檔、放棄須手動刪檔」的問題，不引入新模式：
1. **skill 延後建檔** — 建檔時機從「開場」延後到「第一個實質 round」：開場只宣告預計 slug，scout、選 mode、列假設都先在對話中進行；第一次 `add-round` 前才執行 `discuss new` 並補寫 context。誤觸與一句話答完的情況永不落地。
2. **新增 `speclink discuss discard <slug>`** — 刪除 live discussion 的第一類 lifecycle 出口。`rounds > 0` 時要求 `--force`，把有實質內容的討論推向 conclude + archive 而非直接蒸發。硬刪除（openspec/ 受 git 管理，已 commit 內容可復原）。
3. **skill 開場加分流指引** — ask-shaped 話題（想理解、非裁決）不進記錄流程，直接對話回答。

**Rationale**: 使用者往往聊到一半才知道討論是「裁決型」（結論值得留存防翻案）還是「純理解型」（無須留痕），因此機制必須雙向便宜、不可要求預知結局。延後建檔消除最尷尬的誤觸情境；discard 讓中途發現不需要時只花一個命令；裁決型維持既有 conclude/archive/promote 出口。跨 session durability 是 speclink discuss 相對 Spectra ephemeral discuss 的核心價值，不可為了乾淨而犧牲。

**Rejected alternatives**:
- CLI 層 lazy creation（`new` 不寫檔）— slug 由 topic 決定性推導使不寫檔的 new 近乎 no-op；存在檢查、跨 session resume、list 可見性全須重做，成本高於收益。
- write-on-conclude（結論時才寫檔）— 誤觸零成本，但 session 中斷即全丟，犧牲 durable thread 這個核心價值。
- 開場 `--ephemeral` 旗標 — 要求使用者預知結局，與實際情況矛盾。
- archive 作為放棄歸宿 — 空殼記錄污染 archive「有結論、值得重讀」的語意。
- discard 軟刪除（trash 目錄）— git 已提供復原能力，YAGNI。

**Deferred**: none

**Capture to**: 已依使用者指示直接實作（未走 SDD 流程）：`crates/speclink-core/src/discuss.rs`（discard_discussion）、`crates/speclink-cli/src/{main,commands}.rs`（discuss discard 子命令）、`crates/speclink-core/assets/skills/discuss.md`（延後建檔 + ask-shaped 分流 + discard 指引）、README.md、docs/speclink-design.md。

**Next**: 已實作完成並重新生成 skill 檔（`speclink update`），無後續 change。
