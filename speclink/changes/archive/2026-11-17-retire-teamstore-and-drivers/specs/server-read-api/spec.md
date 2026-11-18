## MODIFIED Requirements

### Requirement: archived 列舉為 Store 契約的一部分
<!-- BEFORE: 實作站點列的是「測試 store 與 host 的 TeamStore 橋接」；情境對 TeamStore 橋接操作 -->

core Store 契約 SHALL 提供 archived changes 的列舉（dated name 清單）；各實作站點 SHALL 補齊——測試 store 與 host 經儲存接口的儲存視圖回真值、drift 專用唯讀最小 adapter 顯式回空並註明用途限定、檔案系統實作以 archive 目錄列舉。列舉排序 SHALL 為 dated name 降冪。

#### Scenario: 橋接列舉與點讀一致

- **WHEN** 經儲存接口的儲存視圖先 archive 一個 change 再呼叫列舉與 exists 點讀
- **THEN** 列舉含該 dated name 且 exists 為真，兩者一致
