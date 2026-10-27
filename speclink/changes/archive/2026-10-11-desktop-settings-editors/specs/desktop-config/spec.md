## MODIFIED Requirements

### Requirement: 設定頁編輯專案說明與產出規則

<!-- REMOVED-SCENARIO: 唯讀優先與各卡就地編輯 -->
<!-- REMOVED-SCENARIO: 各卡儲存僅寫對應鍵 -->


專案設定的「專案說明」頁與「產出規則」頁各 SHALL 呈現一張卡，各頁預設唯讀、各持編輯態——一頁進入編輯 SHALL NOT 影響另一頁的唯讀呈現與可用性。專案說明唯讀 SHALL 以 markdown 渲染呈現 context 全文，SHALL NOT 收合（專案說明自成一頁，收合只會藏起該頁唯一的內容）；未設定時 SHALL 顯示空狀態提示。產出規則唯讀 SHALL 僅列出有條目的 artifact 鍵，鍵名為小節標題、條目為清單。各頁頁標題右側的編輯鈕 SHALL 切換該頁為編輯態並將按鈕列改為取消與儲存（儲存鈕標示 ⌘S／Ctrl+S，該快捷鍵於編輯器內與頁內皆 SHALL 觸發儲存）。兩頁 SHALL 共用同一個程式碼編輯器元件：行號、自動換行、配色取自設計 token、底部狀態列（模式、自動換行、第 N 行第 M 欄）。專案說明為 markdown 模式（標題、行內 code、連結、強調上色），頁標題右側另 SHALL 有「編輯／預覽」分段切換：預覽以共用 Markdown 元件渲染目前草稿，切回編輯 SHALL 保留草稿與游標；儲存 SHALL 把編輯器內容原樣寫入 context——SHALL NOT 去除頭尾空白、SHALL NOT 增刪檔尾換行、SHALL NOT 改動縮排（全為空白時仍視為清空）。產出規則 SHALL 以活躍 schema 的 artifact id 為固定鍵各渲染一張小卡（等寬鍵名＋即時的「N 條」計數）內含純文字模式編輯器，SHALL NOT 提供自由鍵輸入；一行即一條規則，行序 SHALL 即為寫入檔案的條目順序（亦即指令注入順序）。產出規則儲存時 SHALL 逐行去除頭尾空白，空行 SHALL NOT 寫入；專案說明清空儲存 SHALL 移除 context 鍵，某鍵編輯器清空 SHALL 移除該 artifact 鍵，全部鍵皆空 SHALL 移除 rules 鍵。專案說明頁儲存 SHALL 僅寫入 context、產出規則頁儲存 SHALL 僅寫入 rules——各頁儲存 SHALL NOT 改動另一頁對應的鍵。取消 SHALL 還原該頁唯讀呈現且 SHALL NOT 寫入檔案。寫入 SHALL 經與政策欄位相同的雙重解析驗證流程；序列化 SHALL 為以 YAML 保留字元（反引號、at 符號等）開頭的條目自動加引號——寫出檔案 SHALL 可被引擎解析且條目值逐字元還原。config.yaml 解析失敗時，兩頁 SHALL 依分類的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。

#### Scenario: 唯讀優先與各頁獨立編輯

- **WHEN** 使用者開啟專案設定的專案說明頁
- **THEN** 專案說明以 markdown 渲染唯讀（超長時仍呈現全文、無「顯示更多」）；點頁標題右側的編輯後內容卡換成帶行號的 markdown 編輯器、按鈕列為「編輯／預覽」切換、取消與儲存 ⌘S；切到產出規則頁，該頁維持唯讀、只列有條目的鍵，且仍可獨立進入自身的編輯態

#### Scenario: 各頁儲存僅寫對應鍵

- **WHEN** config.yaml 原含 context 與 rules，使用者僅於產出規則頁進入編輯、修改後儲存
- **THEN** 重新讀取 config.yaml 的 rules 依編輯更新，而 context 值與寫入前逐字元一致

#### Scenario: 編輯專案說明並儲存

- **WHEN** 使用者於專案說明頁進入編輯，於編輯器輸入多行 markdown（含兩層縮排的清單與檔尾換行）並按 ⌘S
- **THEN** 重新讀取 config.yaml 解析出的 context 值與輸入逐字元一致（縮排與檔尾換行皆保留）、其餘鍵原樣保留，該頁回唯讀並渲染新內容，重開專案設定呈現同一文字

#### Scenario: 以保留字元開頭的規則條目寫入後仍可解析

- **WHEN** 使用者於產出規則頁某鍵編輯器新增一行以 YAML 保留字元開頭的規則並儲存
- **THEN** 寫出的 config.yaml 可被引擎解析（必要引號由寫入自動加上）、該條目值逐字元還原，整份工作流政策未退回預設

##### Example: 保留字元條目自動加引號

- **GIVEN** rules 原含 proposal 節一條「提案必須列出影響的 crates」
- **WHEN** 於 tasks 編輯器新增一行「@完成後執行全部測試」並儲存
- **THEN** 重讀 config.yaml 可解析，rules 的 tasks 節含值「@完成後執行全部測試」（逐字元一致），proposal 節與 schema 等其餘鍵原樣保留

#### Scenario: 清空即移除鍵

- **WHEN** 使用者於專案說明頁清空編輯器全部行並儲存，再於產出規則頁清空某鍵編輯器並儲存
- **THEN** 重新讀取 config.yaml 已無 context 鍵與該 artifact 鍵；其餘鍵原樣保留

##### Example: 鍵移除語意

| 操作前檔案狀態 | 編輯操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| context: 舊說明、rules 含 tasks 兩行 | 專案說明頁清空並儲存 | context 鍵被移除，rules.tasks 原樣保留 |
| rules 含 proposal 與 tasks 兩節 | 產出規則頁清空 tasks 編輯器並儲存 | rules 僅餘 proposal 節 |
| rules 僅含 tasks 一節 | 產出規則頁清空該編輯器並儲存 | rules 鍵整個被移除 |

#### Scenario: 行序即寫入順序

- **WHEN** 使用者於產出規則頁某鍵編輯器將第二行搬至第一行並儲存
- **THEN** 重新讀取 config.yaml 該節條目順序對調，後續該 artifact 的指令注入依新順序呈現規則

##### Example: 行對調

- **GIVEN** tasks 節依序含條目「先寫失敗測試」「更新文件」
- **WHEN** 於 tasks 編輯器將「更新文件」一行搬到「先寫失敗測試」之前並儲存
- **THEN** config.yaml 的 tasks 節依序為「更新文件」「先寫失敗測試」

#### Scenario: 固定鍵分節不可自由輸入

- **WHEN** 使用者於使用 spec-driven schema 的專案進入產出規則頁的編輯
- **THEN** 編輯器小卡恰為 proposal、design、specs、tasks 四個固定鍵各一（各帶「N 條」計數），介面不提供自由新增分節鍵的輸入；回唯讀後僅有條目的鍵被列出

#### Scenario: 取消放棄編輯

- **WHEN** 使用者於專案說明頁進入編輯、修改內容後點取消
- **THEN** 該頁還原為編輯前的唯讀呈現，config.yaml 逐字元未變，且產出規則頁全程不受影響

#### Scenario: 解析失敗停用編輯

- **WHEN** config.yaml 被外部改壞為無法解析後使用者開啟專案設定
- **THEN** 專案說明頁與產出規則頁浮出解析失敗說明、編輯鈕停用，不提供任何寫入途徑

#### Scenario: 編輯與預覽切換保留草稿

- **WHEN** 使用者於專案說明頁編輯中輸入三行草稿、切到「預覽」再切回「編輯」
- **THEN** 預覽以 markdown 渲染該三行；切回後編輯器內容仍為該三行且游標位置不變；期間 config.yaml 未被寫入

#### Scenario: 產出規則條數即時計數

- **WHEN** 使用者於產出規則頁的 tasks 編輯器有兩行規則與一個空行，再新增一行
- **THEN** tasks 小卡的計數由「2 條」變為「3 條」（空行不計）；儲存後 config.yaml 的 tasks 節恰為三條
