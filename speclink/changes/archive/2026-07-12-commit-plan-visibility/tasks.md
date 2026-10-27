## 1. 守則測試先行（紅）

- [x] 1.1 撰寫渲染內容測試：斷言 claude 與 codex 渲染產物中的 speclink-commit 技能滿足（a）「生成 commit 訊息」步驟位於使用者確認步驟之前、（b）Guardrails 含可見性守則字句（呼叫確認工具前，commit 計畫與 commit 訊息必須已作為可見文字輸出；確認問題不得指涉未曾輸出的內容）。檔案：crates/speclink-core/tests/render_golden.rs（或同層新增測試檔，命名遵循既有慣例）。驗證：cargo test -p speclink-core 執行後新測試失敗（紅）——現行資產尚未含守則與新順序。對應 spec Requirement: commit 確認閘門所見即所簽（scenario「渲染產物將訊息生成排在確認之前」「渲染產物含可見性守則」）。

## 2. 修改事實來源（綠）

- [x] 2.1 重排 crates/speclink-core/assets/skills/commit.md 的步驟順序：收集檔案（change artifacts＋tracked source files＋unrelated 分組）→ 生成 commit 訊息 → 以單一可見訊息一次輸出「Commit Plan（分組檔案清單）＋commit 訊息全文」→ AskUserQuestion 單一確認閘門（維持 Commit as shown / Include all dirty files / Customize / Archive first, then commit together 四選項）→ 選擇性暫存與提交。行為契約：使用者確認當下，完整檔案清單與訊息全文已存在於對話可見文字中。驗證：1.1 的順序斷言轉綠。
- [x] 2.2 修改 archive 子流程（Archive first, then commit together）收尾：archive 執行完成後 SHALL 重新輸出更新後的 Commit Plan 與含 Archived: yes 的 commit 訊息，並再次經 AskUserQuestion 確認後才執行暫存與提交。檔案：crates/speclink-core/assets/skills/commit.md。驗證：內容審視——子流程段落含「重新輸出＋再次確認」步驟，對應 spec scenario「archive 子流程後重新確認」。
- [x] 2.3 Guardrails 段新增可見性守則（呼叫 AskUserQuestion 前計畫與訊息必須已作為可見文字輸出；確認問題文字不得指涉對話中未曾輸出的內容），並保留既有「AskUserQuestion 不可用時以純文字提問」降級——降級路徑同樣 SHALL 先輸出計畫與訊息後才提問。檔案：crates/speclink-core/assets/skills/commit.md。驗證：1.1 的 guardrail 斷言轉綠，對應 spec scenario「確認工具不可用時的降級」。
- [x] 2.4 移除確認之後無把關的「Show the generated message to the user and allow editing before proceeding」路徑——修改訊息的機會併入確認閘門之前（Customize 選項或使用者自由文字要求）。檔案：crates/speclink-core/assets/skills/commit.md。驗證：內容審視——確認步驟之後不存在任何顯示或修改訊息的步驟。

## 3. golden 再生與 repo 實例同步

- [x] 3.1 於乾淨樹（除本 change 已知修改外，git status 不得有其他未提交變更，避免把無關狀態烙進 golden）執行 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生四份 snapshot：crates/speclink-core/tests/golden/claude.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md。驗證：審視 git diff 僅含本次預期的 commit 技能段落變更，且 cargo test -p speclink-core --test render_golden 通過。
- [x] 3.2 同步 repo 技能實例 .claude/skills/speclink-commit/SKILL.md 與 .agents/skills/speclink-commit/SKILL.md 至新版渲染內容。驗證：diff 比對兩實例與 assets 渲染結果，除既有的 / 與 $ 呼叫前綴差異外逐行一致。

## 4. 回歸確認

- [x] 4.1 執行 cargo test -p speclink-core 全套測試。行為契約：本 change 未動引擎程式碼，CLI 人眼與 --json 輸出不變。驗證：測試輸出 0 failed；抽查 speclink status --change commit-plan-visibility --json 輸出結構正常（changeName、artifacts 欄位存在且 camelCase）。
