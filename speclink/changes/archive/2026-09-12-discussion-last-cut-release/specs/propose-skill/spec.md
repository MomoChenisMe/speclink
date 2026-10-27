## ADDED Requirements

### Requirement: 從討論轉出最後一刀時帶 --last

內嵌 speclink-propose 技能（事實來源 crates/engine/speclink-core/assets/skills/propose.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 於「以 --from-discussion 建立變更」那一步規定最後一刀的判定與標記：代理人 SHALL 讀取討論記錄 Conclusion 的 Decision 段所列的分期刀清單（刀一／刀二／…或 cut A／cut B 等任一寫法）與記錄 frontmatter 的 promoted_to 清單；本次建立的變更是結論規劃的最後一刀時，speclink new change 的 --from-discussion SHALL 併帶 --last；立案期把結論的一刀拆成多個變更時，SHALL 只在拆出的最後一段帶 --last；結論未規劃分期（單刀）時 SHALL NOT 帶（記錄無 hold 行時帶了亦為無害無操作，但技能檔 SHALL 以「不帶」為規定）。技能檔 SHALL 說明帶 --last 的效果：記錄的 hold 解除，其後最後一個轉出變更封存時記錄自動隨行封存；並 SHALL 註明標錯（不是最後一刀卻帶了）的後果與救援：記錄會在最後一個在途變更封存時被隨行封存，再立一刀前把記錄自 openspec/discussions/archive/ 搬回 openspec/discussions/。技能檔 SHALL 在指令範例中展示帶 --last 的 speclink new change 形式。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 最後一刀的判定規則

- **WHEN** 檢視渲染產出的 speclink-propose 技能檔（claude 與 codex 兩工具）的 --from-discussion 建變更段落
- **THEN** 內容 SHALL 規定讀 Decision 的刀清單與 promoted_to 比對、最後一刀帶 --last、拆刀時只在最後一段帶、單刀不帶；SHALL 含帶 --last 的指令範例；SHALL 說明效果與標錯的搬回救援；render golden SHALL 同步反映

##### Example: 三刀系列的判定

| Decision 的刀清單 | promoted_to | 本次立的變更 | 帶 --last？ |
| --- | --- | --- | --- |
| 刀一、刀二、刀三 | （空） | 刀一 | 否 |
| 刀一、刀二、刀三 | cut-a, cut-b | 刀三 | 是 |
| 刀一、刀二、刀三 | cut-a, cut-b | 刀三拆出的前半 | 否 |
| 刀一、刀二、刀三 | cut-a, cut-b, cut-c1 | 刀三拆出的後半 | 是 |
| 單刀（未規劃分期） | （空） | 唯一的變更 | 否 |
