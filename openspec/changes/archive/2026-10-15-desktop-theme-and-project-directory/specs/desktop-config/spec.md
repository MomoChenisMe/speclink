## ADDED Requirements

### Requirement: 裝置級外觀偏好

應用程式設定的本機設定頁籤 SHALL 提供「外觀」三選：跟隨系統／淺色／深色。未設定時 SHALL 跟隨系統；選擇 SHALL 立即套用全部專案的桌面介面並保存於此裝置，重啟後 SHALL 恢復選擇，SHALL NOT 寫入任何專案或 Server 設定。外觀選項 SHALL 與介面語言同樣獨立於作用中專案，無專案時亦可操作；選中狀態 SHALL 可辨識且可由鍵盤操作。

手動選擇 SHALL 優先於系統深淺模式，系統變更 SHALL NOT 覆蓋明示選擇；跟隨系統時 SHALL 即時回應系統切換。儲存值缺席、非法或讀取失敗時 SHALL 採跟隨系統且不自動覆寫儲存值；保存失敗時 SHALL 保留原選擇與原配色、於外觀卡呈現單行錯誤，不得顯示保存成功。

介面為 zh-TW 時 SHALL 顯示「外觀／跟隨系統／淺色／深色」，en 時 SHALL 顯示「Appearance／System／Light／Dark」。工作流產出語言 tw、ja、en 或未設定 SHALL NOT 影響本偏好或 UI 詞條，SHALL NOT 新增中文弱語言偵測。

#### Scenario: 預設跟隨系統

- **WHEN** 使用者尚未設定外觀、在系統深色模式下啟動 app，之後將系統改為淺色
- **THEN** 外觀選中跟隨系統，畫面先為深色再立即改為淺色

#### Scenario: 明示選擇立即生效且重開保留

- **WHEN** 系統為深色，使用者選淺色、切換專案，再重新啟動 app
- **THEN** 畫面立即變淺，切換專案與重開後仍選中淺色且內容以淺色呈現，專案與 Server 設定均未寫入

##### Example: 系統與偏好的配色矩陣

| 儲存偏好 | 系統模式 | 畫面配色 |
| --- | --- | --- |
| 未設定或跟隨系統 | 淺色 | 淺色 |
| 未設定或跟隨系統 | 深色 | 深色 |
| 淺色 | 淺色或深色 | 淺色 |
| 深色 | 淺色或深色 | 深色 |

#### Scenario: 固定模式不受系統改變影響

- **WHEN** 使用者選深色後將系統切換為淺色，再把外觀改回跟隨系統
- **THEN** 系統切換時畫面仍深色，改回跟隨系統後才立即轉淺色

#### Scenario: 無專案與英文介面仍可設定

- **WHEN** app 沒有開啟專案、UI 語言為 en，使用者以鍵盤選擇 Dark
- **THEN** Appearance 卡顯示 System／Light／Dark，Dark 選中且畫面立即轉深色

#### Scenario: 偏好無法讀取

- **WHEN** 啟動時外觀儲存值非法或儲存讀取被拒絕
- **THEN** app 正常啟動並跟隨系統，未自動改寫該儲存值

#### Scenario: 外觀保存失敗

- **WHEN** 目前為淺色，使用者選深色但裝置偏好寫入失敗
- **THEN** 選項與畫面維持淺色，外觀卡顯示保存失敗的單行錯誤，使用者可重試

### Requirement: 專案設定顯示完整本機目錄

專案設定頁 SHALL 在頁面頂端、各設定頁籤共用的區域顯示「專案目錄」及作用中專案的完整本機路徑。本機專案 SHALL 使用已開啟的專案根目錄；遠端專案有本機副本時 SHALL 使用副本目錄，沒有時 SHALL 顯示「未連結本機目錄」。目錄 SHALL 為唯讀等寬文字、可選取複製，長路徑 SHALL 換行且保留完整文字，SHALL NOT 以省略號截斷、縮寫家目錄或提供修改位置操作。切換專案 SHALL 同步更新所顯示的路徑，SHALL NOT 保留上一專案位置。

目錄展示 SHALL 使用已知的工作區位置，SHALL NOT 新增磁碟或 Server 查詢。已有 session 的遠端專案離線時 SHALL 照常展示已知目錄；認證失效而無 session 時 SHALL 沿用既有恢復畫面，不強行進入專案設定；政策 revision 衝突 SHALL NOT 改變目錄展示或觸發額外寫入。

介面為 zh-TW 時 SHALL 顯示「專案目錄／未連結本機目錄」，en 時 SHALL 顯示「Project directory／No local directory linked」。路徑內容 SHALL 保留原始平台分隔符，SHALL NOT 隨 UI 語言或工作流 tw／ja／en／未設定而變動。

#### Scenario: 本機目錄顯示於各頁籤上方

- **WHEN** 專案根目錄為 /Users/mosu/project/ai_dev/speclink，使用者開啟專案設定並切換 config.yaml、Schema、.speclink.yaml 頁籤
- **THEN** 每個頁籤上方均可看到完整根目錄，文字可選取複製且無編輯位置操作

#### Scenario: 切換專案更新路徑

- **WHEN** 使用者從根目錄 /work/project-a 切換至 /work/project-b，檢視專案設定
- **THEN** 專案目錄顯示 /work/project-b，不再顯示 /work/project-a

#### Scenario: 長路徑完整換行

- **WHEN** 專案目錄包含中文、空白或 Windows 分隔符且寬度超過設定卡
- **THEN** 路徑完整換行，不截斷或超出卡片，選取複製得到原始完整字串

#### Scenario: 遠端有副本與無副本

- **WHEN** 使用者檢視已連結 /work/checkout-a 的遠端專案設定，再切換至沒有本機副本的遠端專案設定
- **THEN** 前者顯示 /work/checkout-a，後者顯示「未連結本機目錄」，兩者皆不顯示 Server 實體目錄

#### Scenario: 離線與衝突不影響已知目錄

- **WHEN** 遠端專案仍有 session，但處於離線或設定保存 revision 衝突狀態
- **THEN** 專案目錄維持該遠端專案的本機副本路徑或未連結提示，不發起額外位置查詢或寫入

#### Scenario: 無 session 的認證恢復

- **WHEN** 遠端專案認證失效且沒有 session，使用者前往專案設定
- **THEN** 沿用既有恢復畫面，不顯示上一專案的目錄或猜測 Server 路徑
