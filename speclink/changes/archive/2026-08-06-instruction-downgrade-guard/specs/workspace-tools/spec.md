## MODIFIED Requirements

### Requirement: 指令檔過期探測

<!-- BEFORE: 四態（缺失／過期／現版／無法判定）；版號比對為字串相等判定，SHALL NOT 解析版本語意；缺失與過期並存時回報缺失。分不出檔案比引擎新還是舊，降級被包裝成「更新」。 -->

引擎 SHALL 提供唯讀的指令檔過期探測：依 .speclink.yaml 的 tools 清單，讀取各內建工具指令檔的 SPECLINK 標記版號並與當前產物層版號比對，回報五態之一——缺失（任一工具的指令檔不存在，即從未安裝）、過期（任一工具的標記版號舊於現版）、較新（任一工具的標記版號新於現版，即工作區檔案領先引擎）、現版、無法判定（設定解析失敗或指令檔存在但讀取錯誤）。方向 SHALL 以版號數值比較判定：去除 v 前綴、以點號拆段、逐段數值比較，段數不足補零；任一邊無法完整解析為數字段時，該工具 SHALL 退回字串相等判定（不等即過期），SHALL NOT 對無法解析的版號排序方向、SHALL NOT 據以判較新。聚合優先序 SHALL 為 較新 > 缺失 > 過期 > 現版：任一工具較新即整體回報較新。指令檔存在但不含 SPECLINK 標記時，該工具 SHALL 視為已退出受管、不計入任何狀態；指令檔不存在 SHALL 判缺失，SHALL NOT 與退出受管或無法判定混同。過期、缺失或較新時 SHALL 一併回報「更新將新建或改寫且內容與現版 render 不同」的受管檔清單（專案根相對路徑）與各工具的方向資訊；比對前 SHALL 正規化換行，僅換行形式差異的檔案 SHALL NOT 列入清單。探測 SHALL NOT 寫入任何檔案。

#### Scenario: 舊版工作區判過期並列差異檔

- **WHEN** 工作區 CLAUDE.md 的標記版號數值舊於當前產物層版號，執行過期探測
- **THEN** 回報過期，並列出內容與現版 render 不同的受管檔相對路徑（含技能檔與指令檔）

#### Scenario: 工作區檔案領先引擎判較新

- **WHEN** 工作區 CLAUDE.md 的標記版號數值新於當前產物層版號（如新引擎再生後以舊版 app 探測），執行過期探測
- **THEN** 回報較新，並回報差異檔清單；SHALL NOT 與過期混同

##### Example: 引擎 v1.11.0 探測 v1.14.0 工作區

- **GIVEN** 引擎產物層版號 v1.11.0，工作區 CLAUDE.md 標記 v1.14.0
- **WHEN** 執行過期探測
- **THEN** status 為 "newer"（2026-08-05 事故情境：舊判準回報「過期」並導致按「更新」降級 30 檔）

#### Scenario: 較新優先於缺失與過期

- **WHEN** tools 清單含 claude 與 codex，CLAUDE.md 標記版號新於現版而 AGENTS.md 不存在，執行過期探測
- **THEN** 回報較新（非缺失）——任何會改寫領先檔案的動作都不應被提供

#### Scenario: 無法解析的版號退回相等判定

- **WHEN** 工作區 CLAUDE.md 的標記版號為無法解析為數字段的字串（如手改壞的標記），與現版不等，執行過期探測
- **THEN** 該工具判過期（字串不等），SHALL NOT 判較新

#### Scenario: 現版工作區不過期

- **WHEN** 工作區全部受管檔由當前版本的 init 或 update 生成，執行過期探測
- **THEN** 回報現版，差異清單為空

#### Scenario: 標記移除視為退出受管

- **WHEN** tools 清單僅含 claude 且 CLAUDE.md 不含 SPECLINK 標記（使用者整塊移除），執行過期探測
- **THEN** 回報現版（不過期），不列任何差異檔

#### Scenario: 指令檔不存在判缺失

- **WHEN** tools 清單含 claude 與 codex，CLAUDE.md 為現版而 AGENTS.md 不存在（如 clone 後指令檔未進版控），執行過期探測
- **THEN** 回報缺失，並列出更新將新建或改寫且內容與現版 render 不同的受管檔相對路徑；不與退出受管（檔案存在但無標記）或無法判定混同

#### Scenario: 設定損壞回報無法判定

- **WHEN** .speclink.yaml 無法解析，執行過期探測
- **THEN** 回報無法判定；SHALL NOT 與現版或過期混同

#### Scenario: 換行差異不誤報

- **WHEN** 工作區技能檔內容與現版 render 僅換行形式不同（CRLF 對 LF），執行過期探測
- **THEN** 該檔不出現在差異清單

## ADDED Requirements

### Requirement: 受管檔再生的降級守門

引擎的受管檔再生 SHALL 於任何寫入之前判定方向：即將被新建或改寫的指令檔中，任一檔的標記版號數值領先當前產物層版號時 SHALL 拒絕執行——輸出單行英文說明（含工作區領先的版號與引擎現版版號）、exit code 非零、SHALL NOT 寫入任何檔案（含設定檔）。判定目標 SHALL 取自該次再生的實際寫入集（tools 清單選集、無清單時的目錄偵測、自訂描述子的指令檔），SHALL NOT 僅取內建工具。守門 SHALL 一體適用於所有經再生入口的路徑：`speclink update`、`speclink init --force` 的重建、工具選集收斂、`workflow-config` 寫入後的技能足跡同步（CLI 與桌面設定頁）、桌面的指令檔更新動作。旗標 `--allow-downgrade`（`speclink update`）SHALL 為唯一的明示越過入口；`--force` SHALL NOT 被視為同意降級。缺失、過期、現版與無法判定情境下的既有行為 SHALL 不變。守門訊息 SHALL 為英文單行（與 update 既有輸出語言一致），不隨 locale 設定變化。

#### Scenario: 較新工作區拒絕 update

- **WHEN** 工作區標記版號新於引擎現版，執行 speclink update
- **THEN** stderr 單行說明含兩個版號、exit code 非零、工作區零檔案變動

#### Scenario: --allow-downgrade 越過守門

- **WHEN** 工作區標記版號新於引擎現版，執行 speclink update --allow-downgrade
- **THEN** 受管檔照常再生為引擎現版內容、exit code 0

#### Scenario: 過期工作區照常更新

- **WHEN** 工作區標記版號舊於引擎現版，執行 speclink update（不帶旗標）
- **THEN** 受管檔照常再生、exit code 0，行為與守門引入前相同

#### Scenario: --force 重建不等於同意降級

- **WHEN** 工作區標記版號新於引擎現版，執行 speclink init --force
- **THEN** 單行說明含兩個版號、exit code 非零、工作區零檔案變動；`--force` SHALL NOT 越過守門

#### Scenario: 無 tools 清單的工作區同受守門

- **WHEN** `.speclink.yaml` 未記錄 tools 清單（再生走目錄偵測），而既有指令檔的標記版號新於引擎現版，執行 speclink update
- **THEN** 拒絕執行、零檔案變動；SHALL NOT 因判定面只看內建清單而放行

#### Scenario: 自訂描述子的指令檔同受守門

- **WHEN** tools 清單只含自訂描述子，其指令檔的標記版號新於引擎現版，執行 speclink update
- **THEN** 拒絕執行、零檔案變動

#### Scenario: 技能足跡同步被拒時的失敗形狀

- **WHEN** 工作區標記版號新於引擎現版，執行 speclink workflow-config set worktree true
- **THEN** 設定檔已寫入新值，技能足跡同步被守門拒絕——單行說明含兩個版號、exit code 非零、受管檔零變動（沿用同步失敗的既有形狀）

### Requirement: 引擎版號查詢面

`speclink --version` SHALL 同時輸出套件版號、架構與產物層版號，格式為 `<套件版號> (<架構>, engine <產物層版號>)`；產物層版號 SHALL 與指令檔標記使用的版號同源。輸出 SHALL 為單行至 stdout、exit code 0。

#### Scenario: --version 含引擎版號

- **WHEN** 執行 speclink --version
- **THEN** stdout 單行同時含套件版號、架構與 engine 產物層版號，exit code 0

##### Example: 版號輸出格式

- **GIVEN** 套件版號 0.1.0、arm64 架構、產物層版號 v1.14.0
- **WHEN** 執行 speclink --version
- **THEN** 輸出含 `0.1.0 (arm64, engine v1.14.0)`
