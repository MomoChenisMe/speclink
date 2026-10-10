# claude-code-skill-bar Specification

## Purpose

讓 Claude Code 使用者在輸入框上方選擇已安裝的 Speclink 技能，填入指令並保留既有參數。範圍包含技能分頁、合併指令、說明、窄視窗呈現與介面語言；工作焦點與側邊面板分屬其他能力。

## Requirements

### Requirement: 依工作流程呈現已安裝技能

技能列 SHALL 只呈現已安裝的 `speclink-` 技能，排除面板開關指令，依「規劃／實作／品質／收尾／其他」分頁。未知技能 SHALL 列於「其他」，空分頁 SHALL 隱藏；只有 archive 與 commit 都已安裝時才呈現 `archive+commit`。無可列技能或輸入框正在顯示問卷時 SHALL 保留原畫面而不加技能列。對話開始、clear、resume 與每輪完成後 SHALL 重新取得技能清單。

#### Scenario: 未知技能與合併按鈕

- **WHEN** 已安裝 verify、apply、archive、commit 與 new-thing
- **THEN** 分頁依序呈現實作的 apply、品質的 verify、收尾的 archive 與 archive+commit、其他的 commit 與 new-thing，不呈現規劃分頁

#### Scenario: 合併所需技能不齊

- **WHEN** 只安裝 archive
- **THEN** 收尾分頁只呈現 archive，不呈現 archive+commit

#### Scenario: 接回或清空對話

- **WHEN** clear 或 resume 發生而沒有一般對話開始事件
- **THEN** 技能列仍重新取得技能，呈現可用技能與面板按鈕

#### Scenario: 無可用技能

- **WHEN** 沒有可列的 Speclink 技能，或輸入框正在顯示問卷
- **THEN** 原輸入框上方內容維持，不新增技能列

Evidence: `integrations/claude-code/speclink-skills/hooks/register.tsx`、`integrations/claude-code/speclink-skills/hooks/groups.ts`、`integrations/claude-code/speclink-skills/hooks/skills.ts`，以及 `integrations/claude-code/speclink-skills/tests/register.test.tsx` 的技能分頁、合併按鈕與 clear／resume 測試；無技能與問卷情境由已讀取程式確認。

### Requirement: 點技能只填入指令並保留參數

點擊技能 SHALL 將對應斜線指令放在輸入框最前方，取代原先最前方的斜線指令並保留後方文字；SHALL 不自動送出或執行指令。`archive+commit` SHALL 填入 `/speclink-archive + /speclink-commit`，更換技能時 SHALL 將兩個指令一起取代。此操作 SHALL 不建立或修改 proposal.md、delta spec、.openspec.yaml、任務 evidence 或 snapshots；這些效果屬之後送出指令的工作流程。

#### Scenario: 保留原有參數

- **WHEN** 輸入框是 `/speclink-review add-x`，使用者選擇 verify
- **THEN** 輸入框成為 `/speclink-verify add-x`

#### Scenario: 換掉合併指令

- **WHEN** 輸入框是 `/speclink-archive + /speclink-commit add-x`，使用者選擇 apply
- **THEN** 輸入框成為 `/speclink-apply add-x`

Evidence: `integrations/claude-code/speclink-skills/hooks/skills.ts`、`integrations/claude-code/speclink-skills/hooks/register.tsx`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的指令填入、合併指令與技能列互動測試。

### Requirement: 技能說明與窄視窗呈現

寬視窗的技能列 SHALL 在滑鼠移到技能時顯示該技能說明，平常隱藏說明；合併技能 SHALL 以 `/speclink-archive → /speclink-commit` 作為說明。分頁與面板按鈕放不下一列時，面板按鈕 SHALL 移到 speclink 標題列，且 SHALL 不顯示技能說明。

#### Scenario: 窄視窗切換

- **WHEN** 測試中已安裝技能形成規劃、實作、收尾、其他四頁，繁中介面可用寬度為 41 格
- **THEN** 仍呈現規劃、面板與 propose，分頁列不含 ` │ `，不顯示技能說明；寬度為 42 格時恢復分隔與說明區

#### Scenario: 合併技能說明

- **WHEN** 使用者在寬視窗收尾分頁移到 archive+commit
- **THEN** 顯示 `/speclink-archive → /speclink-commit`

Evidence: `integrations/claude-code/speclink-skills/hooks/register.tsx`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的窄視窗與技能說明測試。滑鼠顯示條件由已讀取程式確認，測試確認說明內容存在。

### Requirement: 介面語言由 mod 與 Claude Code 設定決定

mod 的 language 選項 SHALL 支援 `auto`、`en`、`zh-TW`，預設為 auto。明選 en 或 zh-TW SHALL 優先於 Claude Code 設定；auto SHALL 在 Claude Code 語言文字含「中文」、chinese 或 zh（英文不分大小寫）時選繁中，其餘含未設定時選英文。工作流 locale 的 `tw`／`ja`／`en` SHALL 不作為此介面的語言來源；此介面只有繁中與英文，日文設定在 auto 下落到英文，中文弱偵測依上述文字條件。

本規格 SHALL 記錄既有 UI 輸出，沒有刻意變更。技能按鈕不是 CLI 子指令，SHALL 不新增旗標、stdin、exit code、stdout／stderr 或 `--json` 輸出；`--no-color` 是 CLI 的選項，不控制這些 UI 按鈕。

#### Scenario: 中文弱偵測與未設定

- **WHEN** auto 遇到 `台灣繁體中文zh-tw` 或 `Chinese`
- **THEN** 介面使用繁中；遇到 `English` 或未設定時使用英文

#### Scenario: 明選英文

- **WHEN** mod language 為 en，Claude Code 語言為 `台灣繁體中文zh-tw`
- **THEN** 實作分頁顯示 `Build`

#### Scenario: 日文設定

- **WHEN** mod language 為 auto，Claude Code 語言文字為 `日本語`
- **THEN** 介面使用英文

Evidence: `integrations/claude-code/speclink-skills/.claude-plugin/plugin.json`、`integrations/claude-code/speclink-skills/hooks/text.ts`、`integrations/claude-code/speclink-skills/hooks/register.tsx`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的語言測試。日文情境由已讀取的語言判定確認。
