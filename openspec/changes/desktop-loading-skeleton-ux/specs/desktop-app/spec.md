## ADDED Requirements

### Requirement: 分頁切換中即時回饋

點擊主視窗分頁列上的非活躍本地分頁後、專案探測（probe）完成前，目標分頁 SHALL 立即顯示切換中 spinner，且原 workspace 畫面 SHALL 照常顯示並可互動；探測成功翻頁或失敗轉分頁錯誤時 spinner SHALL 消失。切換順序（探測成功才切換活躍分頁）與既有分頁錯誤呈現 SHALL 維持不變。spinner SHALL 帶有切換中語意的 aria-label。

#### Scenario: 點擊分頁立即出現 spinner

- **WHEN** 點擊非活躍的本地分頁且專案探測尚未完成
- **THEN** 目標分頁立即顯示 spinner，畫面停留在原 workspace 且可互動

#### Scenario: 探測成功後切頁

- **WHEN** 專案探測成功完成
- **THEN** spinner 消失，活躍分頁切換為目標分頁

#### Scenario: 探測失敗回饋照舊

- **WHEN** 專案探測失敗（如目錄已不存在）
- **THEN** spinner 消失，分頁以既有錯誤樣式呈現，活躍分頁不變

### Requirement: 看板首訪以 skeleton 佔位

活躍 workspace 的快照尚未完成首次載入時，看板 SHALL 以 skeleton 佔位卡呈現卡片區，欄位標題 SHALL 照常顯示真實文字；快照已載入（含上次造訪的舊快取）時 SHALL 直接顯示既有資料，整批重載完成後靜默更新，SHALL NOT 於重載期間顯示 skeleton。skeleton 區塊 SHALL 標記 aria-busy，其動畫 SHALL 於 prefers-reduced-motion 下停用。

#### Scenario: 首訪 workspace 顯示骨架卡

- **WHEN** 切換至首次造訪的 workspace 且整批載入尚未完成
- **THEN** 看板欄名照常顯示，卡片區為 skeleton 佔位卡，與空 workspace 的空態呈現可區分

#### Scenario: 載入完成換為真資料

- **WHEN** 首訪 workspace 的整批載入完成
- **THEN** skeleton 消失，顯示實際卡片；workspace 確實無內容時顯示既有空態呈現

#### Scenario: 已訪 workspace 不閃 skeleton

- **WHEN** 切換至先前造訪過的 workspace
- **THEN** 立即顯示上次快照的資料，全程無 skeleton，重載完成後畫面靜默更新

### Requirement: 抽屜文件載入以 skeleton 呈現

詳情抽屜（變更、規格、討論、已封存）的各文件分頁 SHALL 統一區分文件三態：載入中 SHALL 渲染文件 skeleton（標題條與數行內文條），載入完成且檔案不存在才 SHALL 顯示該分頁的空態文案，載入完成且有內容 SHALL 渲染內容。任何文件分頁 SHALL NOT 於載入中顯示「無文件」類空態文案。

#### Scenario: 文件載入中顯示骨架

- **WHEN** 開啟抽屜的任一文件分頁且文件內容尚未抵達
- **THEN** 該分頁顯示文件 skeleton，不顯示任何空態文案

#### Scenario: 載入完成檔案不存在

- **WHEN** 文件載入完成且該檔案不存在
- **THEN** skeleton 消失，顯示該分頁既有的空態文案

#### Scenario: 載入完成有內容

- **WHEN** 文件載入完成且有內容
- **THEN** skeleton 消失，內容以既有渲染路徑呈現
