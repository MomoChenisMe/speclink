# user-documentation Specification

## Purpose

TBD - created by archiving change 'unify-user-documentation'. Update Purpose after archive.

## Requirements

### Requirement: 使用者文件採漸進揭露與單一責任

Speclink 使用者文件 SHALL 以 README、getting-started、workflow、product-status、平台架構與實作路線圖形成漸進揭露入口。README SHALL 保留品牌圖片、標語「一套 SDD Engine，支援 Local Repo 與 Remote Store」、語言切換、Rust SDD 引擎與工具平台定位、PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意、Local Repo／Remote Store 雙路徑、Spectra App 2.3.1 CLI 行為參考與 parity／golden tests 相容基線，並 SHALL 提供由 product-status 校正的目前狀態摘要、最短流程心智模型、Local Repo 開始入口與文件地圖。getting-started SHALL 只承載可直接完成的 Local Repo 第一輪，workflow SHALL 作為完整使用流程正典，product-status SHALL 作為目前能力狀態正典，平台架構 SHALL 維持唯一目標架構正典，實作路線圖 SHALL 維持其下的交付順序伴隨文件。各文件 SHALL 以連結導向下一層細節，SHALL NOT 在 README 或 getting-started 複製完整架構與狀態矩陣。

#### Scenario: README 保留專案定位與起源

- **WHEN** 使用者開啟繁體中文或英文 README 判斷 Speclink 是什麼及為何存在
- **THEN** 首段可見品牌圖片、SDD Engine 標語、語言切換、Rust 實作與共同流程語意、Local Repo／Remote Store 說明及 Spectra App 2.3.1 相容性起源，後續目前狀態清楚區分已可運作與分階段建置內容，並連到實作重構路線圖
- **AND** 文件整理只校正過時事實、術語與連結，不得將上述首段改成僅含導覽連結的入口

#### Scenario: 首次使用者由 README 到完成第一輪

- **WHEN** 首次使用者從繁體中文或英文 README 尋找安裝與第一輪 Local Repo 操作
- **THEN** README 提供可見的 getting-started 入口，getting-started 以目前存在的 CLI／skill 完成 init、提案、實作檢查與封存，遇到選用分支時可連到 workflow

#### Scenario: 進階使用者查詢目前能力與目標

- **WHEN** 使用者要判斷 Desktop Remote Workspace 或 Server 某項能力目前是否可用及最終目標
- **THEN** README 導向 product-status 取得目前狀態、證據與限制，並由該列導向平台架構或實作路線圖取得目標與交付順序，兩者 SHALL NOT 混成同一狀態描述


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: 完整工作流指南說明用途與使用時機

中英文 workflow SHALL 涵蓋 onboard、discuss、propose、apply、ingest、drift、analyze、validate、audit、commit、archive，以及目前可觀察的 verify／evidence 能力；每個階段 SHALL 說明目的、使用與跳過時機、輸入、產物、Agent skill 與底層 CLI／Host 的呼叫層級、完成判準、下一步與常見恢復方式。workflow SHALL 明確區分必經生命週期階段、條件式階段與 utility skill，並 SHALL 說明 skill 是工作流知識、CLI／Host 是執行引擎。

#### Scenario: 需求明確與需求模糊採不同入口

- **WHEN** 使用者比較一項已明確需求與一項仍需取捨的需求
- **THEN** workflow 指示前者直接 propose，後者先 discuss；若只是理解問題且沒有待決事項，SHALL 指示直接問答且不建立 discussion 記錄

#### Scenario: 續作與需求改變採不同入口

- **WHEN** 使用者要恢復一個閒置 change，或實作途中收到會改變 artifacts 的新需求
- **THEN** workflow 分別指示閒置 change 先 drift、需求改變走 ingest，且列出檢查結果如何回到 apply 或再次 ingest

#### Scenario: utility skill 不被誤列為生命週期必經步驟

- **WHEN** 使用者查詢 audit 或 commit 在流程中的位置
- **THEN** workflow 將 audit 說明為安全檢查、commit 說明為限定特定 change 檔案的 Git 工具，且 SHALL NOT 把兩者畫成每個 change 必經的狀態轉移


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: 討論結論後的轉出與併入分流完整

workflow SHALL 以決策表說明 discussion 結論後至少四條路徑：以 `$speclink-propose --from-discussion <slug>` 直接建立完整新 change、以 `speclink discuss promote <slug>` 快速轉為變更後再由 propose 補齊必要 artifacts、以 `speclink discuss link <slug> <change>` 連結既有 change 後由 ingest 反映內容並以 seal 標記已轉出，以及決定不做時直接封存 discussion。文件 SHALL 說明 promote 只建立 change 骨架並預填 proposal 的 Why，SHALL NOT 宣稱其結果已可直接 apply；一份 discussion 可轉出多個 change，其最後一個存活 change 封存時 discussion 依現行生命週期共行封存。

#### Scenario: 快速轉為變更不被誤認為完整提案

- **WHEN** 使用者比較 `discuss promote` 與 `$speclink-propose --from-discussion`
- **THEN** workflow 顯示前者只建立連結與 proposal 骨架、仍須 propose 補齊必要 artifacts，後者直接執行完整 artifact workflow 並在完成後可交給 apply

#### Scenario: 結論併入既有 change

- **WHEN** discussion 的結論要修正或補充一個已存在 change，而非建立新 change
- **THEN** workflow 指示依序執行 link、ingest、seal，並說明 link 只建立 change 側來源鏈、seal 只在內容已反映後標記 discussion 已轉出

#### Scenario: 決定不實作

- **WHEN** discussion 結論是明確不進行任何 change
- **THEN** workflow 指示封存 discussion 並保留推理記錄，SHALL NOT 建立空 change 或留下 concluded discussion 無限期停在待收尾狀態


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: Getting Started 僅使用已驗證入口

中英文 getting-started SHALL 提供一條可複製的 Local Repo 最短成功路徑；文件中的 CLI 子指令與旗標 SHALL 可由目前 `speclink --help` 或對應子指令 help 觀察，Agent skill 名 SHALL 存在於相應生成 surface。文件 SHALL 分別說明 Claude slash command、Codex `$skill` 與直接 CLI 的差異，SHALL NOT 將 optional artifact 寫成固定必產，SHALL NOT 將不存在的 skill 寫成可呼叫入口。結構檢查 SHALL 使用目前存在的 validate／analyze，實作驗證或 evidence 若無公開 skill 入口 SHALL 以目前限制標示並導向 product-status。

#### Scenario: Codex 使用者照入門文件操作

- **WHEN** Codex 使用者依 getting-started 建立並完成範例 change
- **THEN** 文件使用 `$speclink-*` skill 語法，底層 CLI 命令與旗標皆存在，且不要求呼叫未安裝的 `$speclink-verify`

#### Scenario: Claude 使用者照入門文件操作

- **WHEN** Claude 使用者依 getting-started 執行相同流程
- **THEN** 文件使用該 Host 生成的 slash command 語法，並與 Codex 版本產生相同 Speclink artifacts 與生命週期結果

#### Scenario: optional design 被正確說明

- **WHEN** 範例 change 不符合 design artifact 的建立條件
- **THEN** getting-started 說明 propose 只需完成 applyRequires 鏈上的必要 artifacts，design 可依指令條件跳過，SHALL NOT 宣稱每個 change 固定產出四份 artifact


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: 產品狀態以證據分類目前與規劃能力

中英文 product-status SHALL 對 Local CLI／skills、Local Desktop、Node SDK、Server／Store drivers、Desktop Remote Workspace、Agent／Protocol 整合與營運能力逐列標記 Available、Partial、Planned 或 Deprecated；繁體中文對應為可用、部分可用、規劃中、已棄用。每列 SHALL 包含使用者入口、至少兩項互相獨立的現行證據或一項可執行端到端證據、目前限制／下一步，以及最後查核日期。crate、canonical spec、已封存 change 或目標架構文字單獨存在 SHALL NOT 足以標記 Available。

#### Scenario: 已有底層實作但產品路徑未閉合

- **WHEN** 某能力已有 crate 與 canonical spec，但缺少完整使用者入口或端到端流程
- **THEN** product-status 將其標為 Partial 或 Planned，列出可用子集與缺少入口，SHALL NOT 僅因程式目錄存在寫成已完整交付

#### Scenario: 已棄用路徑仍存在

- **WHEN** legacy remote REST 路徑仍可在程式或歷史文件找到但已明確不是目標架構
- **THEN** product-status 將其標為 Deprecated，說明替代方向並連到目前目標架構，SHALL NOT 與新的 Remote Platform 混寫為同一可用能力

#### Scenario: 狀態查核可重做

- **WHEN** 維護者在之後的 checkout 重新更新 product-status
- **THEN** 文件提供查核日期與證據位置，使維護者可用當下 CLI help、skills、workspace members、tests／操作文件與正典 specs 重做判斷，而非沿用舊日期結論


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: 中英文文件保持結構與事實對等

`README.md`／`README.en.md`、`docs/getting-started.zh-TW.md`／`docs/getting-started.md`、`docs/workflow.zh-TW.md`／`docs/workflow.md`、`docs/product-status.zh-TW.md`／`docs/product-status.md` SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意及交叉連結。兩版 README SHALL 共用品牌圖片並保持標語、產品定位、共同流程語意、Local Repo／Remote Store、Spectra App 2.3.1 相容基線及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `openspec/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。

#### Scenario: 語言切換不遺失流程資訊

- **WHEN** 使用者在任一成對文件切換繁體中文與英文版本
- **THEN** 兩版呈現相同流程階段、決策分支、能力列與限制，只改變自然語言，不新增或遺漏事實

#### Scenario: 繁體中文採正典詞彙

- **WHEN** 繁體中文文件描述 discuss promote、promoted discussion 與 archive
- **THEN** 散文分別使用「轉為變更」「已轉出變更」「封存」，`promote`、`promoted`、`archive` 只在 CLI／欄位／code span 或必要的引擎動詞對照中出現


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: 目標架構與目前狀態維持清楚邊界

平台架構藍圖 SHALL 保持唯一目標架構基準與既有章節順序；實作重構路線圖 SHALL 保持該架構下的執行伴隨定位與既有 Phase／Gate 順序。兩者 SHALL 在文件開頭連到 product-status 取得目前可用能力，product-status SHALL 反向連到目標架構與路線圖；README 與 getting-started 中的「目前」敘述 SHALL 由 product-status 摘要而來，不得把 architecture 的未交付目標寫成可用操作。

#### Scenario: 閱讀架構藍圖不誤認為全部已交付

- **WHEN** 使用者開啟平台架構藍圖閱讀 Server、Remote Workspace 或 Agent 生態設計
- **THEN** 文件開頭明示其為目標架構並提供 product-status 連結，使用者可立即查到相應能力的目前狀態與限制

#### Scenario: 路線圖不成為第二份目標架構

- **WHEN** 使用者比較平台架構藍圖與實作重構路線圖
- **THEN** 兩者維持「唯一目標架構／其下執行順序」關係，既有 Phase 與 Gate 順序不因本次文件整理而重排


<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: 文件準確性具可重複驗證清單

本 change 的 tasks SHALL 記錄並執行文件查核：所有相對 Markdown 連結目標存在；中英文成對文件 H2 集合與順序一致；product-status 成對矩陣列一致且每列有證據、限制／下一步與查核日期；getting-started／workflow 的 CLI 命令與旗標可由目前 help 觀察；skill 名存在於相應生成 surface；繁體中文散文遵循正典詞彙。已確認缺失且刻意延後的文件 SHALL 以純文字缺口呈現，SHALL NOT 建立失效連結或空白 placeholder。

#### Scenario: 文件連結與語言對等查核通過

- **WHEN** 維護者執行 tasks 指定的相對連結與中英文 H2／矩陣結構查核
- **THEN** 所有實際連結目標存在、成對文件結構與狀態列一致，且查核以 exit code 0 完成

#### Scenario: 不存在的 skill 或旗標使查核失敗

- **WHEN** getting-started 或 workflow 把目前 help／生成 surface 不存在的 skill、CLI 子指令或旗標寫成可直接使用
- **THEN** 文件查核以非零結果指出該名稱，該文件 SHALL 在交付前修正為現行入口或明確的 Partial／Planned 限制

#### Scenario: 已知文件缺口不偽裝成完成

- **WHEN** 稽核發現 `docs/verb-contract.md` 等已被引用但未存在的進階文件且本 change 明確不補其內容
- **THEN** 文件地圖或 product-status 以無超連結的缺口項目記錄並導向後續 change，SHALL NOT 產生空檔、失效連結或宣稱其已交付

<!-- @trace
source: unify-user-documentation
updated: 2026-07-17
code:
  - README.en.md
  - README.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
  - packages/ui/src/__tests__/sonner.test.tsx
-->

---
### Requirement: Remote Getting Started 提供可重複的完整操作路徑

中英文 Remote Getting Started SHALL 以目前已交付的入口，提供從全新本地開發資料到可操作 Remote Desktop 與 Remote CLI 的端到端流程。文件 SHALL 依序涵蓋 `npm run dev:reset`、`npm run dev`、stdout 的 `/setup?token=...`、建立第一位 Admin 與第一組 Project／Repo、在 `/admin/users` 授予實際登入帳號 Project membership、以 `/account` 建立 PAT、Desktop device login 與 PAT fallback、Project／Repo scope 選擇、spec-only 與 checkout 分流、CLI `link`／`auth login`／讀寫 smoke test、一般重啟持久化、離線唯讀與恢復、完全重置。文件 SHALL 明確區分 Server base URL、瀏覽器帳號／管理頁 URL 與 project-scoped API URL，且 SHALL NOT 把架構目標或 Phase 4 能力寫成現行操作。

#### Scenario: 第一位 Admin 完成 Desktop Remote Workspace

- **WHEN** 使用者從沒有 `.dev/` 的本地 checkout 依繁體中文或英文教學啟動 Server 與 Desktop、完成 setup，並使用 setup 建立的 Admin 登入 Desktop
- **THEN** 教學指示使用者先在 `/admin/users` 為該 Admin 授予所建 Project 的 `editor` membership，再重新載入 Desktop scope 清單，選到所建 Project／Repo 並開啟 spec-only 或 checkout workspace

#### Scenario: Remote CLI 使用 project-scoped URL

- **WHEN** 使用者依教學在獨立測試資料夾連接 setup 顯示的 Project 與 Repo
- **THEN** 教學使用 `speclink link http://localhost:8080/api/speclink/v1/projects/demo --repo backend`、`speclink auth login` 與目前存在的讀寫動詞完成 smoke test，並警告不得在產品 repo 根目錄誤寫測試 binding

#### Scenario: 持久化、失聯與重置行為可觀察

- **WHEN** 使用者依教學停止並重啟 server，再執行完全重置
- **THEN** 教學分別要求觀察一般重啟不重印 setup token且資料與 remote tab 保留、失聯期間 snapshot 僅可讀且寫入被拒、恢復後以 Query／ETag 收斂，以及 `npm run dev:reset` 後重新出現全新 setup token

<!-- @trace
source: add-remote-getting-started
updated: 2026-07-24
code:
  - README.en.md
  - README.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/remote-getting-started.md
  - docs/remote-getting-started.zh-TW.md
  - docs/server-deployment.zh-TW.md
  - scripts/remote-docs.test.mjs
-->

---
### Requirement: 帳號、PAT 與 membership 的操作邊界明確

Remote Getting Started 與架構交叉連結 SHALL 指示瀏覽器開啟 `/account` 管理自身 PAT，由該頁表單 POST `/account/tokens`；文件 SHALL 明說直接以 GET 開啟 `/account/tokens` 會得到 HTTP 405 Method Not Allowed。文件 SHALL 說明建立 Project／Repo registry 不會授予 membership，Server Admin 身分不會繞過 Project membership；管理員 SHALL 由 `/admin/users` 對帳號授予或更新 `reader`／`editor`，Desktop scope 清單才顯示該 Project 及其 Repos。PAT 明文只顯示一次，文件 SHALL NOT 以 URL、repo 設定或帶值的 shell argument 示範保存 PAT。

#### Scenario: 使用者直接開啟 PAT action URL

- **WHEN** 使用者在瀏覽器網址列開啟 `http://localhost:8080/account/tokens`
- **THEN** 故障排除指出 405 代表該路由只接受表單 POST，並導向 `http://localhost:8080/account` 登入後由 Personal Access Tokens 表單建立 PAT

#### Scenario: Desktop scopes 回傳空清單

- **WHEN** Desktop 顯示「此帳號目前沒有任何 Project／Repo membership」但 registry 已有 Project 與 Repo
- **THEN** 故障排除指示管理員到 `/admin/users` 對 Desktop 實際登入帳號授予該 Project 的 `reader` 或 `editor`，說明 Admin flag 不是 scope bypass，並要求回到 Desktop 重新載入 chooser

#### Scenario: PAT 不進入不安全載體

- **WHEN** 使用者選擇 Desktop PAT fallback 或 CLI `auth login`
- **THEN** 教學要求從 `/account` 複製只顯示一次的 PAT並貼入應用或 stdin，不把真實 PAT 放進 URL、`.speclink.yaml`、repo、文件範例或 shell history

<!-- @trace
source: add-remote-getting-started
updated: 2026-07-24
code:
  - README.en.md
  - README.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/remote-getting-started.md
  - docs/remote-getting-started.zh-TW.md
  - docs/server-deployment.zh-TW.md
  - scripts/remote-docs.test.mjs
-->

---
### Requirement: Remote 教學具雙語導流與可重複查核

README 與產品能力狀態的中英文版本 SHALL 連到對應語言的 Remote Getting Started，既有繁中 Server 部署指南 SHALL 連到繁中 Remote Getting Started；兩份新教學 SHALL 維持相同 H2 集合與順序、命令語意、網址角色、membership 規則、故障排除症狀集合及目前能力邊界。文件查核 SHALL 驗證所有相對 Markdown 連結目標存在、現行 Server 路由與 CLI 指令可由 source／help 觀察、繁中與英文 H2 對等，並 SHALL 驗證關鍵字串涵蓋 `/account`、POST `/account/tokens`、`/admin/users`、HTTP 405、membership、project-scoped URL、spec-only、checkout、offline 與 `npm run dev:reset`。

#### Scenario: 使用者從既有文件找到 Remote 教學

- **WHEN** 使用者從任一語言 README、任一語言產品能力狀態或繁中 Server 部署指南尋找 Remote Server／Desktop／CLI 的首次操作方式
- **THEN** 文件提供有效連結到同語言 Remote Getting Started，而平台架構仍只作目標與安全邊界的正典

#### Scenario: 雙語與入口查核阻止文件漂移

- **WHEN** 維護者執行 tasks 指定的雙語 H2、相對連結、路由、CLI surface 與關鍵流程檢查
- **THEN** 全部檢查以 exit code 0 完成；任一語言缺少步驟、連到不存在檔案、把 `/account/tokens` 寫成 GET 頁面或引用不存在指令時以非零結果指出缺口

<!-- @trace
source: add-remote-getting-started
updated: 2026-07-24
code:
  - README.en.md
  - README.md
  - docs/platform-architecture.zh-TW.md
  - docs/product-status.md
  - docs/product-status.zh-TW.md
  - docs/remote-getting-started.md
  - docs/remote-getting-started.zh-TW.md
  - docs/server-deployment.zh-TW.md
  - scripts/remote-docs.test.mjs
-->