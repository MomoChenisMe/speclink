---
name: release
description: "執行即發版——在乾淨的 main 上守門、判定版號、把 commit 策展成白話更新日誌、寫檔、commit、附註 tag 並 push，push 前停一次讓使用者點頭。使用者說「發版」「發布新版本」「release」「壓 tag」「出 0.x.y」時使用。這是本 repo 自用的技能，不是 speclink 產品技能。"
---

# /release — 執行即發版

這個技能把發版一次做完：守門 → 版號 → 更新日誌 → 停一次 → 寫檔 → commit／tag／push。
唯一的不可逆點是 push；push 之前一定會停下來讓使用者點頭。

**這是 repo 本地技能**：不進 ASSET_VERSION／golden／assets.lock，不隨 `speclink init`／`speclink update`
裝進使用者專案；目錄名不以 `speclink-` 開頭，`speclink update` 的孤兒清理不會動它。

**輸入**：可選的目標版號（例：`/release 0.3.0`）。沒給就由 commit 判定並建議。

**前提**：需要 `git`、`gh`（已登入）、`cargo`、`node`。任一缺少即報錯停止。

---

## 1. 守門（任一不成立即停，不改任何檔）

依序檢查，印出不成立的那一項就停止：

1. **在 main**：`git rev-parse --abbrev-ref HEAD` 必須是 `main`。
2. **工作樹乾淨**：`git status --porcelain` 必須為空。非空時印出「工作樹不乾淨」與未提交檔案清單，停止。
3. **與 origin/main 同步**：先 `git fetch origin`，`git rev-parse HEAD` 必須等於 `git rev-parse origin/main`。
   不同時說明是本機領先（先 push）還是落後（先 pull），停止。
4. **HEAD 的 CI 全綠**：
   ```bash
   gh run list --workflow CI --commit "$(git rev-parse HEAD)" --json conclusion,status,url
   ```
   必須有一筆且 `conclusion` 為 `success`。沒有紀錄、`status` 仍是 in_progress／queued、或 conclusion 不是
   success，都印出「CI 尚未通過」（附網址）並停止，請使用者等 CI 綠燈後重跑。
5. **進行中變更**（不擋）：`speclink list --json` 的 `changes` 中 `status` 為 `in-progress` 者，列出名稱，
   用 AskUserQuestion 問「仍要發版嗎？」。使用者確認才繼續；否則停止。

守門階段不建立任何檔、不改任何檔、不建 tag。

## 2. 範圍

- 上次 tag：`git describe --tags --abbrev=0`（形如 `vX.Y.Z`）。
- 唯一輸入：`git log <上次 tag>..HEAD --pretty=format:'%H%n%s%n%b%n---END---'`。
  版號判定與日誌策展都只看這批 commit 的主旨與內文，不看其他來源。
- 範圍為空（HEAD 就是上次 tag）：印出「上次 tag 之後沒有新 commit」並停止。

## 3. 版號判定

逐筆解析 conventional commit 的 type，取最高影響：

| 條件 | 影響 |
| --- | --- |
| 主旨 type 後帶 `!`，或內文有 `BREAKING CHANGE:` footer | major |
| `feat` | minor |
| `fix`、`perf` | patch |
| `chore`、`docs`、`refactor`、`test`、`ci`、`build`、`style` | 不提高版號 |
| 無法解析 | 當 patch，並列出請使用者確認 |

規則：

- **0.x 封頂**：上次 tag 低於 1.0.0 時，major 封頂為 minor（`v0.2.0` 有 breaking → 建議 0.3.0，不是 1.0.0）。
  升到 1.0.0 必須由使用者明示指定版號。
- **只有雜訊**：範圍內全是不提高版號的 type 時，警告「這批沒有面向使用者的變更」，用 AskUserQuestion 問是否仍發
  （仍發時當 patch）。使用者選不發即停止，不改任何檔。
- **建議而非決定**：判定結果連同理由（哪幾筆 commit 決定了這個等級）一起呈現；使用者可覆寫為任一合法的 `X.Y.Z`。
  使用者在輸入就給了版號時，仍印出判定結果供對照，但以使用者的為準。

## 4. 更新日誌策展

把範圍內的 commit 改寫成**使用者語氣的繁體中文條目**，分三組：

- **新功能**：feat
- **修正**：fix、perf
- **改善**：其餘有保留價值者（例：面向使用者的改名、流程調整、相容層移除）

策展規則：

1. **封存變更優先、一變更一條**：主旨形如 `speclink(<變更名>): …` 的 commit 是封存變更。讀
   `openspec/changes/archive/*-<變更名>/proposal.md` 的 `## Why`，連同該變更開發過程的 feat／fix commit，
   合寫成**一條**。開發過程的 commit 不另列。
2. **不屬於任何變更的 feat／fix／perf 逐筆改寫**成一條。
3. **雜訊不進**：`chore`、`test`、`refactor`、`docs`、`ci`、`build`、`style` 一律不出現在條目中
   （純內部重構的封存變更也不進）。
4. **使用者語氣、不照抄主旨**：寫「使用者會看到什麼／能做什麼」，不寫「修了哪個函式」。
   每條一句話，句尾句號。
5. **技術名詞保留原文**：CLI、worktree、Tauri、slug、capability、server、npm 等直接寫，不硬翻。
6. **詞彙**：使用者可見文案遵守 `openspec/LANGUAGE.md`（例：「變更」不寫 change、「產出流程」不寫 schema、
   「更新日誌」不寫 changelog）。
7. 沒有條目的分組不寫。

## 5. 停一次

寫任何檔之前，呈現：

- 建議版號與理由（決定等級的 commit）
- 三組日誌草稿（markdown 條目）
- 將被改動的檔案清單（見第 6 節）

用 AskUserQuestion 等使用者點頭。使用者要改條目文字或版號時，改完**重新呈現**再問一次；
點頭之前工作樹零改動。

## 6. 寫檔（順序固定，任一步失敗即停在該步）

設 `VERSION` 為點頭的版號、`DATE` 為當天（`YYYY-MM-DD`）：

1. **更新日誌 JSON**：`apps/desktop/src/release-notes/release-notes.json` 頂端 prepend
   `{ "version": VERSION, "date": DATE, "sections": [...] }`（title 只用「新功能」「修正」「改善」；
   空分組不寫；維持兩空格縮排與檔尾換行）。
2. **workspace Cargo.toml**：`[workspace.package]` 的 `version = "VERSION"`。
3. **tauri.conf.json**：`apps/desktop/src-tauri/tauri.conf.json` 的 `version`。
4. **刷 Cargo.lock**：`cargo metadata --format-version 1 > /dev/null`。
5. **產 CHANGELOG.md**：`node scripts/release/release-notes-render.mjs --write`。
6. **測試與比對**：
   ```bash
   node --test scripts/release/release-notes-render.test.mjs scripts/release/release-notes.test.mjs
   node scripts/release/release-notes-render.mjs --check
   ```

任一步失敗：印出失敗的步驟與原因，**不 commit**；已改的檔留在工作樹供使用者檢視，並提示
`git checkout -- . && git clean -f CHANGELOG.md` 可整批還原（CHANGELOG.md 已存在時只需前半句）。

## 7. 提交、附註 tag、原子 push

全部成功後：

```bash
git add apps/desktop/src/release-notes/release-notes.json Cargo.toml Cargo.lock \
  apps/desktop/src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "chore(release): 版號升至 VERSION"
git tag -a "vVERSION" -m "vVERSION"
git push --atomic origin main "vVERSION"
```

- commit 訊息依 conventional-commit 技能，固定為 `chore(release): 版號升至 X.Y.Z`；不帶其他檔。
- `--atomic`：main 與 tag 同一次推，全成或全不成，不會出現 main 推了、tag 沒推的半套。
- **push 失敗**（origin 拒絕、網路）：commit 與 tag 留在本機，印出原因與可直接重試的同一句：
  ```
  git push --atomic origin main vVERSION
  ```
  不要自行 reset 或刪 tag。

## 8. 收尾

成功後印出：

1. Release 工作流網址：`https://github.com/MomoChenisMe/speclink/actions/workflows/release.yml`
   （或 `gh run list --workflow Release --limit 1 --json url` 取到的那筆）。
2. 三項提醒（來自 release-pipeline-npm-pitfalls 備忘）：
   - job 綠不等於套件已在 registry——到 npm／ghcr／tap 各看一眼。
   - 新套件的 packument 可能延遲可見，先等、別重發。
   - 發完重裝本機 app：`node scripts/desktop/desktop-install.mjs --install`（更新日誌彈窗會在下一版才對這台機器彈出，
     本版內容可從設定頁「更新日誌」看）。

發版失敗後的重跑（移 tag 重推）不在本技能內，維持 release-pipeline-npm-pitfalls 備忘的手動程序。
