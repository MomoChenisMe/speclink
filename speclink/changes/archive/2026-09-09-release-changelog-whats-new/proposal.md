## Why

每個 release 版本目前沒有人看得懂的更新內容：Release 說明只有下載指南，GitHub 自動產的 changelog 因 commit 直進 main、無 PR 而只剩一行比較連結；repo 也沒有 CHANGELOG 檔。桌面 app 更新後使用者不知道這版改了什麼。v0.1.3 到 v0.2.0 之間有 95 個 conventional commit，要變成使用者看得懂的白話日誌，需要有人（或 Claude）策展——這件事只能在發版當下由技能完成，CI 裡沒有 Claude。

目標使用者：維護 speclink 的開發者（發版技能）與桌面 app 的所有使用者（更新日誌彈窗）。對應階段：發版是 archive 之後、tag 之前的收尾動作；彈窗屬 desktop 啟動流程。

來源討論：release-changelog-whats-new。

## What Changes

- **新增 repo 本地發版技能 `/release`**（`.claude/skills/release/SKILL.md`）：執行即發版。守門（在 main、工作樹乾淨、與 origin/main 同步、HEAD 的 CI 工作流全綠、列出進行中變更請使用者確認）→ 由上次 tag 到 HEAD 的 conventional commit 判定版號（feat→minor、fix／perf→patch、0.x 階段 breaking 封頂 minor；只有 chore／test／refactor／docs／ci／build／style 時警告）→ 把 commit 策展成白話繁中日誌（封存變更的 commit 優先、一變更一條，其餘 feat／fix 逐筆，雜訊不進；分組「新功能／修正／改善」）→ 呈現版號與草稿，使用者點頭後才寫檔 → 一次改齊 workspace Cargo.toml、tauri.conf.json、更新日誌 JSON 頂端條目、刷 Cargo.lock、以腳本自 JSON 產出 CHANGELOG.md → commit、附註 tag、push。這是 repo 自用技能，不是 speclink 產品技能：不進 ASSET_VERSION／golden／assets.lock，不裝進使用者專案，目錄名不以 `speclink-` 開頭（避免 update 的孤兒清理）。
- **更新日誌 JSON 為唯一真相**：`apps/desktop/src/release-notes/release-notes.json`，最新在前，每筆 version（不帶 v，與 tauri.conf.json 同字面）、date、sections（title 限「新功能／修正／改善」、items 繁中字串）。
- **腳本衍生兩份呈現**：新增 `scripts/release-notes-render.mjs`，同一份渲染邏輯（a）把全部條目畫成 repo 根 `CHANGELOG.md`；（b）把單一版本的條目畫成 markdown 片段。`scripts/release-notes.mjs` 產下載指南時把該版片段接在指南之後，取代目前空的自動 changelog 位置。
- **release job 兩道新守門**（`.github/workflows/release.yml`）：JSON 頂端 version 必須等於 tag 去 v（同版守門由三處擴為四處）；重跑渲染比對 CHANGELOG.md，不一致即紅燈（fail-closed，比照 latest.json）。
- **桌面更新日誌彈窗**：desktop 打包 JSON。啟動時 app 版號等於 JSON 頂端版號、且與本機記住的「已看過版號」不同時，以 modal 彈出「更新內容」，列出比已看過版號新的全部條目（已看過版號不在清單內時只列頂端一筆）；關閉即記下現版號。無記錄（首次安裝）不彈、直接記現版號；dev 建置永不自動彈。設定頁軟體更新卡加「更新日誌」按鈕，開同一個對話框的瀏覽全部模式，關閉不寫記錄。
- **回填首筆**：以 v0.1.3..v0.2.0 的 commit 回填 0.2.0 條目，使 CHANGELOG.md 與桌面瀏覽模式一上線就有內容。
- **詞彙**：`openspec/LANGUAGE.md` 新增「更新日誌」詞條（avoid：更新資訊、release notes、changelog 於使用者可見文案）。
- release.yml 標頭的 Usage 註解改為指向 `/release` 技能。

相容性影響：Release 說明的下載指南內容不變，其後多出該版更新內容；`generate_release_notes` 維持開啟，「Full Changelog」比較連結仍在最後。無 CLI 指令變動、無 `--json` 變動。既有 Release（v0.2.0 以前）不回改。

涉及的技能與工具：只新增 repo 本地的 `.claude/skills/release/`（claude）；不鏡射 `.agents/`（目前無本地技能先例，用 Codex 發版時再補）。不影響任何 speclink-* 產品技能。

## Capabilities

### New Capabilities

- `release-skill`：repo 本地發版技能的契約——守門條件、版號判定規則、日誌策展規則、寫檔順序與 tag／push 邊界。步驟 3 掃描到的最近規格：desktop-release 只涵蓋 tag push 之後的管線與產物；commit-skill 是產品技能、只管單一變更的提交；dev-harness 是本機開發環境啟動。沒有規格涵蓋「tag 之前由人與 Claude 完成的發版動作」。
- `release-notes`：更新日誌的資料契約與衍生物——JSON 檔位置與欄位、CHANGELOG.md 與 Release 說明片段的渲染規則、release job 的四處同版與渲染比對守門。最近規格 desktop-release 的「Release 說明含下載指南」只規範指南本身，「更新描述檔隨 release 發布」規範的是 updater 用的 latest.json，兩者都不涵蓋人讀的更新內容從哪來。

### Modified Capabilities

- `desktop-app`：新增需求「更新日誌彈窗」——版號變更後首次啟動自動彈出、看過即止、設定頁瀏覽入口、首次安裝與 dev 不彈。
- `desktop-release`：「Release 說明含下載指南」改為指南之後接該版策展更新內容（原文「自動產生的 changelog 接續其後」）。

## Impact

- Affected specs：新增 release-skill、release-notes；修改 desktop-app、desktop-release。
- Affected code：
  - New：
    - `.claude/skills/release/SKILL.md`
    - `apps/desktop/src/release-notes/release-notes.json`
    - `apps/desktop/src/release-notes/release-notes.ts`
    - `apps/desktop/src/core/whatsNew.ts`
    - `apps/desktop/src/components/ReleaseNotesDialog.tsx`
    - `apps/desktop/src/__tests__/whatsNew.test.ts`
    - `apps/desktop/src/__tests__/releaseNotesDialog.test.tsx`
    - `scripts/release-notes-render.mjs`
    - `scripts/release-notes-render.test.mjs`
    - `CHANGELOG.md`
  - Modified：
    - `.github/workflows/release.yml`
    - `scripts/release-notes.mjs`
    - `scripts/release-notes.test.mjs`
    - `apps/desktop/src/App.tsx`
    - `apps/desktop/src/views/AppSettingsView.tsx`
    - `apps/desktop/src/i18n/messages.ts`
    - `apps/desktop/src/__tests__/appSettingsView.test.tsx`
    - `openspec/LANGUAGE.md`
  - Removed：無。
- 影響的 app／crate：apps/desktop（前端）、repo 腳本與 CI；不動任何 Rust crate。
- 依賴：無新套件。JSON 由 Vite 原生 import；modal 沿用 `@speclink/ui` 既有 AlertDialog 元件。
