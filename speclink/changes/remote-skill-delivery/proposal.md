## Why

技能檔是放在 checkout 裡、給 AI agent 讀的步驟說明。今天不論專案是本機還是遠端，技能檔一律由工程師電腦上的 CLI（或 Speclink Desktop 內建的引擎）用自己內建的範本產生。對遠端專案，這有四個問題：

- **技能和執行它的引擎可能不同版。** 遠端專案的動詞由 Server 上的引擎執行。技能的文字卻來自每個人電腦上的 CLI，每個人的版本不一樣，技能叫 agent 做的步驟就可能和 Server 的引擎對不上。
- **遠端專案永遠拿不到 worktree 的兩支技能。** 要不要產生它們，看的是本機規格資料夾裡的 `config.yaml`。遠端專案的 checkout 沒有這個檔，結果永遠是「不產生」，即使 Server 上的流程設定已經開啟 worktree。
- **Speclink Desktop 對遠端專案完全不檢查技能檔。** 技能檔缺少或過期時，使用者不會收到任何提示。
- **用戶端不知道 Server 的技能版本。** 握手回應裡的版本是套件的版本，不是技能範本的版號；而且沒有任何用戶端的程式讀它。

討論 embedded-host-needs 的裁定是：遠端專案的技能由遠端提供。入口用引擎內建的範本渲染，宿主只能選擇提供哪幾支，不能改文字；用戶端安裝後仍然是 checkout 裡的檔案。

- **目標使用者**：連到遠端專案的開發者；架設宿主的整合者（決定提供哪些技能）。
- **使用情境**：第一次把一個 checkout 連到遠端專案之後、Server 升級之後、Server 上的 worktree 政策改變之後——執行 `speclink update`，或在 Speclink Desktop 按「更新技能檔」。
- **既有規格的對照**：workspace-tools 規定技能的產生、worktree 技能依本機的 `config.yaml` 產生、降級守門、技能檔過期的探測，全部以本機的引擎為基準。desktop-app「指令檔過期提示」明寫不對遠端專案探測。host-kit 與 client-protocol 沒有任何關於技能的路由或欄位。node-sdk 的 `createHost` 沒有技能的設定。

Source discussion: embedded-host-needs（cut 13）。

## What Changes

**Server 這一邊（宿主套件的 HTTP 入口）**

- 握手回應的 `capabilities` 新增選填的 `skills`：`version` 是這台 Server 的技能範本版號。沒有這個欄位的 Server 被用戶端視為「不提供技能」。
- 新增一條唯讀的路由：用戶端送出它要的工具清單（內建工具的名稱，或自訂工具的名稱與呼叫形式），入口回傳每個工具要安裝的技能——每支技能的名稱、內容的摘要，以及（要求時）全文。
- 產生哪些技能由三件事決定：工具（有些技能只給 Claude）、這個範圍的流程設定（worktree 開啟才有那兩支）、宿主建立宿主套件時給的技能清單（選填；沒有給就是全部）。
- 內容用引擎內建的範本，以「規格資料夾不在 checkout」渲染。宿主不能改文字。
- 建立宿主套件時新增一個選填的輸入：要提供的技能名稱清單。清單裡有不認得的名稱、或清單是空的時，建立就失敗。

**用戶端這一邊**

- `speclink update` 在遠端專案：Server 有宣告技能時，從 Server 取得技能並寫進 checkout；checkout 裡多出來的舊技能資料夾照既有的規則清掉。Server 沒有宣告（舊版 Server）或連不上時，照舊用本機的範本產生，並在 stderr 說明用的是本機的範本。
- 遠端專案的技能版本跟著 Server：Server 提供的版本比 checkout 裡的舊時也照裝，並印一行說明。`--allow-downgrade` 在這條路徑上不需要。
- CLI 的提醒：執行遠端動詞時，如果本機 CLI 的技能版號比 Server 宣告的舊，在 stderr 印一行提醒，內容有兩邊的版號與更新的方式。同一台 Server 一天最多一次，不擋操作，不影響 exit code。CLI 不做自我更新。
- `speclink init --store remote`：流程不變（先用本機的範本產生，讓還沒登入的人也有技能可用），結尾多一行提示登入後執行 `speclink update`。
- **CLI 指令**：`speclink update` 的旗標不變。stdout 的摘要多一行技能的來源（Server 或本機）與版本。exit code：成功 0；取得技能失敗而且沒有退路可用時非 0。沒有 stdin。

**Speclink Desktop**

- 對有 checkout 的遠端專案啟用技能檔的探測，基準是 Server 提供的技能（用摘要比對，不下載全文）。
- 提示卡與確認框沿用本機專案的那一套：
  - checkout 裡的技能和 Server 提供的不同 → 「更新技能檔」。
  - checkout 裡沒有技能 → 「安裝技能檔」。
  - Speclink Desktop 內建的技能版號比 Server 宣告的舊 → 「更新 Speclink」（仍然可以先安裝 Server 提供的技能）。
- 確認框多一列「Server 提供的技能版本」。
- 沒有 checkout 的遠端專案（只看規格）不探測。

**Node SDK**

- `createHost` 新增選填的 `skills`（技能名稱的陣列）。`host.handleRequest` 因此能回答技能的路由。

**相容性影響**

- 本機專案：沒有變化。
- 遠端專案連到新版 Server：`speclink update` 寫出的技能改由 Server 決定，worktree 的兩支會在 Server 開啟政策時出現。
- 遠端專案連到舊版 Server：行為不變（本機的範本），多一行 stderr 的說明。
- 舊版用戶端連到新版 Server：握手多一個它不認得的選填欄位，照常運作；技能照舊由本機產生。
- 示範用 server：宣告技能並回答新路由，提供全部技能。
- **設定欄位**：`speclink/config.yaml` 與 `.speclink.yaml` 沒有新增或變更。CLI 在使用者設定目錄多一個小的狀態檔，記錄每台 Server 上次提醒的日期。

## Non-Goals

- **宿主改技能的文字、加自己的技能**：討論的擴充規則不允許。
- **CLI 自我更新**：只提醒。
- **本機專案的技能由別處提供**：本機專案維持本機產生。
- **技能在執行期向 Server 取文字**：安裝後就是 checkout 裡的檔案，agent 讀的是檔案。
- **簽章或驗證 Server 提供的技能**：連到一台 Server 等於信任它；這一點寫進文件。
- **內嵌宿主自己的 Agent 怎麼拿技能**：它在宿主的程序裡，用既有的 `skills.render`。
- **依個人權限產生不同的技能**：技能檔會進版本控制，同一個範圍的每個人拿到的內容相同。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-kit`: 新增「入口提供這個範圍的技能」。
- `client-protocol`: 新增「握手宣告技能版本」與「技能清單的請求與回應」。
- `workspace-tools`: 新增「remote 專案的技能取自 Server」與「CLI 的技能版本提醒」；修改「worktree 技能的政策條件式生成」與「受管檔再生的降級守門」（兩者加上 remote 專案的例外）。
- `desktop-app`: 修改「指令檔過期提示」（對有 checkout 的遠端專案探測，基準是 Server）。
- `node-sdk`: 新增「createHost 的技能清單」。

## Impact

- Affected specs: host-kit（新增一條）、client-protocol（新增兩條）、workspace-tools（新增兩條、修改兩條）、desktop-app（修改一條）、node-sdk（新增一條）
- Affected crates 與 app: `speclink-core`、`speclink-host`、`speclink-protocol`、`speclink-remote`、`speclink-cli`、`speclink-server`、`speclink-node`、apps/desktop（core、Tauri 殼與前端）。
- Affected code:
  - New:
    - crates/host/speclink-host/src/http/skills.rs
    - crates/protocol/speclink-protocol/src/skills.rs
    - crates/protocol/speclink-remote/src/skills.rs
    - crates/adapters/speclink-cli/src/skill_notice.rs
    - crates/adapters/speclink-cli/tests/it/remote_skills.rs
  - Modified:
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/src/workspace/init/tests.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/http/mod.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-host/tests/station_wire.rs
    - crates/protocol/speclink-protocol/src/binding.rs
    - crates/protocol/speclink-protocol/src/lib.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/protocol/speclink-remote/src/lib.rs
    - crates/adapters/speclink-cli/src/verbs/init.rs
    - crates/adapters/speclink-cli/src/remote_base.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs
    - crates/adapters/speclink-cli/Cargo.toml
    - crates/host/speclink-server/src/state.rs
    - crates/host/speclink-server/tests/it/api/write_guard.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/handle-request.spec.ts
    - apps/desktop/core/src/project.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - apps/desktop/src/App.tsx
    - apps/desktop/src/adapter/workspace.ts
    - apps/desktop/src/components/AssetNoticeCard.tsx
    - apps/desktop/src/store.ts
    - apps/desktop/src/assetPrompt.ts
    - apps/desktop/src/components/AssetNoticeDialog.tsx
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/assetNotice.test.tsx
    - apps/desktop/src/__tests__/assetPrompt.test.ts
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/store.test.ts
    - apps/desktop/src/__tests__/helpers/assetFixtures.ts
  - Removed: (none)
- 相依：排在變更 skills-verbs-second-wave 之後（要用它的「規格資料夾不在 checkout」渲染）。
- 三種形態下的結果：
  - 本機 repo：不適用。本機專案的技能維持由本機產生。
  - 連到遠端的 checkout：可用。技能來自 Server，worktree 的兩支跟著 Server 的流程設定。已知缺口：連到沒有宣告技能的舊版 Server 時，仍然用本機的範本，worktree 的兩支不會產生。
  - 沒有本機規格檔的內嵌宿主：宿主經 `createHost` 的 `skills` 決定提供哪些技能給連進來的工程師。宿主自己的 Agent 不適用：它直接用 `skills.render`。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - Remote 入門：連上之後執行 `speclink update`；技能來自 Server；連到一台 Server 等於信任它提供的技能。
  - 「把 Speclink 內嵌進你的系統」與 Node SDK 文件：`createHost` 的 `skills`。
  - 設定說明：CLI 的提醒與它的狀態檔。
  - Speclink Desktop 的說明：遠端專案的技能檔提示。
