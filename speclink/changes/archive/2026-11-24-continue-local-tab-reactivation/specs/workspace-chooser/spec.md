## MODIFIED Requirements

### Requirement: remote marker 資料夾的探測分流

專案探測 SHALL 辨識 remote marker。資料夾僅含 marker 且對應 connection 已登入時，若 `.speclink.yaml` 有至少一個有效 built-in tool，Desktop SHALL 先依該選集 reconciliation，成功後才 handshake 開啟 remote tab並以呼叫端原始 path 作為 checkoutRoot；同步失敗 SHALL 顯示錯誤且不建立 session。marker 缺少有效 built-in tool 選集時 SHALL 導向 chooser 的 checkout 步驟、預填 server／scope／path 並要求明示選擇 Claude／Codex／Copilot／Pi／Cursor，不得直接開啟。無對應 connection 或未登入 SHALL 引導至 chooser 的 server 步驟並預填 server 位址。

marker 與本地 `speclink/` 並存時 SHALL 停下強制選擇，提供三個出口且皆無靜默覆蓋：「繼續本地」（以本地開啟、marker 不動）；「以 server 為準」（本地 `speclink/` 改名為帶日期備份後，資料夾轉為 checkout，完成工具 reconciliation 後開啟 remote tab，不上傳本地內容、不改動 server）；「遷移本地內容」（進入 workspace-migration 的遷移流程、目標為空 scope）。對話文案 SHALL 明說「以 server 為準」為備份後棄用本地、非合併。marker YAML 損壞 SHALL 沿 `.speclink.yaml` 既有 fail-closed 語意呈現錯誤。

「繼續本地」開出的本地分頁 SHALL 延續這個選擇：之後切回這個分頁、以快捷鍵或系統匣切換到它，以及 app 重開時（這個分頁是上次作用中的分頁，或在背景接受啟動時的路徑有效性探測），資料夾仍是本地 `speclink/` 與 marker 並存者 SHALL 照樣以本地開啟並照常計算背景分頁的待收尾數，SHALL NOT 轉錯誤態、SHALL NOT 再次停下強制選擇、SHALL NOT 發出 handshake。這個延續只適用於分頁路徑本身就是並存專案根的分頁：分頁路徑是並存專案根底下的子目錄，或探索命中的專案根不是分頁路徑時，本地分頁 SHALL 轉錯誤態，SHALL NOT 開出上層專案的內容。本地規格資料夾已不存在、只剩 marker 時，本地分頁 SHALL 照舊轉錯誤態。經「新增專案」或最近開啟清單再次開啟同一個並存資料夾時，SHALL 照舊停下強制選擇。

#### Scenario: 有工具選集的 checkout 直達 remote tab

- **WHEN** 開啟僅含 remote marker 的資料夾、built-in tools 為 `[codex]` 且對應 server 已登入
- **THEN** Desktop 先補齊或更新 Codex 受管產物，成功後不經 chooser 完成 handshake，remote tab 開啟且 checkoutRoot 為該資料夾

#### Scenario: 缺少工具選集時導向 checkout 選擇

- **WHEN** 開啟僅含 remote marker 的資料夾、tools 缺席或不含任何內建工具（claude、codex、copilot、pi、cursor），且對應 server 已登入
- **THEN** Desktop 導向 chooser checkout 步驟並預填原 path 與 scope，內建工具選集需由使用者確認，且提交成功前不建立 remote tab／session

#### Scenario: 自動補齊失敗不 handshake

- **WHEN** 有 built-in tools 的既有 checkout 在 reconciliation 時遇到檔案系統錯誤
- **THEN** Desktop 顯示帶路徑與失敗階段的錯誤，remote tab／session 不建立且不發出 handshake

#### Scenario: 並存衝突三出口

- **WHEN** 開啟同時含本地 `speclink/` 與 remote marker 的資料夾
- **THEN** 呈現強制選擇對話含三出口：繼續本地以本地開啟；以 server 為準將本地改名備份後執行工具 reconciliation 再開 remote tab且 server 內容未變；遷移本地內容進入遷移流程；無任何自動覆蓋

#### Scenario: 繼續本地的分頁切回仍以本地開啟

- **WHEN** 對同時含本地 `speclink/` 與 remote marker 的資料夾選「繼續本地」，切到另一個專案後再點回這個分頁的方塊
- **THEN** 這個分頁成為作用中並顯示本地看板，方塊沒有錯誤態，沒有出現強制選擇對話，也沒有發出 handshake

#### Scenario: 重開 app 後繼續本地的分頁不轉錯誤態

- **WHEN** 圖示列有一個「繼續本地」開出的並存資料夾分頁，關掉 app 再開；分別試這個分頁是上次作用中的分頁，與它在背景兩種情況
- **THEN** 兩種情況下這個分頁都沒有錯誤態：上次作用中時以本地看板開啟；在背景時方塊照常呈現，點下去以本地開啟

#### Scenario: 本地規格資料夾移除後切回轉錯誤態

- **WHEN** 「繼續本地」開出的分頁所在資料夾，本地 `speclink/` 被刪除、只剩 remote marker，之後切回這個分頁
- **THEN** 方塊轉為錯誤態，提示「已不是 speclink 專案」，不發出 handshake

#### Scenario: 並存專案子目錄的分頁切回轉錯誤態

- **WHEN** 「繼續本地」開出的分頁路徑是並存專案根底下的子目錄（或是巢狀專案：它的本地 `speclink/` 已刪除，而上層資料夾是並存專案），之後切回這個分頁
- **THEN** 方塊轉為錯誤態，提示「已不是 speclink 專案」，不開出上層專案的內容，不發出 handshake

#### Scenario: 再次開啟並存資料夾仍強制選擇

- **WHEN** 圖示列已有「繼續本地」開出的並存資料夾分頁，使用者再經「新增專案」選取同一個資料夾
- **THEN** Desktop 照舊停下強制選擇並呈現三出口，選擇前不切換作用中專案

#### Scenario: 壞 marker fail-closed

- **WHEN** 選取的 checkout 具有無法解析的 `.speclink.yaml`
- **THEN** Desktop 顯示解析錯誤，不修改資料夾、不建立 tab／session且不發出 handshake

#### Scenario: 只選 Copilot 的 checkout 直達 remote tab

- **WHEN** 開啟僅含 remote marker 的資料夾、built-in tools 為 `[copilot]` 且對應 server 已登入
- **THEN** Desktop 先補齊或更新 `.github/skills/` 下的 Copilot 受管產物，成功後不經 chooser 完成 handshake，remote tab 開啟且 checkoutRoot 為該資料夾

#### Scenario: 只選 Pi 的 checkout 直達 remote tab

- **WHEN** 開啟僅含 remote marker 的資料夾、built-in tools 為 `[pi]` 且對應 server 已登入
- **THEN** Desktop 先補齊或更新 `.pi/skills/` 下的 Pi 受管產物，成功後不經 chooser 完成 handshake，remote tab 開啟且 checkoutRoot 為該資料夾
