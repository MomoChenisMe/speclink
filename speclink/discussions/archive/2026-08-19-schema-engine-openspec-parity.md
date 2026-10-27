---
topic: 同步 speclink 的 schema 引擎對齊 OpenSpec（存放方式、驗證強度、CLI 旗標），並評估 desktop 檢視／替換預設 schema 與 remote 模式
slug: schema-engine-openspec-parity
status: promoted
promoted_to: schema-engine-openspec-parity
created: 2026-08-19
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 同步 speclink 的 schema 引擎對齊 OpenSpec（存放方式、驗證強度、CLI 旗標），並評估 desktop 檢視／替換預設 schema 與 remote 模式

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：比對 OpenSpec v1.0.0（Fission-AI/OpenSpec）的 schema 設計後，發現 speclink 的 Rust 重寫版在三處實質分歧：內建 schema 的存放方式（手寫 `spec_driven()` + fork.schema.yaml 雙份定義）、驗證強度（缺重複 id／懸空 requires／名稱 regex 等檢查）、三個收下不做事的 CLI 旗標（`which --all`、`validate --verbose`、`init --default`）。使用者要求：存放方式與驗證強度對齊 OpenSpec、補齊三旗標；spec-driven 的內容（instruction／template 加料）維持 speclink 現行作法；修正 `schema.rs:83` 與 `fork.schema.yaml:3` 兩份 description 對不起來的問題。後續擴充：desktop 專案設定要能檢視預設 schema、提供替換預設 schema 的作法，並納入 remote 模式的考量。

模式：assumptions（前置對話已掃過 schema.rs、toolchain.rs、instructions.rs、newcmd.rs、config.rs 五個源檔，足以先列假設）。

相關碼位：crates/speclink-core/src/schema.rs（引擎正典）、crates/speclink-core/assets/schema/spec-driven/（模板與 fork dump）、crates/speclink-cli/src/verbs/toolchain.rs（CLI 四指令）、crates/speclink-core/src/instructions.rs（payload 的 frozen output shape）、crates/speclink-core/src/config.rs（config.yaml 的 schema 欄位）。無進行中的相關 change；無既有討論。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-19)

**Focus**: 引擎面三項同步（存放方式、驗證強度、三旗標）的落點與邊界
**Position**: 五條假設全數獲使用者確認：
- 「同步存放方式」= 單一正典 + 同一條解析路徑，非真搬磁碟：`fork.schema.yaml` 升格為內建 schema 唯一定義，啟動時走與自訂 schema 相同的 serde 解析路徑；刪除手寫 `spec_driven()`（schema.rs:70）；template 仍 `include_str!` 內嵌查表。`schema.rs:410` 的雙份同步測試作廢。
- 驗證強度全跟 OpenSpec，含 breaking 部分：純加強四項（重複 artifact id、懸空 requires、循環相依印完整環路徑、schema 名稱 regex `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`）＋必填化三項（version 正整數、artifact description、artifact template——移除 `<id>.md` 預設容錯）。驗證時機落在 load（resolve_with 既有的 Some(Err) 口）。
- 三旗標補實作：`which --all` 列全部 schema 與來源；`validate --verbose` 印逐步驗證並補 template 檔存在檢查；`init --default` 寫 `schema: <name>` 進 openspec/config.yaml（config.rs:319 欄位與保序更新機制已在）。OpenSpec 的互動 prompt／`--no-default` 不搬。
- description 修正以 fork.schema.yaml 為準（"proposal → specs → tasks (design optional)"，符合實際 requires 圖）；採單一正典後此 bug 結構性消失；fork.schema.yaml 不動 → 不 bump MARKER_VERSION。
- frozen output shape 不動：instructions payload 的 template 只查內建（instructions.rs:181-187）、自訂 schema 列表不顯示 description（schema.rs:172）——歸入「spec-driven 內容維持現行」的同一裁定。
**Open**: desktop 專案設定檢視預設 schema 的讀取路徑；「替換預設 schema」的具體語意（設為專案預設 vs fork 出去改）；remote 模式下 schema 解析的層級歸屬（user 層在 server 側是否成立）

### Round 2 — assumptions (2026-08-19)

**Focus**: desktop 檢視／替換預設 schema 的落點，與 remote 模式的範圍
**Position**: 三條假設全數獲使用者確認，佐以三個掃描事實：
- 事實：store 層（speclink-store／store-fs／fs）對 `openspec/schemas/` 零提及——remote 下專案層自訂 schema 在 server 側今天就不存在。
- 事實：remote 設定快照有既有怪癖——settings.rs:110 傳 workspace: None 後仍拿 client 本機 user 目錄解析 server 專案的 schema 名稱。
- 事實：desktop core 直接連結 speclink-core（settings.rs:149）——單一正典後內建 schema 定義就在 desktop 手上。
- 「替換預設 schema」語意 = 切換（寫 config.yaml `schema:` 欄位，走既有 dual verb 寫入路徑）＋客製（`schema fork` 到專案後自行編輯）兩動作；不含編輯內建本身。
- 檢視面：desktop core 新增 schema 快照組裝（列表：名稱／來源／artifact 圖；詳情：description／instruction／template 全文唯讀），local 三層全看、remote 由 desktop 本地解析內建——零 server 改動。顯示位置為 ProjectSettingsView 的 workflow 區塊旁新節。
- remote 本次限縮「內建 schema only」：下拉只列內建；config 的 schema 名稱非內建時顯示「遠端自訂 schema 尚不支援」，並停止本機 user 目錄誤解析（順修怪癖）。
- 介面深度檢查通過：seam 在 desktop core 的快照組裝（含來源標註與 parse-error 語意，非純轉發）；單層 adapter；刪除測試有意義。
**Ruled out**: desktop 內建 schema 編輯器（YAML 編輯 UI，成本高一個量級）；為檢視加 server route＋protocol DTO（desktop 自己就有資料）；本次把 openspec/schemas/ 納入 store 文件模型（DocumentId／conformance gate／三驅動／import／backup 全要動，屬 store 契約變更）
**Open**: 無——三輪決策樹全數收斂，進入結論

## Conclusion

**Decision**: 分三塊落地，建議拆兩個 change：
- 引擎面（change 1）：`fork.schema.yaml` 升格為內建 schema 唯一定義，啟動時走與自訂 schema 同一條 serde 解析路徑，刪除手寫 `spec_driven()`；驗證強度全跟 OpenSpec——純加強四項（重複 id、懸空 requires、循環環路徑、名稱 regex）＋必填化三項（version 正整數、description、template，移除 `<id>.md` 預設），驗證落在 load；補實作 `which --all`、`validate --verbose`（含 template 檔存在檢查）、`init --default`（寫 config.yaml `schema:` 欄位）；兩份 description 對不齊隨單一正典結構性消失，以 fork.schema.yaml 字面為準。spec-driven 的 instruction／template 內容維持 speclink 現行；frozen output shape（instructions 只查內建 template、自訂列表無 description）不動；fork.schema.yaml 不動 → 不 bump MARKER_VERSION。
- desktop 面（change 2，依賴 change 1）：ProjectSettingsView 新增 schema 節——檢視（列表：名稱／來源／artifact 圖；詳情：description／instruction／template 全文唯讀）、切換（下拉寫 config.yaml `schema:`，走既有 dual verb 路徑）、fork 按鈕（local only）。desktop core 本地組裝快照，不加 server route。
- remote 面（併入 change 2）：限內建 schema；schema 名稱非內建時顯示「遠端自訂 schema 尚不支援」，並修掉拿 client 本機 user 目錄誤解析 server 專案 schema 名稱的既有怪癖（settings.rs:110→149）。
**Rationale**: cargo install 只裝單一執行檔、無 npm 式套件目錄——內嵌單一正典是「同一條讀取路徑」設計目標在 Rust 側的最小實現，且一舉消滅雙份定義的同步負擔；desktop core 連結 speclink-core，內建 schema 檢視零 server 改動；store 層不認識 schemas，remote 自訂支援屬 store 契約變更，拆開才不會讓 UI change 揹上三個 store 驅動的波及面。
**Rejected alternatives**: 內建 schema 真落磁碟（需安裝落地目錄＋過期同步機制，重蹈安裝版 CLI 過期同型坑）；保留寬鬆驗證預設（fork 出去的 schema 在兩邊工具合法性不一致）；搬 OpenSpec 互動 prompt／`--no-default`（speclink CLI 非互動，無物可抑制）；desktop 內建 schema 編輯器（成本高一個量級）；為檢視加 server route（拿 desktop 自己就有的資料）；本次擴充 store 納管 schemas（範圍升級為 store 契約變更）。
**Deferred**: `openspec/schemas/` 納入 store 文件模型（remote 自訂 schema 的前置，含 DocumentId 擴充、conformance gate、fs／sqlite／postgres 驅動、import／backup）；frozen output shape 對齊 OpenSpec（instructions 用自訂 template、列表顯示自訂 description）——維持現行，除非未來自訂 schema 使用者回報需求。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion schema-engine-openspec-parity（change 1 沿用 slug 名；change 2 建議名 desktop-schema-panel，於 change 1 落地後再轉出）
