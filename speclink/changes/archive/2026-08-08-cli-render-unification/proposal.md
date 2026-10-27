## Why

CLI 的本機（fs）與 remote 兩種模式，同一個動詞的輸出目前是兩份手寫程式碼，parity 靠複製貼上與凍結測試盯著，已出現活漂移：本機 list 會渲染 `(invalid .openspec.yaml)` 標記，remote 版 wire 帶著 meta_error 卻不渲染。專案裡已有正確樣板（status／show／instructions／validate／analyze 走「wire DTO 轉回 core 型別、餵同一支渲染函式」），本變更把它推成全動詞規則，讓兩模式輸出一致從測試保證變成結構保證。

目標使用者是透過 AI 代理跑 SDD 的開發者：CLI 是 agent 的操作面，所有技能（apply／discuss／archive 等）的判讀都吃 CLI 輸出，兩模式輸出同形是技能不分模式運作的前提。對應 workflow 全階段的動詞執行面。

來源討論：improve-cli-command-layer（結論「候選 1 全量版」）。

## What Changes

- **渲染收斂（A 類・wire 已載夠）**：list、discuss 全家（new／list／show／context／add-round／conclude／archive／promote／discard／link／seal）、task done／undone、in-progress remove、discard、station add-round／stamp／discard 的輸出，改為每動詞單一渲染函式、吃 core outcome 型別；remote 路徑一律 wire→core 轉接後餵同一支渲染。模式差異只准活在資料組裝與守門（bail），不准活在渲染。影響 crate：speclink-cli。
- **wire 補欄位（B 類・三層同批）**：`ArchiveResponse` 補 dated_name、各 capability 的 added／modified／removed／renamed 計數、snapshot_created、archived_discussions、evidence_recorded；`ReviewTicketResponse` 補工單原文欄位（station show 人眼路徑印原文用）。全部欄位 serde default 向後相容。server 端點回填新欄位、remote client 對應更新。影響 crate：speclink-protocol、speclink-server、speclink-remote。
- **remote 輸出對齊本機（刻意輸出變更）**：remote list 開始渲染 invalid 標記（漂移修正）；remote archive 改印與本機同形的完整結果（封存目的地、規格計數、封存討論、零證據提示）；remote station show 人眼路徑改印工單原文。
- **station show 的 --json 組裝收斂**：本機走 ticket_json（對外契約組裝）、remote 直印 wire DTO 的兩條路收斂為同一條組裝；落地前先驗證兩者現有形狀是否已同形，以同形結果為準對齊。
- 不新增任何子指令或旗標；stdin 用法與 exit code 全部不變。
- 不涉及設定欄位（openspec/config.yaml、.speclink.yaml 皆不動）；不涉及技能或注入區塊。

**相容性影響**：

- 本機模式：人眼與 --json 輸出零變更，既有凍結對照不動。
- remote 模式：上列三處人眼輸出刻意變更（對齊本機）；對應凍結對照（crates/speclink-cli/tests/it/remote_verb_parity.rs 等）同步更新。這是修正非破壞——變更方向一律是「補上本機已有的資訊」。
- wire 契約：新欄位全部 serde default。舊 server × 新 CLI：缺欄位以 default 缺席，渲染對缺席欄位靜默略過（不炸、退化為現行輸出）；新 server × 舊 CLI：舊 CLI 忽略未知欄位，行為不變。
- 既有使用者無需遷移動作。

## Non-Goals

- 不動本機/remote 的分岔決策結構（22 處 remote_ctx 分岔收攏 dispatch 是候選 2，deferred）。
- 不改 include! 文字包含與檔案切分（候選 3，deferred）。
- wire→core 轉接不搬進 speclink-remote（候選 4，deferred）——轉接維持在 speclink-cli 內、每動詞單層。
- C 類明文分歧不抹平：new change 的 Path 行、list 的 worktree 欄、status --schema 的 remote bail、workflow-config 的 config.yaml 標籤維持現狀（design 列分歧清單與理由）。
- 本機模式輸出不做任何變更（特別是 station show 維持印工單原文）。
- desktop 不在本變更範圍：wire 新欄位向後相容，desktop 是否採用另議。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 動詞輸出的單一渲染規則與兩模式同形要求入契約
- `client-protocol`: ArchiveResponse 與 ReviewTicketResponse 的欄位擴充
- `server-verb-api`: archive 與 station ticket 端點回填新欄位

## Impact

- Affected specs: verb-contract、client-protocol、server-verb-api
- Affected code:
  - New: (none)
  - Modified:
    - crates/speclink-cli/src/commands.rs（渲染函式抽出與共用）
    - crates/speclink-cli/src/remote_commands.rs（刪除重複渲染、補 wire→core 轉接）
    - crates/speclink-protocol/src/command.rs（ArchiveResponse、ReviewTicketResponse 欄位擴充）
    - crates/speclink-server/src/routes.rs（archive 與 station ticket 端點回填）
    - crates/speclink-remote/src/client.rs（client 方法隨 DTO 更新）
    - crates/speclink-cli/tests/it/remote_verb_parity.rs（remote 輸出對照更新）
    - crates/speclink-cli/tests/it/no_raw_wire_json.rs（station show --json 組裝收斂的守門對照）
    - 其餘受影響整合測試（crates/speclink-remote/tests/it/、crates/speclink-server/tests/it/ 下對應檔案）
  - Removed: (none)
