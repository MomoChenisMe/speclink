## MODIFIED Requirements

### Requirement: store 文件讀取動詞
<!-- BEFORE: 只規定閱讀指示用 artifact cat 與 language show；沒有規定正式規格的讀法與改寫的做法 -->

CLI SHALL 提供雙模式的文件讀取動詞：speclink artifact cat <artifact> --change <name> 輸出該 artifact 的內容；speclink language show 輸出共用詞彙文件內容（fs 模式讀本地檔、remote 模式經契約端點）；文件不存在時 SHALL 以非 0 exit code 與語義化訊息結束。生成技能資產中的文件閱讀指示 SHALL 使用這些動詞，SHALL NOT 指示直接讀取規格目錄下的檔案路徑。讀取正式規格的指示 SHALL 以 speclink show <capability> --item-type spec 表述（不指定類型時，同名的作用中變更優先），SHALL NOT 指示以 speclink list --specs --json 回傳的 path 欄位開啟檔案——該欄位在 remote 模式是儲存的鍵，不是本機路徑。discuss、propose、ingest 三支技能中改寫既有 artifact 的指示 SHALL 以「speclink artifact cat 讀出全文、修改後以 speclink new artifact 帶 --stdin --force 整份寫回」表述，SHALL NOT 留下未指名動詞的讀取或改寫步驟。

#### Scenario: artifact cat 於兩模式輸出一致
- **WHEN** 分別於 fs 模式與 remote 模式（server 存有相同內容）執行 speclink artifact cat proposal --change 某 change
- **THEN** 兩模式 stdout 輸出相同的 artifact 內容，exit code 為 0

#### Scenario: 詞彙文件缺失
- **WHEN** 專案無共用詞彙文件時執行 speclink language show
- **THEN** exit code 非 0，stderr 訊息說明詞彙文件不存在（技能據此靜默跳過詞彙載入）

#### Scenario: 技能資產不含直接讀檔指示
- **WHEN** 執行 speclink update 後掃描生成的全部 SKILL.md 內容
- **THEN** 不存在指示直接開啟規格目錄檔案路徑的語句，文件閱讀一律以 speclink 動詞表述

#### Scenario: discuss 以 show 讀正式規格

- **WHEN** 檢視生成的 discuss 技能中讀取候選規格 Purpose 的步驟
- **THEN** 該步驟以 speclink show <capability> --item-type spec 表述；全文不存在把 path 欄位與 spec.md 寫在同一句的指示

#### Scenario: 三支技能的改寫經動詞

- **WHEN** 檢視生成的 propose 與 ingest 技能中修正或更新既有 artifact 的步驟
- **THEN** 每個這樣的步驟都指向「speclink artifact cat 讀出、speclink new artifact 帶 --stdin --force 寫回」的做法；兩支技能的全文不含 config.yaml

#### Scenario: remote 模式下 discuss 讀得到正式規格

- **WHEN** 於 remote 模式（checkout 內沒有規格資料夾）依 discuss 技能的步驟讀取某個 capability 的 Purpose
- **THEN** 執行的是 speclink show <capability> --item-type spec，stdout 含該規格的 Purpose，exit code 為 0

#### Scenario: 同名的作用中變更不會被誤讀成正式規格

- **WHEN** 專案有一個與某 capability 同名的作用中變更，依 discuss 技能的步驟讀那個 capability 的 Purpose
- **THEN** 執行的指令帶 --item-type spec，stdout 是正式規格的內容，不是那個變更的內容

### Requirement: 模式分岔的單點宣告
<!-- BEFORE: FsOnly 的子情形有兩個：change 的 rank 子指令與 plan 的 --strict-overlap 旗標 -->

CLI 頂層動詞 SHALL 逐一歸屬四種模式形狀之一，本機（fs）/remote 的分岔決策 SHALL 集中於 dispatch 的宣告層，SHALL NOT 散佈於各動詞函式內：

- **ModeFree**（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion）：執行 SHALL NOT 觸發 store 模式解析，dispatch SHALL NOT 因宣告層而對其引入 .speclink.yaml 的解析失敗——不讀取專案設定的動詞（completion、config）SHALL 不受壞的 .speclink.yaml 影響；部分動詞（如 schemas、templates、update）的 workspace 探索本就讀取 .speclink.yaml 以解析 spec_dir，其於壞檔下的既有失敗行為維持不變；連線管理動詞（link、unlink、auth）的連線解析由動詞自理。
- **Dual**（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify、plan、change）：fs 模式 SHALL 作用於本機 store，remote 模式 SHALL 作用於 remote store，SHALL NOT 於 remote 模式靜默作用於本機 store；宣告 SHALL 同時載明本機臂與 remote 臂，缺任一臂 SHALL 構成建置失敗而非執行期靜默回退；不消費 store 的前置步驟（instructions 的 --skill 分流、workflow-config 的 argv／stdin 正規化）SHALL 先於模式解析執行，維持既有可觀察順序。
- **FsOnly**（demo、trace；以及 Dual 家族內只限本機的一個子情形——plan 的 --strict-overlap 旗標）：remote 模式 SHALL 以非零 exit code 明確拒絕，拒絕判定 SHALL 僅解析模式而不建立連線——SHALL NOT 發出任何 server 請求，離線環境同樣拒絕。子情形的歸屬 SHALL 同樣在 dispatch 的宣告層依子指令或旗標決定，同一動詞的其餘子指令與不帶該旗標的呼叫維持 Dual。
- **RemoteOnly**（claim）：fs 模式 SHALL 以非零 exit code 明確拒絕並於 stderr 說明需要 remote store。

模式判定 SHALL 惰性執行：僅於宣告形狀需要時解析模式，僅於 remote 臂將執行時建立連線。

#### Scenario: ModeFree 動詞不受壞連線設定影響

- **WHEN** 於 .speclink.yaml 內容無法解析的專案目錄下執行 speclink completion generate zsh 與 speclink config list
- **THEN** 兩者正常輸出且 exit code 為 0，stderr 不含 .speclink.yaml 的解析錯誤

#### Scenario: FsOnly 動詞於 remote 模式零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下執行 speclink demo
- **THEN** exit code 非零，stderr 說明該動詞僅限本機模式，且過程未發出任何 server 請求

#### Scenario: Dual 家族內的 FsOnly 子情形零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下執行 speclink plan --strict-overlap
- **THEN** exit code 非零，stderr 說明 plan --strict-overlap 僅限本機模式，且過程未發出任何 server 請求

#### Scenario: RemoteOnly 動詞於 fs 模式明確拒絕

- **WHEN** 於 fs 模式專案執行 speclink claim 指定 change
- **THEN** exit code 非零，stderr 說明該動詞需要 remote store

#### Scenario: plan 與 change depends 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink plan --json 與 speclink change depends add-b --on add-a
- **THEN** 兩者分別對 server 發出 GET /plan 與 POST /changes/add-b/depends，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: change rank 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink change rank c --before a
- **THEN** 對 server 發出 POST /changes/c/rank，stdout 形狀與 fs 模式一致，未觸碰本機 store
