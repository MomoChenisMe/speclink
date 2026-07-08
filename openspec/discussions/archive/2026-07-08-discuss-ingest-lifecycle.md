---
topic: 討論結論走 ingest 的生命週期斷鏈
slug: discuss-ingest-lifecycle
status: promoted
promoted_to: discuss-link-verb
created: 2026-07-08
---

# Discussion: 討論結論走 ingest 的生命週期斷鏈

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：討論「專案選擇對齊-spectra」的結論走 /speclink-ingest 進既有變更 desktop-config-multiproject，變更完成封存後討論仍以 concluded 卡在看板。病因：自動封存（archive.rs:218-227）由變更側 from_discussion 鏈驅動，而該鏈只有 promote()（discuss.rs:350-378，建新變更時）會鑄；ingest 僅為技能、非引擎動詞，兩側皆無連結。conclude 模板的 Next 欄位明文支持 ingest 路徑，故屬設計缺口而非誤用。
模式：assumptions——引擎相關碼（discuss.rs、archive.rs、CLI commands）已定位，事故現場已驗證。
相關 changes/specs：已封存變更 2026-07-07-desktop-config-multiproject（事故現場）；discuss／ingest 技能（crates/speclink-core/assets/skills/）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: ingest 型結論的生命週期斷鏈如何補
**Position**: 新引擎動詞 `speclink discuss link <slug> <change>`——把「鑄鏈」自「建新變更」拆出：對既有變更 meta 寫入 from_discussion、討論側 mark_promoted（status→promoted、promoted_to 累加）。鏈成立後下游零改動全接上既有機制：看板「已轉出變更的討論」群組、抽屜衍生變更分頁與雙向互跳、最後一個變更封存時自動帶走討論。配套：discuss 技能 conclude 步驟遇「Capture to 既有變更」即執行 link；ingest 技能加對應提示；LANGUAGE.md「已轉出變更」定義放寬為「連結至至少一個變更」或新增「併入變更」詞條。守衛：目標變更已有其他 from_discussion 時拒絕（欄位單值）。介面深度檢查：縫落在 discuss.rs（單一 core fn 同時寫兩側）、單一 adapter（CLI→core）、深度＝frontmatter 機制＋change meta 寫入＋守衛邏輯、刪除測試＝回到本次事故——通過。使用者裁定採此案。
**Ruled out**: 純流程修正（技能指示 ingest 後手動封存）——事故正是「靠人記得」造成，且失去在途雙向連結；conclude 時解析結論 Next 文字自動連結——脆弱的自由文字解析；`promote --into` 旗標——旗標改變 promote 的 scaffold 本質，獨立動詞較誠實。
**Open**: 動詞定名（link vs attach）、是否限定 concluded 才可 link、node bridge／桌面 GUI 是否曝露——皆屬 propose 階段細節，逕列 Deferred。

## Conclusion

**Decision**: 新增引擎動詞 `speclink discuss link <slug> <change>`：對既有變更 meta 寫入 from_discussion、討論側標記 promoted（promoted_to 累加），使 ingest 型結論接上既有的看板群組、抽屜互跳與自動封存機制。配套：discuss 技能 conclude 步驟在 Capture to 指向既有變更時執行 link；ingest 技能加提示；LANGUAGE.md 詞條修訂（「已轉出變更」放寬為「連結至至少一個變更」或新增「併入變更」）。守衛：目標變更已有其他 from_discussion 時拒絕。
**Rationale**: 生命週期規則掛在 from_discussion 鏈上而只有 promote 鑄鏈——ingest 是官方認可路徑卻無鑄鏈機制。補鏈一次，下游（群組、互跳、自動封存）零改動全接上；機制歸引擎、不靠人記得。
**Rejected alternatives**: 純技能指示的手動封存（重蹈「靠人記得」覆轍、失去在途連結）；解析結論 Next 文字自動連結（脆弱）；promote --into 旗標（語意混淆）。
**Deferred**: 動詞定名（link/attach）、是否限定 concluded 才可 link、node bridge／桌面 GUI 曝露、from_discussion 多值累加——propose 階段裁定。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion discuss-ingest-lifecycle
