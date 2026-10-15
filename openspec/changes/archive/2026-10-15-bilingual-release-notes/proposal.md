## Summary

把更新日誌改成中英雙語：一份 JSON 同時存兩種語言，由它畫出英文 `CHANGELOG.md`、繁中 `CHANGELOG.zh-TW.md`、英文在前且繁中收合的 GitHub Release 說明，桌面 app 依介面語言顯示；release 技能改為兩種語言一起策展，0.2.0–0.8.0 全部補上英文。

## Motivation

目標讀者有三種：在 GitHub Release 頁下載安裝檔的人、讀 `CHANGELOG.md` 的開源讀者、以及更新桌面 app 後看到「更新內容」對話框的使用者。這是 release 技能（發版）與桌面 app 更新流程的一部分。

現況只有繁中：

- `apps/desktop/src/release-notes/release-notes.json` 的條目只有繁中字串，分組標題寫死「新功能」「修正」「改善」。
- `CHANGELOG.md` 是繁中。readme-open-source-refresh 已把 `README.md` 改為英文主版，但文件地圖仍連到一份註明「Traditional Chinese」的更新日誌。
- GitHub Release 說明的下載指南與更新內容都是繁中，國際讀者看不懂該下載哪個檔案。
- 桌面 app 切到英文介面時，「更新內容」對話框與更新頁仍顯示繁中條目。

規格掃描：`release-notes`（JSON 契約與渲染）、`release-skill`（策展與寫檔順序）、`desktop-release`（Release 說明的下載指南）與 `desktop-app`（更新日誌彈窗）都綁了單一語言，需要一起改。`delivery-baseline` 的路徑守門排除 `CHANGELOG.md`，但兩份更新日誌都沒有舊形路徑，不需要改。

## Proposed Solution

- **JSON 改成雙語**：分組改用 `kind`（`features`、`fixes`、`improvements`）取代繁中 `title`；每個條目改成 `{"en": "…", "zh-TW": "…"}`，兩種語言都必填、非空。分組標題由渲染端依語言決定。
- **兩份更新日誌**：
  - 渲染腳本 `--write` 同時產出英文 `CHANGELOG.md`（`# Changelog`、`## X.Y.Z (YYYY-MM-DD)`）與繁中 `CHANGELOG.zh-TW.md`（`# 更新日誌`、`## X.Y.Z（YYYY-MM-DD）`），兩份頂端有語言切換。
  - `--check` 兩份都比對。
  - `--version` 新增 `--lang en|zh-TW`（預設 `en`）。
- **Release 說明**：英文下載指南與英文更新內容在前；其後是一個預設收合的 `<details>` 區塊，標題為「繁體中文」，裡面是繁中下載指南與繁中更新內容；GitHub 自動產生的比較連結在最後。
- **桌面 app**：「更新內容」對話框與更新頁「更新日誌」卡依介面語言顯示條目與分組標題；彈出與已看過的判定不變。
- **release 技能**：草稿同時呈現兩種語言的條目，使用者確認後寫入雙語 JSON，再產出兩份更新日誌。繁中照舊用使用者語氣與正典詞彙，英文用簡化技術英文（短句、主動語態、不用完成式）。
- **補英文**：0.2.0–0.8.0 七個版本的條目全部翻成英文，內容與繁中一一對應。
- **文件地圖**：兩版 README 的「更新日誌」列改連到對應語言的更新日誌。

## Non-Goals

（design.md 會建立，範圍排除寫在 design 的 Goals / Non-Goals。）

## Alternatives Considered

- **兩份 JSON（每種語言一份）**：條目要靠位置對齊，漏翻或多一條時很難察覺；改成同一條目內並存兩種語言，缺一種就驗不過。
- **Release 頁只放英文**：繁中讀者要多點一次連結；使用者選了英文在前、繁中收合。
- **舊版本只保留繁中**：英文 CHANGELOG 與英文桌面介面會出現一半繁中；使用者選擇全部補英文。

## Impact

- Affected specs: `release-notes`（JSON 雙語契約、兩份更新日誌與 `--lang`、管線比對兩份檔）、`release-skill`（策展改為中英雙語、寫檔順序產出兩份更新日誌）、`desktop-release`（Release 說明英文在前、繁中收合）、`desktop-app`（更新日誌依介面語言呈現）
- Affected apps／scripts：`apps/desktop`（更新日誌型別、對話框、更新頁、選單）、`scripts/release/`（渲染與下載指南腳本）、`.github/workflows/release.yml`、repo 本地 release 技能。不動任何 crate、CLI 指令、`--json` 輸出與設定欄位；生成的 speclink 產品技能不受影響。
- 相容性：`release-notes.json` 的形狀改變，舊形狀（繁中 `title` 與字串條目）不再被讀取。所有讀取端在同一個變更內一起改，沒有外部讀者。
- Affected code:
  - New:
    - `CHANGELOG.zh-TW.md`
  - Modified:
    - `apps/desktop/src/release-notes/release-notes.json`
    - `apps/desktop/src/release-notes/release-notes.ts`
    - `apps/desktop/src/components/ReleaseNotesDialog.tsx`
    - `apps/desktop/src/views/settings/UpdatesPage.tsx`
    - `apps/desktop/src/core/whatsNew.ts`
    - `apps/desktop/src/i18n/messages.ts`
    - `apps/desktop/src/__tests__/releaseNotesDialog.test.tsx`
    - `apps/desktop/src/__tests__/updatesPage.test.tsx`
    - `apps/desktop/src/__tests__/whatsNew.test.ts`
    - `scripts/release/release-notes-render.mjs`
    - `scripts/release/release-notes-render.test.mjs`
    - `scripts/release/release-notes.mjs`
    - `scripts/release/release-notes.test.mjs`
    - `scripts/release/release-notes-scratch.mjs`
    - `.github/workflows/release.yml`
    - `.claude/skills/release/SKILL.md`
    - `CHANGELOG.md`
    - `README.md`
    - `README.zh-TW.md`
