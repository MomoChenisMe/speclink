# Language — 共用詞彙

專案的正典詞彙。GUI 文案、artifacts 散文、skills 說明遵循此表；Rust 識別符、CLI 輸出（英文）、
結構標記（### Requirement: 等）不在此範圍。

## 原則

- 動詞直說結果：使用者看到動詞就能推出「按下去會發生什麼」。
- 一個概念一個詞：同義詞在 avoid 列出，舊文案陸續汰換；歷史 artifacts（已封存的討論／變更）不回改。
- 工程詞（meta 欄位名、kebab-case、slug 等）不出現在使用者可見文案，只出現在給 agent 的文件。

## 詞彙

### 轉為變更

- **definition**: 把一份已結論的討論升級成一個新的變更（change）——建立變更卡、提案以討論結論開頭、討論記為已轉出。對應引擎動詞 `discuss promote`。
- **avoid**: 促轉、promote（中文散文中）
- **why**: 「促轉」是自造縮譯，無法從字面推出結果；「轉為變更」與看板「變更」頁名直接呼應。

### 已轉出變更

- **definition**: 討論的 promoted 狀態——至少連結一個變更（轉出新變更，或以引擎動詞 `discuss link` 併入既有變更）。看板上以「已轉出變更的討論」群組收合呈現。
- **avoid**: 已促轉
- **why**: 同上；名詞化後仍可讀。定義自「轉出過至少一個變更」放寬：ingest 型結論經 link 併入既有變更也走同一狀態與生命週期（discuss-link-verb，2026-07-08）。

### 再轉出一個變更

- **definition**: 對同一份討論再次轉為變更（一份討論可扇出多個變更）。
- **avoid**: 再促轉
- **why**: 扇出語意明確。

### 封存

- **definition**: 把完成的變更或收尾的討論移入 archive（`openspec/changes/archive/`、`openspec/discussions/archive/`），於「已封存」頁唯讀檢視。對應引擎動詞 `archive`。
- **avoid**: 歸檔
- **why**: 「封存」是 change 側與已封存頁的既定詞；同概念兩詞（歸檔/封存）曾在討論卡按鈕上並存。

### 衍生變更

- **definition**: 一份討論轉出的變更清單（討論抽屜的分頁名；引擎欄位 `promoted_to`）。
- **avoid**: 促轉分頁、子 change（使用者可見文案中）
- **why**: 分頁內容是「生出來的變更們」，不是動作本身。

### 輪

- **definition**: 討論的推進單位（引擎的 round）。文案寫「N 輪」「討論 N 輪」。
- **avoid**: 回合、N 回合
- **why**: 口語、更短。

### 背景

- **definition**: 討論記錄的 Context 區段（討論抽屜分頁名）。
- **avoid**: 脈絡（分頁名中）
- **why**: 較常用的日常詞。

### 專案說明

- **definition**: `openspec/config.yaml` 的 `context` 欄位——注入 AI 指令的專案自由文字說明（設定頁的編輯區段名）。
- **avoid**: context（使用者可見文案中）、背景（此概念上）
- **why**: 「背景」已被討論記錄的 Context 區段佔用，同詞兩義會混淆；對齊 Spectra 用詞。2026-07-07 討論「config-context-與-rules-gui-編輯」定案。

### 產出規則

- **definition**: `openspec/config.yaml` 的 `rules` 欄位——依 artifact 注入產出指令的規則清單（設定頁的編輯區段名）。
- **avoid**: rules（使用者可見文案中）、規則（單獨使用時）
- **why**: 「產出」點明規則作用於 artifacts 的產出過程；對齊 Spectra 用詞。2026-07-07 討論「config-context-與-rules-gui-編輯」定案。
