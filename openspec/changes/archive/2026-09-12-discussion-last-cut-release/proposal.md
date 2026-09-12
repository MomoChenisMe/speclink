## Why

一份討論的結論拆成多刀依序立案時，記錄以 `conclude --hold` 留在途；2026-09-10 的 discussion-hold-until-release 讓 hold 不再因轉出而清除，代價是最後一刀封存後記錄不會自動隨行封存，使用者要自己判斷「全部做完了」再跑一次 `speclink discuss archive`。`improve-repo-layout` 是新語意上路後第一個走到底的三刀系列：三刀全數封存後記錄仍掛在途，使用者把它當成規格遺留的 bug。引擎缺的訊號只有一個——「這是最後一刀」——而這個事實只在立最後一刀的那一刻才確定（`improve-lifecycle-layer` 結論寫三刀、實際四刀：刀三在立案期拆成兩個變更，後半在前半封存之後才立案），所以不能在寫結論時用刀數預告，只能在轉出最後一刀時標記。

目標使用者：透過 AI 代理跑 SDD、把一份討論分多刀立案的開發者；對應 `/speclink-propose --from-discussion`（轉出最後一刀時標記）、`/speclink-archive`（最後一個封存自動收尾）與 `/speclink-discuss`／`/speclink-improve`（多刀說明文字）三個階段。影響 crate：speclink-core、speclink-cli、speclink-protocol、speclink-remote、speclink-server、apps/desktop/core 與 apps/desktop/src-tauri（呼叫端簽名跟進）。

來源討論：hold-countdown-auto-close（結論已寫，本變更即其 Next）。

## What Changes

- **引擎（speclink-core）轉出動詞加「最後一刀」標記**：`DiscussionHead::promote` 增 `last` 參數；三條轉出路徑（`discuss promote`、`new change --from-discussion`、`discuss seal`）共用的 `promoted_text`／`mark_promoted` 帶 `last` 時，在累加 promoted_to 的同一次寫入移除 frontmatter 的 `hold:` 行。hold 的解除點因此有三個：不帶 --hold 的 conclude、`discuss archive`、帶 `--last` 的轉出。`close_if_finished` 三條件（無在途引用、已有結論、無 hold）與 archive.rs 的隨行封存段一行不改——最後一刀不論封存順序，最後一個封存的變更觸發隨行封存。不帶 `--last` 的轉出行為逐位元不變。
- **CLI（speclink-cli）三個子指令各加布林旗標 `--last`**：`speclink discuss promote <slug> [--name <change>] [--last]`、`speclink discuss seal <slug> <change> [--last]`、`speclink new change <name> --from-discussion <slug> [--last]`（`--last` 不帶 `--from-discussion` 時由 clap 拒絕，exit code 2、stderr 說明相依）。三者不吃 stdin；成功 exit code 0；人眼與 `--json` 輸出與本變更前逐位元一致（含帶 `--last` 時——轉出從不報告 hold，效果由記錄本身與看板呈現）。remote 模式同旗標、同行為。
- **wire（speclink-protocol、speclink-remote、speclink-server）轉出請求加 `last` 欄位**：`PromoteDiscussionRequest`、`CreateChangeRequest`、`BindDiscussionRequest`（link 與 seal 共用；link 端點忽略此欄位，記錄逐位元不變）各增 `last: bool`（camelCase、serde default、缺席即 false、false 不序列化）；server 三個端點直通引擎命令；typed client 三個方法帶參數。conclude 請求、`DiscussionInfo.hold`、看板與系統匣分區規則不動。
- **技能文字同步（propose、discuss、improve 三份 asset；claude 與 codex 兩工具）**：propose 技能在 `speclink new change --from-discussion` 那一步規定「讀結論的刀清單（Decision 段的刀一／刀二／…）與記錄的 promoted_to；本次立的是結論規劃的最後一刀時帶 `--last`；立案期把一刀拆成多個變更時只有拆出的最後一段帶」；discuss 與 improve 技能的多刀說明改為「conclude --hold 一次；最後一刀由 propose 帶 `--last`，最後一個封存自動收尾；忘了帶時記錄留在途，`speclink discuss archive <slug>` 一次收尾」，刪掉「最後一刀封存後手動 discuss archive 收尾」的常態敘述。ASSET_VERSION、render golden、assets.lock 三連動。
- **桌面呼叫端跟進簽名**：本地路徑 `apps/desktop/core/src/discussions.rs` 的 `promote_discussion_at` 與 remote 路徑 `apps/desktop/src-tauri/src/remote.rs` 的 `promote_discussion` 都傳 `last: false`（桌面「轉為變更」按鈕不提供最後一刀標記）；其餘既有的 `CreateChangeRequest` struct literal 與 typed client 呼叫（server 與 remote 的測試）補 `last: false`，斷言不變。

相容性影響：所有轉出動詞不帶 `--last` 時人眼與 `--json` 輸出逐位元不變；帶 `--last` 時輸出同樣不變，唯一可觀察差異是記錄的 `hold:` 行消失、其後最後一個封存把記錄移入 openspec/discussions/archive/（封存輸出多一行既有的「Discussion archived」）。既有 CLI 整合測試「分期轉出的生命週期全程保留、手動封存」的期望值改為「第三刀帶 --last、第三次封存自動收尾」。wire 三個請求多一個選填鍵，舊 server 忽略未知鍵（效果等同未帶 --last）、舊 client 不送即 false。無設定欄位變動。

## Capabilities

### New Capabilities

（無。步驟 3 掃描命中 discussion-docs、discuss-skill、improve-skill、propose-skill、client-protocol、server-verb-api 六份既有規格，全部以 MODIFIED 或 ADDED requirement 承接；desktop-app 與 tray-status-menu 的分區規則不變、不動。）

### Modified Capabilities

- `discussion-docs`：「conclude 以 --hold 保留討論在途」——轉出動詞帶 `--last` 時解除 hold，Example「分期三刀的生命週期」改為第三刀帶 `--last`、第三次封存自動移入 archive；「conclude 於全數轉出變更已封存時順手封存討論」——結尾與隨行封存互補的敘述改為「帶 hold 的記錄等待帶 --last 的轉出清掉旗標後，由最後一個轉出變更封存時隨行封存」。
- `discuss-skill`：「中途轉出教學」——分期轉出教學改為最後一刀由 propose 帶 `--last`、自動收尾，忘帶時才手動 archive。
- `improve-skill`：「candidates 以討論記錄承載」——扇出段的分期立案敘述同步。
- `propose-skill`：新增「從討論轉出最後一刀時帶 --last」要求。
- `client-protocol`：新增「轉出請求攜帶最後一刀旗標」要求（三個請求型別）。
- `server-verb-api`：新增「轉出端點轉傳最後一刀旗標」要求（promote、建立變更、seal 三端點）。

## Impact

- Affected specs: discussion-docs、discuss-skill、improve-skill（MODIFIED）；propose-skill、client-protocol、server-verb-api（ADDED requirement）
- Affected code:
  - New: （無）
  - Modified:
    - crates/engine/speclink-core/src/lifecycle/discuss.rs（`DiscussionHead::promote`、`promoted_text`、`mark_promoted`、`promote`、`seal` 增 `last`；doc 註解）、crates/engine/speclink-core/src/lifecycle/discuss/tests.rs
    - crates/engine/speclink-core/src/command/mod.rs（`Command::DiscussPromote`／`DiscussSeal`／`NewChange` 增 `last`；`run_new_change` 轉傳）、crates/engine/speclink-core/src/command/tests.rs（既有 Command 建構補 `last: false`）
    - crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION bump）
    - crates/engine/speclink-core/assets/skills/propose.md、crates/engine/speclink-core/assets/skills/discuss.md、crates/engine/speclink-core/assets/skills/improve.md
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md、crates/engine/speclink-core/tests/golden/codex.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md（UPDATE_GOLDEN 再生）與 crates/engine/speclink-core/tests/golden/assets.lock（UPDATE_ASSETS_LOCK 再生）
    - crates/adapters/speclink-cli/src/verbs/discuss.rs（Promote／Seal 的 `--last`，fs 與 remote 兩臂）、crates/adapters/speclink-cli/src/verbs/new.rs（`--last` requires `--from-discussion`，fs 與 remote 兩臂）
    - crates/adapters/speclink-cli/tests/it/discuss_conclude_auto_archive.rs（分期生命週期測試改期望；新增三路徑帶 `--last` 解除 hold 的測試）、crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs（promote --last 的請求 body 對照）
    - crates/protocol/speclink-protocol/src/command.rs（三個請求型別增 `last`）
    - crates/protocol/speclink-remote/src/client.rs（`discussion_promote`、`create_change`、`seal_discussion` 帶 `last`）
    - crates/host/speclink-server/src/api/routes.rs（promote、create_change、seal 三個 handler 轉傳 `last`）、crates/host/speclink-server/tests/it/api/discussion_routes.rs
    - 既有 struct literal 與方法呼叫跟進簽名（純機械、不改斷言）：crates/protocol/speclink-remote/tests/it/typed_client.rs、crates/host/speclink-server/tests/it/api/verb_api.rs、crates/host/speclink-server/tests/it/admin/backup_e2e.rs、crates/host/speclink-server/tests/it/e2e_cli.rs、crates/host/speclink-server/tests/it/serverfs_store.rs、crates/host/speclink-server/tests/postgres_store.rs（獨立 `[[test]]` 目標，CI 以 `--test postgres_store` 點名編譯）
    - apps/desktop/core/src/discussions.rs（`promote_discussion_at` 傳 `last: false`）、apps/desktop/src-tauri/src/remote.rs（`promote_discussion` 呼叫 typed client 時傳 `last: false`）、crates/adapters/speclink-node/src/lib.rs（`new change` 的 argv 轉接補 `last` 旗標，與 CLI 同名）
    - .claude/skills/ 與 .agents/skills/ 下由 `speclink update` 再生的 SKILL.md（隨版號再生，不逐檔列）
  - Removed: （無）
