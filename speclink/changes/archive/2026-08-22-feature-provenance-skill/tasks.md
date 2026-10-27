## 1. 引擎：溯源鏈組裝純函式（speclink-core）

- [x] 1.1 以 TDD 先寫紅燈測試再實作 crates/speclink-core/src/trace.rs：鏈組裝純函式吃 Store trait 讀出的資料（正典規格的 @trace 歸屬、封存 change 目錄名清單與各目錄的 delta capability 集合、.openspec.yaml 的 from_discussion、討論 frontmatter 的 promoted_to、.evidence.json 原文），輸出 TraceReport 結構。單元測試（#[cfg(test)]）涵蓋 trace-verb spec 的行為：封存目錄含該 capability 的 delta 子目錄即入鏈、依封存日期由舊至新排序、進行中 change 排除、evidence 檔存在則解析逐 task files 而不存在則為 None、@trace code 清單永不讀取、from_discussion 缺欄與 @trace 歸屬指向不存在封存目錄時的寬容組裝（對應需求「溯源鏈組裝」「evidence 的存在性偵測」「單環髒資料的寬容組裝」）。同步在 crates/speclink-core/src/lib.rs 掛模組。驗證：cargo test -p speclink-core trace 綠燈。 <!-- speclink-task:tsk_01M0MM9ADG6HHJSJTZS3X4SK1N -->
- [x] 1.2 以 TDD 補 capability 不存在的近似建議：trace 入口在 capability 無正典規格時回錯誤，錯誤內容重用 crates/speclink-core/src/capname.rs 的排序邏輯給至多三筆近似名（對應需求「找不到 capability 的近似建議」）。若 Store trait（crates/speclink-core/src/store.rs）缺列舉封存目錄、讀 .evidence.json 或讀討論 frontmatter 的把手，於本 task 增讀取方法並由 fs adapter（crates/speclink-fs/src/layout.rs 與 crates/speclink-store-fs 對應實作）補齊。驗證：cargo test -p speclink-core trace 綠燈、cargo test -p speclink-store-fs 綠燈。 <!-- speclink-task:tsk_01M0MM9ADG4GGNENK8AACDHKCF -->

## 2. CLI：trace 動詞與雙輸出

- [x] 2.1 先寫紅燈整合測試 crates/speclink-cli/tests/it/trace.rs 並掛入 crates/speclink-cli/tests/it/main.rs：對含封存演進的 fixture 專案斷言人讀縮排樹（由舊至新、封存目錄名、來源討論 slug、兄弟變更與其 capability、Requirement 歸屬段）、--json payload 結構（capability／requirements／changes／discussions 四鍵存在、camelCase 欄位名與型別、evidence null 路徑、payload 外無雜訊）、--no-color 下內容不變僅無色碼、capability 不存在時非零 exit 且 stderr 帶至多三筆建議而 stdout 無成功 payload（對應需求「溯源鏈組裝」「--json 輸出形狀」「evidence 的存在性偵測」「找不到 capability 的近似建議」「單環髒資料的寬容組裝」的 Scenario）。驗證：cargo test -p speclink-cli --test it trace 先紅。 <!-- speclink-task:tsk_01M0MM9ADGCAWE52X1EBQCS8CX -->
- [x] 2.2 實作 crates/speclink-cli/src/verbs/trace.rs 並在 crates/speclink-cli/src/verbs/mod.rs 註冊、經 crates/speclink-core/src/command/mod.rs 的 dispatch 慣例接上 1.x 的組裝函式，交付人讀樹與 --json 雙輸出讓 2.1 轉綠。驗證：cargo test -p speclink-cli --test it trace 綠燈。 <!-- speclink-task:tsk_01M0MM9ADGTRKW3WJ7WY0HV131 -->

## 3. 技能資產：speclink-trace

- [x] 3.1 撰寫技能資產 crates/speclink-core/assets/skills/trace.md 並依 crates/speclink-core/src/skills.rs 既有慣例註冊為新技能：內容依 trace-skill spec 規定——canon pass、呼叫 speclink trace --json、讀討論結論與提案動機、依 evidence 讀檔、live code 收尾、evidence null 的 git 反查（標明盡力線索）、查無規格的 codebase 考古、答案禁用內部處置字眼並附來源路徑（對應需求「問題對應與敘事答案」「evidence 缺失的靜默補查」「查無規格的考古降級」「降級不可見原則」）。連動 bump MARKER_VERSION 並再生 crates/speclink-core/tests/golden（含 assets.lock）。驗證：cargo test -p speclink-core --test it render_golden 綠燈（新技能 golden 為同批刻意更新）。 <!-- speclink-task:tsk_01M0MM9ADG7AV7DVTR2XADF2E2 -->
- [x] 3.2 驗證發佈路徑：在本 repo 執行 speclink update，確認 .claude/skills/speclink-trace/SKILL.md 生成且內容與資產一致（對應需求「技能資產發佈」）；再生的其餘 SKILL.md 版號行不進本 change 的提交範圍之外（收尾以 git status 盤點）。驗證：speclink update 後檔案存在、git status 僅含預期檔案。 <!-- speclink-task:tsk_01M0MM9ADG6V872PECSTX4524F -->

## 4. 詞彙與收尾

- [x] 4.1 openspec/LANGUAGE.md 新增「溯源」詞條：definition（沿規格→變更→討論→程式碼的鏈回答功能怎麼來的；對應引擎動詞 trace）、avoid（追溯、trace（中文散文中））、why（動詞直說結果、與技能／動詞名對齊）。驗證：內容審閱——詞條三欄齊備且不與既有詞條衝突。 <!-- speclink-task:tsk_01M0MM9ADGZVXYVTGVPD7B4GG1 -->
- [x] 4.2 收尾回歸：跨面改動後逐 crate 跑受影響面測試——cargo test -p speclink-core、cargo test -p speclink-core --test it render_golden、cargo test -p speclink-cli --test it trace、cargo test -p speclink-store-fs。驗證：全數綠燈。 <!-- speclink-task:tsk_01M0MM9ADGK8M9DT5YDVGB91RK -->
- [x] [M] 4.3 真實驗收：對本 repo 一個有封存演進的 capability（如 discussion-docs）跑一次 /speclink-trace 的完整問答，確認敘事答案含決策、被否方案、關聯規格與來源路徑，且無內部處置字眼。驗證：使用者認可答案品質。 <!-- speclink-task:tsk_01M0MM9ADG339VP9E0MHFCET15 -->
