# Antigravity 與技能目錄標籤（2026-10-21）

## 需求與規劃

來源為使用者追加：「增加 Antigravity」「每個項目後面加上 skill 放置的目錄」。已經由 ingest 追加到既有提案，保留原始九項需求與全部 55 個已完成任務，新增任務 17.1–17.3；既有 Proxy 12.3 人工項維持未勾。Artifacts analyze 無 Critical／Warning，validate 通過。

## 完成行為

| 工具 | 畫面標籤 | 產生位置 |
|---|---|---|
| claude | claude [.claude] | .claude/skills |
| codex | codex [.agents] | .agents/skills |
| copilot | copilot [.github] | .github/skills |
| antigravity | antigravity [.agent] | .agent/skills |

- 四個 Desktop 入口共用 builtinTools 的清單及標籤：專案設定、初始化、啟用、checkout 綁定。控制項無障礙名稱含目錄，儲存仍是純工具名；預設選集仍為 claude，窄畫面可換行。
- Antigravity 進入 Tool::ALL，沿既有 init／update／reconcile／過期探測。採非 Claude frontmatter 及 /speclink-；Codex 的 $speclink- 內容各自保存。不生成 GEMINI.md 或修改全域工具設定。
- 取消 Antigravity 只清受管 speclink-*，保留自有技能、規則、Codex 檔案；symlink 不跟隨清理。撞名／撞技能目錄的舊描述子先拒絕，沒有寫入副作用，提供遷移提示。
- [官方 Antigravity 文件](https://antigravity.google/docs/skills)現行預設 .agents/skills，仍相容 .agent/skills。本專案明確採相容路徑，讓兩工具的不同呼叫語法分別生成；不將 .agent 說成官方現行預設。
- 資產依版本守門升至 v1.50.3，明示更新 golden 與 assets.lock，再以無更新旗標的 render 測試通過；本專案既有 claude／codex／copilot 受管技能透過 CLI update 同步。

## 同類位置排查

- Tool::ALL／Tool::parse／skills_dir／instructions_path：擴充唯一 registry；無指令檔的工具共用分支，描述子撞名與撞目錄驗證自動沿 registry。
- CLI：--tools 說明、非互動錯誤、互動第四題、空選集重問及未知工具名一致列四種工具。
- Node render：沿同一 Tool::parse，新增 target 接受與錯誤選項提示；build 重新產生 binding.d.ts。
- Desktop：SpeclinkPage、App 的初始化／啟用對話框、WorkspaceChooser 的 checkout 共用標籤；Tauri bind_checkout 空選集提示由 Tool::ALL 取得，並補雙工具共存測試。
- 既有明選 claude／codex／copilot 子集的範例與測試仍是合法組合，保留；只把宣稱完整工具矩陣的文字及斷言擴四。
- 舊 footprint 探測只推斷 Claude／Codex 的既有規則保持；設定檔內 antigravity 會正常預選，不把 .agent 內其他用途檔案當成已選工具。
- 中英 README、getting-started、configuration、Node SDK、product-status 同步。歷史整合清單及先前安全審查實測紀錄是當時證據，保留原文，不倒改歷史。

## 驗證結果

- TDD：兩項 Antigravity 共存／描述子測試先在舊版失敗；設定頁新標籤測試先失敗，再實作轉綠。
- Core lib 1,081 通過、1 ignored（含 Remote init 生成而不建本機 spec tree）；render golden 無更新旗標 63 通過。
- CLI init 單元 16 通過、integration 27 通過；CLI build 通過。
- Node addon 重建後，8 檔 57 通過；新增測試逐一比對 SDK Antigravity render 與 CLI init 的實際檔案。
- Desktop 52 檔 1,310 通過、正式 build 通過；checkout 最終定向重跑 45 通過。
- Desktop core 358 通過；原生 checkout 25 通過。首次誤用 connections::tests 篩選得到 0 項，已改正為 connections::checkout_tests，沒有把 0 項計為覆蓋。
- 實際在空暫存目錄以四工具 init 核對，預設政策下技能數量為 Claude 18、Codex／Copilot／Antigravity 各 17。
- 最後 scripts 全組 386 通過、0 失敗；git diff --check 通過。

## 尚未執行

沒有 Antigravity 實機互動驗收；Windows／UNC／NAS 與公司 Proxy 人工驗收仍未執行。本次不打包、不蓋章、不封存、不提交。原 13 項 Round 2 清零先於此追加範圍，新增範圍需要重新完整品質檢查。完整 CI 尚有 [品質紀錄](quality-fixes-2026-10-21.md)記載的既有 skill 原稿格式守門失敗，未宣稱整個 CI 綠燈。

最終任務狀態：58/59 完成，程式任務 58/58；唯一未勾為人工 Proxy 12.3。三項新增任務均由 task done 記錄，未直接修改 checkbox。
