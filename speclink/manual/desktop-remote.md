---
title: 桌面連線 server 與 remote 專案
section: Remote 模式
order: 470
keywords: [伺服器頁, 裝置登入, 存取金鑰, Keychain, 新增專案, 預設連線, 最近開啟, Project／Repo, checkout, 只看規格, Copilot]
sources: [desktop-connections, workspace-chooser, server-device-auth]
generated: 2026-10-27T11:03:57+08:00
---

# 桌面連線 server 與 remote 專案

桌面 app 連 server 分兩步：先在應用程式設定的「伺服器」頁登入一個 server，再用「新增專案」開一個 remote 專案。remote 專案可以只看規格，也可以連接一個本機的 checkout。

## 伺服器頁

點圖示列的設定齒輪進入應用程式設定，分類欄的「伺服器」就是這一頁。它是 app 全域的，與任何專案無關。

- 頁標題右側的「新增連線」：在頁頂展開新增卡，填 URL 與顯示名，按新增或取消。新增成功後新增卡收起，連線卡立即出現該列並進入登入流程。
- 同一個位址重複新增時，app 只更新顯示名，不新增第二筆。
- 「連線」卡：每個 server 一列。左邊是顯示名與狀態籤（已登入、未登入、登入過期、等待授權、失敗），下面是等寬字的位址與登入身分。右邊依狀態是「登出」（已登入），或「登入」／「重新登入」（未登入、登入過期），再加一個「⋯」選單，內含「開啟專案」與「移除」。登入過期的狀態籤是警示色。等待授權的細節與存取金鑰輸入，在那一列下方就地展開。沒有任何連線時顯示空狀態。
- 「預設」卡：一列「新增遠端專案時預設使用」下拉，選項是已登入的連線與「不預選」，記在 app 本機。之後在「新增專案」選 Server 來源時，Server 步驟會預選這個連線，你仍可改選。那個連線已登出或被移除時不預選；對話框是帶著特定目的打開的（指定了某個連線，或資料夾的 remote 標記指向某個伺服器）時也不預選。
- 「登入方式」卡：兩列說明裝置登入（預設）與存取金鑰（伺服器不支援裝置登入時就地改用），沒有其他動作。
- 清單跨重啟保留。清單檔只有識別、位址、顯示名、身分顯示名，不含任何憑證。清單檔壞掉時清單歸零，app 不會崩潰。
- 登入成功後，那一列出現開啟專案的行動呼籲並取得鍵盤焦點。app 不會自動打開「新增專案」對話框。

## 登入

app 先嘗試裝置登入。server 支援時：

1. app 開啟系統瀏覽器到核准頁，裝置碼已預填。
2. 發起登入的地方就地顯示等待授權面：裝置碼與驗證網址（各附複製）、剩餘時間倒數、取消。你不必依賴那個瀏覽器分頁，可以換另一部裝置開驗證網址完成核准。
3. 瀏覽器還沒登入 server 時，先登入。登入後瀏覽器回到核准頁，裝置碼仍預填。
4. 在核准頁按下一步，再明確選核准或拒絕。結果頁會提示你回到 Speclink app 繼續，app 會自己取得結果。
5. 核准後，app 把續約憑證存進 OS Keychain，並顯示登入身分。

其他結果：

- 你在瀏覽器拒絕：app 停止輪詢，顯示已拒絕，不留任何憑證。
- 逾時沒人核准：app 顯示逾時。
- 你按取消：輪詢立即停止，該連線回到未登入，不留任何憑證。瀏覽器端的授權請求自然逾期。
- server 明確不支援裝置登入（回 404 或 405）：等待授權面改為存取金鑰的貼上輸入。貼入有效的存取金鑰後登入成功並顯示身分。app 先向 server 查驗身分，確認有效後才存進 Keychain。
- 網路不可達或 server 錯誤（5xx）：顯示連線錯誤，不會進入存取金鑰輸入。

在「新增專案」的 Server 步驟就地新增連線並登入時，看到的是同一套等待授權面。取消時停留在 Server 步驟。

## 憑證存在哪

- 憑證只存在 OS Keychain：macOS 用 Keychain，Windows 用 Credential Manager。每個 server 一筆。
- 憑證不在 localStorage、連線清單檔、repo、URL 或 log 裡。存取金鑰只在你貼上的那一刻經過一次。
- 短效通行證只留在記憶體。續約後，Keychain 裡的憑證立即換新。重啟 app 後不必重新登入。

CLI 端怎麼登入，見 [CLI 連接 remote](cli-remote.md)。

## 登出與移除

- 登出會刪掉該 server 的 Keychain 條目，並清掉清單上的身分顯示名。
- 裝置登入的連線：app 盡力呼叫 server 撤銷這組裝置憑證。server 不可達時，本機清理照常完成，畫面顯示未登入。
- 存取金鑰登入的連線：app 刪掉本機條目，並提示你到 server 的帳號頁撤銷那把金鑰。
- 移除連線：先做登出，再刪清單條目。

## 新增 remote 專案

圖示列的「＋」、沒有任何專案時空狀態頁的「新增專案」與「連線 Server」、macOS 原生選單的「開啟專案…」，都通到同一個「新增專案」對話框。本機資料夾那條路見[專案、新增專案與設定頁](desktop-projects.md)；這裡說 Server 這條路。

1. 第一步選「Server」來源卡，按「下一步」。從空狀態按「連線 Server」會直接到 Server 步驟；從伺服器頁的「開啟專案」進來時，那個 server 已預選，直接到 Project／Repo 步驟。
2. Server 步驟：已登入連線的單選列，加上「新增 Server」。可以就地新增並登入，流程見上方「登入」。設了預設連線時會預選它。
3. Project／Repo 步驟：清單依 Project 分組列出 Repo，單選。你不必手打任何識別字串。
4. checkout 步驟：選要不要連接本機 checkout。略過時，app 以只看規格的方式開啟 remote 專案。

步驟 2 到 4 都是整列可點的單選列。對話框標題是「新增專案」。

沒有任何成員資格時，Project／Repo 清單是空的，並顯示繁中說明，這不是錯誤。解法是請管理員授予成員資格，見 [帳號、邀請、存取金鑰與 membership](accounts.md)。

例：預設連線是已登入的 Y。你打開一個 remote 標記指向伺服器 X（還沒登入）的本機資料夾，對話框直接進入 Server 步驟。這時 Y 不會被預選、頁尾也沒有「下一步」，你要自己選連線，或依 X 新增並登入。

### 從最近開啟直接開回來

「新增專案」第一步兩張來源卡的下方，以及沒有任何專案時的空狀態頁，都有「最近開啟」，列出你開過、但現在沒開著的專案。remote 條目帶「遠端」籤，顯示連線名稱與 Project／Repo。點它，app 先確認原本綁的 checkout 仍對得上那個 Project／Repo，再用同一個連線、同一個 Project／Repo、同一個 checkout 資料夾開啟，不必重走上面四步。沒綁 checkout 的條目跳過確認、直接開。

- 條目所屬的連線已從伺服器頁移除：顯示「連線已移除」，開啟停用，移除仍可用。
- 連線還在但已登出：顯示「連線已登出」，開啟停用，移除仍可用。先回伺服器頁重新登入或重新新增連線，再開。
- 伺服器清單讀取失敗或還沒讀完：app 不先判定上面兩種狀態，開啟維持可用。你點條目時，app 在開啟流程裡用現有清單補判：連線解不出來、且條目綁有 checkout 時，轉成「連線已移除」、不開啟；沒綁 checkout 的條目照常以原連線與 Project／Repo 開啟。
- 開啟失敗（例如 checkout 資料夾已被刪除）：條目變成錯誤態並顯示一行原因，不開啟、不切換專案。錯誤態的條目可以再點一次重試，重試開始時錯誤態清掉。

錯誤態在你下一次打開「新增專案」、或空狀態頁重新出現時清掉。完整的記錄、顯示與移除規則見[專案、新增專案與設定頁](desktop-projects.md)。

## 連接 checkout

選擇連接 checkout 時，app 先在不寫入的前提下檢查資料夾：

- 資料夾已有 remote 標記（`.speclink.yaml` 的 remote 區段）：它指向的 server 與 repo 必須與你選的 Project／Repo 一致。不一致時，app 以繁體中文訊息拒絕，並指出標記指向哪個 server 與 repo。磁碟不動。
- 資料夾沒有標記：它必須是一個 Git repository。不是的話拒絕。

檢查通過後：

- 畫面顯示 Claude、Codex、Copilot 三個勾選框與資料夾路徑。
- 既有 `.speclink.yaml` 記錄的內建工具成為預選值。沒有工具清單時，app 只依資料夾裡實際存在的工具痕跡預選，不會預設勾 Claude。資料夾裡有 `.github/` 不會讓 Copilot 被預選。
- 至少勾一個工具、且路徑齊備之前，開啟按鈕停用。

> [!NOTE]
> 規格的 checkout 綁定段把這顆按鈕寫作「開啟 Workspace」；較晚的「新增專案」對話框需求規定使用者可見文案不得出現「Workspace」，但沒有寫新字面。實際字面以產品為準，詳見 [本手冊的來源](about.md)。

按下開啟後：

- app 再做一次標記檢查，把勾選的工具寫進 `.speclink.yaml`，生成或更新那些工具的技能檔，並清掉未勾工具的 Speclink 受管產物。
- app 不生成任何指令檔：AGENTS.md、CLAUDE.md、`.github/copilot-instructions.md` 都不產生。AGENTS.md 與 CLAUDE.md 裡舊版留下的 Speclink 區塊會被剝掉，區塊外你自己寫的文字保留。
- 自訂工具描述子、app 看不懂的設定項、remote 區段與其他設定都保留。
- 沒有標記的 checkout，app 寫入與 CLI remote init 同構的 remote 區段。已有相符標記的 checkout 也會重新同步，不會提前成功。
- 全部同步成功後，app 才建立 remote 專案並向 server 連線確認。
- 同步失敗時，對話框顯示一行含失敗階段的錯誤，並保留路徑與工具選集。修好後用相同選集再按一次即可。這時不會建立專案。
- 滑鼠停在圖示列的專案方塊上，提示顯示已連接的 checkout 路徑。
- 這個過程不建立本機 `speclink/`，也不上傳或修改 server 上的內容。

| 情況 | 按下開啟後 |
| --- | --- |
| 沒標記的 Git repository，勾 Claude 與 Codex | `.speclink.yaml` 記兩個工具與 remote 區段，兩組技能檔就位，不產生 AGENTS.md、CLAUDE.md |
| 沒標記、含 `.github/workflows/ci.yml` 的 repository，勾 Claude 與 Copilot | `.claude/skills/` 與 `.github/skills/` 兩組技能檔就位，`ci.yml` 一個位元都不變 |
| 原本只用 Claude、CLAUDE.md 有舊 Speclink 區塊與你的文字，改成只勾 Codex | 補上 Codex 技能檔；Claude 技能檔與 CLAUDE.md 的舊區塊移除，你的文字與 remote 區段保留；不產生 AGENTS.md |
| 標記相符、工具只有 Codex，但 Codex 技能檔不見了 | 補齊 Codex 技能檔，remote 區段不變，同步成功後才開啟 |

## 直接開啟帶 remote 標記的資料夾

從本機資料夾入口打開一個帶 remote 標記的資料夾時：

- 對應的 server 已登入，且 `.speclink.yaml` 至少有一個有效的內建工具（claude、codex、copilot）：app 先補齊或更新那些工具的受管產物，成功後直接開 remote 專案，不經對話框。checkout 就是你打開的那個資料夾。
- 工具清單缺席或不含任何內建工具：app 導向對話框的 checkout 步驟，預填 server、Project／Repo 與路徑，要你明示勾選 Claude、Codex 或 Copilot。送出成功前不會開啟。
- 沒有對應連線或未登入：app 導向對話框的 Server 步驟，預填 server 位址。
- 補齊受管產物時遇到檔案系統錯誤：app 顯示含路徑與失敗階段的錯誤，不開啟、不連線。
- 標記無法解析：app 顯示解析錯誤，不動資料夾，不開啟。

資料夾同時有本地 `speclink/` 與 remote 標記時，app 停下來要你選。三個出口都不會自動覆蓋：

- 「繼續本地」：這次以本地模式開啟，標記不動。
- 「以 server 為準」：本地 `speclink/` 改名為帶日期的備份，資料夾轉為 checkout，同步工具後開 remote 專案。對話文案明說這是備份後棄用本地，不是合併。本地內容不上傳，server 不改動。
- 「遷移本地內容」：進入遷移流程，目標必須是空的 Project／Repo。見 [本地工作區遷移到 remote](migrate.md)。

remote 專案的專案設定（政策版次、版次衝突、reader 唯讀）見[專案、新增專案與設定頁](desktop-projects.md)。

**出處**：`desktop-connections`、`workspace-chooser`、`server-device-auth`
