## ADDED Requirements

### Requirement: 看板卡片統一解剖學

看板全尺寸卡（變更卡與討論卡）SHALL 採統一三列骨架：識別列（標題＋複製鈕＋右端 meta icons）、描述列（一行截斷，無內容時缺席）、meta 列。標題 SHALL 以等寬字型呈現（變更名稱與討論 slug 同為可複製把手）。複製鈕 SHALL 行內尾隨於標題最後一個字元後（標題折行時位於末行文字尾），meta icons 維持靠右；SHALL NOT 將複製鈕推至列右緣。標題 SHALL NOT 截斷——完整顯示並自然折行。變更卡描述列 SHALL 顯示 proposal Why 首句（一行截斷）；proposal 缺席、Why 區段缺席或為空時描述列 SHALL 缺席。描述資料 SHALL 由變更清單 payload 一次帶出，SHALL NOT 逐卡讀取 proposal 全文；該欄位屬呈現層輔助欄位，不屬 CLI --json 對齊範圍。建立者 SHALL 以頭像圓點呈現且 hover 顯示全名，SHALL NOT 於卡面直出全名文字；createdBy 缺席時圓點缺席。狀態 chip SHALL 僅在所在位置無法表達狀態時出現：討論卡（討論欄一欄兩態）帶狀態 chip，變更卡（所在欄即階段）SHALL NOT 帶狀態 chip。

#### Scenario: 變更卡三列骨架

- **WHEN** 看板載入一個 proposal Why 首句非空、createdBy 存在、任務 5/21 的變更
- **THEN** 變更卡識別列以等寬字型顯示名稱且複製鈕緊跟名稱文字後（hover 顯現、點擊寫入名稱至剪貼簿且不開詳情抽屜）、右端呈建立者圓點；描述列一行截斷顯示 Why 首句；meta 列顯示進度條與 5/21；卡上無狀態 chip

#### Scenario: 變更卡無 Why 內容時描述列缺席

- **WHEN** 某變更無 proposal.md（或 Why 區段為空）
- **THEN** 該變更卡不顯示描述列，識別列與 meta 列照常呈現，看板不因該筆缺件而報錯

#### Scenario: 長標題折行時複製鈕仍緊跟末字元

- **WHEN** 變更名稱長於卡片可用寬度
- **THEN** 標題折行完整顯示不截斷，複製鈕緊跟末行最後一個字元後且維持可點，右端 meta icons 不被擠出卡外

## MODIFIED Requirements

### Requirement: 討論於看板第 0 欄兩級呈現

<!-- BEFORE: 全卡建立者全名直出、輪數於卡身、複製鈕推至列右緣（基底為 desktop-ux-polish 落地後版本；封存順序本變更在後） -->

看板 SHALL 於最左新增「討論」欄，欄內分上下兩區同屏呈現：上區為討論中（active）全卡清單，下區為欄底的「已轉出」常駐收合列；SHALL NOT 以互斥檢視切換兩者。

討論中區 SHALL 顯示 status 為 open 或 concluded 的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題（等寬字型）、topic 為卡身描述；複製 slug 鈕 SHALL 行內尾隨於標題最後一個字元後（版面規則見「看板卡片統一解剖學」）；建立者（createdBy，缺席時省略）SHALL 以頭像圓點呈現且 hover 顯示全名；輪數與建立時間 SHALL 並排於卡底 meta 列，狀態 chip 保留；open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）。

當存在至少一筆 promoted 討論時，欄底 SHALL 呈現「已轉出 N」收合列（N 為 promoted 計數，預設收合）；點按 SHALL 就地展開 promoted 細列清單，再點按即收合；展開狀態 SHALL NOT 跨啟動持久化。無任何 promoted 討論時收合列 SHALL 缺席。

promoted 細列 SHALL 以討論 slug 為首行（等寬字型）且帶複製 slug 鈕、topic 為次行描述，其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

slug 為題與複製鈕（討論全卡標題、promoted 細列首行）屬 openspec/LANGUAGE.md 明載的受控例外。討論欄的計數徽章 SHALL 顯示 active（open 與 concluded）數。當討論欄無任何 active 討論但存在 promoted 討論時，SHALL NOT 顯示「尚無討論」空狀態（欄底收合列已傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by 與 created）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄顯示兩張全卡，各以 slug 為標題（等寬字型）、topic 為描述，複製 slug 鈕行內尾隨於標題末字元後，建立者呈頭像圓點且 hover 顯示全名、卡面無全名直出文字，卡底 meta 列並排「3 輪」與建立時間；open 卡無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，欄底無「已轉出」收合列

#### Scenario: 複製討論 slug

- **WHEN** 點討論全卡或已轉出細列的複製鈕
- **THEN** 該討論的 slug 寫入剪貼簿並短暫顯示已複製回饋，且不開啟討論抽屜

#### Scenario: 欄底收合列就地展開與收合

- **WHEN** 討論欄存在一筆 active 討論與一筆 promoted 討論
- **THEN** 上區顯示 active 全卡、欄底呈現「已轉出 1」收合列（預設收合）；點按收合列就地展開 promoted 細列（active 全卡維持可見），再點按即收合；欄計數徽章全程顯示 1（active 數）

#### Scenario: 無已轉出討論時收合列缺席

- **WHEN** 討論欄無任何 promoted 討論
- **THEN** 欄底不顯示「已轉出」收合列

#### Scenario: 僅有已轉出討論時討論中區不顯空狀態

- **WHEN** 討論欄無任何 active（open 或 concluded）討論、但存在至少一筆 promoted 討論
- **THEN** 討論中區不顯示「尚無討論」文案，欄底收合列傳達存在已轉出討論

#### Scenario: 已轉出細列的 slug 首行與子變更樹配色

- **WHEN** 展開欄底收合列，一筆 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
- **THEN** 該細列首行為 slug（等寬字型）帶複製鈕、次行為 topic，其下兩列樹狀子項——前者帶 ├ 前綴與「提案中」chip（呈提案中欄的 teal 濃度）、後者帶 └ 前綴與「已封存」chip（中性色）

##### Example: chip 階段派生與配色矩陣

| promoted_to 子變更的所在 | 階段標示 | chip 配色 |
| ------------------------ | -------- | --------- |
| active 清單，無 started、0/24 | 提案中 | 提案中欄的 teal 濃度 |
| active 清單，有 started、13/24 | 進行中 | 進行中欄的 teal 濃度 |
| active 清單，24/24 | 已就緒 | 已就緒欄的 teal |
| 封存清單（dated name 尾碼命中） | 已封存 | 中性色 |
| 兩清單皆無 | 已刪除（討論維持已轉出） | destructive 加刪除線 |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作
