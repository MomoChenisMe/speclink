## ADDED Requirements

### Requirement: scopes 清單依 membership 過濾且身分即可呼叫

server SHALL 提供授權範圍查詢端點：以 Bearer 憑證（PAT 或 access token）驗身分即可呼叫、SHALL NOT 要求 repo 綁定 header——此端點正是用來選擇 scope 的。回應 SHALL 為呼叫者具 membership 的 Projects 及其 Repos（識別、key、顯示名）；無任何 membership SHALL 回空清單而非錯誤；停用帳號 SHALL 拒於 403、缺憑證 401。admin SHALL NOT 特權繞過 membership 過濾。

#### Scenario: 不同 membership 互不可見

- **WHEN** 使用者甲僅具專案 A 的 membership、使用者乙僅具專案 B 的，各自呼叫 scopes 端點
- **THEN** 甲只見 A 及其 repos、乙只見 B；互相看不見對方的專案

#### Scenario: 無 membership 回空清單

- **WHEN** 具有效憑證但無任何專案 membership 的使用者呼叫 scopes 端點
- **THEN** 回應為空清單、狀態 200，非 403

### Requirement: 正典 spec 內文可讀

server SHALL 提供綁定 scope 下正典 spec 內文的讀取端點（capability 定址）：存在回內文、缺席回 404（沿用既有 wire 錯誤詞彙）；SHALL 為唯讀且不觸發任何寫入。

#### Scenario: 讀取存在的正典 spec

- **WHEN** 對已封存合併過的 capability 呼叫 spec 內文端點
- **THEN** 回應內文與 store 中正典 spec.md 一致

### Requirement: archived 瀏覽三端點

server SHALL 提供綁定 scope 下 archived changes 的清單（dated name 降冪，含 date、name、任務計數、觸及規格數、建立者與來源討論——自 archived meta 與 artifacts 衍生）、單一 archived change 的 artifact 內文（缺席 404）與 delta capabilities 清單。三端點 SHALL 唯讀且不跨 scope 可見。

#### Scenario: archive 後即可瀏覽

- **WHEN** 一個 change 經 archive 動詞封存後呼叫 archived 清單與其 proposal 內文端點
- **THEN** 清單含該 dated name 且欄位如實，內文與封存時一致

### Requirement: workspace 全文搜尋與桌面語意對齊

server SHALL 提供綁定 scope 下的全文搜尋端點：不分大小寫子字串比對 active 變更全部 artifacts 與 live 討論記錄全文，每卡回傳首個命中（卡片種類、識別、命中 artifact 檔名、含命中原文的前後文 snippet，截斷端補 …）；空或全空白查詢 SHALL 回空陣列。語意 SHALL 與桌面本地搜尋一致，使 remote 分頁的搜尋結果形狀與本地同形。

#### Scenario: 搜尋命中變更與討論

- **WHEN** 對含關鍵字的 change artifact 與討論記錄各一的 scope 以該關鍵字搜尋
- **THEN** 回應恰兩筆命中：一筆 kind 為 change、一筆為 discussion，各含 artifact 檔名與 snippet

### Requirement: archived 列舉為 Store 契約的一部分

core Store 契約 SHALL 提供 archived changes 的列舉（dated name 清單）；各實作站點 SHALL 補齊——測試 store 與 host 的 TeamStore 橋接回真值、drift 專用唯讀最小 adapter 顯式回空並註明用途限定、檔案系統實作以 archive 目錄列舉。列舉排序 SHALL 為 dated name 降冪。

#### Scenario: 橋接列舉與點讀一致

- **WHEN** 對 TeamStore 橋接先 archive 一個 change 再呼叫列舉與 exists 點讀
- **THEN** 列舉含該 dated name 且 exists 為真，兩者一致
