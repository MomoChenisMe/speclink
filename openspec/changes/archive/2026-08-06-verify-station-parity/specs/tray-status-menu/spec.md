## ADDED Requirements

### Requirement: 面板變更列的品質站章

macOS 面板的變更列 SHALL 依協定的審查與驗證狀態，於名稱與任務數之間並排渲染行內站章，順序固定（審查章在前、驗證章在後）：`none` 無章；`inReview`／`inVerify` 顯示進行中章；`reviewed`／`verified` 顯示已蓋章章；`reviewedStale`／`verifiedStale` 顯示降級樣式章。各章的圖示、色調與 tooltip 詞條 SHALL 與看板卡片的對應章共用同一組樣式與 i18n 詞條，SHALL NOT 另建第二份對照。變更列 SHALL NOT 渲染看板卡片的其他行內符號（建立者頭像、來源討論標記、restale 標記、metaError 標記）。原生選單（非 macOS 平台，及 macOS 面板建立失敗的後備）的變更列標籤 SHALL 維持既有「名稱＋文字進度條＋任務數」組成、SHALL NOT 加入站章。

#### Scenario: 面板兩章並排

- **WHEN** 面板開啟且某變更的協定資料為 `reviewStatus: "reviewed"` 且 `verifyStatus: "verified"`
- **THEN** 該變更列於名稱與任務數之間依序顯示審查章與驗證章，tooltip 分別為「已審查」與「已驗證」（tw）

#### Scenario: 僅驗證章

- **WHEN** 某變更的協定資料為 `reviewStatus: "none"` 且 `verifyStatus: "inVerify"`
- **THEN** 該變更列僅顯示「驗證中」驗證章，無審查章

#### Scenario: 無章時列組成不變

- **WHEN** 某變更的兩站狀態皆為 `none`
- **THEN** 該變更列不渲染任何站章與其他新增元素，列組成與本變更前一致

#### Scenario: 原生選單不受影響

- **WHEN** 於原生選單樣式（非 macOS 平台，或 macOS 面板建立失敗後備）展開系統匣選單，某變更為 `reviewStatus: "reviewed"`
- **THEN** 該變更列標籤仍為「名稱＋文字進度條＋任務數」，不含任何站章字元
