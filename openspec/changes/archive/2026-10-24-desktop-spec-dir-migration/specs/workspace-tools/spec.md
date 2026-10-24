## MODIFIED Requirements

### Requirement: 工作區補齊入口

<!-- BEFORE: 對「已有 openspec/ 但無 .speclink.yaml」的目錄就地補齊，既有 openspec/ 零觸碰 -->
<!-- REMOVED-SCENARIO: 補齊工作區檔且既有內容零觸碰 -->

引擎 SHALL 提供冪等的工作區補齊（adopt）入口：對「有規格資料夾但無 `.speclink.yaml`」的目錄啟用 speclink。規格資料夾是舊佈局的 `openspec/`（且沒有 `speclink/`）時，adopt SHALL 先以與 `speclink init` 匯入相同的程式把 `openspec/` 改名為 `speclink/`、每個 `.openspec.yaml` 改名為 `.meta.yaml`，檔案內容逐位元組不變；在 git repo 裡、有主目錄以外的 worktree 或 `openspec/` 有未提交修改時，adopt SHALL 回單行錯誤說明原因且零寫入。之後 SHALL 補 `speclink/` 骨架缺件（specs/ 與 changes/archive/ 目錄；config.yaml 僅在不存在時寫入範本）、於專案根寫入 `.speclink.yaml` 記錄所選 tools（不寫入 `spec_dir`）、為每個所選工具生成技能檔，並確保專案根 `.gitignore` 涵蓋 `.speclink/` 工作資料夾。既有規格文件（specs、changes、discussions、schemas 及既有 config.yaml）的內容 SHALL 不被改寫。入口 SHALL NOT 受 init 的「已初始化即擋下」限制；tools 空清單 SHALL 回錯誤。重複執行 SHALL 冪等收斂於相同結果。

#### Scenario: 匯入後既有內容位元級不變

- **WHEN** 對已 commit、含 openspec/（內有規格文件、一個含 .openspec.yaml 的變更與自訂 config.yaml）但無 .speclink.yaml 的目錄以 tools=[claude] 執行 adopt
- **THEN** openspec/ 不存在、speclink/ 存在；speclink/ 內的規格文件與 config.yaml 與原檔位元級相同，變更資料檔為 .meta.yaml 且內容與原 .openspec.yaml 相同；專案根產生 .speclink.yaml（tools 含 claude、沒有 spec_dir）與 .claude/skills/ 技能檔，不生成 CLAUDE.md

#### Scenario: 骨架缺件補齊

- **WHEN** 對 openspec/ 內缺 specs/ 目錄與 config.yaml 的未啟用目錄執行 adopt
- **THEN** 匯入後 speclink/specs/、speclink/changes/archive/ 目錄建立，speclink/config.yaml 以範本寫入

#### Scenario: 工作資料夾納入版控忽略

- **WHEN** 對 .gitignore 不存在、或存在但未涵蓋 `.speclink/` 的未啟用目錄執行 adopt
- **THEN** 專案根 .gitignore 涵蓋 `.speclink/`，且該檔既有內容原樣保留

##### Example: 既有 .gitignore 追加而非覆寫

- **GIVEN** 專案根 .gitignore 內容為 `node_modules/\ndist/\n`
- **WHEN** 以 tools=[claude] 執行 adopt
- **THEN** .gitignore 仍含 `node_modules/` 與 `dist/` 兩行，並多出 `.speclink/` 條目

#### Scenario: 重複執行冪等

- **WHEN** 對同一目錄以相同 tools 連續執行 adopt 兩次
- **THEN** 第二次成功且所有生成檔內容與第一次相同

#### Scenario: tools 空清單拒絕

- **WHEN** 以空的 tools 清單執行 adopt
- **THEN** 回單行錯誤，目錄零寫入

#### Scenario: 有未提交修改時拒絕匯入

- **WHEN** 未啟用目錄的 openspec/changes/demo/proposal.md 有未提交修改，以 tools=[claude] 執行 adopt
- **THEN** 回單行錯誤說明 openspec/ 有未提交修改，openspec/ 原封不動，沒有產生 .speclink.yaml、speclink/ 或任何技能檔
