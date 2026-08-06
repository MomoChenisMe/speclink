## 1. 測試先行——引擎方向判定

- [ ] 1.1 [測試先行] speclink-core「指令檔過期探測」方向測試紅燈：新增案例——引擎 v1.11.0 對工作區 v1.14.0 判 "newer"（spec Example 值）、claude 較新且 codex 缺失時整體判 "newer"（較新優先於缺失）、無法解析的版號字串退回相等判定（判過期、絕不判較新）、既有現版／過期／缺失／退出受管案例維持不破。驗證：cargo test -p speclink-core 新案例紅燈、既有案例綠。 <!-- speclink-task:tsk_01KZ99P1E6HJ3GWKPP543VS2A3 -->
- [ ] 1.2 實作「指令檔過期探測」的方向判定（crates/speclink-core/src/init.rs）：InstructionStatus 加 Newer 變體（序列化 "newer"）、版號比較去 v 前綴以點拆段補零逐段數值比較、聚合優先序 Newer > Missing > Stale > Current、ToolInstructionState 加 newer 布林（camelCase）、Newer 時照常回報差異檔清單。驗證：1.1 全數轉綠。 <!-- speclink-task:tsk_01KZ99P1E6PEFADBYFHMWQQ78N -->

## 2. CLI 出口——update 守門與版號查詢面

- [ ] 2.1 [測試先行]「update 動詞的降級守門」測試紅燈（新檔 crates/speclink-cli/tests/update_downgrade_guard.rs）：較新工作區執行 update 被拒（stderr 單行含工作區與引擎兩版號、exit code 非零、工作區零檔案變動）、--allow-downgrade 越過後受管檔再生為引擎現版、過期工作區不帶旗標照常更新成功。驗證：cargo test -p speclink-cli 新案例紅燈。 <!-- speclink-task:tsk_01KZ99P1E60RK124RHDDTN7RZT -->
- [ ] 2.2 實作「update 動詞的降級守門」（crates/speclink-cli/src/main.rs）：UpdateArgs 加 --allow-downgrade 旗標，update 於任何寫入前執行探測、整體 "newer" 且未帶旗標即拒絕（單行英文 stderr、非零 exit、零寫入）。驗證：2.1 轉綠。 <!-- speclink-task:tsk_01KZ99P1E6PREAB8PHEXMYKSSN -->
- [ ] 2.3 「引擎版號查詢面」——--version 加引擎版號：先掃 crates/speclink-cli/tests/ 是否有釘住現行 --version 格式的測試（有則同批更新並列為刻意變更）；新增測試斷言輸出含 engine 與產物層版號、格式 <套件版號> (<架構>, engine <產物層版號>)；實作為執行期組字串（LazyLock，不引新依賴）。驗證：cargo test -p speclink-cli 全綠。 <!-- speclink-task:tsk_01KZ99P1E650EJHB3671TQJ47A -->

## 3. desktop 出口——較新提示

- [ ] 3.1 [測試先行]「指令檔過期提示」較新形態的前端裁決與呈現測試紅燈：instructionPrompt.ts 對 status "newer" 回 kind "newer"（略過記憶同鍵值、同版略過後不提示）；InstructionUpdatePrompt 於 kind "newer" 呈現「app 本體是舊版」語意文案、無「更新」「安裝」動作、僅「保留現狀」。驗證：npm test -w apps/desktop 新案例紅燈。 <!-- speclink-task:tsk_01KZ99P1E69EJYMFVESYXD6PQJ -->
- [ ] 3.2 實作「指令檔過期提示」的較新形態：instructionPrompt.ts 加 "newer" kind、apps/desktop/src/i18n/messages.ts 加標題與描述鍵（遵循 LANGUAGE.md、不出現工程詞）、InstructionUpdatePrompt.tsx 分支呈現；apps/desktop/core/src/project.rs 的 camelCase 契約測試擴充 status "newer" 與 newer 欄位。驗證：3.1 轉綠、cargo test -p speclink-desktop-core 綠。 <!-- speclink-task:tsk_01KZ99P1E6R6YW5YH86823RQVY -->

## 4. 安裝腳本

- [ ] 4.1 「本機安裝的新鮮度斷言」——新增 scripts/desktop-install.mjs 斷言鏈：印 HEAD／分支／dirty 與源碼 MARKER_VERSION → 執行 scripts/desktop-sidecar.mjs（永遠重建）→ 前端建置與 tauri bundle（簽章 env 缺失單行錯誤停止）→ 斷言 bundle 內 sidecar CLI 的 --version engine 版號等於源碼版號；帶 --install 續行：app 執行中單行錯誤（不代關）、覆蓋 /Applications、斷言安裝版同版；非 macOS 帶 --install 單行錯誤；任一步失敗非零結束不續行。驗證：本機實跑 node scripts/desktop-install.mjs 通過 bundle 斷言（exit 0）；以過期 binary 模擬（暫時替換 src-tauri/binaries/ 內檔案後跳過 sidecar 重跑斷言段）確認斷言非零並印出兩邊版號。 <!-- speclink-task:tsk_01KZ99P1E6VXPQZGN82K1MQ9VP -->

## 5. 回歸收尾

- [ ] 5.1 全量回歸：cargo test -p speclink-core --test it render_golden:: 全綠且 golden 與 assets.lock 零變動（本 change 不動 assets 與 marker render）；cargo test -p speclink-core、-p speclink-cli、npm test -w apps/desktop 全綠；確認除 --version 之外無 CLI 人眼或 --json 輸出波及、git status 無非預期檔案變動。驗證：上述指令全數通過。 <!-- speclink-task:tsk_01KZ99P1E6G2V764NGQ34ACRBC -->
