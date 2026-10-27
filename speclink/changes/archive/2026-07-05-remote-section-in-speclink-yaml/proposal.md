## Why

討論 sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios 的 Round 11 定案為兩檔佈局：`.speclink.yaml`（workspace 設定）＋ `.speclink.remote.yaml`（remote 專屬連接檔，存在即模式訊號）。維護者重審後認定：政策鍵已遷居 openspec/config.yaml 之後，`.speclink.yaml` 僅剩 tools 一鍵，為連接設定另立一檔的代價（兩個設定檔、兩條檔案發現路徑、使用者心智負擔）高於其收益（存在即訊號、可單獨 gitignore）。本 change 將 remote 連接設定回歸為 `.speclink.yaml` 內的區段，廢除獨立連接檔——推翻 Round 11 的分檔決定，回歸 Round 8 的單檔方向（但不重新開放政策鍵）。

目標使用者：情境 1 與情境 3 的 RD/QA（本地工作區連接團隊 store，對應 init／link 與所有 remote 動詞的執行者），以及維護 `.speclink.yaml` 的專案管理者。

**前置條件：本 change 必須待 verb-contract-and-remote-client 完成歸檔後才可實作**——它修改的正典需求（remote-connection capability）由該 change 的歸檔建立，實作對象也是該 change 交付的程式碼。

## What Changes

- `.speclink.yaml` 新增 `remote:` 區段：`url`（連線目標，含專案範疇；可缺省改由環境變數 SPECLINK_STORE_URL 供給）、`repo`（選填，本 repo 在專案內的註冊名）。
- 模式訊號從「`.speclink.remote.yaml` 檔案存在與否」改為「`.speclink.yaml` 的 `remote:` 區段存在與否」：有區段即 remote 模式，無區段即 fs 模式；區段存在但 url 與環境變數皆缺時明確失敗並提示。
- speclink init --store remote、speclink link、speclink unlink 的檔案效果改變：寫入／更新／移除 `.speclink.yaml` 的 `remote:` 區段（保留檔內其他欄位如 tools），不再建立或刪除獨立連接檔。
- **BREAKING**：`.speclink.remote.yaml` 廢除——CLI 不再解析該檔；偵測到殘留檔案時於 stderr 輸出一行遷移警告（指引把 url／repo 搬進 `.speclink.yaml` 的 `remote:` 區段），殘留檔不影響模式判定。
- 行為語意不變的部分：remote 與 openspec/ 並存警告、SPECLINK_STORE_URL 覆寫、repo 身分攜帶與歸屬防呆、git remote 參考值輔助警告、marker remote 變體——僅其讀取來源從連接檔改為 `remote:` 區段。
- 更新團隊模式雙語文件中關於連接設定的段落。

## Non-Goals

- 不改動動詞契約（verb-contract）與認證（remote-auth）的任何需求——本 change 只換連接設定的載體。
- 不提供殘留 `.speclink.remote.yaml` 的自動遷移指令——一次性手動搬移兩個欄位，警告訊息給出指引即可。
- 不改變 fs 模式的任何行為與輸出（回歸對照不受影響）。
- 不把政策鍵（locale／spec_locale／tdd／audit）搬回 `.speclink.yaml`——政策的家仍是 store 側 WorkflowConfig。
- 敏感 url 的處理不另設機制——沿用既有的 SPECLINK_STORE_URL 環境變數（committed 檔可缺省 url）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `remote-connection`: 連接設定載體從獨立連接檔改為 `.speclink.yaml` 的 `remote:` 區段——模式解析、init --store remote／link／unlink 的檔案效果、repo 名的讀取來源隨之修改，並新增殘留舊檔的遷移警告。

## Impact

- Affected specs: `remote-connection`（修改；其正典規格由 verb-contract-and-remote-client 歸檔時建立，本 change 的 delta 以該版本為基準）
- Affected crates: speclink-core（模式解析、workspace 設定結構、init 的 remote 分支）、speclink-cli（link／unlink／init 的檔案效果、遷移警告輸出）
- CLI 指令: speclink init --store remote、speclink link、speclink unlink 三者的檔案系統效果改變（讀寫 `.speclink.yaml` 而非獨立檔）；子指令、旗標、exit code 語意均不變；新增殘留舊檔偵測的 stderr 單行警告
- 相容性影響: **BREAKING**——`.speclink.remote.yaml`（verb-contract-and-remote-client 交付）自本 change 起不再被解析，殘留檔觸發警告；fs 模式的人眼與 `--json` 輸出完全不變（回歸對照不受影響）；remote 模式指令的輸出形狀不變，僅連接設定來源改變
- 設定欄位: `.speclink.yaml` 新增 `remote.url`（無預設；remote 模式下必要，可由 SPECLINK_STORE_URL 供給或覆寫）與 `remote.repo`（選填，無預設）；`.speclink.remote.yaml` 的 url／repo 欄位廢止
- 技能/marker 影響: 無——marker 的 remote 內容變體與各技能不變，僅觸發 remote 變體的模式訊號來源改變（claude／codex 兩者的生成結果不受影響）
- Affected code:
  - New: crates/speclink-cli/tests/remote_section.rs（整合測試）
  - Modified: crates/speclink-core/src/workspace.rs、crates/speclink-core/src/config.rs、crates/speclink-core/src/init.rs、crates/speclink-cli/src/main.rs、crates/speclink-cli/src/commands.rs、docs/team-mode.md、docs/team-mode.zh-TW.md
  - Removed: （無——被廢除的是使用者專案內的設定檔，非本 repo 原始碼）
