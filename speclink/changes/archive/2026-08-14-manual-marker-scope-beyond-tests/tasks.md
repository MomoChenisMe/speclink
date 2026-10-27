> 純語意＋文案變更，引擎行為零改動。TDD：每組先動測試期望值看紅，再改實作轉綠。改資產後的 speclink 動詞一律用 ./target/debug/speclink（安裝版過期陷阱）；本 repo 的 .claude/skills 與 .agents/skills 再生檔不逐一列入 evidence，收尾以 git status 盤點。

## 1. UI 文案（packages/ui，先紅後綠）

- [x] 1.1 更新兩份 UI 測試的期望字串——taskList 測試期望徽章文字 tw「手動」／en「Manual」，awaitingManualBadge 測試期望章文字 tw「待手動」／tooltip「待手動·剩 N 項」／en「Awaiting manual」「Awaiting manual · N left」；跑 npm test -w packages/ui 確認先紅（紅在字串期望，不紅在其他處）；此字面即規格「任務列的手動任務徽章」與「看板卡片的待手動標示」釘住的文案 <!-- speclink-task:tsk_01KZZQKQ76NZ6WYYP3WG5ZFBQD -->
- [x] 1.2 改 packages/ui/src/i18n.tsx 六個詞條值：zh-TW 的 tasks.manual＝「手動」、card.awaitingManual＝「待手動」、card.awaitingManualTitle＝「待手動·剩 {n} 項」；en 的 tasks.manual＝「Manual」、card.awaitingManual＝「Awaiting manual」、card.awaitingManualTitle＝「Awaiting manual · {n} left」；鍵名一律不動；npm test -w packages/ui 全綠 <!-- speclink-task:tsk_01KZZQKQ76Z2YTN5M9J9XJ4TRG -->

## 2. 正典詞彙與守門

- [x] 2.1 openspec/LANGUAGE.md 詞條改名：「手動測試」→「手動任務」（definition 放寬為「agent 無法代行、需使用者親手操作的任務——不限於測試」並保留 [M] 前綴與徽章的呈現敘述；avoid 加「手動測試（此概念上）」且承接存量詞：人工測試、手工測試、手動驗證（標記語境中）、manual test（中文散文中）；why 補記 2026-08-14 討論 manual-marker-scope-beyond-tests 放寬裁定）、「待手測」→「待手動」（definition 同步換詞；avoid 列「待手測」不帶語境限定並承接存量詞；why 同步補記）；落實規格「手動任務與待手動以 LANGUAGE.md 詞條承載」與 design「D2：avoid 詞的語境限定」的分流；驗證：兩詞條齊備、舊詞條名消失（人工檢視 diff） <!-- speclink-task:tsk_01KZZQKQ76A2W2Q1QY6RT5ZG93 -->
- [x] 2.2 scripts/vocabulary-guard.test.mjs 的 SURFACE_FILES 加入 packages/ui/src/i18n.tsx；跑 node --test "scripts/**/*.test.mjs"——若揭露該檔存量 avoid 詞違規，逐條修正繁中字面（不動鍵名與結構、不動英文），全套守門測試綠燈收尾；落實規格「使用者可見文案面的範圍與詞彙約束」的守門面擴充（design「D4：守門面擴充」） <!-- speclink-task:tsk_01KZZQKQ76R1BRMV7XSK7E51QW -->

## 3. 技能資產與產物三連動

- [x] 3.1 六份技能資產放寬 [M] 敘述（crates/speclink-core/assets/skills/）：apply.md 的 [M] 段改為 manual task 語意（不限測試）並補明文「寫碼任務依賴未勾的 [M] 任務時，停下請使用者先完成，不代勾、不繞過」；ingest.md 與 propose.md 的定義句放寬為「agent 無法代行、使用者必須親手做的事（操作產品驗收、建立外部服務帳號、放置金鑰等）」；propose.md 排除句改為「agent 做得到的任務（寫碼與自動化測試）都不帶標記」方向；review.md、verify.md、quality.md 對 [M] 的稱呼自 manual-verification 改為 manual 語意、行為敘述（時序、點名義務）不變；驗證：六份 diff 人工檢視，對照 delta 條文「apply 技能的手動任務處理」（design「D5：前置手動任務的 apply 指示」——被擋即停）、「ingest 技能的起草標記指引」與「手動任務的起草標記」逐項核對 <!-- speclink-task:tsk_01KZZQKQ76ETJ7VNKKQRGVY2NG -->
- [x] 3.2 crates/speclink-core/src/init.rs 的 MARKER_VERSION 自 v1.19.13 進版至 v1.19.14；重生產物：先 UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden:: 重生 golden，再於乾淨樹 UPDATE_ASSETS_LOCK=1 同指令重生 assets.lock；驗證：不帶開關重跑 cargo test -p speclink-core --test it 全綠 <!-- speclink-task:tsk_01KZZQKQ76EY6X2BCH3R4ASG11 -->
- [x] 3.3 用新建置 CLI 再生本 repo 受管技能檔：cargo build -p speclink-cli 後執行 ./target/debug/speclink update，.claude/skills/ 與 .agents/skills/ 兩工具產出隨資產更新；驗證：抽查 claude 與 codex 的 apply 技能檔含「被 [M] 擋住即停」明文與放寬後語意，git status 盤點再生檔全數屬預期路徑 <!-- speclink-task:tsk_01KZZQKQ76Q273K5XQS7PFB90E -->

## 4. 規格 Purpose 直編

- [x] 4.1 直編兩份正典規格的 Purpose 段為放寬後語意（openspec/specs/manual-task-marker/spec.md：標記語意不限於測試、保證代理不代勾使用者才能完成的手動項；openspec/specs/propose-skill/spec.md：起草時替需人親手操作的任務標上手動任務標記），嚴禁動 ## Requirements 起的任何內容（design「D6：Purpose 直編」的前例）；驗證：./target/debug/speclink validate --specs 全綠 <!-- speclink-task:tsk_01KZZQKQ769G469KNEE6TZFGG0 -->

## 5. 全量驗收

- [x] 5.1 四面驗收全綠：cargo test -p speclink-core --test it（含解析與守門既有測試——覆蓋規格「任務行的手動任務標記與解析」的行為逐位元不變）、npm test -w packages/ui、node --test "scripts/**/*.test.mjs"、./target/debug/speclink validate manual-marker-scope-beyond-tests 皆通過；git status 盤點改動檔集合與 proposal Impact 清單一致（.claude/skills 與 .agents/skills 再生檔除外），並確認 design「D1：規格散文的追討界線」外（client-protocol 等四份規格散文、程式碼註解）零改動；確認四條舊需求「任務行的手動測試標記與解析」「任務列的手動測試徽章」「看板卡片的待手測標示」「手動測試任務的起草標記」皆以 REMOVED＋ADDED 成對宣告承接（design「D3：需求改名機制」），validate 無未宣告刪除 <!-- speclink-task:tsk_01KZZQKQ76MBZ1SBX746R20GQH -->
- [x] [M] 5.2 desktop 目測驗收：開啟 desktop app，含 [M] 任務的變更其任務分頁徽章顯示「手動」二字（勾選後保留不劃線）；寫碼任務全完成、尚餘未勾 [M] 的變更其看板卡片顯示「待手動」章且 tooltip 為「待手動·剩 N 項」；切英文介面各為「Manual」與「Awaiting manual」 <!-- speclink-task:tsk_01KZZQKQ76Y2NC02PZQ6M5H4X6 -->
