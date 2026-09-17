# worktree-apply-skill Specification

## Purpose

/speclink-apply-with-worktree 技能的內容：技能檔如何由 apply 本體加上 worktree 前後段組合生成、前置指示（政策檢查、變更存在性、產物入 HEAD、worktree 路徑與分支慣例），以及收尾指示（在 worktree 內提交、不自行合併、交棒給 worktree-merge）。本 capability 保證平行作業的每個變更各自待在自己的 worktree 裡，主 checkout 全程不被動到。

## Requirements

### Requirement: apply-with-worktree 技能的生成與組合

技能再生（speclink 的技能同步機制）SHALL 產出 apply-with-worktree 技能：claude 目標生成 .claude/skills/speclink-apply-with-worktree/SKILL.md，codex 目標生成對應產物。生成內容 SHALL 為自包含文件：worktree 前置段、完整的 apply 本體流程（與 apply 技能同源的全文，非摘要亦非引用）、worktree 收尾段，三段依序組合。既有 apply 技能的生成輸出 SHALL 維持位元級不變。

#### Scenario: claude 目標生成技能檔

- **WHEN** 於已初始化 claude 工具的 workspace 執行技能再生
- **THEN** .claude/skills/speclink-apply-with-worktree/SKILL.md 存在，內容含 worktree 前置段與 apply 本體流程的既有段落標記

#### Scenario: apply 本體同源不走樣

- **WHEN** 技能再生後比對 speclink-apply-with-worktree 與 speclink-apply 的生成內容
- **THEN** 前者完整包含後者的本體流程段落（逐段可對應），且後者的輸出與本能力導入前位元級一致


<!-- @trace
source: worktree-parallel-apply
updated: 2026-08-04
-->

---
### Requirement: apply-with-worktree 技能的前置指示

生成的技能內文 SHALL 指示執行代理依序完成前置：(0) 解析輸入——偵測到多於一個 change 名時 SHALL 先執行 `speclink plan --json`，把 `blockedBy` 為空的名字列成並行配方（其餘 change 各開一個新 session 執行本技能）、`blockedBy` 非空的名字逐一列出其前置並說明須等前置落地後再開，然後停下請使用者自並行名單擇一；恰有一個可並行時 SHALL 改為請使用者確認是否在此執行該 change（不印配方），沒有可並行者時 SHALL 列出須等與不可用的名字並停止；SHALL NOT 靜默依序批次執行多個 change；(1) 讀取有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層），值非 true 時 SHALL 拒絕執行並向使用者說明「本專案未啟用 worktree 流程」與啟用方式（workflow-config set worktree true），SHALL NOT 逕行在主資料夾執行 apply 本體；(2) 以 plan 選 change 並守門——於主 checkout 執行 `speclink plan --json`：未指名時 SHALL 取 `next`，`next` 為 null 時 SHALL 列出每個 change 的 `blockedBy` 並停止；指名時 SHALL 讀該 change 的 `blockedBy`，非空時 SHALL 印出前置清單並停止；指名的 change 不在 `changes` 內（已封存、不存在、或列於 `skipped`）時 SHALL 停止並列出可用的 change 名；plan 失敗（依賴成環、專案未初始化）時 SHALL 顯示錯誤並停止；本步驟 SHALL 位於產物提交與建立 worktree 之前，任一停止情況 SHALL NOT 提交產物、SHALL NOT 建立 worktree；(3) 確認該 change 的產物目錄已提交進 HEAD——未提交時僅提交該 change 目錄本身，SHALL NOT 夾帶其他髒檔（worktree 由 HEAD 具現化，產物不在 HEAD 的 worktree 不含該 change）；(3.5) 讀取該 change 目錄 .evidence.json 的 touched 檔清單並對主樹查 git 狀態——evidence 檔不存在或清單為空時靜默續行；任一 touched 檔於主樹為髒時 SHALL 停下列出髒檔，以推薦序提供三選項：「先執行 speclink-commit 將本 change 的程式碼提交進 HEAD 再回來」「照樣繼續（明知 worktree 缺這些實作）」「停止」，SHALL NOT 未經使用者選擇逕行建立 worktree；(4) 以分支 speclink/<change名> 於 sibling 巢 <repo資料夾名>.worktrees/<change名>/ 建立 worktree（該分支或 worktree 已存在時沿用既有者續作，SHALL NOT 重複建立）；(5) 向使用者印出建置成本提示（worktree 為完整原始碼副本，須自備依賴安裝與建置產物）；(6) 其後的 apply 本體流程 SHALL 於該 worktree 資料夾內執行，且內文 SHALL 明寫：本體第 1 步在 worktree 內執行的 plan 查詢只是複查——worktree 內的 plan 讀不到其他 worktree 的進度與主線後續的封存——其結果與 (2) 不同時 SHALL 以主 checkout 的判定為準，且本體第 2 步（含 review prepare 與 in-progress add）SHALL 照常執行。apply 本體資產 SHALL NOT 因本要求而改動，speclink-apply 的生成輸出除 frontmatter 的 version 行外 SHALL 維持位元級不變。

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
- **THEN** 內文含「多個 change 名時先以 plan 分出可並行與須等前置者，請使用者自並行名單擇一並印出多 session 配方；恰有一個可並行時改為確認是否在此執行；沒有可並行者時列出須等與不可用的名字並停止」與「禁止靜默依序批次」的指示，且位於政策檢查步驟之前

#### Scenario: 內文含進度與程式碼分家偵測指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「讀 .evidence.json 的 touched 清單對主樹查 git 狀態」的指示與三選項推薦序（speclink-commit 為首選、其次照樣繼續、最後停止），且位於產物進 HEAD 步驟之後、建立 worktree 步驟之前

#### Scenario: 內文含 plan 選擇與守門指示

- **WHEN** 技能再生後讀取 speclink-apply-with-worktree 的 SKILL.md
- **THEN** 前置段含執行 `speclink plan --json`、未指名取 `next`、`blockedBy` 非空即停止、不在 `changes` 即停止並列可用名單的指示，且該指示位於政策檢查步驟之後、產物進 HEAD 步驟之前，並含「不提交產物、不建立 worktree」字面

#### Scenario: 內文含本體複查以主 checkout 為準指示

- **WHEN** 技能再生後讀取 speclink-apply-with-worktree 的 SKILL.md
- **THEN** 前置段含「本體第 1 步的 plan 在 worktree 內只是複查、結果不同時以主 checkout 的判定為準、第 2 步照常執行」的指示，且 speclink-apply 的 SKILL.md 除 frontmatter 的 version 行外與本變更前位元級一致

##### Example: 多 change 名依 plan 分組

| 輸入的 change 名 | plan 的 blockedBy | 代理人的處置 |
| --- | --- | --- |
| add-a、add-b、add-c | add-a：空；add-b：空；add-c：[add-a] | 並行配方列 add-a、add-b；add-c 說明「等 add-a 落地再開」；請使用者自 add-a、add-b 擇一 |
| add-a、add-c | add-a：空；add-c：[add-a] | add-c 說明「等 add-a 落地再開」；不印配方，請使用者確認是否在此執行 add-a |
| add-b、add-c | add-b：[add-a]；add-c：[add-a] | add-b、add-c 各說明「等 add-a 落地再開」；沒有可選的 change，停止；不提交產物、不建立 worktree |
| add-c | add-c：[add-a] | 印出前置 add-a 並停止；不提交產物、不建立 worktree |
| （未指名） | next 為 add-a | 選 add-a，宣告「Using change: add-a」後續行 (3) |
| （未指名） | next 為 null | 列出每個 change 的 blockedBy 並停止 |


<!-- @trace
source: worktree-apply-plan-preflight
updated: 2026-09-17T15:45:03+08:00
-->

---
### Requirement: apply-with-worktree 技能的收尾指示

生成的技能內文 SHALL 指示執行代理於 apply 本體完成後：於 worktree 內完成該 change 的提交（沿用 commit 技能的歸屬慣例），SHALL NOT 執行合併回主分支，SHALL NOT 移除 worktree，並以明確文字依正典順序交棒：建議先於 worktree 內執行品質關卡——交棒句 SHALL 明列三個入口：review（工藝品質）∥ verify（規格符合度），或 quality（兩站合跑）——由使用者判斷是否執行；蓋章寫入的 meta 變更 SHALL 提示補提交，再以 worktree-merge 技能收尾。

#### Scenario: 內文含停點與正典順序交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「不合併、不移除 worktree」的停點指示，交棒段的品質關卡建議明列 review、verify、quality 三個入口（於 worktree 內執行、蓋章後補提交）並點名 worktree-merge 技能為後續步驟


<!-- @trace
source: worktree-handoff-quality-mention
updated: 2026-08-27
-->