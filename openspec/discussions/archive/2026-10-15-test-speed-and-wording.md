---
topic: 我想優化測試，然後重新調整用詞：1. 目前的用詞「全量測試」英文是什麼？Full Test 嗎？因為台灣不太會用全量這個詞 2. 目前跑全量測試時間蠻久的，怎麼優化比較好？
slug: test-speed-and-wording
status: promoted
created: 2026-10-15
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: dev-profile-no-debuginfo, taiwan-full-wording
---

# Discussion: 我想優化測試，然後重新調整用詞：1. 目前的用詞「全量測試」英文是什麼？Full Test 嗎？因為台灣不太會用全量這個詞 2. 目前跑全量測試時間蠻久的，怎麼優化比較好？

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出兩件事：「全量測試」不是台灣慣用詞，想知道英文與合適的中文；本機跑完整測試很久，想知道怎麼優化。第 1 題目標明確（找台灣慣用詞），不需要追問；第 2 題原本沒有門檻，但盤點時已實測找到主因（本機 target/debug/deps 累積約 208 萬個 .o 檔、rustc 每次啟動都整個讀一遍），因此把門檻寫成假設（本機完整測試回到與 CI 同級、約 10 分鐘）交使用者確認，未另開追問。相關正式規格：delivery-baseline（root 單一指令全量驗證）、config-skill（第五問測試範圍）、review-skill、verify-skill，以及「全量」另四種意思所在的 manual-task-marker、remote-resilience、remote-workspace-data、server-event-stream、phase2-acceptance、manual-skill、context-projection、server-context-api、desktop-app；在途變更 config-skill-rule-routing: config-skill（delta 內有 5 處「全量」）、review-ticket-applied-rules: review-skill（delta 內無「全量」）。
Prior discussions: slow-global-test-suite, change-scoped-test-policy, quality-gate-speed, canonical-spec-zh-wording, config-skill-rule-routing

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-15)

**Focus**: 用詞與測試加速的七條初始假設是否成立（兩項需求一次攤開）
**Position**: 使用者確認七條全部成立；用詞與加速兩項需求都已有方向，進入收斂。
- 決策樹：A 用詞（A1 英文、A2 中文詞、A3 範圍、A4 與在途變更的先後）＋B 加速（B1 慢在哪、B2 一次清空、B3 防再累積、B4 清完後是否再優化）
- A1 英文維持 full test suite：技能原稿 config.md:93「run the full test suite」、station-shared.md:20「the repo's full test command」已如此；「Full Test」單用讀成「把某物徹底測一次」，不道地
- A2 中文改「完整測試」（對照詞「只跑受影響的測試」），立 LANGUAGE.md 詞條；vocabulary-guard 掃描面（三份 i18n、README、docs、技能原稿）目前零處「全量」，立詞條不會打紅
- A3「全量」五種意思（測試範圍、全部重新載入、任務總數、手冊全部重生、投影完整文件集）一起改，只改給人讀的文件：約 16 份正式規格、4 頁手冊、config.yaml；程式碼註解與測試名稱約 25 檔漸進汰換（前例 canonical-spec-zh-wording）
- A4 用詞變更排在 config-skill-rule-routing 封存之後：其 delta 有 5 處「全量」，封存時會整塊蓋回正式規格
- B1 主因是本機 target/ 膨脹：target/debug/deps 有 2,076,545 個 .o（target/ 共 173 GB，ls 該目錄 2 分 45 秒）；sample 正在編譯的 rustc，2346 次取樣有 2339 次在 __getdirentries64，來源 rustc_session::search_paths::SearchPath::new——每個 rustc 啟動都整讀 -L dependency 目錄；平行 session 單跑一支 desktop 測試編譯逾 9 分鐘、rustc 僅 6–18% CPU
- B1 成因：macOS dev profile 預設 split-debuginfo=unpacked，把除錯資訊留在 .o、重編不刪舊檔；scratchpad 小專案實測改一行再編 .o 25→50；這也解釋了記憶中「desktop lib 測試 main 1000–1800 秒、worktree 5 秒」的懸案
- B1 對照：CI 的 Rust 測試步驟 macOS／Linux 4–5 分鐘、Windows 5.6–9.3 分鐘（CI 每次從 rust-cache 起、不累積）
- B2 一次清空用 cargo clean，須等無其他 session 使用 cargo
- B3 防再累積採 [profile.dev] debug = false：實測 0 個 .o；測試失敗訊息仍帶檔名行號（panicked at src/lib.rs:2:13），只失去 lldb 逐行除錯與 backtrace 行號；日後需要逐行除錯時改走 macOS 限定的 split-debuginfo=packed（實測 0 個 .o、保留 .dSYM）
- B4 清完先量，目標本機完整測試回到約 10 分鐘；cargo-nextest 先不做——slow-global-test-suite 否決它的理由（不減測試執行檔數）仍成立
**Ruled out**: 英文改成「Full Test」——不道地，且原稿已用 full test suite；debug = "line-tables-only"——實測仍留 25 個 .o，不解決累積；只靠定期 cargo clean——治標，三個月級又堆回兩百萬；macOS 改 split-debuginfo=off——同樣失去除錯資訊卻仍付編譯除錯資訊的成本，不如 debug = false；先改正式規格再等 config-skill-rule-routing 封存——封存會把「全量」蓋回來
**Open**: 「全量」其他四種意思各自的替換詞；兩件事拆幾個變更、先後順序

### Round 2 — assumptions (2026-10-15)

**Focus**: 「全量」其他四種意思的替換詞與兩件事的拆法
**Position**: 使用者確認結論草稿：替換原則「修飾動作用『全部』、指一整套東西用『完整』」，拆成兩個變更，加速先做、用詞等 config-skill-rule-routing 封存後再轉出。
- 例句確認：「專案未寫測試政策時跑全量」→「…跑完整測試」；「store 全量重查後清除 stale 標示」→「store 全部重新查詢後…」；「同時回傳全量計數與寫碼計數」→「同時回傳全部任務計數與寫碼計數」；「使用者明示要求時方全量重生」→「…方全部重新產生」；「未給流程參數 SHALL 為全量」→「…SHALL 為完整投影」
- 加速的成功標準：本機 npm run test:all 約 10 分鐘內跑完；之後 target/debug/deps 不再出現 .o
**Open**: 無——進入結論

## Conclusion

**Decision**: 拆兩個變更：先關掉開發設定的除錯資訊並清空本機 target/，讓本機完整測試回到約 10 分鐘；等 config-skill-rule-routing 封存後，再把「全量」換成台灣慣用詞。
- 英文維持 full test suite（技能原稿 config.md:93、station-shared.md:20 已如此），不改英文、不觸發 asset 版號連動。
- 慢的主因：macOS dev profile 預設 split-debuginfo=unpacked，把除錯資訊留在 .o、重編不刪舊檔，target/debug/deps 累積 2,076,545 個 .o（target/ 173 GB）；每個 rustc 啟動時 SearchPath::new 整讀該目錄（sample 2339／2346 落在 __getdirentries64），每編一個 crate 付一次。CI 每次從快取起、不累積，Rust 測試步驟 4–5 分鐘。
- **cut 1 `dev-profile-no-debuginfo`**：關掉開發設定的除錯資訊、清空累積、量時間
  - workspace 根 Cargo.toml 加 `[profile.dev] debug = false`（test profile 繼承 dev）；scratchpad 實測 0 個 .o
  - 一次性 `cargo clean` 主 checkout 的 target/，須先確認無其他 session 使用 cargo
  - 量測本機 `npm run test:all`，成功標準：約 10 分鐘內跑完，且之後 target/debug/deps 不再出現 .o
  - 代價寫進 design：測試失敗訊息仍帶檔名行號（實測 panicked at src/lib.rs:2:13），失去 lldb 逐行除錯與 RUST_BACKTRACE 的行號；日後需要逐行除錯，改走 macOS 限定的 `-C split-debuginfo=packed`（實測 0 個 .o、除錯資訊收進 .dSYM）
  - 附帶效果：CI 編譯與連結也變輕；release profile 不受影響
- **cut 2 `taiwan-full-wording`**：把「全量」換成台灣慣用詞（排在 config-skill-rule-routing 封存之後）
  - 替換原則：修飾動作時用「全部」，指一整套東西時用「完整」
  - 測試範圍：全量測試 → 完整測試；「專案未寫測試政策時跑全量」→「…跑完整測試」；delivery-baseline 需求標題「root 單一指令全量驗證」→「root 單一指令完整驗證」
  - 重新載入：全量重載／全量重查 → 全部重新載入／全部重新查詢；例「store 全量重查後清除 stale 標示」→「store 全部重新查詢後清除 stale 標示」
  - 任務計數：全量計數 → 全部任務計數；例「同時回傳全量計數與寫碼計數」→「同時回傳全部任務計數與寫碼計數」
  - 手冊重生：全量重生 → 全部重新產生；例「使用者明示要求時方全量重生」→「…方全部重新產生」
  - 一整套東西：全量投影 → 完整投影、還原全量 → 還原全部；例「未給流程參數 SHALL 為全量」→「…SHALL 為完整投影」
  - 範圍：約 16 份正式規格、4 頁手冊、openspec/config.yaml；直接改字、不走 delta（前例 canonical-spec-zh-wording），Requirement／Scenario 標題同批改名
  - openspec/LANGUAGE.md 新增「完整測試」詞條（對照詞「只跑受影響的測試」），avoid 收「全量」；vocabulary-guard 掃描面目前零處「全量」，立詞條不打紅
  - 程式碼註解與測試名稱（約 25 檔）不在範圍
**Rationale**: 本機慢與 CI 快的落差來自本機獨有的 target/ 累積，不是測試本身；關掉除錯資訊同時根除累積與省下編譯連結成本，代價只有本專案目前用不到的逐行除錯，且一行設定可換回 packed。用詞變更必須等 config-skill-rule-routing 封存，因為它的 delta 有 5 處「全量」，先改正式規格會被封存整塊蓋回。
**Rejected alternatives**:
- 英文改成「Full Test」 — 單用讀成「把某物徹底測一次」，不道地，原稿已用 full test suite
- debug = "line-tables-only" — 實測仍留 25 個 .o，不解決累積
- macOS 改 split-debuginfo=packed 當首選 — 保留除錯資訊但每支執行檔多跑一次 dsymutil，本專案目前不用逐行除錯；保留為日後的退路
- macOS 改 split-debuginfo=off — 一樣失去除錯資訊，卻仍付編譯除錯資訊的成本
- 只靠定期 cargo clean — 治標，幾個月又堆回兩百萬個
- cargo-nextest 當主解 — slow-global-test-suite 否決的理由（不減測試執行檔數）仍成立
- 只改「測試」這個意思的「全量」 — 其他四種意思照留，問題只解一半
- 先改正式規格再等 config-skill-rule-routing 封存 — 封存會把「全量」蓋回來
**Deferred**:
- 程式碼註解與測試名稱裡的「全量」 — 不是給使用者讀的文件，順手漸進汰換
- phase3_chain 等本身就慢的測試、cargo-nextest — 清完若本機仍超過約 10 分鐘再談
- packed 模式的 dsymutil 成本實測 — 只有改走 packed 時才需要
**Capture to**: proposal（cut 1、cut 2）、design（cut 1 的除錯取捨）、LANGUAGE.md（詞彙漂移：「全量」→「完整測試」／「全部」／「完整」）
**Next**: /speclink-propose --from-discussion test-speed-and-wording
