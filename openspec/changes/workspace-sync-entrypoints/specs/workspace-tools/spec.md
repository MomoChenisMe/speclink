## MODIFIED Requirements

### Requirement: update 清除孤兒技能目錄
<!-- BEFORE: 只有 speclink update 清除孤兒目錄；init（含 --force）保留前一次生成的 speclink-* 目錄與下架工具的足跡 -->

每一個再生入口——`speclink update`、`speclink init` 與 `speclink init --force`（filesystem 與 Remote Store）、工具選集收斂、工作區補齊、`workflow-config` 寫入後的技能足跡同步、桌面的技能檔更新動作——於各生成目標（內建工具與自訂描述子）完成技能生成後，SHALL 清除該目標 skills 目錄下名稱以 speclink- 為前綴、且不屬於該目標本次應生成集合的目錄。本次應生成集合 SHALL 依既有規則計算：claude 為 registry 全集、codex 與自訂描述子為 for_codex 子集，worktree 政策關閉時排除兩顆 worktree 技能。名稱非 speclink- 前綴的目錄 SHALL NOT 被移除。任一目錄刪除失敗時該入口 SHALL 以非零 exit code（或單行錯誤）結束，已生成的檔案保留；重跑 SHALL 收斂到同一終態。本清理 SHALL 與既有三條清理路徑（工具自 tools 下架、自訂描述子移除、worktree 政策關閉）並存，不改變其行為。`speclink init --force` 的選集 SHALL 視為內建工具的完整期望狀態：未選工具的 speclink- 技能目錄 SHALL 移除，自訂描述子的足跡記錄 SHALL 隨設定檔重寫歸零，兩個內建指令檔的遺留 `SPECLINK:START..END` 區塊無論選取與否 SHALL 剝除（使用者內容保留）。init 的 stdout 摘要 SHALL 維持既有兩行，不新增清理明細。

#### Scenario: 技能改名後舊目錄被清除

- **WHEN** 工作區的 skills 目錄含舊版生成的 speclink-onboard 目錄，執行 speclink update
- **THEN** speclink-onboard 目錄不存在，speclink-baseline 目錄存在，兩份技能不並存

#### Scenario: 非前綴目錄不受清理影響

- **WHEN** skills 目錄含使用者自建、名稱非 speclink- 前綴的技能目錄（如 conventional-commit），執行 speclink update
- **THEN** 該目錄與其內容位元級不變

#### Scenario: 前綴保留給生成物

- **WHEN** skills 目錄含名稱以 speclink- 為前綴、但不在本次應生成集合內的目錄，執行 speclink update
- **THEN** 該目錄被清除——speclink- 前綴的目錄一律視為引擎生成物

#### Scenario: init --force 切換工具時清除下架足跡

- **WHEN** 工作區 `.speclink.yaml` 的 tools 為 `[claude]` 且 `.claude/skills/` 含現版技能檔，執行 `speclink init --force --tools codex`
- **THEN** exit code 為 0，stdout 仍為 Initialized 與 Generated files 兩行；`.speclink.yaml` 的 tools 僅含 codex；`.agents/skills/` 含 Codex 生成集合；`.claude/skills/` 下不存在任何 speclink- 目錄，因而變空的 `.claude/skills/` 與 `.claude/` 一併移除

##### Example: 各再生入口的清理面

| 入口 | 觸發 | 清理結果 |
| --- | --- | --- |
| speclink update | tools=[claude] 但 .claude/skills/ 含 speclink-onboard | speclink-onboard 移除 |
| speclink init --force --tools codex | 原 tools=[claude] | .claude/skills/speclink-* 全部移除、.agents/skills/ 補齊 |
| speclink init --force --tools claude | .claude/skills/ 含 speclink-onboard | speclink-onboard 移除、其餘現版 |
| 工具選集收斂 [codex] | 原 tools=[claude] | 同 update（既有行為） |

#### Scenario: init --force 清除改名技能的舊目錄與描述子足跡

- **WHEN** 工作區 `.claude/skills/` 含 speclink-onboard 目錄，且 `.speclink.yaml` 曾含一個描述子、`.speclink/generated-tools.yaml` 記錄其足跡、描述子 skills_dir 下有生成物，執行 `speclink init --force --tools claude`
- **THEN** speclink-onboard 目錄不存在；描述子 skills_dir 下的 speclink- 目錄移除；`.speclink/generated-tools.yaml` 不存在；`.speclink.yaml` 為樣板加 tools=[claude]

#### Scenario: init --force 剝除未選工具的遺留區塊

- **WHEN** `CLAUDE.md` 含遺留 `SPECLINK:START..END` 區塊與使用者段落，執行 `speclink init --force --tools codex`
- **THEN** `CLAUDE.md` 的區塊被剝除、使用者段落原樣保留；`AGENTS.md` 不存在時不被建立

#### Scenario: 不帶 force 的 init 改寫殘留技能檔

- **WHEN** 目錄無 `.speclink.yaml`、無 `openspec/`，但 `.claude/skills/speclink-apply/SKILL.md` 為舊版內容且 `.claude/skills/speclink-onboard/` 殘留，執行 `speclink init --tools claude`
- **THEN** exit code 為 0；speclink-apply 的 SKILL.md 為現版內容；speclink-onboard 目錄不存在；其餘生成集合補齊

### Requirement: 技能檔過期探測
<!-- BEFORE: 探測只讀 tools 清單內的內建工具（claude／codex），自訂描述子的技能檔不在判定面與差異清單內 -->

引擎 SHALL 提供唯讀的技能檔過期探測：依 .speclink.yaml 的 tools 清單——內建工具與通過驗證的自訂描述子——讀取各工具 skills 目錄下技能檔 frontmatter 的版本欄位並與當前產物層版號比對，回報五態之一——缺失（任一工具的 skills 目錄下無任何 speclink- 技能檔，即從未安裝或整組移除）、過期（任一工具的技能版號舊於現版）、較新（任一工具的技能版號新於現版，即工作區檔案領先引擎）、現版、無法判定（設定解析失敗或技能檔存在但讀取錯誤）。逐工具方向資訊的工具名 SHALL 為內建名（claude、codex）或描述子的 name。無法通過驗證的描述子 SHALL NOT 參與探測，也 SHALL NOT 使結果變為無法判定（壞描述子由 update 的錯誤告知）。方向 SHALL 以版號數值比較判定：去除 v 前綴、以點號拆段、逐段數值比較，段數不足補零；任一邊無法完整解析為數字段時，該工具 SHALL 退回字串相等判定（不等即過期），SHALL NOT 對無法解析的版號排序方向、SHALL NOT 據以判較新。聚合優先序 SHALL 為 較新 > 缺失 > 過期 > 現版：任一工具較新即整體回報較新。過期、缺失或較新時 SHALL 一併回報「更新將新建或改寫且內容與現版 render 不同」的受管檔清單（專案根相對路徑，描述子的路徑以其 skills_dir 起頭）與各工具的方向資訊；比對前 SHALL 正規化換行，僅換行形式差異的檔案 SHALL NOT 列入清單。探測 SHALL NOT 寫入任何檔案。

#### Scenario: 舊版工作區判過期並列差異檔

- **WHEN** 工作區技能檔的版號數值舊於當前產物層版號，執行過期探測
- **THEN** 回報過期，並列出內容與現版 render 不同的技能檔相對路徑

#### Scenario: 工作區檔案領先引擎判較新

- **WHEN** 工作區技能檔的版號數值新於當前產物層版號（如新引擎再生後以舊版 app 探測），執行過期探測
- **THEN** 回報較新，並回報差異檔清單；SHALL NOT 與過期混同

##### Example: 引擎 v1.11.0 探測 v1.14.0 工作區

- **GIVEN** 引擎產物層版號 v1.11.0，工作區技能檔 frontmatter 版本 v1.14.0
- **WHEN** 執行過期探測
- **THEN** status 為 "newer"（2026-08-05 事故情境：舊判準回報「過期」並導致按「更新」降級 30 檔）

#### Scenario: 較新優先於缺失與過期

- **WHEN** tools 清單含 claude 與 codex，.claude/skills/ 技能版號新於現版而 .agents/skills/ 下無任何 speclink- 技能檔，執行過期探測
- **THEN** 回報較新（非缺失）——任何會改寫領先檔案的動作都不應被提供

#### Scenario: 無法解析的版號退回相等判定

- **WHEN** 工作區技能檔的版本欄位為無法解析為數字段的字串（如手改壞的 frontmatter），與現版不等，執行過期探測
- **THEN** 該工具判過期（字串不等），SHALL NOT 判較新

#### Scenario: 現版工作區不過期

- **WHEN** 工作區全部技能檔由當前版本的 init 或 update 生成，執行過期探測
- **THEN** 回報現版，差異清單為空

#### Scenario: 技能目錄缺少判缺失

- **WHEN** tools 清單含 claude 與 codex，.claude/skills/ 為現版而 .agents/skills/ 下無任何 speclink- 技能檔（如 clone 後技能未進版控），執行過期探測
- **THEN** 回報缺失，並列出更新將新建或改寫且內容與現版 render 不同的受管檔相對路徑；不與無法判定混同

#### Scenario: 設定損壞回報無法判定

- **WHEN** .speclink.yaml 無法解析，執行過期探測
- **THEN** 回報無法判定；SHALL NOT 與現版或過期混同

#### Scenario: 換行差異不誤報

- **WHEN** 工作區技能檔內容與現版 render 僅換行形式不同（CRLF 對 LF），執行過期探測
- **THEN** 該檔不出現在差異清單

#### Scenario: 描述子技能檔缺失判缺失

- **WHEN** tools 清單含 claude 與一個合法描述子（name 為 cursor、skills_dir 為 .cursor/skills），.claude/skills/ 為現版而 .cursor/skills/ 下無任何 speclink- 技能檔，執行過期探測
- **THEN** 回報缺失；逐工具資訊含 tool 為 cursor 且 missing 為真的一項；差異清單含以 `.cursor/skills/speclink-` 起頭的描述子技能檔路徑（for_codex 子集，worktree 政策關閉時不含兩顆 worktree 技能），不含任何 .claude/skills/ 路徑

##### Example: 描述子缺失時的回報

- **GIVEN** tools 為 `[claude, {name: cursor, skills_dir: .cursor/skills, …}]`，.claude/skills/ 全為現版，.cursor/skills/ 不存在
- **WHEN** 執行過期探測
- **THEN** status 為 "missing"；tools 含 `{tool: "claude", missing: false, stale: false, newer: false}` 與 `{tool: "cursor", missing: true, workspaceVersion: null}`；differingFiles 每一項皆以 `.cursor/skills/speclink-` 起頭並以 `/SKILL.md` 結尾

#### Scenario: 描述子技能檔過期判過期

- **WHEN** tools 清單含一個合法描述子，其 skills_dir 下技能檔的版號數值舊於現版，執行過期探測
- **THEN** 回報過期；逐工具資訊該描述子 stale 為真；差異清單列出其內容與現版 render 不同的技能檔路徑

#### Scenario: 無效描述子不影響探測

- **WHEN** tools 清單含 claude 與一個無法通過驗證的描述子（缺 skills_dir），.claude/skills/ 為現版，執行過期探測
- **THEN** 回報現版，逐工具資訊只含 claude，差異清單為空
