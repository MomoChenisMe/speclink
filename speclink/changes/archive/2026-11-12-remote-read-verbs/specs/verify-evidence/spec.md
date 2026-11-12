## ADDED Requirements

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
