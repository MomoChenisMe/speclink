## ADDED Requirements

### Requirement: archive 與工單讀取端點回填完整結果

封存端點 SHALL 自引擎的封存結果回填擴充欄位：datedName、各 capability 的 added／modified／removed／renamed 計數、snapshotCreated、archivedDiscussions（封存時一併封存的來源討論清單）、evidenceRecorded。review 與 verify 的工單讀取端點 SHALL 回填 content 為 store 中工單文件的原文全文。討論結論端點 SHALL 回填 restaleFlagged 為引擎打回重收的變更名清單。討論轉出端點 SHALL NOT 回傳新變更的目錄位置——那是 store 端的檔案系統位置，對呼叫端無意義。上列端點的既有欄位、狀態碼與錯誤語意 SHALL 維持不變。

#### Scenario: 封存端點回填完整結果

- **WHEN** 對一筆就緒的變更呼叫封存端點且封存改動了規格
- **THEN** 200 回應含 datedName 與各 capability 的四項計數，其值與引擎在 store 中實際落地的封存效果一致

#### Scenario: 封存端點回填來源討論與證據旗標

- **WHEN** 封存一筆帶來源討論（該討論的衍生變更僅剩此筆）且無任務證據的變更
- **THEN** 回應的 archivedDiscussions 含該討論的 slug 與封存檔名，evidenceRecorded 為 false

#### Scenario: 工單讀取端點回填原文

- **WHEN** 對一筆存在工單的變更呼叫工單讀取端點（review 或 verify）
- **THEN** 200 回應的 content 等於該站工單文件的原文全文，rounds 與 lastRound 照舊在場

#### Scenario: 結論端點回填被打回的變更

- **WHEN** 對一份已轉出變更的討論呼叫結論端點，且該變更仍在進行中
- **THEN** 200 回應的 restaleFlagged 含該變更名

#### Scenario: 工單不存在時行為不變

- **WHEN** 對一筆無工單的變更呼叫工單讀取端點
- **THEN** 回應維持既有的 404 語意與錯誤形狀，不因欄位擴充而改變
