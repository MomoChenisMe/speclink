## 1. 引擎 adopt 入口（決策 2）

- [x] 1.1 撰寫 adopt 單元測試：crates/speclink-core/src/init.rs 的 #[cfg(test)] 覆蓋規格「工作區補齊入口」四場景——補齊工作區檔且既有 openspec/ 文件與自訂 config.yaml 位元級不變、骨架缺件補齊（specs/ 與 changes/archive/ 與範本 config.yaml）、相同 tools 重複執行冪等、tools 空清單回錯誤且零寫入。驗證：cargo test -p speclink-core 紅燈。 <!-- speclink-task:tsk_01KYV1DZTWSXNW6SN196RG3JYB -->
- [x] 1.2 實作 pub fn adopt(root, tools)：組合骨架冪等補件（create_dir_all＋config.yaml 僅缺席時寫範本）與 reconcile_builtin_tools（寫 tools 進 .speclink.yaml＋整套再生受管檔），不經 init 的「Already initialized」擋板；spec_dir 固定 openspec。落實決策 2：引擎 adopt 入口＝store_init 冪等補件＋reconcile_builtin_tools 組合。驗證：1.1 測試綠燈。 <!-- speclink-task:tsk_01KYV1DZTWN1F1CWPWJ8Z718FA -->

- [x] 1.3 撰寫 adopt 的 .gitignore 單元測試（crates/speclink-core/src/init.rs 的 #[cfg(test)]），覆蓋規格場景「工作資料夾納入版控忽略」：.gitignore 缺席時 adopt 後該檔存在且含 `.speclink/`；既有 .gitignore 內容為 `node_modules/\ndist/\n` 時（規格 Example 逐值）兩行原樣保留並多出 `.speclink/`；已含 `.speclink/` 時重跑 adopt 不重複追加（檔案位元級不變）。驗證：cargo test -p speclink-core 紅燈。 <!-- speclink-task:tsk_01KYVJMQCGFMJKV8K2Y4ST5WVF -->
- [x] 1.4 於 pub fn adopt 補呼叫 ensure_gitignore(root.join(".gitignore"))——reconcile_builtin_tools 走的 update 路徑不含這步，僅 init 的 workspace_init 有。驗證：1.3 測試綠燈。 <!-- speclink-task:tsk_01KYVJQAY9Z9SY86JGF4KCAEYQ -->

## 2. desktop 探測第四態（決策 1）

- [x] 2.1 撰寫 desktop-core 探測測試：apps/desktop/core/src/project.rs 的 #[cfg(test)]——「openspec/ 在、.speclink.yaml 不在」回報 unadopted（root 為向上命中的專案根、含子目錄開啟情境）且零寫入；「.speclink.yaml 在」仍回報 project；完全未命中仍 uninitialized；壞 .speclink.yaml 仍 fail-closed 單行 Err。驗證：cargo test -p speclink-desktop-core 紅燈。 <!-- speclink-task:tsk_01KYV1DZTWCS9SC0SP3NKPQ92X -->
- [x] 2.2 實作 ProjectProbe::Unadopted { root } 變體（serde camelCase、status: "unadopted"）與 open_project_at 判定：discover 命中、resolve 為 StoreMode::Fs、root 無 .speclink.yaml → Unadopted；並新增 adopt 包裝函式（呼叫 core adopt 後重跑探測回報 Project）。落實決策 1：判準＝.speclink.yaml 存在與否，第四態只加在 desktop 探測層。驗證：2.1 測試綠燈。 <!-- speclink-task:tsk_01KYV1DZTWMETR61GNRG2VKEG4 -->
- [x] 2.3 src-tauri 新增 adopt_project(path, tools) command（單行委派 desktop-core adopt 包裝）並註冊。落實決策 4：IPC 為獨立 adopt command。驗證：cargo build --release -p speclink-desktop 通過（重建前先關閉執行中的 exe）。 <!-- speclink-task:tsk_01KYV1DZTW5M7JG6KPE90CNTZQ -->

## 3. 前端啟用確認框（決策 3）

- [x] 3.1 撰寫前端分流與確認框測試（vitest）：apps/desktop/src/__tests__/workspace.test.ts 與 App.test.tsx，覆蓋規格「未啟用資料夾經確認後補齊啟用」的前端場景——probe 回報 unadopted → pendingAdopt 設定且開啟啟用對話框（工具多選、預設勾選 claude）；confirmAdopt 成功 → 以回報 root 切入專案、狀態清空；cancelAdopt → 狀態清空、無任何 IPC 寫入呼叫；confirmAdopt 失敗 → 單行錯誤浮出、不切換專案；project／uninitialized 分流行為凍結（既有測試不變）。驗證：npm test -w apps/desktop 紅燈。 <!-- speclink-task:tsk_01KYV1DZTWBDED4KEADFZQPY3H -->
- [x] 3.2 實作前端：apps/desktop/src/adapter/workspace.ts probe 型別聯集加 unadopted 與 adopt 呼叫；store.ts 新增 pendingAdopt／confirmAdopt／cancelAdopt（openProjectAt 分流新增 unadopted 分支）；App.tsx 新增啟用確認對話框（與初始化確認框同型、獨立狀態）。落實決策 3：啟用確認框沿用初始化確認框同型。驗證：3.1 測試綠燈。 <!-- speclink-task:tsk_01KYV1DZTW5GEY45F15KBDQ2P4 -->
- [x] 3.3 i18n 文案：apps/desktop/src/i18n/messages.ts 新增啟用對話框鍵（標題、說明、主動作「啟用」、取消），zh-TW 與 en 鍵集合維持相等；文案遵循 openspec/LANGUAGE.md（不出現工程詞與設定檔檔名）。驗證：npm test -w apps/desktop 綠燈（含既有 messages 鍵集合測試）；文案對照 LANGUAGE.md 審視。 <!-- speclink-task:tsk_01KYV1DZTWY6DRAXJP5JMVKTKG -->

## 4. 收尾驗證

- [x] 4.1 全套測試：cargo test（workspace 全量）、npm test -w apps/desktop、npm test -w packages/ui。驗證：全綠。 <!-- speclink-task:tsk_01KYV1DZTWRKK3933X10KM7YY1 -->
- [x] 4.2 真實視窗 GUI 驗證（依 CLAUDE.md 備忘流程，操作前確認使用者未使用螢幕；路徑以剪貼簿貼上）：準備一個含 openspec/ 規格文件但無 .speclink.yaml 的資料夾→開啟→啟用對話框出現→確認後看板呈現既有內容、專案根有 .speclink.yaml 與 .claude/skills/；另備一資料夾走取消→檢查零寫入；再開一個正常專案確認無對話框。驗證：截圖檢視三個狀態。 <!-- speclink-task:tsk_01KYV1DZTWCX3ZEJV2W53B39S4 -->
- [x] 4.3 speclink validate desktop-enable-speclink-prompt 通過。驗證：無 Critical 與 Warning。 <!-- speclink-task:tsk_01KYV1DZTW2V7F57KJHJTPQAZD -->
