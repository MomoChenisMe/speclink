# 跨層實作稽核紀錄

範圍：D1–D6 中檢查紀錄讀取、狀態傳遞、Desktop adapter 與活動／封存詳情頁。此紀錄是實作取捨檢查，不是 OWASP/CWE 安全審查或安全驗收。

## Scoundrel（刻意濫用）

- 查閱端只接受 `Review`／`Verify` 的 browse 入口；`Security` 被明確拒絕並維持原有嚴格 `index`／`record` 路徑。
- Host 的 Remote reader 先依 scope 建立授權快照；單筆查詢使用索引中的 ID 與相對路徑，不接受 UI 傳入任意檔案路徑。Bridge 的窄讀取另驗證 checks 路徑，並限制在活動或已封存快照。
- Desktop 品質關卡固定只請求 `review` 與 `verify`，不把安全收據／報告查詢混入同一組 Promise；UI 類別與錯誤狀態使用列舉，沒有可停用完整性檢查的設定。
- 未發現本次新增的路徑注入、權限繞過或安全機制降級入口。

## Lazy Developer（照最簡使用方式）

- 查閱只讀索引並逐筆標出缺檔、完整性錯誤或讀取失敗；預設保存政策不變，`omitted` 不觸發全文讀取。
- `Store::read_check_document` 的預設實作沿用既有 Option 讀取，適用於純記憶體 Store。具有外部 I/O 的 Store 必須覆寫以保留錯誤；FsStore 與 BridgeStore 已覆寫。這符合 D1 的明確相容要求，新增外部 I/O Store 時仍須實作窄讀取。
- 清單讀取失敗保留明確錯誤與重試入口；單筆讀取失敗不保留舊正文、不顯示永久載入骨架。無 adapter 時顯示來源不可用，而非空清單。

## Confused Developer（參數誤用）

- 檢查種類在 Core 使用 `CheckKind`，UI 使用有限 union；readError 也只有三種 wire/UI 對應，錯誤不會覆寫原 outcome。
- `archived` 布林是 Store 窄介面的既定形狀。FsStore 依該值進行活動／封存的受控讀取；BridgeStore 只從相符的已授權 snapshot 取資料。傳錯位置只會讀取對應來源或失敗，不會變成任意路徑能力。
- UI 回應同時綁定 adapter、change、refresh generation 與選取紀錄 ID；舊項目晚到的內容不能覆蓋新選取。

## 嚴格路徑與回歸

- `history::index`、`history::record` 仍用於安全報告及寫入／驗收守門；review／verify 的查閱入口才使用 `browse_index`、`browse_record`。蓋章、報告提交、風險決策與封存未改用 browse。
- FsStore 的一般 artifact 讀取仍保留原有 Option 相容介面；新增的檢查紀錄窄介面不吞 I/O 錯誤。Remote 查詢不修改 revision 或文件。
- `npm test -w packages/ui -- --maxWorkers=2 --minWorkers=1`：81 個測試檔、1,349 個測試通過。
- `npm test -w apps/desktop -- --maxWorkers=2 --minWorkers=1`：52 個測試檔、1,325 個測試通過。
- `npx tsc --noEmit -p packages/ui/tsconfig.json` 通過。
- `speclink validate archived-quality-history-access` 通過。
- `speclink analyze archived-quality-history-access --json`：Coverage、Consistency、Gaps 為 Clean；Ambiguity 有 18 項 Suggestion，均為情境沒有具體 Example，沒有 Critical／Warning。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs`：384/386 通過；兩項失敗均是 `docs/product-status.md` 與 `docs/product-status.zh-TW.md` 指向不存在的 `desktop-workflow-and-offline-security-review/evidence/*.md`。這兩份文件不在本次變更範圍且 `git diff` 未修改，故未改動產品文件來掩蓋既有連結問題。

結論：此範圍未發現需修正的高風險誤用；外部 I/O Store 必須覆寫窄讀取介面，是需遵守的實作條件，已在介面註解及此紀錄中標明。
