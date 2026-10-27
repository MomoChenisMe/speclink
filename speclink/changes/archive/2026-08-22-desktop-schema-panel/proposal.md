## Why

引擎面收斂完成後（change schema-engine-openspec-parity，已封存），desktop 對 schema 的消費仍停在間接層：設定頁只透過產出規則分節的固定鍵（schemaArtifacts）看得到 artifact id 清單，使用者看不到專案用哪個 schema、有哪些可選、內容長什麼樣，也沒有切換或客製的入口。remote 模式另有一個既有怪癖：設定快照以 client 本機的 user 層目錄解析 server 專案的 schema 名稱，同名時產出規則分節顯示錯誤來源的固定鍵。本 change 依討論 schema-engine-openspec-parity 的結論（封存於 openspec/discussions/archive/2026-08-19-schema-engine-openspec-parity.md）補齊 desktop 面。

## What Changes

- 設定頁新增「產出流程」獨立頁籤（使用者可見文案的詞彙裁定見下；2026-08-21 使用者驗收裁定：自成頁籤，不塞在 config.yaml 簽內）：
  - **檢視**：清單列出每個可解析的 schema（名稱、來源層級、artifact 圖）；點入唯讀詳情，含每個 artifact 的 description、instruction、template 全文
  - **切換**：下拉選擇專案 schema，寫入 config.yaml 的 schema 鍵——複用引擎的 byte-preserving setter（set_workflow_schema_text），local 直寫、remote 走既有 revision 守門的 config 寫入通道
  - **客製**：fork 按鈕（僅 local 模式）——呼叫引擎既有的 fork，把選中的 schema 複製到專案 openspec/schemas/，成功後清單即時反映
  - **建立**：建立表單（僅 local 模式；2026-08-21 使用者裁定新增）——收 kebab-case 名稱，呼叫引擎既有的 init_schema 產專案層骨架（schema.yaml＋templates/，artifact 佈局用引擎預設），成功後清單即時反映；內容編輯仍交外部編輯器（非編輯器 Non-Goal 不變）
  - **編輯入口**：有磁碟路徑的項目（專案層／user 層）提供「開啟所在資料夾」按鈕（僅 local 模式；2026-08-22 使用者驗收裁定——建立後沒有任何編輯去路是死路，補跳板但不內建編輯器）——經 tauri opener 在檔案管理器顯示 schema 目錄
  - **刪除**：專案層項目提供刪除動作（僅 local 模式；2026-08-22 使用者驗收裁定新增）——經確認對話框後移除 openspec/schemas/<name>/；使用中的 schema 拒刪顯性失敗；內建與 user 層不提供（內建無檔案、user 層跨專案共用）
- 頁籤標籤用「Schema」（2026-08-22 使用者裁定：與 config.yaml、.speclink.yaml 同列的原生詞一致性——頁籤列全是技術 token，唯一中文標籤反而突兀）；籤內文案維持「產出流程」（與 config.yaml 簽內人話卡對稱），LANGUAGE.md 詞條補明文例外
- remote 限縮：清單只列內建 schema；config 的 schema 名稱非內建時顯性呈現「遠端自訂尚不支援」而非猜測；同時修掉上述 user 層誤解析怪癖（remote 快照的 schema 解析不再讀 client 本機 user 目錄）
- 詞彙：使用者可見文案以「產出流程」承載 schema 概念（經使用者裁定用譯詞，與既有「產出規則」「產出政策」同族）；openspec/LANGUAGE.md 新增詞條，schema 僅留於技術 token（鍵名、CLI 指令、識別符）
- 引擎零改動：檢視、切換、fork 全部消費 change schema-engine-openspec-parity 已落地的引擎函式

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-config`: 設定頁新增產出流程頁籤（檢視／切換／fork／建立）與 remote 模式的內建限縮及誤解析修正

## Impact

- Affected specs: desktop-config（delta：ADDED requirements）
- Affected code:
  - Modified: apps/desktop/core/src/settings.rs（schema 快照組裝、remote 怪癖修正）、apps/desktop/src-tauri/src/lib.rs（IPC 查詢與寫入指令）、apps/desktop/src-tauri/src/remote.rs（remote 切換通道）、apps/desktop/src/adapter/workspace.ts 與 apps/desktop/src/session.ts（adapter 面與 provider 介面）、apps/desktop/src/i18n/messages.ts（文案）、apps/desktop/src/views/ProjectSettingsView.tsx（Schema 頁籤 UI）、apps/desktop/src/__tests__/projectSettingsView.test.tsx（view 測試）、openspec/LANGUAGE.md（新詞條）
  - New: (視 design 決定是否抽節元件；預設同檔內新節，無新檔)
  - Removed: (none)
- 零 server 改動：remote 的內建 schema 由 desktop core 本地解析（desktop 連結 speclink-core，內建定義在 binary 內）
