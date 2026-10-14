## Why

透過 AI 代理跑 SDD 的開發者（以及替團隊訂標準的 PO／PM）用 `/speclink-config` 或直接下 `speclink workflow-config` 維護 openspec/config.yaml 時，CLI 的寫入回饋有兩個缺口：寫入審查規則時，工作區技能檔若比引擎舊，舊版審查技能根本不讀 review 區塊，規則形同無效，CLI 卻只印成功訊息（另一個專案實際發生：技能檔仍是舊版時寫入十三條規則，直到人工檢查才發現）；`--dry-run` 的差異預覽則把夾在兩處改動之間、內容沒變的行也列成刪除再加回，使用者得逐字比對才敢確認寫入。兩者都削弱 config 技能「先看 diff、再核准寫入」流程的可信度。本變更是討論 config-skill-rule-routing 規劃的第 1 刀；第 2 刀（config 技能文字）排在 quality-gate-speedup 封存後另行轉出。

## What Changes

- **審查規則寫入後的技能檔過期提示**：`speclink workflow-config review <critical|warning|suggestion> --stdin` 與 `speclink workflow-config review baseline <extend|replace>` 實際寫入成功後，CLI 以引擎既有的技能檔過期探測檢查本機工作區；任一已宣告工具的技能檔判為過期時，於 stderr 多印一行提示，點名過期的工具與版號，並指示執行 `speclink update`。
  - stdout 維持既有的單行成功訊息、exit code 維持 0——提示只走 stderr，依賴 stdout 的腳本不受影響。
  - 探測整體為「較新」（工作區檔案領先引擎）或「無法判定」時不提示：更新會降級或無從判斷；工具技能檔整組缺失不算過期、不提示。
  - `--dry-run` 不探測、不提示（沒有寫入）；remote 模式寫入成功後同樣探測本機工作區的技能檔。
  - 只有 review 寫入加提示：context、rules 與政策欄位由 CLI 在指令注入時讀取，不經技能檔，技能檔新舊不影響它們生效。
- **`--dry-run` 差異只標示實際改動的行**：workflow-config 所有支援 `--dry-run` 的寫入子指令（set、context、rules、review）共用的差異預覽，改為逐行比對——內容相同的行一律以 context 行（前綴空白）呈現，只有真正刪除或新增的行帶 `-`／`+`；輸出仍是單一 hunk 的 unified diff，檔頭與 hunk 標頭格式不變。
- 文件：docs/configuration.md 與 docs/configuration.zh-TW.md 的 review 寫入說明補上過期提示。

### 相容性影響

- `workflow-config review` 寫入：技能檔過期時 stderr 多一行（刻意變更）；stdout 與 exit code 不變。
- 所有 workflow-config 子指令的 `--dry-run` 輸出：改動夾著未改行時，該些行由 `-`／`+` 成對改為 context 行（刻意變更）；只有單一連續改動的輸出位元級不變。CLI 整合測試同批更新。
- 預覽與實際寫入內容一致的保證不變：差異仍由同一份改寫結果產生。
- 不改任何技能範本，引擎的技能版號（ASSET_VERSION）不變，不需要 `speclink update`；不改 `--json` 輸出、不改 config 欄位。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 新增「審查規則寫入後的技能檔過期提示」與「dry-run 差異只標示實際改動的行」兩條需求；既有寫入與 dry-run 需求的其他行為不變。

步驟 3 的規格掃描命中 workflow-config（寫入與 dry-run 的正典）、workspace-tools（技能檔過期探測，本變更只消費、不改其需求）與 desktop-app（desktop 已有指令檔過期提示，本變更不動 desktop）。

## Impact

- Affected specs: workflow-config
- Affected crates／apps：speclink-cli（workflow-config 寫入臂的提示與差異產生）；speclink-core 的技能檔過期探測只被呼叫、不修改；speclink-server、apps/desktop 不受影響
- 生成的技能：無（不改技能範本，claude、codex、copilot 的技能檔都不變）
- Affected code:
  - New: (none)
  - Modified:
    - crates/adapters/speclink-cli/src/verbs/config.rs
    - crates/adapters/speclink-cli/tests/it/workflow_config.rs
    - docs/configuration.md
    - docs/configuration.zh-TW.md
  - Removed: (none)
