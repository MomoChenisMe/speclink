## MODIFIED Requirements

### Requirement: remote marker 資料夾的探測分流

專案探測 SHALL 辨識 remote marker：資料夾僅含 marker 時——對應 connection 已登入即以 handshake 開啟 remote 分頁並以該資料夾為 checkoutRoot；無對應 connection 或未登入即引導至 chooser 的 server 步驟並預填 server 位址。marker 與本地 openspec/ 並存時 SHALL 停下強制選擇，提供三個出口且皆無靜默覆蓋：「繼續本地」（本次以本地開啟、marker 不動）；「以 server 為準」（本地 openspec/ 改名為帶日期備份後，資料夾轉為 checkout 開啟 remote 分頁——不上傳本地內容、不改動 server）；「遷移本地內容」（進入 workspace-migration 能力的遷移流程、目標為空 scope）。對話文案 SHALL 明說「以 server 為準」為備份後棄用本地、非合併。marker YAML 損壞 SHALL 沿 .speclink.yaml 既有 fail-closed 語意呈現錯誤。

#### Scenario: RD 重開 checkout 直達 remote 分頁

- **WHEN** 開啟僅含 remote marker 的資料夾且對應 server 已登入
- **THEN** 不經 chooser 步驟，handshake 後 remote 分頁開啟且 checkoutRoot 為該資料夾

#### Scenario: 並存衝突三出口

- **WHEN** 開啟同時含本地 openspec/ 與 remote marker 的資料夾
- **THEN** 呈現強制選擇對話含三出口：繼續本地以本地開啟；以 server 為準將本地改名備份後轉 checkout 開 remote 分頁且 server 內容未變；遷移本地內容進入遷移流程；無任何自動覆蓋
