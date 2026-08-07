## MODIFIED Requirements

### Requirement: 技能規定固定輸入來源與四條內容判準

渲染產出的 speclink-config 技能檔 SHALL 規定：掃描輸入限固定的結構性來源——workspace 清單與相依 manifest（含 Cargo workspace 成員與 workspace 相依、關鍵邊界相依、各 package 的相依清單）、README、docs 索引、既有 openspec/config.yaml、以及 speclink language show 的共用詞彙（若有）；SHALL NOT 全 repo 掃描原始碼。技能檔 SHALL 載明四條內容判準：(1) 已由政策開關或 schema 內建 instruction 自動注入的內容，以及品質站技能已承載的正典標準（如審查站的 smell baseline），context 與 rules 皆不得重述——引擎注入內容的判定 SHALL 以 speclink instructions <artifact> --json 取得的實際 payload 逐條反證，品質站正典的判定 SHALL 對照生成的品質站技能檔內容，皆不得憑印象；(2) 只對單一 artifact 咬合的內容歸 rules，不入 context；(3) 會過時的內容（版本號、計數、統計數字）不寫；(4) context 與 rules 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行皆核實——核實 SHALL 以靜態便宜手段進行（路徑查檔案系統、測試名以文字搜尋命中原始碼、npm script 查 package.json 宣告、CLI 子指令對照 --help 輸出），SHALL NOT 執行被引用的測試或建置指令；判準一的 speclink instructions <artifact> --json payload 探測不受此限。

技能檔 SHALL 載明刪除理由限定：一條既有 rule 只因不過四條判準、或使用者本人於政策詢問中明確撤回而被刪除，SHALL NOT 因「無法自固定輸入來源導出」而被刪除——使用者裁決後落地的 rule（如討論結論轉入者）因此不需任何標記即受保護，且落地裁決與撤回裁決同源。

技能檔 SHALL 載明 scope hint 的收窄語意：呼叫帶範圍提示時，判準一至三的全面重審收窄至範圍內的 artifacts；判準四的引用核實恆為全文件掃描；未帶範圍提示時維持全文件重審。

#### Scenario: 渲染產物含固定來源清單

- **WHEN** 檢視渲染產出的 speclink-config 技能檔
- **THEN** 技能檔載明固定輸入來源清單（manifest、README、docs 索引、既有 config.yaml、language show），並明示不做全 repo 原始碼掃描

#### Scenario: 渲染產物含四條判準與反證步驟

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準段落
- **THEN** 四條判準俱在，且判準一明定以 speclink instructions <artifact> --json 的 payload 逐條反證引擎注入內容、明定品質站技能承載的正典標準對照生成技能檔反證、判準四明定引用存在性以靜態手段核實

#### Scenario: 品質站正典不得重述進 rules

- **WHEN** 候選 rules 條目與品質站技能檔內嵌的正典標準同義（如要求避免特定 code smell）
- **THEN** 技能檔的判準指引該條目被淘汰，正典標準維持品質站技能檔單一落點

#### Scenario: 判準四核實不執行引用指令

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準四段落
- **THEN** 段落明定驗證引用以靜態手段（檔案系統、文字搜尋、package.json 宣告、--help 對照）進行、明文禁止執行被引用的測試或建置指令，並明示判準一的 payload 探測不在禁令範圍

#### Scenario: 使用者裁決型 rule 不因來源被刪

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於既有 rule 汰留的段落
- **THEN** 段落明定 rule 只因不過四條判準或使用者明確撤回而被刪，「無法自固定輸入來源導出」不構成刪除理由

#### Scenario: scope hint 收窄語意

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於範圍提示的段落
- **THEN** 段落明定範圍提示收窄判準一至三的重審至範圍內 artifacts、判準四恆為全文件掃描、未帶提示時全文件重審

## ADDED Requirements

### Requirement: 技能規定任務驗證測試範圍的第五問

渲染產出的 speclink-config 技能檔 SHALL 於政策詢問流程增列第五問：任務清單（tasks）的驗證步驟要包含全量測試，或只跑受影響面的測試——與政策四欄同性質，SHALL 逐項詢問使用者、不得自 repo 推斷；現行文件已有測試範圍相關 rule 時，提問 SHALL 帶出現值供確認。使用者答「只跑受影響面」時，技能檔 SHALL 指引自已讀取的 dependency manifests 組出該專案客製的對應規則（按專案的組件型態對應其測試指令）寫入 rules 的 tasks 段，並沿既有 dry-run 核准流程落地；使用者答「全量」時 SHALL NOT 寫入任何測試範圍規則——現行文件已有測試範圍 rule 時，該答案即為使用者對其之明確撤回，技能檔 SHALL 指引沿同一 dry-run 核准流程移除之，無既有 rule 時現行文件維持原樣。

#### Scenario: 渲染產物含第五問

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）的政策詢問段落
- **THEN** 段落載明任務驗證測試範圍的第五問、與四欄同樣不得推斷、以及現行文件已有測試範圍 rule 時帶現值確認的指引

#### Scenario: 答受影響面時組出客製規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於第五問後續處理的段落
- **THEN** 段落明定自已讀取的 dependency manifests 組出專案客製的測試對應規則、寫入 rules 的 tasks 段、經 dry-run 核准後落地

#### Scenario: 答全量時不寫規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於第五問後續處理的段落
- **THEN** 段落明定使用者選擇全量時不寫入任何測試範圍規則；現行文件已有測試範圍 rule 時視為明確撤回、經 dry-run 核准移除該 rule，無既有 rule 時現行文件維持原樣
