## MODIFIED Requirements

### Requirement: 更新日誌策展為使用者語氣的中英雙語條目

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能 SHALL 把範圍內 commit 策展為中英雙語條目並分組為 features（feat）、fixes（fix、perf）、improvements（其餘有保留價值者）。每個條目 SHALL 同時有繁體中文與英文兩種寫法，兩者描述同一件事：繁中 SHALL 用使用者語氣與 `speclink/LANGUAGE.md` 的正典詞彙；英文 SHALL 用簡化技術英文（一句一事、主動語態、SHALL NOT 用完成式）。封存變更的 commit（主旨形如 `speclink(<變更名>): …`）SHALL 優先處理、一個變更一條，以其摘要與該變更 proposal 的 Why 改寫成使用者語氣；不屬於任何變更的 feat、fix、perf SHALL 逐筆改寫。chore、test、refactor、docs、ci、build、style SHALL NOT 出現在條目中。技術名詞（CLI、worktree、Tauri、slug 等）在兩種語言 SHALL 保留原文。條目 SHALL NOT 照抄 commit 主旨。

#### Scenario: 封存變更收成一條

- **WHEN** 範圍內有 `speclink(discussion-spinout-hold): 一份討論可以分期轉出多個變更` 與其開發過程的三筆 feat／fix commit
- **THEN** 日誌只出現一條 features 條目描述分期轉出，該條同時有英文與繁中，開發過程的三筆不另列

#### Scenario: 雜訊不進日誌

- **WHEN** 範圍內有 `chore(release): 版號升至 0.2.0` 與 `test(server): 消除 SSE 保留窗地板的競態`
- **THEN** 兩筆都不出現在任何分組

#### Scenario: 每條都有兩種語言

- **WHEN** 技能呈現草稿
- **THEN** 每個分組下的每一條都列出英文與繁中兩行，沒有只有一種語言的條目
