## 1. snapshot provider 與投影佈局（design 決策一：materializer 落 speclink-host，snapshot 來源以 trait 注入）

- [x] 1.1 撰寫失敗測試，覆蓋「投影佈局與 manifest」：以測試 snapshot provider（本地 Store 快照替身）materialize 後，投影含 manifest.json（snapshotId、policyRevision、逐文件 digest 與 revision、camelCase）、INDEX.md 與 openspec 鏡像文件；整目錄刪除後重建等價；本地 fs 模式不建立投影（crates/speclink-host/src/projection.rs 的 #[cfg(test)]）。cargo test -p speclink-host 觀察紅燈。
- [x] 1.2 實作 snapshot provider trait（輸入 protocol 的 ContextSnapshotRequest、輸出 protocol 的 ContextSnapshot）與投影寫出，1.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 2. staging 與原子切換（design 決策二：staging 目錄加原子 rename 切換）

- [x] 2.1 撰寫失敗測試，覆蓋「staging 產生後原子切換」：materialize 先寫 staging 再原子切換；故障注入使 staging 中途失敗時現行投影逐位元不變且錯誤指出階段；切換失敗保留 staging 供重試（crates/speclink-host/src/projection.rs 測試，涵蓋 Windows rename 受限情境的錯誤路徑）。紅燈。
- [x] 2.2 實作 staging 與切換（舊投影 rename 後刪除或交換，任一失敗不留半套），2.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 3. 完整性、stale 與唯讀（design 決策三：完整性驗證 fail closed、stale 為顯式標記）

- [x] 3.1 撰寫失敗測試，覆蓋「完整性驗證 fail closed」與「stale 標記與 refresh」：修改任一投影文件一字元後 verify_projection 拒絕並指出 digest 不符文件；manifest 缺失同拒；mark_stale 只寫 marker、文件逐位元不變；refresh 全量重建、marker 清除、snapshotId 更新；唯讀屬性盡力設定且完整性以 digest 為準（crates/speclink-host/src/projection.rs 測試）。紅燈。
- [x] 3.2 實作 verify_projection、mark_stale 與 refresh，3.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 4. gitignore 保證（design 決策六：gitignore 保證沿 init／update 既有管理）

- [x] 4.1 撰寫並轉綠測試，覆蓋「投影必為 gitignore 涵蓋」：gitignore 未涵蓋 speclink 工作目錄的 workspace 執行 materialize 時補寫 gitignore 並輸出警告、git status 不見投影文件；已涵蓋時不動 gitignore 無警告（crates/speclink-host/src/projection.rs 測試）。

## 5. 依流程縮小 context（design 決策四：依流程縮小 context 為挑選規則、預設全量）

- [x] 5.1 撰寫失敗測試，覆蓋「依流程縮小 context」：五種流程參數各得對應預設集合（apply 含 proposal／design／tasks／delta specs／base specs 且不含無關 change；verify 為 apply 集合加最新 tasks 與驗證規則；discuss／propose／archive 各依表）；未給流程參數為全量（crates/speclink-host/src/projection.rs 測試）。紅燈。
- [x] 5.2 實作流程挑選規則（materializer 單一實作），5.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 6. remote instructions 與 skill 接線（design 決策五：remote instructions 與 skill 的接線為最小文案變更）

- [x] 6.1 撰寫並轉綠測試，覆蓋「remote skill 讀投影且禁止寫回」的 instructions 面：remote 模式 apply 階段 instructions 的 contextFiles 每個值為投影下路徑（key 與集合邏輯不變）、materialize 由 remote 動詞流程觸發（crates/speclink-core/src/instructions.rs、crates/speclink-cli/src/remote_commands.rs）；本地模式同動詞 instructions 輸出與現行逐位元一致（twin 對照的 remote instructions 期望值同步更新，其餘情境不變）。
- [x] 6.2 更新 apply 與 verify 技能的 remote 段落（讀投影、唯讀、修改必經 speclink 動詞）並完成三處同步：先修改 crates/speclink-core/assets 下技能與 .claude/skills 與 .agents/skills 實例並提交，再於乾淨樹以 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生 golden、逐 diff 審視後提交；驗證 render golden 與 cargo test -p speclink-core 全綠。

## 7. 全量收尾

- [x] 7.1 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；驗證 cargo test --workspace 與 npm run test:all 全綠；parity 31 項／color 16 項／twin 8 情境全綠（remote instructions 新期望值除外皆逐位元不變）；git diff --stat 對照 proposal Impact 清單檢查改動面無溢出（本地 fs 模式零行為變更）。
