# discussion-docs Specification

## Purpose

討論記錄文件的語意：檔名 slug 的指定與自主題衍生、以 link 動詞併入既有變更、以 seal 標記內容已轉出、重新結論時把變更標為待重新反映，以及 from_discussion 鏈的可觀測性與隨變更廢棄時的解鏈。本 capability 保證討論與變更之間的鏈結雙向可查、討論記錄蓋建立者章，空內容與壞 change metadata 一律拒絕寫入，且動詞在 remote 模式與本機同語意。

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

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），鑄造變更側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）SHALL 逐位元不變——link SHALL NOT 標記 status: promoted、SHALL NOT 寫 promoted_to；「已轉出」的標記職責移交 speclink discuss seal。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查兩個條件皆成立才隨行封存：無其他在途變更的 from_discussion 清單引用該討論，且該討論的 Conclusion 段已寫入內文（scaffold 佔位註解不算內文；判準 SHALL NOT 依 frontmatter status——promoted 討論寫入結論後 status 仍為 promoted）；兩條件皆成立時既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/。Conclusion 未寫入的討論 SHALL 維持在途、SHALL NOT 隨行封存、SHALL NOT 出現於封存輸出的隨行封存清單，其後 discuss add-round 與 discuss conclude SHALL 照常可用；討論記錄讀取失敗時 SHALL 視同未寫入結論（留在途，不吞進封存區）。僅單一來源討論且該討論已有結論之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 成功併入既有變更

- **WHEN** 執行 speclink discuss link 並給定一份未封存討論的 slug 與一個尚無 from_discussion 的既有變更名
- **THEN** exit code 0；stdout 單行成功訊息含該 slug 與變更名；openspec/changes/<change>/.openspec.yaml 增寫 from_discussion: <slug>；討論記錄逐位元不變（status 與 promoted_to 皆不變）；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: link 不改討論記錄

- **WHEN** 對任一狀態（open、concluded、或已由 seal 標記 promoted）的討論執行 speclink discuss link 指向既有變更
- **THEN** exit code 0；變更 meta 的 from_discussion 累加該 slug；討論記錄 frontmatter 與內文逐位元不變

#### Scenario: 出身自討論的變更再併入新討論

- **WHEN** 對 meta 已有 from_discussion 指向其他討論的變更執行 speclink discuss link 給定另一份討論的 slug
- **THEN** exit code 0；變更 meta 的 from_discussion 於既有值尾端累加本 slug、既有值保留；本討論與先前連結的討論記錄皆逐位元不變

##### Example: 累加後的 meta 欄位

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search
- **WHEN** 執行 speclink discuss link beta-cache cut-a
- **THEN** cut-a 的 meta 欄位為 from_discussion: alpha-search, beta-cache；alpha-search 的記錄逐位元不變

#### Scenario: 同一組合重跑為冪等

- **WHEN** 對已互相連結的同一組討論與變更再次執行 speclink discuss link
- **THEN** exit code 0；變更 meta 檔與討論記錄內容逐位元不變

#### Scenario: 守衛拒絕且不落檔

- **WHEN** 執行 speclink discuss link 且命中任一守衛：討論不存在、討論已封存、變更不存在
- **THEN** 指令以非零 exit code 結束，stderr 說明原因，openspec/changes/ 與 openspec/discussions/ 下任何檔案逐位元不變

#### Scenario: 併入後隨變更自動封存

- **WHEN** 已 link 且 Conclusion 段已寫入內文的討論，其變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

#### Scenario: 未有結論的討論不隨變更封存

- **WHEN** 討論中途轉出（或併入）的變更執行封存，該討論的 Conclusion 段仍為 scaffold 佔位註解、無其他在途變更引用它
- **THEN** 變更照常封存（exit code 0），該討論維持於 openspec/discussions/、SHALL NOT 移入 openspec/discussions/archive/，封存輸出的隨行封存清單不含該討論；其後對該討論執行 discuss add-round 與 discuss conclude 皆照常成功

#### Scenario: 多來源討論的變更封存逐一共行

- **WHEN** 封存 from_discussion 清單含兩份討論的變更
- **THEN** 清單中每份討論各自檢查存活引用與結論：無其他在途變更引用且已有結論者移入 openspec/discussions/archive/、人眼輸出逐討論各一行共行訊息；仍被引用或未有結論者維持在途不動


<!-- @trace
source: conclusion-gated-discussion-archive
updated: 2026-09-01
-->

---
### Requirement: 技能指示引導 ingest 型結論先鑄鏈

speclink 生成的 discuss 技能內容 SHALL 指示 agent：討論結論的 Capture to 指向既有變更時，先執行 speclink discuss link 鑄鏈、再導向 /speclink-ingest 更新該變更的 artifacts。生成的 ingest 技能內容 SHALL 指示 agent：目標變更 meta 帶 from_discussion 時，經 speclink discuss show 讀取該討論結論作為一等來源、併入既有對話脈絡或 plan（不取代），並於 artifacts 更新完成時執行 speclink discuss seal 標記已轉出。內嵌技能資產、repo 技能實例（claude 與 codex 兩工具）與 render golden 基準 SHALL 同步反映此指示。

#### Scenario: 生成的 discuss 技能含 link 指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 discuss 技能
- **THEN** 生成的技能檔內容包含「Capture to 指向既有變更時先執行 speclink discuss link 再走 /speclink-ingest」的指示，render golden 測試以更新後基準通過

#### Scenario: 生成的 ingest 技能含讀討論與封印指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 ingest 技能
- **THEN** 生成的技能檔內容包含「目標變更帶 from_discussion 時經 speclink discuss show 讀結論併入來源，並於完成時執行 speclink discuss seal」的指示，render golden 測試以更新後基準通過


<!-- @trace
source: discussion-reflection-seal
updated: 2026-07-09
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_seal.rs
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/ingest.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
-->

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

---
### Requirement: 內容落地以 seal 動詞標記已轉出

speclink discuss seal SHALL 接受兩個位置參數（討論 slug 與變更名）。前置守衛 SHALL 全數通過方可寫入：討論 SHALL 存在且未封存、變更 SHALL 存在、且變更 meta 的 from_discussion 清單 SHALL 含該 slug（鏈須先由 link／promote／new change 鑄妥）——任一不滿足時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案逐位元不變。守衛通過時：討論記錄 frontmatter 的 status SHALL 標記 promoted（由 open 或 concluded 轉入），promoted_to SHALL 以逗號累加該變更名、既有值保留不覆蓋。指令不吃 stdin，旗標僅 --json 與 --no-color。成功時 exit code 0、stdout 單行成功訊息（--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。同一組合重跑 SHALL 為冪等成功：promoted_to 已含該變更名時不改檔、exit code 0。本指令為 Speclink 自有延伸。

#### Scenario: 成功封印標記已轉出

- **WHEN** 對一份 concluded 討論執行 speclink discuss seal，且目標變更 meta 的 from_discussion 已含該 slug
- **THEN** exit code 0；討論 frontmatter 變為 status: promoted 且 promoted_to 含該變更名；stdout 單行成功訊息；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: 鏈未鑄妥守衛拒絕

- **WHEN** 執行 speclink discuss seal 但目標變更 meta 的 from_discussion 不含該 slug
- **THEN** 指令以非零 exit code 結束、stderr 說明鏈未存在；討論記錄與變更 meta 皆逐位元不變

#### Scenario: 重跑封印為冪等

- **WHEN** 對 promoted_to 已含該變更名的討論再次執行 speclink discuss seal
- **THEN** exit code 0；討論記錄逐位元不變

##### Example: seal 守衛一覽

| 情境 | 結果 |
| ---- | ---- |
| slug 無對應討論記錄 | 拒絕：討論不存在 |
| slug 僅存在於 discussions/archive/ | 拒絕：討論已封存 |
| 變更名無對應目錄 | 拒絕：變更不存在 |
| 變更 meta 的 from_discussion 未含該 slug | 拒絕：鏈未鑄妥 |
| promoted_to 已含該變更名 | 冪等成功，不改檔 |


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: from_discussion 鏈可經 show --json 觀察

speclink show <change> --json 的 payload SHALL 含 fromDiscussions 欄位（camelCase），值為變更 meta from_discussion 逗號清單解析後去除空白的有序字串陣列；變更無 from_discussion 時 SHALL 為空陣列。既有 payload 欄位 SHALL 逐位元不變。

#### Scenario: 有連結時列出討論 slug

- **WHEN** 對 meta 含 from_discussion: alpha-search, beta-cache 的變更執行 speclink show <change> --json
- **THEN** payload 的 fromDiscussions 為 ["alpha-search", "beta-cache"]（順序與 meta 一致）

#### Scenario: 無連結時為空陣列

- **WHEN** 對 meta 無 from_discussion 的變更執行 speclink show <change> --json
- **THEN** payload 的 fromDiscussions 為空陣列 []；其餘欄位不變

<!-- @trace
source: discussion-reflection-seal
updated: 2026-07-09
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_seal.rs
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/ingest.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
-->

---
### Requirement: 討論重新結論標記已反映變更待重新反映

speclink discuss conclude 寫入結論後 SHALL 檢查討論記錄 frontmatter 的 promoted_to：非空時 SHALL 對其中每個變更名判存活——僅 openspec/changes/<name>/ 存在（active）者納入蓋章，僅存在於 openspec/changes/archive/ 者 SHALL 跳過。對每個納入的 active 變更，其 meta 檔（openspec/changes/<name>/.openspec.yaml）的 restale_from 欄位 SHALL 以逗號累加本討論 slug——尚無該欄位時增寫為單值、已有其他值時於尾端累加、已含本 slug 時 SHALL 冪等不改該檔；既有 meta 欄位 SHALL 逐字保留。判存活的鍵 SHALL 為 promoted_to 非空（曾被反映），SHALL NOT 綁 status 欄位值。promoted_to 為空、或其項全為已歸檔變更時，SHALL NOT 寫入任何變更 meta。討論記錄的 Context、Rounds 區 SHALL 逐位元不變（僅 Conclusion 區依既有 conclude 行為改寫）。成功時 stdout SHALL 於既有結論訊息後，另報告被標記待重新反映的 active 變更清單（無則不報告）；帶 --json 時 payload SHALL 含被標記變更名的陣列。本行為為 Speclink 自有延伸；未觸發蓋章時（promoted_to 空）既有 conclude 的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 重新結論已反映討論蓋章其 active 變更

- **WHEN** 對 promoted_to 含一個 active 變更名的討論執行 speclink discuss conclude
- **THEN** 該變更 meta 的 restale_from 累加本討論 slug；討論記錄除 Conclusion 區外逐位元不變；stdout 報告該變更被標記待重新反映

#### Scenario: 蓋章跳過已歸檔變更

- **WHEN** 對 promoted_to 同時含一個 active 變更與一個已歸檔變更的討論執行 speclink discuss conclude
- **THEN** 僅 active 變更 meta 的 restale_from 累加本 slug；已歸檔變更目錄下任何檔案逐位元不變

#### Scenario: promoted_to 空的結論不蓋章

- **WHEN** 對 promoted_to 為空（尚未 seal）的討論執行 speclink discuss conclude
- **THEN** 不寫入任何變更 meta；既有 conclude 的人眼與 --json 輸出逐位元不變

#### Scenario: 重複重新結論為冪等

- **WHEN** 對已因先前結論而使某 active 變更 restale_from 含本 slug 的討論再次執行 speclink discuss conclude
- **THEN** 該變更 meta 逐位元不變（restale_from 已含本 slug 不重複累加）

##### Example: 蓋章後的變更 meta

- **GIVEN** 討論 alpha-search 的 promoted_to 含 active 變更 cut-a，cut-a 的 meta 無 restale_from
- **WHEN** 執行 speclink discuss conclude alpha-search 帶新結論
- **THEN** cut-a 的 meta 增寫 restale_from: alpha-search


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: seal 清除變更的 restale 旗標

speclink discuss seal 通過既有守衛並標記討論 promoted 後 SHALL 額外自目標變更 meta 檔的 restale_from 逗號清單移除本討論 slug：清單移除後仍有值時保留其餘值、變空時 SHALL 移除 restale_from 行；本 slug 不在清單（或無該欄位）時 SHALL 冪等不改該檔。清除 SHALL 僅動 restale_from 欄位，變更 meta 其餘欄位與討論記錄 SHALL 逐位元不變。此清除使 re-conclude → re-ingest → seal 成閉環：seal 作為誠實的「內容落地」動作，清掉對應該討論的過期標記。既有 seal 的守衛、promoted 標記、輸出與冪等行為 SHALL 不變。

#### Scenario: seal 清除對應 slug 的 restale 旗標

- **WHEN** 對 restale_from 含本討論 slug 的目標變更執行 speclink discuss seal
- **THEN** 該變更 meta 的 restale_from 移除本 slug；其餘 restale_from 值與所有其他欄位逐位元不變；討論如常標記 promoted

#### Scenario: restale_from 變空移除整行

- **WHEN** seal 移除的 slug 是目標變更 restale_from 的唯一值
- **THEN** 該變更 meta 的 restale_from 行消失；其他欄位逐位元不變

#### Scenario: 無對應旗標時清除為冪等

- **WHEN** 對 restale_from 不含本 slug（或無該欄位）的目標變更執行 speclink discuss seal
- **THEN** 變更 meta 逐位元不變（除既有 seal 的 from_discussion／promoted 相關行為外）

##### Example: 多討論過期時 per-slug 清除

- **GIVEN** 變更 cut-a 的 meta 含 restale_from: alpha-search, beta-cache
- **WHEN** 執行 speclink discuss seal alpha-search cut-a
- **THEN** cut-a 的 restale_from 變為 beta-cache（beta-cache 仍待其各自 re-seal）

<!-- @trace
source: reconclude-restale
updated: 2026-07-09
code:
  - apps/desktop/core/src/query.rs
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/reconclude_restale.rs
  - crates/speclink-core/assets/skills/ingest.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 討論內容寫入動詞拒絕空內容

討論內容寫入動詞（context、add-round、conclude）SHALL 在其內容去除前後空白後為空時以錯誤中止，SHALL NOT 靜默寫入空的 Context／Round／Conclusion；conclude 遇空內容 SHALL NOT 將討論 status 翻為 concluded。錯誤訊息 SHALL 指出內容為空並提醒可能漏帶 --stdin。此空內容檢查 SHALL 置於引擎（core::discuss）以覆蓋所有前門（本地 CLI、遠端 CLI、桌面）。CLI SHALL 於標準輸入為管線（非互動終端）時讀取其內容作為動詞內容，不論是否帶 --stdin 旗標；--stdin 旗標 SHALL 維持被接受以相容既有腳本。add-round SHALL 維持純附加，SHALL NOT 提供改寫既有輪的動詞；context 與 conclude SHALL 維持重跑即覆寫該區段的既有行為。

#### Scenario: add-round 空內容中止且不新增輪

- **WHEN** 對某討論執行 add-round，而提供的內容去除前後空白後為空（例如漏帶 --stdin 且無管線輸入）
- **THEN** 指令以錯誤中止、不新增任何 Round，錯誤訊息指出內容為空並提醒 --stdin

#### Scenario: conclude 空內容中止且不翻狀態

- **WHEN** 對某討論執行 conclude，而內容去除前後空白後為空
- **THEN** 指令以錯誤中止、不寫入空 Conclusion，且該討論 status 不翻為 concluded

#### Scenario: context 空內容中止且不覆寫

- **WHEN** 對某討論執行 context，而內容去除前後空白後為空
- **THEN** 指令以錯誤中止、不以空內容覆寫既有 Context 區段

#### Scenario: 管線內容於未帶 --stdin 時仍被寫入

- **WHEN** 以管線將非空內容送入 add-round，但未帶 --stdin 旗標
- **THEN** 該內容被當作輪內容寫入，不因漏帶旗標而靜默成空

#### Scenario: 空內容 guard 覆蓋桌面前門

- **WHEN** 桌面經內嵌引擎對某討論以空內容執行任一內容寫入動詞
- **THEN** 該寫入以與 CLI 相同的 core guard 被拒，不產生空區段

<!-- @trace
source: discuss-content-guard
updated: 2026-07-09
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_content_guard.rs
  - crates/speclink-core/src/discuss.rs
-->

---
### Requirement: 討論記錄蓋建立者章

建立討論記錄（discuss new）時，引擎 SHALL 於記錄 frontmatter 蓋建立者 created_by，取自 git 身分（user.name 與 email）；git 身分不可得時 SHALL 省略該欄位。discuss list 與 show 的 --json 輸出 SHALL 以 camelCase createdBy 曝露該值，缺席時省略。既有無 created_by 的討論記錄 SHALL 照常運作，其 createdBy 缺席、不報錯。

#### Scenario: discuss new 於有 git 身分時蓋建立者

- **WHEN** 於設有 git user.name 與 email 的專案執行 discuss new
- **THEN** 產生的討論記錄 frontmatter 含 created_by 為該 git 身分，且 discuss show --json 的 createdBy 為同值

#### Scenario: 無 git 身分時省略建立者

- **WHEN** 於無可解析 git 身分的環境執行 discuss new
- **THEN** 討論記錄不含 created_by，--json 的 createdBy 缺席，記錄仍正常建立

#### Scenario: 既有無建立者記錄照常運作

- **WHEN** list 或 show 一筆 frontmatter 無 created_by 的既有討論
- **THEN** 指令正常輸出、其 createdBy 缺席，不報錯

<!-- @trace
source: desktop-card-identity
updated: 2026-07-09
code:
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/query.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/discuss.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 討論鏈結動詞對壞 change metadata 拒絕

speclink discuss link 與 speclink discuss seal 在對象 change 的 `.openspec.yaml` 存在但 YAML 解析失敗時 SHALL 以帶檔案位置與解析原因的錯誤拒絕：SHALL NOT 寫入該 change 的 metadata，也 SHALL NOT 改動討論記錄。壞 metadata SHALL NOT 被解讀為「無 from_discussion 鏈」或「無 restale 旗標」——拒絕原因 SHALL 是 metadata 損壞，而非既有守衛的「鏈不存在」誤導訊息。

#### Scenario: link 對壞 metadata 拒絕且兩側皆不寫

- **WHEN** 執行 speclink discuss link 給定一份未封存討論的 slug 與一個 `.openspec.yaml` 為壞 YAML 的既有變更名
- **THEN** 以非零 exit code 結束；該 `.openspec.yaml` 與 discussions/ 下該討論記錄皆逐位元不變

#### Scenario: seal 對壞 metadata 拒絕且不誤報鏈缺失

- **WHEN** 對壞 metadata 的變更執行 speclink discuss seal（兩個位置參數：討論 slug 與該變更名）
- **THEN** 以非零 exit code 結束；stderr 指出 metadata 檔損壞（而非 from_discussion 不含該 slug）；兩側檔案逐位元不變

---
### Requirement: 討論動詞於 remote 模式與本機同語意

remote 模式下 speclink discuss new 的 --slug 覆寫、speclink discuss discard（含 --force）、speclink discuss link 與 speclink discuss seal SHALL 可用，語意與 fs 模式一致。slug 的 ASCII kebab-case 驗證與 discard 的有輪 guard SHALL 由引擎於 server 端執行（單一事實來源），CLI 與 server 路由 SHALL NOT 重複實作驗證邏輯；引擎拒絕 SHALL 映射為語義化錯誤訊息與非零 exit code。未升級的舊 server 對新動詞回應找不到端點時，CLI SHALL 呈現語義化錯誤訊息並以非零 exit code 結束，SHALL NOT panic。

#### Scenario: remote 帶 --slug 以中文主題建立討論

- **WHEN** 於 remote 模式執行 speclink discuss new 並給定中文主題「看板搜尋列」與 --slug board-search-bar
- **THEN** server 端建立 slug 為 board-search-bar 的討論記錄，topic 保留中文原文；stdout 顯示建立訊息含 board-search-bar，--json 的 slug 欄位為 board-search-bar

#### Scenario: remote 非法 --slug 被拒且 server 不落檔

- **WHEN** 於 remote 模式執行 speclink discuss new 並帶 --slug「中文slug」
- **THEN** exit code 非 0，stderr 說明 slug 格式要求，server 端未建立任何討論記錄

#### Scenario: remote discard 的輪數 guard 與本機一致

- **WHEN** 於 remote 模式對 server 上一筆 0 輪討論執行 speclink discuss discard，再對一筆已有 2 輪的討論執行同指令（無 --force）
- **THEN** 0 輪記錄被刪除且 exit code 為 0；2 輪記錄保留、exit code 非 0 且 stderr 提示需 --force；對 2 輪記錄帶 --force 重試則刪除成功

#### Scenario: remote link 鑄鏈可經 show 觀察

- **WHEN** 於 remote 模式執行 speclink discuss link 某已結論討論與某既有 change，隨後執行 speclink show 該 change --json
- **THEN** link 指令 exit code 為 0，show 的 payload 中 from_discussion 鏈含該討論 slug，與 fs 模式同欄位形狀

#### Scenario: remote seal 標記已轉出

- **WHEN** 於 remote 模式對已 link 的討論執行 speclink discuss seal
- **THEN** exit code 為 0，該討論於 server 端標記為已轉出（promoted），speclink discuss list --json 反映該狀態

<!-- @trace
source: remote-cli-parity
updated: 2026-07-31
code:
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_slug.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: 討論記錄以 --kind 標記改進討論

speclink discuss new SHALL 接受選配旗標 --kind,白名單僅接受 improve。合法值時 SHALL 於 frontmatter 增 kind 欄位,--json payload SHALL 增 kind 欄位,人眼輸出 SHALL 沿用既有建立訊息格式、SHALL NOT 新增行。非白名單值時指令 SHALL 以非零 exit code 結束、於 stderr 說明僅接受 improve,且 SHALL NOT 建立任何檔案。未帶 --kind 時人眼輸出與 --json SHALL 逐位元不變,回歸對照不受影響。無 kind 欄位的既有記錄 SHALL 視為一般討論,SHALL NOT 要求遷移。speclink discuss list --json 與 speclink discuss show --json SHALL 於記錄有 kind 時曝露該欄位、無 kind 時省略該鍵。本旗標為 Speclink 自有延伸。

#### Scenario: 帶 --kind improve 建立改進討論

- **WHEN** 執行 speclink discuss new 並給定主題、合法 --slug 與 --kind improve
- **THEN** 建立的記錄 frontmatter 含 kind: improve;帶 --json 時 payload 的 kind 欄位為 improve;人眼輸出與未帶 --kind 時的建立訊息格式一致

#### Scenario: 非法 kind 值被拒且不落檔

- **WHEN** 執行 speclink discuss new 且 --kind 的值不是 improve
- **THEN** 指令以非零 exit code 結束,stderr 說明僅接受 improve,openspec/discussions/ 下不新增任何檔案

#### Scenario: 未帶 --kind 輸出不變

- **WHEN** 執行 speclink discuss new 未帶 --kind
- **THEN** 人眼輸出與 --json payload 與本旗標引入前逐位元一致,frontmatter 無 kind 欄位

#### Scenario: list 與 show 曝露 kind

- **WHEN** 對含 kind: improve 的記錄執行 speclink discuss list --json 與 speclink discuss show --json
- **THEN** 兩者 payload 均含 kind 欄位且值為 improve;對無 kind 的記錄則 payload 不含該鍵

<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->

---
### Requirement: 討論記錄結構錨定與撞名內容跳脫

引擎解析討論記錄時 SHALL 只把整行為「## Context」「## Rounds」「## Conclusion」的行視為結構標題，並保留 pre-scaffold 容忍：未經跳脫、行首為「## Round 」的行亦視為區段邊界（pre-scaffold 版面的既有輪與結論不得因區段替換而遺失）；輪內文中其他以「## 」開頭的行 SHALL NOT 截斷區段、SHALL NOT 改變 add-round 的插入點。討論內容寫入動詞（context、add-round、conclude）SHALL 在落盤前把撞名內容行（整行為結構標題，或行首為「### Round 」「## Round 」前綴）以 markdown 反斜線跳脫；成對 fenced code block（``` 或 ~~~ 圍欄）內的行 SHALL NOT 跳脫、SHALL NOT 視為結構或輪標題，且寫入動詞 SHALL 使落盤內容的圍欄行成對——內容含奇數個圍欄行時，最後一個落單的圍欄行一併以反斜線跳脫。輪計數 SHALL 只認合法輪標題形狀（「### Round <編號> — <mode> (<日期>)」，與 UI 輪切分同形；並保留 pre-scaffold「## Round 」前綴的容忍），SHALL NOT 因內文撞名而膨脹或跳號。桌面與 Web 的區段切分 SHALL 與引擎採同一結構標題白名單與容忍規則。

#### Scenario: 輪內文含二級標題行時新輪仍落於既有輪之後

- **WHEN** 既有 Round 1 的內文含一行「## 背景」，執行 add-round 追加第二輪
- **THEN** Round 2 標題插在 Round 1 完整內文之後、Conclusion 結構標題之前；Round 1 的內文（含該「## 背景」行）完整留在 Round 1 標題之下

##### Example: 兩輪順序與內文歸屬

- **GIVEN** Round 1 內文兩行——「## 背景」與「首輪本文」
- **WHEN** add-round 追加內文為「次輪本文」的 Round 2
- **THEN** 文件順序為：Round 1 標題、「## 背景」、「首輪本文」、Round 2 標題、「次輪本文」、Conclusion 結構標題

#### Scenario: 結論寫入不落入輪內

- **WHEN** 某輪內文原始輸入含整行「## Conclusion」，其後執行 conclude
- **THEN** 該內容行以跳脫形式留在該輪內文中；結論內容寫入文件的結構 Conclusion 區段；既有輪內文不被改寫

#### Scenario: 撞名輪標題行不膨脹輪計數

- **WHEN** 某輪內文含行首為「### Round 」的行，隨後 add-round 追加新輪
- **THEN** 新輪編號等於既有合法輪數加一，無跳號；discuss list 回報的輪數等於合法輪標題數

#### Scenario: 區段切分不被輪內文截斷

- **WHEN** 桌面或 Web 檢視的討論記錄，其某輪內文含以「## 」開頭的行
- **THEN** Rounds 區段完整切分至 Conclusion 結構標題為止，該輪內文完整顯示於該輪之下

<!-- @trace
source: fix-discuss-section-anchor
updated: 2026-08-27
-->

---
### Requirement: conclude 於全數轉出變更已封存時順手封存討論

speclink discuss conclude 寫入結論後 SHALL 檢查閉環條件：討論 frontmatter 的 promoted_to 清單非空，且無任何在途變更的 from_discussion 清單引用本討論。兩條件皆成立時 SHALL 於結論寫入後將討論記錄移入 openspec/discussions/archive/（沿用既有討論封存的檔名與同日撞名解法），stdout 於既有輸出之後 SHALL 多一行告知已順手封存（--no-color 下無 ANSI 色彩），帶 --json 時 payload SHALL 增 autoArchived 欄位（camelCase 布林）且僅於觸發時出現。任一條件不成立時 SHALL NOT 封存，人眼與 --json 輸出 SHALL 與變更前逐位元一致（不出現 autoArchived 鍵）。寫入順序為兩步：先寫結論、再嘗試封存；封存步失敗時結論寫入 SHALL NOT 回滾——可觀察狀態為「已結論、記錄仍在 openspec/discussions/」，指令以非零 exit code 結束、stderr 說明封存步失敗原因，其後執行 speclink discuss archive SHALL 可收尾。此閉環與連帶封存守門互補：conclude 時仍有轉出變更在途則交由最後一個變更封存時隨行封存。

#### Scenario: 全數轉出變更已封存時 conclude 順手封存

- **WHEN** 對 promoted_to 含一個變更名、該變更已封存、無在途變更引用的討論執行 speclink discuss conclude 寫入結論
- **THEN** exit code 0；結論寫入記錄且 status 保持 promoted；記錄移入 openspec/discussions/archive/；stdout 多一行告知順手封存；帶 --json 時 payload 含 autoArchived: true

#### Scenario: 仍有轉出變更在途時 conclude 不封存

- **WHEN** 對 promoted_to 非空、但仍有一個在途變更的 from_discussion 引用本討論的討論執行 speclink discuss conclude
- **THEN** exit code 0；結論寫入，記錄維持於 openspec/discussions/；人眼與 --json 輸出與本變更前的 conclude 行為逐位元一致，無 autoArchived 鍵

#### Scenario: 未曾轉出的討論 conclude 行為不變

- **WHEN** 對 promoted_to 缺席的 open 討論執行 speclink discuss conclude
- **THEN** exit code 0；status 轉為 concluded，記錄維持在途；人眼與 --json 輸出與本變更前逐位元一致

#### Scenario: 閉環封存步失敗保留結論

- **WHEN** conclude 的閉環條件成立、結論寫入成功、但封存步因儲存層錯誤失敗
- **THEN** 指令以非零 exit code 結束，stderr 說明封存步失敗原因；結論已寫入且不回滾，記錄仍於 openspec/discussions/；其後執行 speclink discuss archive 該 slug 成功收尾

<!-- @trace
source: conclusion-gated-discussion-archive
updated: 2026-09-01
-->

---
### Requirement: 討論定案以 search 動詞可查

speclink discuss search SHALL 接受一個以上的位置參數作為關鍵字，旗標僅 --json 與 --no-color，不吃 stdin。比對 SHALL 為不分大小寫的子字串比對，多個關鍵字任一命中即算命中。比對範圍 SHALL 限於記錄的 topic、slug 與四種決定行：各輪內以 `**Ruled out**:` 起頭的行，以及 Conclusion 區內以 `**Decision**:`、`**Rejected alternatives**:`、`**Deferred**:` 起頭的行；每個決定行 SHALL 連同其後緊接的條列行（以 `- `、`* `、`+ ` 或 `N. ` 起頭、直到第一個非條列行為止）一併參與比對，每個命中的條列行各為一筆 match，kind 與 where 同其標記行。其他行（Focus、Position、Open、Evidence 與散文）SHALL NOT 參與比對。搜尋範圍 SHALL 預設同時涵蓋在途（openspec/discussions/）與封存（openspec/discussions/archive/）記錄，無旗標可縮減。每筆命中 SHALL 帶該記錄的既有列表欄位（slug、topic、status、rounds、created、createdBy 有才出、kind 有才出、path、archived）與 matches 陣列；每個 match SHALL 帶 kind（topic、slug、ruled-out、decision、rejected、deferred 之一）、where（frontmatter、round-N 或 conclusion）與 text（該行原文去前後空白）。排序 SHALL 為 topic 或 slug 命中者排前、其餘其後；兩群內各依 created 由新到舊、同日依 slug 字典序；同一記錄的 matches 依文件順序。人眼輸出：零命中 SHALL 於 stdout 印 `No discussions match "<關鍵字以空白接起>".`；有命中 SHALL 以標題行 `Discussions matching "<關鍵字>":` 起頭，每筆一行 `  • <slug> [<status>, archived|live] (<created>) — <topic>`，其下每個 match 各一行縮排的 `<where> <kind>: <text>`；--no-color 下 SHALL 無 ANSI 色彩。--json SHALL 輸出 `{ "hits": [...] }`（欄位 camelCase），零命中為空陣列。成功（含零命中）exit code SHALL 為 0；未帶任何關鍵字 SHALL 以非零 exit code 結束、stderr 說明用法、stdout 無輸出。記錄缺輪標題或 Conclusion 區時 SHALL 仍以 topic 與 slug 參與比對，缺的區段視為無決定行，SHALL NOT 使整個查詢失敗。本動詞 SHALL NOT 寫入任何檔案。既有 discuss list 與 discuss show 的人眼與 --json 輸出 SHALL 逐位元不變。remote 模式下本動詞 SHALL 可用且人眼與 --json 輸出與本機同形（path 缺席與 promotedTo、concluded 增欄沿用 discuss list 的既定分歧）；離線、認證失效與 revision 衝突的可觀察行為 SHALL 沿既有 remote 讀取動詞的錯誤分類與訊息，不另立訊息。人眼輸出為英文，與既有 discuss 動詞一致，不隨 locale 設定變動。本動詞為 Speclink 自有延伸。

#### Scenario: 決定行命中回傳輪號與原文

- **WHEN** 某封存記錄第 2 輪含一行 `**Ruled out**: RichDetailDrawer 加 readOnly 旗標（分支地獄）`，執行 speclink discuss search drawer --json
- **THEN** exit code 0；hits 含該記錄，其 archived 為 true，matches 含一筆 kind 為 ruled-out、where 為 round-2、text 為該行原文（去前後空白）的項目

#### Scenario: 標記獨占一行時其下條列行命中

- **WHEN** 某封存記錄第 1 輪的 `**Ruled out**:` 獨占一行，其下兩行條列 `- 只在 tray.ts 修落頁` 與 `- 把 drawer 拿掉`，接著空行與 `**Open**: drawer naming`，執行 speclink discuss search drawer --json
- **THEN** 該記錄的 matches 恰含一筆 kind 為 ruled-out、where 為 round-1、text 為 `- 把 drawer 拿掉` 的項目；`**Open**:` 行不命中

#### Scenario: 非決定行不命中且零命中回空

- **WHEN** 唯一含關鍵字 sidecar 的記錄只在 Evidence 行提到它，執行 speclink discuss search sidecar 與 speclink discuss search sidecar --json
- **THEN** 人眼輸出恰為一行 `No discussions match "sidecar".`、exit code 0；--json 輸出 `{ "hits": [] }`、exit code 0

#### Scenario: 多關鍵字任一命中並依 topic 命中優先排序

- **WHEN** 記錄 A（created 2026-07-01）的 topic 含 golden、記錄 B（created 2026-08-01）只在 Conclusion 的 `**Deferred**:` 行含 SSE，執行 speclink discuss search golden sse --json
- **THEN** hits 依序為 A、B：A 的 matches 含 kind 為 topic、where 為 frontmatter 的項目；B 的 matches 含 kind 為 deferred、where 為 conclusion 的項目；大小寫差異（SSE 對 sse）不影響命中

##### Example: 排序規則

| 記錄 | 命中位置 | created | 輸出順位 |
| ---- | -------- | ------- | -------- |
| A | topic | 2026-07-01 | 1 |
| C | slug | 2026-06-01 | 2 |
| B | conclusion Deferred | 2026-08-01 | 3 |
| D | round-1 Ruled out | 2026-05-01 | 4 |

#### Scenario: 人眼輸出格式與 --no-color

- **WHEN** 對前述 A、B 兩筆命中執行 speclink discuss search golden sse --no-color
- **THEN** stdout 第一行為 `Discussions matching "golden sse":`，接著每筆一行 `  • <slug> [<status>, archived|live] (<created>) — <topic>`，其下每個 match 各一行縮排的 `<where> <kind>: <text>`；全程無 ANSI 色彩；exit code 0

#### Scenario: 未帶關鍵字

- **WHEN** 執行 speclink discuss search 不帶任何位置參數
- **THEN** 以非零 exit code 結束、stderr 說明用法、stdout 無輸出；不建立或改動任何檔案

#### Scenario: 記錄缺區段時不使查詢失敗

- **WHEN** 某在途記錄尚無任何輪與 Conclusion 內文，其 topic 含關鍵字，執行 speclink discuss search 該關鍵字 --json
- **THEN** exit code 0；該記錄出現在 hits，matches 僅含 kind 為 topic 的項目

#### Scenario: remote 模式輸出同形

- **WHEN** workspace 綁定 server 後執行 speclink discuss search drawer --json，與本機對同一組記錄執行同指令
- **THEN** 兩者 hits 的順序、每筆的 slug 與 matches 陣列相同；差異僅限 path 缺席與 promotedTo、concluded 增欄（與 discuss list 的既定分歧一致）

#### Scenario: 既有 list 與 show 輸出不變

- **WHEN** 於本變更前後對同一 workspace 執行 speclink discuss list、speclink discuss list --archived 與 speclink discuss show 某 slug（人眼與 --json）
- **THEN** stdout、stderr 與 exit code 逐位元一致

<!-- @trace
source: discuss-search-recall
updated: 2026-09-05
-->