## 1. 渲染腳本（D1、D2）

- [x] 1.1 Red：依「D7 先寫測試」改寫 `scripts/release/release-notes-render.test.mjs` 與夾具 `scripts/release/release-notes-scratch.mjs`：夾具改用 D1 的雙語形狀並建立 `CHANGELOG.zh-TW.md`；測試涵蓋 `--write` 產出兩份檔（首行與語言切換列）、`--version` 預設英文與 `--lang zh-TW`、`--lang` 非法值非零結束、`--check` 點名漂移的檔名、缺 `en` 或 `kind` 不在三值時非零結束並點名版號、CRLF 正規化（「更新日誌 JSON 為唯一真相」「渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段」）。驗證：`node --test scripts/release/release-notes-render.test.mjs` 對新斷言失敗。 <!-- speclink-task:tsk_01M4Z3SWKQ3JFXNA7R36MJM790 -->
- [x] 1.2 Green：依「D1 JSON 形狀：`kind` 與雙語條目」與「D2 渲染腳本產兩份檔」修改 `scripts/release/release-notes-render.mjs`：形狀驗證、依語言渲染、`--write` 寫兩份、`--check` 比對兩份、`--version` 加 `--lang`。驗證：`node --test scripts/release/release-notes-render.test.mjs` 全綠。 <!-- speclink-task:tsk_01M4Z3SWKQ8FAZCCCAP7Q4JJ65 -->

## 2. Release 說明（D3）

- [x] 2.1 Red：改寫 `scripts/release/release-notes.test.mjs`：輸出以英文指南與英文片段開頭、接 `<details>` 且 `<summary>繁體中文</summary>` 內含繁中指南與繁中片段、`<details>` 內 Markdown 前後各有空行、兩段檔名逐字相同、JSON 缺該版時非零結束且 stdout 空白（「Release 說明含下載指南」）。驗證：`node --test scripts/release/release-notes.test.mjs` 對新斷言失敗。 <!-- speclink-task:tsk_01M4Z3SWKQH5DXGXZFB4AWF18V -->
- [x] 2.2 Green：依「D3 Release 說明：英文在前、繁中收合」修改 `scripts/release/release-notes.mjs`，兩種語言的指南共用同一個資產檔名產生邏輯。驗證：`node --test scripts/release/release-notes.test.mjs` 全綠；`node scripts/release/release-notes.mjs --tag v0.8.0` 的輸出依序為英文指南、英文 0.8.0 片段、收合的繁中區塊。 <!-- speclink-task:tsk_01M4Z3SWKQXAW6YE774CJEDT7K -->
- [x] 2.3 修改 `.github/workflows/release.yml` 的版本閘門錯誤訊息，說明 `--check` 同時比對 `CHANGELOG.md` 與 `CHANGELOG.zh-TW.md`，並在失敗時提示執行 `--write` 後提交兩份檔（「release 管線守門版號四處同版與渲染一致」）。驗證：`node --test scripts/release/delivery-gate.test.mjs` 通過，workflow 中 `release-notes-render.mjs --check` 的呼叫仍存在。 <!-- speclink-task:tsk_01M4Z3SWKQHBFZV6M63Y7J4RFJ -->

## 3. 資料遷移與補英文（D6）

- [x] 3.1 把 `apps/desktop/src/release-notes/release-notes.json` 的 7 個版本（0.2.0–0.8.0）轉成 D1 形狀：繁中 `title` 對應為 `kind`、每條繁中字串放進 `zh-TW`，並依「D6 補英文的方式」寫出一對一的英文 `en`（簡化技術英文、技術名詞保留原文）。驗證：`node scripts/release/release-notes-render.mjs --write` 成功，產出的 `CHANGELOG.zh-TW.md` 與遷移前的 `CHANGELOG.md` 內容（除首兩行外）逐字相同；`--check` exit 0。 <!-- speclink-task:tsk_01M4Z3SWKQ8FXN98FMDMX52NMX -->

## 4. 桌面 app（D4）

- [x] 4.1 Red：依 D7 改寫 `apps/desktop/src/__tests__/releaseNotesDialog.test.tsx`、`apps/desktop/src/__tests__/updatesPage.test.tsx` 與 `apps/desktop/src/__tests__/whatsNew.test.ts` 的夾具為 D1 形狀，新增英文介面顯示英文條目、New features／Fixes／Improvements 與「0.3.1 (2026-09-20)」版號小標、繁中介面維持繁中、切換語言後已開啟的對話框改用新語言的測試（「更新日誌依介面語言呈現」）。驗證：`npm test -w apps/desktop` 對新測試失敗。 <!-- speclink-task:tsk_01M4Z3SWKQCD41DW6V8KD5K4WM -->
- [x] 4.2 Green：依「D4 桌面 app 依介面語言呈現」修改 `apps/desktop/src/release-notes/release-notes.ts`（新型別與 `entryText`）、`apps/desktop/src/i18n/messages.ts`（三個 `releaseNotes.kind.*` 鍵與版號小標的 `releaseNotes.versionHeading` 鍵）、`apps/desktop/src/components/ReleaseNotesDialog.tsx`、`apps/desktop/src/views/settings/UpdatesPage.tsx` 與 `apps/desktop/src/core/whatsNew.ts`。驗證：`npm test -w apps/desktop` 全綠，含既有的彈出判定測試。 <!-- speclink-task:tsk_01M4Z3SWKQG80K32S4DJR27SP3 -->

## 5. release 技能與文件（D5）

- [x] 5.1 依「D5 release 技能雙語策展」修改 `.claude/skills/release/SKILL.md`：策展步驟改為中英雙語、草稿每條列 en／zh-TW 兩行、寫檔順序產出兩份更新日誌、還原指令與提交清單加入 `CHANGELOG.zh-TW.md`（「更新日誌策展為使用者語氣的中英雙語條目」取代已移除的「更新日誌策展為使用者語氣的繁中條目」，並符合「呈現草稿並在使用者點頭後才寫檔」）。驗證：通讀技能檔，每個提到 `CHANGELOG.md` 的步驟都同時處理 `CHANGELOG.zh-TW.md`；技能檔中不再要求「只寫繁中條目」。 <!-- speclink-task:tsk_01M4Z3SWKQN6X29EVY8BQ4Z5C4 -->
- [x] 5.2 更新兩版 README 的文件地圖：`README.md` 的 Changelog 列連到 `CHANGELOG.md` 並移除「(Traditional Chinese)」註記，`README.zh-TW.md` 的「更新日誌」列連到 `CHANGELOG.zh-TW.md`。驗證：`node --test scripts/docs/docs-links.test.mjs scripts/docs/docs-parity.test.mjs` 通過。 <!-- speclink-task:tsk_01M4Z3SWKQJ0H6EEX1RWDWHVT4 -->
- [x] [M] 5.3 使用者檢視 3.1 補好的英文 `CHANGELOG.md`，確認七個版本的英文條目與繁中意思一致、用語可接受；有修改意見時由 agent 改 JSON 的 `en` 並重跑 `--write` 與 `--check`，使用者再看一次。 <!-- speclink-task:tsk_01M4Z3SWKQHXHXPNDZR8J8PA2J -->

## 6. 收尾

- [x] 6.1 跑全部受影響的測試：`npm test -w apps/desktop` 全綠，且 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束。 <!-- speclink-task:tsk_01M4Z3SWKQ2K24SJ3XQWK6XX2M -->
