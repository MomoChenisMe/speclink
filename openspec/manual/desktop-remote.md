---
title: 桌面連線 server 與 remote 工作區
section: Remote 模式
order: 470
keywords: [伺服器頁籤, 裝置登入, Keychain, 新增 Workspace, scope, checkout, spec-only]
sources: [desktop-connections, workspace-chooser, server-device-auth]
generated: 2026-09-02
---

# 桌面連線 server 與 remote 工作區

桌面 app 連 server 分兩步：先在伺服器頁籤登入一個 server，再用「新增 Workspace」開一個 remote 分頁。remote 分頁可以只看規格，也可以連接一個本機的 checkout。

## 伺服器頁籤

設定頁有一個伺服器頁籤。它是 app 全域的，與任何專案分頁無關。

- 清單列出已存的 server：顯示名、位址、登入狀態、登入身分。
- 新增連線：填 URL 與顯示名。新增後清單立即出現該條目，並進入登入流程。
- 同一個位址重複新增時，app 只更新顯示名，不新增第二筆。
- 清單跨重啟保留。清單檔只有識別、位址、顯示名、身分顯示名，不含任何憑證。
- 每一列有登入、登出、移除三個操作。
- 登入成功後，該列出現「開啟工作區」的入口並取得鍵盤焦點。app 不會自動打開工作區選擇器。

## 登入

app 先嘗試裝置登入。server 支援時：

1. app 開啟系統瀏覽器到核准頁，裝置碼已預填。
2. 發起登入的地方就地顯示等待授權面：裝置碼與驗證網址（各附複製）、剩餘時間倒數、取消。你不必依賴那個瀏覽器分頁，可以換另一部裝置開驗證網址完成核准。
3. 瀏覽器還沒登入 server 時，先登入。登入後瀏覽器回到核准頁，裝置碼仍預填。
4. 在核准頁按下一步，再明確選核准或拒絕。
5. 核准後，app 把續約憑證存進 OS Keychain，並顯示登入身分。

其他結果：

- 你在瀏覽器拒絕：app 停止輪詢，顯示已拒絕，不留任何憑證。
- 逾時沒人核准：app 顯示逾時。
- 你按取消：輪詢立即停止，該連線回到未登入，不留任何憑證。瀏覽器端的授權請求自然逾期。
- server 明確不支援裝置登入（回 404 或 405）：等待授權面改為存取金鑰的貼上輸入。貼入有效的存取金鑰後登入成功並顯示身分。app 先向 server 查驗身分，確認有效後才存進 Keychain。
- 網路不可達或 server 錯誤（5xx）：顯示連線錯誤，不會進入存取金鑰輸入。

從工作區選擇器的 server 步驟新增連線並登入，看到的是同一套等待授權面。取消時停留在 server 步驟。

## 憑證存在哪

- 憑證只存在 OS Keychain：macOS 用 Keychain，Windows 用 Credential Manager。每個 server 一筆。
- 憑證不在 localStorage、連線清單檔、repo、URL 或 log 裡。存取金鑰只在你貼上的那一刻經過一次。
- 短效通行證只留在記憶體。續約後，Keychain 裡的憑證立即換新。重啟 app 後不必重新登入。
- 同一台機器、同一個 server，桌面 app 與 CLI 共用同一份續約憑證。桌面 app 登入後，CLI 免登入。詳見 [CLI 連接 remote](cli-remote.md)。

## 登出與移除

- 登出會刪掉該 server 的 Keychain 條目，並清掉清單上的身分顯示名。
- 裝置登入的連線：app 盡力呼叫 server 撤銷這組裝置憑證。server 不可達時，本機清理照常完成，畫面顯示未登入。
- 存取金鑰登入的連線：app 刪掉本機條目，並提示你到 server 的帳號頁撤銷那把金鑰。
- 移除連線：先做登出，再刪清單條目。

## 新增 remote 工作區

視窗頂列、空狀態、分頁列的加號、伺服器頁籤，四個入口都通到同一個「新增 Workspace」選擇器。

1. 第一步選來源：「本機資料夾」或「Server」。從伺服器頁籤進來時，該 server 已預選，直接到 scope 選擇步驟。
2. 選一個已登入的 server。也可以就地新增並登入。
3. 從 scopes 清單選一個 Project 底下的 Repo。清單依 Project 分組，單選。你不必手打任何識別字串。
4. 選要不要連接 checkout。略過時，app 以 spec-only 開啟 remote 分頁。

沒有任何成員資格時，清單是空的，並顯示「此帳號目前沒有任何 Project／Repo membership」。這不是錯誤。解法是請管理員授予成員資格，見 [帳號](accounts.md)。

## 連接 checkout

選擇連接 checkout 時，app 先在不寫入的前提下檢查資料夾：

- 資料夾已有 remote 標記（`.speclink.yaml` 的 remote 區段）：它指向的 server 與 repo 必須與你選的 scope 一致。不一致時，app 以繁體中文訊息拒絕，並指出標記指向哪個 server 與 repo。
- 資料夾沒有標記：它必須是一個 Git repository。不是的話拒絕。

檢查通過後：

- 畫面顯示 Claude／Codex 兩個勾選框與資料夾路徑。
- 既有 `.speclink.yaml` 的 tools 成為預選值。沒有 tools 清單時，app 只依資料夾裡實際存在的工具痕跡預選，不會預設勾 Claude。
- 至少勾一個工具、且路徑齊備之前，「開啟 Workspace」按鈕停用。

按下「開啟 Workspace」後：

- app 把勾選的工具寫進 `.speclink.yaml`，生成或更新那些工具的 Skills 與指令檔區塊，並清掉未勾工具的受管產物。你自己寫的內容、remote 區段與其他設定都保留。
- 沒有標記的 checkout，app 寫入與 CLI remote init 同構的 remote 區段。已有相符標記的 checkout 也會重新同步，不會提前成功。
- 全部同步成功後，app 才建立分頁並向 server 確認專案與 repo。
- 同步失敗時，選擇器顯示一行含失敗階段的錯誤，並保留路徑與工具選集。修好後用相同選集再按一次即可。分頁不建立。
- 分頁以 tooltip 顯示已連接的 checkout 路徑。
- 這個過程不建立本機 `openspec/`，也不上傳或修改 server 上的內容。

## 直接開啟帶標記的資料夾

從本機資料夾入口打開一個帶 remote 標記的資料夾時：

- 對應的 server 已登入，且 tools 有效：app 先補齊或更新受管產物，成功後直接開 remote 分頁，不經選擇器。
- tools 缺席或無效：app 導向選擇器的 checkout 步驟，預填 server、scope 與路徑，要你明示勾選 Claude 或 Codex。
- 沒有對應連線或未登入：app 導向選擇器的 server 步驟，預填 server 位址。
- 補齊受管產物時遇到檔案系統錯誤：app 顯示含路徑與失敗階段的錯誤，不建立分頁。
- 標記無法解析：app 顯示解析錯誤，不動資料夾，不建立分頁。

資料夾同時有本地 `openspec/` 與 remote 標記時，app 停下來要你選。三個出口都不會自動覆蓋：

- 「繼續本地」：這次以本地模式開啟，標記不動。
- 「以 server 為準」：本地 `openspec/` 改名為帶日期的備份，資料夾轉為 checkout，同步工具後開 remote 分頁。對話文案明說這是備份後棄用本地，不是合併。本地內容不上傳，server 不改動。
- 「遷移本地內容」：進入遷移流程，目標必須是空的 scope。見 [本地工作區遷移到 remote](migrate.md)。

**出處**：`desktop-connections`、`workspace-chooser`、`server-device-auth`
