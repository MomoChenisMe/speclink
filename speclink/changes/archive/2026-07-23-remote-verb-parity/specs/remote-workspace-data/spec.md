## MODIFIED Requirements

### Requirement: capability 驅動停用且不偽造缺口

RemoteDataSource SHALL 附帶逐操作的 capability 描述（來源＝handshake 回應與端點覆蓋矩陣）；封存瀏覽、全文搜尋、正典 spec 內文、validate/analyze 動詞、刪除變更與任務拖排 SHALL 直達 server 對應端點、與本地 session 同形呈現（刪除與任務拖排依 role：reader 呈現停用附繁體中文說明）；server 仍無對應端點的操作（看板拖排）SHALL 於 UI 停用並附繁體中文說明，對應 DataSource 方法 SHALL 回拒絕錯誤；SHALL NOT 於 client 端偽造或近似實作缺口。本地 session 的全部操作 SHALL 維持可用且行為零改動。批次任務操作以逐任務寫回組合時，中途失敗 SHALL 中止並回報已完成筆數。

#### Scenario: 動詞與寫入面直達而看板拖排停用

- **WHEN** 以 editor 身分於 remote 分頁執行 validate、analyze、刪除一個變更並拖排一個任務
- **THEN** 四者皆如本地生效並呈現真實結果（刪除後卡片消失、任務落位並重編號）；同時看板卡片拖排維持停用附繁中說明，本地分頁全功能照常

#### Scenario: reader 的寫入面呈現停用

- **WHEN** 以 reader 身分開啟 remote 分頁
- **THEN** 刪除變更與任務拖排呈現停用附繁中說明、validate/analyze 照常可用，對應停用方法回拒絕錯誤
