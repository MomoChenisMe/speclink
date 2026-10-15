## Context

- 更新日誌的唯一來源是 `apps/desktop/src/release-notes/release-notes.json`：陣列、最新在前，每筆 `{version, date, sections:[{title, items}]}`；`title` 是繁中三值，`items` 是繁中字串。
- 衍生物：
  - `scripts/release/release-notes-render.mjs` 畫出 `CHANGELOG.md`，並提供單版片段與 `--check`。
  - `scripts/release/release-notes.mjs --tag` 印出 Release 說明：繁中下載指南，加上該版片段。
  - `.github/workflows/release.yml` 在建置前斷言版號，並跑 `--check`。
  - 桌面 app 的 `ReleaseNotesDialog`、`UpdatesPage`、`whatsNew` 與 macOS 選單讀這份 JSON。
- 兩支腳本的測試用 `release-notes-scratch.mjs` 夾具，在暫存目錄重建目錄形狀。
- release 技能是 repo 本地技能 `.claude/skills/release/SKILL.md`，不是 speclink 產品技能，也不經 asset 版號。
- 桌面 app 的介面語言由 `packages/ui` 的 `I18nProvider` 提供（`zh-TW` 或 `en`）。
- 本變更不動任何 crate 與 CLI。

## Goals / Non-Goals

**Goals:**

- 一份 JSON 存兩種語言，缺一種就驗不過。
- 英文 `CHANGELOG.md`＋繁中 `CHANGELOG.zh-TW.md`；Release 說明英文在前、繁中收合；桌面依介面語言顯示。
- release 技能一次策展兩種語言。
- 0.2.0–0.8.0 全部補英文。

**Non-Goals:**

- 不支援第三種語言，也不做「缺翻譯時退回另一種語言」：兩種語言都是必填。
- 不改彈出判定、已看過記錄與版號斷言的邏輯。
- 不改 Release 資產與下載指南的檔名契約（只改語言與排版）。
- 不改 speclink 產品技能與 CLI。
- 不翻譯 git 歷史中的 commit 訊息。

## Decisions

### D1 JSON 形狀：`kind` 與雙語條目

```json
{
  "version": "0.8.0",
  "date": "2026-09-25",
  "sections": [
    { "kind": "features", "items": [ { "en": "…", "zh-TW": "…" } ] }
  ]
}
```

- `kind` 只取 `features`、`fixes`、`improvements` 三值，順序固定為這三值的順序。
- 沒有條目的分組不寫入。
- 每個條目的 `en` 與 `zh-TW` 都是非空字串。
- 分組標題由語言決定：
  - en：New features、Fixes、Improvements
  - zh-TW：新功能、修正、改善

替代方案：保留繁中 `title` 再加 `title_en`。被否決，因為標題是固定詞，存在資料裡只會重複。

### D2 渲染腳本產兩份檔

`release-notes-render.mjs` 的渲染函式多一個 `lang` 參數：

| 用法 | 行為 |
| --- | --- |
| `--write` | 寫出 `CHANGELOG.md`（en）與 `CHANGELOG.zh-TW.md`（zh-TW） |
| `--check` | 兩份都重算比對；任一份不一致就以非零結束，並點名檔名與差異摘要 |
| `--version X.Y.Z [--lang en\|zh-TW]` | 印出該版該語言的片段，預設 `en` |

格式：

- **en**：`# Changelog`、`## X.Y.Z (YYYY-MM-DD)`、`### New features`。
- **zh-TW**：`# 更新日誌`、`## X.Y.Z（YYYY-MM-DD）`、`### 新功能`（與現在相同）。
- 兩份檔在標題下一行有語言切換：en 為「**English** · [繁體中文](CHANGELOG.zh-TW.md)」，zh-TW 為「[English](CHANGELOG.md) · **繁體中文**」。
- 換行一律 LF。比對前把 CRLF 正規化為 LF。

### D3 Release 說明：英文在前、繁中收合

`release-notes.mjs --tag vX.Y.Z` 的輸出依序為：

1. 英文下載指南：對照表、CLI 三條指令、server 一行啟動、自動更新檔說明。
2. 英文更新內容片段。
3. `<details>` 區塊，`<summary>` 為「繁體中文」，內含繁中下載指南與繁中更新內容片段。

`<details>` 內的 Markdown 前後要空一行，GitHub 才會渲染。檔名清單在兩種語言都用同一個資產命名函式產生，所以兩段的檔名逐字相同。tag 格式不符或 JSON 缺該版時，照舊以非零結束，且 stdout 保持空白。

### D4 桌面 app 依介面語言呈現

- `release-notes.ts` 匯出新的型別（`kind` 與雙語條目），並匯出依語言取文字的函式 `entryText(item, locale)`。
- 分組標題改用 i18n 鍵：`releaseNotes.kind.features`、`releaseNotes.kind.fixes`、`releaseNotes.kind.improvements`，寫在 `apps/desktop/src/i18n/messages.ts`，中英各一份。
- `ReleaseNotesDialog` 與 `UpdatesPage`（含首條摘要）從 `useI18n()` 取得 `locale`，呈現對應語言。
- `whatsNew` 的判定只看版號，不受影響；它的測試夾具改成新形狀。

### D5 release 技能雙語策展

`.claude/skills/release/SKILL.md` 的策展步驟改為每條同時寫：

- 繁中：使用者語氣、正典詞彙。
- 英文：簡化技術英文（一句一事、主動語態、不用完成式；技術名詞保留原文）。

草稿以「分組 → 每條 en／zh-TW 兩行」呈現，使用者確認後寫入 JSON。寫檔順序中的「產出 CHANGELOG.md」改為「產出 CHANGELOG.md 與 CHANGELOG.zh-TW.md」。還原指令與提交清單把 `CHANGELOG.zh-TW.md` 一起列入。

### D6 補英文的方式

0.2.0–0.8.0 的每條繁中條目，翻成意思對應的英文：

- 一條對一條，不合併也不刪減。
- 技術名詞與指令保留原文。
- 遵守 D5 的英文寫法。

翻完後由渲染腳本產出兩份更新日誌，並把英文版呈給使用者檢視。

### D7 先寫測試

腳本測試與桌面測試先改成新形狀與新行為，看它們失敗，再改實作與資料：

- `release-notes-render.test.mjs`：兩份檔、`--lang`、語言切換列、`--check` 點名檔名。
- `release-notes.test.mjs`：英文在前、`<details>` 收合、兩段檔名一致。
- `releaseNotesDialog.test.tsx`、`updatesPage.test.tsx`：英文介面顯示英文條目與英文分組標題。
- JSON 形狀測試：每筆的 `kind` 是三值之一，`en` 與 `zh-TW` 都非空。

## Implementation Contract

**行為：**

- `node scripts/release/release-notes-render.mjs --write` 產出英文 `CHANGELOG.md` 與繁中 `CHANGELOG.zh-TW.md`；`--check` 兩份一致時 exit 0。
- `--version 0.8.0 --lang zh-TW` 印出繁中片段；不加 `--lang` 印出英文片段。
- `node scripts/release/release-notes.mjs --tag v0.8.0` 的輸出：英文指南與英文 0.8.0 片段在前，接著是 `<details><summary>繁體中文</summary>`，內含繁中指南與繁中片段。
- 桌面 app 英文介面的「更新內容」對話框與更新頁顯示英文條目與「New features／Fixes／Improvements」；繁中介面與現在相同。

**資料形狀：** 見 D1。舊形狀不再被讀取。

**失敗模式：**

- JSON 條目缺 `en` 或 `zh-TW`、或 `kind` 不在三值內時，渲染腳本以非零結束並點名版號與位置；桌面的形狀測試失敗。
- `--check` 任一份漂移時以非零結束，並點名檔名。
- `--lang` 不是 `en` 或 `zh-TW` 時以非零結束。

**驗收：**

- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠。
- `npm test -w apps/desktop` 全綠。
- `node scripts/release/release-notes-render.mjs --check` exit 0。

**範圍：**

- 範圍內：上述 JSON、兩支腳本與測試、夾具、release workflow 的 `--check` 步驟、桌面四個讀取端與其測試、release 技能、兩份更新日誌、README 兩版的更新日誌連結。
- 範圍外：crate、CLI、speclink 產品技能、彈出判定邏輯、Release 資產。

## Risks / Trade-offs

- [舊形狀的 JSON 被舊版 app 讀到] → JSON 是打包進 app 的，舊版 app 讀的是自己打包的舊 JSON，不受影響。
- [翻譯品質] → 補英文後把英文 CHANGELOG 呈給使用者檢視；之後每次發版由 release 技能在草稿階段一起確認。
- [Release workflow 改了要等下一個 tag 才實測] → 下載指南腳本與渲染腳本都有本機測試，workflow 只改 `--check` 呼叫面；下一次發版時檢查 Release 頁。
- [跨平台] → 兩份檔的換行一律 LF，比對前正規化 CRLF；腳本測試在 Windows CI 照常跑。
- [回歸對照] → 不動 CLI 與 golden；回歸面是兩支腳本的固定輸出測試與桌面測試。
