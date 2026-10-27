## Context

發版目前是手動三步：改 workspace Cargo.toml 與 apps/desktop/src-tauri/tauri.conf.json 的 version、刷 Cargo.lock、壓 tag push；release.yml 由 `v*` tag 觸發後全自動（CLI 壓縮檔、桌面安裝檔、latest.json、Docker、npm、tap）。Release 說明由 scripts/release-notes.mjs 產下載指南，其後由 action-gh-release 的 generate_release_notes 接自動 changelog——但 commit 直進 main、無 PR，自動 changelog 只剩一行「Full Changelog」比較連結。repo 無 CHANGELOG 檔。

桌面 app 已有：updater 狀態機（apps/desktop/src/core/updater.ts，純 reducer）、設定頁軟體更新卡（顯示現版號與「檢查更新」）、localStorage「記版號、看過不再顯示」先例（apps/desktop/src/assetPrompt.ts）、modal 先例（apps/desktop/src/components/MigrationDialog.tsx 以 @speclink/ui 的 AlertDialog 構成）。dev 建置與安裝版共用 localStorage 且同版號（見 tauri-dev-static-dist 備忘）。

參考實作：wadpilot 專案的 release-notes 技能與 useWhatsNew——單一 release-notes.json 為版號真相、bundle 帶它顯示、彈窗以 lastSeenVersion 比對、dev 不自動彈。本設計採其資料形狀與彈窗判定，差異在：技能負責壓 tag（speclink 管線由 tag 觸發），且另以腳本衍生 CHANGELOG.md 與 Release 說明片段。

來源討論 release-changelog-whats-new 已排除：手寫 CHANGELOG.md 為真相（解析 markdown 脆弱）、md 與 json 都手寫、技能止於寫檔 tag 交 CI、changelog 放 git 外、執行期抓 GitHub、橫幅代替 modal、首次安裝也彈、做成產品技能、CLI changelog 子指令。

## Goals / Non-Goals

**Goals:**

- 每個 release 版本都有一份白話繁中更新日誌，且來源只有一份（JSON）。
- 發版由一個技能一次做完：守門、版號、日誌、寫檔、tag、push；不可逆點（push）之前停一次。
- Release 說明與 repo 根 CHANGELOG.md 由同一支腳本從 JSON 畫出，CI 比對不一致即擋。
- 桌面 app 版本更新後首次啟動彈出更新內容，看過即止，設定頁可隨時瀏覽全部版本。

**Non-Goals:**

- latest.json 的 notes 欄位（更新橫幅預覽下一版內容）——延後。
- `.agents/skills/` 鏡射本地技能——用 Codex 發版時再補。
- CLI／server 專屬的日誌入口或 `speclink changelog` 子指令。
- 雙語日誌內文；介面標籤才走 i18n。
- 回改 v0.2.0 以前的 GitHub Release 說明。
- 發版失敗後的重跑（移 tag 重推）——維持 release-pipeline-npm-pitfalls 備忘的手動程序。

## Decisions

### 更新日誌 JSON 為唯一真相

檔案 `apps/desktop/src/release-notes/release-notes.json`，陣列、最新在前。每筆：

```json
{
  "version": "0.3.0",
  "date": "2026-09-10",
  "sections": [
    { "title": "新功能", "items": ["…"] },
    { "title": "修正", "items": ["…"] },
    { "title": "改善", "items": ["…"] }
  ]
}
```

- `version` 不帶 v：與 tauri.conf.json 的 version、Tauri `getVersion()` 回傳值、latest.json 的 version 同字面，桌面端比對免去前綴處理。tag 仍為 `vX.Y.Z`。
- `sections[].title` 只允許「新功能／修正／改善」三值，`items` 為非空繁中字串陣列；空 section 不寫入。
- 放在 desktop 原始碼樹下而非 repo 根：desktop 是唯一要打包它的消費者，Vite 原生 import JSON 免設定；腳本與 CI 讀同一路徑。
- 型別檔 `apps/desktop/src/release-notes/release-notes.ts` 匯出 `ReleaseNotesEntry`／`ReleaseNotesSection` 型別與 `RELEASE_NOTES` 常數（JSON 的 title 由 string 收窄為字面聯集，比照 wadpilot）。

替代方案：CHANGELOG.md 為真相解析成 JSON（解析 markdown 脆弱，標題層級一歪即錯）；兩份都手寫（兩個真相必漂移）。

### 渲染腳本一支三用

新增 `scripts/release-notes-render.mjs`，純 Node、無依賴，匯出兩個函式並提供 CLI：

- `renderEntry(entry)` → 單一版本的 markdown 片段：`## X.Y.Z（YYYY-MM-DD）` 標題，各 section 以 `### 新功能` 等三級標題接 `- ` 條目。
- `renderChangelog(entries)` → 完整 CHANGELOG.md：首行 `# 更新日誌`，其後依序接每筆 `renderEntry`，條目間空一行，檔尾一個換行，一律 LF。
- CLI 三種用法：`--write` 讀 JSON 寫出 repo 根 CHANGELOG.md；`--version X.Y.Z` 把該版片段印到 stdout（找不到該版以非零結束並印出 JSON 頂端版號）；`--check` 重算與 CHANGELOG.md 比對，不一致以非零結束並印出 diff 摘要。比對前把讀入檔的 CRLF 正規化為 LF（Windows checkout 可能被 autocrlf 改寫）。
- `scripts/release-notes.mjs`（下載指南）改為：印完指南後 import `renderEntry`，找出 tag 對應版本的條目接在指南之後；JSON 沒有該版即非零結束。generate_release_notes 維持開啟，「Full Changelog」連結仍在最後。

替代方案：兩支腳本各自渲染（同一格式兩份實作必漂移）；在 release-notes.mjs 內直接讀 JSON 不抽模組（CHANGELOG.md 與 Release 片段就無法共用同一段渲染）。

### release job 守門由三處同版擴為四處並比對渲染

release.yml 的 desktop job 既有「Assert tag matches desktop version」步驟（比對 tauri.conf.json 與 tag，跑在所有建置之前）擴充兩項：

1. JSON 頂端 `version` 必須等於 tag 去 v，否則 `::error::` 點名兩邊版號、非零結束。
2. 執行 `node scripts/release-notes-render.mjs --check`，CHANGELOG.md 與 JSON 不一致即非零結束。

放在建置之前而非 release job：十分鐘建置之後才紅燈是浪費；desktop job 的 runner 預設有 node，既有步驟已用 `node -p`。matrix 每個平台各跑一次是重複但便宜（毫秒級）。release job 只需維持「產指南＋片段」一步。

### 發版技能為 repo 本地技能、執行即發版、push 前停一次

`.claude/skills/release/SKILL.md`（目錄名不以 `speclink-` 開頭，避開 `speclink update` 的孤兒清理；不進 ASSET_VERSION／golden／assets.lock）。步驟：

1. **守門**（任一不成立即停、不改任何檔）：目前分支為 main；`git status --porcelain` 為空；`git fetch origin` 後 HEAD 等於 origin/main；`gh run list --workflow CI --commit <HEAD sha> --json conclusion` 有一筆且 conclusion 為 success（沒有紀錄或仍在跑即停，請使用者等）；`speclink list --json` 有 in-progress 變更時列出名稱請使用者確認繼續（不擋）。
2. **範圍**：`git describe --tags --abbrev=0` 取上次 tag；`git log <tag>..HEAD --pretty=format:'%s%n%b'` 為唯一輸入。
3. **版號**：逐筆解析 conventional type，取最高影響：`!` 或 BREAKING CHANGE footer→major、feat→minor、fix／perf→patch；上次 tag 低於 1.0.0 時 breaking 封頂 minor（升到 1.0.0 須使用者明示）；只有 chore／docs／refactor／test／ci／build／style 時警告「這批沒有面向使用者的變更」並問是否仍發；無法解析的 commit 當 patch 並列出請使用者確認。版號為建議，使用者可覆寫。
4. **日誌策展**：封存變更的 commit（`speclink(<name>): …`）優先，一個變更一條，以其摘要與該變更的 proposal Why 改寫成使用者語氣；不屬於任何變更的 feat／fix／perf 逐筆改寫；chore／test／refactor／docs／ci／build／style 不進。分組「新功能」（feat）、「修正」（fix／perf）、「改善」（其餘有保留價值者）。技術名詞（CLI、worktree、Tauri、slug 等）保留原文。
5. **停一次**：呈現建議版號、bump 理由、日誌草稿；使用者點頭才進下一步，修改即重呈現。
6. **寫檔**（順序固定）：JSON 頂端 prepend 條目（date 為當天）→ workspace Cargo.toml version → tauri.conf.json version → `cargo metadata --format-version 1 > /dev/null` 刷 Cargo.lock → `node scripts/release-notes-render.mjs --write` 產 CHANGELOG.md → `node --test scripts/release-notes-render.test.mjs scripts/release-notes.test.mjs` 與 `node scripts/release-notes-render.mjs --check` 全綠。任一步失敗即停在該步、印出失敗原因，已改的檔留在工作樹供使用者檢視（`git checkout -- .` 可整批還原）；不 commit。
7. **提交與發布**：以 conventional-commit 技能提交，訊息 `chore(release): 版號升至 X.Y.Z`；附註 tag `git tag -a vX.Y.Z -m "vX.Y.Z"`；`git push --atomic origin main vX.Y.Z`（main 與 tag 同一次推、全成或全不成）。
8. **收尾**：印出 Release workflow 的網址與 release-pipeline-npm-pitfalls 備忘的三項提醒（job 綠不等於套件在、新套件 packument 延遲、發完重裝本機 app）。

替代方案：wadpilot 模式（技能只寫檔、CI 讀 JSON 頂端壓 tag）——與使用者「執行即發版」定義不合，且 speclink 管線本就由 tag 觸發；做成產品技能（會裝進所有使用者專案）。

### 桌面更新日誌純判定與 modal 兩模式

`apps/desktop/src/core/whatsNew.ts`（純函式、不依賴 Tauri，比照 core/updater.ts）：

```ts
export type WhatsNewDecision =
  | { kind: "show"; entries: ReleaseNotesEntry[] }
  | { kind: "record" }
  | { kind: "none" };

export function whatsNewDecision(input: {
  appVersion: string;
  entries: ReleaseNotesEntry[];
  lastSeen: string | null;
  dev: boolean;
}): WhatsNewDecision;
```

規則依序：`dev` 為真→none；entries 為空或頂端 version 不等於 appVersion→none（dev 或建置鏈版號不齊時不彈）；lastSeen 為 null→record（首次安裝：不彈、記現版號）；lastSeen 等於 appVersion→none；否則 show，entries 為頂端起到 lastSeen 那筆之前的全部（lastSeen 不在清單內時只含頂端一筆）。同檔提供 `readLastSeenVersion(storage)`／`writeLastSeenVersion(version, storage)`，localStorage 鍵 `speclink.lastSeenVersion`，壞值視為 null（比照 assetPrompt.ts）。

`apps/desktop/src/components/ReleaseNotesDialog.tsx`：props `open`、`mode: "whatsNew" | "browse"`、`entries`、`onOpenChange`。以 @speclink/ui 的 AlertDialog 構成（MigrationDialog 先例）。whatsNew 模式標題「X.Y.Z 更新內容」（X.Y.Z 為 entries 頂端版號）、主鈕「知道了」；browse 模式標題「更新日誌」、主鈕「關閉」、entries 為全部條目。每筆以「版號（日期）」為小標，其下三類分組列點；內容區可捲動、寬度沿 markdown 行寬上限。

接線在 App.tsx（React 本地狀態，不進 zustand store）：既有「設定頁軟體更新卡的常駐現版號」effect 取得 appVersion 後呼叫 `whatsNewDecision`（`dev` 傳 `import.meta.env.DEV`）；show→開 whatsNew 模式；record→只寫 lastSeen。whatsNew 模式關閉時寫入 appVersion；browse 模式關閉不寫。設定頁 `AppSettingsUpdaterProps` 加 `onShowReleaseNotes`，更新卡在「檢查更新」旁加「更新日誌」按鈕開 browse 模式。updater 未注入（非 Tauri 殼）時兩個入口都不存在。

替代方案：橫幅（看過的判定模糊，內容也塞不下）；判定放進 store（純 UI 開關，store 無需知道）；比對「appVersion ≠ lastSeen」而不檢查頂端版號（dev 或版號不齊時會彈出錯版內容）。

### 回填 0.2.0 與詞彙

- JSON 首筆以 v0.1.3..v0.2.0 的 commit 依上述策展規則回填 0.2.0（date 取 v0.2.0 tag 的提交日期），使 CHANGELOG.md 與瀏覽模式一上線就有內容；0.1.x 不回填。
- `openspec/LANGUAGE.md` 新增詞條「更新日誌」：definition＝每個 release 版本的白話更新內容（JSON 為真相、CHANGELOG.md 與 Release 說明為衍生；桌面設定頁按鈕與瀏覽模式標題）；avoid＝更新資訊、release notes、changelog（使用者可見文案中）；why＝與「軟體更新」卡同族、「日誌」點明是逐版紀錄。彈窗自動模式的標題用「更新內容」不另立詞條——它是「更新日誌」中某一版的內容，同族不同層。

## Implementation Contract

**Behavior**

- 開發者執行 `/release`：守門不過時印出原因即停、工作樹零改動；通過後看到建議版號與日誌草稿，點頭後工作樹多出一個 commit（六個檔：JSON、Cargo.toml、tauri.conf.json、Cargo.lock、CHANGELOG.md，加首次的 CHANGELOG.md 新檔）與一個附註 tag，main 與 tag 已推到 origin。
- push tag 後 release.yml：desktop job 第一步斷言 tag、tauri.conf.json、JSON 頂端三者同版且 CHANGELOG.md 與 JSON 一致，否則紅燈、不建置；release job 的 Release 說明＝下載指南＋該版更新內容片段＋「Full Changelog」連結。
- 桌面使用者從 0.2.0 更新到 0.3.0 後首次啟動：彈出「0.3.0 更新內容」，按「知道了」後重開不再彈；跳過 0.3.0 直接到 0.3.1 時同時列出 0.3.1 與 0.3.0。全新安裝不彈。設定頁軟體更新卡的「更新日誌」按鈕隨時開全部版本，關閉不影響自動彈判定。

**Interface / data shape**

- JSON 條目：`{ version: string（X.Y.Z）, date: string（YYYY-MM-DD）, sections: { title: "新功能"|"修正"|"改善", items: string[] }[] }[]`，最新在前。
- 渲染腳本 CLI：`node scripts/release-notes-render.mjs --write | --version X.Y.Z | --check`；exit 0 成功；`--version` 找不到版本、`--check` 不一致皆 exit 1 並於 stderr 說明。
- `whatsNewDecision(input) → WhatsNewDecision`（見上）；localStorage 鍵 `speclink.lastSeenVersion`。
- `AppSettingsUpdaterProps.onShowReleaseNotes: () => void`；i18n 新鍵：`releaseNotes.whatsNewTitle`（含 {version}）、`releaseNotes.browseTitle`、`releaseNotes.gotIt`、`releaseNotes.close`、`releaseNotes.open`（按鈕「更新日誌」）、`releaseNotes.empty`（瀏覽模式無條目時的提示），zh-TW 與 en 皆補。

**Failure modes**

- 技能：守門失敗與寫檔步驟失敗都是明示停止，不靜默；push 失敗（如 origin 拒絕）時 commit 與 tag 留在本機，技能印出手動 `git push --atomic origin main vX.Y.Z` 指令。
- CI：版號不齊或 CHANGELOG.md 漂移＝紅燈（fail-closed），訊息點名哪一處。
- 桌面：JSON 匯入失敗不可能（build 時打包）；lastSeen 壞值視為無記錄→走 record 路徑。

**Acceptance criteria**

- `node --test scripts/release-notes-render.test.mjs`：renderEntry／renderChangelog 的固定輸出、`--check` 一致與不一致的 exit code、`--version` 找不到版本的 exit code、CRLF 正規化。
- `node --test scripts/release-notes.test.mjs`：輸出含指南且其後接該版片段；JSON 缺該版時非零。
- `npm test -w apps/desktop`：whatsNew.test.ts 涵蓋 dev、頂端不符、首次安裝、已看過、跳版、lastSeen 不在清單六種；releaseNotesDialog.test.tsx 兩模式標題與主鈕；appSettingsView.test.tsx 更新卡「更新日誌」按鈕呼叫 onShowReleaseNotes。
- `node scripts/release-notes-render.mjs --check` 於工作樹 exit 0（回填後 CHANGELOG.md 與 JSON 一致）。
- release.yml 的斷言步驟以本機 shell 對三種不齊情境各驗一次（tag≠conf、tag≠JSON 頂端、CHANGELOG 漂移）——手動任務。
- 技能：在乾淨 main 上以 dry-run 方式走到「停一次」（不點頭）確認守門與草稿——手動任務。

**Scope boundaries**

- In：技能檔、JSON 與型別、渲染腳本與測試、release-notes.mjs 接片段、release.yml 斷言與標頭註解、桌面判定／對話框／設定頁按鈕／i18n／測試、0.2.0 回填、LANGUAGE.md 詞條。
- Out：Non-Goals 全部；updater 狀態機與 UpdateBanner 不動；任何 Rust crate 不動；`.agents/` 不動。

## Risks / Trade-offs

- [CHANGELOG.md 被手改而與 JSON 漂移] → `--check` 在 desktop job 建置前紅燈，訊息點名；技能寫檔步驟也跑 `--check`。
- [dev 與安裝版共用 localStorage] → dev 永不自動彈、也不寫 lastSeen（record 路徑同樣被 dev 短路），安裝版的記錄不受 dev 干擾。
- [Windows checkout 的 autocrlf 改寫 CHANGELOG.md] → `--check` 讀入時 CRLF 正規化為 LF；寫出永遠 LF。
- [JSON 頂端與 app 版號不齊（本機建置忘了 bump）] → 桌面判定以「頂端等於 appVersion」為前提，不齊即不彈；CI 端由斷言擋。
- [回歸對照] → 不動任何 CLI 輸出與 `--json`，golden 不受影響；下載指南既有測試維持，只新增「其後接片段」斷言。
- [跨平台] → 渲染腳本純字串、路徑用 node:path 由 repo 根解析；技能的 shell 指令（git、gh、cargo、node）三平台皆有。
- [技能誤觸即發版] → 守門五項＋「停一次」是唯一防線；push 用 `--atomic` 避免 main 推了 tag 沒推的半套。

## Migration Plan

1. 本變更合併到 main 後，下一次發版即第一次執行 `/release`（預期 0.3.0）。
2. 不需資料遷移：既有使用者更新到第一個帶 JSON 的版本時，lastSeen 無記錄→走 record 路徑不彈（等同首次安裝）；再下一版才開始彈。這是接受的行為：第一個帶此功能的版本本身的日誌，使用者可從設定頁瀏覽。
3. 回滾：移除技能檔與 release.yml 斷言即可，JSON 與 CHANGELOG.md 留著無害。

## Open Questions

無。
