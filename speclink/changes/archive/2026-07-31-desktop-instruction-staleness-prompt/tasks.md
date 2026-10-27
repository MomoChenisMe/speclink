## 1. 前置閘門

- [x] 1.1 確認平行變更 discuss-propose-from-docs 已完成並封存（speclink list 不再列其為進行中），其內嵌資產修改已合入 main——本變更的 MARKER_VERSION bump 與 golden 再生必須疊在其資產之上，避免同四份 golden 互踩。驗證：speclink list --json 無該變更、git log 可見其提交。 <!-- speclink-task:tsk_01KYTYJZ9R016REZX2MMBDRMMD -->

## 2. 產物層版本戳同源（決策 1）

- [x] 2.1 撰寫版本戳同源測試：三種 render 路徑（claude、codex、custom descriptor）生成的技能檔 frontmatter 版本欄位值 SHALL 等於 MARKER_VERSION、不再是固定 "1.0"。位置 crates/speclink-core/src/skills.rs 的 #[cfg(test)]。驗證：cargo test -p speclink-core 新測試紅燈（現況仍為 "1.0"）。 <!-- speclink-task:tsk_01KYTYJZ9RJH93Y2XCDVHFCQ0P -->
- [x] 2.2 實作 frontmatter 版本戳同源：crates/speclink-core/src/skills.rs 的技能 frontmatter 生成改寫入 MARKER_VERSION；crates/speclink-core/src/init.rs 遞增 MARKER_VERSION（本變更自身的 render 內容變動）。落實決策 1：版本戳同源與字串相等比對，及規格「產物層版本戳同源」。驗證：2.1 測試綠燈。 <!-- speclink-task:tsk_01KYTYJZ9R1FRA29X1RMRCT819 -->
- [x] 2.3 於乾淨樹同批再生輸出基線：UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生四份 snapshot（crates/speclink-core/tests/golden/），並執行 speclink update 再生 .claude/skills/ 與 .agents/skills/ 技能實例。驗證：diff 僅含版本欄位與 marker 版號變動；cargo test -p speclink-core --test render_golden 綠燈（對應規格「中性渲染目標」的 golden 基線場景）。 <!-- speclink-task:tsk_01KYTYJZ9RJ8Q5WG3EZ5PYH4G8 -->

## 3. 內嵌資產版本鎖定紀律（決策 8）

- [x] 3.1 撰寫鎖定測試：於 crates/speclink-core/tests/render_golden.rs 新增 version–hash 鎖定測試——自寫 FNV-1a 指紋函式、讀取 crates/speclink-core/tests/golden/assets.lock（版號＋指紋兩欄）、判定「指紋不符且版號未變即失敗」，失敗訊息含修復步驟（遞增 MARKER_VERSION 後 UPDATE_ASSETS_LOCK=1 重生）；重生路徑含防呆（指紋變而版號未變時拒絕改寫並失敗）。落實決策 8：version–hash 鎖定測試與重生防呆。驗證：無 lock 檔時測試以指引訊息失敗（紅燈確認訊息內容）。 <!-- speclink-task:tsk_01KYTYJZ9R8P2398EP5YS29ECS -->
- [x] 3.2 首次生成鎖定檔：乾淨樹上 UPDATE_ASSETS_LOCK=1 生成 crates/speclink-core/tests/golden/assets.lock。驗證三情境：cargo test -p speclink-core 全綠；手動改動任一 assets 檔一字元後測試紅燈且訊息含修復步驟、還原後復綠；僅遞增版號不改內容時綠燈（對應規格「內嵌資產版本鎖定紀律」三場景）。 <!-- speclink-task:tsk_01KYTYJZ9RK4Y4W2PEZGJCD13X -->
- [x] 3.3 ~~CLAUDE.md 開發備忘「內嵌技能三處必須同步」條目補一句~~ **取消（2026-07-31 使用者裁定）**：該備忘已於 commit c0303d8 被刻意移除，CLAUDE.md 現僅剩引擎生成的 SPECLINK 受管區塊（手改會被 update 覆蓋）。紀律落點改為鎖定測試自身——其失敗訊息已完整寫明「bump MARKER_VERSION 後以 UPDATE_ASSETS_LOCK=1 於乾淨樹重生」，是機械強制且對所有貢獻者一視同仁（與提案 Non-Goal「不以 harness hooks 強制 bump 紀律」同理）。 <!-- speclink-task:tsk_01KYTYJZ9R7ZQVMS9H0MVKVZ5C -->

## 4. 指令檔過期探測（引擎，決策 2、3）

- [x] 4.1 撰寫探測單元測試：crates/speclink-core/src/init.rs 的 #[cfg(test)] 覆蓋規格「指令檔過期探測」全部六場景——舊版 marker 判過期並列差異檔、現版工作區不過期、標記移除視為退出受管、指令檔不存在判缺失（含「一工具現版、另一工具檔案不存在 → 缺失優先於過期」與「缺失不與退出受管、無法判定混同」）、.speclink.yaml 損壞回報無法判定、CRLF 換行差異不誤報。驗證：cargo test -p speclink-core 紅燈。 <!-- speclink-task:tsk_01KYTYJZ9RXCFJQ2KHDV6BKCDP -->
- [x] 4.2 實作唯讀探測函式於 crates/speclink-core/src/init.rs：依 tools 清單讀各工具 instruction 檔——檔案不存在判缺失、存在則讀 marker 版號字串相等比對——四態回報（缺失／過期／現版／無法判定，缺失優先於過期）＋differingFiles 清單（render 期望內容對磁碟內容、不存在檔視為空內容必列入、換行正規化）、零寫入。落實決策 2：marker 權威判定與退出語意，及決策 3：探測回報形狀。驗證：4.1 測試綠燈。 <!-- speclink-task:tsk_01KYTYJZ9R0DQ62RW97TB034QW -->

## 5. desktop 搭載（決策 4、5、6、7，規格「指令檔過期提示」）

- [x] 5.1 撰寫 desktop-core 包裝測試：apps/desktop/core 對探測回報的序列化欄位為 camelCase（currentVersion、stale、missing、differingFiles 等）與 update 委派回報形狀。驗證：cargo test -p speclink-desktop-core 紅燈後隨 5.2 轉綠。 <!-- speclink-task:tsk_01KYTYJZ9R0BD7NR2TEYHS0NZB -->
- [x] 5.2 實作 apps/desktop/core/src/project.rs 探測與更新包裝，apps/desktop/src-tauri/src/lib.rs 新增兩個單行委派 command（唯讀探測、呼叫既有 update() 的更新）。落實決策 4：獨立唯讀 command 搭載，及決策 5：更新動作復用既有再生入口。驗證：cargo test -p speclink-desktop-core 綠燈；cargo build --release -p speclink-desktop 通過。 <!-- speclink-task:tsk_01KYTYJZ9RT0VS272BHX0DWDBE -->
- [x] 5.3 撰寫前端顯示裁決測試（vitest，apps/desktop/src/store.ts）：過期＋未略過→提示（更新文案）；缺失＋未略過→提示（安裝文案）；過期或缺失＋已略過同版→不提示；已略過舊版＋版號變動→重新提示；無法判定→不提示且不記入略過；remote 分頁→不執行探測。驗證：npm test -w apps/desktop 紅燈。 <!-- speclink-task:tsk_01KYTYJZ9R8MGAH1EJ7AZS0BNA -->
- [x] 5.4 實作前端狀態與略過記憶：apps/desktop/src/store.ts 新增探測狀態、本地專案分頁活躍時呼叫探測、更新完成與 workspace-changed 後重查、「保留現狀」寫入 localStorage（專案路徑 → 已略過版號）。落實決策 6：略過記憶存前端本地持久化，及規格「指令檔過期提示」的顯示裁決。驗證：5.3 測試綠燈（含外部 speclink update 後提示自然消失的場景）。 <!-- speclink-task:tsk_01KYTYJZ9RQE1X785D3DCEC7KN -->
- [x] 5.5 撰寫提示元件測試（vitest）：apps/desktop/src/components/InstructionUpdatePrompt.tsx 呈現差異檔數、主動作依探測態分文案（過期→「更新」、缺失→「安裝」）＋「保留現狀」、更新失敗錯誤於原位呈現且可重試、非阻斷（不遮蔽分頁內容）。驗證：npm test -w apps/desktop 紅燈。 <!-- speclink-task:tsk_01KYTYJZ9RKXQCYBSMAX1Z8SZT -->
- [x] 5.6 實作提示元件與文案：InstructionUpdatePrompt 橫幅（UpdateBanner 同構視覺語彙）掛載於過期或缺失專案分頁內容頂部，主動作依態分文案（更新／安裝）、共用同一再生入口；apps/desktop/src/i18n/messages.ts 新增 zh-TW 與 en 文案鍵（含安裝文案，遵循 openspec/LANGUAGE.md，不出現工程詞，兩語系鍵集合維持相等）。落實決策 7：提示形態為分頁內非阻斷橫幅。驗證：5.5 測試綠燈；文案對照 LANGUAGE.md 審視。 <!-- speclink-task:tsk_01KYTYJZ9RQRVK7YN9V9DS2YKM -->

## 6. 側欄版號刪除（決策 9）

- [x] 6.1 刪除 apps/desktop/src/App.tsx 側欄底部版號的條件渲染（設定沉底列之後的版本文字），currentVersion 保留供設定頁軟體更新卡。落實決策 9：側欄版號刪除，及規格「側欄導覽結構」的側欄無常駐版號場景。驗證：npm test -w apps/desktop 綠燈（含既有側欄結構測試調整）；規格「側欄無常駐版號」場景——側欄任何位置無版號文字、設定頁軟體更新卡仍顯示目前版本。 <!-- speclink-task:tsk_01KYTYJZ9RNPAXZE1PS6G0WP2X -->

## 7. 收尾驗證

- [x] 7.1 全套測試：cargo test（workspace 全量）、npm test -w apps/desktop、npm test -w packages/ui。驗證：全綠；render golden 與鎖定測試均通過。 <!-- speclink-task:tsk_01KYTYJZ9RVGK1XE66545A5F4B -->
- [x] 7.2 真實視窗 GUI 驗證（依 CLAUDE.md 備忘流程，操作前確認使用者未使用螢幕）：以新 build 開啟一個指令檔為舊版的本地專案→提示出現且顯示檔案數→點「更新」後受管檔為現版、提示消失；開啟一個 tools 宣告但指令檔不存在的專案→提示以安裝語意呈現→點「安裝」後受管檔生成、提示消失；另一專案點「保留現狀」→重開分頁不再提示；側欄底部無版號。驗證：截圖檢視四個狀態。 <!-- speclink-task:tsk_01KYTYJZ9RC1NPGWC45K9T0CGK -->
- [x] 7.3 speclink validate desktop-instruction-staleness-prompt 通過。驗證：無 Critical 與 Warning。 <!-- speclink-task:tsk_01KYTYJZ9RYKQV05X0N8EBH9GD -->
