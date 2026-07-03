## Why

目前 speclink 只支援規格文件與 git 儲存庫同倉的純本地模式。十六輪討論定案的團隊情境（PO/PM 在客製系統操作、RD 在本地 repo 實作，或 RD 全流程在本地而文件存於團隊系統）需要：規格與狀態的真相移到 server、code 與 git 留在本地，兩端以「領域動詞層」的 REST 契約（附 PAT 認證）接縫。本 change 交付這份動詞契約的規格與 CLI 端的 remote 薄 client——team 系統（第一個消費者為 wadpilot）以 SDK 內嵌引擎實作 server 端後即可對接。

目標使用者：情境 1 與情境 3 的 RD/QA（本地 Claude Code + remote store）、實作 server 端的團隊系統開發者（契約文件的讀者）、以及自情境 4 升級的既有使用者。

## What Changes

- **連接檔與模式解析**：新增 `.speclink.remote.yaml`（欄位：url 含專案範疇、repo 為本 repo 在專案內的註冊名，單 repo 專案可缺省）；檔案存在即 remote 模式、不存在即 fs 模式；與 openspec/ 目錄並存時 remote 勝出並輸出警告。
- **初始化與連接指令**：`speclink init --store remote --url <url> [--repo <name>]` 執行 workspace init（指令檔、技能——不建 openspec/ 樹）並寫入連接檔；新增 `speclink link <url> [--repo <name>]` 與 `speclink unlink` 建立／移除連接檔。
- **PAT 認證**：新增 `speclink auth login`（貼上 PAT，依 url 存於使用者層級設定目錄）與 `speclink auth status`；環境變數 SPECLINK_TOKEN 供 CI／headless 覆寫；憑證永不寫入 repo 內任何檔案。
- **動詞契約（規格文件化）**：定義 server 端 REST 契約——change 列舉與讀取、artifact 讀寫、discussion 動詞、claim／done／archive、instructions 計算、政策（workflow-config）side-car 端點、身分查驗；payload 欄位與既有 `--json` 對齊（camelCase）；409 一律附機器可判 reason；artifact 寫入採 version/If-Match 樂觀並行控制；API 版本以 header 協商。
- **remote 動詞路由**：remote 模式下，既有 CLI 指令改為呼叫契約端點（server 執行引擎），人眼與 `--json` 輸出形狀與 fs 模式一致；任何非 2xx 回應 SHALL 翻譯為語義化訊息與建議動作（401 → 提示 speclink auth login），絕不把裸狀態碼交給使用者或 agent 判讀；斷線／連不上即明確失敗，不做快取 fallback。
- **repo 身分驗證鏈**：每個動詞自動攜帶連接檔的 repo 名；server 驗證 repo 屬於專案、change 歸屬 repo 相符（claim 原子、搶佔回 409）；跑錯 repo 時 fail loud。change 歸屬規則為 **v1 一 change 一 repo**（建立時取自當前 repo、列舉依 repo 過濾、跨 repo 需求拆分為多個 change）。另提供 git remote 參考值的輔助警告（fork／鏡像偵測，僅警告、不影響指令結果）。
- **store 文件讀取動詞**：新增 `speclink artifact cat`（讀取 change 的 artifact 內容）與 `speclink language show`（讀取共用詞彙），兩模式皆可用；技能內容中殘留的直接讀檔指示改為使用這些動詞（單一來源技能因此在兩模式通用）。
- **指令區塊 remote 變體**：remote 模式下 init 生成的 CLAUDE.md/AGENTS.md marker 內容改用 remote 措辭（「規格與 change 存於團隊系統，一律使用 speclink 動詞，絕不本地讀寫規格檔」），渲染矩陣自此為（工具目標）×（fs｜remote）。
- 新增團隊模式雙語文件（連接、認證、動詞契約參考、repo 識別、情境升級指引），README 增列連結。

## Non-Goals

（範圍排除與被否決方案記錄於 design.md 的 Goals / Non-Goals 章節。）

## Capabilities

### New Capabilities

- `remote-connection`: 連接檔格式與模式解析、remote 初始化與 link/unlink、並存警告、repo 身分攜帶與 fail loud、git remote 參考值輔助警告、marker 區塊 remote 變體。
- `verb-contract`: REST 動詞集與 payload 形狀、政策 side-car 端點、樂觀並行控制、409 reason、API 版本協商、非 2xx 錯誤翻譯紅線、store 文件讀取動詞、change 的 repo 歸屬規則（一 change 一 repo）。
- `remote-auth`: PAT 登入與狀態查詢、憑證儲存位置、SPECLINK_TOKEN 覆寫、401 處理。

### Modified Capabilities

（無——remote 模式為新增行為面；fs 模式的既有需求不變。）

## Impact

- Affected specs: 新增 `remote-connection`、`verb-contract`、`remote-auth`
- Affected crates: speclink-remote（新 crate：HTTP client 與認證，core 維持無網路呼叫紅線）、speclink-core（模式解析、init remote 分支、marker 變體、技能資產動詞化）、speclink-cli（動詞路由、link/unlink/auth 子指令、artifact cat 與 language show）
- 相容性影響: fs 模式所有既有輸出不變（parity 維持）；新增子指令 link、unlink、auth login、auth status、artifact cat、language show；remote 模式為全新行為面、無既有基線
- 設定欄位: 新增 `.speclink.remote.yaml`（url、repo）；新增環境變數 SPECLINK_TOKEN、SPECLINK_STORE_URL（覆寫連接 url）
- 技能/marker 影響: marker 區塊新增 remote 內容變體；全部技能資產中的直接讀檔指示（如讀取 openspec/LANGUAGE.md、直接開 artifact 檔）改為 speclink 動詞——claude/codex fs 模式的生成內容因此刻意更新
- Affected code:
  - New: `crates/speclink-remote/Cargo.toml`、`crates/speclink-remote/src/lib.rs`、`crates/speclink-remote/src/client.rs`、`crates/speclink-remote/src/auth.rs`、`docs/team-mode.md`、`docs/team-mode.zh-TW.md`、`docs/verb-contract.md`、`docs/verb-contract.zh-TW.md`
  - Modified: `Cargo.toml`（workspace members）、`crates/speclink-core/src/workspace.rs`（模式解析）、`crates/speclink-core/src/init.rs`（remote 分支與 marker 變體）、`crates/speclink-core/src/instructions.rs`、`crates/speclink-core/src/skills.rs` 與 `crates/speclink-core/assets/skills/` 全部技能資產（動詞化）、`crates/speclink-cli/src/main.rs`、`crates/speclink-cli/src/commands.rs`、`README.md`
  - Removed: （無）
