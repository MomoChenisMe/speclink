# 交付回歸

本次保留原始九項需求、驗收 checkbox 與追加的所有專案提案預設條列顯示。未執行此變更的正式 review／verify／archive，也未提交 Git。

## 通過的檢查

- Core lib：1073 passed、1 ignored。
- FS：45 unit、1 integration、22 store tests passed、1 ignored。
- Host：160 unit、5 bridge、1 成對 Remote 安全流程 passed。測試使用本機埠，於沙箱外執行。
- Protocol：76 passed；Remote：20 unit、104 integration passed。
- Server：35 unit、352 integration passed、1 ignored；3 PostgreSQL 環境測試因未設定 SPECLINK_TEST_POSTGRES_URL 跳過，未宣稱完整資料庫驗證。
- Desktop core：352 passed；Tauri cargo check、Desktop TypeScript noEmit 通過。
- 共用 UI：81 files、1342 passed。
- Desktop UI：第一次 1293 passed、1 failed。失敗是舊 App 測試把新拖曳功能的隱藏無障礙 status 當成更新橫幅；斷言排除該提示後，僅重跑 App 相關 3 files／184 tests，全通過。沒有宣稱初次整套 Desktop 執行全通過。
- Node SDK：重新建置後 8 files／55 passed；build 需使用 npm --prefix crates/adapters/speclink-node，不是 root workspace 名稱。
- CLI remote_verb_parity：49 passed，舊 server、參考模式人眼輸出維持一致。測試 server 補齊正式政策 GET，未略過權威設定讀取。
- CLI review_verbs：49 passed；security：18 passed；station 篩選：3 passed。
- render_golden：63 passed。內嵌資產維持本次未發行的 v1.50.2，鎖定與三個工具的技能由 CLI 一次重新產生，未手改生成檔。
- 最後執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs：386 passed。
- git diff --check 通過。

## 文件與技能交付

中英產品狀態文件逐項列出原九項需求與 checkbox，H2 錨點對等；技能數量同步。新 security-review 技能已歸入外掛品質類別，另新增缺本機知識庫不得下載或虛構 PASS 的評測案例及 grader。已檢查 fixture shell 語法與註冊涵蓋；未執行付費模型評測。

最後 analyze 無 Critical／Warning，45 個 Suggestion 為規格情境可補具體 Example 的提醒。安全政策目前 enabled=false，所以沒有替專案開啟或執行額外安全審查。

## 唯一未完成項

公司 Proxy 實機操作（12.3）需使用者於實際環境完成；company-proxy.md 明確記為未執行，不以模擬結果代簽。
