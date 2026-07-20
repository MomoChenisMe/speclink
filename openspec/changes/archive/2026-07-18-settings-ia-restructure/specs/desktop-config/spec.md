## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定

設定 SHALL 拆分為兩頁：**專案設定頁**（跟隨 active 專案分頁）與**應用程式設定頁**（與任何專案分頁無關）。

專案設定頁 SHALL 以兩頁簽組織，標籤依序為 config.yaml、.speclink.yaml，預設 SHALL 落在 config.yaml 簽：

- **config.yaml** 簽 SHALL 含「專案說明」卡與「產出規則」卡（行為見需求「設定頁編輯專案說明與產出規則」），及「產出政策」卡——locale、spec_locale（下拉）與 tdd、audit（開關）。
- **.speclink.yaml** 簽 SHALL 含「AI 工具」卡——內建工具 claude／codex 多選，自訂工具描述子原樣呈現為不可編輯項。

應用程式設定頁 SHALL 以兩頁簽組織，標籤依序為本機設定、伺服器，預設 SHALL 落在本機設定簽：

- **本機設定** 簽 SHALL 含「介面語言」卡（行為見需求「UI 介面語言支援 zh-TW 與 en」），並 SHALL 註記其內容僅存於此裝置、不寫入版本庫。
- **伺服器** 簽行為見 desktop-connections 能力的需求「伺服器管理最小面」。

config.yaml 與 .speclink.yaml 簽首 SHALL 以等寬字註記對應檔案路徑。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，專案設定頁對應頁簽（config.yaml 簽掛工作流層、.speclink.yaml 簽掛應用層）的標籤 SHALL 帶警示點、簽內 SHALL 浮出解析失敗說明且該簽表單 SHALL 停用；應用程式設定頁 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 分頁時，專案設定頁 SHALL 整頁呈現單一不可用說明且 SHALL NOT 渲染頁簽、SHALL NOT 發出 settings 讀取呼叫；應用程式設定頁不受 active 分頁種類影響。

#### Scenario: 兩頁分工與預設簽

- **WHEN** 使用者於 local 專案分頁開啟專案設定頁與應用程式設定頁
- **THEN** 專案設定頁頁簽依序為 config.yaml、.speclink.yaml 且預設落在 config.yaml 簽（含專案說明、產出規則、產出政策三卡，簽首等寬字註記檔案路徑），切至 .speclink.yaml 簽見 AI 工具卡；應用程式設定頁頁簽依序為本機設定、伺服器且預設落在本機設定簽（含介面語言卡與「僅存於此裝置」註記）

#### Scenario: 遠端分頁的專案設定不可用說明

- **WHEN** active 分頁為遠端 workspace，使用者開啟專案設定頁
- **THEN** 整頁呈現單一不可用說明、無 config.yaml／.speclink.yaml 頁簽，且 app 未對該分頁發出 settings 讀取呼叫；切至應用程式設定頁，本機設定與伺服器兩簽照常可操作

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於專案設定頁將 tdd 切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於專案設定頁將 locale 改回「未設定（English）」並儲存
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

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於專案設定頁變更內建工具勾選並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，專案設定頁將其呈現為不可編輯項

#### Scenario: 解析失敗簽級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟專案設定頁
- **THEN** config.yaml 頁簽標籤帶警示點；切至該簽可見解析失敗說明，產出政策卡表單與專案說明、產出規則兩卡的編輯鈕停用；應用程式設定頁的介面語言三選仍可正常使用
