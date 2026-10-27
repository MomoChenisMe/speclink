## MODIFIED Requirements

### Requirement: 規格頁提供清單、搜尋與展開檢視

左側導覽的「規格」項 SHALL 進入規格頁，以卡片清單呈現全部正典 spec：每張卡 SHALL 含 spec 名稱、最後修改相對時間（自檔案系統 mtime 衍生，天級：今天／昨天／N 天前；mtime 不可得時該資訊缺席）、複製名稱鈕與展開／縮合控制。規格頁 SHALL 提供搜尋列，以大小寫不敏感的名稱子字串即時過濾清單。點卡片標題 SHALL 展開顯示該 spec 的正典 spec.md 全文（markdown 渲染），再點 SHALL 縮合；全文內容 SHALL 於首次展開時才載入。展開檢視的 spec.md 全文下方 SHALL 顯示一行來源變更 footer，列出該 spec 內所有 @trace 區塊的 source 變更名（去重、依文件首次出現順序）；spec.md 不含任何帶 source 的 @trace 時該 footer SHALL 缺席。此 footer SHALL 僅呈現、SHALL NOT 可點擊，且 SHALL NOT 顯示 @trace 的 updated 或 code。無 spec 的專案與搜尋無結果 SHALL 各顯示空狀態文案。規格頁 SHALL 為唯讀，SHALL NOT 提供任何規格寫入操作。

#### Scenario: 進入規格頁顯示卡片清單

- **WHEN** 於含多個正典 spec 的專案點左側導覽「規格」
- **THEN** 導覽項呈 active 樣式，主內容顯示全部 spec 卡片，各含名稱與最後修改相對時間

#### Scenario: 搜尋列名稱過濾

- **WHEN** 於搜尋列輸入部分 spec 名稱
- **THEN** 清單即時縮至名稱含該子字串（大小寫不敏感）的卡片；清空輸入後清單還原

##### Example: 過濾行為

| 既有 specs | 輸入 | 顯示 |
| ---------- | ---- | ---- |
| desktop-app、desktop-config、node-sdk | desktop | desktop-app、desktop-config |
| desktop-app、desktop-config、node-sdk | SDK | node-sdk |
| desktop-app、desktop-config、node-sdk | zzz | 無結果空狀態 |

#### Scenario: 展開卡片顯示正典全文

- **WHEN** 點一張縮合卡片的標題
- **THEN** 卡片展開顯示該 spec 的 spec.md 全文 markdown 渲染（首次展開先呈載入態），再點標題即縮合，其他已展開卡片不受影響

#### Scenario: 展開檢視顯示來源變更 footer

- **WHEN** 展開一張其 spec.md 含至少一個帶 source 的 @trace 的卡片
- **THEN** 全文下方顯示一行來源變更 footer，內容為該檔所有 @trace 的 source 去重、依首次出現順序排列，前置在地化標籤

##### Example: 來源去重與排序

| spec.md 內 @trace source 出現序 | footer 顯示 |
| ------------------------------- | ----------- |
| A、A、B | A、B |
| B、A、B | B、A |
| （無 @trace 或無 source） | 無 footer |

#### Scenario: 複製名稱

- **WHEN** 點卡片的複製名稱鈕
- **THEN** spec 名稱寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 無 spec 專案顯示空狀態

- **WHEN** 於無任何正典 spec 的專案進入規格頁
- **THEN** 顯示空狀態文案而非空白頁

#### Scenario: 外部變更後反映

- **WHEN** 規格頁開啟期間外部寫者修改某 spec 的 spec.md
- **THEN** 世代重載後清單的修改時間更新，已展開卡片的內容反映新全文
