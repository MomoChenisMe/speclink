## ADDED Requirements

### Requirement: 文件作為預填樹來源逐條分診

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 規定：topic 指定文件路徑（自寫 markdown、plan mode 產出、repo 內 docs 或任意可讀路徑）時，代理人 SHALL 讀取該文件並萃取其主張作為決策樹節點，逐條對 codebase 分診為三類——證實（附程式碼證據）、牴觸（指出文件內容與程式碼實況的差異並附證據）、真決策（送使用者裁定）。文件 SHALL NOT 僅作背景素材一次性閱讀。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含文件分診紀律

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 規定：文件主張萃取為決策樹節點，逐條分診為證實／牴觸／真決策三類，且每類附對應證據或裁定去向

#### Scenario: 文件主張與程式碼牴觸時逐條呈現

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔對牴觸類主張的處理規定
- **THEN** 技能檔 SHALL 規定牴觸須逐條指出文件內容與程式碼實況的差異並附程式碼證據，SHALL NOT 允許籠統帶過或僅摘要文件

### Requirement: Source doc 記錄慣例

技能檔 SHALL 規定：以文件為輸入的討論，其記錄的 Context 段 SHALL 含一行 Source doc: <路徑>；輪的 Evidence 引用文件時 SHALL 以段落標題或短句為之；討論記錄 SHALL 只存討論結果，SHALL NOT 內嵌整份規劃文件；代理人 SHALL NOT 修改使用者的原始規劃文件。

#### Scenario: Context 記錄文件來源

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的記錄規則
- **THEN** 技能檔 SHALL 規定 Context 含 Source doc: <路徑> 一行、輪 Evidence 以段落標題或短句引用文件、記錄不內嵌整份文件、不修改原始文件

#### Scenario: 未給文件時記錄照舊

- **WHEN** 討論的 topic 未指定任何文件
- **THEN** 技能檔 SHALL 規定記錄流程與現行相同，Context 無 Source doc 行，無額外文件讀取步驟
