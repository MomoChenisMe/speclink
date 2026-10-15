## REMOVED Requirements

### Requirement: 更新日誌策展為使用者語氣的繁中條目

**Reason**: 更新日誌改為中英雙語，策展要同時產出英文與繁中，這條需求只規範繁中。

**Migration**: 由「更新日誌策展為使用者語氣的中英雙語條目」取代；分組規則、封存變更收成一條與雜訊排除的規則全部保留。

## ADDED Requirements

### Requirement: 更新日誌策展為使用者語氣的中英雙語條目

技能 SHALL 把範圍內 commit 策展為中英雙語條目並分組為 features（feat）、fixes（fix、perf）、improvements（其餘有保留價值者）。每個條目 SHALL 同時有繁體中文與英文兩種寫法，兩者描述同一件事：繁中 SHALL 用使用者語氣與 `openspec/LANGUAGE.md` 的正典詞彙；英文 SHALL 用簡化技術英文（一句一事、主動語態、SHALL NOT 用完成式）。封存變更的 commit（主旨形如 `speclink(<變更名>): …`）SHALL 優先處理、一個變更一條，以其摘要與該變更 proposal 的 Why 改寫成使用者語氣；不屬於任何變更的 feat、fix、perf SHALL 逐筆改寫。chore、test、refactor、docs、ci、build、style SHALL NOT 出現在條目中。技術名詞（CLI、worktree、Tauri、slug 等）在兩種語言 SHALL 保留原文。條目 SHALL NOT 照抄 commit 主旨。

#### Scenario: 封存變更收成一條

- **WHEN** 範圍內有 `speclink(discussion-spinout-hold): 一份討論可以分期轉出多個變更` 與其開發過程的三筆 feat／fix commit
- **THEN** 日誌只出現一條 features 條目描述分期轉出，該條同時有英文與繁中，開發過程的三筆不另列

#### Scenario: 雜訊不進日誌

- **WHEN** 範圍內有 `chore(release): 版號升至 0.2.0` 與 `test(server): 消除 SSE 保留窗地板的競態`
- **THEN** 兩筆都不出現在任何分組

#### Scenario: 每條都有兩種語言

- **WHEN** 技能呈現草稿
- **THEN** 每個分組下的每一條都列出英文與繁中兩行，沒有只有一種語言的條目

## MODIFIED Requirements

### Requirement: 呈現草稿並在使用者點頭後才寫檔

技能 SHALL 在寫任何檔之前呈現建議版號、版號理由與分組的中英雙語日誌草稿，並等待使用者確認；使用者修改任一語言的條目時 SHALL 重新呈現。確認後 SHALL 依序：更新日誌 JSON 頂端加入新條目（date 為當天，每個條目含 `en` 與 `zh-TW`）→ workspace Cargo.toml 的 version → apps/desktop/src-tauri/tauri.conf.json 的 version → 以 cargo metadata 刷新 Cargo.lock → 以渲染腳本產出 `CHANGELOG.md` 與 `CHANGELOG.zh-TW.md` → 執行渲染腳本與下載指南腳本的測試及 `--check`。任一步失敗 SHALL 停在該步、印出原因、SHALL NOT 提交，已改的檔留在工作樹供使用者檢視。

#### Scenario: 使用者未點頭不寫檔

- **WHEN** 技能呈現草稿後使用者要求修改某一條的英文文字
- **THEN** 技能重新呈現修改後的雙語草稿，工作樹仍無任何改動

#### Scenario: 寫檔中途失敗

- **WHEN** 渲染腳本的 `--check` 回非零
- **THEN** 技能停止並印出比對差異，JSON、Cargo.toml、tauri.conf.json、`CHANGELOG.md` 與 `CHANGELOG.zh-TW.md` 的改動留在工作樹，沒有新 commit 與 tag
