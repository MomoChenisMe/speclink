# 回歸對照記錄 — remote-section-in-speclink-yaml

日期:2026-07-05
方法:自我基線雙沙盒(先例:store-trait-and-fs-adapter、config-system-rework、verb-contract-and-remote-client)

## 基線

- 舊 exe:`git worktree add <tmp> HEAD`(42e5b21)建置的 release binary——動工前 working tree 的修改全數屬於本 change,HEAD 即乾淨基線,無遺留 hunks 需標注。
- 新 exe:working tree 的 release binary。

## 比對(regress.ps1,session scratchpad)

32 個區塊逐 byte 比對:init(含重複 init 錯誤路徑)、list(人眼/--json/--specs/--no-color)、status(人眼/--json/--no-color)、instructions(apply --json/預設/--skill tdd)、show、analyze --json、validate、artifact cat、task done(含 already-done 錯誤路徑/--json)、in-progress、discuss(new/list/--json)、update、schemas、archive,加上最終沙盒樹全檔內容比對。

正規化:CRLF→LF、沙盒路徑三種拼法、ISO 時間戳/日期、clap Usage 行的 exe 檔名。

**結果:PASS——fs 模式輸出與檔案效果逐 byte 一致,無刻意更新。**

註:link/unlink/auth 不在比對範圍——它們屬 remote-connection capability,本 change 刻意改變其檔案效果與訊息,由 remote_section.rs/remote_connect.rs 整合測試覆蓋。

## 煙霧測試(release exe)

1. `speclink link <url> --repo backend`:`.speclink.yaml` 出現 `remote:` 區段,既有 `tools:` 值保留 ✓
2. remote 模式生效:`list --json` 回 auth 提示、exit 1;與 openspec/ 並存時恰一行並存警告 ✓
3. `speclink unlink`:區段移除、`tools:` 保留;`list --json` 回到 fs 模式,形狀 `{"changes": []}` ✓
4. 放置殘留 `.speclink.remote.yaml`:恰一行 `speclink: warning:` 遷移警告,指令照常 fs 模式執行、exit 0 ✓
