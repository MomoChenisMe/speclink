## Why

CLI 指令層的三個檔案（main.rs 917 行、commands.rs 3,217 行、remote_commands.rs 1,469 行）以 include! 文字包含拼成單一 5,603 行編譯單元，零模組邊界——任何程式都看得到任何東西，越界依賴無阻力也無痕跡。切法也切錯方向：按「本機檔／remote 檔」分層，但實際改動都以動詞為單位，同一動詞的完整故事（clap 參數在 main.rs、fs 臂與渲染在 commands.rs、remote 臂與轉接在 remote_commands.rs）散在三檔，近三月 140 檔次共動為實證。前兩刀（cli-render-unification 渲染單份化、cli-mode-dispatch-convergence 的 dispatch 模式表）已把形狀整理好，本刀是純搬移的收官：include! 改真模組、按動詞族重切。

## What Changes

- 拆除 main.rs 尾端的 include! 文字包含，crates/speclink-cli 全面改為真正的 Rust 模組結構。
- 按動詞族重切為 13 個族檔（src/verbs/ 子目錄）：init（init、update）、connection（link、unlink、auth）、query（list、show、status）、checks（validate、analyze、drift）、lifecycle（archive、discard、claim）、progress（task、in-progress）、new、instructions、documents（artifact、language）、station（review、verify）、discuss、config（config、workflow-config）、toolchain（schemas、templates、schema、completion、feedback、demo）。每個族檔裝齊該族完整故事：clap 參數定義（自 main.rs 遷入）、fs 臂、remote 臂、渲染、wire→core 轉接、內嵌單元測試。
- 底座模組留在 src/：main.rs 瘦身後保留 Cli／Commands enum、dispatch 模式表、dual／fs_only／remote_only 模式組合子與 main()；新增 remote_base.rs 收 RemoteCtx、remote_ctx() 握手、remote_resolve_change 等 remote 共通件；新增 common.rs 收 run_command、print_json、read_stdin 系列、require_workspace、open_project、info_if_no_changes、warn_ 系列等通用管線件；color.rs 不動。
- 依賴紀律立為硬規則：族檔之間禁止互相 import，跨族共用件一律升底座；可見性最小 pub(crate) 面——族檔僅對外開放 dispatch 表要呼叫的臂函式與 Commands enum 要引用的 clap 參數型別，渲染／轉接／helper／測試全私有。
- 行為零變更：純搬移，不改任何邏輯、輸出、exit code；凍結輸出整合測試逐位元不動即是守門，搬移後零 dead-code warning 為輔助斷言。
- 相容性影響：speclink CLI 的行為、輸出、artifacts 均不變——本 change 只動 crates/speclink-cli/src 的檔案切分與模組可見性。

## Capabilities

### New Capabilities

（無——CLI 原始碼模組結構重組，不引入能力）

### Modified Capabilities

（無——無任何 spec 層級行為變更；正典規格中出現的 crates/speclink-cli/src 舊檔名引用皆位於 @trace 出處註記區塊，屬歷史紀錄，依專案慣例不回改）

## Impact

- Affected specs:（無）
- Affected code:
  - New:
    - `crates/speclink-cli/src/verbs/mod.rs`（族模組宣告，依 repo 目錄模組慣例）
    - `crates/speclink-cli/src/verbs/init.rs`
    - `crates/speclink-cli/src/verbs/connection.rs`
    - `crates/speclink-cli/src/verbs/query.rs`
    - `crates/speclink-cli/src/verbs/checks.rs`
    - `crates/speclink-cli/src/verbs/lifecycle.rs`
    - `crates/speclink-cli/src/verbs/progress.rs`
    - `crates/speclink-cli/src/verbs/new.rs`
    - `crates/speclink-cli/src/verbs/instructions.rs`
    - `crates/speclink-cli/src/verbs/documents.rs`
    - `crates/speclink-cli/src/verbs/station.rs`
    - `crates/speclink-cli/src/verbs/discuss.rs`
    - `crates/speclink-cli/src/verbs/config.rs`
    - `crates/speclink-cli/src/verbs/toolchain.rs`
    - `crates/speclink-cli/src/remote_base.rs`
    - `crates/speclink-cli/src/common.rs`
  - Modified:
    - `crates/speclink-cli/src/main.rs`（移除 include!，瘦身為 clap 頂層＋dispatch 模式表＋模式組合子＋main()，加掛 mod 宣告）
  - Removed:
    - `crates/speclink-cli/src/commands.rs`（內容全數遷入族檔與底座）
    - `crates/speclink-cli/src/remote_commands.rs`（內容全數遷入族檔與底座）
