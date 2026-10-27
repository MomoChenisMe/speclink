## MODIFIED Requirements

### Requirement: export 與 import 以 versioned bundle 往返

export SHALL 輸出帶格式版本、scope、project revision 與逐文件 digest 的 bundle；import SHALL 驗證格式版本與 digest、依指定模式（全新建立或覆蓋）套用並回報逐文件結果，驗證失敗 SHALL 拒絕且不部分套用。全新建立模式的前置 SHALL 為「目標 scope 不持有任何文件」——scope 持有任何文件（無論是否與 bundle 內文件同名）即 SHALL 整筆拒絕（backend 類別）、不部分套用、scope 狀態不變；SHALL NOT 以「bundle 內文件是否已存在」代替此檢查。conformance suite SHALL 含此邊界的 gate，全部實作 SHALL 一致通過。

#### Scenario: round-trip 內容一致

- **WHEN** 對含多份文件的 repo 執行 export，將 bundle import 到全新 store 後逐文件比對
- **THEN** 全部文件內容一致；新 store 的每份文件歷史以 import 為起點；digest 驗證通過

#### Scenario: 全新建立模式拒絕非空 scope

- **WHEN** 目標 scope 已持有一份 bundle 外的文件 X，以全新建立模式 import 只含文件 Y 的 bundle
- **THEN** import 整筆拒絕且錯誤為 backend 類別；scope 仍只持有 X、project revision 未動、無任何 Y 的痕跡

#### Scenario: 覆蓋模式不受空 scope 前置影響

- **WHEN** 對持有既有文件的 scope 以覆蓋模式 import 同名文件的 bundle
- **THEN** import 成功且逐文件結果回報覆蓋；全新建立前置不適用於此模式
