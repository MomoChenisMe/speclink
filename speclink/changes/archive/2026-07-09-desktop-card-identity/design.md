## Context

discuss 卡（DiscussionColumn）現以 topic 為題、slug 完全不上看板；change 卡（ChangeCard）以 change.name（kebab）為題並帶複製鈕，author 頭像僅在詳情抽屜。change 的 createdBy 由 newcmd 以 util::git_identity 蓋章；discuss new 目前不蓋作者章（DiscussionItem 無 createdBy）。change 卡的「來自討論」徽章與 restale 徽章以原生 title 提示，shadcn Tooltip 元件已存在。承 discuss desktop-card-identity-and-meta 結論。

## Goals / Non-Goals

**Goals:**

- discuss 卡與 change 卡身分呈現對齊（皆以 kebab 檔名為題），且 slug 可見可複製（CLI 動詞把手）。
- 討論記錄可溯建立者，服務本機小團隊與遠端。
- change 卡加建立者頭像、關係徽章以 hover 提示呈現。

**Non-Goals:**

- 不改變 open/concluded 卡的動詞與 promoted 群組呈現（那屬其他變更）。
- 不給討論加 started_at（討論無開工概念，其生命週期已由階梯呈現）。
- 不把抽屜整組 meta 搬上 change 卡（僅加 author 頭像與關係提示）。
- 不改 core 的 analyzer／IPC 契約語意（僅新增 created_by 欄位曝露）。

## Decisions

### D1：discuss 卡以 slug 為題、topic 為描述，並記為詞彙受控例外

discuss 卡標題改為 slug（檔名），topic 降為卡身次要描述，加複製 slug 鈕。此與「slug 不出現於使用者可見文案」原則刻意抵觸，記為受控例外於 openspec/LANGUAGE.md（比照 config.yaml 頁簽先例：開發者工具中檔名即最直觀的心智模型、且是 CLI 動詞把手）。

- 替代：維持 topic 為題、slug 全隱藏——否決：與 change 卡不對稱，且 CLI 把手（--from-discussion slug）取不到。

### D2：引擎替討論蓋 createdBy，比照 change 的 git_identity 機制

discuss new 於 scaffold frontmatter 蓋 created_by，取自 util::git_identity（比照 newcmd 對 change 的作法）；git 身分不可得時省略。經 discuss list／show --json 以 camelCase createdBy 曝露，DiscussionItem 帶 createdBy，抽屜與卡片顯示「誰發起」。

- 替代：討論加 started_at／started_by——否決：討論無開工階段，類別錯置。

### D3：change 卡加建立者頭像，關係徽章以 hover 提示呈現

change 卡新增 createdBy 頭像（比照抽屜的首字母圓標）；「來自討論」與「同源」關係於 hover 以主題化提示（shadcn Tooltip）呈現對應資訊，取代原生 title。提示內容不變（來源討論清單／同源變更）。

- 替代：把 author、agent、started 全搬上卡——否決：破壞卡片極簡、看板變雜。

## Implementation Contract

- 行為：discuss 卡標題為 slug、其下以次要文字呈 topic 描述、複製鈕複製 slug；discuss 卡與抽屜顯示建立者（createdBy）。change 卡顯示 createdBy 頭像；「來自討論」與「同源」關係於 hover 以提示呈現。discuss new 於有 git 身分時於記錄 frontmatter 蓋 created_by、無身分時省略；discuss list／show --json 以 createdBy 曝露。
- 介面／資料形狀：core::discuss 的 new 以 util::git_identity 蓋 created_by frontmatter；DiscussionItem 新增 createdBy 欄位（--json camelCase）；desktop core query 讀 created_by；tauriDataSource 與 packages/ui adapter 傳遞。UI 以既有頭像樣式與 shadcn Tooltip 呈現。無新 Tauri command。
- 失敗模式：git 身分不可得 → created_by 省略、頭像／建立者標示缺席（比照 change）；無來源討論／同源 → 關係提示缺席。
- 驗收：core 測試驗 discuss new 於有 git 身分時蓋 created_by、無身分時省略且 --json 帶／不帶 createdBy；packages/ui 測試驗 discuss 卡以 slug 為題＋topic 描述＋複製 slug、change 卡 author 頭像與關係 hover 提示。openspec/LANGUAGE.md 記受控例外。驗證：`cargo test -p speclink-core`（Windows 如遇 cdylib 連結問題以 `--lib` 限縮）、`npm test -w packages/ui`。
- 範圍邊界：in scope＝discuss created_by 蓋章與曝露、discuss 卡身分（slug 題／topic 描述／複製）、change 卡 author 頭像與關係提示、LANGUAGE.md 例外。out scope＝open/concluded 卡動詞與 promoted 群組呈現、started_at、抽屜其餘 meta、analyzer／IPC 語意。

## Risks / Trade-offs

- [「討論於看板第 0 欄兩級呈現」被多個變更同修] → 本變更僅改 discuss 卡身分（slug 題／topic 描述／複製／建立者）；promoted-discussion-toggle 改 promoted 開關與 chip、desktop-verb-drawer-surface 移除 concluded 卡轉為變更動詞。三者同修此需求，apply 時 SHALL 對後套用者跑 drift 對齊，避免全需求重現互相覆蓋。
- [slug 為題違反詞彙原則] → 緩解：以 LANGUAGE.md 明文受控例外承接（比照 config.yaml 先例），範圍限 discuss 卡標題，其餘使用者文案仍禁工程詞。
