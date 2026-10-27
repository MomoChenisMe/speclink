## ADDED Requirements

### Requirement: 設定頁的產出流程頁籤
設定頁 SHALL 提供獨立的產出流程頁籤，頁籤標籤 SHALL 直出「Schema」（與 config.yaml、.speclink.yaml 同列的原生詞一致性，經使用者裁定為 LANGUAGE.md 明文例外；籤內使用者可見文案仍用「產出流程」）。local 頁簽序 config.yaml → Schema → .speclink.yaml；remote 頁簽序 Workflow → Schema。籤內列出每個可解析的 schema（顯示名稱、來源層級、artifact 圖），並可點入唯讀詳情——含每個 artifact 的 description、instruction 與 template 全文。詳情 SHALL 為唯讀；內建 schema 的內容不可在 desktop 編輯。清單資料 SHALL 由 desktop core 以引擎的解析函式在本地組裝，不經 server 端點。config.yaml 頁簽 SHALL NOT 含產出流程內容。

#### Scenario: 產出流程自成頁籤
- **WHEN** 使用者開啟設定頁（local 模式）
- **THEN** 頁簽依序為 config.yaml、Schema、.speclink.yaml，產出流程清單在 Schema 頁籤內呈現，config.yaml 簽內無此節

#### Scenario: 清單列出可解析的 schema
- **WHEN** 使用者開啟設定頁的產出流程頁籤（local 模式，專案有一個自訂 schema）
- **THEN** 清單顯示內建 spec-driven 與該自訂 schema，各自帶來源層級與 artifact 圖

#### Scenario: 詳情唯讀呈現內容
- **WHEN** 使用者點入 spec-driven 的詳情
- **THEN** 顯示四個 artifact 各自的 description、instruction 與 template 全文，無任何編輯入口

##### Example: 清單一列的形狀
| 欄位 | 值（內建為例） |
|------|----------------|
| 名稱 | spec-driven |
| 來源層級 | 內建 |
| artifact 圖 | proposal → design → specs → tasks |

### Requirement: 產出流程的切換寫入
產出流程頁籤 SHALL 提供下拉切換專案 schema：選定後把 schema 鍵寫入 openspec/config.yaml，寫入 SHALL 複用引擎的 byte-preserving setter（set_workflow_schema_text）——其餘內容逐位元組保留、無法解析的文件拒寫。local 模式直寫檔案；remote 模式 SHALL 走既有 revision 守門的 config 寫入通道，revision 落後時顯性失敗。切換成功後產出規則分節的固定鍵 SHALL 隨新 schema 的 artifact 圖更新——例外：產出規則正在編輯中時，編輯面 SHALL 凍結在開編輯當下的分節（草稿不因換集而丟棄或清空），固定鍵於該次編輯儲存或取消後才跟上；編輯期間換入的新固定鍵其既有規則 SHALL 在儲存時原樣保留。

#### Scenario: 切換寫入且其餘內容保留
- **WHEN** 使用者把專案 schema 從 spec-driven 切到自訂 schema
- **THEN** config.yaml 的 schema 鍵更新為該名稱，檔內其餘既有內容逐位元組不變，產出規則分節改列新 schema 的 artifact id

#### Scenario: 壞檔拒寫顯性失敗
- **WHEN** config.yaml 無法解析且使用者嘗試切換
- **THEN** 寫入被拒、錯誤浮出於表單，檔案一個位元組不變

### Requirement: 產出流程的客製 fork
產出流程頁籤 SHALL 提供 fork 動作（僅 local 模式顯示）：把選中的 schema 複製到專案 openspec/schemas/ 下，複用引擎既有的 fork 函式（複本名為引擎預設 <source>-custom，不收自訂名）；成功後清單 SHALL 即時反映新的專案層 schema。同名跨層時 fork 動作 SHALL 只出現在引擎解析會命中的那一層（project→user→內建的第一命中，含壞檔——引擎的層命中只看檔案存在）：被 shadow 的清單項不提供 fork，避免複製到前層內容。remote 模式 SHALL 不顯示 fork 動作。

#### Scenario: fork 產出專案層複本
- **WHEN** 使用者在 local 模式對 spec-driven 按下 fork
- **THEN** openspec/schemas/spec-driven-custom/ 建立（schema.yaml 與 templates 目錄），清單新增該專案層項目

#### Scenario: remote 模式無 fork 入口
- **WHEN** 工作區連線 remote store 且使用者開啟產出流程頁籤
- **THEN** 介面不出現 fork 動作

### Requirement: 產出流程的建立
產出流程頁籤 SHALL 提供建立動作（僅 local 模式顯示）：收 kebab-case 名稱，呼叫引擎既有的 init_schema 在專案 openspec/schemas/ 下產出預設骨架（schema.yaml 與 templates/ 內每個 artifact 的範本檔）；成功後清單 SHALL 即時反映新的專案層 schema。名稱驗證 SHALL 由引擎承擔（前端不重複規則）：名稱不合法或目標已存在時 SHALL 浮出引擎的錯誤訊息且磁碟不變。建立 SHALL NOT 提供 artifact 佈局輸入——骨架佈局用引擎預設，內容客製交外部編輯器。remote 模式 SHALL 不顯示建立動作。

#### Scenario: 建立產出專案層骨架
- **WHEN** 使用者在 local 模式輸入名稱 my-flow 並送出建立
- **THEN** openspec/schemas/my-flow/ 建立（schema.yaml 與 templates/ 內引擎預設 artifact 的範本檔），清單新增該專案層項目

#### Scenario: 不合法名稱顯性失敗
- **WHEN** 使用者輸入非 kebab-case 名稱（如 My Flow）並送出建立
- **THEN** 引擎的名稱錯誤訊息浮出於表單，openspec/schemas/ 無任何新目錄

##### Example: 建立輸入與結果
| 輸入名稱 | 結果 |
|----------|------|
| my-flow | openspec/schemas/my-flow/ 骨架建立，清單新增專案層項目 |
| My Flow | 拒絕：引擎 kebab-case 錯誤浮出，磁碟不變 |
| my-flow（已存在） | 拒絕：引擎 already exists 錯誤浮出，磁碟不變 |

#### Scenario: remote 模式無建立入口
- **WHEN** 工作區連線 remote store 且使用者開啟產出流程頁籤
- **THEN** 介面不出現建立動作

### Requirement: 產出流程的編輯入口
產出流程頁籤的清單項 SHALL 對有磁碟路徑的 schema（專案層與 user 層）提供「開啟所在資料夾」動作（僅 local 模式顯示）：按下後在系統檔案管理器顯示該 schema 的目錄（schema.yaml 與 templates/ 所在處），內容編輯交外部編輯器。內建 schema（內嵌於程式、無磁碟檔案）SHALL 不顯示此動作。快照的每個清單項 SHALL 帶其 schema 目錄的絕對路徑（內建為空）——user 層路徑由快照組裝端解析，前端不自行拼路徑。remote 模式 SHALL 不顯示此動作。

#### Scenario: 專案層項目開啟所在資料夾
- **WHEN** 使用者在 local 模式對建立出的專案層 schema 按下開啟所在資料夾
- **THEN** 系統檔案管理器顯示 openspec/schemas/<name>/ 目錄

#### Scenario: 內建項無編輯入口
- **WHEN** 使用者在 local 模式檢視內建 spec-driven 的清單項
- **THEN** 該項不出現開啟所在資料夾動作

### Requirement: 產出流程的刪除
產出流程頁籤 SHALL 對專案層項目提供刪除動作（僅 local 模式顯示；內建無檔案、user 層跨專案共用，均不提供）：按下 SHALL 先開確認對話框，取消 SHALL 零變動；確認後移除專案 openspec/schemas/<name>/ 整個目錄，成功後清單 SHALL 即時反映。刪除目標 SHALL 由名稱固定解析為專案層目錄（不接受任意路徑）。config 的 schema 鍵正指著的 schema（使用中）SHALL 拒刪並浮出顯性錯誤、磁碟不變。remote 模式 SHALL 不顯示刪除動作。

#### Scenario: 刪除經確認後移除專案層目錄
- **WHEN** 使用者對非使用中的專案層 schema 按刪除並在確認對話框按下確認
- **THEN** openspec/schemas/<name>/ 整個目錄移除，清單不再列出該項

#### Scenario: 取消確認零變動
- **WHEN** 使用者按刪除後在確認對話框取消
- **THEN** 磁碟與清單皆無任何變動

#### Scenario: 使用中的 schema 拒刪
- **WHEN** config.yaml 的 schema 鍵指著 my-flow 且使用者確認刪除 my-flow
- **THEN** 錯誤浮出於表單、openspec/schemas/my-flow/ 原封不動

### Requirement: remote 模式的內建限縮與誤解析修正
remote 模式下產出流程頁籤 SHALL 只列內建 schema，切換下拉的可選目標 SHALL 只含內建（config 的名稱非內建時 SHALL 以停用項顯示現值——沿政策下拉未知值顯性呈現的既有模式，不可被選取）。remote 設定快照解析 schema 名稱時 SHALL NOT 讀取 client 本機的 user 層目錄——名稱為內建即以內嵌定義解析；非內建時 SHALL 顯性呈現「遠端自訂尚不支援」的狀態而非猜測，且產出規則分節不呈現猜測的固定鍵。

#### Scenario: remote 快照不讀本機 user 層
- **WHEN** remote 專案的 config 指定 schema 名稱 X，且 client 本機 user 層目錄恰有同名 schema
- **THEN** 設定快照不以本機定義解析 X；X 非內建時產出規則分節為空並顯示遠端自訂尚不支援的狀態

##### Example: remote 解析結果表
| config 的 schema 名稱 | 本機 user 層有同名 | 快照結果 |
|-----------------------|--------------------|----------|
| spec-driven | 否 | 內建定義解析，artifact 圖正常 |
| spec-driven | 是 | 內建定義解析（本機定義不參與） |
| my-flow | 是 | 不解析，顯示遠端自訂尚不支援 |

#### Scenario: remote 下拉僅內建
- **WHEN** 工作區連線 remote store 且使用者開啟切換下拉
- **THEN** 可選的切換目標只有內建 spec-driven；config 的名稱非內建時以停用項顯示現值（沿政策下拉未知值顯性呈現的既有模式），不可被選取
