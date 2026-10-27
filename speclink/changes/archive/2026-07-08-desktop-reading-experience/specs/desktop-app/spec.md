## ADDED Requirements

### Requirement: 介面文字以打包的 Noto Sans TC 呈現

桌面 app SHALL 將 Noto Sans TC 字體隨應用程式打包，並以其為介面與內容文字的第一優先字體；未安裝該字體的機器與離線環境 SHALL 呈現相同字體，SHALL NOT 依賴網路下載字體資產。等寬文字（inline code 與程式碼區塊）SHALL 維持等寬字體，不受此變更影響。

#### Scenario: 未安裝字體的機器呈現打包字體

- **WHEN** 在未安裝 Noto Sans TC 的作業系統上啟動桌面 app 並開啟任一文件檢視
- **THEN** 介面與 markdown 內容文字以打包的 Noto Sans TC 呈現，無對外字體網路請求

#### Scenario: 等寬文字不受字體變更影響

- **WHEN** 檢視含 inline code 或程式碼區塊的文件
- **THEN** 該段文字以等寬字體呈現，與周圍的 Noto Sans TC 內文明顯可辨

### Requirement: markdown 內容保留文件結構呈現

桌面 app 渲染 markdown 內容（變更抽屜的提案／設計／規格分頁、討論抽屜各分頁、已封存檢視）SHALL 保留來源的文件結構：無序清單 SHALL 顯示列表符號、有序清單 SHALL 顯示編號、段落之間 SHALL 有可辨識的垂直間距、來源中的單一換行 SHALL 呈現為換行。內容基準字級 SHALL 為 16px，任務分頁的任務文字 SHALL 同為 16px。排版與內容色彩 SHALL 於淺色與深色主題（跟隨系統偏好）一致生效。

#### Scenario: 清單顯示列表符號與編號

- **WHEN** 檢視含無序清單與有序清單的提案文件
- **THEN** 無序清單項目前顯示列表符號、有序清單項目前顯示編號，清單相對內文有縮排

#### Scenario: 單一換行呈現為換行

- **WHEN** 檢視討論記錄的討論過程分頁，其中一輪的 Focus 與 Position 行在來源中各佔一行、以單一換行分隔
- **THEN** 兩行在渲染結果中分行呈現，不塌成同一段連續文字

#### Scenario: 內容基準字級為 16px

- **WHEN** 檢視變更抽屜的提案分頁與任務分頁
- **THEN** markdown 內文與任務清單文字的計算字級皆為 16px

#### Scenario: 深色主題下排版一致

- **WHEN** 系統偏好為深色時檢視同一份文件
- **THEN** 列表符號、段落間距、字級與淺色主題一致，內容色彩取自深色 token 且可讀

### Requirement: raw HTML 不以原文呈現

桌面 app 渲染 markdown 內容時，來源中的 raw HTML（含 HTML 註解）SHALL NOT 以原始文字出現在渲染結果；渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 討論記錄的 scaffold 註解不顯示

- **WHEN** 開啟討論抽屜的討論過程分頁，其來源在 Rounds 區段含 CLI scaffold 產生的 HTML 註解行
- **THEN** 渲染結果不含該註解的任何文字，openspec/ 下的討論記錄檔案內容位元不變

#### Scenario: 程式碼區塊內的 HTML 原文照常顯示

- **WHEN** 檢視在 code fence 內含 HTML 標籤範例的文件
- **THEN** code fence 內容以原文完整顯示（過濾僅及於 code fence 之外的 raw HTML）
