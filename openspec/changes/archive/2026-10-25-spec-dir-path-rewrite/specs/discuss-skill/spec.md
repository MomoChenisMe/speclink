## MODIFIED Requirements

### Requirement: 事實與決策分診及逐節點查證

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能檔 SHALL 規定每個決策節點解決前先分診：環境（程式碼、檔案系統、工具）查得到的事實 SHALL 由代理人沿樹逐節點自行查證，SHALL NOT 拿去問使用者、SHALL NOT 憑印象作答；僅真正的決策（使用者裁定事項）交由使用者。開場偵察 SHALL 為漏斗式：先跑 speclink list --specs --json（候選 ≤5，候選順序為名稱命中關鍵字數多者優先、同數依指令列出順序；依該順序讀 Purpose ≤3、主題直接動到的 capability 才讀全文、零命中靜默略過）；命中 capability 名後 SHALL 以開場已執行的 speclink list --json 各變更項的 deltaCapabilities 欄位比對（SHALL NOT 以 status 過濾：未封存的變更不論 status，其 delta 皆尚未併入正式規格；SHALL NOT 為比對逐一執行 speclink show），deltaCapabilities 含該 capability 者列為「進行中 delta 命中」並記下該變更名；再以 speclink artifact cat specs/<capability> --change <name> 只讀該 delta 的 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 區段標記、至多 3 份（依候選順序取前 3 個命中，同一 capability 有多個變更時依 speclink list --json 的回傳順序）、不讀全文；該變更的清單項帶 worktree 物件時 SHALL 於其 worktree.path 下執行 artifact cat（清單的 deltaCapabilities 取自該 worktree 副本，讀到的標題須來自同一副本）；正式規格零命中、沒有進行中變更或沒有變更的 deltaCapabilities 含命中的 capability 時 SHALL 靜默跳過 delta 比對、不提及進行中 delta；技能檔 SHALL NOT 指示直接開啟規格目錄的檔案路徑（remote 模式走同一組動詞，不另闢路徑）。之後以命中的 capability 名與正典詞彙轉譯搜尋詞後再掃原始碼（至多讀 5 檔）；主題已含具體檔名或符號時，程式碼軌 SHALL 直接開跑不等正式規格。偵察三段順序 SHALL 維持「正式規格 → 舊討論查核 → 程式碼」，進行中 delta 規格 SHALL 屬規格段的一部分、SHALL NOT 另立一段。偵察用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 沿樹逐節點進行（確定會走到的分支才深讀）。

#### Scenario: 渲染產物含事實決策分診規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔
- **THEN** 技能檔 SHALL 規定：查得到的事實由代理人自行查證後附為 Evidence，不得提問使用者、不得憑印象作答；決策類問題才交由使用者裁定

#### Scenario: 開場 scout 維持淺掃

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察段落
- **THEN** 偵察 SHALL 維持時間盒（正式規格讀取至多 3 份 Purpose、進行中 delta 規格至多 3 份且只讀 Requirement 標題與區段標記、原始碼至多 5 檔），用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 規定於決策樹逐節點進行

#### Scenario: 正式規格先行的漏斗偵察

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察順序規定
- **THEN** 技能檔 SHALL 規定先查正式規格（speclink list --specs --json）、以命中詞彙轉譯搜尋詞後再掃原始碼；主題含具體檔名或符號時程式碼軌直接開跑；正式規格零命中時靜默退回關鍵字掃描

#### Scenario: 規格段納入進行中 delta 規格

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 Canon pass 段落
- **THEN** 技能檔 SHALL 規定命中 capability 名後以 speclink list --json 各變更項的 deltaCapabilities 欄位比對（不以 status 過濾、不逐一執行 speclink show），含該 capability 者列為進行中 delta 命中並記變更名，以 speclink artifact cat specs/<capability> --change <name> 只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 標記、至多 3 份且依候選順序取前 3 個命中，清單項帶 worktree 物件時於其 worktree.path 下執行 artifact cat；正式規格零命中、沒有進行中變更或沒有變更的 deltaCapabilities 含命中的 capability 時靜默跳過；技能檔 SHALL NOT 含直接開啟 speclink/changes/ 路徑的讀檔指示；三段順序描述仍為「正式規格 → 舊討論查核 → 程式碼」且 SHALL NOT 出現獨立的「進行中變更」偵察段

##### Example: 命中 client-protocol 且有進行中 delta

- **GIVEN** 正式規格 client-protocol 命中，且 speclink list --json 中 add-change-plan-remote 的 deltaCapabilities 含 client-protocol
- **WHEN** 代理人依技能檔執行 Canon pass
- **THEN** 代理人記下「進行中 delta 命中：add-change-plan-remote: client-protocol」與該 delta 的 Requirement 標題清單，不讀該 delta 全文

### Requirement: 中途轉出教學

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能檔 SHALL 規定：多需求討論中單項談定、使用者要先立案時，代理人 SHALL 教執行 speclink discuss promote 即刻轉出（引擎於無結論時以 topic 預填提案），討論 SHALL 繼續加輪談剩餘項，SHALL NOT 要求先寫結論。最終 conclude SHALL 照常執行——引擎保留 promoted 狀態、寫入結論、並將已轉出變更標為待重新反映；技能檔 SHALL 註明該標記與最終結論無關時僅需一次確認。技能檔 SHALL 另規定分期轉出：結論規劃「之後回同一份記錄再轉出一個或多個變更」（例：多刀依序立案）時，conclude SHALL 帶 --hold 一次；技能檔 SHALL 說明 hold 不因不帶 --last 的轉出而清除，只由不帶 --hold 的 conclude、speclink discuss archive，或帶 --last 的轉出解除；最後一刀 SHALL 由 /speclink-propose 於 speclink new change --from-discussion 帶 --last 轉出（判定規則在 propose 技能），其後最後一個轉出變更封存時記錄自動隨行封存，SHALL NOT 教執行 speclink discuss archive 作為常態收尾；技能檔 SHALL 註明忘了帶 --last 時記錄留在途、看板標「已轉出・保留中」，執行 speclink discuss archive <slug> 一次即可收尾；未帶 --hold 的記錄會在最後一個轉出變更封存時隨行封存，之後的刀 SHALL 走新討論。技能檔 SHALL NOT 含「旗標由下一次轉出清除」、「discard 後要重跑 conclude --hold」或「最後一刀封存後手動 discuss archive 收尾」作為常態步驟的規定。技能檔 SHALL 另註明救援路徑：討論被誤封存時（含在不是最後一刀的變更上帶了 --last），把記錄檔自 speclink/discussions/archive/ 搬回 speclink/discussions/ 即可續用（引擎對已封存討論的轉出錯誤訊息即指向此路）。conclude 指令範例 SHALL 標示 --hold 的用途。

#### Scenario: 單項談定即中途轉出

- **WHEN** 討論尚未結論，使用者要求先把已談定的需求轉為變更
- **THEN** 技能檔 SHALL 規定直接執行 promote 並繼續討論剩餘項，SHALL NOT 要求先 conclude 整份討論

#### Scenario: 中途轉出後補結論

- **WHEN** 中途轉出過的討論最終執行 conclude
- **THEN** 技能檔 SHALL 說明：狀態保持已轉出、結論照常寫入、先轉出的變更被標為待重新反映，與結論無關時一次確認即可

#### Scenario: 分期轉出帶 --hold

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的中途轉出段與 conclude 指令範例
- **THEN** 內容 SHALL 規定結論規劃之後回同一記錄再轉出時 conclude 帶 --hold 一次、hold 只由不帶 --hold 的 conclude、discuss archive 或帶 --last 的轉出解除、最後一刀由 propose 帶 --last 轉出且最後一個封存自動收尾、忘帶 --last 時執行 discuss archive 一次收尾、未帶 --hold 時後續刀走新討論；SHALL NOT 含「旗標由下一次轉出清除」、「discard 後重跑 conclude --hold」與「最後一刀封存後手動 discuss archive 收尾」的常態敘述；SHALL 含誤封存的搬回救援路徑；conclude 範例 SHALL 標示 --hold 的用途；claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映

### Requirement: 討論記錄的撰寫語言依 locale

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

渲染產出的 speclink-discuss 技能檔 SHALL 在建立討論記錄的步驟規定記錄的撰寫語言：以 `speclink workflow-config show --json` 讀取 `locale` 欄位，新建記錄的 Context、各輪 Focus／Position／Ruled out／Open 的內容與結論散文 SHALL 以該語言撰寫——`tw` 為繁體中文、`ja` 為日文、`en` 或 null 為英文；指令失敗時 SHALL 以英文撰寫。topic SHALL 維持使用者輸入的原文（沿用「討論技能指示要求英文 slug」的 topic 規定）。欄位標記（`**Focus**:`、`**Position**:`、`**Ruled out**:`、`**Open**:` 與結論的 `**Decision**:`、`**Rationale**:`、`**Rejected alternatives**:`、`**Deferred**:`、`**Capture to**:`、`**Next**:`）、固定標記（`Prior discussions:`、`Source doc:`、單字 `none`、``**cut N `change-name`**:`` 刀頭與 **Capture to** 的值）、slug、指令行與程式碼 SHALL 維持英文。恢復已存在的記錄時 SHALL 沿用該記錄原本的語言，SHALL NOT 依 `locale` 改寫既有內容或在新增的輪次改用另一種語言；恢復流程（Resume ritual）SHALL 以一句話指回此規則。對話本身的語言依共用對話語言段落，SHALL NOT 由 `locale` 決定。

此為刻意變更：變更前技能未規定記錄語言，記錄通常跟著對話語言；變更後新建記錄依 `locale`，未設定 `locale` 時為英文。

#### Scenario: 渲染產物含記錄語言規則

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** speclink-discuss 技能檔在建立記錄的步驟含讀取 `speclink workflow-config show --json` 的 `locale` 欄位、依該語言撰寫記錄散文、欄位標記與固定標記（`Prior discussions:`、`Source doc:`、`none`、cut 刀頭、Capture to 的值）及 slug 維持英文，以及恢復既有記錄沿用原語言的規定；Resume ritual 與結尾自我檢查清單各以一句話指回此規則

#### Scenario: locale 與對話語言不同時記錄依 locale

- **WHEN** 專案 speclink/config.yaml 設 `locale: tw`，使用者以英文開啟一個新討論
- **THEN** Agent 以英文跟使用者對話；新建記錄的 Context、各輪內容與結論為繁體中文，`**Focus**:` 等欄位標記與 slug 為英文，topic 為使用者輸入的英文原文

#### Scenario: 恢復既有記錄沿用原語言

- **WHEN** 專案未設定 `locale`，使用者恢復一份以繁體中文撰寫、status 為 open 的既有記錄並新增一輪
- **THEN** 新增輪次的內容為繁體中文，既有輪次內容不變

##### Example: 新建記錄的散文語言

| locale | `workflow-config show --json` 結果 | 新建記錄的散文 | 欄位標記與 slug |
| ------ | ---------------------------------- | -------------- | --------------- |
| tw | `"locale": "tw"` | 繁體中文 | 英文 |
| ja | `"locale": "ja"` | 日文 | 英文 |
| en | `"locale": "en"` | 英文 | 英文 |
| 未設定 | `"locale": null` | 英文 | 英文 |
| 任一 | 指令失敗 | 英文 | 英文 |
