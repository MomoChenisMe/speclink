## Summary

清除全 repo 對 Spectra 的進行式對齊語意——輸出凍結的權威從「與 Spectra 一致」改為 speclink 自身已發佈的契約；README 保留明確的歷史參考。零行為變更。

## Motivation

speclink 已完成自立，對 Spectra 的對齊是歷史事實而非進行式目標。殘留的進行式措辭有三類實害：

1. README 聲稱以「parity/golden tests」保護相容基準，但 parity suite 並不存在於 repo——文件失真。
2. 正典規格 12 份以「對 Spectra 2.3.1 的 parity 基線」定義輸出凍結，其中 store-abstraction 更點名不存在的 parity_suite 31 項／color_suite 16 項／twin harness 8 情境為驗證載體——規格要求執行不存在的測試，AI 代理照做時只能編造結果或卡住。
3. 源碼約 128 處「(matches Spectra)」類註解誤導新維護者以為需安裝 Spectra 驗證行為。

目標使用者：接手本 repo 的開發者，以及經 AI 代理跑 SDD 流程的維護者（產出 proposal／specs／design／tasks 時會讀取正典規格與 README）。使用情境：verify 與 archive 階段的規格閱讀、propose 階段的規則遵循、日常維護的源碼導讀。

## Proposed Solution

六類處置，詳細措辭對照見 design：

1. **README 兩份與 docs/platform-architecture.zh-TW.md**：改為歷史參考語氣——保留「設計之初以 Spectra App 2.3.1 所附 CLI 為行為參考」的明確提及，移除「相容基準」進行式承諾與不存在的 parity tests 字樣；回歸保護改述為 golden 與 CLI 整合測試。
2. **正典規格 12 份 18 處**：以 delta 改寫為自有契約——SHALL 句的凍結主體不變，「對 Spectra 2.3.1 的 parity 基線」改為「既有輸出基線」、「不在 Spectra 對照範圍」刪去 Spectra 子句、「屬對 Spectra 的刻意分歧」歷史對比改為直述刻意設計；store-abstraction 的驗證載體改指向實際存在的 golden 與 CLI 整合測試；user-documentation 對 README 的強制要求同步改為歷史參考語氣。
3. **內嵌技能資產**：archive.md 的「Unlike Spectra…」對比句改為直述 RENAMED 行為，三處同步（core assets、claude 技能實例、codex 技能實例）。
4. **golden 快照四份**：於乾淨樹再生（與 3 同批的刻意變更）。
5. **源碼註解批次改寫**：以中性措辭（如描述輸出形狀凍結的理由）取代 Spectra 指涉，單一 mechanical commit、零行為變更。
6. **不動**：封存變更、討論記錄、LANGUAGE.md 的 why 記錄、prompt.md、規格內 @trace 歷史清單。

## Non-Goals

- 不刪除任何測試——現存 render_golden 與 CLI 整合測試全是自我基線，保護的是現行為。
- 不回改封存 artifacts 與 @trace 清單（歷史不回改）。
- 不改變任何 CLI 行為、人眼輸出、exit code 或 --json 欄位——本變更零行為變更。
- 不移除 README 的 Spectra 參考（使用者明確要求保留歷史起源）。
- 不新增「禁止 Spectra 字樣」的常駐守衛測試——收尾以一次性 grep 驗證即可，常駐守衛會誤傷歷史記錄。

## Alternatives Considered

- 源碼註解 boy-scout 漸改：被否——當下無 in-flight 變更，批次一刀零衝突；漸改讓誤導性註解長期存留。
- 刪除提及 Spectra 的測試：被否——那些測試是自我基線，與 Spectra 無關。
- README 完全移除 Spectra：被否——使用者明確要求保留參考。

## 相容性影響

- CLI 人眼輸出與 `--json`：零變更，既有回歸測試須全數通過。
- 渲染技能產物（init／update 產出的 speclink-archive 技能檔）文字變更：對比句改為直述——屬刻意變更，golden 四份同批更新並審視 diff。
- 技能與注入區塊：影響 speclink-archive 一支技能、claude 與 codex 兩工具的渲染產物；CLAUDE.md／AGENTS.md 注入區塊不變。

## Impact

- Affected specs: board-card-order、change-lifecycle、command-runtime、commit-skill、dev-harness、discussion-docs、remote-connection、store-abstraction、user-documentation、verb-contract、workflow-config、workspace-tools（12 個能力全為 MODIFIED——措辭改寫、行為不變）
- Affected code:
  - Modified: README.md、README.en.md、docs/platform-architecture.zh-TW.md、crates/speclink-core/assets/skills/archive.md、.claude/skills/speclink-archive/SKILL.md、.agents/skills/speclink-archive/SKILL.md、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md；另含源碼註解批次改寫——crates/speclink-core/src/、crates/speclink-cli/src/、crates/speclink-cli/tests/、crates/speclink-host/src/context.rs、packages/ui/src/、apps/desktop/src/ 下含 Spectra 字樣的註解檔（以 grep 定位，清單見 tasks）
  - New: （無）
  - Removed: （無）

影響的 crate 或 app：speclink-core（assets、源碼註解、golden）、speclink-cli、speclink-host、packages/ui、apps/desktop；文件面 README 與 docs。
