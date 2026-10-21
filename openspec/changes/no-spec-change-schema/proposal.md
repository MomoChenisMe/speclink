## Why

speclink 只有一個內建 schema（spec-driven），每個變更都要寫 delta spec；不改任何行為的重構、用詞修正與平台小修沒有官方路徑，只能寫一份空洞的 delta，或乾脆繞過 speclink（775 個 commit 有 20% 完全沒碰 openspec/）。資料顯示這類變更確實存在：284 個封存變更有 14 個沒有 delta，多為重構與用詞修正，而且其中 9 個仍跑了兩道品質關卡——它們缺的是「不改規格」的正式流程，不是更少的檢查。本變更是討論 positioning-and-process-cost 分刀計畫的第 3 刀，切換已建立變更 schema 的指令由下一刀 change-schema-verb 提供。

目標使用者與情境：透過 AI agent 開發的開發者（團隊情境下含 PO 與 PM），在 propose 階段由 AI 依討論結論或需求判定 schema，在 verify 階段確認「不改規格」的變更真的沒有改到正式規格承諾的行為。

## What Changes

- 新增第二個內建 schema `no-spec-change`（中文「不改規格」）：proposal → design（選用）→ tasks，沒有 specs；由內嵌的正典 YAML 定義，與 spec-driven 走相同的載入與驗證路徑；本機、遠端與桌面都可用。
- 判準只有一個：「這個變更會不會改到正式規格承諾的行為？」不會才選 `no-spec-change`。修 bug 多半要寫 delta 把正確行為釘住（52 個修 bug 的變更有 51 個寫了 delta），所以不等於 `no-spec-change`。
- `no-spec-change` 的 proposal 在 Why 寫出「為什麼這個變更不改行為」，並列出查過的正式規格；tasks 最後要有一項以既有測試確認行為未變。
- `speclink validate` 對 schema 沒有 specs 產出的變更，不再警告「No delta specs found」。
- propose 技能新增 schema 選擇：使用者指定時照指定；專案預設是自訂 schema 時用預設；來自討論時依討論結論判定；不經討論時依需求與正式規格掃描判定；判斷不了就問使用者；建立變更時帶 `--schema`。
- verify 技能對 `no-spec-change` 變更改查「凍結 patch 有沒有改到正式規格承諾的行為」，改到時回報 WARNING（必修，擋章），建議改用 spec-driven 並補 delta。
- LANGUAGE.md 新增「不改規格」詞條。
- 桌面產出流程頁的清單與切換下拉列出兩個內建 schema；遠端模式的下拉也包含兩者。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-schemas`: 「單一正典載入」由只有 spec-driven 改為每個內建 schema 各有一份內嵌正典 YAML；新增 no-spec-change 內建 schema 的組成與 validate 行為。
- `propose-skill`: 新增依「是否改到正式規格」選擇 schema 的規則。
- `verify-skill`: 新增「不改規格」變更的行為檢查。
- `desktop-config`: 遠端模式切換下拉的可選目標由「只有 spec-driven」改為兩個內建 schema。

## Impact

- Affected specs: `workflow-schemas`, `propose-skill`, `verify-skill`, `desktop-config`
- Affected code:
  - New: crates/engine/speclink-core/assets/schema/no-spec-change/fork.schema.yaml, crates/engine/speclink-core/assets/schema/no-spec-change/proposal.template.md
  - Modified: crates/engine/speclink-core/src/workspace/schema.rs, crates/engine/speclink-core/src/quality/validate.rs, crates/engine/speclink-core/assets/skills/propose.md, crates/engine/speclink-core/assets/skills/verify.md, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/tests/golden/assets.lock, apps/desktop/core/src/settings.rs, openspec/LANGUAGE.md
- 影響的 crate／app：speclink-core（內建 schema 登記、validate、技能原稿）、apps/desktop/core（schema 快照測試）；speclink-cli 與 speclink-server 透過既有的 schema 解析自動支援，不需改程式。
- CLI：沒有新指令或旗標。`speclink schemas`、`speclink schema which --all` 多列出 `no-spec-change`；`speclink schema fork no-spec-change` 可用；`speclink new change --schema no-spec-change` 可用。
- 設定欄位：`openspec/config.yaml` 的 `schema:` 可設為 `no-spec-change`（不建議當專案預設）；沒有新增欄位。
- 生成的技能：speclink-propose 與 speclink-verify 的技能檔（claude、codex、copilot）內容改變；ASSET_VERSION 由 v1.49.0 升為 v1.50.0，golden snapshot 與 assets.lock 再生。
- 相容性影響：`speclink schemas` 與 `schema which --all` 的人眼與 `--json` 輸出多一個內建項（刻意變更）；spec-driven 的載入、instructions 輸出與 fork 結果位元級不變；既有變更的 validate 輸出不變（它們的 schema 都有 specs 產出）。
