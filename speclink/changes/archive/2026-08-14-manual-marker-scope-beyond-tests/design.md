## Context

`[M]` 標記在引擎裡是一個純機械旗標：解析器剝前綴、預測子看 manual 布林、位置 lint 看字面位置，沒有一處判斷「是不是測試」。語意只存在於人讀的文字層——LANGUAGE.md 詞條、desktop 徽章與章、六支技能資產的敘述、規格散文。本變更把這個文字層的語意自「手動測試」放寬為「任何需要使用者親手操作的任務」，引擎零改動。

前例：zh-tw-vocabulary-drawer-and-quality-station（2026-08-14 封存）建立了詞彙收斂的完整模式——LANGUAGE.md 詞條、守門測試、資產三連動；spec-purpose-backfill（2026-08-12 封存）建立了正典規格 Purpose 直編的前例。本設計沿用兩者。

利害關係人：透過 AI 代理跑 SDD 的開發者（看板判讀、任務起草）；claude 與 codex 兩工具的技能產出。

## Goals / Non-Goals

**Goals**

- `[M]` 的正典語意放寬且四個文字面（詞彙、UI、資產、規格）同步收斂，不留新舊兩套詞並存。
- 防漂回：新詞納入詞彙守門，且守門面補上文案實際所在的 packages/ui/src/i18n.tsx。
- apply 技能對「前置型手動任務擋住寫碼任務」的行為有明文指示。

**Non-Goals**（詳見 proposal，此處僅列設計相關者）

- 不追討僅字面提及舊詞的規格散文（D1）。
- 不改引擎行為、不加第二種標記、不改 `[M]` 字母。

## Decisions

### D1：規格散文的追討界線

界線：釘住的實質變了才動。

只對「條文釘住的實質因語意放寬而失效」的規格開 delta：manual-task-marker（標記語意的正典本體）、desktop-app（徽章與章的文案字面被條文釘死）、propose-skill（起草標記需求的定義字面是「手動測試類任務…需使用者實際操作驗證」，與放寬後語意直接矛盾）。client-protocol、quality-skill、review-skill、verify-skill 的條文僅在描述行為時字面提及「手動測試任務」，其釘住的實質（時序、守門、點名義務）放寬後仍逐字成立——沿前例 zh-tw 變更 design D1「不追討存量」的界線不動。

界線的例外（品質關卡回合經使用者裁定納入）：引用「已被本變更改名之需求」的程式碼註解與旁測敘述更新為新名（否則封存後指向不存在的需求，追溯線斷掉）；引擎的使用者可見拒絕訊息（station.rs）與相鄰 doc 註解（query.rs、tasks.rs 的 MANUAL_MARKER 文件）自 "manual-verification" 放寬為 "manual"。規格散文的界線本身不變。

替代案「全量回改所有規格散文」落選：多開四份 delta 只換字面、不換實質，稀釋變更焦點且違反前例界線。

### D2：avoid 詞的語境限定

分流：「手動測試」帶限定、「待手測」不帶。

「手動任務」詞條的 avoid 收「手動測試（此概念上）」：docs/implementation-refactor-roadmap.zh-TW.md 有「每日手動測試」這類指真測試的正當用法，機械守門必然誤命中，依 ui-copy-vocabulary 既有分流規則以語境限定退出機械掃描、由撰稿時人工判斷。「待手測」是自造詞、無其他正當用法，進「待手動」的 avoid 不帶限定、由守門測試機械釘死。原「手動測試」詞條 avoid 欄的存量詞（人工測試、手工測試、手動驗證（標記語境中）、manual test（中文散文中））隨新詞條保留。

### D3：需求改名機制

機制：REMOVED＋ADDED 成對，不用 RENAMED。

含舊詞的需求名（manual-task-marker 一條、desktop-app 兩條、propose-skill 一條）需要「改名＋改文」同時發生。引擎的 RENAMED 只換 header 不動 body，且同一需求名不得跨區段重複宣告——RENAMED 與 MODIFIED 不能疊在同一條需求上。故一律以 REMOVED（舊名）＋ADDED（新名、新文）成對宣告。代價是舊 @trace 隨 REMOVED 消失、新塊以本變更為 source 重新起算——可接受，內容本來就是重寫。

### D4：守門面擴充

擴充：packages/ui/src/i18n.tsx 納入使用者可見文案面。

本次改的徽章與章字串住在 packages/ui/src/i18n.tsx，卻不在 ui-copy-vocabulary 界定的守門面內——「待手動」若日後漂回「待手測」沒有任何機制擋。修 ui-copy-vocabulary 的範圍條文＋scripts/vocabulary-guard.test.mjs 的 SURFACE_FILES 各加這一個檔案。納面時全檔受既有 avoid 詞掃描，實測揭露的存量違規比照前例「連帶修正、經任務載明」處理。

替代案「另開一支 packages/ui 專用守門」落選：同一份 LANGUAGE.md、同一套分流規則，第二支測試是複製貼上。

### D5：前置手動任務的 apply 指示

指示：補「被擋即停」一句，不加排序規則。

起草通則已有依賴排序（tasks.instruction.md 的 Order tasks by dependency），`[M]` 專屬排序規則等於把通則抄一次——不加。真正的洞在 apply.md 的「Skip it and move on」：舊語意下每個 `[M]` 都是驗收後置、跳過永遠正確；放寬後會出現前置型 `[M]`（建外部帳號、放金鑰）擋住下游寫碼任務的情形，照字面跳過會在下游撞牆。補一句明文：寫碼任務依賴未勾的 `[M]` 時，停下來請使用者先完成該手動任務，不得代勾、不得繞過。此指示同時落入 manual-task-marker 的 apply 技能需求條文（MODIFIED），使規格與資產互相釘住。

### D6：Purpose 直編

archive 合併既有規格時 header（含 Purpose）逐位元保留，delta 無法觸及。manual-task-marker 與 propose-skill 兩份 Purpose 含「手動測試」語意，比照 spec-purpose-backfill 前例以任務直編，嚴禁動 Requirements 區之外的其他內容。

## Implementation Contract

**範圍內**：openspec/LANGUAGE.md 兩詞條、packages/ui/src/i18n.tsx 六個詞條值、兩份 UI 測試期望值、六支技能資產敘述、MARKER_VERSION 進版與 golden／assets.lock 重生、四份規格 delta、兩份 Purpose 直編、守門測試 SURFACE_FILES 擴充。
**範圍外**：一切引擎行為、協定欄位、`[M]` 字面、封存區、D1 列出的四份規格散文、識別符；程式碼註解除 D1 例外（已更名需求的引用）外不追討。

可觀察行為與驗收判準：

1. **i18n 字面**（packages/ui/src/i18n.tsx）：zh-TW 的 tasks.manual＝「手動」、card.awaitingManual＝「待手動」、card.awaitingManualTitle＝「待手動·剩 {n} 項」；en 的 tasks.manual＝Manual、card.awaitingManual＝Awaiting manual、card.awaitingManualTitle＝Awaiting manual · {n} left。鍵名一律不變。驗收：npm test -w packages/ui 全綠（含更新後的 taskList 與 awaitingManualBadge 測試）。
2. **技能資產語意**：六支資產不再以 manual-verification／manual test 稱呼 `[M]` 任務，改以放寬後的 manual task 語意（定義句表達「agent 無法代行、使用者親手操作」，不限測試）；apply.md 含「寫碼任務依賴未勾的 `[M]` 時停下請使用者先做」的明文；propose.md 的排除句表達「agent 做得到的（寫碼與自動化測試）都不帶標記」。驗收：golden 快照重生後 cargo test -p speclink-core 全綠。
3. **產物三連動**：MARKER_VERSION 自 v1.19.13 進一版；golden 以 UPDATE_GOLDEN=1 重生、assets.lock 以 UPDATE_ASSETS_LOCK=1 於乾淨樹重生（兩開關獨立，順序：先 bump 版號再重生）。驗收：不帶開關重跑 render_golden 全綠。
4. **詞彙守門**：scripts/vocabulary-guard.test.mjs 的 SURFACE_FILES 含 packages/ui/src/i18n.tsx；LANGUAGE.md 改詞後全守門面 avoid 詞歸零。驗收：node --test scripts/**/*.test.mjs 全綠。
5. **規格**：四份 delta 通過 speclink validate；直編後 speclink validate --specs 全綠。

## Risks / Trade-offs

- [平行 change 同時 bump MARKER_VERSION → 版號行與 golden 對撞] → 沿平行提交衛生慣例：開工前確認無平行 session 持有版號 bump；合併衝突時重生衍生物（golden、assets.lock），不挑邊。
- [packages/ui/src/i18n.tsx 納面後守門揭露存量違規，範圍膨脹] → 違規逐條列給使用者裁定後連帶修正（前例：zh-tw 變更連帶修正兩處）；僅修 avoid 詞字面，不動鍵名與結構。
- [「手動測試」帶語境限定退出機械守門 → 此概念的漂回只能靠人工判斷] → 接受：同類限定（「此概念上」）在既有詞條已是常態；徽章與章的字串本身在機械守門面內，最容易漂的位置仍有硬擋。
- [REMOVED＋ADDED 改名使舊 @trace 斷鏈] → 接受：@trace 的 source 本來就指向最後改寫者，封存後新塊指向本變更即為正確溯源。

## Migration Plan

純文案與正典詞彙，無資料遷移。既有工作區執行 speclink update 取得新技能文案；未執行者維持舊文案、功能不受影響。既有 tasks.md 的 `[M]` 標記零遷移。回滾即 git revert 整批（規格、資產、版號、golden 同批進退）。

## Open Questions

（無——討論 manual-marker-scope-beyond-tests 已裁定全部分歧點。）
