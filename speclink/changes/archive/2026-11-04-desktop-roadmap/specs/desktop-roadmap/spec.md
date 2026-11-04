## Purpose

桌面看板頁的路線圖檢視：一張以波次為欄、每個進行中的變更一列的表，畫出依賴線、標出 plan 的下一個、滑鼠停留時亮起整條路線，並列出討論結論中尚未轉出的刀。本 capability 只規定路線圖的呈現與入口切換；資料一律取自引擎 plan（經 client-protocol 的變更清單回應），路線圖上沒有寫入操作。

## ADDED Requirements

### Requirement: 看板頁的路線圖檢視

桌面 app 的看板頁 SHALL 有兩種檢視：卡片（既有看板）與路線圖。看板頁頁首 SHALL 有「看板｜路線圖」切換，兩種檢視的頁首皆 SHALL 呈現此切換；切換 SHALL 只改變看板頁顯示哪一種檢視，SHALL NOT 改變專案欄的導覽項與其高亮——兩種檢視下專案欄的「變更」項皆為現行項。app 啟動時 SHALL 為卡片檢視；同一次 app 執行期間切換分頁或切到其他頁再回到看板頁時 SHALL 維持上次選的檢視；此選擇 SHALL NOT 寫入任何檔案。

切到路線圖時卡片檢視 SHALL 保持掛載但不可見、不可互動，切回卡片時 SHALL 以切換前的捲動位置、搜尋字串與篩選狀態呈現。變更與討論詳情頁的宿主為看板頁（依 desktop-app「變更與討論詳情頁的宿主為看板」），開啟時 SHALL NOT 改變看板頁的檢視：自路線圖開啟的詳情頁關閉後 SHALL 回到路線圖；詳情頁的 ‹ n／m › 序列 SHALL 為看板卡片檢視的呈現順序（依 desktop-app「詳情頁的導覽與返回」），不另定路線圖順序。路線圖的資料 SHALL 與卡片檢視同一次載入，SHALL NOT 另行查詢；local 與 remote 專案的行為 SHALL 相同。

#### Scenario: 切到路線圖再切回看板

- **WHEN** 使用者於看板輸入搜尋字串 search、把「進行中」欄捲到底，點頁首切換的「路線圖」，再點「看板」
- **THEN** 點「路線圖」後主區為路線圖、專案欄「變更」項仍高亮；點「看板」後卡片檢視的搜尋字串仍為 search、「進行中」欄仍捲在底部

#### Scenario: 自路線圖開詳情頁後回到路線圖

- **WHEN** 使用者於路線圖點變更 add-b 的站，接著按 Esc
- **THEN** 開啟 add-b 的變更詳情頁，麵包屑右端的 ‹ n／m › 依看板卡片的順序；按 Esc 後主區回到路線圖而非卡片

#### Scenario: 切到其他頁再回來維持路線圖

- **WHEN** 使用者於路線圖點專案欄「規格」，再點「變更」
- **THEN** 主區先為規格頁，再回到看板頁時為路線圖檢視

### Requirement: 路線圖依波次排列變更

路線圖 SHALL 為一張表：最左為變更欄，其右每一波一欄、依波次升冪由左而右排列；每個波次欄的欄頭 SHALL 寫「第 N 波」與該波的變更數，第 1 波 SHALL 另寫「可以並行」，第 N 波（N ≥ 2）SHALL 另寫「等第 N−1 波」。每個帶波次的變更 SHALL 為表中的一列（一站），列序 SHALL 依波次升冪、同一波內依引擎 plan 的配置順序由上而下；該站的圓點 SHALL 落在所屬的波次欄，其餘波次欄留白。

每站 SHALL 在變更欄寫變更名與次行「完成數/總數 · 狀態」，並依階段呈現圓點：已就緒（全部任務已勾）為實心圓點、狀態寫「已就緒」；進行中以圓圈表示任務進度、狀態寫「進行中」；提案中為外框圓點、狀態寫「提案中」。阻擋清單非空的站 SHALL 整列變淡，狀態改寫所等的變更名。名稱等於 plan `next` 的站，狀態 SHALL 為「下一個」籤，圓點 SHALL 以主色光暈強調；`next` 為 null 時 SHALL 沒有站帶此籤。每站 SHALL 可以 Tab 聚焦，點擊或按 Enter SHALL 開啟該變更的詳情頁。

沒有波次的變更（metadata 無法解析）SHALL NOT 成為站；有這樣的變更時，路線圖頁首下方 SHALL 以一行說明「N 個變更無法排程，只在看板顯示」。路線圖 SHALL NOT 呈現討論卡與已封存的變更。

#### Scenario: 兩波與下一個

- **WHEN** 專案有 add-a（0/4 任務）、add-c（2/5 任務）、add-b（depends_on add-a，0/3 任務）三個進行中的變更，plan 把 add-a、add-c 配在第 1 波、add-b 配在第 2 波、`next` 為 add-a
- **THEN** 路線圖有變更欄與兩個波次欄，欄頭為「第 1 波」「2 · 可以並行」與「第 2 波」「1 · 等第 1 波」；列由上而下為 add-a、add-c、add-b，add-a 與 add-c 的圓點在第 1 波欄、add-b 的圓點在第 2 波欄；add-a 帶「下一個」籤；add-c 以圓圈表示進度並寫 2/5；add-b 整列變淡並寫出等 add-a

##### Example: 站的階段文字

`next` 為 null：

| 變更 | 任務 | 阻擋 | 圓點 | 站的次行 |
| ---- | ---- | ---- | ---- | -------- |
| add-a | 0/4，未開始 | 無 | 外框 | 0/4 · 提案中 |
| add-c | 2/5 | 無 | 圓圈進度 | 2/5 · 進行中 |
| add-d | 6/6 | 無 | 實心 | 6/6 · 已就緒 |
| add-b | 0/3 | add-a | 虛線圈 | 0/3 · 等待：add-a，整列變淡 |

#### Scenario: 無法排程的變更不成為站

- **WHEN** 專案有一個 .meta.yaml 無法解析的變更 broken-meta 與兩個正常的變更
- **THEN** 路線圖只有兩站，頁首下方寫「1 個變更無法排程，只在看板顯示」；卡片檢視照舊列出 broken-meta

### Requirement: 路線圖畫出依賴線並亮起路線

路線圖 SHALL 為每一條兩端都是路線圖上的站的依賴（`dependsOn`）畫一條線，起點為前置站、終點為依賴它的站；一端不在路線圖上（已封存、無法排程）的依賴 SHALL NOT 畫線。尚未轉出的刀 SHALL NOT 與任何站或刀連線。

滑鼠停留或鍵盤焦點落在一站時，該站、其全部遞移前置、其全部遞移依賴者，以及兩端都在這組站內的線 SHALL 維持原樣（線以主色強調），路線外的站與線 SHALL 變淡；滑鼠移出且焦點離開後 SHALL 恢復。高亮 SHALL 只改變呈現，SHALL NOT 改變任何資料或選定狀態。

#### Scenario: 停在中間的站亮起整條路線

- **WHEN** add-b depends_on add-a、add-c depends_on add-b、add-x 與三者無依賴，使用者把滑鼠停在 add-b 的站上
- **THEN** 路線圖有 add-a→add-b、add-b→add-c 兩條線；add-a、add-b、add-c 與兩條線維持原樣，add-x 變淡；滑鼠移出後 add-x 恢復

##### Example: 路線的範圍

| 停留的站 | 亮起的站 | 亮起的線 |
| -------- | -------- | -------- |
| add-a | add-a、add-b、add-c | add-a→add-b、add-b→add-c |
| add-c | add-a、add-b、add-c | add-a→add-b、add-b→add-c |
| add-x | add-x | 無 |

#### Scenario: 前置已封存時不畫線

- **WHEN** add-b depends_on add-a，add-a 已封存
- **THEN** 路線圖沒有連到 add-b 的線，add-b 沒有變淡

### Requirement: 路線圖列出尚未轉出的刀

變更清單回應的 `unpromotedCuts` 非空時，路線圖的表最右 SHALL 多一欄「尚未轉出」，欄頭寫刀數；每一項 SHALL 為表中的一列，排在所有站的列之下，順序 SHALL 與 `unpromotedCuts` 相同。刀的列 SHALL 在變更欄寫該刀尚未轉出的變更名（`unpromoted`），次行寫所屬討論 slug 與刀編號，再下一行以與討論詳情頁相同的指令籤寫出 `/speclink-propose --from-discussion <slug>`，按籤上的「複製指令」鈕後剪貼簿為該指令；範圍句（`title`）SHALL 在滑鼠停在變更名上時以提示呈現，空字串時沒有提示。刀的列在波次欄留白，在「尚未轉出」欄 SHALL 以討論詳情頁「尚未轉出」相同的橄欖色虛線方塊標示。討論 slug 在目前專案的討論清單內時 SHALL 可點，點擊 SHALL 開啟該討論的詳情頁；不在清單內時 SHALL 為不可點的文字。`unpromotedCuts` 為空時 SHALL 沒有這一欄與刀的列。

#### Scenario: 列出一把尚未轉出的刀

- **WHEN** 討論 herdr-speclink-panel 的結論有 `` **cut 4 `discussion-cuts-from-engine`**: 討論詳情頁改用引擎的刀 ``，其 promoted_to 不含 discussion-cuts-from-engine
- **THEN** 路線圖最右一欄的欄頭為「尚未轉出」「1 刀」；站的列之下有一列，變更名為 discussion-cuts-from-engine、次行為「herdr-speclink-panel · cut 4」、再下一行的指令籤為 `/speclink-propose --from-discussion herdr-speclink-panel`，滑鼠停在變更名上提示「討論詳情頁改用引擎的刀」，該列在「尚未轉出」欄有虛線方塊；按該列的「複製指令」後剪貼簿為 `/speclink-propose --from-discussion herdr-speclink-panel`

#### Scenario: 點討論名稱開討論詳情

- **WHEN** 使用者點上述刀的列次行的 herdr-speclink-panel
- **THEN** 開啟討論 herdr-speclink-panel 的詳情頁；關閉後回到路線圖

#### Scenario: 全部刀都已轉出時沒有這一欄

- **WHEN** 每份未封存討論的結論中的刀都已轉出
- **THEN** 路線圖只有變更欄與波次欄，沒有「尚未轉出」欄與刀的列

### Requirement: 路線圖的成環與空狀態

變更清單回應的 `planError` 非空（依賴成環）時，路線圖 SHALL NOT 畫波次欄、站與線，SHALL 顯示與看板相同文字的成環提示（含引擎的成環訊息）。沒有任何進行中的變更（帶波次與無法排程的都沒有）且 `unpromotedCuts` 為空時，路線圖 SHALL 顯示空狀態「沒有進行中的變更」；全部變更都無法排程且 `unpromotedCuts` 為空時，路線圖 SHALL 只呈現「N 個變更無法排程，只在看板顯示」一行，SHALL NOT 顯示空狀態與表。資料載入中 SHALL 呈現與卡片檢視相同的載入狀態。

#### Scenario: 依賴成環

- **WHEN** add-a 與 add-b 互相 depends_on，使用者切到路線圖
- **THEN** 路線圖沒有波次欄與線，顯示成環提示與 `dependency cycle: add-a -> add-b -> add-a`

#### Scenario: 沒有進行中的變更

- **WHEN** 專案沒有任何進行中的變更，也沒有尚未轉出的刀
- **THEN** 路線圖顯示「沒有進行中的變更」

#### Scenario: 全部變更都無法排程

- **WHEN** remote 專案的 server 沒有 plan 端點，兩個進行中的變更都沒有波次，也沒有尚未轉出的刀
- **THEN** 路線圖只寫「2 個變更無法排程，只在看板顯示」，沒有表，也沒有「沒有進行中的變更」
