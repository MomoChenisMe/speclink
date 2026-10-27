## Purpose

repo 本地發版技能 `/release` 的契約：執行即發版——守門條件、版號判定規則、更新日誌策展規則、寫檔順序、提交與 tag／push 的邊界。邊界：只涵蓋 tag push 之前由人與 Claude 完成的動作；tag 之後的管線屬 desktop-release、server-release、cli-distribution；更新日誌的資料形狀與衍生物屬 release-notes。

## ADDED Requirements

### Requirement: 發版技能為 repo 本地技能

repo SHALL 提供 `.claude/skills/release/SKILL.md` 作為發版入口。此技能 SHALL NOT 納入 speclink 的受管技能資產（不進 ASSET_VERSION、golden 與 assets.lock），SHALL NOT 隨 `speclink init`／`speclink update` 安裝到使用者專案，且目錄名 SHALL NOT 以 `speclink-` 開頭。

#### Scenario: update 不清除發版技能

- **WHEN** 在本 repo 執行 `speclink update`
- **THEN** `.claude/skills/release/` 原封不動，受管技能目錄照常再生

### Requirement: 守門不過即停且不改動工作樹

技能執行時 SHALL 依序檢查：目前分支為 main；`git status --porcelain` 輸出為空；`git fetch origin` 後 HEAD 等於 origin/main；HEAD 這個 commit 的 CI 工作流有一筆紀錄且結論為 success。任一項不成立 SHALL 印出不成立的那一項並停止，SHALL NOT 改動任何檔案。`speclink list --json` 回報進行中變更時 SHALL 列出名稱並請使用者確認是否繼續，SHALL NOT 自行阻擋。

#### Scenario: 工作樹有未提交改動

- **WHEN** 執行技能時 `git status --porcelain` 非空
- **THEN** 技能印出「工作樹不乾淨」與未提交檔案清單並停止，未改任何檔、未建 tag

#### Scenario: HEAD 的 CI 尚未跑完

- **WHEN** HEAD 的 CI 工作流沒有紀錄或仍在執行
- **THEN** 技能印出「CI 尚未通過」並停止，請使用者等 CI 綠燈後重跑

#### Scenario: 有進行中變更仍可繼續

- **WHEN** 守門四項皆通過但 `speclink list --json` 有 in-progress 變更
- **THEN** 技能列出這些變更名稱並詢問是否繼續；使用者確認後才進入版號判定

### Requirement: 版號由上次 tag 到 HEAD 的 commit 判定

技能 SHALL 以 `git describe --tags --abbrev=0` 取得上次 tag，並以該 tag 到 HEAD 的 commit（主旨與內文）為版號判定與日誌策展的唯一輸入。技能 SHALL 逐筆解析 conventional commit type 並取最高影響：`!` 或 BREAKING CHANGE footer 為 major、feat 為 minor、fix 或 perf 為 patch；chore、docs、refactor、test、ci、build、style SHALL NOT 提高版號。上次 tag 低於 1.0.0 時 breaking SHALL 封頂為 minor，升到 1.0.0 SHALL 由使用者明示。範圍內只有不提高版號的 type 時 SHALL 警告「這批沒有面向使用者的變更」並詢問是否仍發。無法解析的 commit SHALL 當 patch 並列出請使用者確認。判定結果 SHALL 以建議呈現，使用者得覆寫為任一合法版號。

#### Scenario: 0.x 階段的 breaking 封頂 minor

- **WHEN** 上次 tag 為 v0.2.0 且範圍內有 `refactor(init)!:` 的 commit
- **THEN** 建議版號為 0.3.0，不是 1.0.0

##### Example: 版號判定

| 上次 tag | 範圍內 commit type | 建議版號 |
| -------- | ------------------ | -------- |
| v0.2.0 | fix, fix | 0.2.1 |
| v0.2.0 | feat, fix | 0.3.0 |
| v0.2.0 | refactor!, fix | 0.3.0 |
| v1.4.0 | feat! | 2.0.0 |
| v0.2.0 | chore, test | 無（技能警告並詢問） |

#### Scenario: 只有雜訊 commit

- **WHEN** 範圍內全是 chore 與 test
- **THEN** 技能警告這批沒有面向使用者的變更並詢問是否仍發；使用者選擇不發時技能停止且未改任何檔

### Requirement: 更新日誌策展為使用者語氣的繁中條目

技能 SHALL 把範圍內 commit 策展為繁體中文條目並分組為「新功能」（feat）、「修正」（fix、perf）、「改善」（其餘有保留價值者）。封存變更的 commit（主旨形如 `speclink(<變更名>): …`）SHALL 優先處理、一個變更一條，以其摘要與該變更 proposal 的 Why 改寫成使用者語氣；不屬於任何變更的 feat、fix、perf SHALL 逐筆改寫。chore、test、refactor、docs、ci、build、style SHALL NOT 出現在條目中。技術名詞（CLI、worktree、Tauri、slug 等）SHALL 保留原文。條目 SHALL NOT 照抄 commit 主旨。

#### Scenario: 封存變更收成一條

- **WHEN** 範圍內有 `speclink(discussion-spinout-hold): 一份討論可以分期轉出多個變更` 與其開發過程的三筆 feat／fix commit
- **THEN** 日誌只出現一條「新功能」條目描述分期轉出，開發過程的三筆不另列

#### Scenario: 雜訊不進日誌

- **WHEN** 範圍內有 `chore(release): 版號升至 0.2.0` 與 `test(server): 消除 SSE 保留窗地板的競態`
- **THEN** 兩筆都不出現在任何分組

### Requirement: 呈現草稿並在使用者點頭後才寫檔

技能 SHALL 在寫任何檔之前呈現建議版號、版號理由與分組日誌草稿，並等待使用者確認；使用者修改時 SHALL 重新呈現。確認後 SHALL 依序：更新日誌 JSON 頂端加入新條目（date 為當天）→ workspace Cargo.toml 的 version → apps/desktop/src-tauri/tauri.conf.json 的 version → 以 cargo metadata 刷新 Cargo.lock → 以渲染腳本產出 CHANGELOG.md → 執行渲染腳本與下載指南腳本的測試及 `--check`。任一步失敗 SHALL 停在該步、印出原因、SHALL NOT 提交，已改的檔留在工作樹供使用者檢視。

#### Scenario: 使用者未點頭不寫檔

- **WHEN** 技能呈現草稿後使用者要求修改條目文字
- **THEN** 技能重新呈現修改後的草稿，工作樹仍無任何改動

#### Scenario: 寫檔中途失敗

- **WHEN** 渲染腳本的 `--check` 回非零
- **THEN** 技能停止並印出比對差異，JSON、Cargo.toml、tauri.conf.json 的改動留在工作樹，沒有新 commit 與 tag

### Requirement: 提交、附註 tag 與原子 push

寫檔全部成功後技能 SHALL 以一個 commit 提交全部改動，訊息為 `chore(release): 版號升至 X.Y.Z`；SHALL 建立附註 tag `vX.Y.Z`（訊息為 `vX.Y.Z`）；SHALL 以 `git push --atomic origin main vX.Y.Z` 一次推送 main 與 tag。push 失敗時 commit 與 tag SHALL 留在本機，技能 SHALL 印出可手動重試的同一句 push 指令。成功後 SHALL 印出 Release 工作流的網址與三項收尾提醒：job 綠不等於套件已在 registry、新套件 packument 延遲可見、發完重裝本機 app。

#### Scenario: 成功發版

- **WHEN** 使用者點頭且寫檔與測試全綠
- **THEN** origin/main 多出一個 `chore(release): 版號升至 0.3.0` commit、origin 有附註 tag v0.3.0 指向該 commit，技能印出 Release 工作流網址

#### Scenario: push 被拒

- **WHEN** `git push --atomic` 因 origin 拒絕而失敗
- **THEN** origin 上 main 與 tag 都沒有變動，本機保有 commit 與 tag，技能印出 `git push --atomic origin main v0.3.0` 供重試
