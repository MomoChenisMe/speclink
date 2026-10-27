---
topic: speclink-quality 編排技能收進引擎正典：兩站都跑、兩章皆過才蓋章
slug: quality-skill-canonicalization
status: promoted
promoted_to: quality-skill-canonicalization
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: speclink-quality 編排技能收進引擎正典：兩站都跑、兩章皆過才蓋章

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：本機編排技能 .claude/skills/speclink-quality（討論 cross-station-staleness 2026-08-04 定案產物）dogfood 後，使用者裁定「兩站都跑」為預設正路，據此啟動該討論 Deferred 的「重議產品化」條款——要求把 quality 收進引擎正典：走 quality 則 review 與 verify 都做、兩邊皆過才正式蓋章；單站 review／verify 保留使用者自行觸發，蓋章後被他站修正打黃屬正常警示範圍。規則補進 init.rs 引擎範本（CLAUDE.md／AGENTS.md 生成源）與 README 說明文件。

模式：assumptions——相關素材充足（skills.rs 正典技能清單、本機 SKILL.md、cross-station-staleness 全紀錄、verify-station-parity 規格與任務、init.rs 指引範本）。

相關 changes／specs：verify-station-parity（進行中 0/19，verify 站工單與章的落地前置）、config-station-canon-guard（「正典不得重述」紅線）、已封存 converge-review-remediation-rounds（review 收尾迴圈與「先不蓋章」出口）。前情：cross-station-staleness Round 2 曾否決編排技能正典化，本討論即其 Deferred 條款的正式重啟。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: 重啟條件是否成立，以及「進入引擎」的具體形狀（技能正典化 vs 引擎狀態層支援）
**Position**: 五項假設經使用者確認（其中文件落點經兩點釐清），quality 以正典技能形式進引擎：
- 重啟條件成立——使用者裁定「兩站都跑」為預設正路、單站自行觸發為例外（黃章屬正常警示），正中 cross-station-staleness Deferred 的重議條款
- 形狀＝skills.rs 新增 B_QUALITY 正典技能（同既有 13 技能形制，crates/speclink-core/src/skills.rs:95-114）；quality 仍只管時序，工單／章／裁決全留兩站；刪除測試誠實承認不過——進正典的理由是預設流程需要官方入口＋三處同步＋文件自動生成，不是深度
- 「兩章皆過才正式蓋」升為正典保證——順手堵「零缺失自動蓋章」縫：兩站正典各補一行「quality 時序中改走先不蓋章出口」例外；正典稅（golden 再生、MARKER_VERSION）本次本就要付，邊際成本小
- 排程＝新 change、相依 verify-station-parity（0/19）落地後開工——verify 站的章未落地前「兩章接連蓋」無實體；不併入以免其 19 任務再膨脹，其任務 5.2 的 README 慣例句照原樣先落地不動
- 文件雙落點（使用者釐清）：規則進 init.rs 引擎範本（CLAUDE.md／AGENTS.md workflow 行由範本生成、speclink update 刷新，不手改現有檔）；真正要補的說明文件＝README／README.en 兩站分工表（「兩站都跑 → /speclink-quality」）
**Ruled out**: 引擎狀態層支援——quality 無自有狀態可擁有（無工單、無章、無裁決軸），為深度造動詞＋GUI 屬過度工程；手改現有 CLAUDE.md／AGENTS.md——範本生成物，下次 update 即被蓋掉；併入 verify-station-parity——關注點混雜、範圍膨脹
**Open**: 無——可收結論

## Conclusion

**Decision**: 把 quality 編排收進引擎正典：skills.rs 新增 B_QUALITY 技能（review 檢查先不蓋 → verify 檢查先不蓋 → 兩站 findings 統一修正 → 各自複驗 → 兩章接連蓋 → 封存），並在兩站正典各補一行「quality 時序中零缺失不自動蓋章、改走先不蓋章出口」例外，使「兩章皆過才正式蓋」成為無星號的正典保證。單站 review／verify 語意零變更：使用者自行觸發、蓋章後被後續修正打黃屬正常警示。落地為新 change（quality-skill-canonicalization），相依 verify-station-parity 封存後開工。文件：init.rs 範本補 workflow 行規則（CLAUDE.md／AGENTS.md 經 speclink update 生成、不手改現有檔）、README／README.en 兩站分工表補「兩站都跑 → /speclink-quality」。
**Rationale**: 「兩站都跑」已成使用者的預設流程，cross-station-staleness Deferred 的重議條款正式成立；正典化買到的是三處同步、文件自動生成、與修改兩站正典（堵零缺失縫）的正當性——而非介面深度（刪除測試仍不過，誠實記錄）。正典稅本次本就要付，堵縫的邊際成本小。
**Rejected alternatives**: 引擎狀態層支援（quality 無自有工單／章／裁決軸，造動詞＋GUI 成本不成比例）；維持本機技能（預設流程缺官方入口與同步保障，零缺失縫從外層攔不住）；手改現有 CLAUDE.md／AGENTS.md（範本生成物，update 即蓋掉）；併入 verify-station-parity（範圍膨脹、關注點混雜）。
**Deferred**: 看板／GUI 顯示「品質關卡進行中」狀態——需 meta 欄位與 GUI 支援，實務有感再另開討論；本機 .claude/skills/speclink-quality/SKILL.md 由引擎生成物取代的細節（隨 change 的 speclink update 落地處理）。
**Capture to**: proposal（轉出新 change quality-skill-canonicalization）
**Next**: /speclink-propose --from-discussion quality-skill-canonicalization（或 speclink discuss promote quality-skill-canonicalization）；開工排在 verify-station-parity 封存之後
