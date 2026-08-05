## MODIFIED Requirements

### Requirement: 技能規定固定輸入來源與四條內容判準
<!-- BEFORE: 判準一的反證集合僅涵蓋引擎注入內容（instructions payload），品質站技能承載的正典標準不在反證範圍 -->

渲染產出的 speclink-config 技能檔 SHALL 規定：掃描輸入限固定的結構性來源——workspace 清單與相依 manifest（含 Cargo workspace 成員與 workspace 相依、關鍵邊界相依、各 package 的相依清單）、README、docs 索引、既有 openspec/config.yaml、以及 speclink language show 的共用詞彙（若有）；SHALL NOT 全 repo 掃描原始碼。技能檔 SHALL 載明四條內容判準：(1) 已由政策開關或 schema 內建 instruction 自動注入的內容，以及品質站技能已承載的正典標準（如審查站的 smell baseline），context 與 rules 皆不得重述——引擎注入內容的判定 SHALL 以 speclink instructions <artifact> --json 取得的實際 payload 逐條反證，品質站正典的判定 SHALL 對照生成的品質站技能檔內容，皆不得憑印象；(2) 只對單一 artifact 咬合的內容歸 rules，不入 context；(3) 會過時的內容（版本號、計數、統計數字）不寫；(4) context 與 rules 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行皆核實。

#### Scenario: 渲染產物含固定來源清單

- **WHEN** 檢視渲染產出的 speclink-config 技能檔
- **THEN** 技能檔載明固定輸入來源清單（manifest、README、docs 索引、既有 config.yaml、language show），並明示不做全 repo 原始碼掃描

#### Scenario: 渲染產物含四條判準與反證步驟

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準段落
- **THEN** 四條判準俱在，且判準一明定以 speclink instructions <artifact> --json 的 payload 逐條反證引擎注入內容、明定品質站技能承載的正典標準對照生成技能檔反證、判準四明定引用存在性核實

#### Scenario: 品質站正典不得重述進 rules

- **WHEN** 候選 rules 條目與品質站技能檔內嵌的正典標準同義（如要求避免特定 code smell）
- **THEN** 技能檔的判準指引該條目被淘汰，正典標準維持品質站技能檔單一落點
