## MODIFIED Requirements

### Requirement: 動詞人眼輸出的兩模式同形

<!-- BEFORE: 明文分歧清單只有五項，plan 沒有只限本機的旗標 -->

CLI 動詞的人眼輸出（stdout 文本，含 --no-color 模式）在本機與 remote 兩模式 SHALL 逐位元一致，僅下列明文分歧清單除外，清單外的任何輸出差異 SHALL 視為缺陷：

1. new change 的 Path 行——本機印、remote 不印（server 端路徑對本機使用者無意義）
2. list 的 worktree 標示——remote 恆缺席（worktree 是本機主 checkout 的觀察面）
3. status 的 schema 覆寫旗標——remote 以固定訊息明確拒絕（server 的 workflow config 決定 schema）
4. workflow-config 的文件標籤——remote 以 config.yaml 為標籤（server 端無本機路徑可印）
5. discuss promote 的 Path 行與其後的 propose 提示行——本機印、remote 不印（新變更目錄是 store 端的檔案系統位置，同第 1 項的裁定；兩行綁在一起去留）
6. plan 的 --strict-overlap 旗標——remote 以固定訊息明確拒絕，拒絕判定只解析模式、不發出任何 server 請求（server 只以 requirement 級重疊規劃，沒有目錄級開關）

同形範圍涵蓋 list、discuss 全部子指令、task done 與 task undone、in-progress remove、discard、archive、review 與 verify 的 add-round／stamp／discard／show。模式差異 SHALL 只存在於資料取得與守門拒絕，SHALL NOT 存在於輸出文本的組版。

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

#### Scenario: plan 的 --strict-overlap 於 remote 明確拒絕

- **WHEN** 於 remote 模式設定的專案執行 speclink plan --strict-overlap --json
- **THEN** exit code 非零，stderr 含 `plan --strict-overlap is not available in remote mode`，stdout 為空，且未發出任何 server 請求（連握手都不發）

### Requirement: 模式分岔的單點宣告

<!-- BEFORE: FsOnly 只有 demo、trace，plan 與 change 整個家族皆為 Dual -->

CLI 頂層動詞 SHALL 逐一歸屬四種模式形狀之一，本機（fs）/remote 的分岔決策 SHALL 集中於 dispatch 的宣告層，SHALL NOT 散佈於各動詞函式內：

- **ModeFree**（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion）：執行 SHALL NOT 觸發 store 模式解析，dispatch SHALL NOT 因宣告層而對其引入 .speclink.yaml 的解析失敗——不讀取專案設定的動詞（completion、config）SHALL 不受壞的 .speclink.yaml 影響；部分動詞（如 schemas、templates、update）的 workspace 探索本就讀取 .speclink.yaml 以解析 spec_dir，其於壞檔下的既有失敗行為維持不變；連線管理動詞（link、unlink、auth）的連線解析由動詞自理。
- **Dual**（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify、plan、change）：fs 模式 SHALL 作用於本機 store，remote 模式 SHALL 作用於 remote store，SHALL NOT 於 remote 模式靜默作用於本機 store；宣告 SHALL 同時載明本機臂與 remote 臂，缺任一臂 SHALL 構成建置失敗而非執行期靜默回退；不消費 store 的前置步驟（instructions 的 --skill 分流、workflow-config 的 argv／stdin 正規化）SHALL 先於模式解析執行，維持既有可觀察順序。
- **FsOnly**（demo、trace；以及 Dual 家族內只限本機的兩個子情形——change 的 rank 子指令、plan 的 --strict-overlap 旗標）：remote 模式 SHALL 以非零 exit code 明確拒絕，拒絕判定 SHALL 僅解析模式而不建立連線——SHALL NOT 發出任何 server 請求，離線環境同樣拒絕。子情形的歸屬 SHALL 同樣在 dispatch 的宣告層依子指令或旗標決定，同一動詞的其餘子指令與不帶該旗標的呼叫維持 Dual。
- **RemoteOnly**（claim）：fs 模式 SHALL 以非零 exit code 明確拒絕並於 stderr 說明需要 remote store。

模式判定 SHALL 惰性執行：僅於宣告形狀需要時解析模式，僅於 remote 臂將執行時建立連線。

#### Scenario: ModeFree 動詞不受壞連線設定影響

- **WHEN** 於 .speclink.yaml 內容無法解析的專案目錄下執行 speclink completion generate zsh 與 speclink config list
- **THEN** 兩者正常輸出且 exit code 為 0，stderr 不含 .speclink.yaml 的解析錯誤

#### Scenario: FsOnly 動詞於 remote 模式零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下執行 speclink demo
- **THEN** exit code 非零，stderr 說明該動詞僅限本機模式，且過程未發出任何 server 請求

#### Scenario: Dual 家族內的 FsOnly 子情形零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下分別執行 speclink change rank c --before a 與 speclink plan --strict-overlap
- **THEN** 兩者 exit code 非零，stderr 分別說明 change rank 與 plan --strict-overlap 僅限本機模式，且過程未發出任何 server 請求

#### Scenario: RemoteOnly 動詞於 fs 模式明確拒絕

- **WHEN** 於 fs 模式專案執行 speclink claim 指定 change
- **THEN** exit code 非零，stderr 說明該動詞需要 remote store

#### Scenario: plan 與 change depends 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink plan --json 與 speclink change depends add-b --on add-a
- **THEN** 兩者分別對 server 發出 GET /plan 與 POST /changes/add-b/depends，stdout 形狀與 fs 模式一致，未觸碰本機 store
