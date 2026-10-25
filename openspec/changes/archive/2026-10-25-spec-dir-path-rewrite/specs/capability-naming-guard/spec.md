## MODIFIED Requirements

### Requirement: 建立點主閘——未收錄名稱預設拒絕

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink new artifact spec <capability> --change <name>` 當 `<capability>` 不存在於正式規格（以正式規格 capability 清單逐字比對、區分大小寫——不以檔案系統存在性判定）、該 change 亦無同名 delta spec、且未帶 `--new` 旗標時，SHALL 拒絕：exit code 非零，錯誤訊息輸出至 stderr，且 SHALL NOT 建立或修改任何檔案（stdin 內容不落盤）。錯誤訊息 SHALL 包含：至多三筆近似既有名建議，以及兩條指引——修改既有 capability 就沿用其確切名稱、確為新 capability 則帶 `--new` 重跑。`<capability>` 命中正式規格時，指令行為與輸出 SHALL 與現行位元級一致。該 change 已存在同名 delta spec（先前已顯性宣告過）時，SHALL 不再要求 `--new`，直接進入既有的覆寫保護流程（未帶 `--force` 報 already-exists、帶 `--force` 覆寫）。指定的 change 不存在時，SHALL 維持現行的錯誤行為，不因主閘而改變。本需求約束引擎的 new artifact 建立動詞（fs 與 Node host 入口）；remote 模式的 raw artifact PUT 屬直接寫入通道，不經此閘，由 spec-validation 的近似名 warning 第二網涵蓋。帶 `--json` 時：成功路徑的 payload SHALL 維持現行欄位形狀（artifact、change、path、status、validated、warnings，camelCase 命名不變）；主閘拒絕 SHALL 循此指令現行錯誤在 `--json` 下的同一呈現路徑（非零 exit code），SHALL NOT 產出成功 payload。`--no-color` 下錯誤訊息 SHALL 依既有慣例不含色彩控制碼。

#### Scenario: 帶 --json 的成功與拒絕路徑

- **WHEN** 分別以命中正式規格的名稱與未收錄的名稱（未帶 --new）執行 `speclink new artifact spec <capability> --change <name> --json`
- **THEN** 前者輸出現行欄位形狀的成功 payload 且 exit code 為 0；後者以非零 exit code 結束且 stdout 無成功 payload

#### Scenario: 未收錄名稱未帶 --new 遭拒且不落盤

- **WHEN** 正式規格有 `auth` 而無 `authentication`，執行 `speclink new artifact spec authentication --change some-change` 並經 stdin 提供 delta 內容
- **THEN** 指令以非零 exit code 結束，stderr 含近似建議與兩條指引，且 `speclink/changes/some-change/specs/authentication/` SHALL NOT 存在

#### Scenario: 命中正式規格名稱照常放行

- **WHEN** 正式規格已有 `auth`，執行 `speclink new artifact spec auth --change some-change` 且 delta 內容合法
- **THEN** 指令成功建立 `specs/auth/spec.md`，stdout 輸出與本變更導入前位元級一致

#### Scenario: change 不存在時維持既有錯誤

- **WHEN** 執行 `speclink new artifact spec brand-new-cap --change no-such-change`
- **THEN** 指令以現行「找不到變更」的錯誤行為結束，主閘不改變其訊息與 exit code
