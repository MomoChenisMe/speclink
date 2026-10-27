---
topic: config context 與 rules GUI 編輯
slug: config-context-與-rules-gui-編輯
status: promoted
promoted_to: desktop-config-rules-context
created: 2026-07-07
---

# Discussion: config context 與 rules GUI 編輯

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：討論「專案選擇對齊-spectra」結論把 config.yaml 的 context/rules GUI 編輯列為延後項目（主刀 desktop-config-multiproject 的 Non-Goal 明文排除）；使用者裁定三個延後項中此項先開討論——寫入面有「解析失敗靜默退回預設」既知地雷、且屬新的 GUI→config.yaml 寫入路徑。

模式：assumptions——相關程式碼充分：crates/speclink-core/src/config.rs（WorkflowConfig 僅 Deserialize、from_text 解析失敗 unwrap_or_default 靜默退預設、rules 為 BTreeMap<String, Vec<String>>）、instructions.rs:175（rules_for 唯一消費點、以 schema artifact id 查詢）、主刀 design.md 的 D4（speclink-core text→text 政策純函式、serde_yaml::Mapping 讀-改-寫、未觸及鍵原樣保留、註解遺失為已明文接受取捨）與 D5（寫入前後雙重解析驗證、載入時解析失敗警告＋停用表單）。

相關 changes/specs：desktop-config-multiproject（in-progress 0/29——Impact 列 settings.rs、SettingsView.tsx、D4 純函式為新檔，皆為本題依賴的基建）；desktop-config spec（將由主刀新增）；workflow-config spec（解析與驗證規則不變，本題僅新增圖形化寫入者）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-07)

**Focus**: rules/context GUI 編輯的架構落點與範圍——五項假設的確認
**Position**: 五項假設全數獲使用者確認：①非新架構縫——擴充主刀 D4 的政策純函式（同模組）、沿用 D5 雙重驗證與「載入解析失敗→警告＋停用表單」，不另立模組、不另設 Tauri command 類別（介面深度檢查：另立模組會不過刪除測試——純轉發、無隱藏行為）；②註解不保護——含使用者手寫註解（明確確認），沿用 D4 已接受的「寫入即失去註解」取捨，不引入保留註解的 YAML 庫；③rules 表單為 schema 驅動固定鍵（合法鍵＝活躍 schema 的 artifact ids），每 artifact 一節的條目清單，空清單即移除鍵（維持「未設定＝預設」語意）；serde_yaml 序列化自動為反引號等 YAML 保留字元加引號——GUI 寫入反而消除手改反引號地雷（CLAUDE.md 既知風險）；④context 為設定頁多行文字區、與 rules 同頁分節，空值移除鍵，不做 Spectra 式專案首頁入口（前討論已否決儀表板首頁）；⑤時序：等主刀 desktop-config-multiproject 的設定頁基建（D4 函式、settings.rs、SettingsView.tsx）落地後才 propose，本討論先收斂需求做種子。
**Ruled out**: 保留註解的 YAML 庫（使用者確認手寫註解不需保護；D4 亦已否決「新外部依賴只為註解」）；自由鍵值 rules 編輯器（打錯鍵靜默永不生效，重蹈靜默失效風險）；Spectra 式專案首頁編輯入口（看板即儀表板，前討論已裁決）。
**Open**: GUI 使用者可見文案的正典詞——context/rules 是工程詞，LANGUAGE.md 原則禁止出現在使用者文案，且「背景」已被討論抽屜的 Context 分頁佔用，需另定詞（詞彙飄移候選）；rules 條目是否開放排序（Vec 順序即注入順序）。

### Round 2 — assumptions (2026-07-07)

**Focus**: 文案詞彙、rules 排序與寫入行為模型定稿
**Position**: 使用者全數核可：①GUI 文案定為「專案說明」（context）與「產出規則」（rules）——對齊 Spectra 用詞、避開已被討論抽屜 Context 分頁佔用的「背景」；屬詞彙飄移，記入 openspec/LANGUAGE.md。②rules 條目順序有語意（Vec 順序＝指令注入順序），清單編輯器支援排序（拖曳或上下移按鈕實作時再定；dnd-kit 需 activationConstraint 為既知備忘）。③寫入行為模型以具體前後例確認：未觸及鍵原樣保留、新條目自動引號化（手改反引號炸檔地雷在 GUI 路徑上不存在）、清空清單或清空文字即移除鍵、檔內註解遺失（已接受取捨）。
**Open**: 無——進 conclude。

## Conclusion

**Decision**: desktop-config-multiproject 落地後另開一刀，為桌面設定頁新增「專案說明」（config.yaml 的 context）與「產出規則」（rules）的 GUI 編輯。架構：擴充主刀 D4 的 speclink-core text→text 政策純函式（同模組），沿用 D5 寫入前後雙重驗證與「載入解析失敗→警告＋停用表單」；不另立模組、不另設 Tauri command 類別。形態：「產出規則」為 schema 驅動固定鍵（合法鍵＝活躍 schema 的 artifact ids）、每 artifact 一節的可排序條目清單（順序＝指令注入順序），空清單移除鍵；「專案說明」為多行文字區，空值移除鍵；serde_yaml 自動引號化消除手改反引號地雷；註解（含手寫）不保護——寫入即遺失，沿用 D4 取捨。
**Rationale**: 主刀 D4/D5 已解決寫入安全與未觸及鍵保留的核心難題，本刀縮為表面擴充；設計重心是把「政策靜默失效」風險做成結構上不可能——固定鍵防打錯鍵、自動引號防解析炸檔、雙重驗證防壞檔落地、解析失敗停用表單防覆蓋手寫壞檔。
**Rejected alternatives**: 保留註解的 YAML 庫（使用者確認手寫註解不需保護；D4 亦已否決「新外部依賴只為註解」）；自由鍵值 rules 編輯器（打錯鍵靜默永不生效）；Spectra 式專案首頁編輯入口（看板即儀表板，討論「專案選擇對齊-spectra」已裁決）；raw YAML 文字編輯模式（語法負擔留給使用者，GUI 的意義即結構化）。
**Deferred**: .speclink.yaml 自訂工具描述子與 remote 段的 GUI 編輯（維持主刀 Non-Goal）；遠端 store 情境的設定寫入（桌面接遠端文件時需 store 層寫入介面——D4 純函式已為此解耦，web-server-postgres 落地後另議）；規則語意 lint 與注入預覽（不納入）。
**Capture to**: proposal（經 --from-discussion 種子）；openspec/LANGUAGE.md（新詞「專案說明」「產出規則」——詞彙飄移）；spec delta 落點為 desktop-config capability（修訂主刀「rules/context 僅原樣保留」敘述＋新增編輯需求）。
**Next**: desktop-config-multiproject 完成後 /speclink-propose --from-discussion config-context-與-rules-gui-編輯
