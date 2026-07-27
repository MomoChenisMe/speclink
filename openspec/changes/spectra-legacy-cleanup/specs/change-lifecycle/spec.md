## MODIFIED Requirements

### Requirement: 任務完成蘊含開工標記

speclink task done 成功完成一項任務時，若該 change 的 .openspec.yaml 尚無 started_* 欄位，SHALL 於同一操作內蓋開工章：started_at 為當日 ISO 日期；started_by 依 git 身分可得性寫入——可歸屬者寫入、不可歸屬者缺席；本指令無 agent 識別來源，started_with 缺席。meta 既有欄位 SHALL 逐字元保留。該 change 已有 started_* 時 SHALL 保留首章不變。touched-files 記錄（.speclink/ 下）行為 SHALL 維持現行語意。

本需求為輸出凍結敏感：指令的人眼輸出、--json payload（change、status、taskDesc、taskId 對應之既有欄位形狀）與 exit code SHALL 與現行位元級一致（既有輸出基線不變）；.openspec.yaml 的開工章為刻意檔案效果變更——自我基線的檔案樹對照 SHALL 隨本需求更新並記載此差異。錯誤路徑（tasks.md 缺失、任務序號無效、任務已完成）SHALL 維持現行訊息與非零 exit code，且 SHALL NOT 寫入任何檔案。

#### Scenario: 首次完成任務蓋開工章

- **WHEN** 對 meta 含 created_* 而無 started_* 的 change 執行 speclink task done 完成一項未完成任務
- **THEN** tasks.md 該任務標記為 [x]，stdout 與 exit code 與現行一致，.openspec.yaml 新增 started_at（git 身分可得時另含 started_by），schema 與 created_* 欄位逐字元保留

##### Example: meta 前後對照

- **GIVEN** .openspec.yaml 內容為 schema、created、created_by、created_with 四欄，tasks.md 為 0/5
- **WHEN** speclink task done 1 --change demo 成功
- **THEN** .openspec.yaml 於既有四欄之後新增 started_at: <當日> 與 started_by: <git 身分>，無其他變動；tasks.md 成 1/5

#### Scenario: 已開工的 change 完成後續任務不改章

- **WHEN** 對已含 started_* 的 change 執行 speclink task done 完成另一項任務
- **THEN** started_at、started_by、started_with 值與執行前完全相同（首章保留），tasks.md 正常勾章

#### Scenario: 任務已完成時無任何檔案效果

- **WHEN** 對已標記 [x] 的任務再執行 speclink task done
- **THEN** 指令以現行「already done」錯誤訊息與非零 exit code 結束，tasks.md、.openspec.yaml 與 touched 記錄皆無變動

#### Scenario: tasks.md 缺失時不蓋章

- **WHEN** 對無 tasks.md 的 change 名執行 speclink task done
- **THEN** 指令以現行「tasks.md not found」錯誤結束，該 change 的 .openspec.yaml（若存在）無任何變動

### Requirement: 變更以 discard 動詞廢棄

speclink discard SHALL 接受一個位置參數（變更名）與 --force、--json 旗標，廢棄一個尚未動工的變更：刪除 openspec/changes/<change>/ 目錄整棵，並刪除該變更的 touched 紀錄檔（若存在）。變更不存在時 SHALL 以非零 exit code 結束並於 stderr 說明。變更有動工痕跡——meta 含 started_at，或 tasks.md 有任何已勾任務——且未帶 --force 時 SHALL 拒絕：非零 exit code、stderr 提示動工痕跡與 --force，且 SHALL NOT 改動任何檔案；帶 --force 時 SHALL 照常執行。成功時 exit code 0，stdout 報告已刪除的變更名與每份解鏈討論的 slug 及回退後狀態（--no-color 下無 ANSI 色彩）；帶 --json 時 SHALL 輸出 camelCase payload：變更名與解鏈討論清單（各含 slug 與回退後狀態）。remote store 模式下 SHALL 以非零 exit code 於 stderr 報 discard 不支援。變更目錄刪除失敗時 SHALL 以非零 exit code 回報，已完成的討論解鏈不回滾且輸出 SHALL 明示已解鏈清單。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 未動工變更成功廢棄

- **WHEN** 對 meta 無 started_at 且 tasks.md 無已勾任務的變更執行 speclink discard
- **THEN** exit code 0；openspec/changes/<change>/ 目錄消失；stdout 報告已刪除的變更名與解鏈結果

#### Scenario: 動工痕跡守衛拒絕

- **WHEN** 對有動工痕跡的變更執行 speclink discard 且未帶 --force
- **THEN** 非零 exit code；stderr 提示動工痕跡與 --force；openspec/ 下任何檔案逐位元不變

##### Example: 動工痕跡判定

| meta 有 started_at | tasks.md 有已勾任務 | 未帶 --force 的結果 |
| ------------------ | ------------------- | ------------------- |
| 否                 | 否                  | 放行                |
| 是                 | 否                  | 拒絕                |
| 否                 | 是                  | 拒絕                |
| 是                 | 是                  | 拒絕                |

#### Scenario: --force 放行動過工的變更

- **WHEN** 對有動工痕跡的變更執行 speclink discard --force
- **THEN** exit code 0；變更目錄與其 touched 紀錄檔皆刪除；stdout 報告同成功路徑

#### Scenario: 變更不存在報錯

- **WHEN** 執行 speclink discard 給定不存在的變更名
- **THEN** 非零 exit code；stderr 說明變更不存在；無任何檔案變動

#### Scenario: remote store 模式不支援

- **WHEN** 於 remote store 綁定的專案執行 speclink discard
- **THEN** 非零 exit code；stderr 報 discard 不支援於 remote 模式；無任何檔案變動

#### Scenario: --json 輸出 payload

- **WHEN** 執行 speclink discard <change> --json 成功廢棄
- **THEN** stdout 為 JSON：含變更名欄位與解鏈討論陣列（每項含 slug 與回退後狀態），欄位名一律 camelCase

### Requirement: restale_from 記錄變更待重新反映的討論並經 CLI 觀測

變更 meta 檔（openspec/changes/<name>/.openspec.yaml）MAY 帶 restale_from 欄位——逗號分隔的討論 slug 清單，語意為「本變更曾反映（seal）這些討論，其後這些討論被重新結論，內容相對新結論過期、待 re-ingest」。ChangeMeta SHALL 提供 restale_from() accessor 回傳 Vec<String>：欄位缺席時回空、逗號值 SHALL 各段 trim 後分割，行為平行既有 from_discussion／from_discussions()。此欄位由 discuss conclude 寫入、discuss seal 清除（見 discussion-docs 正典），本需求規範其讀取與觀測。speclink show <change> --json SHALL 於變更 payload 恆曝 restaleFrom（camelCase 字串陣列，無旗標為空陣列），平行既有 fromDiscussions。speclink list --json SHALL 於 restale_from 非空的變更 payload 曝 restaleFrom 陣列、為空時省略該欄位——以維持 list --json 對無旗標變更的既有輸出逐位元不變。speclink analyze <change> 於某變更 restale_from 非空時 SHALL 出一條資訊性 finding，指明該變更反映的討論已重新結論、需重新 ingest 以同步新結論。此欄位讀取 SHALL 為零 per-load 掃描——僅讀既存 meta 欄位，不掃描討論記錄。

#### Scenario: restale_from() accessor 讀取

- **WHEN** 變更 meta 含 restale_from: alpha-search, beta-cache
- **THEN** ChangeMeta::restale_from() 回傳 ["alpha-search", "beta-cache"]；meta 無該欄位時回傳空 Vec

#### Scenario: show 恆曝 restaleFrom

- **WHEN** 對 restale_from 含 alpha-search 的變更、以及無該欄位的變更，各執行 speclink show <change> --json
- **THEN** 前者 payload 的 restaleFrom 為 ["alpha-search"]；後者 payload 的 restaleFrom 為空陣列（欄位恆存在）

#### Scenario: list 曝 restaleFrom 且對無旗標變更輸出不變

- **WHEN** 對含一個 restale_from 非空變更與一個無該欄位變更的專案執行 speclink list --json
- **THEN** 非空變更的 payload 含 restaleFrom 陣列（如 ["alpha-search"]）；無旗標變更的 payload 省略 restaleFrom 欄位，其 list --json 輸出與本變更前逐位元一致

#### Scenario: analyze 對過期變更出資訊性 finding

- **WHEN** 對 restale_from 非空的變更執行 speclink analyze <change>
- **THEN** 輸出含一條資訊性 finding，指明該變更反映的討論已重新結論、需 re-ingest；restale_from 為空時無此 finding，且 analyze 輸出與本變更前逐位元一致
