## REMOVED Requirements

### Requirement: 新增 Workspace 的來源分流

**Reason**: 使用者可見詞彙「Workspace」收斂為「專案」(openspec/LANGUAGE.md「專案（開啟的工作目錄）」詞條),需求名與對話框標題一併改名;入口清單隨外殼改版(圖示列「＋」、空狀態兩鈕)重寫。

**Migration**: 內容整段移入本 delta ADDED 的「新增專案的來源分流」:來源分流、本機開啟行為不變、伺服器頁籤預選 server 三項語意照舊,新增可選取來源卡、主要鈕與單選列的呈現要求。

## ADDED Requirements

### Requirement: 新增專案的來源分流

Desktop 的所有開啟入口(圖示列「＋」、零專案空狀態的「新增專案」與「連線 Server」、macOS 原生選單的「開啟專案…」)SHALL 匯流至單一「新增專案」對話框:第一步 SHALL 分流「本機資料夾」與「Server」兩張可選取的來源卡,選取後頁尾主要鈕才可按——本機為「選擇資料夾…」、Server 為「下一步」,未選取時主要鈕 SHALL 停用;「連線 Server」入口 SHALL 直接進入 Server 步驟;伺服器頁籤入口 SHALL 預選該 server 直達 scope 選擇步驟。本機路徑 SHALL 沿用既有資料夾選擇、專案探測與初始化流程且行為不變。步驟 2–4(Server、Project/Repo、checkout)SHALL 以整列可點的單選列呈現,選定資料與既有流程一致。對話框標題 SHALL 為「新增專案」(en「New project」),使用者可見文案 SHALL NOT 出現「Workspace」。

#### Scenario: 選取來源卡後才可按主要鈕

- **WHEN** 開啟「新增專案」對話框尚未選取來源卡
- **THEN** 頁尾主要鈕停用;選取「本機資料夾」後主要鈕為「選擇資料夾…」,按下即開原生資料夾選擇器;改選「Server」後主要鈕為「下一步」,按下進入 Server 步驟

#### Scenario: 本機開啟行為不變

- **WHEN** 經對話框選擇本機資料夾開啟既有 speclink 專案
- **THEN** 專案方塊建立與看板呈現與改版前一致;未初始化資料夾仍走既有初始化確認流程

#### Scenario: 空狀態的連線 Server 直達 Server 步驟

- **WHEN** 零專案空狀態按「連線 Server」
- **THEN** 對話框開啟並直接呈現 Server 步驟(已登入連線的單選列與「新增 Server」),步驟條顯示第 2 步

## MODIFIED Requirements

### Requirement: 最近開啟清單

「新增專案」對話框的第一步 SHALL 在「本機資料夾」與「Server」兩張來源卡下方列出最近開啟清單;零專案空狀態 SHALL 以同一份清單元件在主區列出同一筆記錄(本機條目顯示資料夾名稱與完整路徑、remote 條目帶「遠端」籤),點條目與移除的語意 SHALL 與對話框內一致,清單為空時 SHALL NOT 顯示該區段。app SHALL 於每次本機或 remote workspace 成功開啟時（經 chooser、分頁點擊、remote marker 探測或本機轉 remote）把該 workspace（locator 與顯示名）記入清單最前；同 locator 的 workspace SHALL 只保留一筆；清單 SHALL 最多保留 20 筆，超過時 SHALL 丟棄最舊的一筆。本機轉 remote 成功時 SHALL 移除該資料夾的本機條目並記入 remote 條目。記錄 SHALL 持久化於 app 本機狀態（localStorage 鍵 `speclink.recentWorkspaces`，JSON `{ version: 1, entries: [{ locator, name }] }`），SHALL NOT 寫入任何專案目錄；關閉分頁與分頁列的上限淘汰 SHALL NOT 改變記錄。

清單顯示時 SHALL 濾掉目前分頁列上已開著的 workspace（以 locator key 比對）；濾後為空時 SHALL NOT 顯示「最近開啟」區段（含標題）。本機條目 SHALL 顯示資料夾名稱與完整路徑；remote 條目 SHALL 顯示連線名稱與 workspace 顯示名（projectName/repoName），連線名稱 SHALL 自連線登錄即時查得。每筆條目 SHALL 提供移除操作(hover 才顯示的 ×),執行後 SHALL 自畫面與持久化記錄移除;remote 條目 SHALL 帶「遠端」籤。

點本機條目時 app SHALL 先探測該路徑，探測成功 SHALL 關閉 chooser 並執行與「本機資料夾」選同一路徑相同的開啟流程（既有專案直接開啟、未初始化資料夾走 init 確認、帶 remote marker 的資料夾走既有分流）；探測失敗 SHALL 把該條目轉為錯誤態並顯示錯誤原因，SHALL NOT 建立分頁或切換專案，記錄 SHALL 保留至使用者移除。點 remote 條目時 app SHALL 先驗證原 checkout 綁定仍與該 scope 一致（無 checkout 綁定的條目 SHALL 跳過此驗證），再執行與 scope 選擇流程相同的 remote 開啟；驗證或 handshake 失敗 SHALL 轉為錯誤態並顯示錯誤原因。

remote 條目的連線狀態 SHALL 於連線清單讀取成功後才判定：連線已自連線登錄移除 SHALL 以錯誤態呈現「連線已移除」，連線仍在但未登入 SHALL 以錯誤態呈現「連線已登出」，兩者 SHALL 停用開啟操作且 SHALL 保留移除操作。連線清單讀取失敗或尚未完成時 SHALL NOT 在顯示面判定為任一錯誤態，開啟操作 SHALL 維持可用；此時使用者點該條目，app SHALL 於開啟流程內以現有清單補判——連線仍解不出且該條目綁有 checkout 時 SHALL 以「連線已移除」轉錯誤態，無 checkout 綁定的條目 SHALL 照常執行 remote 開啟。錯誤態 SHALL 於 chooser 重新開啟、零專案空狀態重新出現時清除。點擊開啟失敗的錯誤態 SHALL NOT 停用該條目:使用者 SHALL 可再點一次重試,重試開始時清除該條目的錯誤態;停用開啟操作只用於連線已移除與連線已登出。

app 升級後首次啟動、localStorage 尚無最近開啟鍵時，app SHALL 以持久化分頁補種清單（最後開啟的分頁在最前）；鍵已存在（含空清單）時 SHALL NOT 補種。鍵內容為壞 JSON、version 不為 1 或形狀不識別時 SHALL 讀為空清單且 app 照常啟動；個別條目形狀不識別時 SHALL 只丟棄該條目。介面文案 SHALL 依介面語言提供：zh-TW 為「最近開啟」／「自最近開啟移除」／「連線已移除」／「連線已登出」，en 為「Recently opened」／「Remove from recently opened」／「Connection removed」／「Connection signed out」。

#### Scenario: 關閉分頁後仍列於最近開啟

- **WHEN** 使用者依序開啟本機專案 A 與 B，關閉 B 的分頁，再開啟「新增專案」
- **THEN** 第一步兩張來源卡下方顯示「最近開啟」區段，列出 B（資料夾名稱與完整路徑）且不列 A；A 與 B 的專案目錄內均無因此新增的檔案

#### Scenario: 記錄全在分頁列上時不顯示區段

- **WHEN** 最近開啟記錄只含 A 與 B，且 A、B 都在分頁列上時開啟「新增專案」
- **THEN** 第一步只顯示兩張來源卡，無「最近開啟」標題與任何條目

#### Scenario: 去重上移與上限截尾

- **WHEN** 使用者依序成功開啟多個 workspace
- **THEN** 記錄依最新在前排列、同 workspace 只留一筆、最多 20 筆

##### Example: 順序、去重與上限

| 開啟順序 | 記錄（最新在前） | 說明 |
| -------- | ---------------- | ---- |
| A, B, A | A, B | A 重複開啟只留一筆並移到最前 |
| A, B, C（B 分頁其後被關閉） | C, B, A | 關閉分頁不動記錄 |
| W1 … W21（21 個相異 workspace） | W21 … W2 | 第 21 筆記入時丟最舊的 W1 |

#### Scenario: 點本機條目直接開啟

- **WHEN** 使用者點最近開啟中的本機專案 B（資料夾仍是 speclink 專案）
- **THEN** chooser 關閉，B 的分頁出現於分頁列並成為活躍分頁，看板呈現 B 的內容，與經「本機資料夾」選同一路徑的結果一致

#### Scenario: 點未初始化資料夾的條目仍走 init 確認

- **WHEN** 使用者點最近開啟中的本機條目，而該資料夾的 openspec 骨架已被移除
- **THEN** app 顯示既有的初始化確認對話框，未經確認前不寫入該資料夾

#### Scenario: 點 remote 條目以原綁定開啟

- **WHEN** 使用者點最近開啟中的 remote 條目（連線仍已登入、原本綁有本機 checkout）
- **THEN** app 以同一 connection、同一 Project／Repo 與同一 checkout 資料夾開啟 remote workspace，分頁列出現該 workspace 並成為活躍分頁，chooser 關閉

#### Scenario: 本機資料夾已消失時轉錯誤態

- **WHEN** 最近開啟中本機專案 B 的資料夾已被刪除，使用者點該條目
- **THEN** 該條目顯示錯誤標記與單行錯誤原因，chooser 保持開啟，分頁列與活躍專案不變；條目仍在清單中，直到使用者以移除操作清除

#### Scenario: remote 連線已移除時直接呈現錯誤態

- **WHEN** 最近開啟中 remote 條目所屬的連線已自「伺服器」頁移除，使用者開啟「新增專案」
- **THEN** 該條目以錯誤態呈現「連線已移除」（en 為「Connection removed」），開啟操作停用，移除操作仍可用

#### Scenario: remote 連線已登出時停用開啟

- **WHEN** 最近開啟中 remote 條目所屬的連線仍在連線登錄但已登出，使用者開啟「新增專案」
- **THEN** 該條目以錯誤態呈現「連線已登出」（en 為「Connection signed out」），開啟操作停用，移除操作仍可用

#### Scenario: 連線清單讀取失敗時不判定錯誤態

- **WHEN** 開啟「新增專案」時連線清單讀取失敗（清單保持為空）
- **THEN** remote 條目 SHALL NOT 呈現「連線已移除」或「連線已登出」，開啟操作維持可用

#### Scenario: 清單未就緒時點條目由開啟流程補判

- **WHEN** 連線清單尚未讀到（讀取失敗或仍在進行中），使用者點一個綁有 checkout 的 remote 條目
- **THEN** 該條目以「連線已移除」轉錯誤態，SHALL NOT 建立分頁；同一情況下點無 checkout 綁定的 remote 條目 SHALL 照常以原 connection 與 scope 開啟

#### Scenario: 點 remote 條目時原 checkout 已失效

- **WHEN** 最近開啟中 remote 條目的 checkout 資料夾已被刪除，使用者點該條目
- **THEN** 該條目顯示錯誤標記與單行錯誤原因，SHALL NOT 建立分頁或切換專案，移除操作仍可用

#### Scenario: 移除條目後重啟不再出現

- **WHEN** 使用者對最近開啟中的條目執行移除，再重啟 app 並開啟「新增專案」
- **THEN** 該條目不再出現於清單，其餘條目順序不變

#### Scenario: 升級後首次啟動自分頁補種

- **WHEN** localStorage 存有分頁 A、B（依序開啟）而無最近開啟鍵時啟動新版 app，隨後關閉 A 的分頁並開啟「新增專案」
- **THEN** 最近開啟列出 A；持久化記錄為 B、A（最新在前）；再次重啟不會重複補種

#### Scenario: 壞資料歸零且不補種

- **WHEN** localStorage 的最近開啟鍵被手改為無法解析的內容後啟動 app
- **THEN** app 照常啟動、不崩潰、不彈錯誤；「新增專案」第一步無「最近開啟」區段；下一次成功開啟 workspace 後該鍵寫回 version 1 的合法內容且只含這一筆

#### Scenario: 零專案空狀態列出同一份最近開啟

- **WHEN** app 於零專案狀態啟動,最近開啟記錄含本機專案 B 與 remote 專案 R
- **THEN** 主區空狀態的「最近開啟」卡列出 B(資料夾名稱與完整路徑)與 R(連線名稱與 Project/Repo 顯示名、「遠端」籤),點 B 即以與對話框本機條目相同的流程開啟,× 移除後持久化記錄同步移除;記錄為空時空狀態不顯示「最近開啟」區段
