## ADDED Requirements

### Requirement: 討論資訊 payload 增選填 kind 欄位

討論列表與單筆讀取的 payload(CLI --json、server 讀取路徑與型別化 client 共用同一 wire contract)SHALL 增選填欄位 kind(字串,目前唯一合法值 improve):記錄有 kind 時 SHALL 曝露、無 kind 時 SHALL 省略該鍵,既有 payload 形狀 SHALL 逐位元不變。欄位名 SHALL 為 camelCase 的 kind。本欄位為唯讀資訊,SHALL NOT 改變 remote 路徑既有的離線、認證失效與 revision 行為。

#### Scenario: 改進討論經讀取路徑曝露 kind

- **WHEN** 對 kind 為 improve 的討論執行討論列表或單筆讀取(本地 --json 或經 server 讀取路徑)
- **THEN** payload 含 kind 欄位且值為 improve

#### Scenario: 一般討論 payload 形狀不變

- **WHEN** 對無 kind 欄位的既有討論執行討論列表或單筆讀取
- **THEN** payload 不含 kind 鍵,形狀與本欄位引入前逐位元一致
