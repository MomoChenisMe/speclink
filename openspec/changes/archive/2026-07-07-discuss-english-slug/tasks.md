## 1. speclink-core 建立流程擴充

- [x] 1.1 紅:於 crates/speclink-core 撰寫失敗測試,覆蓋需求「討論記錄以 --slug 覆寫檔名」的核心半邊——slug 覆寫值驗證(合法/非法值對照 spec 的「非法值一覽」Example 表:大寫、非 ASCII、底線、空白、首尾連字號、連續連字號、空字串拒絕;合法 kebab-case 接受)、建立討論時以覆寫值為檔名與 frontmatter slug 且 topic 維持原文、覆寫值與既有討論同名時報已存在;並覆蓋需求「未帶 --slug 時自主題衍生檔名」——後備衍生規則與現行逐位元一致(對照 spec 的「衍生規則對照」Example 表,含衍生為空報錯)。驗證:cargo test -p speclink-core 出現預期紅燈。
- [x] 1.2 綠:實作 slug 驗證與建立函式的覆寫參數(維持 core 不含終端呈現的邊界),1.1 測試全綠。驗證:cargo test -p speclink-core 全綠。

## 2. speclink-cli 旗標接線

- [x] 2.1 紅:於 crates/speclink-cli 撰寫失敗測試——discuss new 帶合法 --slug 時 stdout 顯示覆寫 slug、--json 的 slug 欄位為覆寫值且 topic 欄位為原文;非法 --slug 時非零 exit code、stderr 說明格式要求、openspec/discussions/ 不落檔。驗證:cargo test -p speclink-cli 出現預期紅燈。
- [x] 2.2 綠:接上 clap 的 --slug 旗標並呼叫 1.2 的 core 函式,2.1 測試全綠;確認未帶 --slug 的人眼與 --json 輸出逐位元不變。驗證:cargo test -p speclink-cli 全綠,且既有 discuss 相關測試無需修改即通過。
- [x] 2.3 對新增的 --slug 參數路徑套用 sharp-edges 檢查清單(speclink instructions --skill audit 取得)——重點:非法輸入不落檔、錯誤訊息指明格式要求、驗證只在 CLI 邊界做一次。驗證:檢查清單逐項核對並將結果記於本任務完成註記,無未處理項。
  審核註記(2026-07-07):壞蛋鏡——字元白名單(小寫英數/連字號)排除路徑跳脫與 frontmatter 注入,無安全開關;懶惰鏡——空字串拒絕且不落檔、無旗標即原行為、錯誤訊息含格式規則與範例;搞混鏡——remote 帶 --slug 明確報錯不靜默丟棄、驗證單一來源於 core、拒絕大寫而非靜默轉換、同名不覆寫。無未處理項。

## 3. 技能三處同步(需求「討論技能指示要求英文 slug」)

- [x] 3.1 落實需求「討論技能指示要求英文 slug」:更新 crates/speclink-core/assets/skills/discuss.md,建立討論記錄的指示改為「從主題衍生英文 kebab-case slug 並以 --slug 傳入;topic 維持使用者語言原文」。驗證:內容審視含該指示且與 spec 需求措辭一致。
- [x] 3.2 於乾淨樹(工作區無未提交改動)執行 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生 golden 基準,審視 diff 僅含 --slug 指引相關差異。驗證:再生後 cargo test -p speclink-core --test render_golden 綠燈。
- [x] 3.3 同步 repo 技能實例 .claude/skills/speclink-discuss/SKILL.md 與 .agents/skills/speclink-discuss/SKILL.md 的建檔指示,與 3.1 的內嵌資產語意一致。驗證:兩檔皆含「衍生英文 kebab-case slug 並以 --slug 傳入」指示。

## 4. 回歸收尾

- [x] 4.1 全工作區測試與回歸確認:cargo test 全綠;speclink discuss new 未帶 --slug 之行為與本變更前一致(既有討論檔案清單與輸出無差異)。驗證:cargo test 全綠、手動以中文主題各執行一次帶與不帶 --slug 的 discuss new 於臨時工作區並核對檔名與 frontmatter 後清除。
