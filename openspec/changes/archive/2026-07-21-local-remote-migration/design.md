## Context

store 契約已定 Bundle{format_version, scope, project_revision, documents}、ImportMode（CreateNew｜Overwrite）、ImportReport；TeamStore 的 import 為單一原子操作，CreateNew 的「目標 scope 必須為空」由 conformance gate 與 import-createnew-gate 刀在四個 driver 釘死。server 只有離線 Backup／VerifyBackup／Restore 子命令，無任何 HTTP import 面。desktop 端：chooser 的並存衝突對話「使用 remote」停用附「待正式遷移功能」；引擎 core Store 對本地 workspace 有完整讀取面（changes、canonical specs、討論 live/archived、archived changes、config、LANGUAGE.md）；remote marker 寫入（write_remote_section）與 checkoutRoot 開啟路徑皆為既有零件。

## Goals / Non-Goals

**Goals:**

- 一鍵正式遷移：本地 workspace 全集原子上傳空 scope，資料夾轉為 checkout，分頁原地轉 remote。
- 失敗零副作用：任何失敗（scope 非空、網路、驗證）本地分毫不動、server 無部分寫入。
- 本地內容一律備份保留——無刪除路徑。

**Non-Goals:**

- 不做 remote→local 反向遷移（需求出現再立刀）；不做非空 scope 的合併遷移（CreateNew 專用——Overwrite 屬維運 Restore，不上 wire）；不做遷移中斷續傳（Bundle 單請求原子、失敗即重來）；不做多 repo 批次遷移；不動既有離線備份工具。

## Decisions

### 決策 1：import 端點固定 CreateNew、不上 wire 暴露 Overwrite

POST /import 走既有 Binding 認證（scope 由 header 綁定）、reader 403（沿 policy_write capability 的 role 檢查）；body＝wire Bundle；模式硬編碼 CreateNew——遷移是「搬進空房」語意，Overwrite 上 wire 等於給所有 client 一把整 scope 覆寫槍，被否；維運覆寫需求由既有離線 Restore 工具承擔。目標 scope 非空時 store 拒絕原樣上拋為 wire conflict 類錯誤（訊息保留 store 的 create-new 語意）；成功回 ImportReport 的 wire 形（projectRevision、逐文件 outcome）。整包單一 UoW——部分寫入不存在，與 conformance 的原子斷言同源。

### 決策 2：Bundle 組裝在 desktop Rust、以 core Store 讀取面為準

src-tauri 對本地 root 以引擎 core Store 逐類讀取組 Bundle：list_changes（meta＋全 artifacts）、list_canonical_capabilities（spec 內文）、討論 live 與 archived、list_archived_changes（meta＋全文件）、workflow config、LANGUAGE.md——DocumentId 全集與 store 契約一一對應；format_version 沿契約常數。組裝為純函式（root 進、Bundle 出），單元可測。壞檔（meta 解析失敗等）即中止並點名檔案——fail-closed，不做部分遷移。

### 決策 3：遷移流程與本地轉換順序

流程固定：選擇 connection 與 scope（重用 chooser scopes 步驟；UI 提示僅可遷入空 scope）→ 破壞性確認（指出目標 Project/Repo 與「本地 openspec/ 將改名備份」）→ 組 Bundle → POST /import → **成功後才動本地**：openspec/ 改名 openspec.migrated-{YYYY-MM-DD}/（既存同名備份則加序號）→ write_remote_section 寫 marker → 原地開 remote 分頁（checkoutRoot＝該資料夾）、原 local 分頁條目轉為該 remote 分頁。順序保證失敗零副作用：import 失敗時本地未動；改名失敗（極端）時 server 已有內容但本地完整——錯誤訊息指引手動改名，marker 未寫、不會產生並存假象。

### 決策 4：兩個入口同一流程

並存衝突對話的「使用 remote」啟用後進遷移流程（預填 marker 指向的 connection/scope 作為建議目標——但 CreateNew 要求空 scope，marker 指向的 scope 通常已有內容，此情境實為「本地與 server 都有內容」的合併需求，不在本刀——對話中「使用 remote」改為兩支：「以 server 內容開啟（本地 openspec/ 改名備份後轉 checkout，不上傳）」與「遷移本地內容至其他空 scope」）。chooser 本機路徑開到含 openspec/ 專案時提供「遷移到 server…」次要動作進同一流程。兩入口共用 MigrationDialog。

### 決策 5：並存對話的第三選項語意

決策 4 揭示並存情境的真實三選：繼續本地（既有）、以 server 為準（本地改名備份、轉 checkout——不上傳、不覆蓋 server）、遷移本地至空 scope（完整遷移流程）。三者皆無靜默覆蓋；「以 server 為準」是備份後棄用本地、非合併——對話文案明說。規格修訂據此改寫並存需求。

## Implementation Contract

- server 整合測試（crates/speclink-server/tests/import_api.rs）：空 scope 匯入成功且逐文件 Created、匯入後清單/文件/討論/archived 端點回真值；非空 scope 拒絕且無任何寫入；reader 403；壞 Bundle（未知 format_version、缺欄位）拒收；原子性——大 Bundle 匯入中斷模擬不留部分狀態（沿 conformance 模式）。
- desktop：Bundle 組裝純函式單元測試（played 本地 workspace 夾具→文件全集斷言、壞 meta 中止點名）；MigrationDialog vitest（scopes 選擇、確認文案含目標與備份說明、成功轉分頁、失敗原樣呈現本地不動）；並存對話三選項 vitest。
- GUI 鐵律手動（remote-dev-harness；操作前確認使用者未在使用螢幕）：npm run dev → 以本 repo 的測試副本走完整遷移至空 scope → 資料夾出現 openspec.migrated-*/ 與 marker、分頁原地轉 remote、看板內容完整 → CLI 於該資料夾以 remote 模式 speclink list 同形 → 對非空 scope 重試遷移被拒且本地不動 → 並存資料夾三選項各走一次。
- 回歸：cargo test --workspace、npm test -w apps/desktop、cargo build --release -p speclink-desktop 全綠。
