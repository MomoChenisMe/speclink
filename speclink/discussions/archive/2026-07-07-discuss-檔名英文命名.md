---
topic: discuss 檔名英文命名
slug: discuss-檔名英文命名
status: promoted
promoted_to: discuss-english-slug
created: 2026-07-07
---

# Discussion: discuss 檔名英文命名

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出:discuss 建立的文件檔名在中文主題下是中文檔名,希望與 changes 一致採英文 kebab-case 命名。模式:假設(相關源檔充足)。掃描發現:change 的英文名由 agent 在 propose 時自行衍生,引擎不翻譯;speclink discuss new 只收 topic、無 slug 覆寫入口;slugify(crates/speclink-core/src/util.rs)刻意保留 CJK 字元以保語意;桌面 app 不建立討論,唯一建立者是跑 /speclink-discuss 的 agent。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-07)

**Focus**: 英文 slug 應由誰產生、引擎後備行為的嚴格度
**Position**: speclink discuss new 加選配 --slug 參數,由討論技能(agent)從主題衍生英文 kebab-case 傳入;topic 留中文供顯示;--slug 值驗證純 ASCII kebab-case、非法報錯。slugify 的 CJK 後備保留(無 --slug 時維持現行為)。既有三份中文檔名討論不回改(改名會斷 from_discussion 連結)。順帶收益:discuss promote 不帶 name 時預設變更名等於 slug,英文化後此預設值變得可用。使用者確認全部假設,採「技能層落實、引擎寬容」。
**Ruled out**: 引擎端音譯(需拼音庫、拼音對臺灣使用者不可讀);slugify 改為丟棄 CJK(純中文主題會塌成無意義 slug,如「四情境預設 GUI 工具矩陣」塌成 gui);引擎硬擋非 ASCII topic(直接用 CLI 的人被擋,嚴格度過頭);回改既有討論檔名(遷移成本換零功能收益)。
**Open**: 無

## Conclusion

**Decision**: speclink discuss new 新增選配 --slug 參數(驗證純 ASCII kebab-case,非法報錯);討論技能改為一律從主題衍生英文 slug 傳入,topic 維持中文顯示;slugify 的 CJK 後備行為不變;既有中文檔名討論不回改。
**Rationale**: 與 change 命名模式對齊——英文名由雙語的 agent 衍生,引擎不承擔翻譯;技能層落實、引擎寬容,直接用 CLI 的人不受影響。
**Rejected alternatives**: 引擎音譯(拼音庫依賴且對臺灣使用者不可讀);slugify 丟棄 CJK(純中文主題塌成無意義 slug);引擎硬性拒絕非 ASCII(擋到無 agent 的 CLI 使用);回改既有討論(斷 from_discussion 連結、零收益)。
**Deferred**: 無
**Capture to**: proposal(CLI 參數與 core 簽名)、tasks(技能三處同步:core assets、repo 技能實例、render golden 於乾淨樹再生)
**Next**: /speclink-propose --from-discussion discuss-檔名英文命名
