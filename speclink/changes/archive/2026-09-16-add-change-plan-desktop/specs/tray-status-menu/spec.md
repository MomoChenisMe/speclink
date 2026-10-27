## ADDED Requirements

### Requirement: 變更列的波次標示與順序同源

系統匣的變更列順序 SHALL 與看板同一份清單 payload 的順序一致（即 plan 配置順序投影到各階段），SHALL NOT 另行排序。macOS 面板的變更列 SHALL 於清單項帶 `wave` 時在名稱之前渲染波次數字（與看板卡片波次章共用同一 i18n tooltip 詞條「第 N 波」）；`blockedBy` 非空的列 SHALL 整列降透明度且 tooltip 列出前置名稱；hover 反白期間數字 SHALL 隨列改以前景色。原生選單（非 macOS 平台，及 macOS 面板建立失敗的後備）的變更列標籤 SHALL 於 `wave` 存在時前綴「N· 」，SHALL NOT 做變淡或 tooltip。清單項缺 `wave`（remote 或 plan 成環）時面板與原生選單的列 SHALL 與本需求引入前一致。

#### Scenario: 面板列首波次與被擋變淡

- **WHEN** 面板開啟，進行中分區有 wave=1 的 add-a 與 wave=2、blockedBy=["add-a"] 的 add-b
- **THEN** 兩列依 add-a、add-b 順序，列首分別為「1」「2」，add-b 整列降透明度且 tooltip 為「等待：add-a」

#### Scenario: 原生選單前綴數字

- **WHEN** 於原生選單樣式展開系統匣，某變更 wave=3
- **THEN** 該變更列標籤為「3· 名稱＋文字進度條＋任務數」

#### Scenario: 缺欄位不變

- **WHEN** remote 專案的面板開啟（清單項無 wave）
- **THEN** 變更列無波次數字、不變淡，與本需求引入前一致
