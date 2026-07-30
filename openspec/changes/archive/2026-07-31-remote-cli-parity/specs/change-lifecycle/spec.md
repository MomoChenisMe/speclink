## ADDED Requirements

### Requirement: in-progress 標記經 remote 通道寫入 server meta

remote 模式下 speclink in-progress add SHALL 路由至 server：started_at 與 started_by SHALL 以 server 端認證身分蓋進該 change 的 meta 文件（與 created_* 同一身分機制——可歸屬者寫入、不可歸屬者缺席）；started_with 維持缺席（CLI 現無 agent 識別來源，fs 與 remote 一致）。CLI 的 stdout、stderr 與 exit code SHALL 維持 parity 凍結形狀：首次蓋章、重複執行、change 不存在三種情形皆靜默成功（無輸出、exit 0）。change 不存在或已有 started_* 時 server SHALL NOT 寫入任何文件、SHALL NOT 發布事件、scope revision SHALL NOT 前進；實際蓋章時 SHALL 發布對應領域事件。變更清單摘要 SHALL 攜帶選填 startedAt 欄位（camelCase，None 缺席、缺席時反序列化為預設），值來自 change meta 的 started_at，供消費端做欄位推導。

#### Scenario: remote 蓋章帶認證歸屬

- **WHEN** 於 remote 模式以認證使用者 momo 對 server 上未開工的 change 執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端該 change 的 meta 含 started_at（ISO 日期）與 started_by（momo 的身分），既有欄位逐字元保留

#### Scenario: 不存在的 change 靜默成功且零寫入

- **WHEN** 於 remote 模式對 server 上不存在的 change 名稱執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端零文件寫入、零事件發布、scope revision 不前進

#### Scenario: startedAt 隨清單上 wire

- **WHEN** 已蓋開工章的 change 出現於 server 的變更清單回應（GET /changes）
- **THEN** 該清單項含 startedAt 欄位（camelCase）且值等於 meta 的 started_at；未開工的 change 清單項不含該欄位。CLI 的 speclink list --json 維持與 fs 模式同形（fs 清單項凍結不帶 started_*——verb-contract 的列舉形狀 parity 優先），欄位推導由桌面等消費端在 wire payload 上進行

##### Example: 清單項的 startedAt

| change meta | GET /changes 清單項 |
| ----------- | ------------------- |
| `started_at: 2026-07-30` | `{"name":"demo","completedTasks":0,"totalTasks":15,"startedAt":"2026-07-30"}` |
| （無 started_at） | `{"name":"demo","completedTasks":0,"totalTasks":15}`（無 startedAt 鍵） |
