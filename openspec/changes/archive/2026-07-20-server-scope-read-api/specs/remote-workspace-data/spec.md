## MODIFIED Requirements

### Requirement: capability 驅動停用且不偽造缺口

RemoteDataSource SHALL 附帶逐操作的 capability 描述（來源＝handshake 回應與端點覆蓋矩陣）；封存瀏覽、全文搜尋與正典 spec 內文 SHALL 直達 server 讀取端點、與本地 session 同形呈現；server 仍無對應端點的操作（validate/analyze 動詞、刪除變更、任務拖排、看板拖排）SHALL 於 UI 停用並附繁體中文說明，對應 DataSource 方法 SHALL 回拒絕錯誤；SHALL NOT 於 client 端偽造或近似實作缺口。本地 session 的全部操作 SHALL 維持可用且行為零改動。批次任務操作以逐任務寫回組合時，中途失敗 SHALL 中止並回報已完成筆數。

#### Scenario: 讀取面直達而動詞面停用

- **WHEN** 於 remote 分頁開啟 archived 頁、開啟規格卡內文並使用看板搜尋
- **THEN** 三者皆如本地呈現真實資料；同時刪除、拖排與 validate/analyze 維持停用附說明，本地分頁全功能照常

#### Scenario: 不支援操作呈現停用

- **WHEN** 於 remote 分頁的看板嘗試刪除變更或拖排卡片
- **THEN** 該操作呈現停用附繁中說明，對應方法回拒絕錯誤；同時本地分頁上述操作照常可用
