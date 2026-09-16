## ADDED Requirements

### Requirement: 變更清單的排程欄位

desktop 協定的 local 變更清單回應 SHALL 以引擎 plan 的配置順序排列 change 項，並增列 `wave`（整數）、`blockedBy`（字串陣列）、`dependsOn`（字串陣列）、`overlaps`（陣列，每項 `change` 字串與 `capabilities` 字串陣列升冪）四欄，值 SHALL 取自引擎 plan 計算的同一入口，SHALL NOT 於呈現層另算；回應頂層 SHALL 增列 `planError`（字串或 null）。plan 因依賴成環失敗時，項目順序 SHALL 退回既有 board 排序、`wave`、`blockedBy`、`overlaps` 三欄 SHALL 缺席、`dependsOn` SHALL 仍為各項 meta 的宣告原文（與 plan 列同一來源；詳情抽屜靠它讓使用者移除成環的前置）、`planError` SHALL 為引擎的成環訊息。壞 metadata 的 change 項 SHALL 照舊列出且四欄缺席。CLI `speclink list --json` SHALL NOT 包含四欄與 `planError`；remote 變更清單的排程欄位 SHALL 依「remote 變更清單的排程欄位」需求（另一變更定義）；本需求只規定 local 回應。

#### Scenario: 清單項帶排程欄位

- **WHEN** desktop 載入 local 變更清單，add-b 的 depends_on 含 add-a
- **THEN** 回應 changes 內 add-a 在 add-b 之前，add-b 項含 wave=2、blockedBy=["add-a"]、dependsOn=["add-a"]、overlaps=[]，頂層 planError 為 null，既有欄位不變

#### Scenario: 成環退回

- **WHEN** add-a 與 add-b 互相 depends_on
- **THEN** 回應 changes 為既有 board 排序、各項無 wave、blockedBy、overlaps 三欄，add-a 的 dependsOn 為 ["add-b"]、add-b 的為 ["add-a"]，planError 為 `dependency cycle: add-a -> add-b -> add-a`

#### Scenario: CLI 清單不含排程欄位

- **WHEN** 執行 speclink list --json
- **THEN** change 項不含 wave、blockedBy、dependsOn、overlaps，頂層無 planError，輸出與本需求引入前逐位元一致
