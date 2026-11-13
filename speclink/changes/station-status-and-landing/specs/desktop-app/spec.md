## ADDED Requirements

### Requirement: remote 專案的審查與驗證標示

桌面 app SHALL 為 remote 專案的變更呈現與本機專案相同的審查與驗證標示（看板卡片、詳情頁、已封存頁），資料取自線上回應的關卡欄位：state 為 inProgress → 審查中／驗證中；state 為 stamped 且 tasksTotal 與目前的任務數不同 → 已蓋章·其後有變動；state 為 stamped 且任務數相符 → 已蓋章；state 為 none 或沒有資料 → 無標示。remote 專案綁有 checkout 時，對任務數相符的已蓋章變更 SHALL 另以單筆回應的 scope 與 checkout 內檔案的現值比對內容指紋，不符者 SHALL 呈現為其後有變動；沒有 checkout 時 SHALL 只依任務數判定。這項比對 SHALL 只在 checkout 握有蓋章的那一版時進行——checkout 的 HEAD 包含章記下的 commit；章沒有記下 commit、或 checkout 不包含它（章蓋在別的分支或別台機器）時 SHALL 只依任務數判定，不因 checkout 的檔案不同而呈現為其後有變動。比對時，內容與記錄的指紋不同的檔案，若仍與章記下的 commit 當時的內容相同（當時不存在、現在也不存在亦同），SHALL NOT 算作其後有變動：章記下的是蓋章當下的 HEAD，被審的內容那時可能還沒有提交，checkout 只是還沒有那批內容。四態的判定 SHALL 與本機專案共用同一份實作。已封存頁 SHALL 依線上的結局欄位呈現已審查、曾審查未通過與對應的驗證標示。remote 專案的封存入口 SHALL 於目標變更為審查中或驗證中時出現與本機相同的三選項對話框；選擇照樣帶走時 SHALL 把對應的旗標帶進封存請求。Server 的回應沒有關卡欄位時 SHALL 維持沒有標示、封存不出現對話框。這些標示 SHALL 沿用既有的元件與配色。

#### Scenario: remote 看板顯示審查中

- **WHEN** remote 專案的變更 demo 在 Server 上有審查工單
- **THEN** demo 的卡片顯示審查中的標示

#### Scenario: 有 checkout 時偵測其後有變動

- **WHEN** remote 專案綁有 checkout，demo 的審查已蓋章且任務數相符，checkout 的 HEAD 包含章記下的 commit，checkout 內 scope 的一個檔案內容與記錄的指紋不同，也與該 commit 當時的內容不同
- **THEN** demo 的卡片顯示已審查·其後有變動

#### Scenario: 被審的內容蓋章時還沒有提交

- **WHEN** remote 專案綁有 checkout，demo 的審查已蓋章且任務數相符，checkout 的 HEAD 包含章記下的 commit，checkout 內 scope 的一個檔案內容與記錄的指紋不同，但與該 commit 當時的內容相同
- **THEN** demo 的卡片顯示已審查

#### Scenario: checkout 沒有蓋章的那一版

- **WHEN** remote 專案綁有 checkout，demo 的審查已蓋章且任務數相符，章記下的 commit 不在 checkout 的 HEAD 之內，checkout 內 scope 的檔案內容與記錄的指紋不同
- **THEN** demo 的卡片顯示已審查

#### Scenario: 沒有 checkout 時只看任務數

- **WHEN** remote 專案沒有 checkout，demo 的審查已蓋章且任務數相符
- **THEN** demo 的卡片顯示已審查

#### Scenario: 封存審查中的 remote 變更

- **WHEN** 對審查中的 remote 變更按下封存，並在對話框選擇照樣帶走
- **THEN** 封存請求帶上保留審查工單的旗標；封存後已封存頁顯示曾審查未通過

#### Scenario: 舊 Server 沒有欄位

- **WHEN** remote 專案連到回應不含關卡欄位的 Server
- **THEN** 卡片沒有審查與驗證的標示，封存不出現三選項對話框
