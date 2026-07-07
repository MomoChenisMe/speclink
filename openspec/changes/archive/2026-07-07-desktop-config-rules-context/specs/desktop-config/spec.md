## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定

<!-- BEFORE: 未觸及鍵保留名單含 rules 與 context（僅原樣保留、不可經 GUI 編輯）；設定頁表單不含此二欄位 -->

設定頁 SHALL 以結構化表單呈現並寫入兩層設定：.speclink.yaml 的 tools（內建工具 claude／codex 多選）與 openspec/config.yaml 的 locale、spec_locale（下拉）、tdd、audit（開關），以及 context 與 rules（「專案說明」與「產出規則」區段，行為見需求「設定頁編輯專案說明與產出規則」）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 於表單原樣呈現為不可編輯項且寫入後保留。

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於設定頁將 tdd 切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於設定頁將 locale 改回「未設定（English）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 為預設 English

##### Example: 政策欄位寫入效果

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true |
| tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false） |
| locale: tw、含 rules 區塊 | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留 |

#### Scenario: tools 變更後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者加選 codex 並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 AGENTS.md marker 區塊與 .agents/skills/ 技能檔

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於設定頁變更內建工具勾選並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，設定頁將其呈現為不可編輯項

## ADDED Requirements

### Requirement: 設定頁編輯專案說明與產出規則

設定頁 SHALL 提供「專案說明」區段（多行文字區，呈現與寫入 config.yaml 的 context）與「產出規則」區段（呈現與寫入 rules）。產出規則 SHALL 以活躍 schema 的 artifact id 為固定鍵分節，SHALL NOT 提供自由鍵輸入；每節為條目清單，SHALL 支援新增、編輯、刪除與排序，清單順序 SHALL 即為寫入檔案的條目順序（亦即指令注入順序）。條目存入前 SHALL 去除頭尾空白，空字串條目 SHALL NOT 寫入。專案說明清空儲存 SHALL 移除 context 鍵；某節清單清空 SHALL 移除該 artifact 鍵；全部節皆空 SHALL 移除 rules 鍵。寫入 SHALL 經與政策欄位相同的雙重解析驗證流程；序列化 SHALL 為以 YAML 保留字元（反引號、at 符號等）開頭的條目自動加引號——寫出檔案 SHALL 可被引擎解析且條目值逐字元還原。config.yaml 解析失敗時，兩區段 SHALL 隨該檔表單一併停用。

#### Scenario: 編輯專案說明並儲存

- **WHEN** 使用者於「專案說明」輸入多行文字並儲存
- **THEN** 重新讀取 config.yaml 解析出的 context 值與輸入逐字元一致、其餘鍵原樣保留，重開設定頁呈現同一文字

#### Scenario: 以保留字元開頭的規則條目寫入後仍可解析

- **WHEN** 使用者於「產出規則」某節新增以 YAML 保留字元開頭的條目並儲存
- **THEN** 寫出的 config.yaml 可被引擎解析（必要引號由寫入自動加上）、該條目值逐字元還原，整份工作流政策未退回預設

##### Example: 保留字元條目自動加引號

- **GIVEN** rules 原含 proposal 節一條「提案必須列出影響的 crates」
- **WHEN** 於 tasks 節新增條目「@完成後執行全部測試」並儲存
- **THEN** 重讀 config.yaml 可解析，rules 的 tasks 節含值「@完成後執行全部測試」（逐字元一致），proposal 節與 schema 等其餘鍵原樣保留

#### Scenario: 清空即移除鍵

- **WHEN** 使用者清空「專案說明」文字並將某節條目全數刪除後儲存
- **THEN** 重新讀取 config.yaml 已無 context 鍵與該 artifact 鍵；其餘節原樣保留

##### Example: 鍵移除語意

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| context: 舊說明、rules 含 tasks 兩條 | 清空專案說明 | context 鍵被移除，rules.tasks 原樣保留 |
| rules 含 proposal 與 tasks 兩節 | 刪除 tasks 節全部條目 | rules 僅餘 proposal 節 |
| rules 僅含 tasks 一節 | 刪除該節全部條目 | rules 鍵整個被移除 |

#### Scenario: 排序即寫入順序

- **WHEN** 使用者以排序操作將某節第二條目移至第一並儲存
- **THEN** 重新讀取 config.yaml 該節條目順序對調，後續該 artifact 的指令注入依新順序呈現規則

##### Example: 條目對調

- **GIVEN** tasks 節依序含條目「先寫失敗測試」「更新文件」
- **WHEN** 將「更新文件」上移一位並儲存
- **THEN** config.yaml 的 tasks 節依序為「更新文件」「先寫失敗測試」

#### Scenario: 固定鍵分節不可自由輸入

- **WHEN** 使用者於使用 spec-driven schema 的專案檢視「產出規則」區段
- **THEN** 分節恰為 proposal、design、specs、tasks 四節，且介面不提供自由新增分節鍵的輸入
