## Why

手動測試標記 `[M]` 只在「checkbox 之後的前綴槽」被解析（manual-task-marker 正典的 Example 表凍結此行為）。但「編號在前」（`- [ ] 6.2 [M] …`）是任何起草者的自然寫法，寫錯後該任務被靜默算成寫碼任務：codeRemaining 永遠降不到 0、apply 被誘導代勾手動任務、兩個品質站依各自守門拒絕。propose 技能早已附正確範例仍失守（desktop-loading-skeleton-ux 在範例存在後起草照樣寫錯），而 validate 與 analyze 對此零檢查——錯誤要到 station 守門卡死才被發現。

目標使用者是透過 AI 代理跑 SDD 的開發者：起草發生在 propose／ingest 階段，攔截發生在 validate。結論來自討論 manual-marker-placement-lint：兩關防護——文字規定降低發生率，validate 的 error 保證攔截；解析器本體不動（嚴格但會報錯，優於寬容但靜默改變守門行為）。

## What Changes

- **validate 關（speclink-core）**：change 驗證新增手動標記位置檢查，error 級。解析 tasks.md 後，任務描述命中兩種錯型即報 error 使驗證 invalid：「編號緊接 `[M]`」（首個空白分隔 token 僅含數字與句點、次 token 為字面 `[M]`）與「行首殘留 `[M]`」（checkbox 後多打空格致前綴槽漏接）。錯誤訊息比照 Purpose 早期檢查慣例自帶修復指引：正誤例並列、點名把 `[M]` 移到編號前。無 tasks.md 或無命中時零輸出，既有驗證結果逐位元不變。
- **技能關（assets）**：propose 與 ingest 兩個技能 asset 的任務起草指引改為「對比對」呈現——正例與誤例並列、附後果（引擎不認、codeRemaining 卡住）與「checkbox 後恰一個空格」規則。asset 內文變動走 MARKER_VERSION／golden／assets.lock 三連動，claude 與 codex 兩形正典化生成。
- **正典規格**：manual-task-marker 增兩條（標記位置的 change 驗證檢查、ingest 技能的起草標記指引——後者比照既有「apply 技能的手動任務處理」條文落點）；propose-skill 的「手動測試任務的起草標記」修訂為要求對比對呈現。

相容性影響：`speclink validate` 的人眼與 `--json` 輸出僅在 tasks.md 含誤置標記時新增 error 行；格式正確的變更輸出逐位元不變，不破壞既有回歸對照。無新指令、無旗標變動。

## Non-Goals

- **不放寬解析器**：不接受「編號後的 `[M]`」。解析行為被 manual-task-marker 的 Example 表凍結，且 core 與 packages/ui 有兩套同步解析器；描述含字面 `[M]` 時寬容解析會誤判並靜默改變守門行為。
- **analyze 不重複報**：三鏡頭（覆蓋率／一致性／缺口）是內容品質檢查，格式守門歸 validate 單一落點。
- **lint 不出 warning**：warning 會被忽略，重演「validate 全綠但 gate 卡死」。
- **不含 desktop-loading-skeleton-ux 6.2 的現場修正**：該行已在其 worktree 修正，隨 worktree-merge 流回。
- **不動 packages/ui 的解析器與顯示行為**：UI 剝離規則維持現狀。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `manual-task-marker`: 新增「標記位置的 change 驗證檢查」（validate 的 error 級 lint）與「ingest 技能的起草標記指引」兩條需求
- `propose-skill`: 「手動測試任務的起草標記」修訂——起草指引須以正誤例並列（對比對）呈現，附後果與空格規則

## Impact

- Affected specs: manual-task-marker、propose-skill
- Affected code:
  - New: (none)
  - Modified:
    - crates/speclink-core/src/tasks.rs（標記誤置判斷式，供 validate 消費）
    - crates/speclink-core/src/validate.rs（change 驗證接線與錯誤訊息）
    - crates/speclink-core/assets/skills/propose.md（起草指引對比對）
    - crates/speclink-core/assets/skills/ingest.md（同上）
    - crates/speclink-core/src/init.rs（MARKER_VERSION 提升）
    - crates/speclink-core/tests/golden/assets.lock 與 golden 對照檔（三連動再生）
    - .claude/skills/speclink-propose/SKILL.md 等本 repo 技能渲染副本（再生同步）
  - Removed: (none)
