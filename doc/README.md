# 文件索引

日期：2026-05-19

本文整理目前 `docs/` 內值得保留的文件與使用定位。

## 主文件

### `speclink-provider-api-and-runtime-design.md`

新 SDD workflow engine 的主設計文件。

用途：

- 說明 Skill / CLI / Provider / Remote Service 分工。
- 定義人類負責 auth、provider、project binding，AI skill 只呼叫 CLI。
- 定義 local provider fallback。
- 定義固定 Provider API Contract。
- 定義 discuss、propose、pack、unpack、apply、finish、archive 的流程邊界。
- 說明新 skill 可參考 Spectra skills，但不硬耦合 `spectra-cli` 或本機 `openspec/` layout。

後續新專案設計應優先更新這份。

## Spectra 參考文件

### `spectra-cli-reverse-engineering.zh-TW.md`

Spectra CLI 反組譯與實驗紀錄的繁體中文版本。

用途：

- 作為 Spectra CLI 行為分析的證據來源。
- 保留 analyzer、drift scoring、schema resolution、SQLite、touched files、archive、park/unpark 等反推結果。
- 支撐新工具設計中的 workflow engine、artifact DAG、instructions、validation、state sync 等概念。

### `spectra-cli-skill-runtime-flow.md`

Spectra CLI 與 `$spectra-*` skills 的運作流程整理。

用途：

- 說明 Spectra 如何透過 skill 驅動 AI，再透過 CLI 管理 artifact、instructions、analysis、validation 與狀態。
- 作為新專案撰寫 skills 與 CLI command contract 的主要參考。
- 可用來比對哪些 Spectra 模式應保留，哪些本機耦合應移除。

## 過渡分析文件

### `spectra-discuss-propose-service-split.md`

早期針對「將 discuss / propose 拆到服務端」的拆分分析。

用途：

- 保留 discuss / propose 所需 CLI 能力的早期分析。
- 可作為拆解 `$spectra-discuss` 與 `$spectra-propose` 能力時的參考。

注意：

- 目前主方向已經演化成「Skill + CLI + 可替換 Provider + 狀態同步」。
- 這份文件不是最新主架構文件。
- 日後若主設計文件已完全吸收其內容，可以再考慮刪除或移入 archive。

## Archive

### `archive/spectra-cli-reverse-engineering.en.md`

Spectra CLI 反組譯紀錄的英文原稿。

用途：

- 保留原始英文版本，避免直接刪除造成資訊遺失。
- 平常以 `spectra-cli-reverse-engineering.zh-TW.md` 為主要閱讀版本。

## 建議閱讀順序

1. `speclink-provider-api-and-runtime-design.md`
2. `spectra-cli-skill-runtime-flow.md`
3. `spectra-cli-reverse-engineering.zh-TW.md`
4. `spectra-discuss-propose-service-split.md`

