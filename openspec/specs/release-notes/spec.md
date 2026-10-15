# release-notes Specification

## Purpose

更新日誌的資料契約與衍生物：JSON 檔的位置與欄位、由它畫出 CHANGELOG.md 與 Release 說明片段的渲染規則、release 管線對版號一致與渲染一致的守門。邊界：日誌內容怎麼產生屬 release-skill；桌面 app 怎麼呈現屬 desktop-app；Release 說明的其餘內容屬 desktop-release。

## Requirements

### Requirement: 更新日誌 JSON 為唯一真相

repo SHALL 在 `apps/desktop/src/release-notes/release-notes.json` 保存更新日誌，為一個陣列、最新版本在前。每筆 SHALL 含 `version`（`X.Y.Z`，不帶 v 前綴，與 tauri.conf.json 的 version 同字面）、`date`（`YYYY-MM-DD`）、`sections`（陣列，每個元素含 `kind` 與 `items`）。`kind` SHALL 只取 `features`、`fixes`、`improvements` 三值之一，同一筆內的分組 SHALL 依此順序排列；`items` SHALL 為非空陣列，每個條目 SHALL 為物件 `{"en": <英文字串>, "zh-TW": <繁體中文字串>}`，兩個欄位 SHALL 都是非空字串且描述同一件事；沒有條目的分組 SHALL NOT 寫入。分組標題 SHALL NOT 存於 JSON，而由呈現端依語言決定（英文 New features／Fixes／Improvements，繁中 新功能／修正／改善）。`CHANGELOG.md`、`CHANGELOG.zh-TW.md` 與 Release 說明的更新內容 SHALL 只由此檔衍生，SHALL NOT 另行手寫。

#### Scenario: 條目形狀

- **WHEN** 讀取 release-notes.json 頂端條目
- **THEN** 其 version 為 `X.Y.Z` 格式、date 為 `YYYY-MM-DD`、每個 section 的 kind 為三值之一且 items 非空、每個條目的 `en` 與 `zh-TW` 都是非空字串

#### Scenario: 缺一種語言即拒絕

- **WHEN** 某筆條目只有 `zh-TW` 沒有 `en`，執行渲染腳本任一用法
- **THEN** 腳本以非零結束，stderr 點名該版號與條目位置，不寫出任何檔案

##### Example: 一筆條目

- **GIVEN** 0.3.0 於 2026-09-10 發版，含兩個新功能與一個修正
- **WHEN** 檢視頂端條目
- **THEN** `{"version":"0.3.0","date":"2026-09-10","sections":[{"kind":"features","items":[{"en":"…","zh-TW":"…"},{"en":"…","zh-TW":"…"}]},{"kind":"fixes","items":[{"en":"…","zh-TW":"…"}]}]}`，沒有 improvements 分組


<!-- @trace
source: bilingual-release-notes
updated: 2026-10-15T17:28:24+08:00
-->

---
### Requirement: 渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段

repo SHALL 提供渲染腳本 `scripts/release/release-notes-render.mjs`，同一份渲染邏輯依語言支援三種用法：`--write` 把全部條目寫成 repo 根的英文 `CHANGELOG.md` 與繁中 `CHANGELOG.zh-TW.md`；`--version X.Y.Z [--lang en|zh-TW]` 把該版該語言的片段印到 stdout，`--lang` 預設為 `en`；`--check` 重算兩份檔並分別與現有檔比對。英文片段格式 SHALL 為 `## X.Y.Z (YYYY-MM-DD)` 標題，其下每個分組為 `### <英文分組標題>` 接 `- ` 條目；繁中片段格式 SHALL 為 `## X.Y.Z（YYYY-MM-DD）` 標題，其下每個分組為 `### <繁中分組標題>` 接 `- ` 條目。`CHANGELOG.md` SHALL 以 `# Changelog` 開頭、次行為語言切換「**English** · [繁體中文](CHANGELOG.zh-TW.md)」；`CHANGELOG.zh-TW.md` SHALL 以 `# 更新日誌` 開頭、次行為「[English](CHANGELOG.md) · **繁體中文**」；其後依 JSON 順序接每版片段，換行一律 LF。`--version` 找不到該版 SHALL 以非零結束並於 stderr 印出 JSON 頂端版號；`--lang` 不是 `en` 或 `zh-TW` SHALL 以非零結束；`--check` 任一份不一致 SHALL 以非零結束並於 stderr 點名該檔名與差異摘要，比對前 SHALL 把讀入檔的 CRLF 正規化為 LF。腳本 SHALL 有自動化測試涵蓋兩種語言的固定輸出、各用法的 exit code 與 CRLF 正規化。

#### Scenario: 產出 CHANGELOG.md

- **WHEN** JSON 有 0.3.0 與 0.2.0 兩筆並執行 `--write`
- **THEN** `CHANGELOG.md` 首行為 `# Changelog`，其後先 `## 0.3.0 (…)` 再 `## 0.2.0 (…)`；`CHANGELOG.zh-TW.md` 首行為 `# 更新日誌`，其後先 `## 0.3.0（…）` 再 `## 0.2.0（…）`；exit 0

#### Scenario: 比對漂移

- **WHEN** 有人手改 `CHANGELOG.zh-TW.md` 一個字後執行 `--check`
- **THEN** exit 1，stderr 點名 `CHANGELOG.zh-TW.md` 並印出差異摘要

#### Scenario: 找不到版本

- **WHEN** 執行 `--version 0.9.9` 而 JSON 無此版
- **THEN** exit 1 且 stderr 印出頂端版號

#### Scenario: 依語言印出片段

- **WHEN** 對同一版分別執行 `--version 0.3.0` 與 `--version 0.3.0 --lang zh-TW`
- **THEN** 前者印出英文片段（`## 0.3.0 (2026-09-10)`、`### New features`），後者印出繁中片段（`## 0.3.0（2026-09-10）`、`### 新功能`）

##### Example: 單版片段

- **GIVEN** 條目 0.3.0（2026-09-10）含 features 兩條、fixes 一條
- **WHEN** 執行 `--version 0.3.0 --lang zh-TW`
- **THEN** stdout 為

```
## 0.3.0（2026-09-10）

### 新功能

- …
- …

### 修正

- …
```


<!-- @trace
source: bilingual-release-notes
updated: 2026-10-15T17:28:24+08:00
-->

---
### Requirement: release 管線守門版號四處同版與渲染一致

release 管線 SHALL 在桌面建置開始之前斷言：tag 去 v 前綴後等於 tauri.conf.json 的 version、且等於 release-notes.json 頂端條目的 version；並執行渲染腳本 `--check`，同時比對 `CHANGELOG.md` 與 `CHANGELOG.zh-TW.md`。任一項不成立 SHALL 以 `::error::` 點名不一致的兩邊（或漂移的檔案）並以非零結束，SHALL NOT 進入建置、SHALL NOT 建立 Release。

#### Scenario: JSON 頂端版號落後

- **WHEN** push tag v0.3.0 但 release-notes.json 頂端仍為 0.2.0
- **THEN** desktop job 於斷言步驟紅燈、訊息含 0.3.0 與 0.2.0，沒有任何安裝檔建置

#### Scenario: CHANGELOG.md 漂移

- **WHEN** push tag 時 `CHANGELOG.md` 或 `CHANGELOG.zh-TW.md` 與 JSON 重算結果不一致
- **THEN** desktop job 於斷言步驟紅燈、訊息點名漂移的檔案並含差異摘要


<!-- @trace
source: bilingual-release-notes
updated: 2026-10-15T17:28:24+08:00
-->