## Why

目前 `.speclink.yaml` 同時承載三種性質的設定：工作流政策（locale、spec_locale、tdd、audit）、workspace 設定（tools、spec_dir）與（隱含的）儲存綁定。十六輪討論證明：在團隊情境下，server 端讀不到任何 repo 的 `.speclink.yaml`，工作流政策若留在宿主檔案會產生雙真相（第 10 輪反事實檢驗）。本 change 把設定歸屬整理為「政策跟 store、workspace 設定跟 repo、個人差異跟環境變數」三分，並把 tools 從封閉枚舉開放為可自訂描述子——這是後續 remote 模式（verb-contract-and-remote-client）與 SDK（node-sdk）的設定前提。

目標使用者：現有純本地使用者（遷移壓力最小化）、使用客製 AI harness 的本地開發者（tools 描述子）、以及後續團隊情境的 PO/PM/RD（政策單一真相）。

## What Changes

- **政策欄位搬家**：`locale`、`spec_locale`、`tdd`、`audit` 的正典歸屬改為 `openspec/config.yaml`（WorkflowConfig，本已支援 locale/spec_locale，補上 tdd/audit）；`speclink init` 範本改為把政策欄位寫入 config.yaml，`.speclink.yaml` 範本瘦身為 tools 與 spec_dir。
- **解析順序**：SPECLINK_* 環境變數（個人/CI 覆寫）＞ `.speclink.yaml` 舊政策鍵（相容層，讀取時於 stderr 輸出 deprecation 警告，指引搬至 config.yaml）＞ `openspec/config.yaml`（正典）＞ 內建預設。既有專案不改檔案即可繼續運作。
- **tools 開放自訂描述子**：`tools` 清單除內建名（claude、codex）外接受物件描述子（name、skills_dir、instructions_file、invocation: cli|tool-call）；init/update 對描述子與內建工具一視同仁（生成、同步、清理）。
- **新增中性渲染目標**：技能與指令區塊的渲染支援 neutral 目標（無 slash 前綴、無 plan mode 語彙、invocation 決定動詞措辭），作為自訂描述子的渲染基底。
- **init 內部拆分**：初始化重組為 workspace init（指令檔、技能、settings、gitignore——永遠本地、不需網路）與 store init（建 openspec/ 樹——僅 fs 儲存）兩個階段；本 change 內對外行為僅範本內容改變，拆分為 remote 模式鋪路。
- 新增設定篇雙語文件（兩檔一目錄體系、歸屬判定規則、遷移指引），README 增列連結。

## Non-Goals

（範圍排除與被否決方案記錄於 design.md 的 Goals / Non-Goals 章節。）

## Capabilities

### New Capabilities

- `workflow-config`: 工作流政策欄位的正典歸屬（openspec/config.yaml）、四層解析順序（環境變數＞舊鍵相容層＞正典＞預設）、deprecation 警告行為、init 範本寫入位置。
- `workspace-tools`: tools 自訂描述子的格式與驗證、init/update 對描述子的生成／同步／清理、中性（neutral）渲染目標的技能與指令區塊輸出。

### Modified Capabilities

（無——「政策正典值經儲存介面提供、宿主層疊加覆寫」的需求歸入新 capability `workflow-config`，不回頭修改 store-abstraction 的既有需求。）

## Impact

- Affected specs: 新增 `workflow-config`、`workspace-tools`
- Affected crates: speclink-core（config、init、skills、instructions）、speclink-cli（警告輸出）、speclink-fs（config.yaml 序列化含新欄位）
- 相容性影響: 人眼輸出新增 deprecation 警告行（僅在 `.speclink.yaml` 含舊政策鍵時，輸出至 stderr）——屬刻意分歧，parity 對照需同步更新此情境；`speclink init` 產生的兩個範本內容改變；`--json` payload 無欄位變更；既有專案（舊鍵仍在 .speclink.yaml）行為值不變、僅多警告
- 設定欄位: `openspec/config.yaml` 新增 `tdd`、`audit`（nullable，預設沿用現行——tdd 預設 false、audit 預設 false，與現行 AppConfig 預設一致）；`.speclink.yaml` 的 `locale`、`spec_locale`、`tdd`、`audit` 降為 deprecated 相容鍵；新增環境變數 SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT
- 技能/marker 影響: 新增 neutral 渲染目標（不影響既有 claude/codex 生成內容）；自訂描述子生成的技能檔落於描述子指定目錄；CLAUDE.md/AGENTS.md marker 內容本 change 不變
- Affected code:
  - New: `docs/configuration.md`、`docs/configuration.zh-TW.md`
  - Modified: `crates/speclink-core/src/config.rs`、`crates/speclink-core/src/init.rs`、`crates/speclink-core/src/skills.rs`、`crates/speclink-core/src/instructions.rs`、`crates/speclink-cli/src/commands.rs`、`README.md`
  - Removed: （無）
