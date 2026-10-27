## Purpose

更新日誌的資料契約與衍生物：JSON 檔的位置與欄位、由它畫出 CHANGELOG.md 與 Release 說明片段的渲染規則、release 管線對版號一致與渲染一致的守門。邊界：日誌內容怎麼產生屬 release-skill；桌面 app 怎麼呈現屬 desktop-app；Release 說明的其餘內容屬 desktop-release。

## ADDED Requirements

### Requirement: 更新日誌 JSON 為唯一真相

repo SHALL 在 `apps/desktop/src/release-notes/release-notes.json` 保存更新日誌，為一個陣列、最新版本在前。每筆 SHALL 含 `version`（`X.Y.Z`，不帶 v 前綴，與 tauri.conf.json 的 version 同字面）、`date`（`YYYY-MM-DD`）、`sections`（陣列，每個元素含 `title` 與 `items`）。`title` SHALL 只取「新功能」「修正」「改善」三值之一；`items` SHALL 為非空的繁體中文字串陣列；沒有條目的分組 SHALL NOT 寫入。CHANGELOG.md 與 Release 說明的更新內容 SHALL 只由此檔衍生，SHALL NOT 另行手寫。

#### Scenario: 條目形狀

- **WHEN** 讀取 release-notes.json 頂端條目
- **THEN** 其 version 為 `X.Y.Z` 格式、date 為 `YYYY-MM-DD`、每個 section 的 title 為三值之一且 items 非空

##### Example: 一筆條目

- **GIVEN** 0.3.0 於 2026-09-10 發版，含兩個新功能與一個修正
- **WHEN** 檢視頂端條目
- **THEN** `{"version":"0.3.0","date":"2026-09-10","sections":[{"title":"新功能","items":["…","…"]},{"title":"修正","items":["…"]}]}`，沒有「改善」分組

### Requirement: 渲染腳本由 JSON 產出 CHANGELOG.md 與單版片段

repo SHALL 提供渲染腳本 `scripts/release-notes-render.mjs`，同一份渲染邏輯支援三種用法：`--write` 把全部條目寫成 repo 根 `CHANGELOG.md`；`--version X.Y.Z` 把該版條目的片段印到 stdout；`--check` 重算並與現有 CHANGELOG.md 比對。片段格式 SHALL 為 `## X.Y.Z（YYYY-MM-DD）` 標題，其下每個分組為 `### <title>` 標題接 `- ` 條目。CHANGELOG.md SHALL 以 `# 更新日誌` 開頭，其後依 JSON 順序接每版片段，換行一律 LF。`--version` 找不到該版 SHALL 以非零結束並於 stderr 印出 JSON 頂端版號；`--check` 不一致 SHALL 以非零結束並於 stderr 印出差異摘要，比對前 SHALL 把讀入檔的 CRLF 正規化為 LF。腳本 SHALL 有自動化測試涵蓋固定輸出、三種用法的 exit code 與 CRLF 正規化。

#### Scenario: 產出 CHANGELOG.md

- **WHEN** JSON 有 0.3.0 與 0.2.0 兩筆並執行 `--write`
- **THEN** CHANGELOG.md 首行為 `# 更新日誌`，其後先 `## 0.3.0（…）` 再 `## 0.2.0（…）`，exit 0

#### Scenario: 比對漂移

- **WHEN** 有人手改 CHANGELOG.md 一個字後執行 `--check`
- **THEN** exit 1 且 stderr 印出差異摘要

#### Scenario: 找不到版本

- **WHEN** 執行 `--version 0.9.9` 而 JSON 無此版
- **THEN** exit 1 且 stderr 印出頂端版號

##### Example: 單版片段

- **GIVEN** 條目 0.3.0（2026-09-10）含「新功能」兩條、「修正」一條
- **WHEN** 執行 `--version 0.3.0`
- **THEN** stdout 為

```
## 0.3.0（2026-09-10）

### 新功能

- …
- …

### 修正

- …
```

### Requirement: release 管線守門版號四處同版與渲染一致

release 管線 SHALL 在桌面建置開始之前斷言：tag 去 v 前綴後等於 tauri.conf.json 的 version、且等於 release-notes.json 頂端條目的 version；並執行渲染腳本 `--check`。任一項不成立 SHALL 以 `::error::` 點名不一致的兩邊（或漂移的檔案）並以非零結束，SHALL NOT 進入建置、SHALL NOT 建立 Release。

#### Scenario: JSON 頂端版號落後

- **WHEN** push tag v0.3.0 但 release-notes.json 頂端仍為 0.2.0
- **THEN** desktop job 於斷言步驟紅燈、訊息含 0.3.0 與 0.2.0，沒有任何安裝檔建置

#### Scenario: CHANGELOG.md 漂移

- **WHEN** push tag 時 CHANGELOG.md 與 JSON 重算結果不一致
- **THEN** desktop job 於斷言步驟紅燈、訊息含差異摘要
