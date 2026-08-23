## Summary

全面取消 init／update 對指令檔（CLAUDE.md、AGENTS.md、custom tool 的 instructions_file）的 SPECLINK marker 區塊注入，路由職責全數改由技能承載：description 改寫為觸發情境句管入口、各技能結尾補「建議下一步、絕不代跑」交棒句管出口，`speclink update` 自動剝除既有專案的 marker 區塊。

## Motivation

Speclink 的動態指示機制（status＝artifact DAG、instructions＝逐 artifact 範本與 preflight 入場檢查）已與 OpenSpec 1.0 的「dynamic instructions」架構同構，marker 區塊是疊在其上的靜態重複層：路由表與技能 description 重複、workflow 政策 bullet 與引擎閘門（archive 拒絕 worktree、apply preflight 漂移警告）重複、store 路徑段與技能本文的 {{SPEC_DIR}} 代換重複、custom tool 的 invocation 句與每份 SKILL.md 內嵌的 Invocation 前言重複。每次資產改字都觸發 MARKER_VERSION→golden→assets.lock 三連動並波及所有使用者 repo 的指令檔重注入。OpenSpec 1.0 已實證無注入路線可行（CHANGELOG 1.0.0 明文移除 CLAUDE.md／.cursorrules／AGENTS.md 生成，AGENTS.md 系工具改用 vendor-neutral skills 目錄）。（源自討論 init-marker-openspec-alignment 的結論；全面取消、自動剝除、完全去中心化、一個 change 全包均為該討論定案。）

## Proposed Solution

1. **取消注入**：Node SDK 的 `instructions.render()` 隨兩支 body 函式一併移除（公開 API 的 breaking change——自建 harness 的 system prompt 路由改由技能檔 description 承載，與其他 host 同一條路）；init（fs 與 remote）、update、tools 收斂、desktop 初始化與工具同步不再生成任何 SPECLINK:START..END 區塊；受管生成物僅剩技能檔、.speclink.yaml 與 .gitignore 條目。custom tool 描述子的 instructions_file 欄位轉為選填且不生效（保留欄位僅為舊設定檔可解析，validate 提示已棄用）。
2. **遺留剝除**：`speclink update` 對所有工具目標（內建與描述子）偵測既有 marker 區塊並自動剝除——只動區塊、保留使用者內容，剝除後全空的檔案刪除（沿用既有 tools 移除時的剝除語意，擴大到「更新時一律剝」）。
3. **入口路由＝description 觸發情境句**：18 個對外技能的 registry description 改寫為「情境 → 用我」句式（如 apply：tasks 就緒要實作或恢復做到一半的變更時使用），涵蓋原 marker 路由表的每一條 bullet；host 的技能清單即路由表，不留任何集中式流程總表。
4. **出口路由＝交棒句**：依討論第 5 輪邊集，流程鏈技能（propose、apply、apply-with-worktree、worktree-merge、drift、ingest、review、verify、quality、archive）結尾補狀態相依的「建議下一步」段，明文只建議、絕不代跑；入口技能（onboard、discuss、improve）出口指向流程鏈開頭；工具技能（commit、analyze、audit、config、trace）無固定出邊；內部技能（tdd、clarify、sync）不參與路由。原 marker 的 workflow 政策 bullet 各自歸位：drift-first 由 apply preflight 承載、worktree 品質站位置與 archive 主 checkout 限制由 worktree 技能交棒句載明、plan mode→ingest 由 ingest description 承載。
5. **過期探測改基準**：marker 消失後，工作區產物層版號的探測（CLI update 的 stale 判定與 desktop 的過期提示）改以技能檔 frontmatter 的 version 欄位為準；降版拒絕（refuse_downgrade）同步改讀技能版本。MARKER_VERSION 常數更名為技能產物層版號（語意不變：管技能檔版本與 golden）。
6. **文件同步**：docs/getting-started、configuration、verb-contract、platform-architecture、server-store-drivers、implementation-refactor-roadmap（含各 zh-TW 版）中 marker 注入的描述改為技能路由描述。

## Non-Goals

- 不新增 continue 式「一次建一份 artifact」技能（討論裁定為獨立功能決策，不在本 change）。
- 不保留任何形式的瘦身 marker 或集中式流程總表（CLI 印全流程、onboard 放命令表均已否決——第二份要同步的真相）。
- 不改動態指示機制本身（status／instructions／preflight 的 payload 與行為不動）。
- 不做 marker 剝除的互動確認（討論裁定自動剝除，比照 OpenSpec 的 legacy cleanup）。
- 不回改已封存 change 或歷史文件中對 marker 的描述。

## Alternatives Considered

- **瘦身保留極小 marker（只剩 store 路徑段）**：路徑已透過 {{SPEC_DIR}} 代換烙進技能本文，且 remote 模式由 instructions payload 給 contextFiles，保留即冗餘——否決。
- **custom tool 分層保留注入**：render_skill_file_custom 已為每份 SKILL.md 內嵌 Invocation 前言，marker 的 invocation 句重複——否決。
- **拆兩個 change（先技能路由、後拆 marker）**：中間態存在雙重路由來源（marker 與 description 各說各話）——否決。

## Impact

- 既有規格掃描：需求正文命中 workspace-tools（工具檔生成與 marker 場景最密）、desktop-config（初始化對話框與 tools 同步）、remote-connection（remote init 生成物與「指令區塊的 remote 變體」需求）；verb-contract、store-abstraction、workspace-migration、remote-workspace-data 僅於 @trace 歷史清單或不同概念（remote marker＝.speclink.yaml remote section）命中，需求正文無涉、不需 delta。跨技能路由（description 觸發句＋交棒句邊集）無既有規格涵蓋——per-skill 規格各管單一技能內容，workspace-tools 只管生成機制，故立新 capability skill-routing。
- Affected specs: skill-routing（新增）、workspace-tools、desktop-config、remote-connection
- Affected code:
  - Modified: crates/speclink-core/src/init.rs、crates/speclink-core/src/skills.rs、crates/speclink-core/src/config.rs、crates/speclink-core/assets/skills/（18 份對外技能資產的 description 與交棒句）、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-core/tests/golden/（marker 快照移除、技能快照再生、assets.lock）、crates/speclink-cli/src/main.rs、crates/speclink-node/src/render.rs、crates/speclink-node/index.js、crates/speclink-node/index.d.ts、crates/speclink-node/__test__/render.spec.ts、crates/speclink-cli/tests/it/（update_downgrade_guard、engine_version、workflow_config、remote_connect、tools_descriptor 等 marker 斷言面）、apps/desktop/core/src/project.rs、apps/desktop/core/src/settings.rs、apps/desktop/src-tauri/src/connections.rs、apps/desktop/src/i18n/messages.ts、apps/desktop/src/instructionPrompt.ts、apps/desktop/src/store.ts、docs/getting-started.md、docs/getting-started.zh-TW.md、docs/configuration.md、docs/configuration.zh-TW.md、docs/platform-architecture.zh-TW.md、docs/sdk-node.md、docs/sdk-node.zh-TW.md（verb-contract、server-store-drivers、implementation-refactor-roadmap 三處的「marker」屬 worktree 標記／store 標記／stale marker 等不同概念，逐檔確認語境後不需改動）
  - New: （無新檔——skill-routing 為規格 capability，程式面無新模組）
  - Removed: crates/speclink-core/tests/golden/remote-claude.marker.md、本 repo 的 CLAUDE.md（整份僅有 marker 區塊，剝除後全空即刪；AGENTS.md 的區塊剝除後保留使用者段落）（唯一獨立 marker golden；claude.snapshot.md 等多檔渲染快照不刪除，於再生時失去 CLAUDE.md／AGENTS.md marker 段）
