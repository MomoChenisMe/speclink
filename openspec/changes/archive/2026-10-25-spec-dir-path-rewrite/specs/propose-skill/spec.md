## MODIFIED Requirements

### Requirement: 從討論轉出最後一刀時帶 --last

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

內嵌 speclink-propose 技能（事實來源 crates/engine/speclink-core/assets/skills/propose.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 於「以 --from-discussion 建立變更」那一步規定最後一刀的判定與標記：代理人 SHALL 讀取討論記錄 Conclusion 的 Decision 段所列的分期刀清單（刀一／刀二／…或 cut A／cut B 等任一寫法）與記錄 frontmatter 的 promoted_to 清單；本次建立的變更是結論規劃的最後一刀時，speclink new change 的 --from-discussion SHALL 併帶 --last；立案期把結論的一刀拆成多個變更時，SHALL 只在拆出的最後一段帶 --last；結論未規劃分期（單刀）時 SHALL NOT 帶（記錄無 hold 行時帶了亦為無害無操作，但技能檔 SHALL 以「不帶」為規定）。技能檔 SHALL 說明帶 --last 的效果：記錄的 hold 解除，其後最後一個轉出變更封存時記錄自動隨行封存；並 SHALL 註明標錯（不是最後一刀卻帶了）的後果與救援：記錄會在最後一個在途變更封存時被隨行封存，再立一刀前把記錄自 speclink/discussions/archive/ 搬回 speclink/discussions/。技能檔 SHALL 在指令範例中展示帶 --last 的 speclink new change 形式。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

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

### Requirement: 依是否改到正式規格選擇 schema

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

propose 技能 SHALL 在建立變更之前決定它的 schema，依下列順序取第一個成立的規則：

1. 使用者在需求或參數中指定 schema → 用指定的 schema。
2. 專案預設 schema（`speclink workflow-config show --json` 的 `schema`）是自訂 schema → 用專案預設，不做判定。
3. 需求來自討論（`--from-discussion`）→ 依討論結論判定：結論寫明不改任何正式規格承諾的行為、或點名 `no-spec-change` → `no-spec-change`；結論會新增、修改或移除行為 → `spec-driven`。
4. 其餘 → 依需求內容與步驟 3 的正式規格掃描判定「這個變更會不會改到正式規格承諾的行為？」：不會 → `no-spec-change`；會 → `spec-driven`。修 bug 預設為 `spec-driven`，因為通常要寫 delta 把正確行為釘住；只有能說明「沒有任何正式規格承諾被改到」時才選 `no-spec-change`。

規則 3 或 4 判斷不了時，技能 SHALL 以兩個選項詢問使用者（建議的選項放第一個並寫出理由），SHALL NOT 自行猜測。技能 SHALL 以 `speclink new change "<name>" --schema <選定的 schema>` 建立變更。選定 `no-spec-change` 時，proposal SHALL 依該 schema 的 template 撰寫（不含 Capabilities），Why SHALL 寫明為什麼不改正式規格承諾的行為並列出查過的正式規格；收尾摘要 SHALL 寫出選定的 schema，`no-spec-change` 以「不改規格（no-spec-change）」呈現。

#### Scenario: 討論結論寫明不改行為

- **WHEN** 使用者以 `--from-discussion` 執行 propose，討論結論寫明「純重構，不改任何正式規格承諾的行為」，使用者沒有指定 schema，專案預設 schema 為 spec-driven
- **THEN** 技能以 `--schema no-spec-change` 建立變更，proposal 沒有 Capabilities 段，Why 列出查過的正式規格，收尾摘要寫「不改規格（no-spec-change）」

#### Scenario: 修 bug 預設寫 delta

- **WHEN** 使用者以「修正封存時日期錯一天」為需求執行 propose，沒有指定 schema，該行為由正式規格承諾
- **THEN** 技能以 `--schema spec-driven` 建立變更，並寫出修正該行為的 delta spec

#### Scenario: 使用者指定優先

- **WHEN** 使用者執行 propose 時寫明「用 spec-driven」，需求內容看起來是不改行為的重構
- **THEN** 技能以 `--schema spec-driven` 建立變更，不再詢問

#### Scenario: 專案預設為自訂 schema

- **WHEN** 專案 `speclink/config.yaml` 的 `schema` 為自訂的 `my-flow`，使用者沒有指定 schema
- **THEN** 技能以專案預設 `my-flow` 建立變更，不做 spec-driven 與 no-spec-change 的判定

#### Scenario: 判斷不了時詢問

- **WHEN** 需求為「調整匯出檔的欄位順序」，技能無法從正式規格判斷欄位順序是否為承諾的行為
- **THEN** 技能以兩個選項詢問使用者，建議的選項放第一個並附理由；使用者回答前不建立變更
