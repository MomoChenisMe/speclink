## ADDED Requirements

### Requirement: 模式分岔的單點宣告

CLI 頂層動詞 SHALL 逐一歸屬四種模式形狀之一，本機（fs）/remote 的分岔決策 SHALL 集中於 dispatch 的宣告層，SHALL NOT 散佈於各動詞函式內：

- **ModeFree**（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion）：執行 SHALL NOT 觸發 store 模式解析——壞的 .speclink.yaml SHALL NOT 影響其執行；連線管理動詞（link、unlink、auth）的連線解析由動詞自理。
- **Dual**（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify）：fs 模式 SHALL 作用於本機 store，remote 模式 SHALL 作用於 remote store，SHALL NOT 於 remote 模式靜默作用於本機 store；宣告 SHALL 同時載明本機臂與 remote 臂，缺任一臂 SHALL 構成建置失敗而非執行期靜默回退。
- **FsOnly**（demo）：remote 模式 SHALL 以非零 exit code 明確拒絕，拒絕判定 SHALL 僅解析模式而不建立連線——SHALL NOT 發出任何 server 請求，離線環境同樣拒絕。
- **RemoteOnly**（claim）：fs 模式 SHALL 以非零 exit code 明確拒絕並於 stderr 說明需要 remote store。

模式判定 SHALL 惰性執行：僅於宣告形狀需要時解析模式，僅於 remote 臂將執行時建立連線。

#### Scenario: ModeFree 動詞不受壞連線設定影響

- **WHEN** 於 .speclink.yaml 內容無法解析的專案目錄下執行 speclink schemas
- **THEN** 正常輸出且 exit code 為 0，stderr 不含 .speclink.yaml 的解析錯誤

#### Scenario: FsOnly 動詞於 remote 模式零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下執行 speclink demo
- **THEN** exit code 非零，stderr 說明該動詞僅限本機模式，且過程未發出任何 server 請求

#### Scenario: RemoteOnly 動詞於 fs 模式明確拒絕

- **WHEN** 於 fs 模式專案執行 speclink claim 指定 change
- **THEN** exit code 非零，stderr 說明該動詞需要 remote store
