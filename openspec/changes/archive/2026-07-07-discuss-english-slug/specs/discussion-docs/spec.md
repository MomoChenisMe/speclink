## ADDED Requirements

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

### Requirement: 討論技能指示要求英文 slug

speclink 生成的 discuss 技能內容 SHALL 指示 agent 在建立討論記錄時,從主題自行衍生英文 kebab-case slug 並以 --slug 傳入;主題(topic)維持使用者語言原文。內嵌技能資產與 render golden 基準 SHALL 同步反映此指示。

#### Scenario: 生成的技能含 --slug 指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 discuss 技能
- **THEN** 生成的技能檔內容包含「衍生英文 kebab-case slug 並以 --slug 傳入」的指示,render golden 測試以更新後基準通過
