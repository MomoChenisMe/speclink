## ADDED Requirements

### Requirement: 任務完成蘊含開工標記

speclink task done 成功完成一項任務時，若該 change 的 .openspec.yaml 尚無 started_* 欄位，SHALL 於同一操作內蓋開工章：started_at 為當日 ISO 日期；started_by 依 git 身分可得性寫入——可歸屬者寫入、不可歸屬者缺席；本指令無 agent 識別來源，started_with 缺席。meta 既有欄位 SHALL 逐字元保留。該 change 已有 started_* 時 SHALL 保留首章不變。touched-files 記錄（.speclink/ 下）行為 SHALL 維持現行語意。

本需求為 parity 敏感：指令的人眼輸出、--json payload（change、status、taskDesc、taskId 對應之既有欄位形狀）與 exit code SHALL 與現行位元級一致（對 Spectra 2.3.1 的輸出 parity 不變）；.openspec.yaml 的開工章為刻意檔案效果分歧——自我基線的檔案樹對照 SHALL 隨本需求更新並記載此差異。錯誤路徑（tasks.md 缺失、任務序號無效、任務已完成）SHALL 維持現行訊息與非零 exit code，且 SHALL NOT 寫入任何檔案。

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
