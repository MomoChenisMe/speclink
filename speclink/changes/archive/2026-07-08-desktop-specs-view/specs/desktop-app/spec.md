## ADDED Requirements

### Requirement: 規格頁提供清單、搜尋與展開檢視

左側導覽的「規格」項 SHALL 進入規格頁，以卡片清單呈現全部正典 spec：每張卡 SHALL 含 spec 名稱、最後修改相對時間（自檔案系統 mtime 衍生，天級：今天／昨天／N 天前；mtime 不可得時該資訊缺席）、複製名稱鈕與展開／縮合控制。規格頁 SHALL 提供搜尋列，以大小寫不敏感的名稱子字串即時過濾清單。點卡片標題 SHALL 展開顯示該 spec 的正典 spec.md 全文（markdown 渲染），再點 SHALL 縮合；全文內容 SHALL 於首次展開時才載入。無 spec 的專案與搜尋無結果 SHALL 各顯示空狀態文案。規格頁 SHALL 為唯讀，SHALL NOT 提供任何規格寫入操作。

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

#### Scenario: 複製名稱

- **WHEN** 點卡片的複製名稱鈕
- **THEN** spec 名稱寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 無 spec 專案顯示空狀態

- **WHEN** 於無任何正典 spec 的專案進入規格頁
- **THEN** 顯示空狀態文案而非空白頁

#### Scenario: 外部變更後反映

- **WHEN** 規格頁開啟期間外部寫者修改某 spec 的 spec.md
- **THEN** 世代重載後清單的修改時間更新，已展開卡片的內容反映新全文

## MODIFIED Requirements

### Requirement: 桌面 app 呈現 change 與 spec 的清單與內容

<!-- BEFORE: 清單與狀態資料的欄位與值一律與對應 CLI --json 輸出一致，無呈現層輔助欄位的豁免。 -->

桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。清單與狀態資料的欄位與值 SHALL 與對應 CLI `--json` 輸出一致；自檔案系統衍生的呈現層輔助欄位（如 spec 的最後修改時間）不屬此對齊範圍，SHALL NOT 出現在 CLI 輸出對照要求中。

#### Scenario: 顯示 change 清單與狀態

- **WHEN** app 於含多個 active change 的專案啟動
- **THEN** 每個 change 以其名稱與 proposal/tasks 狀態呈現，欄位與值對應 speclink list 與 speclink status 的 --json 輸出

#### Scenario: 選定 change 顯示其文件內容

- **WHEN** 使用者在清單中選定一個 change
- **THEN** app 顯示該 change 的 proposal 內容，並可切換檢視其 design 與 tasks（若存在）

#### Scenario: 選定 spec 顯示其正典內容

- **WHEN** 使用者選定一個 spec
- **THEN** app 顯示該 spec 的正典 spec.md 內容
