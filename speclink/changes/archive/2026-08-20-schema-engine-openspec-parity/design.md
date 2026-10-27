## Context

內建 spec-driven schema 目前有兩份定義：手寫的 Rust 函式（crates/speclink-core/src/schema.rs 的 spec_driven()）與 fork 時逐字吐出的 YAML dump（crates/speclink-core/assets/schema/spec-driven/fork.schema.yaml）。兩份靠一個標記同步測試維繫，且已實際飄移——specs 的 instruction 在 .md 資產側多出 Purpose 段規則、BEFORE 註記步驟、REMOVED-SCENARIO 合併門檻約 1.5KB 現行內容，YAML 側是舊版，fork 因此吐出過時指引。另外驗證缺口與三個假旗標見 proposal。上游對照物為 OpenSpec v1.0.0 的 src/core/artifact-graph（zod 驗證）與 src/commands/schema.ts（旗標行為）。

## Goals / Non-Goals

**Goals:**

- 內建 schema 只有一份定義，內建與自訂走同一條解析與驗證路徑
- 驗證強度對齊 OpenSpec：六項載入檢查＋fork／init 名稱格式
- 三個旗標具備真實行為
- instructions payload 逐字維持現行；fork 輸出從過時收斂到現行

**Non-Goals:**

- spec-driven 內容語意變更（只收斂飄移，不回退上游版本）
- frozen output shape 變更（instructions 只查內建 template、自訂列表無 description）
- 內建 schema 落磁碟、互動 prompt、desktop／remote 面、store 納管 openspec/schemas/（見 proposal Non-Goals 與 Deferred）

## Decisions

### D1 單一正典載入

spec_driven() 保留函式名與公開簽名（回傳 Schema），呼叫端（lifecycle、archive、command、validate 測試、desktop verbs）零改動。函式體改為：以 std::sync::OnceLock 快取「解析內嵌 fork.schema.yaml ＋ 依 template 檔名附掛內嵌 template 內容」的結果，每個行程解析一次後 clone。template 附掛表：proposal.md、spec.md、design.md、tasks.md 四個檔名對映到既有的 include_str! template 常數。解析用與自訂 schema 相同的 SchemaYaml serde 結構與同一個驗證函式；source 維持 package、name 維持 spec-driven（解析鍵不變，resolve 中 spec-driven 仍是唯一內建名）。builtin_template() 改查快取後的內建 Schema。內建 description 從 YAML 字面來——列表與 fork 兩處自然一致，雙份 description bug 結構性消失。

**替代方案**：build script 生成 Rust 碼（多一層生成機制，除錯差）；每次呼叫重新解析（浪費且 spec_driven() 在熱路徑被多處呼叫）。皆捨。

### D2 飄移收斂

收斂方向為 .md → YAML：五份 instruction 資產中四份（proposal、design、tasks、apply）與 YAML 已逐位元組相同（已驗證），僅 specs 的 .md 側多出三段現行內容。步驟：把 crates/speclink-core/assets/schema/spec-driven/specs.instruction.md 的全文覆蓋進 fork.schema.yaml 的 specs instruction 區塊，然後刪除五份 instruction .md 資產檔與其 include_str! 常數。既有的雙份標記同步測試（釘 [M] markers 字樣那個）改為只釘正典 YAML 一份。

不 bump MARKER_VERSION 的依據：assets.lock 的指紋輸入（render_fingerprint_input）只涵蓋 instructions body 與 skills render，schema 資產不在雜湊範圍——已讀 crates/speclink-core/tests/it/render_golden.rs 確認。

**行為保持守則**：instructions payload 各 artifact 的 instruction 欄位輸出逐字不變。守門測試釘三段飄移標記（Purpose section (new capabilities only)、BEFORE:、REMOVED-SCENARIO）存在於正典 YAML 的 specs instruction。

### D3 驗證強化

單一驗證函式，內建與自訂同路，於載入口（resolve 的既有錯誤口）生效：

1. artifact id 不得重複
2. requires 只指向存在的 id（指名 artifact 與缺席 id）
3. requires 圖無循環；錯誤訊息帶完整環路徑（形如 a → b → a，對齊 OpenSpec 的 Cyclic dependency detected 訊息形）
4. version 鍵必填且為正整數（serde 收 Value 後驗，同時擋 0、負數、小數、非數值）
5. artifact description 鍵必填（值可為空字串——對齊 OpenSpec zod 的 z.string() 語意）
6. artifact template 鍵必填且非空；移除以 artifact id 推導預設檔名的容錯

fork 與 init 的目的名稱套 OpenSpec 的 isValidSchemaName 正則（小寫 kebab-case），不符以非 0 exit code 拒絕；預設名 spec-driven-custom 天然合法。錯誤訊息維持既有前綴慣例（Schema parse error: ／ Invalid schema: ），無測試凍結這些字串（已 grep 確認），環路徑訊息可自由改進。Breaking 面：現存缺欄位的自訂 schema 載入即錯，錯誤指名欄位，修法為補鍵或重新 fork。

### D4 旗標與 config 寫入

- which --all：對 list_all 的每個名稱跑 sources，列出解析到的位置與被遮蔽位置；--json 形狀為陣列項 {name, resolved, sources[]}（沿用單名 which 的 json 項形）。
- validate：本體補 template 檔存在檢查——自訂查 schema 目錄 templates/ 下該檔、內建查內嵌附掛表；缺席即非 0 退出並指名檔名。--verbose 逐項印出各步驟（解析、重複 id、requires 引用、循環、template 檔）與結果。
- init --default：config.rs 新增 byte-preserving 的 schema 鍵 setter——沿用既有保序更新機制的紀律：只動 schema 一行，其餘內容逐位元組保留；config.yaml 缺席時建立僅含 schema 鍵的檔案；既有檔無法解析時 fail closed 拒寫（與 workflow-config 的 fail-closed 慣例一致）。CLI 端把 init 的 default 旗標接通到 setter。

## Implementation Contract

**可觀察行為**：

- speclink schema fork spec-driven 產出檔與正典 YAML 逐位元組相同；specs instruction 含 Purpose 段規則
- speclink schemas 顯示內建 description 為正典字面（含 design optional 字樣）
- 非法 schema（重複 id、懸空 requires、循環、缺 version／description／template）載入即錯，訊息指名問題欄位；循環訊息帶完整環路徑
- speclink schema which --all、validate --verbose、init --default 具備 D4 所述行為
- speclink instructions 各 artifact 的 instruction 欄位輸出與本變更前逐字相同

**介面**：pub fn spec_driven() -> Schema 簽名不變；config.rs 新增一個公開的 schema 鍵 setter（單一函式，收原文與名稱、回新文）；CLI 旗標面僅移除三個旗標的丟棄行為，不新增旗標。

**失敗形**：載入錯誤走 resolve 既有的 Err(String) 通道；validate 與 init 失敗以非 0 exit code 結束並印單行原因；init --default 遇無法解析的 config.yaml 拒寫且骨架建立結果保留（骨架已建、預設未設，訊息明說）。

**驗收**：cargo test --workspace 全綠；新增 CLI 整合測試 crates/speclink-cli/tests/it/schema_commands.rs 涵蓋 D3 名稱檢查與 D4 三旗標；speclink validate schema-engine-openspec-parity 通過。

**範圍邊界**：in scope——crates/speclink-core/src/schema.rs、crates/speclink-core/src/config.rs、crates/speclink-cli/src/verbs/toolchain.rs、crates/speclink-core/assets/schema/spec-driven/ 資產增刪、相關測試。out of scope——desktop 與 server 任何檔案、store 三驅動、skills 資產、MARKER_VERSION 與 assets.lock、instructions payload 的形狀。

## Risks / Trade-offs

- [必填化讓現存寬鬆自訂 schema 載入即壞] → 錯誤指名缺席欄位；release note 註明補鍵即修；重新 fork 也可得到合法骨架
- [摺入 YAML 時手滑改到內容，payload 靜默變化] → 守門測試釘三段飄移標記；specs instruction 摺入採整檔覆蓋而非手改段落
- [執行期解析 YAML 的啟動成本] → OnceLock 行程內單次解析；fork.schema.yaml 僅 224 行
- [config.yaml setter 損毀使用者內容] → 逐位元組保留紀律＋寫後重讀驗證（沿用 desktop settings 的 rewrite verification 模式）；無法解析即 fail closed

## Migration Plan

spec-driven 使用者零遷移。自訂 schema 擁有者需補齊 version／description／template 三鍵（release note 載明）。回滾＝revert 單一 commit；無資料遷移。

## Open Questions

無。
