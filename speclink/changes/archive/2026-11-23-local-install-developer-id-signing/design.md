## Context

本機安裝腳本（scripts/desktop/desktop-install.mjs）把 sidecar 佈署、前端建置、`tauri build`、版號斷言與覆蓋安裝收成一個指令，`/release` 收尾的「重裝本機 app」也呼叫它。它目前只設定 updater 的簽章環境變數（`TAURI_SIGNING_PRIVATE_KEY` 與密語），沒有設定 OS 程式碼簽章身分，所以 Tauri 產出的 app 與 bundle 內的 sidecar CLI 都是 ad-hoc 簽章。實測 v0.17.0 本機重裝後：CLI 的簽章是 `Identifier=speclink-9973c961f6bb5a30`、`Signature=adhoc`、`TeamIdentifier=not set`，app 是 `speclink_desktop-b3a780fb079386fc`，識別碼帶著隨內容變動的雜湊。

macOS 鑰匙圈以程式的簽章身分記住「哪些程式可以讀這個項目」。CLI 與 Speclink Desktop 的遠端登入憑證存在 `speclink-desktop` 服務底下（refresh、bearer、pat 三種，每個 Server origin 一組）。ad-hoc 身分每次重建都不同，所以每次重裝後每個項目都要重新允許。

Release 工作流在 macOS 簽章組齊備時以 Developer ID 簽章並公證（v0.16.0 的 Release 執行了「Enable macOS code signing and notarization」這一步），公證要求 bundle 內每一支執行檔都簽過，所以 Tauri 會連 sidecar CLI 一起簽。維護者的鑰匙圈裡有同一張「Developer ID Application」憑證。

## Goals / Non-Goals

**Goals:**

- 本機安裝在找得到 Developer ID 憑證時，app 與 sidecar CLI 的簽章身分在每次重建之間都相同，鑰匙圈的「永遠允許」在重裝之後仍然有效。
- 找不到憑證時，行為和現在一樣能建置與安裝，只多一行說明。
- 用了簽章身分時，以斷言確認 bundle 內的 CLI 真的簽上了，不信任「設了環境變數就會簽」。

**Non-Goals:**

- 公證與 staple：本機建置不經下載、沒有隔離屬性，Gatekeeper 不檢查；公證要 Apple ID 與網路，留給 Release 工作流。
- 改 Release 工作流、`tauri.conf.json` 或 Tauri 的 macOS 設定（簽章身分只經環境變數交給這一次的 `tauri build`）。
- 自簽憑證或「Apple Development」憑證：維護者已有 Developer ID，自簽要另外建憑證並信任；Apple Development 與正式版的身分不同。
- 清掉或遷移鑰匙圈裡既有的登入項目，也不改 CLI 與 Desktop 讀鑰匙圈的方式。
- Windows 與 Linux 的本機簽章：安裝步驟只支援 macOS，非 macOS 不查找簽章身分。

## Decisions

**D1 簽章身分的來源與順序。** 只在 macOS 決定：(a) 環境變數 `APPLE_SIGNING_IDENTITY` 有值就用它，不查找鑰匙圈——Tauri 本來就讀這個變數，維護者手動指定時以它為準；(b) 沒有值時執行 `security find-identity -v -p codesigning`，取輸出中第一個名稱以「Developer ID Application:」開頭的有效身分，以引號內的完整名稱當身分；(c) 都沒有時以 ad-hoc 建置。不選「Apple Development」或其他種類，因為只有 Developer ID 和正式版是同一個 Team 的身分。有多張 Developer ID 時取第一張並印出名稱，不另加選擇的選項。

**D2 只交給 `tauri build` 這一個子程序。** 找到的身分以 `APPLE_SIGNING_IDENTITY` 放進 `tauri build` 子程序的環境變數，sidecar 佈署與前端建置不帶。不寫進 Tauri 設定檔，避免 CI 與沒有憑證的電腦被影響。

**D3 查找失敗時退回 ad-hoc，不停止。** `security` 無法執行或以非零結束時，印一行說明（含失敗原因）並以 ad-hoc 建置。ad-hoc 就是現在的行為，不會裝錯版本；版號斷言照常把關。這和「app 是否執行中」的判斷不同：那一步查不清楚時可能覆蓋使用者正在用的 app，所以必須停止。

**D4 簽章斷言。** 決定了身分時，在 bundle 的版號斷言之後執行 `codesign -dv <bundle 內的 sidecar CLI>`（資訊印在 stderr），輸出必須有 `TeamIdentifier=` 後面接非 `not set` 的值，而且沒有 `Signature=adhoc`。不符時印出 codesign 的輸出並以非零結束，不進入安裝。不比對 Team ID 是否等於憑證名稱括號內的值：環境變數可能給的是 SHA-1 雜湊，無從比對。以 ad-hoc 建置時不做這道斷言。

**D5 印出的位置。** 簽章身分的決定與印出併入步驟 (1)「這棵樹是什麼」：印完來源與版號後多印一行「簽章身分：<名稱>」或 ad-hoc 的說明。步驟編號 (0)～(7) 不變，步驟 (0) 的把關（平台、updater 簽章環境變數）不變。

**D6 純函式與測試。** 解析與判斷拆成可匯出的純函式，由既有的 node --test 測試涵蓋：
- `developerIdIdentity(findIdentityOutput)`：回傳第一個 Developer ID Application 身分名稱，沒有時回傳 null。
- `chooseSigningIdentity({ env, platform, lookup })`：依 D1 回傳 `{ identity, note }`；`lookup` 是查找函式，測試注入假的輸出或丟出錯誤。
- `signatureProblem(codesignOutput)`：簽章符合時回傳 null，不符時回傳要印的原因。
實際的簽章、`security` 與 `codesign` 呼叫不進自動測試，由手動任務驗證。

## Implementation Contract

**行為：**
- macOS、未設定 `APPLE_SIGNING_IDENTITY`、鑰匙圈有 Developer ID Application 憑證：步驟 (1) 印出「簽章身分：Developer ID Application: <名稱> (<Team ID>)」，`tauri build` 以它簽章；步驟 (4) 之後的簽章斷言通過；bundle 與安裝版的 sidecar CLI 以 `codesign -dv` 查看時有 Team ID、不是 adhoc，識別碼不再帶雜湊。
- macOS、已設定 `APPLE_SIGNING_IDENTITY`：印出這個值並用它，不執行 `security`。
- macOS、沒有憑證或查找失敗：印出一行「找不到 Developer ID Application 憑證，這次以 ad-hoc 簽章建置；重裝後鑰匙圈會重新詢問」（查找失敗時附上原因），其餘步驟與現在相同，不做簽章斷言。
- 非 macOS：不查找、不印簽章身分那一行，建置步驟與現在相同。
- 腳本的用法、`--install` 旗標、成功與失敗的 exit code 意義不變。

**失敗模式：** 只新增一種失敗——決定了身分但 bundle 內 CLI 的簽章不符（D4），以 `desktop-install: ` 開頭的單行錯誤加上 codesign 輸出、exit code 1、/Applications 不變。

**驗收：**
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過，其中 scripts/desktop/desktop-install.test.mjs 涵蓋 D6 三個函式的正反案例：一張、多張、沒有 Developer ID；只有 Apple Development；`0 valid identities found`；環境變數優先；非 macOS 不查找；查找丟錯退回 ad-hoc；codesign 輸出為 adhoc、`TeamIdentifier=not set`、正常 Team ID。
- 手動：在有憑證的 Mac 上帶 `--install` 連續重裝兩次，兩次安裝版 CLI 的 `codesign -dv` 識別碼與 Team ID 相同；第二次重裝後，在連到遠端專案的 checkout 執行 CLI 指令，鑰匙圈不再詢問。

**範圍：** 只改 scripts/desktop/desktop-install.mjs 與 scripts/desktop/desktop-install.test.mjs，以及 `desktop-release` 規格的「本機安裝的新鮮度斷言」。不改 Release 工作流、Tauri 設定、`/release` 技能、CLI、引擎與 Desktop 的程式。

## Risks / Trade-offs

- [第一次用憑證簽章時，macOS 可能詢問 codesign 能不能使用這張憑證的私鑰；建置放在背景時，詢問框會讓 `tauri build` 停住] → 手動任務要求第一次在前景執行、看得到螢幕時重裝，詢問框出現時選「永遠允許」；之後的重裝（包含 `/release` 收尾放在背景的那一次）不會再問。
- [改完後第一次重裝，鑰匙圈還會對每個項目再問一次] → 這是從 ad-hoc 換到固定身分的一次性成本；提案的相容性影響已寫明，手動任務要求第二次重裝確認不再詢問。
- [Tauri 簽章時會印出「略過公證」的警告] → 屬預期行為，不視為失敗；設計 Non-Goals 已排除公證。
- [維護者的憑證過期或被撤銷] → `security find-identity -v` 只列有效身分，過期的憑證不會被選到，腳本退回 ad-hoc 並印出說明。
- [跨平台] → 簽章身分的查找與斷言只在 macOS 執行；Windows 與 Linux 的建置步驟與現在相同，回歸由既有的測試涵蓋。
