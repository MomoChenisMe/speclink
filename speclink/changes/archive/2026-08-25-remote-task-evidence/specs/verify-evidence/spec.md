## ADDED Requirements

### Requirement: store 模式的 evidence 記錄與查詢

Engine 的 task done SHALL 以「Host 注入優先」取得 touched-file 候選：命令攜帶 Host 解析的候選清單時 SHALL 以之為準（server 路由自 wire 請求填入），未攜帶時 SHALL 沿本地 workspace 探測——本地模式的可觀察行為（.evidence.json 位置、v2 格式、寫入時機）SHALL 不變。歸屬過濾（僅未被先前任務認領的新髒檔）SHALL 維持單點實作，兩模式同一套。store 模式下 evidence SHALL 經 Store seam 寫入並與 tasks.md 勾選、task-completed 事件在同一 Unit of Work 原子 commit——任一步失敗整筆回退，SHALL NOT 留下半套狀態；task-completed 事件 payload SHALL 攜帶 touchedFiles（無候選時可缺席或為空，SHALL NOT 偽造）。store 模式的 evidence SHALL 隨 change 生命週期移動：封存後隨封存文件集保留、discard 隨 change 文件一併消失。無可歸屬候選時 SHALL 沿現行語意不新增任何記錄。

#### Scenario: 遠端 task done 攜檔案後 evidence 可查

- **WHEN** 遠端模式 CLI 於含新髒檔的 git checkout 執行 speclink task done
- **THEN** store 端該 change 的 evidence 記錄含該任務 entry（taskId、actor、repo、headCommit、touchedFiles、recordedAt），且 outbox 的 task-completed payload 攜帶相同 touchedFiles

#### Scenario: 無候選不新增 entry

- **WHEN** 遠端 task done 未攜帶 touchedFiles（或候選皆已被先前任務認領）
- **THEN** 不新增任何 evidence entry，evidence 查詢回空集合，任務勾選仍成功

#### Scenario: store 模式封存攜帶 evidence

- **WHEN** 對 store 模式下含 evidence 記錄的 change 執行封存
- **THEN** 封存文件集含 .evidence.json 且內容與封存前一致；對照組：discard 後該 change 的 evidence 隨文件一併消失
