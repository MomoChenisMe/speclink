## Why

把引擎的結果轉成輸出的程式都寫在 CLI 裡：約 30 個 render 函式、約 300 處直接印到 stdout。它們不能被別的 crate 呼叫，也不能在沒有終端機的程序裡使用。Node SDK 因此對它支援的 9 個動詞各自另外組一次 `--json` 的內容，其餘動詞一律拒絕。宿主的 Agent 工具需要 discuss、propose、archive 用到的全部動詞，而且拿到的輸出要和技能預期的 CLI 輸出相同。

討論 embedded-host-needs 定案的「動詞共用一份」查證後拆成三個變更。這是第二個：把輸出收成共用的純函式，讓 SDK 涵蓋全部有引擎命令的動詞。它排在變更 shared-verb-parsing 之後。

- **目標使用者**：用 Node SDK 的整合者（dispatch 能用全部動詞）；維護 speclink 的開發者（輸出只有一份）。使用 CLI 的開發者看到的輸出不變。
- **使用情境**：每一個會讀寫規格的動詞，涵蓋全部 workflow 階段。技能的文字不變。
- **既有規格的對照**：command-runtime 保證各入口語意一致與既有輸出不變，但沒有規定輸出的實作只有一份；node-sdk 的「dispatch 的蓋章動詞」明寫 review 與 verify 只支援 add-round 與 stamp，這個變更放寬它。verb-contract 的兩模式同形與 `--json` 凍結不變，這個變更靠它們當回歸對照。

## What Changes

- `speclink-verbs` 新增 render 模組：每個動詞一個「結果轉成 `--json` 的值」的函式（該動詞有 `--json` 形式時）與一個「結果轉成人眼文字」的函式。它們是純函式：不印出、不讀終端機、不讀環境變數。
- 人眼文字的顏色由參數決定，不讀全域狀態。ANSI 色碼只在 `speclink-verbs` 定義一次；CLI 自己留下的家族經同一份實作上色。
- 新增單一出口：給解析結果、引擎的結果與輸出選項（是否 `--json`、是否上色），回傳 stdout 的文字、stderr 的文字與 `--json` 的值。
- `speclink-cli` 十一個家族的 render 函式搬進 `speclink-verbs`。CLI 只負責把回傳的文字印出。本機與遠端兩條分支繼續用同一個輸出函式。
- 新增守門測試：`speclink-verbs` 的原始碼裡不能出現直接印出的巨集。
- `speclink-node` 的 dispatch 移除 9 個動詞的允許清單與各自組 `--json` 的程式，改成「共用的解析 → 引擎 → 共用的輸出」。結果：有引擎命令的動詞全部能用。
- **相容性影響**：
  - CLI：每個動詞的 stdout、stderr、exit code 在人眼、`--json`、`--no-color` 三種形式下逐位元不變。
  - Node SDK：原本 9 個動詞的回傳值不變。原本回傳 `invalid_argv` 的動詞（例如 `show`、`validate`、`archive`、`discuss` 全系列、`review show`、`review discard`、`task done`）現在會執行並回傳結果。依賴「這些動詞一定被拒絕」的宿主要調整。
  - 只能在有 checkout 的一端執行的動詞（drift、change rank、review prepare、review scope、verify scope、帶 `--skill` 的 instructions）在 SDK 仍被拒絕。
- 沒有新的 CLI 指令、設定欄位或技能變動。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `command-runtime`: 新增一條需求——動詞的輸出只有一份定義，是不印出的純函式，CLI 與 Node SDK 共用。
- `node-sdk`: 「dispatch 的輸入輸出契約」補上 dispatch 涵蓋全部有引擎命令的動詞；「dispatch 的蓋章動詞」放寬成審查與驗證的 add-round、show、stamp、discard 都能用。

## Impact

- Affected specs: command-runtime（新增一條）、node-sdk（修改兩條）
- Affected crates: `speclink-verbs`（變更 shared-verb-parsing 新增的套件）、`speclink-cli`、`speclink-node`。`speclink-core`、`speclink-host`、`speclink-server`、store crates、桌面 app 不改。
- Affected code:
  - New:
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/paint.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/adapters/speclink-verbs/src/render/checks.rs
    - crates/adapters/speclink-verbs/src/render/documents.rs
    - crates/adapters/speclink-verbs/src/render/trace.rs
    - crates/adapters/speclink-verbs/src/render/instructions.rs
    - crates/adapters/speclink-verbs/src/render/new.rs
    - crates/adapters/speclink-verbs/src/render/progress.rs
    - crates/adapters/speclink-verbs/src/render/lifecycle.rs
    - crates/adapters/speclink-verbs/src/render/plan.rs
    - crates/adapters/speclink-verbs/src/render/discuss.rs
    - crates/adapters/speclink-verbs/src/render/station.rs
    - crates/adapters/speclink-verbs/tests/no_direct_print.rs
  - Modified:
    - crates/adapters/speclink-cli/src/color.rs
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
    - crates/adapters/speclink-cli/tests/it/no_raw_wire_json.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts
    - crates/adapters/speclink-node/__test__/review-verify-verbs.spec.ts
  - Removed: (none)
- 三種形態下的結果：
  - 本機 repo：可用。CLI 的輸出逐位元不變。
  - 連到遠端的 checkout：可用。CLI 的遠端分支用同一個輸出函式，輸出不變。
  - 沒有本機規格檔的內嵌宿主：可用，有兩項已知缺口。SDK 的 dispatch 能用全部有引擎命令的動詞；流程設定的動詞還不行（由變更 workflow-config-commands 補上）；每次呼叫帶身分、事件交給宿主還沒有（由變更 host-kit-agent-door 補上）。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 列出「只支援 9 個動詞」的段落要改寫。
