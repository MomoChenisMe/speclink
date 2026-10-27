## ADDED Requirements

### Requirement: 討論記錄以 --kind 標記改進討論

speclink discuss new SHALL 接受選配旗標 --kind,白名單僅接受 improve。合法值時 SHALL 於 frontmatter 增 kind 欄位,--json payload SHALL 增 kind 欄位,人眼輸出 SHALL 沿用既有建立訊息格式、SHALL NOT 新增行。非白名單值時指令 SHALL 以非零 exit code 結束、於 stderr 說明僅接受 improve,且 SHALL NOT 建立任何檔案。未帶 --kind 時人眼輸出與 --json SHALL 逐位元不變,回歸對照不受影響。無 kind 欄位的既有記錄 SHALL 視為一般討論,SHALL NOT 要求遷移。speclink discuss list --json 與 speclink discuss show --json SHALL 於記錄有 kind 時曝露該欄位、無 kind 時省略該鍵。本旗標為 Speclink 自有延伸。

#### Scenario: 帶 --kind improve 建立改進討論

- **WHEN** 執行 speclink discuss new 並給定主題、合法 --slug 與 --kind improve
- **THEN** 建立的記錄 frontmatter 含 kind: improve;帶 --json 時 payload 的 kind 欄位為 improve;人眼輸出與未帶 --kind 時的建立訊息格式一致

#### Scenario: 非法 kind 值被拒且不落檔

- **WHEN** 執行 speclink discuss new 且 --kind 的值不是 improve
- **THEN** 指令以非零 exit code 結束,stderr 說明僅接受 improve,openspec/discussions/ 下不新增任何檔案

#### Scenario: 未帶 --kind 輸出不變

- **WHEN** 執行 speclink discuss new 未帶 --kind
- **THEN** 人眼輸出與 --json payload 與本旗標引入前逐位元一致,frontmatter 無 kind 欄位

#### Scenario: list 與 show 曝露 kind

- **WHEN** 對含 kind: improve 的記錄執行 speclink discuss list --json 與 speclink discuss show --json
- **THEN** 兩者 payload 均含 kind 欄位且值為 improve;對無 kind 的記錄則 payload 不含該鍵
