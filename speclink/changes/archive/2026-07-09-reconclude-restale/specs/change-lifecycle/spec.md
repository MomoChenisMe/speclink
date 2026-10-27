## ADDED Requirements

### Requirement: restale_from 記錄變更待重新反映的討論並經 CLI 觀測

變更 meta 檔（openspec/changes/<name>/.openspec.yaml）MAY 帶 restale_from 欄位——逗號分隔的討論 slug 清單，語意為「本變更曾反映（seal）這些討論，其後這些討論被重新結論，內容相對新結論過期、待 re-ingest」。ChangeMeta SHALL 提供 restale_from() accessor 回傳 Vec<String>：欄位缺席時回空、逗號值 SHALL 各段 trim 後分割，行為平行既有 from_discussion／from_discussions()。此欄位由 discuss conclude 寫入、discuss seal 清除（見 discussion-docs 正典），本需求規範其讀取與觀測。speclink show <change> --json SHALL 於變更 payload 恆曝 restaleFrom（camelCase 字串陣列，無旗標為空陣列），平行既有 fromDiscussions。speclink list --json SHALL 於 restale_from 非空的變更 payload 曝 restaleFrom 陣列、為空時省略該欄位——以維持 list --json 對無旗標變更的既有（Spectra 對照）輸出逐位元不變。speclink analyze <change> 於某變更 restale_from 非空時 SHALL 出一條資訊性 finding，指明該變更反映的討論已重新結論、需重新 ingest 以同步新結論。此欄位讀取 SHALL 為零 per-load 掃描——僅讀既存 meta 欄位，不掃描討論記錄。

#### Scenario: restale_from() accessor 讀取

- **WHEN** 變更 meta 含 restale_from: alpha-search, beta-cache
- **THEN** ChangeMeta::restale_from() 回傳 ["alpha-search", "beta-cache"]；meta 無該欄位時回傳空 Vec

#### Scenario: show 恆曝 restaleFrom

- **WHEN** 對 restale_from 含 alpha-search 的變更、以及無該欄位的變更，各執行 speclink show <change> --json
- **THEN** 前者 payload 的 restaleFrom 為 ["alpha-search"]；後者 payload 的 restaleFrom 為空陣列（欄位恆存在）

#### Scenario: list 曝 restaleFrom 且對無旗標變更保 parity

- **WHEN** 對含一個 restale_from 非空變更與一個無該欄位變更的專案執行 speclink list --json
- **THEN** 非空變更的 payload 含 restaleFrom 陣列（如 ["alpha-search"]）；無旗標變更的 payload 省略 restaleFrom 欄位，其 list --json 輸出與本變更前逐位元一致

#### Scenario: analyze 對過期變更出資訊性 finding

- **WHEN** 對 restale_from 非空的變更執行 speclink analyze <change>
- **THEN** 輸出含一條資訊性 finding，指明該變更反映的討論已重新結論、需 re-ingest；restale_from 為空時無此 finding，且 analyze 輸出與本變更前逐位元一致
