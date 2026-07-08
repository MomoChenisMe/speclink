## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定

<!-- BEFORE: 單欄卡片清單呈現兩層設定（無頁簽組織），解析失敗僅於卡內浮出橫幅 -->

設定頁 SHALL 以三頁簽組織，標籤依序為 config.yaml、.speclink.yaml、本機設定，預設 SHALL 落在 config.yaml 簽：

- **config.yaml** 簽 SHALL 含「專案說明」卡與「產出規則」卡（行為見需求「設定頁編輯專案說明與產出規則」），及「產出政策」卡——locale、spec_locale（下拉）與 tdd、audit（開關）。
- **.speclink.yaml** 簽 SHALL 含「AI 工具」卡——內建工具 claude／codex 多選，自訂工具描述子原樣呈現為不可編輯項。
- **本機設定** 簽 SHALL 含「介面語言」卡（行為見需求「UI 介面語言支援 zh-TW 與 en」）。

config.yaml 與 .speclink.yaml 簽首 SHALL 以等寬字註記對應檔案路徑；本機設定簽 SHALL 註記其內容僅存於此裝置、不寫入版本庫。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，對應頁簽（config.yaml 簽掛工作流層、.speclink.yaml 簽掛應用層）的標籤 SHALL 帶警示點、簽內 SHALL 浮出解析失敗說明且該簽表單 SHALL 停用；本機設定簽 SHALL NOT 受任何解析失敗影響。

#### Scenario: 三頁簽組織與預設簽

- **WHEN** 使用者開啟設定頁
- **THEN** 頁簽依序為 config.yaml、.speclink.yaml、本機設定且預設落在 config.yaml 簽（含專案說明、產出規則、產出政策三卡，簽首等寬字註記檔案路徑）；切至 .speclink.yaml 簽見 AI 工具卡；切至本機設定簽見介面語言卡與「僅存於此裝置」註記

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

#### Scenario: 解析失敗簽級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟設定頁並停留在本機設定簽
- **THEN** config.yaml 頁簽標籤帶警示點；切至該簽可見解析失敗說明，產出政策卡表單與專案說明、產出規則兩卡的編輯鈕停用；本機設定簽的介面語言三選仍可正常使用

### Requirement: 設定頁編輯專案說明與產出規則

<!-- BEFORE: 單一「專案設定」卡含專案說明／產出規則兩內層分頁，卡層級共享編輯態、一次儲存同時寫 context 與 rules -->

設定頁 config.yaml 頁簽 SHALL 呈現「專案說明」與「產出規則」兩張獨立卡，各卡預設唯讀、各持編輯態——一卡進入編輯 SHALL NOT 影響另一卡的唯讀呈現與可用性。專案說明唯讀 SHALL 以 markdown 渲染呈現 context 內容，超過固定高度 SHALL 收合並提供顯示更多展開；未設定時 SHALL 顯示空狀態提示。產出規則唯讀 SHALL 僅列出有條目的 artifact 鍵，鍵名為小節標題、條目為清單。各卡右上編輯鈕 SHALL 就地切換該卡為編輯態並將按鈕列改為取消與儲存：專案說明為 raw markdown 多行文字區；產出規則 SHALL 以活躍 schema 的 artifact id 為固定鍵各渲染一個多行文字區，SHALL NOT 提供自由鍵輸入；一行即一條規則，行序 SHALL 即為寫入檔案的條目順序（亦即指令注入順序）。儲存時 SHALL 逐行去除頭尾空白，空行 SHALL NOT 寫入；專案說明清空儲存 SHALL 移除 context 鍵，某鍵文字區清空 SHALL 移除該 artifact 鍵，全部鍵皆空 SHALL 移除 rules 鍵。專案說明卡儲存 SHALL 僅寫入 context、產出規則卡儲存 SHALL 僅寫入 rules——各卡儲存 SHALL NOT 改動另一卡對應的鍵。取消 SHALL 還原該卡唯讀呈現且 SHALL NOT 寫入檔案。寫入 SHALL 經與政策欄位相同的雙重解析驗證流程；序列化 SHALL 為以 YAML 保留字元（反引號、at 符號等）開頭的條目自動加引號——寫出檔案 SHALL 可被引擎解析且條目值逐字元還原。config.yaml 解析失敗時，兩卡 SHALL 依所在頁簽的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。

#### Scenario: 唯讀優先與各卡就地編輯

- **WHEN** 使用者開啟設定頁的 config.yaml 簽
- **THEN** 專案說明卡以 markdown 渲染唯讀（超長時收合並可顯示更多）、產出規則卡僅列有條目的鍵；點專案說明卡的編輯後僅該卡就地變為可編輯（按鈕列為取消與儲存），產出規則卡維持唯讀且仍可獨立進入自身的編輯態

#### Scenario: 各卡儲存僅寫對應鍵

- **WHEN** config.yaml 原含 context 與 rules，使用者僅於產出規則卡進入編輯、修改後儲存
- **THEN** 重新讀取 config.yaml 的 rules 依編輯更新，而 context 值與寫入前逐字元一致

#### Scenario: 編輯專案說明並儲存

- **WHEN** 使用者於專案說明卡進入編輯，於文字區輸入多行 markdown 並儲存
- **THEN** 重新讀取 config.yaml 解析出的 context 值與輸入逐字元一致、其餘鍵原樣保留，該卡回唯讀並渲染新內容，重開設定頁呈現同一文字

#### Scenario: 以保留字元開頭的規則條目寫入後仍可解析

- **WHEN** 使用者於產出規則卡某鍵文字區新增一行以 YAML 保留字元開頭的規則並儲存
- **THEN** 寫出的 config.yaml 可被引擎解析（必要引號由寫入自動加上）、該條目值逐字元還原，整份工作流政策未退回預設

##### Example: 保留字元條目自動加引號

- **GIVEN** rules 原含 proposal 節一條「提案必須列出影響的 crates」
- **WHEN** 於 tasks 文字區新增一行「@完成後執行全部測試」並儲存
- **THEN** 重讀 config.yaml 可解析，rules 的 tasks 節含值「@完成後執行全部測試」（逐字元一致），proposal 節與 schema 等其餘鍵原樣保留

#### Scenario: 清空即移除鍵

- **WHEN** 使用者於專案說明卡清空文字區全部行並儲存，再於產出規則卡清空某鍵文字區並儲存
- **THEN** 重新讀取 config.yaml 已無 context 鍵與該 artifact 鍵；其餘鍵原樣保留

##### Example: 鍵移除語意

| 操作前檔案狀態 | 編輯操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| context: 舊說明、rules 含 tasks 兩行 | 專案說明卡清空並儲存 | context 鍵被移除，rules.tasks 原樣保留 |
| rules 含 proposal 與 tasks 兩節 | 產出規則卡清空 tasks 文字區並儲存 | rules 僅餘 proposal 節 |
| rules 僅含 tasks 一節 | 產出規則卡清空該文字區並儲存 | rules 鍵整個被移除 |

#### Scenario: 行序即寫入順序

- **WHEN** 使用者於產出規則卡某鍵文字區將第二行搬至第一行並儲存
- **THEN** 重新讀取 config.yaml 該節條目順序對調，後續該 artifact 的指令注入依新順序呈現規則

##### Example: 行對調

- **GIVEN** tasks 節依序含條目「先寫失敗測試」「更新文件」
- **WHEN** 於 tasks 文字區將「更新文件」一行搬到「先寫失敗測試」之前並儲存
- **THEN** config.yaml 的 tasks 節依序為「更新文件」「先寫失敗測試」

#### Scenario: 固定鍵分節不可自由輸入

- **WHEN** 使用者於使用 spec-driven schema 的專案進入產出規則卡編輯
- **THEN** 文字區恰為 proposal、design、specs、tasks 四個固定鍵各一，介面不提供自由新增分節鍵的輸入；回唯讀後僅有條目的鍵被列出

#### Scenario: 取消放棄編輯

- **WHEN** 使用者於專案說明卡進入編輯、修改內容後點取消
- **THEN** 該卡還原為編輯前的唯讀呈現，config.yaml 逐字元未變，且產出規則卡全程不受影響

#### Scenario: 解析失敗停用編輯

- **WHEN** config.yaml 被外部改壞為無法解析後使用者開啟設定頁
- **THEN** config.yaml 簽浮出解析失敗說明，專案說明卡與產出規則卡的編輯鈕停用，不提供任何寫入途徑
