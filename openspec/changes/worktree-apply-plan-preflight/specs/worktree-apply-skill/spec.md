## MODIFIED Requirements

### Requirement: apply-with-worktree 技能的前置指示

生成的技能內文 SHALL 指示執行代理依序完成前置：(0) 解析輸入——偵測到多於一個 change 名時 SHALL 先執行 `speclink plan --json`，把 `blockedBy` 為空的名字列成並行配方（其餘 change 各開一個新 session 執行本技能）、`blockedBy` 非空的名字逐一列出其前置並說明須等前置落地後再開，然後停下請使用者自並行名單擇一，SHALL NOT 靜默依序批次執行多個 change；(1) 讀取有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層），值非 true 時 SHALL 拒絕執行並向使用者說明「本專案未啟用 worktree 流程」與啟用方式（workflow-config set worktree true），SHALL NOT 逕行在主資料夾執行 apply 本體；(2) 以 plan 選 change 並守門——於主 checkout 執行 `speclink plan --json`：未指名時 SHALL 取 `next`，`next` 為 null 時 SHALL 列出每個 change 的 `blockedBy` 並停止；指名時 SHALL 讀該 change 的 `blockedBy`，非空時 SHALL 印出前置清單並停止；指名的 change 不在 `changes` 內（已封存、不存在、或列於 `skipped`）時 SHALL 停止並列出可用的 change 名；plan 失敗（依賴成環、專案未初始化）時 SHALL 顯示錯誤並停止；本步驟 SHALL 位於產物提交與建立 worktree 之前，任一停止情況 SHALL NOT 提交產物、SHALL NOT 建立 worktree；(3) 確認該 change 的產物目錄已提交進 HEAD——未提交時僅提交該 change 目錄本身，SHALL NOT 夾帶其他髒檔（worktree 由 HEAD 具現化，產物不在 HEAD 的 worktree 不含該 change）；(3.5) 讀取該 change 目錄 .evidence.json 的 touched 檔清單並對主樹查 git 狀態——evidence 檔不存在或清單為空時靜默續行；任一 touched 檔於主樹為髒時 SHALL 停下列出髒檔，以推薦序提供三選項：「先執行 speclink-commit 將本 change 的程式碼提交進 HEAD 再回來」「照樣繼續（明知 worktree 缺這些實作）」「停止」，SHALL NOT 未經使用者選擇逕行建立 worktree；(4) 以分支 speclink/<change名> 於 sibling 巢 <repo資料夾名>.worktrees/<change名>/ 建立 worktree（該分支或 worktree 已存在時沿用既有者續作，SHALL NOT 重複建立）；(5) 向使用者印出建置成本提示（worktree 為完整原始碼副本，須自備依賴安裝與建置產物）；(6) 其後的 apply 本體流程 SHALL 於該 worktree 資料夾內執行，且內文 SHALL 明寫：本體第 1 步在 worktree 內執行的 plan 查詢只是複查——worktree 內的 plan 讀不到其他 worktree 的進度與主線後續的封存——其結果與 (2) 不同時 SHALL 以主 checkout 的判定為準。apply 本體資產 SHALL NOT 因本要求而改動，speclink-apply 的生成輸出 SHALL 維持位元級不變。

#### Scenario: 內文含政策拒跑指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「政策非 true 即拒絕並說明啟用方式」的指示，且指示中含 workflow-config set worktree true 字面

#### Scenario: 內文含建立慣例與續作指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含分支慣例 speclink/<change名>、sibling 巢路徑慣例，與「已存在即沿用續作」的指示

#### Scenario: 內文含產物先進 HEAD 指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「change 產物提交進 HEAD」的指示，且該指示位於建立 worktree 步驟之前

#### Scenario: 內文含多 change 拒收指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「多個 change 名時先以 plan 分出可並行與須等前置者，請使用者自並行名單擇一並印出多 session 配方」與「禁止靜默依序批次」的指示，且位於政策檢查步驟之前

#### Scenario: 內文含進度與程式碼分家偵測指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「讀 .evidence.json 的 touched 清單對主樹查 git 狀態」的指示與三選項推薦序（speclink-commit 為首選、其次照樣繼續、最後停止），且位於產物進 HEAD 步驟之後、建立 worktree 步驟之前

#### Scenario: 內文含 plan 選擇與守門指示

- **WHEN** 技能再生後讀取 speclink-apply-with-worktree 的 SKILL.md
- **THEN** 前置段含執行 `speclink plan --json`、未指名取 `next`、`blockedBy` 非空即停止、不在 `changes` 即停止並列可用名單的指示，且該指示位於政策檢查步驟之後、產物進 HEAD 步驟之前，並含「不提交產物、不建立 worktree」字面

#### Scenario: 內文含本體複查以主 checkout 為準指示

- **WHEN** 技能再生後讀取 speclink-apply-with-worktree 的 SKILL.md
- **THEN** 前置段含「本體第 1 步的 plan 在 worktree 內只是複查、結果不同時以主 checkout 的判定為準」的指示，且 speclink-apply 的 SKILL.md 與本變更前位元級一致

##### Example: 多 change 名依 plan 分組

| 輸入的 change 名 | plan 的 blockedBy | 代理人的處置 |
| --- | --- | --- |
| add-a、add-b、add-c | add-a：空；add-b：空；add-c：[add-a] | 並行配方列 add-a、add-b；add-c 說明「等 add-a 落地再開」；請使用者自 add-a、add-b 擇一 |
| add-c | add-c：[add-a] | 印出前置 add-a 並停止；不提交產物、不建立 worktree |
| （未指名） | next 為 add-a | 選 add-a，宣告「Using change: add-a」後續行 (3) |
| （未指名） | next 為 null | 列出每個 change 的 blockedBy 並停止 |
