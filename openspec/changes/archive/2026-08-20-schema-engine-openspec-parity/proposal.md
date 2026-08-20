## Summary

把內建 spec-driven schema 的存放方式與驗證強度對齊 OpenSpec v1.0.0，並補實作三個目前收下但不做事的 CLI 旗標。

## Motivation

比對 OpenSpec v1.0.0（Fission-AI/OpenSpec，speclink 的設計上游）後發現三處實質分歧：

- **雙份定義**：內建 schema 同時存在於手寫的 Rust 函式（`spec_driven()`）與 fork 時吐出的 YAML dump（fork.schema.yaml）。兩份靠人工同步，且已實際飄移——specs 的 instruction 在 .md 檔一側多出約 1.5KB 的現行規則（Purpose 段、BEFORE 註記、REMOVED-SCENARIO 合併門檻），YAML 一側是舊版。`speclink schema fork` 因此吐出過時的 instruction 給使用者。
- **驗證缺口**：自訂 schema 載入時不檢查重複 artifact id、不檢查 requires 指向不存在的 id、循環相依只回固定字串不印環路徑、schema 名稱無格式檢查；version／description／template 三欄位在 OpenSpec 為必填，speclink 全部容忍缺席。fork 出去的 schema 在兩邊工具的合法性因此不一致。
- **假旗標**：`schema which --all`、`schema validate --verbose`、`schema init --default` 三個旗標 CLI 收下後直接丟棄，對使用者是無聲的謊言。

另有一個衍生 bug：`speclink schemas` 列表顯示的內建 description（proposal → specs → design → tasks）與 fork.schema.yaml 的字面（proposal → specs → tasks (design optional)）對不起來，後者才符合實際的 requires 圖。

## Proposed Solution

- **單一正典**：fork.schema.yaml 升格為內建 schema 的唯一定義。載入時走與自訂 schema 相同的 serde 解析路徑；template 內容仍以 `include_str!` 內嵌，載入時依 template 檔名查表附掛。`spec_driven()` 保留函式名與簽名（呼叫端不動），函式體改為解析內嵌 YAML。`schema fork spec-driven` 吐出的內容與正典逐位元組相同。
- **飄移收斂**：先把五份 `*.instruction.md` 的現行內容摺入 fork.schema.yaml（.md 側是新的、代表現行 instructions payload 行為），再刪除這五份 .md 檔。收斂後 payload 內容不變，fork 輸出從過時變為現行。assets.lock 指紋不涵蓋 schema 資產（已驗證 render_fingerprint_input 的範圍），不需 bump MARKER_VERSION。
- **驗證強化**（載入時生效，`resolve` 既有的錯誤口）：補重複 artifact id、懸空 requires、循環相依印完整環路徑三項檢查；fork／init 的目的名稱套用 OpenSpec 的名稱格式（小寫 kebab-case）。必填化三項：version（正整數）、artifact description（鍵必填，值可為空）、artifact template（必填非空，移除 `<id>.md` 預設容錯）。
- **三旗標補實作**：`which --all` 列出所有 schema 各自的解析位置與被遮蔽的位置；`validate` 本體補 template 檔存在檢查，`--verbose` 印逐步驗證項目；`init --default` 把 `schema: <名稱>` 寫進 openspec/config.yaml，其餘內容逐位元組保留。
- **description 修正**：隨單一正典結構性消失——列表顯示的內建 description 直接來自正典 YAML 的字面。

## Non-Goals

- **spec-driven 內容語意不變**：instruction／template 的規則內容維持 speclink 現行（僅把已飄移的兩份收斂到現行行為），不回退到 OpenSpec 上游版本。
- **frozen output shape 不動**：instructions payload 的 template 只查內建、自訂 schema 列表不顯示 description，兩處凍結行為維持。
- **不真落磁碟**：內建 schema 不寫到安裝目錄——cargo install 只裝單一執行檔，落磁碟需要過期同步機制，重蹈安裝版 CLI 過期的同型坑。
- **不搬互動 prompt**：OpenSpec 的 `--no-default` 用於抑制互動詢問，speclink CLI 非互動，無物可抑制。
- **desktop 與 remote 面不在本 change**：專案設定的 schema 檢視／切換／fork 按鈕與 remote 限縮，屬第二個 change（desktop-schema-panel），依賴本 change 落地。
- **store 不納管 openspec/schemas/**：remote 自訂 schema 的前置屬 store 契約變更，明確遞延。

## Alternatives Considered

- **保留寬鬆驗證預設**：fork 出去的 schema 在 speclink 與 OpenSpec 的合法性不一致，放棄。
- **以 .md 檔為正典、build 時生成 YAML**：fork 需要可逐字出貨的單一檔案，YAML 為正典最簡單且與 OpenSpec 目錄形狀（schema.yaml + templates/）對齊，放棄生成方案。
- **內建 schema 真落磁碟**：見 Non-Goals，放棄。

## Impact

- Affected specs: 新 capability `workflow-schemas`（內建 schema 單一正典、驗證規則、指令旗標行為）
- Affected code:
  - Modified: crates/speclink-core/src/schema.rs（單一正典載入、驗證強化、fork／init 名稱檢查）、crates/speclink-core/assets/schema/spec-driven/fork.schema.yaml（摺入現行 instruction 內容）、crates/speclink-cli/src/verbs/toolchain.rs（三旗標實作、validate 補 template 檢查）、crates/speclink-core/src/config.rs（config.yaml 的 schema 鍵寫入口）
  - New: crates/speclink-cli/tests/it/schema_commands.rs（三旗標與驗證強化的整合測試）
  - Removed: crates/speclink-core/assets/schema/spec-driven/proposal.instruction.md、crates/speclink-core/assets/schema/spec-driven/specs.instruction.md、crates/speclink-core/assets/schema/spec-driven/design.instruction.md、crates/speclink-core/assets/schema/spec-driven/tasks.instruction.md、crates/speclink-core/assets/schema/spec-driven/apply.instruction.md（內容摺入正典 YAML 後移除）
- 呼叫端不受影響：`spec_driven()` 保名保簽名，lifecycle／archive／command／desktop verbs 等呼叫點零改動
