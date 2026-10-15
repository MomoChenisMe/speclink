# 官方 0.9 升級功能決策表

日期：2026-10-15。Fork/共同祖先/官方 SHA、逐檔提交證據見 `FORK_CUSTOMIZATION_INVENTORY.md`。

分類含義：KEEP 完整保留、MERGE 雙方行為整合、REPLACE 證明功能被涵蓋後採官方、ADAPT 保留行為配合新版架構、REVIEW 尚需指定環境或人工確認。此表由合併前盤點形成；整合後程式與本機測試已落實，最新結果見 `UPGRADE_REPORT.md`，Windows／實機項目仍屬待驗證。

| ID | 分類 | 決定及涵蓋證據 | 必須驗證的行為 |
|---|---|---|---|
| C01 | REPLACE | 官方 Tool::ALL 含三工具；init/update/adopt、CLI/Desktop 選擇及停用清理已有對應 tests。採官方，不保留共享 SyncTarget 的重複分支。 | 單選 Copilot、三工具同選、移除工具保留自有技能；不得清掉另一工具。 |
| C02 | REPLACE | 官方 skills.rs 對 Copilot 使用 `.github/skills`、`/speclink-`，對 Codex 使用其原生語法；官方具 symlink／目錄清理測試。舊共享路徑是實作差異，不是必須維持的產品設定。 | 舊 Fork tools 清單可 update；Copilot 生成完整 SDD 技能；既有 Codex 路徑保留。 |
| C03 | REPLACE | 官方 SDK render.rs、型別與測試有 Copilot target。沿用官方各目標語法，不要求 byte-identical。 | SDK skills.render Copilot target 正確；非 builtin custom/neutral 行為不退化。 |
| C04 | MERGE | 官方缺 OS 預設；保留 Fork EnvOverrides.system_locale／Host 注入，整合官方新政策欄位。 | tw/ja/en、明示及環境優先、server OS、不回寫；review_rules 不受影響。 |
| C05 | KEEP | 官方缺 workflow-config languages 與 ConfigResponse.languages；保留相容可選欄位及嚴格輸出驗證。 | 本地／遠端 JSON camelCase、舊 server 明確錯誤、未知語言拒絕、show 可讀舊值。 |
| C06 | MERGE | 官方 v1.48.0 新技能為主；僅保留 OS 語言條款及 languages 查詢，再由整合引擎生成。 | golden 與 ASSET_VERSION 同源，新 review rules／品質站內容存在，沒有舊技能覆蓋新版。 |
| C07 | MERGE | 採官方 theme key/API/GeneralPage/boot script，加入舊 `speclink.uiTheme` 讀取遷移；移除舊控制器前先移轉全部行為。 | Light/Dark/System 六格矩陣、舊 key 恢復、新 key 優先、system 清除舊明示值、首繪與重開。 |
| C08 | MERGE | 使用者已明確選擇 Fork 保存失敗語意；接到官方單一設定列，不保留第二套 UI。 | 寫入失敗原配色／選擇不變、可讀錯誤、重試成功。 |
| C09 | ADAPT | 官方主題事件與 native setTheme 保留；整合 Fork await-listen／首份快照／cleanup／resize 修正至新版 Windows/macOS panel。 | lazy 首開、隱藏重開、workspace 不變仍同步、系統改變、StrictMode/unmount 競態。 |
| C10 | MERGE | 保留官方 0.9 色票、CSS、Markdown；補可傳入的 Sonner theme，不以舊 CSS 替換新版。 | 明示淺色配系統深色及反向組合、已顯示 toast 換色；Server Web 無覆寫仍系統模式。 |
| C11 | ADAPT | 官方新分類式設定為底，增加共用 readonly 目錄，不複製舊整頁設定。 | 每個分類可見、local/remote/null、active project 切換、offline/conflict/recovery。 |
| C12 | KEEP | 保留 pre-wrap、可選取及 anywhere 換行，移轉三種特殊空白案例。 | textContent 與計算樣式；WebView 真實選取複製另如實標註。 |
| C13 | ADAPT | marker/data 隔離仍獨有；適配官方新 Tauri platform config、Windows panel 與啟動順序。 | 安裝版共存、無 marker 不啟用、可搬移、不可寫不回退、每個 WebView data directory。 |
| C14 | ADAPT | 保留 client_store/CLI guard／updater 限制，覆蓋官方新 menu/about/update 入口。 | Portable 不碰 Keychain/credential file，僅 session/env token，不能呼叫 installer updater。 |
| C15 | ADAPT | 保留 Desktop Build 四 target 與自用簽章，接官方 platform-specific config。正式 release workflow 採官方且不發布。 | workflow 語法、config 合成、版本同源、sidecar/resources、實際本機 Tauri build。 |
| C16 | ADAPT + REVIEW | 保留 Actions ZIP，不新增正式 Release asset。Windows runner 真正產出、解壓、執行與搬移驗證前不得標通過。 | staging tests、本機可行檢查；Windows NSIS/ZIP smoke 保留可執行流程並明列尚未執行。 |
| C17 | KEEP | registry 是技能正典，與 Claude 安裝選集無關；適配官方 registry 新內容。 | 沒有 `.claude/skills` 仍可比對；新增／移除 skills 分組有守門。 |
| C18 | MERGE | 保留三工具與客製歷史，正式規格按 requirement 合併，不由整份 Current/Incoming 取代。 | 無重複 requirement／agent 設定，archive/evidence/trace 保留；文件與 schema/實作一致。 |

## 重大決策的使用者確認

1. 已確認保留 Fork 的外觀保存失敗行為。
2. 已確認 Portable 只上傳 Actions artifact，這次不擴充正式 Release workflow。
3. 原始指示已授權重複功能優先採官方；C01–C03 的替代須以前述程式／測試證據及整合後驗證為準，不可僅因名稱相同就刪除。
4. 尚未執行的 Windows／實機驗證屬 REVIEW，不是默認通過，也不會觸發 push、Release 或遠端 workflow。


## 整合後證據（2026-10-16）

- C01–C03：官方原生實作、三工具重新生成、Core/CLI/Desktop/SDK 測試通過；另用 f9d30713 真正舊 Copilot 檔執行遷移，受管檔移轉、自訂技能與設定保留。
- C04–C06：Rust 工作區 2,966 項通過，golden／assets.lock 同源，技能產物 v1.48.1；官方 review rules 保留。
- C07–C12：UI 1,320、Desktop 1,222、Server Web 244 項通過，三專案完整型別檢查通過；實機視覺／選取複製另列待驗。
- C13–C16：Portable staging、Host/CLI/Remote 邊界測試與 workflow 靜態檢查通過，macOS `.app` 建置成功；Windows NSIS/ZIP/GUI/共存仍未實際執行。
- C17–C18：364 項 scripts 測試通過，含 registry 技能分組、文件雙語對等與 release 守門；.speclink.yaml 原文保留。
