## 1. 設定解析 fail-closed（design 決策五：設定解析 fail-closed——存在即必須可解析）

- [x] 1.1 撰寫設定載入 fail-closed 的失敗測試：`.speclink.yaml` 與 openspec/config.yaml「存在但壞 YAML」時載入回帶 workspace 相對路徑與解析原因的錯誤、「缺檔」時回預設（crates/speclink-core/src/config.rs 的 #[cfg(test)]；含 remote 區段與模式解析的 fail-closed 案例——壞 `.speclink.yaml` 不得解析為 fs 模式，crates/speclink-core/tests/mode_resolution.rs）。以 cargo test -p speclink-core 觀察紅燈。
- [x] 1.2 實作 typed 設定載入並於 core 內傳播（crates/speclink-core/src/config.rs、crates/speclink-core/src/workspace.rs 的 spec_dir 解析與 resolve_mode、crates/speclink-core/src/instructions.rs、crates/speclink-core/src/init.rs、crates/speclink-core/src/discuss.rs），1.1 測試轉綠：壞檔一律錯誤、缺檔一律預設、成功解析的檔案行為不變。
- [x] 1.3 撰寫並轉綠 CLI 整合測試，覆蓋「工作流政策的正典歸屬與四層解析順序」的新場景：壞 openspec/config.yaml → speclink instructions tasks 以非零 exit code 失敗且 stderr 指出檔案與原因、SPECLINK_TDD 環境變數不得繞過壞檔、缺檔仍走預設 exit 0；壞 `.speclink.yaml` → speclink list 非零 exit 且不讀 openspec/ 也不發遠端請求（crates/speclink-cli/tests/，沿用既有整合測試佈局）。
- [x] 1.4 跟進其餘呼叫點並收尾本階段：crates/speclink-cli/src/commands.rs 的 deprecated-keys 讀取、crates/speclink-node/src/lib.rs 的工作流設定讀取、apps/desktop/core/src/settings.rs 改呼叫下沉後的 typed 載入（移除自行嚴格解析的繞道，錯誤顯示路徑沿用既有 UI）。驗證：cargo test -p speclink-core（本機 Windows 加 --lib）與 cargo test -p speclink-cli 全綠。

## 2. command 模組型別與查詢群（design 決策一：runtime 落在 speclink-core 的 command 模組；決策二：動詞覆蓋判準——讀寫 Store 的領域動詞才進 runtime）

- [x] 2.1 撰寫 runtime 查詢群的失敗測試：經唯一進入點對測試 store 執行 list／status／validate 回 typed outcome、對不存在 change 回 not_found、查詢不產生事件（新檔 crates/speclink-core/src/command/mod.rs 的 #[cfg(test)]，用既有 teststore）。cargo test 紅燈。
- [x] 2.2 實作 command 模組：Command 封閉 enum（依決策二覆蓋表分組）、各 typed outcome、CommandError（typed error 與穩定錯誤碼註冊表：invalid_argv、not_found、invalid_config、refused、error）、execute 查詢群（list、show、status、instructions、validate、analyze、drift、artifact cat、language show、discuss list／show），2.1 轉綠；bootstrap 動詞（init、update、config、schema 等）不出現在 Command 中。
- [x] 2.3 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；驗證 cargo test -p speclink-core 全綠。

## 3. 變更群與領域事件（design 決策四：domain events 的種類、載荷與發出點）

- [x] 3.1 撰寫「變更型動詞的領域事件」失敗測試：new change 成功回報恰一筆 change-created（主體＋UTC 時間戳）、重名建立失敗不發事件、promote 回報 discussion-promoted 與 change-created 兩筆、覆蓋表 17 種動詞→事件對應逐一斷言（含 task undone → task-uncompleted）（crates/speclink-core/src/command/ 測試）。紅燈。
- [x] 3.2 實作變更群 execute 與事件建構（單一發出點、由 typed outcome 建構；new change、new artifact、task done、task undone、claim、in-progress add、archive、discard、discuss 全系列），3.1 轉綠後重構事件建構樣板；驗證 cargo test -p speclink-core 全綠。

## 4. CLI 分群切換（design 決策六：CLI 與 Node 的遷移策略——逐動詞群、輸出凍結）

- [x] 4.1 建置並保存 baseline exe（cargo build --release -p speclink-cli）於非 scratchpad 位置，準備固定樣本 workspace 供遷移前後雙沙盒對照；記錄對照步驟與樣本內容。
- [x] 4.2 查詢群 handler 改經 runtime（crates/speclink-cli/src/commands.rs，clap 定義與渲染碼不動）：對樣本 workspace 跑覆蓋表查詢動詞（人眼＋--json 兩形式），與 baseline diff 為空、--json 欄位維持既有 camelCase 契約；parity／color 對照相關項通過。
- [x] 4.3 變更群與 discuss 群 handler 改經 runtime：錯誤訊息文字沿用現行 CLI 訊息（穩定錯誤碼註冊表的 CLI 映射，design 決策三：typed error 與穩定錯誤碼註冊表）、refused 類拒絕（discard 未帶 --force）exit code 與訊息不變；與 baseline diff 為空。
- [x] 4.4 重構：移除 commands.rs 內因遷移而孤兒化的組裝碼；驗證 cargo test -p speclink-cli 全綠、cargo build --release 全 workspace 編譯通過。

## 5. Node dispatch 相容層（design 決策六）

- [x] 5.1 撰寫「dispatch 的輸入輸出契約」新場景的失敗測試：宿主 Store 工作流設定文字壞 YAML → dispatch(['new','change','demo']) 以 Error 拒絕且 code 為 invalid_config；不存在 change 的 status → code not_found 且 message 與 CLI 訊息相同（crates/speclink-node/__test__/，vitest）。紅燈。
- [x] 5.2 dispatch 改路由 runtime 並刪除 verb_list／verb_status／verb_new／verb_claim 的重組邏輯（crates/speclink-node/src/lib.rs）：argv 詞彙與 envelope 形狀不變、「動詞覆蓋與跨入口一致性」成立——同一 workspace 狀態下 dispatch 與 CLI 的結果語意與錯誤分類一致；5.1 與既有 vitest 全綠。
- [x] 5.3 驗證：cd crates/speclink-node 後 npm run build 與 npm test 全綠（engine／store-bridge／write-path／stress 既有套件不得回歸）。

## 6. 全量回歸與收尾

- [ ] 6.1 baseline exe 全量對照：對樣本 workspace 逐一執行覆蓋表動詞（人眼＋--json），遷移前後 stdout／stderr／exit code 逐位元一致（壞設定檔情境除外，該情境依 spec 斷言新錯誤行為）；parity 31 項／color 16 項／twin 8 情境全綠。
- [ ] 6.2 清除指向已移除 docs/verb-contract.md 的兩處註解殘引（crates/speclink-node/index.d.ts 的 dispatch 註解、crates/speclink-remote/src/lib.rs 的 crate 註解，改指 docs/platform-architecture.zh-TW.md）；驗證 cargo build --release 全 workspace 通過、git grep 不再命中 docs/verb-contract.md。
