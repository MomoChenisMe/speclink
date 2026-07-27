## MODIFIED Requirements

### Requirement: 真實 CLI 端到端一致

以真實 CLI binary 對真 server（SQLite store）執行 remote 動詞流程 SHALL 與 fs 模式（形狀權威）一致。儲存決定型輸出（無本地路徑欄位者，如 status）SHALL stdout/stderr/exit code 逐位元一致；帶本地路徑或投影欄位的輸出（如 apply 的 changeDir/contextFiles，以及 fs-only 的 preflight）SHALL 在剔除該類欄位後內容一致——此與 stub 對測對同類欄位採欄位形狀（key）比對的語意一致。`crates/speclink-cli/tests/remote_read_path.rs` 全部對照情境的欄位形狀由 stub 對測凍結（設計決策 7：stub 驗 client、e2e 驗 server，互補不互代）；e2e SHALL 以代表性 remote 動詞重放驗證 server 端到端行為並驗證重啟持久性。e2e 的資料播種 SHALL 經命令路由完成，SHALL NOT 直接寫入 store 後端。

#### Scenario: 代表性動詞對真 server 重放

- **WHEN** 啟動 tempdir SQLite 組態的真 server，將 CLI 指向它並重放代表性 remote 動詞（list、status、instructions apply、discuss list）
- **THEN** 儲存決定型輸出與 fs 模式逐位元一致、帶路徑欄位者剔除該類欄位後內容一致；server 重啟後既建立的資料仍可完整查詢
