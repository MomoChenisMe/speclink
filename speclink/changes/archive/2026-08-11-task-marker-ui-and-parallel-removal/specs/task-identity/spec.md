## MODIFIED Requirements

### Requirement: UI 剝離 ID 註解並以 stable ID 操作

任務清單呈現 SHALL 剝離 speclink-task 註解(使用者可見文字與無註解時相同),並 SHALL 剝離行首標記前綴(`[M]`/`[P]`,順序不敏感、各至多一次)——`[M]` 的身分 SHALL 以獨立旗標供呈現層使用(徽章呈現歸 desktop-app 正典),`[P]` 剝離後 SHALL NOT 留下任何呈現;清單項 SHALL 以 stable ID 作呈現 key(無 ID 舊檔退回 ordinal key);勾選操作 SHALL 以 stable ID 定址(無 ID 任務走 ordinal 相容路徑);樂觀就地改寫 SHALL 保留行首前綴與行尾註解原文。

#### Scenario: 桌面顯示無標記且勾選命中

- **WHEN** 桌面載入帶 ID 註解的 tasks.md 並勾選其中一項
- **THEN** 清單顯示的任務文字不含註解;勾選請求攜該任務的 tsk_ ID;tasks.md 該行翻轉且註解原文保留

#### Scenario: 標記前綴剝離且寫回保留

- **WHEN** 桌面載入含「- [ ] [M] 手動驗證匯入結果」與「- [x] [P] 舊任務」的 tasks.md 並勾選 `[M]` 該項
- **THEN** 兩列顯示文字皆不含前綴標記;`[M]` 列攜 manual 旗標、`[P]` 列無任何旗標;勾選寫回後該行的 `[M] ` 前綴原樣保留
