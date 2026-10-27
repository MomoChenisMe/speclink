## Context

已就位：/scopes 端點與 client 方法（membership 過濾）、連線 registry＋Keychain＋device/PAT 登入、remote session 建構（handshake fail-closed、重啟重驗）、.speclink.yaml remote section（url＋repo，speclink-core 的 write_remote_section 讀寫）、desktop-core 的 open_project 三態 PROBE（project／uninitialized）。現況缺口：remote 開啟只有伺服器頁籤的 repo 文字輸入臨時對話；開含 remote marker 的資料夾走本地 PROBE、marker 被無視；Repo registry 無 workspacePath／git remote 欄位（§10.5 深度驗證無資料可比）；workspace-session 正典的「remote 無建構路徑」句已與現實矛盾（刀 3 規格債）。CLI 對 local＋remote 並存的行為是警告後 remote 生效——架構明文 Desktop 必須停下強制選擇，兩表面刻意分歧。

## Goals / Non-Goals

**Goals:**

- §10.5 正典 onboarding 全流程：來源分流、server 選擇/登入、scopes 清單選擇、checkout 分流、開啟。
- remote marker 資料夾可直接開啟（RD 重開 checkout 的日常路徑）；並存衝突停下。
- checkoutRoot 落地於 locator 與持久化；規格債清償。

**Non-Goals:**

- checkout 功能面不解鎖（apply／完整 drift／verify 屬後續 RD 刀）；正式 local-to-remote migration 屬後續刀（衝突對話明示待提供）；git remote／workspacePath 深度驗證待 registry 補欄位的後續 server 刀；CLI 的並存行為不動（表面分歧如實保留）；不做 chooser 內的帳號管理（登入細節仍屬伺服器頁籤，chooser 只引用）。

## Decisions

### 決策 1：chooser 多步對話與入口整合

WorkspaceChooser 為單一對話元件、逐步呈現：(1) 來源——本機資料夾｜Speclink Server；(2a) 本機——資料夾選擇後走既有 PROBE 與 init 流程（行為不變）；(2b) server——已登入 connections 清單＋「新增 server」（就地走刀 2 的新增＋登入，完成後回流）；(3) scopes——/scopes 回應以 Project 分組列 Repos，單選；(4) checkout——「略過（規格模式）」或「選擇資料夾」；(5) 開啟。三個既有入口（右上、空狀態、分頁加號）一律改開 chooser；伺服器頁籤的臨時開啟對話退役、該處按鈕開 chooser 並預選 server 跳至步驟 3。表單控制項用自建元件、文案繁中。

### 決策 2：checkout 驗證＝marker 一致性

驗證規則按資料夾狀態分三路：(a) 有 remote marker——marker 的 url origin 與 repo 必須等於所選 connection origin 與 repo，一致即綁定、不一致以繁中訊息拒絕（指出 marker 指向的 origin/repo）；(b) 無 marker 且為 git repo（含 .git）——綁定並以 speclink-core 的 write_remote_section 寫入 marker（與 CLI init_remote 同構、互通）；(c) 非 git repo——拒絕（checkout 的最低資格）。替代案「對 registry 的 workspacePath/git remote 驗證」不可行——欄位不存在；記為後續 server 刀強化點。

### 決策 3：PROBE 擴充為四態

open_project 的 PROBE payload 擴充：project／uninitialized（既有）＋ remoteBinding{url, repo?, hasLocalOpenspec}。desktop-core 讀 .speclink.yaml remote section 判定；同資料夾 openspec/ 存在即 hasLocalOpenspec=true。store 分流：僅 remoteBinding——origin 對 registry 找 connection：已登入即 handshake 開 remote 分頁（checkoutRoot＝該資料夾）、未登入/無 connection 即開 chooser server 步驟並預填 url；並存（hasLocalOpenspec）——強制選擇對話：「繼續本地」（開 local session、本次忽略 marker）｜「使用 remote」按鈕停用附「待正式 migration 提供」說明——不得靜默覆蓋任一方。CLI 的警告後 remote 生效屬 headless 表面、不改。

### 決策 4：checkoutRoot 的落地邊界

remote locator 的 checkoutRoot 由步驟 4 或 remoteBinding 直開路徑寫入；持久化 v2 天然承載；同 connection/project/repo 而 checkoutRoot 不同視為同一分頁（locator key 不含 checkoutRoot——身分是 scope、checkout 是屬性；重開時新 checkoutRoot 覆寫舊值）。呈現最小面：remote 分頁 tooltip 加「已連接 checkout：路徑」。功能面（apply/drift/verify 解鎖）明確不在本刀——capability 描述不因 checkoutRoot 改變。

### 決策 5：規格債清償的措辭

workspace-session 的 locator 需求句「本階段僅存在於型別、SHALL NOT 有任何建構路徑」改為「經 chooser／handshake 成功路徑建構，checkoutRoot 由 checkout 綁定流程寫入」；remote-workspace-data 的 handshake 需求「開啟入口以 repo 識別發起 handshake」改為「開啟入口（chooser）以 scopes 清單選擇的 repo 發起 handshake」、其情境改以「選擇後因權限變化被拒」表達 fail-closed（/scopes 過濾後仍可能于選擇與 handshake 之間被撤 membership——競態下 handshake 仍是最終防線）。

## Implementation Contract

- vitest（假 adapter）：chooser 全流程（來源分流、server 清單與新增回流、scopes 分組單選、checkout 三路驗證各自訊息、開啟成功建分頁）；PROBE 四態分流（remoteBinding 直開帶 checkoutRoot、未登入引導、並存強制選擇且 remote 停用附說明）；三入口皆開 chooser；臨時對話退役（原測試更新）。
- Rust：desktop-core probe 四態單元測試（marker 解析、hasLocalOpenspec 判定、壞 YAML fail-closed 沿 .speclink.yaml 既有語意）；write_remote_section 綁定路徑與 CLI 同構斷言（寫後 CLI resolve 可讀）。
- 手動驗證（remote-dev-harness；操作前確認使用者未在使用螢幕）：npm run dev → chooser 走 server 路徑選 scope 開 spec-only 分頁；再走 checkout 綁定（無 marker git repo → marker 寫入、重開資料夾直達 remote 分頁）；marker 不一致拒絕訊息如實；本地＋marker 並存停下、繼續本地可用；分頁 tooltip 顯示 checkout 路徑。
- 回歸：npm test -w apps/desktop、npm test -w packages/ui、cargo test -p speclink-desktop-core、cargo build --release -p speclink-desktop 全綠；本機開啟與 init 流程行為凍結（既有測試斷言語意不變）。
