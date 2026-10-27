## MODIFIED Requirements

### Requirement: setup 流程完成開箱四要素
<!-- BEFORE: `/setup` 以 server-rendered 表單完成 Admin、Store 狀態、registry 與連線資訊，提交使用同源 POST。 -->

`/setup` SPA 與 `/api/speclink/v1/web/setup/*` SHALL 以 bootstrap token 門禁，流程 SHALL 涵蓋：建立第一位 Admin（email、顯示名、密碼，直接為 active 且帶 admin 旗標，不經邀請）、顯示 Store 狀態（manifest 的 driver 與 capabilities、health 結果、identity schema version）、建立第一組 Project 與 Repo（寫 registry）、顯示初始連線資訊（部署組態的 public URL 與所建 project／repo keys）。流程 SHALL 冪等可續作：token 未耗用前重入不重建已完成的節；全部 mutation SHALL 驗證同源並回相同 `{data}`／`{error}` browser JSON envelope。setup SHALL NOT 寫入 public URL，其唯一來源 SHALL 為部署組態。

完成最後節點 SHALL 在既有 setup 交易邊界內耗用 bootstrap token、記錄 audit 並建立第一位 Admin 的 Web session；成功回應 SHALL 帶 `destination: "/admin?welcome=1"` 與 `connection: {publicUrl, projectKey, repoKey}`。Session cookie SHALL 在回應送出前設定完成；若 session 建立失敗，Server SHALL 回可重試登入的 recovery error，SHALL NOT 把結果表示為已登入。

#### Scenario: 完成 setup 即可邀請與連線

- **WHEN** 完成 setup 建立 Admin 與第一組 Project／Repo 後，以 invite 子命令對該 project 邀請成員，成員接受邀請、建 PAT 並以 CLI 連線
- **THEN** setup 回應建立 session 並導向 `/admin?welcome=1` 顯示連線資訊；邀請建立成功；成員的 `/binding` 對該 project／repo 成功；CLI remote 動詞照常運作

#### Scenario: 中斷後憑同一 token 續作

- **WHEN** 完成第一位 Admin 建立後關閉瀏覽器，再憑同一 token 進入 `/setup`
- **THEN** SPA 顯示 Admin 節已完成且不重建，可繼續建立 Project／Repo 完成流程

#### Scenario: 重複完成不建立第二份資料

- **WHEN** setup completion 請求因 client 未收到回應而以相同 bootstrap token 重送
- **THEN** Server 不建立第二位 Admin 或第二組 Project／Repo；已耗用 token 回既有不可區分的無效 token 結果

#### Scenario: 跨 origin setup mutation 被拒絕

- **WHEN** 有效 bootstrap token 從不同 origin 提交任一 setup mutation
- **THEN** Server 回 403，不建立或修改 Admin、registry、session 與 audit 成功事件
