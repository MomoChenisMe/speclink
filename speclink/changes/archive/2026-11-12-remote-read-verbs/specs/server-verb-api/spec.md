## ADDED Requirements

### Requirement: 溯源與嚴格重疊的唯讀端點

server SHALL 提供溯源的唯讀端點 GET /trace/{capability}：以引擎的溯源命令組裝該 capability 的鏈並回傳，viewer 以上角色可讀，回應帶範圍的 ETag；capability 不存在時 SHALL 回應 not_found，訊息 SHALL 含引擎的近似建議。GET /plan SHALL 接受選填的查詢參數 `strictOverlap`：值為 `true` 時 SHALL 以引擎的嚴格重疊命令計算並以既有的 plan 回應形狀回傳，且回應 SHALL 增列 `strictOverlap` 為 `true`，讓用戶端分得出「這是嚴格重疊的計畫」與「server 忽略了這個參數」；缺席或為 `false` 時 SHALL 維持既有行為；其他值 SHALL 回應 invalid_argument。嚴格重疊的計算 SHALL 使用與一般 plan 相同的順序來源；依賴成環時 SHALL 與一般 plan 相同地以 refused 回應並轉發引擎訊息。兩者 SHALL NOT 寫入任何文件，SHALL NOT 受寫入能力限制，SHALL NOT 產生事件。

#### Scenario: 溯源端點回傳鏈

- **WHEN** 某 capability 有兩個已封存的變更動過它，以 viewer 身分請求 GET /trace/該 capability
- **THEN** 回應是 200，payload 依封存日期由舊至新列出兩個變更，各帶封存名稱與來源討論；回應帶 ETag

#### Scenario: 溯源端點找不到 capability

- **WHEN** 請求 GET /trace/不存在的名稱
- **THEN** 回應是 404、reason 為 not_found，訊息含近似名稱的建議

#### Scenario: 嚴格重疊的 plan

- **WHEN** 兩個變更皆含同一 capability 的 delta 但動到不同 requirement，分別請求 GET /plan 與 GET /plan?strictOverlap=true
- **THEN** 前者把兩個變更排在同一波，後者排在不同的波次；後者的回應另帶 strictOverlap 為 true，前者沒有這個鍵，其餘欄位集合相同

#### Scenario: strictOverlap 的值不合法

- **WHEN** 請求 GET /plan?strictOverlap=maybe
- **THEN** 回應是 400、reason 為 invalid_argument

#### Scenario: 嚴格重疊下依賴成環

- **WHEN** 兩個變更互相宣告依賴，請求 GET /plan?strictOverlap=true
- **THEN** 回應是 409、reason 為 refused，訊息以 `dependency cycle:` 開頭

## MODIFIED Requirements

### Requirement: task done 消費 touchedFiles 且 evidence 有唯讀端點
<!-- BEFORE: evidence 端點只讀進行中變更的記錄；不存在的變更也回空集合 -->

task done 端點 SHALL 消費請求 payload 的 touchedFiles 並作為 Host 解析的候選交給 Engine，SHALL NOT 丟棄；payload 未攜帶 touchedFiles 時 SHALL 視為無候選（沿無新髒檔語意），SHALL NOT 視為錯誤。server SHALL 提供該 change 的 evidence 唯讀端點：viewer 以上角色可讀，回應欄位為 camelCase 的 evidence 記錄集合；記錄缺席 SHALL 回空集合而非 not_found——缺席是正常狀態，SHALL NOT 讓讀取端以錯誤碼區分「change 存在但無 evidence」。端點 SHALL 以與 evidence show 動詞相同的引擎命令回答，名稱的解析 SHALL 相同：同名的進行中變更優先，否則最新的同名已封存變更；讀到已封存的變更時回應 SHALL 增列 `archivedAs`（字串，封存名稱）。每個成功的回應 SHALL 帶 `change`（字串：這份記錄所屬變更自己的名稱——進行中變更的名稱，或已封存變更的封存名稱去掉開頭日期後的名稱），讓用戶端分得出「以引擎命令回答的 server」與「只讀進行中變更的舊版 server」。進行中與已封存都沒有這個名稱時 SHALL 回應 not_found。

#### Scenario: task done 攜檔案後 evidence 端點可讀回

- **WHEN** 以 editor 角色對某任務執行 task done 且 payload 攜帶 touchedFiles，隨後以 viewer 角色讀取該 change 的 evidence 端點
- **THEN** 回應含該任務 entry，touchedFiles 與 payload 一致，欄位為 camelCase

#### Scenario: 無 evidence 回空集合

- **WHEN** 對存在但從未落 evidence 的 change 讀取 evidence 端點
- **THEN** 回應為成功的空集合，非 not_found；回應的 change 是該 change 的名稱

#### Scenario: evidence 端點讀已封存的變更

- **WHEN** 變更 demo 帶著記錄被封存後，請求 demo 的 evidence 端點
- **THEN** 回應含封存前的記錄集合、archivedAs（封存名稱）與 change（demo）

#### Scenario: 不存在的變更回 not_found

- **WHEN** 請求一個進行中與已封存都沒有的名稱的 evidence 端點
- **THEN** 回應是 404、reason 為 not_found
