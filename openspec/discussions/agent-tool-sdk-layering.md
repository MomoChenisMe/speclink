---
topic: core 引擎、npm SDK 與 agent 工具層的分層——讓 pi、Copilot SDK 等 runtime 把 speclink 掛成工具
slug: agent-tool-sdk-layering
status: open
created: 2026-09-14
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: core 引擎、npm SDK 與 agent 工具層的分層——讓 pi、Copilot SDK 等 runtime 把 speclink 掛成工具

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者想讓 speclink 走 pi（pi.dev）那種「核心引擎＋SDK＋工具／extension」的分層，讓 pi agent、Copilot SDK 等 runtime 能把 speclink 掛成工具呼叫。偵查結果：三層裡前兩層已成立——speclink-core 是唯一流程實作，CLI／桌面（apps/desktop/core）／server／Node 四入口共用；@speclink/engine 0.4.0 已在 npm（docs/product-status 與 docs/sdk-node 仍寫「registry 上還沒有」，文件過期）。CLI 已可遠端（31 個動詞在 dispatch 表宣告模式）。缺的是第三層：把引擎包成 agent runtime 能掛的工具。要求已夠明確（可驗證目標＝在 pi／Copilot SDK 裡註冊一顆 speclink 工具並呼叫動詞），未經 grill 直接進假設。
相關正式規格：node-sdk、node-sdk-release、command-runtime、host-runtime、client-protocol、server-verb-api、skill-routing。相關文件：docs/design/platform-architecture.zh-TW.md §4.1／§8、docs/roadmap.zh-TW.md「Agent 工具整合」、docs/product-status.zh-TW.md 第 46／59 列。
Prior discussions: node-sdk-completion-and-doc-alignment（Deferred 的 MCP／Copilot tools adapter，觸發條件「非 Claude/Codex 平台要接」現已成立）、improve-wire-convert-seam（共層落點規則：第二個消費端出現才搬）、cli-mode-dispatch-convergence（CLI 遠端已落地）、improve-repo-layout（crates 五組分層）

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-14)

**Focus**: pi 式三層（引擎／SDK／工具）中，speclink 已有哪些、真正缺哪一層
**Position**: 引擎與 npm SDK 已成立，這輪要立案的是 agent 工具層；決策樹如下：
- A 核心是否再拆：不用。speclink-core 是唯一實作，CLI／desktop／server／node 共用（Cargo.toml members；正式規格 command-runtime、host-runtime）
- B JS SDK 形狀：現況只有 dispatch(argv, stdin) 與 skills.list／render；是否補型別化方法待決
- C 工具層形狀：一份 runtime 無關的工具描述（name／description／inputSchema／handler）＋各 runtime 薄殼；殼不成套件、先以文件範例交付（先前已否決獨立 @speclink/host 套件；十行轉發殼過不了刪除測試）；落點建議 @speclink/engine 增 createTool(engine) 匯出
- C 粒度：一顆 speclink 工具收 argv＋stdin vs 每動詞一個工具——待使用者裁定
- D 工具的 handler 接哪裡：in-process 引擎優先，遠端 HTTP 待決
- E 技能與工具分工：工具給動詞、技能給流程；node-sdk 渲染 API 已有 invocation: 'tool-call' 選項
- 順帶：product-status 第 46 列、sdk-node 文件「registry 上還沒有套件」與 npm view 0.4.0 不符，須修
- 觸發條件：node-sdk-completion-and-doc-alignment 的 Deferred 寫「有非 Claude/Codex 的 agent 平台要接」再做工具層，使用者點名 pi 與 Copilot SDK，條件成立
- 詞彙：使用者說的「core 核心」正典叫「引擎」
**Ruled out**: 再拆核心——已是單一引擎，再拆只多一層轉發；每個 runtime 一個獨立套件——殼只有轉發、刪掉沒有東西壞
**Open**: 工具粒度（一顆 argv 工具／三分查詢命令 context／每動詞一個）；是否補型別化 JS SDK（B2）；工具 handler 是否要接遠端 HTTP；工具落點（@speclink/engine 匯出 vs 新套件）；pi extension／Copilot SDK／MCP 三個 runtime 的優先序

### Round 2 — interview (2026-09-14)

**Focus**: 「工具先走 in-process」是否等於只支援本地，遠端 HTTP 與自家儲存系統能不能接
**Position**: in-process 說的是引擎跑在 agent 同一行程，與規格存在哪裡是兩個軸；工具層 handler 先接 in-process 引擎，遠端 HTTP client 列為後續獨立的刀（使用者接受）。
- 今天可用：in-process＋本地 openspec 資料夾（createEngine fs store）、in-process＋自家資料庫（createEngine 宿主 Store 物件）——後者正是藍圖 §8 畫的 Copilot SDK 主場
- 今天不可用：工具 → HTTP → speclink-server。speclink-remote 是 Rust crate，消費端只有 CLI／desktop／server，JS 端沒有 client-protocol 實作
- 未來要接 HTTP：另做 JS 版 client（照 client-protocol 與 server-verb-api），工具描述不變、只換 handler 目標；先做 HTTP 得先解 JS 端認證與存取金鑰流程
**Ruled out**: 本輪一併做遠端 HTTP client——藍圖把同系統 agent 定位為 in-process 路徑，HTTP 屬外部 client；工具描述 runtime 無關後 handler 可事後換
**Open**: 工具粒度（一顆 argv 工具／三分／每動詞一個）；是否補型別化 JS SDK；工具落點；三個 runtime 優先序

### Round 3 — interview (2026-09-14)

**Focus**: Node SDK 是否已是給 pi／Copilot SDK 的那層；嵌入式 agent 框架（可能無專案概念、不知何時 init）如何綁定專案與注入技能
**Position**: Node SDK 就是那層，「Agent 工具層」不成新層也不成套件；嵌入式 agent 不需要 init，綁定寫在 createEngine 那行，技能以 skills.render() 文字投遞。
- init 不是引擎動詞，是 CLI 門的宿主動作（init.rs:84）：建 openspec 樹＋config、寫 .speclink.yaml、寫技能檔三件事
- 實測：空目錄 createEngine 後直接 new change 成功，引擎自建 openspec/changes/、用預設 schema——樹與 config 不必預建
- .speclink.yaml 是 CLI／桌面「從 cwd 往上找專案」的把手；嵌入式宿主在程式碼裡指定 store，不用找（藍圖 §4.7：Copilot SDK／In-process Tool 由 tool closure 綁定 session 的 actor／project／repo）
- 框架無專案概念＝一顆 engine 對一份 store；有專案概念＝每專案一顆 engine；時機＝開 session 時，屬宿主決定
- 技能注入三型：有 skills 目錄（pi 讀 .pi/skills、~/.pi/agent/skills、--skill、settings.skills，自帶「列描述、用到才讀本文」）→啟動時寫入渲染結果，frontmatter 相容；只有 system prompt→全塞（19 份太大）或加 speclink_skill(name) 工具按需回本文；Copilot SDK 官方 skills 文件 404 未查證，暫依第二型
- 修正後交付面：@speclink/engine 增匯出 speclink 工具名片（description＋JSON schema，與 tool-call 前言同源）；可選 speclink_skill 第二顆工具；sdk-node 新節「接進自訂 agent」含 pi／Copilot SDK 範例；修 product-status／sdk-node 的「npm 上還沒有」過期句
- 粒度順勢收斂：要注入的東西越少越好，一顆 speclink 總機（argv＋stdin）＋可選 speclink_skill 是最少組合
**Ruled out**: 為嵌入式 agent 新增 init 動詞或 SDK 端 init API——引擎首次寫入自建結構、.speclink.yaml 對嵌入式無意義；獨立 Agent 工具套件——十行殼過不了刪除測試（使用者亦覺得多一層奇怪）；把 19 份技能全塞 system prompt——體積過大，按需載入是 pi／Claude Code 既有作法
**Open**: 使用者是否接受「不用 init、綁定寫在程式碼、技能文字投遞」解掉其顧慮；speclink_skill 第二顆工具做不做；Copilot SDK 的技能／指令注入方式待查證；type: 'fs' 空目錄「靜默自建」是否需要在文件寫明；文件過期修正是否併入同一 change

### Round 4 — interview (2026-09-14)

**Focus**: 技能如何進入自訂 agent 框架，以及有哪些參考專案
**Position**: 格式已有開放標準（Agent Skills，agentskills.io），speclink 渲染輸出已符合；送達分安裝器／套件／執行期三種模式，各有參考專案，speclink 已有前者、缺套件模式、執行期模式缺文件。
- Agent Skills 格式：資料夾＋SKILL.md，frontmatter 必填 name／description，選填 license／compatibility／metadata／allowed-tools；漸進載入（啟動只讀名稱與描述，用到才讀本文）。speclink 渲染的五個 frontmatter 欄位逐一對應
- 支援方超過四十個（pi、GitHub Copilot、VS Code、Cursor、Gemini CLI、OpenCode、Goose、Kiro、Claude Code、Codex 等）
- 安裝器模式：參考 Laravel Boost `boost:install`（選 agent 寫技能與 MCP 設定，新 agent 以 SupportsSkills class 接入）；speclink init 的 tools 自訂描述子 {name, skills_dir, invocation: tool-call} 已等價（docs/configuration.zh-TW.md:85）
- 套件模式：參考 pi packages（package.json 的 pi.skills／pi.extensions，`pi install npm:@foo/bar`，專案 settings 自動補裝）；speclink 無此套件，若做＝19 份技能檔＋一個註冊工具的 extension
- 執行期模式：參考 Copilot SDK session 選項 skillDirectories（功能索引列有，細節文件未抓到）；speclink 有 skills.render()，缺「宿主寫目錄再傳路徑」的文件範例
- 意外發現：pi 讀 .agents/skills/，speclink codex 目標正寫此目錄——已 init 專案裡的 pi 今天就看得到技能（CLI 呼叫版）
- 兩種情境：有 checkout 的 agent 走安裝器（加 tool-call 描述子即可改走工具）；無專案概念的嵌入式 agent 走執行期（開 session 時 render 進目錄），無技能入口者用 speclink_skill 工具按需回本文
**Ruled out**: 另造一套技能格式或注入協定——Agent Skills 標準已被四十餘框架採用且 speclink 輸出已相容
**Open**: 是否做 pi 套件（@speclink/pi：預渲染技能＋extension）；Copilot SDK skillDirectories 細節待查證；speclink_skill 第二顆工具做不做；文件過期修正是否併入同一 change；三種 runtime 的優先序

## Conclusion

<!-- Written by `speclink discuss conclude`:
**Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->
