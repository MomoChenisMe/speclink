# archive-skill Specification

## Purpose

TBD - created by archiving change 'archive-skill-touched-cleanup-order'. Update Purpose after archive.

## Requirements

### Requirement: touched 記錄的刪除排在封存與提交之後

內嵌 speclink-archive 技能（事實來源 crates/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 規定：`.speclink/touched/<change>.json` 的刪除 SHALL 排在 speclink archive 執行完成之後，且 SHALL 排在該次封存的提交完成之後；技能檔 SHALL NOT 指示在封存前刪除該檔。技能檔 SHALL 寫明兩項理由：該記錄是 @trace 檔案清單的來源，封存前刪除會使清單退回掃描工作樹髒檔而混入無關檔案；該記錄同時是 commit 技能的檔案歸屬來源，提交前刪除會使檔案清單消失。單一封存與 bulk 封存兩段 SHALL 採相同時序，不得互相矛盾。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物將清理排在封存之後

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-archive 技能檔中，刪除 `.speclink/touched/<change>.json` 的指示出現在執行 speclink archive 的步驟之後，且該指示註明須待提交完成

#### Scenario: 封存前不得刪除追蹤檔

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔對追蹤檔清理的規定
- **THEN** 技能檔不含任何在 speclink archive 之前刪除該檔的指示，並寫明封存前刪除會使 @trace 清單退回掃描工作樹髒檔

#### Scenario: 單一與 bulk 封存時序一致

- **WHEN** 比對渲染產出的 speclink-archive 技能檔中單一封存流程與 bulk 封存段對追蹤檔清理時機的敘述
- **THEN** 兩處均將刪除排在封存之後，且無一處指示先刪除再封存


<!-- @trace
source: archive-skill-touched-cleanup-order
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
-->

---
### Requirement: @trace 來源敘述與引擎行為一致

技能檔 SHALL 以條件式敘述說明 @trace 檔案清單的來源：該 change 存在 evidence 記錄時 SHALL 敘明清單由該記錄聚合建立，記錄缺席時 SHALL 敘明才退回工作樹的髒檔集；SHALL NOT 將髒檔集無條件敘述為 @trace 的唯一來源。bulk 封存對整潔工作樹的要求 SHALL 保留，其理由 SHALL 敘明為避免記錄缺席時退路取得無關檔案。

#### Scenario: bulk 段敘明來源優先序

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔 bulk 封存段對 @trace 來源的敘述
- **THEN** 敘述指出 evidence 記錄存在時以記錄為準、缺席時才退回工作樹髒檔集，且未將髒檔集寫成無條件的唯一來源

#### Scenario: 整潔工作樹要求保留

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔 bulk 封存段對工作樹狀態的要求
- **THEN** 該要求仍在，且其理由敘明為避免 evidence 記錄缺席時退路取得無關檔案

<!-- @trace
source: archive-skill-touched-cleanup-order
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
-->