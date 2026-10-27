## 1. PROBE 四態（TDD）

- [x] 1.1 紅（規格「remote marker 資料夾的探測分流」；design「決策 3：PROBE 擴充為四態」）：apps/desktop/core 的 probe 單元測試——僅 remote marker 回 remoteBinding{url, repo, hasLocalOpenspec:false}、marker＋openspec/ 並存回 hasLocalOpenspec:true、無 marker 維持既有 project／uninitialized 兩態、marker YAML 損壞沿 .speclink.yaml 既有 fail-closed 語意。cargo test -p speclink-desktop-core 確認新案例全紅。 <!-- speclink-task:tsk_01KXYYTF126VJXHGZ51JGV828C -->
- [x] 1.2 綠：apps/desktop/core/src/project.rs 的 probe 擴充讀 remote section；apps/desktop/src-tauri/src/lib.rs 的 open_project payload 與 apps/desktop/src/adapter/workspace.ts 的 ProjectProbe 型別同步四態。1.1 全綠、既有 workspace 測試不回歸。 <!-- speclink-task:tsk_01KXYYTF122JX7H8X2QNV64CRH -->

## 2. checkout 驗證與 marker 寫入（TDD）

- [x] 2.1 紅（規格「checkout 綁定驗證與 marker 寫入」；design「決策 2：checkout 驗證＝marker 一致性」）：src-tauri 或 desktop-core 測試——marker 一致綁定成功、origin 不一致與 repo 不一致各自拒絕且訊息指出 marker 指向、無 marker 的 git repo 綁定後 marker 寫入且內容與 CLI 的 remote 初始化同形、非 git repo 拒絕。確認全紅。 <!-- speclink-task:tsk_01KXYYTF12TQ4M1WJ9BP4GZT32 -->
- [x] 2.2 綠：實作 checkout 驗證命令（marker 讀取比對＋speclink-core 的 write_remote_section 寫入）；斷言寫入後 CLI 於該資料夾可解析出同一 origin/repo（同構互通）。2.1 全綠。 <!-- speclink-task:tsk_01KXYYTF12W2NVKGTWNMM1RPZ2 -->

## 3. chooser 元件與入口整合

- [x] 3.1（規格「新增 Workspace 的來源分流」「scopes 清單選擇取代文字輸入」；design「決策 1：chooser 多步對話與入口整合」）：新增 apps/desktop/src/components/WorkspaceChooser.tsx——來源分流、connections 清單與就地新增回流、/scopes 的 Project 分組 Repos 單選（無 membership 空清單附繁中說明）、checkout 分流（略過｜選資料夾）、開啟；apps/desktop/src/adapter/connections.ts 補 scopes invoke、apps/desktop/src-tauri/src/remote.rs 補 scopes 命令。新增 apps/desktop/src/__tests__/workspaceChooser.test.tsx 假 adapter 覆蓋全流程與空清單。全綠。 <!-- speclink-task:tsk_01KXYYTF12CHE234BJQSJ8VTVT -->
- [x] 3.2 入口整合：apps/desktop/src/App.tsx（頂列與空狀態）、apps/desktop/src/components/ProjectTabs.tsx（加號）改開 chooser；apps/desktop/src/components/ServersPanel.tsx 臨時開啟對話退役、按鈕改開 chooser 預選 server 直達 scope 步驟；apps/desktop/src/i18n/messages.ts 補文案。App、serversPanel 既有測試更新（本機開啟行為凍結斷言保留）。全綠。 <!-- speclink-task:tsk_01KXYYTF12VPR34QG3RE1SR18S -->

## 4. store 分流與 checkoutRoot

- [x] 4.1（design「決策 3：PROBE 擴充為四態」的 store 半邊；規格「remote marker 資料夾的探測分流」）：apps/desktop/src/store.ts——remoteBinding 分流（已登入直開 remote 分頁帶 checkoutRoot；未登入開 chooser 預填 server 位址；hasLocalOpenspec 強制選擇對話：繼續本地可用、使用 remote 停用附待遷移說明）。apps/desktop/src/__tests__/remoteOpen.test.ts 與 store.test.ts 覆蓋三分流。全綠。 <!-- speclink-task:tsk_01KXYYTF120GD9Y46X6PVS92SB -->
- [x] 4.2（規格 workspace-session 的「分頁身分為 WorkspaceLocator 而非 root 路徑」修訂；design「決策 4：checkoutRoot 的落地邊界」）：apps/desktop/src/session.ts 的 createRemoteSession 帶 checkoutRoot、locator key 不含 checkoutRoot（同 scope 重綁覆寫不分裂分頁）、持久化 v2 承載與重啟恢復；remote 分頁 tooltip 顯示已連接 checkout 路徑；capability 描述不因 checkoutRoot 改變（斷言）。session 與 projectTabs 測試更新。全綠。 <!-- speclink-task:tsk_01KXYYTF12N55MZ1NWMRESB5XP -->

## 5. 驗收

- [x] 5.1 GUI 鐵律手動全鏈（design Implementation Contract；操作前確認使用者未在使用螢幕）：npm run dev → chooser 開 spec-only 分頁（全程無文字輸入 repo）→ 對無 marker git repo 綁 checkout、確認 marker 寫入、關分頁後直接開該資料夾直達 remote 分頁 → marker 不一致資料夾拒絕訊息如實 → 本地＋marker 並存停下且繼續本地可用 → 分頁 tooltip 顯示 checkout 路徑 → 本機資料夾路徑與 init 流程行為與導入前一致。 <!-- speclink-task:tsk_01KXYYTF12FFP0CCY1PZ8FXMFP -->
- [x] 5.2 回歸（design「決策 5：規格債清償的措辭」——workspace-session 與 remote-workspace-data 的 MODIFIED 措辭由本 change 的 spec deltas 承載，行為面由 remoteOpen 測試涵蓋「handshake 成功後才建立 remote session」修訂）：npm test -w apps/desktop、npm test -w packages/ui、cargo test -p speclink-desktop-core、cargo build --release -p speclink-desktop 全綠（重建前關閉執行中 exe）。 <!-- speclink-task:tsk_01KXYYTF12QP1A0326Z9SVET5B -->
