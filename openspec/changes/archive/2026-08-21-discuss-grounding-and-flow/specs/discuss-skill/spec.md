## MODIFIED Requirements

### Requirement: interview 模式以決策樹遍歷提問

<!-- BEFORE: interview 是與 assumptions 並立的全域模式，由「相關原始檔 3 個以上」的檔案數門檻分流 -->

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 將 interview 重新定位為兩種提問位置：需求磨利（grill）階段與 assumptions 內的逐節點退路，SHALL NOT 保留以檔案數門檻分流的全域雙模式。分流軸 SHALL 為需求清晰度：需求鈍（無可驗證目標、無門檻、improve 類措辭）時先走 grill 階段一次一題磨需求，需求已利時 grill SHALL 塌縮為零題並直接進 assumptions（唯一預設姿態）。提問紀律 SHALL 維持決策樹遍歷：開場先攤開決策空間（根節點為「這題到底在決定什麼」，展開子決策與依賴邊），提問 SHALL 依依賴順序進行、一次一題，上游決策先解。停止條件 SHALL 維持使用者主導：one nudge maximum 與結論的 Deferred 欄位續留，技能檔 SHALL NOT 要求「所有分支解完才可收斂」。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含決策樹提問紀律

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 將提問策略描述為決策樹遍歷（grill 階段與節點退路皆適用）：開場攤開決策空間、依依賴順序一次一題、上游決策先解

#### Scenario: 停止條件維持使用者主導

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的收斂（Convergence）相關段落
- **THEN** one nudge maximum 與 Deferred 機制 SHALL 保留，技能檔 SHALL NOT 含「未走完所有分支即不得收斂」的規定，且 SHALL 規定使用者喊停時未走分支記入結論的 Deferred

#### Scenario: 需求清晰度分流取代檔案數門檻

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的分流規則
- **THEN** 技能檔 SHALL 以需求清晰度分流（需求鈍先 grill 磨需求、需求利直接 assumptions），SHALL NOT 含「相關檔案 3 個以上走 assumptions、不足走 interview」的檔案數門檻，且 SHALL 規定需求已利時 grill 塌縮為零題

### Requirement: interview 每題附建議答案與 Evidence

<!-- BEFORE: 單一 interview 模式的每題硬規則：附建議答案＋Evidence（檔案路徑或查證結果） -->

技能檔 SHALL 對兩類提問分別規定證據義務，SHALL NOT 允許任一類空白提問：grill 意圖題（問目標、範圍、門檻、成功判準）SHALL 附框題脈絡——現況或正典證據——或最佳猜測建議；節點退路題（assumptions 中證據撐不起立場的節點）SHALL 附代理人的建議答案並附 Evidence（檔案路徑或查證結果）。兩類皆 SHALL 讓使用者僅需同意或修正。

#### Scenario: 渲染產物將 Evidence 列為 interview 硬規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的提問規則
- **THEN** 技能檔 SHALL 規定 grill 題附框題脈絡或最佳猜測建議、節點退路題附建議答案與 Evidence，兩類皆不得空白提問，使用者僅需同意或修正

### Requirement: 事實與決策分診及逐節點查證

<!-- BEFORE: 開場 scout 只掃原始碼（明文排除 docs 與規格）、用途僅為模式選擇 -->

技能檔 SHALL 規定每個決策節點解決前先分診：環境（程式碼、檔案系統、工具）查得到的事實 SHALL 由代理人沿樹逐節點自行查證，SHALL NOT 拿去問使用者、SHALL NOT 憑印象作答；僅真正的決策（使用者裁定事項）交由使用者。開場偵察 SHALL 為漏斗式：先跑 speclink list --specs --json（候選 ≤5、讀 Purpose ≤3、主題直接動到的 capability 才讀全文、零命中靜默略過），以命中的 capability 名與正典詞彙轉譯搜尋詞後再掃原始碼（至多讀 5 檔）；主題已含具體檔名或符號時，程式碼軌 SHALL 直接開跑不等正典。偵察用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 沿樹逐節點進行（確定會走到的分支才深讀）。

#### Scenario: 渲染產物含事實決策分診規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔
- **THEN** 技能檔 SHALL 規定：查得到的事實由代理人自行查證後附為 Evidence，不得提問使用者、不得憑印象作答；決策類問題才交由使用者裁定

#### Scenario: 開場 scout 維持淺掃

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察段落
- **THEN** 偵察 SHALL 維持時間盒（正典讀取至多 3 份 Purpose、原始碼至多 5 檔），用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 規定於決策樹逐節點進行

#### Scenario: 正典先行的漏斗偵察

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察順序規定
- **THEN** 技能檔 SHALL 規定先查正典（speclink list --specs --json）、以命中詞彙轉譯搜尋詞後再掃原始碼；主題含具體檔名或符號時程式碼軌直接開跑；正典零命中時靜默退回關鍵字掃描

## ADDED Requirements

### Requirement: 正典接地與三分對照

技能檔 SHALL 規定：偵察命中相關正典時，假設清單 SHALL 對使用者需求逐項做三分對照——正典已涵蓋（附 spec 證據）、與正典衝突（指出衝突內容並附證據）、正典沒講（新地盤，順帶檢查 capability 命名鄰近既有規格）。紀律 SHALL 明文為「使用者需求是目標，正典是證據、不是裁決」：偏離正典 SHALL 為允許的結論方向，但 SHALL 記入討論記錄成為有意識的決定。

#### Scenario: 需求與正典衝突時列為假設

- **WHEN** 偵察發現使用者需求與正典既有承諾衝突
- **THEN** 技能檔 SHALL 規定該衝突以假設形式呈現（附 spec 證據），由使用者裁定改正典或改需求，SHALL NOT 逕行擋下或否決討論方向

#### Scenario: 正典零命中時流程照舊

- **WHEN** 主題與任何正典 capability 無關（工具鏈、依賴選型類主題）
- **THEN** 技能檔 SHALL 規定不提及規格掃描，後續流程與純程式碼偵察相同

### Requirement: 多需求 backlog 與恢復摘要

技能檔 SHALL 規定：單一討論載有多個需求時，首輪 Open SHALL 攤開全部需求清單，之後每輪 Open SHALL 復述剩餘未談項；已定案項的去向由該輪 Position 首句承載。續用既有 open 討論時，代理人 SHALL 先呈現恢復摘要——逐輪 Focus 與 Position 首句、最後一輪的 Open 邊界——再接續討論。摘要 SHALL 自既有欄位機械推導，SHALL NOT 引入新記錄格式，既有討論記錄 SHALL NOT 需要遷移。

#### Scenario: 恢復時先呈現摘要

- **WHEN** 開場檢查發現同主題的 open 討論並續用其 slug
- **THEN** 技能檔 SHALL 規定先輸出逐輪「Focus→Position 首句定論」清單與最後一輪 Open 邊界，再接續討論

#### Scenario: 多需求首輪攤清單

- **WHEN** 討論主題含多個需求（一次 5-10 個）
- **THEN** 技能檔 SHALL 規定首輪 Open 列出全部需求清單，後續每輪 Open 復述剩餘項，已定案項去向由該輪 Position 首句承載

### Requirement: 記錄建檔以使用者首次回覆為觸發

技能檔 SHALL 規定：討論記錄的建立時機為第一個實質回合——由使用者的回覆（對假設清單的確認或修正、對提問的回答）使主題實際前進的那一刻；代理人自身的研究產出或首份假設清單 SHALL NOT 單獨構成建檔條件。使用者回覆前 SHALL NOT 在磁碟留下記錄檔，使誤觸發或一輪即答的主題零檔案離場。

#### Scenario: 研究再深也不先建檔

- **WHEN** 代理人完成偵察與線上查證並呈現首份假設清單，使用者尚未回覆
- **THEN** 技能檔 SHALL 規定此時不得執行 speclink discuss new，磁碟上不存在記錄檔

#### Scenario: 使用者首次回覆後建檔

- **WHEN** 使用者對假設清單給出確認或修正，或回答了第一個問題
- **THEN** 技能檔 SHALL 規定於記錄該首輪前執行 speclink discuss new 並補寫 Context，之後每輪照常 add-round

### Requirement: 中途轉出教學

技能檔 SHALL 規定：多需求討論中單項談定、使用者要先立案時，代理人 SHALL 教執行 speclink discuss promote 即刻轉出（引擎於無結論時以 topic 預填提案），討論 SHALL 繼續加輪談剩餘項，SHALL NOT 要求先寫結論。最終 conclude SHALL 照常執行——引擎保留 promoted 狀態、寫入結論、並將已轉出變更標為待重新反映；技能檔 SHALL 註明該標記與最終結論無關時僅需一次確認。

#### Scenario: 單項談定即中途轉出

- **WHEN** 討論尚未結論，使用者要求先把已談定的需求轉為變更
- **THEN** 技能檔 SHALL 規定直接執行 promote 並繼續討論剩餘項，SHALL NOT 要求先 conclude 整份討論

#### Scenario: 中途轉出後補結論

- **WHEN** 中途轉出過的討論最終執行 conclude
- **THEN** 技能檔 SHALL 說明：狀態保持已轉出、結論照常寫入、先轉出的變更被標為待重新反映，與結論無關時一次確認即可
