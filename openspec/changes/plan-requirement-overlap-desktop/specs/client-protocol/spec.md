## MODIFIED Requirements

### Requirement: 變更清單的排程欄位

<!-- BEFORE: local 清單項四欄（wave、blockedBy、dependsOn、overlaps） -->

desktop 協定的 local 變更清單回應 SHALL 以引擎 plan 的配置順序排列 change 項，並增列 `wave`（整數）、`blockedBy`（字串陣列，只含作用中宣告前置）、`dependsOn`（字串陣列）、`overlaps`（陣列，每項 `change` 字串與 `capabilities` 字串陣列升冪）、`requirementOverlap`（陣列，每項 `change`、`capability`、`requirement`、`ownOperation`、`otherOperation` 字串與 `conflict` 布林）、`archiveAfter`（字串陣列）六欄，值 SHALL 取自引擎 plan 計算的同一入口，SHALL NOT 於呈現層另算；回應頂層 SHALL 增列 `planError`（字串或 null）。plan 因依賴成環失敗時，項目順序 SHALL 退回既有 board 排序、`wave`、`blockedBy`、`overlaps`、`requirementOverlap`、`archiveAfter` 五欄 SHALL 缺席、`dependsOn` SHALL 仍為各項 meta 的宣告原文（與 plan 列同一來源；詳情抽屜靠它讓使用者移除成環的前置）、`planError` SHALL 為引擎的成環訊息。壞 metadata 的 change 項 SHALL 照舊列出且六欄缺席。CLI `speclink list --json` SHALL NOT 包含六欄與 `planError`；remote 變更清單的排程欄位 SHALL 依「remote 變更清單的排程欄位」需求；本需求只規定 local 回應。

#### Scenario: 清單項帶排程欄位

- **WHEN** desktop 載入 local 變更清單，add-b 的 depends_on 含 add-a，add-b 與 add-c MODIFIED 同一個 requirement 且 add-c 配置在 add-b 前
- **THEN** 回應 changes 內 add-a 在 add-b 之前，add-b 項含 wave=2、blockedBy=["add-a"]、dependsOn=["add-a"]、overlaps=[]（或目錄級重疊清單）、requirementOverlap 含 change 為 add-c 的一項、archiveAfter=["add-c"]，頂層 planError 為 null，既有欄位不變

#### Scenario: 成環退回

- **WHEN** add-a 與 add-b 互相 depends_on
- **THEN** 回應 changes 為既有 board 排序、各項無 wave、blockedBy、overlaps、requirementOverlap、archiveAfter 五欄，add-a 的 dependsOn 為 ["add-b"]、add-b 的為 ["add-a"]，planError 為 `dependency cycle: add-a -> add-b -> add-a`

#### Scenario: CLI 清單不含排程欄位

- **WHEN** 執行 speclink list --json
- **THEN** change 項不含 wave、blockedBy、dependsOn、overlaps、requirementOverlap、archiveAfter，頂層無 planError，輸出與本需求引入前逐位元一致

### Requirement: remote 變更清單的排程欄位

<!-- BEFORE: remote 清單項四欄 -->

桌面 remote 資料源的變更清單 SHALL 以 GET /plan 的配置順序排列變更項（討論項維持 board resource overlay），並在每項疊 `wave`、`blockedBy`、`dependsOn`、`overlaps`、`requirementOverlap`、`archiveAfter` 六欄，頂層 `planError`（字串或 null）；六欄 SHALL 為桌面側組裝，SHALL NOT 改動 server list 端點的回應與 ChangeSummary 型別；舊 server 的 plan 回應缺 `requirementOverlap` 或 `archiveAfter` 時該兩欄 SHALL 為空陣列而非缺席。清單內有 plan 未列出的變更項時 SHALL 排在末尾、維持 server 回傳序、六欄缺席。plan 端點回 409（成環）時順序 SHALL 退回 board overlay、六欄缺席、planError 為其 message；plan 端點回其他錯誤（404 舊 server、連線失敗）時 SHALL 退回 board overlay、六欄缺席、planError 為 null。

#### Scenario: remote 清單帶排程欄位

- **WHEN** remote 分頁載入，server 的 plan 回 add-a 在 add-b 之前、add-b blockedBy ["add-a"]、add-b archiveAfter ["add-c"]
- **THEN** 桌面清單 add-a 在 add-b 之前，add-b 項含 wave=2、blockedBy=["add-a"]、archiveAfter=["add-c"]，看板顯示波次章與變淡

#### Scenario: 舊 server 退回

- **WHEN** server 對 GET /plan 回 404
- **THEN** 清單順序與本需求引入前一致、各項無六欄、planError 為 null、看板無章且無提示列

#### Scenario: remote 成環退回

- **WHEN** server 對 GET /plan 回 409、message 為 `dependency cycle: add-a -> add-b -> add-a`
- **THEN** 清單順序為 board overlay 序、各項無六欄、planError 為 `dependency cycle: add-a -> add-b -> add-a`，看板顯示成環提示列

#### Scenario: 舊 server 缺新欄位讀作空陣列

- **WHEN** server 的 plan 回應每項只有七鍵（無 requirementOverlap 與 archiveAfter）
- **THEN** 桌面清單項的 requirementOverlap 與 archiveAfter 皆為空陣列，wave 與 blockedBy 照常
