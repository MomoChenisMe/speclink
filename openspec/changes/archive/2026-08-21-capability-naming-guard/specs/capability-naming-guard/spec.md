## Purpose

capability 命名守門機制：在 delta spec 建立點以確定性規則拒絕未宣告的新 capability 名稱、產出近似既有名的排序建議，並統轄 propose／ingest 技能資產中的命名守門指令面。邊界：只守「名稱與既有規格的一致性」，不涉語意重複偵測、不涉 archive 階段的合併行為。

## ADDED Requirements

### Requirement: 建立點主閘——未收錄名稱預設拒絕

`speclink new artifact spec <capability> --change <name>` 當 `<capability>` 不存在於正典規格（以正典 capability 清單逐字比對、區分大小寫——不以檔案系統存在性判定）、該 change 亦無同名 delta spec、且未帶 `--new` 旗標時，SHALL 拒絕：exit code 非零，錯誤訊息輸出至 stderr，且 SHALL NOT 建立或修改任何檔案（stdin 內容不落盤）。錯誤訊息 SHALL 包含：至多三筆近似既有名建議，以及兩條指引——修改既有 capability 就沿用其確切名稱、確為新 capability 則帶 `--new` 重跑。`<capability>` 命中正典時，指令行為與輸出 SHALL 與現行位元級一致。該 change 已存在同名 delta spec（先前已顯性宣告過）時，SHALL 不再要求 `--new`，直接進入既有的覆寫保護流程（未帶 `--force` 報 already-exists、帶 `--force` 覆寫）。指定的 change 不存在時，SHALL 維持現行的錯誤行為，不因主閘而改變。本需求約束引擎的 new artifact 建立動詞（fs 與 Node host 入口）；remote 模式的 raw artifact PUT 屬直接寫入通道，不經此閘，由 spec-validation 的近似名 warning 第二網涵蓋。帶 `--json` 時：成功路徑的 payload SHALL 維持現行欄位形狀（artifact、change、path、status、validated、warnings，camelCase 命名不變）；主閘拒絕 SHALL 循此指令現行錯誤在 `--json` 下的同一呈現路徑（非零 exit code），SHALL NOT 產出成功 payload。`--no-color` 下錯誤訊息 SHALL 依既有慣例不含色彩控制碼。

#### Scenario: 帶 --json 的成功與拒絕路徑

- **WHEN** 分別以命中正典的名稱與未收錄的名稱（未帶 --new）執行 `speclink new artifact spec <capability> --change <name> --json`
- **THEN** 前者輸出現行欄位形狀的成功 payload 且 exit code 為 0；後者以非零 exit code 結束且 stdout 無成功 payload

#### Scenario: 未收錄名稱未帶 --new 遭拒且不落盤

- **WHEN** 正典有 `auth` 而無 `authentication`，執行 `speclink new artifact spec authentication --change some-change` 並經 stdin 提供 delta 內容
- **THEN** 指令以非零 exit code 結束，stderr 含近似建議與兩條指引，且 `openspec/changes/some-change/specs/authentication/` SHALL NOT 存在

#### Scenario: 命中正典名稱照常放行

- **WHEN** 正典已有 `auth`，執行 `speclink new artifact spec auth --change some-change` 且 delta 內容合法
- **THEN** 指令成功建立 `specs/auth/spec.md`，stdout 輸出與本變更導入前位元級一致

#### Scenario: change 不存在時維持既有錯誤

- **WHEN** 執行 `speclink new artifact spec brand-new-cap --change no-such-change`
- **THEN** 指令以現行「找不到變更」的錯誤行為結束，主閘不改變其訊息與 exit code

### Requirement: --new 旗標顯性宣告新 capability

`speclink new artifact spec <capability> --change <name> --new` 對正典未收錄的名稱 SHALL 照現行建立流程執行，既有的 delta 格式驗證與同路徑檔案的覆寫保護 SHALL 全數保留。`--new` 用於正典已收錄的名稱時 SHALL 無額外效果——行為與未帶旗標時一致。

#### Scenario: 帶 --new 建立新 capability 成功

- **WHEN** 正典無 `token-rotation`，執行 `speclink new artifact spec token-rotation --change some-change --new` 且 delta 內容含合法操作區塊
- **THEN** 指令成功建立 `specs/token-rotation/spec.md`，exit code 為 0

#### Scenario: --new 不豁免 delta 格式驗證

- **WHEN** 帶 `--new` 但 stdin 的 delta 內容不含任何操作區塊（ADDED／MODIFIED／REMOVED／RENAMED）
- **THEN** 指令仍以現行格式驗證錯誤拒絕，SHALL NOT 建立檔案

### Requirement: 近似名單的來源與排序

主閘的建議池 SHALL 由兩個來源組成：正典 capabilities，以及未封存 change 的 delta capabilities（含當前 change 的其他 delta），同名以正典優先去重。每筆建議 SHALL 標注來源（正典，或進行中的 change 名稱），並附該規格 Purpose 區段的首行；來源規格無 Purpose 時 SHALL 省略該行而非留空。排序 SHALL 依序比較：名稱 token 的完全包含關係優先，其次 kebab 字段交集數，再次編輯距離；名稱比對 SHALL 做 ASCII 大小寫折疊；輸出 SHALL 至多三筆。與候選完全同名的 in-flight delta SHALL 不列入相似名清單，改以獨立指引點名開立它的 change 並指路 `--new`。相似度 SHALL NOT 作為拒絕與否的條件——無任何近似名時仍 SHALL 拒絕，僅訊息不含建議清單。

#### Scenario: 包含關係排在最前

- **WHEN** 正典有 `auth` 與 `author-tools`，以未收錄名稱 `authentication` 觸發主閘
- **THEN** stderr 的建議清單首位為 `auth`（token 完全包含），且清單至多三筆

#### Scenario: 進行中 change 的 delta 出現在名單

- **WHEN** 另一個未封存 change `add-sso` 含 delta capability `user-auth`，以未收錄名稱 `user-authentication` 觸發主閘
- **THEN** 建議清單含 `user-auth` 且標注其來自進行中的 change `add-sso`

#### Scenario: 無近似名仍拒絕

- **WHEN** 以與既有名毫無交集的未收錄名稱 `zzz-unrelated` 觸發主閘（未帶 --new）
- **THEN** 指令仍以非零 exit code 拒絕，訊息含兩條指引但不含建議清單

### Requirement: 技能資產的命名守門指令

propose 技能資產 SHALL 指示代理：既有規格掃描結果須留痕於 proposal、Capabilities 區段的每個新 capability 附一句「為何既有規格不涵蓋」、並說明 `--new` 旗標的語意與使用時機。ingest 技能資產 SHALL 指示代理在新增 delta capability 前先對照既有名。資產內文變更 SHALL 連動 marker 版號、golden snapshots 與 assets.lock，對應測試 SHALL 維持綠燈。

#### Scenario: 生成的技能檔含命名守門指引

- **WHEN** 資產更新後執行技能生成並比對 golden snapshots
- **THEN** propose 技能內容含掃描留痕、新 capability 理由與 --new 語意三項指引，ingest 技能內容含既有名對照指引，snapshot 測試通過

#### Scenario: 資產連動檔案同步更新

- **WHEN** 資產內文變更完成後執行資產一致性測試
- **THEN** marker 版號已推進、assets.lock 與 golden snapshots 與資產內容一致，測試以綠燈通過
