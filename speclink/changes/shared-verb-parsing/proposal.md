## Why

把動詞文字（argv）轉成引擎命令的程式今天有兩份。CLI 用 clap 定義全部動詞，但 `speclink-cli` 是只有執行檔的 crate，型別都不公開，別的 crate 用不到。Node SDK 因此另外手寫了一份，只認得 9 個動詞，而且行為和 CLI 不一樣：不認得的旗標被靜默忽略、多出來的位置參數被忽略、不接受 `--name=value` 的寫法。每新增一個動詞要改兩處，第二份會持續落後。

討論 embedded-host-needs 定案：動詞文字轉成引擎命令只留一份，CLI 與 SDK 共用。查證後這件事拆成三個變更；這是第一個，只處理解析。它和儲存接口的兩個變更沒有相依，能同時進行。

- **目標使用者**：用 Node SDK 的整合者（argv 的行為和 CLI 一致）；維護 speclink 的開發者（新增動詞只改一處）。透過 AI agent 使用 CLI 的開發者看到的行為不變。
- **使用情境**：每一個會讀寫規格的動詞，涵蓋全部 workflow 階段。技能的文字不變。
- **既有規格的對照**：command-runtime 規定命令執行層只有一份，但沒有規定「動詞文字怎麼變成命令」；node-sdk 規定 dispatch 的 argv 和 CLI 的動詞詞彙一對一，但實作是另一份解析。verb-contract 與 workflow-config 不在這個變更的範圍。

## What Changes

- 新增函式庫 crate `speclink-verbs`，放在 adapters 底下，依賴 `speclink-core` 與 clap。`speclink-cli` 與 `speclink-node` 都依賴它。
- 讀寫規格的十一個動詞家族的 clap 定義，從 `speclink-cli` 原樣搬進 `speclink-verbs` 並公開：查詢（list、show、status）、檢查（validate、analyze、drift）、文件（artifact、language）、trace、instructions、new、進度（task、in-progress）、生命週期（archive、discard、claim）、計畫（plan、change）、discuss、品質關卡（review、verify）。說明文字、旗標名稱、預設值都不變。
- 新增解析入口：收一串動詞文字，回傳解析結果或解析錯誤。它不結束程序，也不印出任何東西。
- 新增「解析結果轉成引擎命令」的單一函式。需要呼叫端提供的資料——stdin 的內容、worktree 的事實、改了哪些檔與 HEAD commit、看板的順序——經一個輸入結構交進來，函式本身不讀 stdin、不讀檔、不呼叫 git。
- 沒有對應引擎命令的動詞（drift、change rank、review prepare、review scope、verify scope、instructions 的 `--skill`）由這個函式明確回報「只能在有 checkout 的一端執行」，不回傳命令。
- 每個動詞的 stdin 規則（不讀、帶 `--stdin` 才讀、帶 `--stdin` 或 stdin 不是終端機就讀）由解析結果回報，CLI 依它決定要不要讀。
- `speclink-cli` 改用這些公開的型別與轉換函式。CLI 的指令、旗標、`--help` 文字、人眼輸出、`--json`、exit code 都不變。初始化、連線、設定、schema 工具、completion 與 workflow-config 這些家族留在 CLI，不搬。
- `speclink-node` 的 dispatch 改用共用的解析，移除手寫的 `parse_argv`。能用的動詞仍是 9 個（list、status、new change、new artifact、claim、review add-round、review stamp、verify add-round、verify stamp）。
- **相容性影響（Node SDK 的 dispatch）**：
  - 不認得的旗標與多出來的位置參數，原本被靜默忽略，現在以 `invalid_argv` 拒絕。
  - 解析錯誤的 `message` 改成和 CLI 相同的文字。`code` 仍是 `invalid_argv`。
  - `--name=value` 與 CLI 有的短旗標現在被接受。
  - 9 個動詞成功時的回傳形狀不變。argv 帶 `--json` 時結果相同。
  - 遷移：傳了 CLI 沒有的旗標的宿主，要把那些旗標拿掉。
- **相容性影響（CLI，apply 期定案）**：
  - `speclink completion generate <shell>` 產出的腳本裡，子指令的排列順序改變（共用的動詞以 flatten 併入頂層，排在一起）。補全的功能不變；`--help` 的順序用 display_order 還原，逐位元相同。
  - `review stamp`／`verify stamp` 的共用定義多一個隱藏的 `--stdin`（給沒有 checkout 的宿主帶蓋章 JSON）。`--help` 不變；補全腳本產生前把它從兩個 stamp 拿掉，所以補全也不列出它。CLI 收到它時以錯誤拒絕：原本是 clap 的解析錯誤（exit 2），改成說明 CLI 自己從 checkout 算蓋章範圍的錯誤（exit 1）。
- 沒有新的 CLI 指令、設定欄位或技能變動。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `command-runtime`: 新增一條需求——動詞文字轉成引擎命令的解析只有一份，CLI 與 Node SDK 共用；解析不結束程序、不讀本機事實；沒有引擎命令的動詞明確回報。
- `node-sdk`: 「dispatch 的輸入輸出契約」補上 argv 由共用的解析處理：不認得的旗標與多出來的位置參數被拒絕，寫法與錯誤訊息和 CLI 相同。

## Impact

- Affected specs: command-runtime（新增一條）、node-sdk（修改一條）
- Affected crates: 新增 `speclink-verbs`；修改 `speclink-cli`、`speclink-node`。`speclink-core`、`speclink-host`、`speclink-server`、store crates、桌面 app 不改。
- Affected code:
  - New:
    - crates/adapters/speclink-verbs/Cargo.toml
    - crates/adapters/speclink-verbs/src/lib.rs
    - crates/adapters/speclink-verbs/src/parse.rs
    - crates/adapters/speclink-verbs/src/inputs.rs
    - crates/adapters/speclink-verbs/src/families/mod.rs
    - crates/adapters/speclink-verbs/src/families/query.rs
    - crates/adapters/speclink-verbs/src/families/checks.rs
    - crates/adapters/speclink-verbs/src/families/documents.rs
    - crates/adapters/speclink-verbs/src/families/trace.rs
    - crates/adapters/speclink-verbs/src/families/instructions.rs
    - crates/adapters/speclink-verbs/src/families/new.rs
    - crates/adapters/speclink-verbs/src/families/progress.rs
    - crates/adapters/speclink-verbs/src/families/lifecycle.rs
    - crates/adapters/speclink-verbs/src/families/plan.rs
    - crates/adapters/speclink-verbs/src/families/discuss.rs
    - crates/adapters/speclink-verbs/src/families/station.rs
  - Modified:
    - Cargo.toml
    - Cargo.lock
    - crates/adapters/speclink-cli/Cargo.toml
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/common.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/src/verbs/checks.rs
    - crates/adapters/speclink-cli/src/verbs/documents.rs
    - crates/adapters/speclink-cli/src/verbs/trace.rs
    - crates/adapters/speclink-cli/src/verbs/instructions.rs
    - crates/adapters/speclink-cli/src/verbs/new.rs
    - crates/adapters/speclink-cli/src/verbs/progress.rs
    - crates/adapters/speclink-cli/src/verbs/lifecycle.rs
    - crates/adapters/speclink-cli/src/verbs/plan.rs
    - crates/adapters/speclink-cli/src/verbs/discuss.rs
    - crates/adapters/speclink-cli/src/verbs/station.rs
    - crates/adapters/speclink-cli/src/verbs/toolchain.rs
    - crates/adapters/speclink-cli/tests/it/no_raw_wire_json.rs
    - crates/adapters/speclink-cli/tests/it/review_verbs.rs
    - crates/adapters/speclink-node/Cargo.toml
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts
  - Removed: (none)
- 三種形態下的結果：
  - 本機 repo：可用。CLI 的行為與輸出逐位元不變。
  - 連到遠端的 checkout：可用。CLI 的遠端分支用同一份解析結果，送出的請求不變。
  - 沒有本機規格檔的內嵌宿主：部分可用，其餘是已知缺口。SDK 的 9 個動詞改用共用的解析；其他動詞仍被拒絕，由變更 shared-verb-output 補上。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 要補上「不認得的旗標會被拒絕」。
