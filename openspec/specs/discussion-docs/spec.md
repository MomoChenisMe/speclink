# discussion-docs Specification

## Purpose

TBD - created by archiving change 'discuss-english-slug'. Update Purpose after archive.

## Requirements

### Requirement: 討論記錄以 --slug 覆寫檔名

speclink discuss new SHALL 接受選配旗標 --slug,用於覆寫討論記錄的檔名與 frontmatter slug 欄位;topic SHALL 維持使用者輸入原文不受影響。--slug 的值 SHALL 僅接受純 ASCII kebab-case:小寫英文字母與數字組成的段落以單一連字號串接(不得為空、不得含大寫、空白、底線或非 ASCII 字元、不得以連字號開頭或結尾、不得出現連續連字號)。非法值時指令 SHALL 以非零 exit code 結束、於 stderr 說明原因,且 SHALL NOT 建立任何檔案。合法時 SHALL 於 openspec/discussions/ 下以該值為檔名建立記錄,人眼輸出與 --json 的 slug 欄位 SHALL 反映覆寫值;--slug 與既有討論同名時 SHALL 沿用現行已存在錯誤行為。本旗標為 Speclink 自有延伸,未帶 --slug 的既有輸出 SHALL 逐位元不變,回歸對照不受影響。

#### Scenario: 帶合法 --slug 以中文主題建立討論

- **WHEN** 執行 speclink discuss new 並給定中文主題「看板搜尋列」與 --slug board-search-bar
- **THEN** 建立 openspec/discussions/board-search-bar.md,frontmatter 的 topic 為「看板搜尋列」、slug 為 board-search-bar;stdout 顯示建立訊息含 board-search-bar;帶 --json 時 payload 的 slug 欄位為 board-search-bar、topic 欄位為原文

#### Scenario: 非法 --slug 值被拒且不落檔

- **WHEN** 執行 speclink discuss new 且 --slug 的值不符合純 ASCII kebab-case
- **THEN** 指令以非零 exit code 結束,stderr 說明 slug 格式要求,openspec/discussions/ 下不新增任何檔案

##### Example: 非法值一覽

| --slug 值 | 結果 | 原因 |
| --------- | ---- | ---- |
| Board-Search | 拒絕 | 含大寫字母 |
| 看板搜尋 | 拒絕 | 含非 ASCII 字元 |
| board_search | 拒絕 | 含底線 |
| board search | 拒絕 | 含空白 |
| -board | 拒絕 | 以連字號開頭 |
| board--search | 拒絕 | 連續連字號 |
| (空字串) | 拒絕 | 不得為空 |
| board-search-2 | 接受 | 合法 kebab-case |

#### Scenario: --slug 與既有討論同名

- **WHEN** 執行 speclink discuss new 且 --slug 的值等於一份既有討論的 slug
- **THEN** 指令以非零 exit code 結束並回報該討論已存在,不覆寫既有檔案

---
### Requirement: 未帶 --slug 時自主題衍生檔名

未提供 --slug 時,speclink discuss new SHALL 維持既有衍生規則自主題產生 slug:前後空白修剪、ASCII 英數字轉小寫、空白與底線轉為連字號、ASCII 標點滌除、非 ASCII 字母(如 CJK)原樣保留、結尾連字號移除。衍生結果為空時 SHALL 以非零 exit code 報錯。此為後備行為的正典化,SHALL 與本變更前逐位元一致。

#### Scenario: 中英混合主題衍生 slug

- **WHEN** 執行 speclink discuss new 並給定主題且未帶 --slug
- **THEN** 以衍生規則產生 slug 並建立 openspec/discussions/ 下的對應檔案

##### Example: 衍生規則對照

| 主題 | 衍生 slug |
| ---- | --------- |
| Board Search | board-search |
| config context 與 rules GUI 編輯 | config-context-與-rules-gui-編輯 |
| 看板 搜尋列 | 看板-搜尋列 |
| !?! | (空,報錯) |

---
### Requirement: 討論技能指示要求英文 slug

speclink 生成的 discuss 技能內容 SHALL 指示 agent 在建立討論記錄時,從主題自行衍生英文 kebab-case slug 並以 --slug 傳入;主題(topic)維持使用者語言原文。內嵌技能資產與 render golden 基準 SHALL 同步反映此指示。

#### Scenario: 生成的技能含 --slug 指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 discuss 技能
- **THEN** 生成的技能檔內容包含「衍生英文 kebab-case slug 並以 --slug 傳入」的指示,render golden 測試以更新後基準通過

---
### Requirement: 討論以 link 動詞併入既有變更

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），將兩側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）的 frontmatter SHALL 標記 status: promoted，且 promoted_to SHALL 以逗號累加該變更名、既有值保留不覆蓋。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查：無其他在途變更的 from_discussion 清單引用該討論時，既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/；僅單一來源討論之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸，不在 Spectra 對照範圍；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 成功併入既有變更

- **WHEN** 執行 speclink discuss link 並給定一份未封存討論的 slug 與一個尚無 from_discussion 的既有變更名
- **THEN** exit code 0；stdout 單行成功訊息含該 slug 與變更名；openspec/changes/<change>/.openspec.yaml 增寫 from_discussion: <slug>；討論 frontmatter 變為 status: promoted 且 promoted_to 含該變更名；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: 已轉出其他變更的討論再併入

- **WHEN** 對 promoted_to 已含其他變更名的討論執行 speclink discuss link 指向另一個既有變更
- **THEN** promoted_to 以逗號累加新變更名且既有值保留；exit code 0

#### Scenario: 出身自討論的變更再併入新討論

- **WHEN** 對 meta 已有 from_discussion 指向其他討論的變更執行 speclink discuss link 給定另一份討論的 slug
- **THEN** exit code 0；變更 meta 的 from_discussion 於既有值尾端累加本 slug、既有值保留；本討論 frontmatter 變為 status: promoted 且 promoted_to 累加該變更名；先前連結的討論記錄逐位元不變

##### Example: 累加後的 meta 欄位

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search
- **WHEN** 執行 speclink discuss link beta-cache cut-a
- **THEN** cut-a 的 meta 欄位為 from_discussion: alpha-search, beta-cache；alpha-search 的記錄逐位元不變

#### Scenario: 同一組合重跑為冪等

- **WHEN** 對已互相連結的同一組討論與變更再次執行 speclink discuss link（含該討論僅為 from_discussion 清單其中一員的情形）
- **THEN** exit code 0；變更 meta 檔與討論記錄內容逐位元不變

#### Scenario: 守衛拒絕且不落檔

- **WHEN** 執行 speclink discuss link 且命中任一守衛：討論不存在、討論已封存、變更不存在
- **THEN** 指令以非零 exit code 結束，stderr 說明原因，openspec/changes/ 與 openspec/discussions/ 下任何檔案逐位元不變

##### Example: 守衛一覽

| 情境 | 結果 |
| ---- | ---- |
| slug 無對應討論記錄 | 拒絕：討論不存在 |
| slug 僅存在於 discussions/archive/ | 拒絕：討論已封存 |
| 變更名無對應目錄 | 拒絕：變更不存在 |
| 變更 meta 已有 from_discussion 指向其他討論 | 累加：既有值尾端追加本 slug |
| 變更 meta 的 from_discussion 清單已含本 slug | 冪等成功，不改檔 |

#### Scenario: 併入後隨變更自動封存

- **WHEN** 已 link 的變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

#### Scenario: 多來源討論的變更封存逐一共行

- **WHEN** 封存 from_discussion 清單含兩份討論的變更
- **THEN** 清單中每份討論各自檢查存活引用：無其他在途變更引用者移入 openspec/discussions/archive/、人眼輸出逐討論各一行共行訊息；仍被引用者維持在途不動

##### Example: 一份隨行一份留下

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search, beta-cache；另一在途變更 cut-b 的 meta 含 from_discussion: beta-cache
- **WHEN** 執行 speclink archive cut-a
- **THEN** alpha-search 移入 openspec/discussions/archive/、輸出含其共行訊息一行；beta-cache 仍為在途記錄不動


<!-- @trace
source: rediscuss-promoted-change
updated: 2026-07-09
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/src/App.tsx
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/teststore.rs
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/siblings.test.ts
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/siblings.ts
-->

---
### Requirement: 技能指示引導 ingest 型結論先鑄鏈

speclink 生成的 discuss 技能內容 SHALL 指示 agent：討論結論的 Capture to 指向既有變更時，先執行 speclink discuss link 鑄鏈、再導向 /speclink-ingest 更新該變更的 artifacts。生成的 ingest 技能內容 SHALL 包含來源討論確認提示：更新內容源自某份討論結論時，確認該討論已與目標變更連結。內嵌技能資產、repo 技能實例（claude 與 codex 兩工具）與 render golden 基準 SHALL 同步反映此指示。

#### Scenario: 生成的 discuss 技能含 link 指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 discuss 技能
- **THEN** 生成的技能檔內容包含「Capture to 指向既有變更時先執行 speclink discuss link 再走 /speclink-ingest」的指示，render golden 測試以更新後基準通過

#### Scenario: 生成的 ingest 技能含來源討論提示

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 ingest 技能
- **THEN** 生成的技能檔內容包含「更新源自討論結論時確認已執行 speclink discuss link」的提示，render golden 測試以更新後基準通過

---
### Requirement: 討論隨變更廢棄解鏈

speclink discard 廢棄變更時，對變更 meta 的 from_discussion 清單中每份討論 SHALL 解鏈：自記錄 frontmatter 的 promoted_to 逗號清單移除該變更名、其餘值保留；清單仍有值時狀態 SHALL 維持 promoted；清單因此變空時 SHALL 移除 promoted_to 行並回退狀態——記錄的 Conclusion 區非空時回 concluded、為空時回 open。記錄的 Context、Rounds 與 Conclusion 區內容 SHALL 逐位元不變。slug 無對應記錄時 SHALL 跳過且不視為錯誤。解鏈 SHALL 於刪除變更目錄前完成；對已解鏈的討論重跑 SHALL 冪等（變更名已不在 promoted_to 即不改檔）。

#### Scenario: 最後連結死亡回退 concluded

- **WHEN** 廢棄的變更名是某份有結論討論 promoted_to 的唯一值
- **THEN** 該記錄的 promoted_to 行消失、status 回 concluded；Context、Rounds 與 Conclusion 逐位元不變

##### Example: 回退前後的 frontmatter

- **GIVEN** 討論 alpha-search 的 frontmatter 含 status: promoted 與 promoted_to: cut-a，Conclusion 區非空
- **WHEN** 執行 speclink discard cut-a
- **THEN** frontmatter 變為 status: concluded 且無 promoted_to 行

#### Scenario: 仍有其他變更時維持 promoted

- **WHEN** 廢棄的變更名只是某討論 promoted_to 逗號清單的其中一員
- **THEN** 清單移除該名、其餘值保留；status 維持 promoted

#### Scenario: 無結論的討論回退 open

- **WHEN** 廢棄的變更名是某份 Conclusion 區為空的討論（open 狀態經 link 併入）promoted_to 的唯一值
- **THEN** 該記錄的 promoted_to 行消失、status 回 open

#### Scenario: 多來源討論逐一解鏈

- **WHEN** 廢棄 from_discussion 清單含兩份討論的變更
- **THEN** 兩份記錄各自依上述規則處理：一份因清單空而回退、另一份仍被其他變更引用則僅縮減清單並維持 promoted

#### Scenario: 缺失記錄跳過

- **WHEN** 廢棄的變更 from_discussion 指向的某 slug 無對應記錄（live 與 archive 皆無）
- **THEN** 該 slug 跳過、其餘討論照常解鏈；指令不因此失敗

<!-- @trace
source: discard-change-verb
updated: 2026-07-09
code:
  - README.en.md
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/discard.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-node/src/store_bridge.rs
-->