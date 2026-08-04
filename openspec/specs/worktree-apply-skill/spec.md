# worktree-apply-skill Specification

## Purpose

TBD - created by archiving change 'worktree-parallel-apply'. Update Purpose after archive.

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

生成的技能內文 SHALL 指示執行代理依序完成前置：(1) 讀取有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層），值非 true 時 SHALL 拒絕執行並向使用者說明「本專案未啟用 worktree 流程」與啟用方式（workflow-config set worktree true），SHALL NOT 逕行在主資料夾執行 apply 本體；(2) 確認目標 change 存在且未封存；(3) 確認該 change 的產物目錄已提交進 HEAD——未提交時僅提交該 change 目錄本身，SHALL NOT 夾帶其他髒檔（worktree 由 HEAD 具現化，產物不在 HEAD 的 worktree 不含該 change）；(4) 以分支 speclink/<change名> 於 sibling 巢 <repo資料夾名>.worktrees/<change名>/ 建立 worktree（該分支或 worktree 已存在時沿用既有者續作，SHALL NOT 重複建立）；(5) 向使用者印出建置成本提示（worktree 為完整原始碼副本，須自備依賴安裝與建置產物）；(6) 其後的 apply 本體流程 SHALL 於該 worktree 資料夾內執行。

#### Scenario: 內文含政策拒跑指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「政策非 true 即拒絕並說明啟用方式」的指示，且指示中含 workflow-config set worktree true 字面

#### Scenario: 內文含建立慣例與續作指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含分支慣例 speclink/<change名>、sibling 巢路徑慣例，與「已存在即沿用續作」的指示

#### Scenario: 內文含產物先進 HEAD 指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「change 產物提交進 HEAD」的指示，且該指示位於建立 worktree 步驟之前


<!-- @trace
source: worktree-parallel-apply
updated: 2026-08-04
-->

---
### Requirement: apply-with-worktree 技能的收尾指示

生成的技能內文 SHALL 指示執行代理於 apply 本體完成後：於 worktree 內完成該 change 的提交（沿用 commit 技能的歸屬慣例），SHALL NOT 執行合併回主分支，SHALL NOT 移除 worktree，並以明確文字告知使用者後續以 worktree-merge 技能收尾。

#### Scenario: 內文含停點與交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「不合併、不移除 worktree」的停點指示，且點名 worktree-merge 技能為後續步驟

<!-- @trace
source: worktree-parallel-apply
updated: 2026-08-04
-->