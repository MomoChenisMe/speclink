## ADDED Requirements

### Requirement: 封存回應的完整結果欄位

封存端點的回應 payload SHALL 增列下列欄位（camelCase，皆帶序列化預設值以向後相容）：datedName（字串，選填——封存目的地的 dated 名稱）、specs 清單各項的 added／modified／removed／renamed（整數，預設 0）、snapshotCreated（布林，選填）、archivedDiscussions（物件清單，各項含 slug 與 file 字串，預設空清單）、evidenceRecorded（布林，選填）。datedName 是新舊 server 的哨兵欄位：讀取方 SHALL 以其在場與否判定回應是否攜帶完整封存結果。

#### Scenario: 缺新欄位的回應可反序列化

- **WHEN** 以不含任何新欄位的既有形狀 JSON（僅 specs 清單）反序列化封存回應
- **THEN** 反序列化成功，datedName 缺席、各計數為 0、archivedDiscussions 為空清單——與既有 server 的回應相容

#### Scenario: 新 server 回應攜帶完整結果

- **WHEN** 新版 server 完成一筆會改動規格與封存來源討論的封存
- **THEN** 回應含 datedName、各 capability 的四項計數、archivedDiscussions 清單與 evidenceRecorded

### Requirement: 開工標記移除回應的移除旗標

開工標記移除端點 SHALL 有具名回應型別，攜帶 removed 欄位（布林，帶序列化預設值 true）——區分實際移除與「本來就沒開工」的冪等 no-op，兩者的人眼輸出是不同的行。缺席讀作 true：既有 server 的裸確認回應對呼叫端一律代表已移除，語意不變，因此不需哨兵欄位。

#### Scenario: 缺欄位的移除回應可反序列化

- **WHEN** 以空物件反序列化開工標記移除回應
- **THEN** 反序列化成功且 removed 為 true——與既有 server 的裸確認回應同義

#### Scenario: 冪等 no-op 可辨識

- **WHEN** 以 removed 為 false 的 JSON 反序列化開工標記移除回應
- **THEN** removed 讀出 false，呼叫端得以印出「本來就沒開工」的行

### Requirement: 工單回應的原文欄位

review 與 verify 兩站共用的工單讀取回應 SHALL 增列 content 欄位（字串，選填，帶序列化預設值）攜帶工單文件原文全文。content 是工單人眼輸出的哨兵欄位：讀取方 SHALL 以其在場與否判定 server 新舊，缺席時 SHALL 退回既有的結構化摘要呈現。

### Requirement: 討論結論回應的重收清單

討論結論端點 SHALL 有具名回應型別，攜帶 restaleFlagged 欄位（字串清單，帶序列化預設值）——re-conclude 打回重收的變更名。空清單即「無變更被打回」，與既有 server 不回報此事實時的讀取結果相同，因此不需哨兵欄位。

#### Scenario: 缺欄位的結論回應可反序列化

- **WHEN** 以空物件反序列化討論結論回應
- **THEN** 反序列化成功且 restaleFlagged 為空清單

#### Scenario: 結論回應攜帶被打回的變更名

- **WHEN** 以含 restaleFlagged 兩筆變更名的 JSON 反序列化討論結論回應
- **THEN** 兩筆變更名依序讀出

#### Scenario: 缺 content 的工單回應可反序列化

- **WHEN** 以不含 content 的既有形狀 JSON 反序列化工單讀取回應
- **THEN** 反序列化成功且 content 缺席——與既有 server 的回應相容

#### Scenario: 新 server 工單回應攜帶原文

- **WHEN** 新版 server 回應一筆存在工單的變更的工單讀取請求
- **THEN** content 等於 store 中工單文件的原文全文，結構化欄位（rounds、lastRound）同時照舊在場
