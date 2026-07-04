## 1. 四層解析順序（config）

- [x] 1.1 撰寫解析順序矩陣測試：於 `crates/speclink-core/src/config.rs` 的 #[cfg(test)] 模組新增 4 鍵（locale、spec_locale、tdd、audit）× 4 層來源（環境變數、舊 app 鍵、config.yaml、預設）的組合測試，含非法布林環境變數落層案例——紅燈
- [x] 1.2 實作使測試轉綠：WorkflowConfig 增加 nullable 的 tdd、audit 欄位（serde 可讀既有檔）；新增四層解析函式與 SPECLINK_LOCALE／SPECLINK_SPEC_LOCALE／SPECLINK_TDD／SPECLINK_AUDIT 讀取；保留既有 resolve_locale／resolve_spec_locale 的對外行為相容
- [x] 1.3 重構：解析邏輯集中於單一模組路徑，cargo clippy 無新警告；驗證 cargo test -p speclink-core 全綠（覆蓋需求：工作流政策的正典歸屬與四層解析順序）

## 2. deprecation 警告

- [x] 2.1 撰寫警告行為測試：於 `crates/speclink-cli` 的整合測試新增「含舊鍵時 stderr 恰一行且列出全部鍵名、stdout JSON 不變、無舊鍵時無警告」斷言（對應指令：speclink list --json）——紅燈
- [x] 2.2 實作：`crates/speclink-cli/src/commands.rs` 於指令進入點檢查 AppConfig 舊政策鍵並輸出單行固定前綴警告至 stderr——綠燈
- [x] 2.3 驗證：cargo test 全綠；手動執行 speclink list 於含舊鍵 fixture 確認警告文字與單行性（覆蓋需求：舊政策鍵的 deprecation 警告）

## 3. init 拆分與範本

- [x] 3.1 撰寫範本快照測試：斷言 speclink init 生成的 openspec/config.yaml 含四個政策鍵的註解示例區、.speclink.yaml 不含政策鍵——紅燈
- [x] 3.2 實作：`crates/speclink-core/src/init.rs` 重組為 workspace init（marker、技能、settings、gitignore）與 store init（openspec/ 樹、config.yaml 範本）兩個內部函式；更新兩個範本常數內容——綠燈
- [x] 3.3 驗證：cargo test 全綠；speclink init 於暫存目錄實跑，檢視兩檔內容與 exit code 0（覆蓋需求：init 範本的政策寫入位置）

## 4. tools 自訂描述子

- [x] 4.1 撰寫描述子測試：解析（字串與物件雙形式）、驗證失敗案例（name 衝突、kebab-case 違規、路徑逸出、invocation 非法值——斷言 exit code 非 0 與單行訊息）、生成（skills_dir 與 instructions_file 效果）、移除後 update 清理（目錄移除、marker 剝除、空檔刪除）——紅燈
- [x] 4.2 實作：`crates/speclink-core/src/config.rs` 的 tools 欄位改為 serde 雙形式；`crates/speclink-core/src/init.rs` 的 generate／prune 路徑支援描述子（與內建工具同生命週期）——綠燈
- [x] 4.3 重構：描述子驗證獨立為可單測函式；驗證 cargo test 全綠（覆蓋需求：tools 自訂描述子的接受與驗證、描述子的同步與清理生命週期）

## 5. 中性渲染目標

- [x] 5.1 撰寫渲染 golden 測試：neutral 目標的 cli 與 tool-call 兩種措辭各一份 golden；claude 與 codex 既有輸出以 golden 鎖定位元級不變——紅燈
- [x] 5.2 實作：`crates/speclink-core/src/skills.rs` 渲染目標抽象為內建 claude／內建 codex／描述子三態，描述子走 neutral 本體（無 slash 前綴、無 plan mode 參照、依 invocation 措辭）——綠燈
- [x] 5.3 驗證：cargo test 全綠，golden 比對通過（覆蓋需求：中性渲染目標）

## 6. instructions 來源切換與回歸

- [x] 6.1 撰寫測試：instructions payload 的 locale／tdd／audit 取值改為四層解析結果的斷言（含只設 config.yaml、舊鍵覆寫、環境變數覆寫三情境，指令：speclink instructions proposal --change 某 change --json）——紅燈
- [x] 6.2 實作：`crates/speclink-core/src/instructions.rs` 政策注入值改取解析結果，`--json` 欄位名不變——綠燈
- [x] 6.3 驗證（回歸對照刻意更新）：更新 parity／color／twin 對照 fixture 至新版設定佈局（記錄哪些情境因警告行與範本內容屬刻意分歧），重跑三套對照全數通過

## 7. 設定篇文件

- [x] 7.1 撰寫 `docs/configuration.md` 與 `docs/configuration.zh-TW.md`：兩檔一目錄體系（.speclink.yaml／openspec/config.yaml／.speclink/）、政策歸屬判定規則（政策跟 store、綁定跟 repo、個人差異跟環境變數）、四層解析順序表、描述子欄位說明與範例、自舊佈局遷移指引；`README.md` Documentation 章節增列雙語連結
- [x] 7.2 驗證：README 引用路徑存在；cargo build --release 成功
