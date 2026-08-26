## MODIFIED Requirements

### Requirement: capability 驅動停用且不偽造缺口

RemoteDataSource SHALL 附帶逐操作的 capability 描述（來源＝handshake 回應與端點覆蓋矩陣）；封存瀏覽、全文搜尋、正典 spec 內文、validate/analyze 動詞、刪除變更、任務拖排與看板拖排、change 詮釋資料與 capability 清單 SHALL 直達 server 對應端點、與本地 session 同形呈現（寫入面操作依 role：reader 呈現停用附繁體中文說明）；其中 change 詮釋資料與 capability 清單 SHALL 以單 change 讀取回應既有 payload 映射實作、不另開請求，接不送新欄位的舊 server 時 SHALL 以欄位缺席呈現（對應 UI 列不顯示）、SHALL NOT 偽造非空值——缺的是欄位而非能力；討論清單的 promotedTo SHALL 映射 wire 欄位、SHALL NOT 以 client 端固定值補齊。未來出現 server 無對應端點的操作時 SHALL 於 UI 停用並附繁體中文說明、對應 DataSource 方法 SHALL 回拒絕錯誤；SHALL NOT 於 client 端偽造或近似實作缺口。本地 session 的全部操作 SHALL 維持可用且行為零改動。批次任務操作以逐任務寫回組合時，中途失敗 SHALL 中止並回報已完成筆數。

#### Scenario: 全操作面直達

- **WHEN** 以 editor 身分於 remote 分頁執行 validate、刪除一個變更、拖排一個任務並拖排一張看板卡片
- **THEN** 四者皆如本地生效並呈現真實結果，無任何操作呈現「server 尚未提供」停用說明；本地分頁全功能照常

#### Scenario: reader 的寫入面呈現停用

- **WHEN** 以 reader 身分開啟 remote 分頁
- **THEN** 刪除變更、任務拖排與看板拖排呈現停用附繁中說明、讀取面與 validate/analyze 照常可用，對應停用方法回拒絕錯誤

#### Scenario: 詮釋資料與 capability 清單直達且誠實降級

- **WHEN** 於 remote 分頁開啟一個 change 的詳情抽屜（server 送齊歸屬欄位），再接一台不送新欄位的舊 server 開啟同畫面
- **THEN** 前者的詮釋資料區塊（建立者、建立工具、開工時間、開工者）與 capability 清單與本地同形呈現、無停用說明；後者無錯誤、對應列不顯示且無任何偽造值

#### Scenario: 討論 promotedTo 映射 wire 欄位

- **WHEN** remote 分頁載入含一筆已轉出討論（promotedTo 非空）與一筆未轉出討論的清單
- **THEN** 已轉出者落入「已轉出變更的討論」群組且連結其 change 名；未轉出者維持一般群組；無討論被固定空清單錯誤歸組
