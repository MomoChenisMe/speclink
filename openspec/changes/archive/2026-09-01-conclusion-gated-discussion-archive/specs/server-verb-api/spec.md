## MODIFIED Requirements

### Requirement: archive 與工單讀取端點回填完整結果

<!-- BEFORE: 討論結論端點僅回填 restaleFlagged，無順手封存事實可觀察 -->

封存端點 SHALL 自引擎的封存結果回填擴充欄位：datedName、各 capability 的 added／modified／removed／renamed 計數、snapshotCreated、archivedDiscussions（封存時一併封存的來源討論清單——僅含 Conclusion 已寫入而實際隨行封存者）、evidenceRecorded。review 與 verify 的工單讀取端點 SHALL 回填 content 為 store 中工單文件的原文全文。討論結論端點 SHALL 回填 restaleFlagged 為引擎打回重收的變更名清單，且於引擎順手封存討論（閉環觸發）時 SHALL 回填 autoArchived: true（camelCase 布林、未觸發時省略鍵）。討論轉出端點 SHALL NOT 回傳新變更的目錄位置——那是 store 端的檔案系統位置，對呼叫端無意義。上列端點的既有欄位、狀態碼與錯誤語意 SHALL 維持不變。

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

#### Scenario: 討論結論端點回填順手封存事實

- **WHEN** 對閉環條件成立（promoted_to 非空且全數轉出變更已封存）的討論呼叫討論結論端點，再對閉環條件不成立的討論呼叫同端點
- **THEN** 前者 200 回應含 autoArchived: true 且該討論自 live 清單消失、出現於封存清單；後者回應無 autoArchived 鍵、討論維持於 live 清單

## ADDED Requirements

### Requirement: 討論列表回應攜帶 concluded

GET /discussions 的每筆討論 SHALL 由 server 於 route 邊緣以引擎的結論查詢組裝 concluded 欄位（camelCase 布林、恆填 true 或 false——true 即該討論的 Conclusion 段已寫入內文，scaffold 佔位註解不算）；引擎的討論列表結構與 CLI 的 discuss list --json 輸出 SHALL 維持逐位元不變。查詢失敗的單筆討論 SHALL 以欄位缺席容錯、列表不失敗。

#### Scenario: 已結論與未結論討論的列表欄位

- **WHEN** scope 內有一筆已寫入結論的 promoted 討論與一筆 Conclusion 仍為佔位註解的 promoted 討論，呼叫 GET /discussions
- **THEN** 前者含 concluded: true、後者含 concluded: false；本地 CLI 的 discuss list --json 輸出與改動前逐位元相同
