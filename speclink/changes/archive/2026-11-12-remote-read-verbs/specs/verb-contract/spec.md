## ADDED Requirements

### Requirement: artifact cat 讀取已封存變更的文件

speclink artifact cat 的 --change 名稱沒有對應的進行中變更時，SHALL 改讀已封存的變更：名稱等於封存名稱、或等於封存名稱去掉開頭日期後的名稱者，有多筆時取最新的一筆——與 evidence show 動詞同一份解析。artifact 的識別沿用既有詞彙（proposal、design、tasks、specs/<capability>）。同名的進行中變更存在時 SHALL 讀進行中的那一份。兩者皆無、或該變更沒有這份文件時 SHALL 以非 0 exit code 與既有的語義化訊息結束。--change 的名稱不是單一路徑段（含 `/` 或 `\`，或為 `.`、`..`、`archive`）時 SHALL 視為找不到，SHALL NOT 查詢任何儲存。對進行中的變更，輸出 SHALL 維持既有的位元級輸出。remote 模式 SHALL 經既有的 artifact 端點得到相同的結果，SHALL NOT 讀取本機 store。

#### Scenario: 以原本的名稱讀已封存變更的提案

- **WHEN** 變更 demo 已封存為 2026-11-12-demo，執行 speclink artifact cat proposal --change demo
- **THEN** stdout 是該封存變更的提案全文，exit code 為 0

#### Scenario: 以封存名稱讀 delta spec

- **WHEN** 執行 speclink artifact cat specs/auth --change 2026-11-12-demo
- **THEN** stdout 是該封存變更對 auth 的 delta spec 全文

#### Scenario: 進行中的變更優先且輸出不變

- **WHEN** 已封存 2026-11-01-demo，另有進行中的變更 demo，執行 speclink artifact cat proposal --change demo
- **THEN** stdout 是進行中那一份提案，與新增這條需求前的輸出逐位元相同

#### Scenario: 兩邊都沒有

- **WHEN** 進行中與已封存都沒有名為 ghost 的變更，執行 speclink artifact cat proposal --change ghost
- **THEN** exit code 非 0，stderr 是既有的找不到訊息

#### Scenario: 名稱不是單一路徑段

- **WHEN** 執行 speclink artifact cat proposal --change archive/2026-11-12-demo
- **THEN** exit code 非 0，stderr 是找不到該變更的訊息，stdout 為空

#### Scenario: remote 模式讀已封存變更

- **WHEN** 於 remote 模式執行 speclink artifact cat proposal --change demo，demo 在 server 上已封存
- **THEN** stdout 與 fs 模式對相同內容的輸出相同

## MODIFIED Requirements

### Requirement: 動詞契約的涵蓋面與 payload 形狀
<!-- BEFORE: 本質本機動詞是 demo 與 trace，trace 於 remote 模式拒絕；list --json 的 change 條目沒有時間欄位 -->
<!-- REMOVED-SCENARIO: trace 於 remote 明確拒絕 -->

動詞契約 SHALL 涵蓋 RD 本地全流程：changes 列舉／讀取／建立／認領／歸檔／捨棄（discard，含 force 語意）／開工標記（in-progress add）、artifacts 讀取／寫入、tasks 勾選／取消勾選／搬移（1-based ordinal 定址與 before 側別）、validate 與 analyze 衍生查詢、instructions 計算、discussions 全動詞（列舉、建立含 slug 覆寫、context、add-round、conclude、archive、promote、discard 含 force 語意、link、seal）、政策讀取（workflow-config 有效值）、詞彙讀取（LANGUAGE 內容）、正式規格讀取、evidence 讀取（evidence show）、溯源（trace）、執行計畫（plan，含 --strict-overlap）、身分查驗。remote 模式下對應 CLI 指令的 stdout 輸出（人眼與 --json）形狀 SHALL 與 fs 模式一致（欄位 camelCase 同名）；validate 的聚合語意（無參數／--all／--changes）SHALL 由 client 以逐 change 端點呼叫組合，聚合輸出形狀與 fs 模式一致；show SHALL 由 client 以讀取端點組合出與 fs 模式同形輸出，SHALL NOT 於 remote 模式讀取本機 store。本質本機動詞（demo）於 remote 模式 SHALL 明確拒絕（非零 exit code、stderr 說明僅限本機模式），SHALL NOT 靜默作用於本機 store，SHALL NOT 發出任何 server 請求。trace 於 remote 模式 SHALL 由 server 以同一份引擎實作組裝溯源鏈，CLI 的 stdout（人眼與 --json）SHALL 與 fs 模式對同一內容一致，SHALL NOT 讀取本機 store。契約的端點、payload 與錯誤形狀 SHALL 以 docs/verb-contract.md 為正典參考文件。

list --json 的 change 條目得含本機觀察面的可空欄位 worktree（物件：path 字串與 branch 字串）——該欄位僅於 fs 模式的主 checkout、worktree 政策開啟且映射成立時出現，缺席時 SHALL NOT 序列化；remote 模式的 list 條目 SHALL 恆缺席此欄位。形狀一致性 SHALL 以「可空且缺席不序列化」維持：無 worktree 情境下兩模式的 list 輸出逐欄位一致。

list --json 的 change 條目 SHALL 含 `created`（字串，`YYYY-MM-DD` 的建立日期；metadata 沒有或無法解析時缺席；metadata 記的值不是日期時照原樣輸出，與 speclink show 相同，SHALL NOT 改寫或捨棄）與 `modifiedAt`（字串，RFC 3339 的 UTC 時間，該 change 底下的文件最後一次被修改的時間；儲存答不出時缺席，SHALL NOT 以 0 或空字串代替）。兩個欄位在兩種模式 SHALL 同名同型；人眼輸出 SHALL NOT 因這兩個欄位而改變。

#### Scenario: remote 列舉輸出形狀與 fs 一致

- **WHEN** 於 remote 模式執行 speclink list --json，server 回傳兩個 change
- **THEN** stdout 的 JSON 欄位名與 fs 模式的 speclink list --json 完全一致，exit code 為 0

#### Scenario: instructions 由 server 計算

- **WHEN** 於 remote 模式執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 含 context、rules、template、locale 等欄位（值來自 server 端政策），欄位名與 fs 模式一致

#### Scenario: remote validate 輸出形狀與 fs 一致

- **WHEN** 於 remote 模式執行 speclink validate --json（無參數聚合）
- **THEN** stdout 為與 fs 模式同形的逐 change results JSON（欄位名一致），有任一 invalid 時 exit code 非 0

#### Scenario: remote discard 的 guard 語意與本地一致

- **WHEN** 於 remote 模式對已勾選任務的 change 執行 speclink discard（無 --force）
- **THEN** exit code 非 0，stderr 語義化訊息說明已開工需 --force（與 fs 模式同語意），server 上該 change 完整保留

#### Scenario: remote 建立討論帶 slug 覆寫

- **WHEN** 於 remote 模式以中文主題執行 speclink discuss new 並帶 --slug board-search-bar
- **THEN** server 端以 board-search-bar 建立記錄，stdout 與 --json 的 slug 欄位形狀與 fs 模式一致；非法 slug 值時 exit code 非 0、stderr 說明原因、server 端不落檔

#### Scenario: remote show 輸出與 fs 一致

- **WHEN** 同一份 change 內容分別存在於 fs 專案與 remote server，兩模式各執行 speclink show 該 change
- **THEN** 兩者的人眼輸出與 --json 逐欄位一致；remote 模式的結果來自 server 資料，未讀取本機 store

#### Scenario: remote in-progress add 蓋章於 server

- **WHEN** 於 remote 模式對 server 上存在的 change 執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端該 change 的 meta 含 started_at 與 started_by（server 認證身分）

#### Scenario: demo 於 remote 明確拒絕

- **WHEN** 於 remote 模式執行 speclink demo
- **THEN** exit code 非 0，stderr 說明 demo 僅限本機模式，本機與 server 均未新增任何 change

#### Scenario: remote list 恆無 worktree 欄位

- **WHEN** 於 remote 模式執行 speclink list --json，而本機恰有合乎慣例的 linked worktree
- **THEN** 所有條目均無 worktree 欄位，欄位名與 fs 模式無 worktree 情境完全一致

#### Scenario: trace 於 remote 走 server

- **WHEN** server 的某個 capability 有兩個已封存的變更動過它，於 remote 模式執行 speclink trace 該 capability，再執行同一指令加 --json
- **THEN** 兩次 exit code 皆為 0；人眼輸出與 --json 和 fs 模式對相同內容的輸出逐位元相同；未讀取本機 store

#### Scenario: remote trace 找不到 capability

- **WHEN** 於 remote 模式執行 speclink trace 一個不存在的 capability
- **THEN** exit code 非 0，stderr 是與 fs 模式相同的找不到訊息與近似建議

#### Scenario: list --json 帶出時間欄位

- **WHEN** 於 fs 模式對一個 2026-11-01 建立的變更執行 speclink list --json
- **THEN** 該條目的 created 為 `2026-11-01`，modifiedAt 是一個 RFC 3339 的 UTC 時間字串；speclink list 的人眼輸出與新增欄位前逐位元相同

#### Scenario: 勾選任務後 modifiedAt 不倒退

- **WHEN** 記下某個變更的 modifiedAt，對它執行 speclink task done，再執行 speclink list --json
- **THEN** 該條目新的 modifiedAt 不早於先前的值

#### Scenario: 儲存答不出最後修改時間

- **WHEN** 於 remote 模式連到一個不回報文件最後修改時間的 server，執行 speclink list --json
- **THEN** 各條目沒有 modifiedAt 欄位，created 照常存在，exit code 為 0

### Requirement: 動詞人眼輸出的兩模式同形
<!-- BEFORE: 分歧清單有六項，第六項是 plan --strict-overlap 於 remote 拒絕 -->
<!-- REMOVED-SCENARIO: plan 的 --strict-overlap 於 remote 明確拒絕 -->

CLI 動詞的人眼輸出（stdout 文本，含 --no-color 模式）在本機與 remote 兩模式 SHALL 逐位元一致，僅下列明文分歧清單除外，清單外的任何輸出差異 SHALL 視為缺陷：

1. new change 的 Path 行——本機印、remote 不印（server 端路徑對本機使用者無意義）
2. list 的 worktree 標示——remote 恆缺席（worktree 是本機主 checkout 的觀察面）
3. status 的 schema 覆寫旗標——remote 以固定訊息明確拒絕（server 的 workflow config 決定 schema）
4. workflow-config 的文件標籤——remote 以 config.yaml 為標籤（server 端無本機路徑可印）
5. discuss promote 的 Path 行與其後的 propose 提示行——本機印、remote 不印（新變更目錄是 store 端的檔案系統位置，同第 1 項的裁定；兩行綁在一起去留）

同形範圍涵蓋 list、plan（含 --strict-overlap）、trace、evidence show、discuss 全部子指令、task done 與 task undone、in-progress remove、discard、archive、review 與 verify 的 add-round／stamp／discard／show。模式差異 SHALL 只存在於資料取得與守門拒絕，SHALL NOT 存在於輸出文本的組版。

#### Scenario: list 的 invalid 標記兩模式同形

- **WHEN** 專案含一筆 metadata 損壞的變更，分別於本機與 remote 模式執行 list
- **THEN** 兩模式 stdout 均在該變更行尾渲染 invalid 標記，整段文本逐位元一致，exit code 均為 0

#### Scenario: discuss 動詞成功訊息兩模式同形

- **WHEN** 分別於本機與 remote 模式執行 discuss 子指令（例如 new、add-round、conclude、archive）
- **THEN** 兩模式的成功訊息文本逐位元一致，exit code 一致

#### Scenario: 封存與工單閱讀對新 server 同形

- **WHEN** remote server 為新版，於 remote 模式執行 archive 與 review show
- **THEN** archive 的 stdout 含封存目的地（dated 名稱）、規格計數行、封存討論行，與本機同文本；review show 的 stdout 印出工單文件原文全文，與本機同文本

#### Scenario: 對舊 server 整體退化

- **WHEN** remote server 為舊版（回應缺新欄位），於 remote 模式執行 archive 與 review show
- **THEN** 兩指令輸出整體退回既有 remote 輸出（簡短封存行、結構化工單摘要），exit code 0，SHALL NOT 出現新舊欄位混合的部分渲染

#### Scenario: plan 的 --strict-overlap 於 remote 同形

- **WHEN** server 與本機 fs 專案含相同的兩個變更（皆含同一 capability 的 delta、動到不同 requirement），分別於兩模式執行 speclink plan --strict-overlap 與加 --json 的同一指令
- **THEN** 兩模式的 stdout 逐位元相同，兩個變更被排進不同的波次；remote 模式未讀取本機 store

#### Scenario: evidence show 兩模式同形

- **WHEN** server 與本機 fs 專案含相同的變更與相同的 evidence 記錄，分別於兩模式執行 speclink evidence show 該變更 --no-color
- **THEN** 兩模式的 stdout 逐位元相同

### Requirement: 模式分岔的單點宣告
<!-- BEFORE: FsOnly 是 demo、trace，以及 plan 的 --strict-overlap 這個子情形 -->
<!-- REMOVED-SCENARIO: Dual 家族內的 FsOnly 子情形零請求拒絕 -->

CLI 頂層動詞 SHALL 逐一歸屬四種模式形狀之一，本機（fs）/remote 的分岔決策 SHALL 集中於 dispatch 的宣告層，SHALL NOT 散佈於各動詞函式內：

- **ModeFree**（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion）：執行 SHALL NOT 觸發 store 模式解析，dispatch SHALL NOT 因宣告層而對其引入 .speclink.yaml 的解析失敗——不讀取專案設定的動詞（completion、config）SHALL 不受壞的 .speclink.yaml 影響；部分動詞（如 schemas、templates、update）的 workspace 探索本就讀取 .speclink.yaml 以解析 spec_dir，其於壞檔下的既有失敗行為維持不變；連線管理動詞（link、unlink、auth）的連線解析由動詞自理。
- **Dual**（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify、plan、change、trace、evidence）：fs 模式 SHALL 作用於本機 store，remote 模式 SHALL 作用於 remote store，SHALL NOT 於 remote 模式靜默作用於本機 store；宣告 SHALL 同時載明本機臂與 remote 臂，缺任一臂 SHALL 構成建置失敗而非執行期靜默回退；不消費 store 的前置步驟（instructions 的 --skill 分流、workflow-config 的 argv／stdin 正規化）SHALL 先於模式解析執行，維持既有可觀察順序。
- **FsOnly**（demo）：remote 模式 SHALL 以非零 exit code 明確拒絕，拒絕判定 SHALL 僅解析模式而不建立連線——SHALL NOT 發出任何 server 請求，離線環境同樣拒絕。Dual 家族內 SHALL NOT 有只限本機的子指令或旗標。
- **RemoteOnly**（claim）：fs 模式 SHALL 以非零 exit code 明確拒絕並於 stderr 說明需要 remote store。

模式判定 SHALL 惰性執行：僅於宣告形狀需要時解析模式，僅於 remote 臂將執行時建立連線。

#### Scenario: ModeFree 動詞不受壞連線設定影響

- **WHEN** 於 .speclink.yaml 內容無法解析的專案目錄下執行 speclink completion generate zsh 與 speclink config list
- **THEN** 兩者正常輸出且 exit code 為 0，stderr 不含 .speclink.yaml 的解析錯誤

#### Scenario: FsOnly 動詞於 remote 模式零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下執行 speclink demo
- **THEN** exit code 非零，stderr 說明該動詞僅限本機模式，且過程未發出任何 server 請求

#### Scenario: RemoteOnly 動詞於 fs 模式明確拒絕

- **WHEN** 於 fs 模式專案執行 speclink claim 指定 change
- **THEN** exit code 非零，stderr 說明該動詞需要 remote store

#### Scenario: plan 與 change depends 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink plan --json 與 speclink change depends add-b --on add-a
- **THEN** 兩者分別對 server 發出 GET /plan 與 POST /changes/add-b/depends，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: change rank 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink change rank c --before a
- **THEN** 對 server 發出 POST /changes/c/rank，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: trace 與 evidence show 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink trace 某 capability 與 speclink evidence show 某 change
- **THEN** 兩者分別對 server 發出 GET /trace/{capability} 與 GET /changes/{name}/evidence，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: plan --strict-overlap 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink plan --strict-overlap --json
- **THEN** 對 server 發出帶 strictOverlap=true 的 GET /plan，stdout 形狀與 fs 模式一致

### Requirement: 動詞 --json 輸出形狀凍結
<!-- BEFORE: 刻意的契約更新只有 manual-task-marker 系列的兩次 -->

動詞的 --json 輸出欄位集合與 camelCase 命名 SHALL 維持既有契約不變;工單原文 SHALL NOT 出現在任何 --json 輸出。manual-task-marker 系列的兩次刻意契約更新:instructions apply 的任務項增列 `manual` 欄位、progress 增列 `codeTotal`/`codeComplete`/`codeRemaining`;隨 `[P]` 語意移除,任務項的 `parallel` 欄位 SHALL 自基線移除——任務項欄位集合為 `id`/`description`/`done`/`manual`。另一次刻意契約更新:list 的 change 條目增列 `created` 與 `modifiedAt` 兩個可缺席的欄位。除上述刻意更新外 SHALL NOT 改名或移除既有欄位;更新後的欄位集合即凍結契約的新基線。

#### Scenario: 工單 --json 兩模式同形且無原文欄位

- **WHEN** 分別於本機與 remote 模式執行 review show --json
- **THEN** 兩模式 payload 欄位集合一致——change、rounds、lastRound,rounds 各項含 index、phase、patchHash、scope、findings——且不存在攜帶工單原文的欄位

#### Scenario: instructions apply 兩模式同形且無 parallel 欄位

- **WHEN** 分別於本機與 remote 模式對同一 change 執行 instructions apply --json
- **THEN** 兩模式任務項欄位集合皆為 id/description/done/manual(無 parallel),progress 皆含 total/complete/remaining 與 codeTotal/codeComplete/codeRemaining,欄位集合一致

#### Scenario: list 新增欄位不影響既有欄位

- **WHEN** 比對新增欄位前後 speclink list --json 對同一專案的輸出
- **THEN** 既有欄位的名稱、型別與值不變，差異只有每個 change 條目多出的 created 與 modifiedAt
