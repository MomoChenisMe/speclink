---
topic: config 技能規則的調整——驗證成本、rule 來源正當性、重審範圍
slug: config-skill-rule-alignment
status: concluded
created: 2026-08-07
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: config 技能規則的調整——驗證成本、rule 來源正當性、重審範圍

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：change-scoped-test-policy 討論落地 scoped 測試規則時，實跑 /speclink-config 暴露技能字面要求與合理做法的脫節（判準四要求「Run the command」但實跑用 ls/grep 驗存在），且該次新 rule 來自討論裁決、不在技能的固定輸入集內。使用者提出：config 技能本身的規則是否也要依這套精神調整。

模式：interview（決策空間先攤開、逐節點收斂）。偵察事實：speclink-config 為引擎生成的正典技能，本文在 crates/speclink-core/assets/skills/config.md，受 render golden 保護（crates/speclink-core/tests/it/render_golden.rs 的 config_skill 測試）——任何調整是引擎變更（asset＋MARKER_VERSION＋golden＋assets.lock 三連動）。平行 session 的 v1.18.1 資產改版尚未提交，本調整需排隊其後。

前情：[[change-scoped-test-policy]]（scoped 測試規則的來源討論）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — interview (2026-08-07)

**Focus**: 「依據這個調整」的範圍——三種讀法哪些成立
**Position**: 使用者三條全選（A 驗證成本、B 來源正當性、C 重審範圍），其中 C 推翻助理「不必改」的建議：
- A：判準四字面「Run the command; check the path」（config.md:53）會執行到貴指令（如 golden 測試須跑數分鐘）；實跑已用 ls/grep 驗存在——字面與實務脫節，改為「驗引用存在用便宜手段，不執行貴指令」
- B：scoped-test rule 來自討論裁決，不在技能固定輸入集（manifests／README／docs 索引／現行文件／詞彙表）內；需明文「非結構宣告可導出」不構成刪除理由
- C：使用者要求把「scope hint 收窄重審範圍」明文化，不接受「技能本意已允許」的隱含解讀
- 影響面事實：config 技能是正典生成技能（crates/speclink-core/assets/skills/config.md，golden 保護於 render_golden.rs config_skill 測試），調整＝引擎變更，排隊於平行 session 未提交的 v1.18.1 之後
**Open**: 三條調整的具體措辭方向；落地路由（promote 成新變更或 propose）

### Round 2 — interview (2026-08-07)

**Focus**: 三條調整的具體措辭方向與 B 的機制選擇
**Position**: 三條方向具體化，B 的機制經排除法只剩原則句一途：
- A：判準四改為「驗引用存在且可解析一律用靜態手段（路徑查檔案系統、測試名 grep 原始碼、npm script 查 package.json、CLI 子指令對 --help 面），SHALL NOT 執行被引用的測試或建置指令」；判準一的 speclink instructions --json 探針不在此限（技能自己的探測，便宜且必要）
- B：唯一穩健機制是原則句「一條 rule 只因不過四判準而被刪，SHALL NOT 因『無法自固定輸入集導出』而被刪」——補上「What is already written is the starting point」失落的另一半
- C：scope hint 收窄「判準一～三的全面重審」至範圍內 artifacts；判準四恆為全文件掃描（A 使其便宜後不再有成本問題）；無 hint 維持全文件。A 與 C 互相成立
- 行為例：/speclink-config rules only for specs → specs rules 走完整判準重審、全文件引用便宜存在檢查、tasks 的討論裁決 rule 不因非結構導出被判掉
**Ruled out**: 來源註記機制（config.yaml 重寫掉註解，標記活不過下次落地）；討論鏈結機制（config.yaml 無 meta 欄位）；判準四維持字面執行指令（貴測試一跑數分鐘，與 scoped 精神抵觸）
**Open**: 落地路由（promote 或 propose）

## Conclusion

**Decision**: 調整 config 技能正典（crates/speclink-core/assets/skills/config.md）三處：(A) 判準四的引用驗證改為靜態便宜手段——路徑查檔案系統、測試名 grep 原始碼、npm script 查 package.json 宣告、CLI 子指令對 --help 面，SHALL NOT 執行被引用的測試或建置指令（判準一的 speclink instructions --json 探針不在此限）；(B) 增列原則句「一條 rule 只因不過四判準而被刪，SHALL NOT 因『無法自固定輸入集導出』而被刪」，保障使用者裁決型 rule（討論結論落地者）不被 convergence 判掉；(C) 明文化 scope hint 語意——收窄判準一～三的全面重審至範圍內 artifacts，判準四恆為全文件掃描，無 hint 維持全文件。
**Rationale**: 實跑 change-scoped-test-policy 落地時暴露字面與實務脫節（判準四字面要求執行指令，實跑用 ls/grep）；貴指令的執行本屬 CI 與 apply 驗證步驟，技能內執行與 scoped 精神抵觸。A 使判準四變便宜，才讓 C 的「全文件引用掃描」在收窄重審下仍可負擔——兩條互相成立。
**Rejected alternatives**: 來源註記機制（config.yaml 重寫掉註解，標記活不過下次落地）；討論鏈結機制（config.yaml 無 meta 欄位承載）；判準四維持字面執行（golden 測試一跑數分鐘）；「C 不必改、技能本意已允許」（使用者裁定要明文化，隱含解讀不足恃）。
**Deferred**: none
**Capture to**: 新變更（引擎變更：asset＋MARKER_VERSION＋golden＋assets.lock 三連動；生成技能檔隨 update 再生；排隊於平行 session 未提交的 v1.18.1 之後）
**Next**: /speclink-propose --from-discussion config-skill-rule-alignment
