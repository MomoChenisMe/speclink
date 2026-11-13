# change-landing Specification

## Purpose

變更的分支與程式碼落地記錄：開工時記下這個變更所在的分支，並以 change landed 動詞回報它的程式碼落在哪個 commit。本 capability 只規定這些資料怎麼寫入與讀出；引擎不依它們做任何判斷，「落地後才能封存」這類規則屬於宿主。

## Requirements

### Requirement: 開工時記下分支

`speclink in-progress add` SHALL 接受由呼叫端提供的分支名稱，並在寫入開工標記的同一次寫入裡把它記進變更的 metadata。CLI SHALL 以 checkout 目前的分支名稱提供它；checkout 不在任何分支上（detached HEAD）或不是 git checkout 時 SHALL 不提供，開工標記照常寫入。專案資料夾自己不是 git repo、只是位於別的 repo 底下時，也算不是 git checkout；git 給的分支名稱過不了下述檢查（形狀不合，或 metadata 的格式存不了）時，CLI 同樣 SHALL 不提供（那不是使用者給的參數，動詞不因此失敗）。變更已有開工標記時 SHALL NOT 改寫已記下的分支。`speclink in-progress remove` SHALL 一併移除分支的記錄。分支名稱 SHALL 是 1 到 255 個字元、不含控制字元與空白，否則 SHALL 以非零 exit code 拒絕且不寫入。形狀合格、但 metadata 的格式存不了的名稱（寫入後讀不回同一個字串，例如含 U+FFFE）SHALL 同樣拒絕且不寫入。引擎 SHALL NOT 讀取 git；它 SHALL NOT 檢查分支是否存在。沒有提供分支時 SHALL NOT 寫入空值。

#### Scenario: 在功能分支上開工

- **WHEN** checkout 在分支 feature/x 上，對變更 demo 執行 speclink in-progress add demo
- **THEN** demo 的 metadata 含 `branch: feature/x` 與開工標記；exit code 為 0

#### Scenario: detached HEAD 不記分支

- **WHEN** checkout 處於 detached HEAD，執行 speclink in-progress add demo
- **THEN** 開工標記寫入，metadata 沒有 branch 鍵

#### Scenario: 已開工時不改分支

- **WHEN** demo 已有開工標記與 `branch: feature/x`，在另一個分支上再執行 speclink in-progress add demo
- **THEN** metadata 的 branch 仍是 feature/x

#### Scenario: 退回提案中移除分支

- **WHEN** 對已記下分支的 demo 執行 speclink in-progress remove demo
- **THEN** metadata 不再有 branch 鍵與開工標記

#### Scenario: 存不進 metadata 的分支名稱

- **WHEN** 呼叫端提供含 U+FFFE 的分支名稱
- **THEN** 以非零 exit code 拒絕；metadata 逐位元不變，仍可解析

#### Scenario: 專案位於別的 repo 底下

- **WHEN** 專案資料夾自己沒有 `.git`、位於另一個 git repo 的子資料夾，執行 speclink in-progress add demo
- **THEN** 開工標記寫入，metadata 沒有 branch 鍵

#### Scenario: 含斜線與中文的分支名稱

- **WHEN** 在分支 `功能/登入` 上執行 speclink in-progress add demo
- **THEN** metadata 的 branch 讀回來是 `功能/登入`，metadata 仍可解析


<!-- @trace
source: station-status-and-landing
updated: 2026-11-13T09:30:51+08:00
-->

---
### Requirement: change landed 動詞回報落地的 commit

CLI SHALL 提供 `speclink change landed <name> [--commit <sha>] [--branch <name>] [--json]`（另有 `--no-color`，無 stdin）。它 SHALL 把這個變更的程式碼落在哪個 commit 記進變更的 metadata：落地的 commit、回報的時間（RFC 3339）、回報的人（匿名時不記），以及有給 `--branch` 時覆寫分支的記錄。不帶 `--commit` 時 CLI SHALL 以 checkout 目前的 HEAD 提供；兩者都沒有時 SHALL 以非零 exit code 拒絕並說明要給 `--commit` 或在 git checkout 裡執行。commit SHALL 是 7 到 64 個十六進位字元，否則 SHALL 拒絕；引擎 SHALL NOT 檢查 commit 是否存在於任何 repo。變更不存在時 SHALL 以 not_found 拒絕；metadata 無法解析時 SHALL 拒絕且不寫入。寫入後的 metadata 讀不回這次回報的值時（手改過的 metadata 把鍵寫成文字手術認不得的形式，補寫會造成重複的鍵）SHALL 拒絕且不寫入。這個動詞 SHALL NOT 依變更的階段拒絕，也 SHALL NOT 改變變更的階段。重複回報 SHALL 覆寫先前的記錄。全部寫入 SHALL 是對 metadata 的一次寫入；任一守門失敗時 SHALL 零寫入。成功時 stdout SHALL 為一行 `✓ <name> landed at <commit 的前 12 個字元>`；`--json` 時 SHALL 為 `{ "change": string, "commit": string, "branch"?: string }`，commit 為完整的值。成功時命令層 SHALL 回報一筆 change-landed 領域事件，payload 含 change、commit 與（有時）branch。這個動詞 SHALL 經命令執行層，本機、remote 模式、Node SDK 的 dispatch 與宿主套件的兩個門對相同輸入 SHALL 得到相同的結果；宿主的執行前掛鉤 SHALL 能看到命令名稱 change-landed 與它的參數。封存時這些記錄 SHALL 隨 metadata 一併保留。

#### Scenario: 以目前的 HEAD 回報

- **WHEN** checkout 的 HEAD 是某個 40 字元的 commit，執行 speclink change landed demo
- **THEN** stdout 是 `✓ demo landed at ` 加該 commit 的前 12 個字元；demo 的 metadata 記下完整的 commit、回報時間與回報的人；exit code 為 0

#### Scenario: 指定 commit 與分支

- **WHEN** 執行 speclink change landed demo --commit 0123456789abcdef0123456789abcdef01234567 --branch speclink/demo --json
- **THEN** stdout 是含 change、完整 commit 與 branch 的 JSON 物件；metadata 的分支是 speclink/demo

#### Scenario: 重複回報覆寫

- **WHEN** demo 已記下一個落地的 commit，再以另一個 commit 執行 speclink change landed
- **THEN** metadata 只有新的 commit 與新的回報時間

#### Scenario: 沒有 commit 可用

- **WHEN** 在不是 git checkout 的資料夾執行 speclink change landed demo，不帶 --commit
- **THEN** exit code 非零，stderr 說明要給 --commit；metadata 逐位元不變

#### Scenario: commit 的形狀不合

- **WHEN** 執行 speclink change landed demo --commit not-a-sha
- **THEN** exit code 非零；metadata 逐位元不變

#### Scenario: 變更不存在

- **WHEN** 執行 speclink change landed ghost --commit 0123456
- **THEN** exit code 非零，stderr 說明找不到 ghost

#### Scenario: 手改的鍵讀不回時拒絕

- **WHEN** demo 的 metadata 有一行 `landed_commit : 0000000`（冒號前有空白），執行 speclink change landed demo --commit 0123456
- **THEN** exit code 非零；metadata 逐位元不變

#### Scenario: 提案中的變更也能回報

- **WHEN** demo 還沒有開工標記，執行 speclink change landed demo --commit 0123456
- **THEN** exit code 為 0；demo 仍然是提案中

#### Scenario: 經儲存接口是一次提交並帶事件

- **WHEN** 經宿主套件執行「demo 落地於某 commit」的命令
- **THEN** 儲存收到正好一次提交，內含 demo 的 metadata 寫入與一筆 change-landed 事件；執行前掛鉤曾收到命令名稱 change-landed 與含該 commit 的參數

#### Scenario: 掛鉤可以拒絕回報

- **WHEN** 執行前掛鉤對 change-landed 回答拒絕
- **THEN** 呼叫以 refused 失敗，metadata 沒有落地的記錄

#### Scenario: remote 模式與本機同形

- **WHEN** 分別於 fs 模式與 remote 模式對內容相同的變更執行 speclink change landed demo --commit 0123456789ab --json
- **THEN** 兩模式的 stdout 相同

#### Scenario: 封存後記錄仍在

- **WHEN** 已回報落地的 demo 被封存
- **THEN** 封存的 metadata 含落地的 commit、時間與分支


<!-- @trace
source: station-status-and-landing
updated: 2026-11-13T09:30:51+08:00
-->

---
### Requirement: 分支與落地記錄的讀取

`speclink show <change> --json` SHALL 增列選填的 `branch`（字串）與 `landed`（物件：`commit` 字串、`at` 字串、有值才出現的 `by` 字串）；沒有記錄時 SHALL 不輸出這兩個欄位。show 的人眼輸出 SHALL 維持既有的位元級輸出。`speclink list --json` SHALL NOT 因這些記錄改變形狀。

#### Scenario: show 帶出落地記錄

- **WHEN** demo 記有分支 feature/x 與落地的 commit，執行 speclink show demo --json
- **THEN** 輸出含 branch `feature/x` 與 landed 物件（commit 與 at）

#### Scenario: 沒有記錄時不輸出

- **WHEN** 對沒有分支與落地記錄的變更執行 speclink show --json
- **THEN** 輸出沒有 branch 與 landed 鍵，其餘欄位與既有行為相同

#### Scenario: list 不變

- **WHEN** 比對回報落地前後 speclink list --json 的輸出
- **THEN** 兩者的欄位集合相同；除了 `modifiedAt`（變更的文件確實被改寫了）以外，值逐位元相同

<!-- @trace
source: station-status-and-landing
updated: 2026-11-13T09:30:51+08:00
-->