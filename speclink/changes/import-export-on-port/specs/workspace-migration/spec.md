## MODIFIED Requirements

### Requirement: import 端點僅限 CreateNew 且原子
<!-- BEFORE: 只接受以文件種類標示的包；由 server 直接呼叫儲存層的匯入；不帶事件 -->

server SHALL 提供綁定 scope 的 import 端點：接受 wire Bundle（format version、文件全集；以文件種類標示的第一版與以鍵標示的第二版皆可），模式固定 CreateNew——wire SHALL NOT 暴露 Overwrite。端點 SHALL 由宿主套件以儲存接口完成，示範用 Server 與內嵌引擎的宿主 SHALL 行為相同。目標 scope 已持有任何文件時 SHALL 拒絕且不寫入任何內容；reader SHALL 拒於 403；未知 format version 或缺欄位 SHALL 拒收。成功時整包 SHALL 以單一原子提交落庫並回報 project revision 與逐文件結果（第二版回報份數）——部分寫入狀態 SHALL NOT 存在；該次提交 SHALL 帶一筆匯入的事件，使已連線的用戶端得知範圍已變動。本文超過宿主設定的上限時 SHALL 以 413 拒絕。

#### Scenario: 空 scope 匯入後全面可讀

- **WHEN** 對空 scope 匯入含 changes、canonical specs、討論與 archived 的 Bundle
- **THEN** 回報逐文件 Created，隨後清單、文件、討論與 archived 各端點回傳與 Bundle 一致的內容

#### Scenario: 非空 scope 拒絕且零寫入

- **WHEN** 對已持有文件的 scope 呼叫 import
- **THEN** 回應衝突錯誤且訊息含 create-new 語意，scope 內容與 revision 皆未改變

#### Scenario: 匯入後已連線的用戶端收到事件

- **WHEN** 一個用戶端訂閱著某個空範圍的事件串流，另一個請求對該範圍匯入成功
- **THEN** 訂閱者收到一筆 invalidation

#### Scenario: 對內嵌引擎的宿主匯入

- **WHEN** 以原版的用戶端對一個用宿主套件服務的空範圍送出匯入請求
- **THEN** 匯入成功，之後的讀取面回傳包的內容

### Requirement: Bundle 組裝涵蓋本地 workspace 全集
<!-- BEFORE: 組裝的內容不含 evidence 與看板順序 -->

desktop SHALL 以引擎讀取面把本地 workspace 組裝為 Bundle：active changes 的 meta 與全部 artifacts、canonical specs、live 與 archived 討論、archived changes 的全部文件（含其 evidence）、active changes 的 evidence、workflow config、共用詞彙文件，以及由各變更與各討論的看板順序鍵組成的看板順序文件（沒有任何順序鍵時不含這份文件）。desktop SHALL 以鍵標示文件的第二版送出；Server 不認得第二版時 SHALL 以第一版重送同一批內容。任一來源檔解析失敗 SHALL 中止組裝並點名該檔——SHALL NOT 部分遷移。

#### Scenario: 全集往返一致

- **WHEN** 對含各類文件的本地 workspace 組裝 Bundle 並匯入空 scope，再自 server 逐端點讀回
- **THEN** 每一份文件內容與本地原檔一致，無缺漏類別

#### Scenario: evidence 隨遷移保留

- **WHEN** 本地 workspace 的變更 demo 有兩筆 evidence 記錄，遷移到空的遠端範圍
- **THEN** 遠端 demo 的 evidence 讀出同樣的兩筆記錄

#### Scenario: 看板順序隨遷移保留

- **WHEN** 本地看板上同一欄的 a、b、c 依序排列（各有順序鍵），遷移到空的遠端範圍
- **THEN** 遠端的 plan 與看板以 a、b、c 的順序呈現這三個變更

#### Scenario: 對舊 Server 退回第一版

- **WHEN** 遷移的目標 Server 只認得第一版的包
- **THEN** 遷移仍然成功
