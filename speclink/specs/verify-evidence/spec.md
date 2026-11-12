# verify-evidence Specification

## Purpose

實作證據的記錄與消費：task done 時把該任務可歸屬的 touched files 連同 actor、repo、head commit 與時間寫成逐任務證據，存於 change 目錄的 .evidence.json、隨 change 同生命週期；封存時則一律注入僅含 source 與 updated 兩欄的 trace 區塊。本 capability 保證正式規格需求能回溯到把它做出來的那個 change，而證據缺席不擋封存——只以一行提示點名。

## Requirements

### Requirement: task done 寫入逐任務 evidence

task done 成功且本次有可歸屬的 touched files（未被先前任務認領的新髒檔）時，SHALL 寫入逐任務證據記錄：task stable ID、actor（來自 ExecutionContext 的顯示身分）、repo（binding key）、head commit、touched files 與 recordedAt（UTC）；無新髒檔時 SHALL 沿現行 touched 語意不新增任何記錄。記錄 SHALL 寫入 change 目錄的 `.evidence.json`，隨 change 的提交、封存與廢棄同生命週期；讀取端 SHALL 於 change 目錄記錄缺席時回退舊路徑 `.speclink/touched/<change>.json`（僅供讀取，永不作為寫入目標），寫入 SHALL 一律落 change 目錄。記錄寫入新位置後、以及 change 封存或廢棄時，舊路徑檔案 SHALL 被移除——其內容已由回退讀取帶入新位置，殘留檔會被日後重用同一名稱的 change 誤讀為自己的記錄。記錄格式 SHALL 帶版本標記（v2）；讀取端 SHALL 相容無版本標記的既有格式（v1 檔案清單語意不變），並 SHALL 忽略既有記錄中的未知欄位（含先前版本寫入的 basisDigests）；commit 檔案歸屬依賴的檔案清單語意 SHALL 保留。entry SHALL NOT 含 basis digests——帳僅存有讀者的歷史事實。task undone SHALL NOT 寫入或修改任何證據記錄。

#### Scenario: 完成任務後證據齊全

- **WHEN** 於有 git 身分與髒檔的 workspace 執行 speclink task done tsk_ 某任務
- **THEN** 該 change 目錄的 `.evidence.json` 含該任務 entry：taskId 為該 tsk_ ID、actor 與 repo 非空、headCommit 為當前 HEAD、touchedFiles 含髒檔、recordedAt 為 UTC 時間戳；entry 無 basisDigests 欄位

#### Scenario: 舊格式記錄可讀

- **WHEN** change 目錄無 `.evidence.json` 而舊路徑存在 v1 或含 basisDigests 的 v2 記錄，執行任何讀取端（drift、commit 檔案歸屬）
- **THEN** 讀取回退舊路徑成功、v1 檔案清單語意不變、v2 的未知欄位被忽略且 all_files 不變；下一次 task done 寫入後新位置成為讀取來源且舊路徑檔案不再存在

#### Scenario: 證據隨 change 生命週期移動

- **WHEN** 對含 `.evidence.json` 的 change 執行封存或 discard
- **THEN** 封存後記錄位於封存 change 目錄內；discard 後記錄隨目錄一併消失；兩者皆不留下舊路徑的孤兒檔


<!-- @trace
source: evidence-home-and-trace-slim
updated: 2026-08-04
-->

---
### Requirement: archive trace 注入與零證據提示

archive 於 ADDED／MODIFIED 物化正式規格需求時 SHALL 一律注入 trace 區塊，內容 SHALL 僅含 source（change 名）與 updated（封存時戳）兩欄；updated SHALL 為帶時區偏移量的 RFC 3339 時戳、秒級（例 `2026-09-05T23:17:28+08:00`），且其日曆日 SHALL 與同一次封存的目錄名日期前綴相同（兩者取自同一個當下）。既有正式規格中純日期的 updated 行 SHALL NOT 被 archive 回改。trace 區塊 SHALL NOT 含檔案清單，SHALL NOT 掃描工作樹髒檔產生任何 trace 內容。archive SHALL NOT 因 evidence 缺席或內容拒絕封存。本機 fs 模式下，change 無任何 v2 evidence entry 時 CLI SHALL 於 stderr 印恰一行提示（點名 change 名、說明無任務證據記錄），exit code 與封存結果 SHALL 不受影響；有任一 entry 時 SHALL NOT 印出提示。引擎 SHALL 以結構化事實（ArchiveOutcome 的 evidence_recorded）回報有無證據，呈現歸呼叫端。遠端 store 模式不在本需求範圍：證據的記錄與提示應由 server 端自記自判（Phase 2 另行設計），wire 契約 SHALL NOT 為此攜帶欄位。

#### Scenario: trace 兩欄一律注入

- **WHEN** 對含 v2 evidence 的 change 執行 speclink archive
- **THEN** 封存後正式規格的 trace 區塊僅含 source 與 updated 兩行，updated 為 RFC 3339 時戳且前十字元等於封存目錄名的日期前綴，無 code 清單；evidence 記錄本身不受注入影響、隨目錄移入封存區

#### Scenario: 既有純日期 trace 不回改

- **WHEN** 封存一個 MODIFIED 既有需求的 change，該正式規格內其他需求的 trace 區塊 updated 為純日期
- **THEN** 只有被物化的需求得到新的 RFC 3339 時戳，其他需求的純日期 updated 行逐位元不變

#### Scenario: 零證據照常封存並提示

- **WHEN** 對無任何 v2 entry 的 change 執行 speclink archive
- **THEN** 封存成功、exit code 0，stderr 恰一行提示點名該 change 無任務證據記錄；正式規格 trace 照樣注入 source 與 updated 兩欄

#### Scenario: 有證據時一字不印

- **WHEN** 對含任一 v2 entry 的 change 執行 speclink archive
- **THEN** 封存成功且 stderr 無任何 evidence 相關提示


<!-- @trace
source: manual-stale-time-granularity
updated: 2026-09-07T12:40:29+08:00
-->

---
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

<!-- @trace
source: remote-task-evidence
updated: 2026-08-25
-->

---
### Requirement: evidence show 動詞

CLI SHALL 提供 `speclink evidence show <change>`（旗標 `--json` 與 `--no-color`，無 stdin）讀出一個變更的 evidence 記錄；它 SHALL 是唯讀查詢，SHALL NOT 寫入任何文件，SHALL NOT 產生領域事件。`<change>` 的解析 SHALL 依序：同名的進行中變更；否則已封存的變更中，名稱等於封存名稱、或等於封存名稱去掉開頭日期後的名稱者，有多筆時取最新的一筆；兩者皆無 SHALL 以非零 exit code 結束、stderr 為找不到的訊息、錯誤分類為 not_found。`<change>` 不是單一路徑段（含 `/` 或 `\`，或為 `.`、`..`、`archive`）時 SHALL 同樣視為找不到，SHALL NOT 查詢任何儲存。帶日期的封存名稱照原樣給定時 SHALL 讀該封存；只給變更名稱時「最新的一筆」SHALL 依封存名稱的日期判定，名稱不帶日期的舊封存排在帶日期的之後。變更存在但沒有任何記錄 SHALL 視為正常：exit code 0。evidence 的內容無法解析時 SHALL 以非零 exit code 回報錯誤，SHALL NOT 當成沒有記錄。

人眼輸出 SHALL 寫至 stdout：第一行含變更名稱與記錄的任務數；讀到的是已封存的變更時 SHALL 有一行指出封存名稱；之後每筆記錄一段，含任務識別、任務說明、記錄時間與它改到的每個檔案；沒有記錄時 SHALL 只印一行說明。`--no-color` 下 SHALL 僅省略色碼。`--json` 時 stdout SHALL 為單一 JSON 物件：`change`（字串）、`archivedAs`（字串，僅在讀到已封存的變更時出現）、`entries`（陣列，每項含 `taskId` 字串、`taskDesc` 字串、`touchedFiles` 字串陣列、`recordedAt` 字串，以及有值才出現的 `actor`、`repo`、`headCommit` 字串）、`files`（字串陣列：全部記錄的 touchedFiles 去重複後依位元組順序排序）。這個動詞 SHALL 經命令執行層，本機、remote 模式、Node SDK 的 dispatch 與宿主套件的兩個門對相同內容 SHALL 得到相同的輸出。

#### Scenario: 讀進行中變更的記錄

- **WHEN** 變更 demo 有兩筆記錄（任務 1.1 改到 src/a.rs 與 src/b.rs，任務 1.2 改到 src/b.rs 與 src/c.rs），執行 speclink evidence show demo --json
- **THEN** exit code 為 0；entries 有兩項，各含 taskId、taskDesc、touchedFiles、recordedAt；files 為 src/a.rs、src/b.rs、src/c.rs；沒有 archivedAs 欄位

#### Scenario: 人眼輸出

- **WHEN** 對同一個變更執行 speclink evidence show demo --no-color
- **THEN** stdout 第一行含 demo 與 2；之後有兩段，各含任務識別、說明、記錄時間與檔案路徑；輸出不含色碼

#### Scenario: 沒有記錄

- **WHEN** 變更 fresh 存在但沒有任何任務完成過，執行 speclink evidence show fresh 與加 --json 的同一指令
- **THEN** 兩次 exit code 皆為 0；人眼輸出是一行說明沒有記錄；--json 的 entries 與 files 是空陣列

#### Scenario: 以原本的名稱讀已封存變更的記錄

- **WHEN** 變更 demo 帶著兩筆記錄被封存為 2026-11-12-demo，之後執行 speclink evidence show demo --json
- **THEN** entries 是封存前的同兩筆；archivedAs 為 `2026-11-12-demo`

#### Scenario: 以封存名稱讀

- **WHEN** 執行 speclink evidence show 2026-11-12-demo --json
- **THEN** 結果與以 demo 讀取相同

#### Scenario: 同名的進行中變更優先

- **WHEN** 已封存 2026-11-01-demo，之後又建立一個新的進行中變更 demo（沒有記錄），執行 speclink evidence show demo --json
- **THEN** entries 是空陣列，沒有 archivedAs 欄位

#### Scenario: 找不到變更

- **WHEN** 進行中與已封存都沒有名為 ghost 的變更，執行 speclink evidence show ghost
- **THEN** exit code 非零，stderr 說明找不到 ghost，stdout 為空

#### Scenario: 名稱不是單一路徑段

- **WHEN** 執行 speclink evidence show .. 與 speclink evidence show archive/2026-11-12-demo
- **THEN** 兩次 exit code 皆非零，stderr 說明找不到該名稱，stdout 為空；規格資料夾與封存資料夾底下的檔案都沒有被讀出

#### Scenario: 不帶日期的舊封存不算最新

- **WHEN** 封存區同時有 demo（名稱不帶日期的舊封存）與 2026-11-12-demo，執行 speclink evidence show demo --json
- **THEN** archivedAs 為 `2026-11-12-demo`

#### Scenario: 壞的記錄不當成沒有記錄

- **WHEN** 變更 demo 的 evidence 文件不是合法的內容，執行 speclink evidence show demo
- **THEN** exit code 非零，stderr 說明記錄無法解析；stdout 沒有「沒有記錄」的說明

#### Scenario: remote 模式與本機同形

- **WHEN** server 與本機 fs 專案含相同的變更與記錄，分別於兩模式執行 speclink evidence show demo --json
- **THEN** 兩模式的 stdout 相同；remote 模式未讀取本機 store

#### Scenario: 經 SDK 讀取

- **WHEN** 以 host.dispatch 對含該變更的範圍執行 ['evidence', 'show', 'demo', '--json']
- **THEN** 回傳的物件與 CLI 的 --json 輸出深度相等；儲存的提交沒有被呼叫

<!-- @trace
source: remote-read-verbs
updated: 2026-11-12T15:18:56+08:00
-->