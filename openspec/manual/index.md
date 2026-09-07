---
title: Speclink 操作手冊
section: 開始使用
order: 10
keywords: [首頁, 入門, 角色, 導覽, SDD, 手冊]
sources: []
generated: 2026-09-07T13:20:04+08:00
---

# Speclink 操作手冊

Speclink 是一套 SDD（規格驅動開發，Spec-Driven Development）引擎：PM、PO、RD 與 AI agent 共用同一套「變更、產物、任務、驗證、封存」的語意，資料可以放在本機的 Local Repo，也可以放在團隊共用的 Remote Store。

## 三個核心觀念

1. **變更（change）與產物**：每一件要做的事都是一個變更。變更底下有提案、設計、任務清單與規格差異（delta）這些產物。任務勾完、品質關卡蓋章後，變更就能封存。
2. **正典規格與封存**：`openspec/specs/` 是系統現況的唯一真相。封存把變更的規格差異併進正典，之後所有人讀到的都是併入後的版本。
3. **本地與 remote 兩條路**：本地模式把一切存在你的 repo 裡，純 Markdown 與 YAML。remote 模式把正典放在 server 上的 Store，本機只有唯讀投影，團隊成員用帳號與 membership 共用同一份資料。

## 依角色選入口

### 今天剛加入，想先跑完一輪

1. [安裝 CLI 與桌面 app](install.md)
2. [建立工作區與指令檔](init-workspace.md)
3. [認識資料：變更、討論與規格](data-layout.md)
4. [工作流總覽：站別與交棒](workflow-overview.md)
5. 專案已有程式碼、還沒有規格：先做[基準盤點](baseline.md)
6. 照順序走：[提案](propose.md) → [實作](apply.md) → [品質關卡](quality-stations.md) → [封存](archive.md)

### 用 AI agent 跑 SDD 的開發者（RD）

- 既有專案第一次採用：[基準盤點](baseline.md)
- 需求還不清楚：[討論](discuss.md)
- 建立與實作變更：[提案](propose.md)、[實作](apply.md)
- 放了一陣子再回來、或需求中途變了：[續作與需求變更](drift-ingest.md)
- 交付前的把關：[品質關卡總覽](quality-stations.md)、[審查站](review.md)、[驗證站](verify.md)、[封存](archive.md)
- 好幾個變更同時做：[平行實作與合回](worktree.md)
- 工具：[提交單一變更的檔案](commit.md)、[溯源](trace.md)、[工作流政策與設定](policy-config.md)、[產出流程 schema 管理](schemas.md)、[操作手冊：生成與導覽](manual.md)

### 產品負責人／PM（看進度、不碰 checkout）

- [認識桌面 app](desktop-overview.md)、[看板與任務](desktop-board.md)
- [規格、討論、已封存與搜尋](desktop-browse.md)、[桌面 app 的手冊頁](desktop-manual.md)、[桌面上的品質關卡](desktop-quality.md)
- [專案分頁、新增工作區與設定頁](desktop-projects.md)、[自動更新、安裝 CLI 與指令檔過期](desktop-update.md)、[系統匣選單](tray.md)
- 接上團隊 server：[桌面連線 server 與 remote 工作區](desktop-remote.md)

### 團隊協作（remote 模式）

- 先讀 [remote 模式總覽](remote-overview.md)
- 開發者接上 server：[CLI 連接 remote](cli-remote.md)、[remote 模式的規格投影](remote-context.md)
- 既有本機資料搬上去：[本地工作區遷移到 remote](migrate.md)
- 網路斷了怎麼辦：[失聯與恢復](remote-offline.md)

### Server 管理員／部署者

1. [啟動 server](server-start.md)
2. [開箱：第一位 Admin 與第一組 Project／Repo](server-setup.md)
3. [帳號、邀請、存取金鑰與 membership](accounts.md)
4. 日常管理：[瀏覽器後台](web-console.md)、[管理面與稽核](server-admin.md)、[備份與還原](server-backup.md)

> [!NOTE]
> 本手冊只取材自 `openspec/specs/` 的正典規格，沒有截圖。畫面若與手冊不同，以實際產品為準。來源與已知矛盾請看[本手冊的來源](about.md)。手冊也能在桌面 app 的「手冊」頁閱讀，見[桌面 app 的手冊頁](desktop-manual.md)。

**出處**：本頁為導覽頁，不直接取材自單一能力；各頁末行列出自己的出處。
