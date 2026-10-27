## 1. 引擎改名與版本連動

- [x] 1.1 registry 改名：crates/speclink-core/src/skills.rs 的技能 id 由 onboard 改為 baseline，asset 常數改指 crates/speclink-core/assets/skills/baseline.md；description 改寫為 baseline 觸發情境句（先陳述「既有專案有行為但無 specs」情境、再說產出），維持「入口路由由技能描述承載」的入口聯集完整。驗證：cargo test -p speclink-core --test it 於 1.4 golden 再生後全綠。 <!-- speclink-task:tsk_01M1E3Z5QW13RQ9VGWYF9FA04B -->
- [x] 1.2 asset 內文改名：crates/speclink-core/assets/skills/onboard.md 改名為 baseline.md，自指名稱全面改用 baseline、內文不留舊稱；流程不變（盤點 codebase、capability map 經 AskUserQuestion 確認後才寫規格、speclink validate --specs --all --strict、不建 change、不改 code）；結尾維持兩條出邊（propose／discuss）且不含命令總表——對應「出口交棒由技能結尾承載」與「去中心化路由不留集中總表」。驗證：內容審視對照 design 的 Implementation Contract，並跑 node --test scripts/vocabulary-guard.test.mjs 確認文案面無 avoid 詞。 <!-- speclink-task:tsk_01M1E3Z5QWD7ER4E3EB1PN6ZS1 -->
- [x] 1.3 ASSET_VERSION 由 v1.24.0 bump 至 v1.25.0（crates/speclink-core/src/init.rs 的常數），speclink --version 顯示 engine v1.25.0。驗證：cargo test -p speclink-core --test it 的資產版本鎖測試於 1.4 lock 再生後全綠。 <!-- speclink-task:tsk_01M1E3Z5QWKB00TJ71ZXFQVX6J -->
- [x] 1.4 golden 與 assets.lock 再生（刻意變更，proposal 已記載）：UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden:: 再生五份 snapshot，乾淨樹上以 UPDATE_ASSETS_LOCK=1 同指令再生 lock；檢視 diff 僅含 onboard→baseline 與版本戳。驗證：不帶環境變數重跑 cargo test -p speclink-core --test it 全綠。 <!-- speclink-task:tsk_01M1E3Z5QW56N0J570SBF9QB6P -->

## 2. update 孤兒清理

- [x] 2.1 於 crates/speclink-core/src/init.rs 的 update 路徑實作 registry 差集 prune，落實 workspace-tools 的「update 清除孤兒技能目錄」：各目標生成完成後，清除該目標 skills 目錄下 speclink- 前綴、不在本次應生成集合的目錄；期望集合按工具（claude 全集、codex 與自訂描述子為 for_codex 子集）與 worktree 政策計算；非 speclink- 前綴目錄不動；刪除失敗以錯誤結束、已生成檔案保留。驗證：2.2 的單元測試。 <!-- speclink-task:tsk_01M1E3Z5QWNGJ1FWZDXRBEW3WV -->
- [x] 2.2 crates/speclink-core/src/init.rs 的 tests 模組新增單元測試：(a) 預置 speclink-onboard 目錄執行 update 後僅存 speclink-baseline；(b) 非 speclink- 前綴的使用者目錄位元不變；(c) speclink- 前綴的非 registry 目錄被清除。驗證：cargo test -p speclink-core init 全綠。 <!-- speclink-task:tsk_01M1E3Z5QWMMK3X565CAQGYDWG -->

## 3. repo 生成物與使用者文件

- [x] 3.1 重建 CLI 後於 repo 根執行 speclink update：.claude/skills/ 與 .agents/skills/ 的 speclink-onboard 目錄消失、speclink-baseline 目錄生成、全部 speclink-* SKILL.md 的 frontmatter 版本戳為 v1.25.0。驗證：git status 盤點生成物異動全數入列，rg 確認兩個 skills 目錄無 onboard 殘留。 <!-- speclink-task:tsk_01M1E3Z5QWS1XMTJN1AZ6THQRA -->
- [x] 3.2 八份使用者文件站名更新（README.md、README.en.md、docs/getting-started.md、docs/getting-started.zh-TW.md、docs/workflow.md、docs/workflow.zh-TW.md、docs/product-status.md、docs/product-status.zh-TW.md）：流程鏈與站別清單改為 baseline、呼叫名改為 /speclink-baseline 與 $speclink-baseline；workflow 兩語言的 baseline 站補「舊稱 onboard」——滿足「完整工作流指南說明用途與使用時機」與「工作流正典逐站列出技能與完成判準」。驗證：rg -i speclink-onboard 於 README 與 docs/ 零命中，並以 onboard 關鍵字搜尋 workflow 兩語言可命中 baseline 站的舊稱補註。 <!-- speclink-task:tsk_01M1E3Z5QWXGGNS58Z3QEJWKFV -->
- [x] 3.3 openspec/LANGUAGE.md 新增「規格基準」詞條：definition 為 baseline 技能的產出（依目前行為建立的第一批正式 specs），why 載明與「Apply baseline」（品質站凍結點）分立，avoid 留空以免守門誤報；docs/workflow.md 兩處裸寫的 baseline（閒置變更檢查句與 drift 站的目的句）補上修飾詞。驗證：node --test scripts/vocabulary-guard.test.mjs 通過。 <!-- speclink-task:tsk_01M1E3Z5QWZJ4HSEWX9ET1WST8 -->

## 4. 隔離驗證與收尾

- [x] 4.1 隔離專案首次 init：於全新目錄以 tools=claude,codex 執行 speclink init，兩側 skills 目錄只生成 speclink-baseline、無 speclink-onboard。驗證：對兩個 skills 目錄做檔案清單斷言。 <!-- speclink-task:tsk_01M1E3Z5QWZCNQ1W89DR9NCB0B -->
- [x] 4.2 隔離專案升級模擬：於 4.1 專案預置 speclink-onboard 目錄與一個非 speclink- 前綴的自建技能目錄，執行 speclink update——舊目錄清除、新舊技能不並存、自建目錄位元不變。驗證：檔案系統斷言。 <!-- speclink-task:tsk_01M1E3Z5QW2BE8C5ECXCQNJ09R -->
- [x] 4.3 生成的 speclink-baseline/SKILL.md 內容審視：frontmatter name 為 speclink-baseline、description 為情境句；內文指示先盤點、capability map 確認後才寫入正式 specs、不建 change、不改 code；出口只有 propose 與 discuss。驗證：對照 design 的 Implementation Contract 逐項核對。 <!-- speclink-task:tsk_01M1E3Z5QWP63ZTSKTVJYJN4T1 -->
- [x] 4.4 收尾全掃與跨面測試：rg -i onboard 全 repo 盤點，非歷史面零殘留，保留清單（封存 changes 與 discussions、@trace 檔案清單、workspace-chooser-onboarding、server e2e 的 team onboarding、scripts/docs-screenshots.mjs 的 team onboarding、workflow 舊稱補註）逐項列出理由；change 橫跨 core／scripts／docs 多面，收尾加跑一次 npm run test:all。驗證：掃描輸出入 change 紀錄、test:all 綠燈。 <!-- speclink-task:tsk_01M1E3Z5QWED6VPN9CYYXXZEAB -->
