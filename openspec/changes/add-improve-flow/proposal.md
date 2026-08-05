## Why

speclink 的討論入口目前只有使用者發起——discuss 由使用者帶題目、搭配 codebase 展開。缺少反向入口:由模型檢視 codebase、主動提出架構改進提案,再走同一套討論與決策回歸 SDD。Matt Pocock 的 improve-codebase-architecture 技能已驗證此型態(scope-before-you-scan、friction 訊號、deletion test、candidates 分級、逐一 grill),而 speclink 已具備其全部下游機件——grilling ≈ discuss 的 interview 紀律、ADR ≈ 已封存討論的 Ruled out、架構詞彙 ≈ interface depth check——唯獨缺「模型播種決策樹」的前段。

目標使用者:透過 AI 代理跑 SDD 的開發者。使用情境:workflow 最上游,與 discuss 同層(discuss?/improve? → propose);使用者想改善 codebase 但沒有具體題目時,以 /speclink-improve 讓模型提出 candidates 再進入討論。

源自討論 architecture-improve-flow(3 rounds,結論已定案六步骨架與全部取捨)。

## What Changes

- **新增引擎技能模板** crates/speclink-core/assets/skills/improve.md,經 init/update 渲染為 claude 與 codex 兩份 /speclink-improve 技能檔。六步骨架:載入詞彙(同 discuss)→ 防重提檢查(讀已封存與開放討論、in-flight changes,已否決方案不得再提)→ 範圍收斂(使用者點名方向優先;否則 git log 熱點推斷、近期常變區域加權,輔以已封存變更的 touched 記錄;熱點分散則放寬網)→ 掃描(五條 friction 訊號逐條保留 Matt 原文、有機探索不逐條打勾、deletion test 為 candidate 准入判準;範圍窄走 inline,寬則派 Explore subagent 硬上限 2)→ 建記錄呈現 candidates(寫入既有討論記錄,Round 1 mode 標籤 scan,卡片欄位 Files/Problem/Solution/Wins/建議強度＋首選建議)→ grilling 收斂(discuss 的 interview 紀律,interface depth check 對每個被挑中的 candidate 無條件執行,經 conclude → promote/link 扇出變更)。技能僅由使用者發起,模型不得自行觸發
- **discuss new 增選配旗標 --kind**:白名單僅接受 improve,非法值以非零 exit code 拒絕且不落檔;合法時 frontmatter 增 kind 欄位。未帶旗標時人眼輸出與 --json 逐位元不變,回歸對照不受影響
- **協定 DiscussionInfo 增選填 kind 欄位**:camelCase,無值時省略序列化,舊 payload 與既有 client 相容
- **desktop 改進討論標示**:討論卡片行內小章(lucide 既有 icon 家族＋Tooltip,不加文字列維持極簡)與討論抽屜同步標示;i18n tw/en 詞條;正典詞「改進討論」入 openspec/LANGUAGE.md
- **文件同步**:CLAUDE.md 與 AGENTS.md 注入區塊的 workflow 段更新為 discuss?/improve? → propose(claude 與 codex 兩份生成檔,經 speclink update 落地);README.md 與 README.en.md 的 workflow 圖與 improve 一節

## Non-Goals

- 不做臨時 HTML 報告——candidates 呈現走對話與討論記錄,desktop 既有呈現面不新增
- 不動 rounds/conclude/promote/link/archive 等討論記錄機件——完全複用
- 不做每 candidate 一份記錄;不以 slug 前綴或 round mode 字串讓 GUI 推斷討論型別
- 不擴充 LANGUAGE.md 承載架構詞彙(seam/depth/adapter 是 agent 工作詞彙,留在技能內文)
- 不含行為正確性審查——improve 只管結構性 deepening,與 verify/audit/品質站分工不重疊
- 系統匣討論項不加改進標示(維持現狀)
- 不新增設定欄位(openspec/config.yaml 與 .speclink.yaml 皆不動)

## Capabilities

### New Capabilities

- `improve-skill`: /speclink-improve 技能模板的內容契約——六步骨架、Matt 原文精髓段(scope-before-you-scan、五條 friction 訊號、deletion test)逐條保留不得濃縮、Explore subagent 硬上限 2、candidates 卡片欄位;渲染產物由 render golden 保護

### Modified Capabilities

- `discussion-docs`: discuss new 新增 --kind 旗標(白名單驗證)與 kind frontmatter 欄位;未帶旗標輸出不變
- `client-protocol`: DiscussionInfo 增選填 kind 欄位,list/show 的 --json 曝露
- `desktop-app`: 改進討論的看板卡片小章與討論抽屜標示

## Impact

- Affected specs: improve-skill(新增)、discussion-docs、client-protocol、desktop-app
- Affected code:
  - New: crates/speclink-core/assets/skills/improve.md
  - Modified: crates/speclink-core/src/discuss.rs、crates/speclink-cli/src/main.rs、crates/speclink-protocol/src/query.rs、crates/speclink-core/tests/golden(乾淨樹再生)、packages/ui/src/adapter.ts、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts、openspec/LANGUAGE.md、README.md、README.en.md、CLAUDE.md(注入區塊,經 speclink update 再生)、AGENTS.md(同前)
  - Removed: (none)
- 相容性影響:discuss new 未帶 --kind 時人眼輸出與 --json 逐位元不變;DiscussionInfo 的 kind 為選填且無值省略,既有 client 不受影響;舊討論記錄無 kind 欄位即一般討論,零遷移
- 影響的 crate 與 app:speclink-core、speclink-cli、speclink-protocol、packages/ui、apps/desktop
- 影響的技能與工具:新增 speclink-improve(claude/codex 兩份);CLAUDE.md/AGENTS.md 注入區塊 workflow 段
