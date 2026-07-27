## 0. 規格 delta 覆蓋確認

- [x] 0.1 確認 12 份 delta（本變更 specs/ 下）僅改寫 Spectra 相關子句、凍結語意不變，涵蓋需求：「board_rank 不進 CLI 輸出且既有輸出逐位元不變」「任務完成蘊含開工標記」「變更以 discard 動詞廢棄」「restale_from 記錄變更待重新反映的討論並經 CLI 觀測」「動詞覆蓋與跨入口一致性」「commit 確認閘門所見即所簽」「checkout 內 CLI 測試入口」「討論以 link 動詞併入既有變更」「內容落地以 seal 動詞標記已轉出」「討論重新結論標記已反映變更待重新反映」「remote 區段與模式解析」「儲存重構後既有指令行為保持不變」（含 design「決策二：驗證載體改指向真實存在的測試」的載體改寫）「使用者文件採漸進揭露與單一責任」「中英文文件保持結構與事實對等」「任務取消勾選動詞」「工作流政策的正典歸屬與四層解析順序」「init 內建 Agent 工具選擇」。驗證：speclink validate spectra-legacy-cleanup 通過；逐份 delta 與正典原文 diff 僅含 Spectra 子句差異 <!-- speclink-task:tsk_01KYGWSKFZJ8J50748SS1VKASX -->

## 1. README 與 docs 歷史語氣改寫

- [x] 1.1 依 design「決策五：README 改寫的錨句」改寫 README.md 與 README.en.md 起源段：保留錨句（設計之初以 Spectra App 2.3.1 所附 CLI 為行為參考、連結保留），移除「相容基準」與「parity/golden tests」字樣，回歸保護改述為 golden 與 CLI 整合測試；兩版概念對等（同段落結構、同事實）。驗證：grep -n "Spectra" README.md README.en.md 各僅命中錨句一處；兩版起源段逐句對照無事實差異 <!-- speclink-task:tsk_01KYGWSKFZZ94NXB4CXNCRS66S -->
- [x] 1.2 改寫 docs/platform-architecture.zh-TW.md 中的 Spectra 進行式措辭為同一歷史語氣。驗證：grep -n "Spectra" docs/platform-architecture.zh-TW.md 僅存歷史參考句；node --test "scripts/**/*.test.mjs" 全數通過（docs 相關測試不因本改動變紅） <!-- speclink-task:tsk_01KYGWSKFZHXAG3VXX52NPTJ9B -->

## 2. 內嵌技能資產與 golden

- [x] 2.1 依 design「決策三：內嵌技能資產的三處同步與 golden 再生程序」：確認工作樹乾淨（git status --porcelain 為空）後，改寫 crates/speclink-core/assets/skills/archive.md 的「Unlike Spectra — which documents ... RENAMED ...」對比句為直述 speclink 對 RENAMED Requirements 的實際行為，不提 Spectra。驗證：grep -c "Spectra" crates/speclink-core/assets/skills/archive.md 為 0 <!-- speclink-task:tsk_01KYGWSKFZQK34YGQWGNN7ZZAB -->
- [x] 2.2 執行 cargo test -p speclink-core --test render_golden 確認先紅（golden 尚含舊句），再以 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生 crates/speclink-core/tests/golden/ 下四份快照。驗證：git diff 僅含該對比句的變動、無其他行；再跑 cargo test -p speclink-core --test render_golden 轉綠 <!-- speclink-task:tsk_01KYGWSKFZZT2TFV7Z2CT6W77K -->
- [x] 2.3 同步 .claude/skills/speclink-archive/SKILL.md 與 .agents/skills/speclink-archive/SKILL.md 兩個渲染實例至與 assets 一致。驗證：grep -c "Spectra" 兩檔皆為 0；diff 對照 assets 渲染區段一致 <!-- speclink-task:tsk_01KYGWSKFZBKSQEC6KPT1F60M0 -->

## 3. 源碼註解批次改寫

- [x] 3.1 依 design「決策一：改寫詞彙表（規格與源碼共用）」與「決策四：豁免清單（不動的路徑）」：以 grep -rn "Spectra" 定位 crates/speclink-core/src/、crates/speclink-cli/src/、crates/speclink-cli/tests/、crates/speclink-host/src/、packages/ui/src/、apps/desktop/src/ 下全部命中行（豁免清單路徑除外），逐行依詞彙表改寫註解；不動任何字串常數、識別符或測試斷言值。驗證：上述路徑 grep -ri "spectra" 命中數為 0 <!-- speclink-task:tsk_01KYGWSKFZJMJ6B9V8TA3NPCWE -->
- [x] 3.2 全套測試確認零行為變更：cargo test --workspace 全綠；npm test -w packages/ui 與 npm test -w apps/desktop 全綠（desktop 測試以 Node 20 執行）。驗證：全部測試通過、git diff 不含任何非註解行 <!-- speclink-task:tsk_01KYGWSKFZMM6Z0DJTXKB163Q1 -->

## 4. 收尾驗證

- [x] 4.1 全樹 grep -ri "spectra"（排除 node_modules、target、.git）斷言僅存 design「決策四：豁免清單（不動的路徑）」與錨句：README.md 與 README.en.md 的錨句、docs/platform-architecture.zh-TW.md 的歷史句、openspec/changes/archive/、openspec/discussions/、openspec/LANGUAGE.md、prompt.md、正典規格 @trace 區塊。驗證：命中清單與豁免清單完全一致，無其他殘留 <!-- speclink-task:tsk_01KYGWSKFZ9ZJZKHBCN21X2X2Y -->
- [x] 4.2 執行 speclink validate spectra-legacy-cleanup 與 speclink analyze spectra-legacy-cleanup 確認 artifacts 一致。驗證：validate 通過、analyze 無 Critical 或 Warning <!-- speclink-task:tsk_01KYGWSKFZ96M9EN7ZBK4SDQBJ -->
