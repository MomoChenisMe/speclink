---
topic: 做一個 herdr 裡面可以用的 speclink 面板（含 macOS 系統匣面板的內容、技能清單，並找其他可整合的 speclink 功能）
slug: herdr-speclink-panel
status: promoted
created: 2026-11-03
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: desktop-deep-link, plan-unpromoted-cuts, desktop-roadmap, discussion-cuts-from-engine
---

# Discussion: 做一個 herdr 裡面可以用的 speclink 面板（含 macOS 系統匣面板的內容、技能清單，並找其他可整合的 speclink 功能）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者在用 Herdr desktop（dev.herdr.app 0.1.12，終端機與 coding agent 的桌面 app），它的介面整個由 `~/.config/herdr-app/` 底下的檔案構成，可以寫 mod（JavaScript 描述 GPUI 原生 UI，`herdr.d.ts`／`gpui-kit.d.ts` 為型別正典，`AGENTS.md` 為說明）。使用者想把 speclink 帶進 Herdr：先請研究「tray 功能能不能加到 herdr」，研究後修正方向——不是沿用 tray 程式碼，而是做一個 Herdr 裡用的 speclink 面板，內容包含 macOS 系統匣面板的內容；Claude Code mod 的技能清單也做成 Herdr mod；再發揮創意找其他可整合的 speclink 功能。需求已有具體交付物，不需要 grill；第 4 項是發想題，以假設清單提出候選。相關規格：tray-status-menu（面板內容的來源定義）、client-protocol（「變更清單的審查狀態欄位」明寫 CLI `speclink list --json` SHALL NOT 含 reviewStatus 等欄位）、review-station 與 verify-station（「CLI 清單輸出的相容性釘住」）、discussion-docs；目前沒有在途變更。先例：`integrations/claude-code/speclink-skills`（Claude Code mod，只靠 CLI `--json` 組看板、沒有自己的 capability 規格，以 feat(claude-code) 直接提交）。
Prior discussions: system-tray-status, tray-rich-panel, tray-station-badges, agent-tool-sdk-layering

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — interview (2026-11-03)

**Focus**: 這個討論要做的是什麼——把 tray 搬進 Herdr，還是做 Herdr 原生的 speclink 面板
**Position**: 做 Herdr 原生的 speclink 面板 mod，不沿用 tray 程式碼；tray 只當「內容清單」的參考。
- 使用者修正：不是要沿用 tray，而是 Herdr 裡可以用的 speclink 面板；內容要包含 macOS 系統匣面板的內容
- Herdr mod 的能力（AGENTS.md）：exec 外部程式、watchFolder 監看資料夾、openOverlay（dialog／popup／window）、pane.split／tab.new 開 mod 窗格或跑 agent、notify 通知、shared 跨 mod 狀態、hc 操作終端機；不能用 HTML／web view／Node API
- 資料面事實：CLI `list --json`／`plan --json`／`discuss list --json` 給得出變更名、任務 n/m、階段、波次、blockedBy、worktree、討論 slug/topic/status
- CLI 缺口事實：審查章／驗證章狀態只在 apps/desktop/core/src/query.rs 疊加；討論的 concluded 與 head（promoted_to、hold）在 crates/engine/speclink-core/src/lifecycle/discuss.rs 標 `#[serde(skip)]`；桌面 app 沒有 deep link
- 需求清單：(1) Herdr 原生面板 (2) 面板含 tray 面板內容 (3) 技能清單做成 Herdr mod (4) 其他可整合的 speclink 功能
**Ruled out**: 沿用 tray 程式碼（TrayPanel.tsx 等 React／Tauri 面板）——Herdr mod 不能用 HTML／web view，且使用者明說不是要沿用；在 Herdr 做系統選單列圖示——herdr.d.ts 沒有 tray／menubar API
**Open**: (A) 面板形態與放置 (B) 資料來源與 CLI 缺口（品質站章、已轉出分區）(C) tray 內容哪些照搬、哪些換成 Herdr 版 (D) 技能清單怎麼執行技能 (E) 其他整合候選 (F) 程式放哪、要不要規格

### Round 2 — assumptions (2026-11-03)

**Focus**: 第一份假設清單（A–F）哪些成立
**Position**: 假設大致成立，兩處修正——專案依目前的 Herdr space 決定、詳情交給 Speclink app 開。
- C 內容：照搬 tray 面板的畫面與資訊；面板不做專案分頁，專案由目前 space 的資料夾決定
- 詳情：點變更或討論時直接呼叫 Speclink 開它的詳情頁，取代原假設的 Herdr 文件窗格（6c）；手冊也已有桌面版頁面，同樣交給 Speclink
- 事實：Speclink.app 內有 `speclink`（CLI）與 `speclink-desktop` 兩支執行檔；Info.plist 沒有 CFBundleURLTypes，apps/desktop/src-tauri/Cargo.toml 沒有 deep-link／single-instance plugin，lib.rs 也不讀啟動參數——目前外部無法叫桌面 app 開某一頁
- 事實：store（apps/desktop/src/store.ts）已有 openProjectAt(path)、openDetail(name)、openDiscussion(slug)，缺的只是對外入口
- 使用者沒有指出錯誤、暫列成立：A 右側常駐面板＋快捷鍵開關；B 只走 CLI `--json`；B1 第一刀不顯示章、已轉出分區用 status 粗分，CLI 選擇性旗標另開一刀；D 技能列另做底部 mod，打進目前分頁的 agent 終端機、不按 Enter；F 放 `integrations/herdr/`，mod 不寫 capability 規格
- 其他整合（E）：使用者要求先用 artifact 呈現候選再決定
**Ruled out**: Herdr 內的文件窗格當詳情——使用者要詳情直接交給 Speclink app；面板保留專案分頁——專案改由目前 space 決定，不另管專案清單
**Open**: (G) Herdr 怎麼叫 Speclink 開詳情（URL scheme、CLI 動詞或啟動參數）(E) 其他整合候選的取捨，看 artifact 後決定

### Round 3 — interview (2026-11-03)

**Focus**: Herdr 怎麼叫 Speclink 開變更或討論的詳情頁
**Position**: 桌面 app 註冊網址格式 `speclink://`，Herdr mod 用 `exec("open", [網址])` 叫它開詳情。
- 網址帶專案路徑與變更名（或討論 slug），例如 `speclink://change/<名稱>?project=<路徑>`；app 收到後先開該專案（還沒開就加成分頁），再開詳情頁——沿用 store 的 openProjectAt、openDetail、openDiscussion
- 好處：Herdr、終端機、瀏覽器、Claude Code 外掛都能用同一個入口；macOS 上 app 沒開時會自動啟動
- 實作需求：Tauri deep-link 外掛；Windows、Linux 另需 single-instance，把網址轉給已開著的 app
- 網址的確切格式屬 propose／design 細節
- 事實（給 b 一鍵開工）：Herdr 內建 spaces mod 以 `git rev-parse --git-common-dir` 判斷 worktree（mods/spaces/main.js），speclink 的 `<repo>.worktrees/<名稱>` worktree space 也會縮排在主 checkout 的 space 底下
- 使用者看了 artifact（整合候選示意）後，要求改用亮色呈現，並問側欄縮排的那一列是什麼——那是 b 開工後出現的 worktree space
**Ruled out**: CLI 動詞 `speclink open <名稱>`——CLI 和桌面 app 是兩支程式，CLI 最後仍要經網址或另一條通道通知 app，單獨存在只是多包一層；啟動參數 `speclink-desktop --open-change`——同樣要 single-instance，且呼叫端要知道 app 內部執行檔路徑
**Open**: (E) 整合候選的取捨（a、b 第一批，d、e 第二批，i、j 可選，f、g、h 不建議——待使用者看完亮色版 artifact 後確認）

### Round 4 — interview (2026-11-03)

**Focus**: 整合候選（E）的去留
**Position**: 做 a、b、d、e、i、j、f、h，不做 g；技能列（D）重新打開，要找更好看的呈現方式。
- 使用者看完 artifact 後決定 i 波次地圖、j 一句話開討論、f 狀態列計數、h space 側欄徽章都要做；建議分批（a、b 第一批，d、e 第二批）未被反對
- 事實：Herdr 內建 spaces mod（438 行）只讀共用值 `space.colors`，沒有擴充點——h 必須複製 spaces mod，之後不會跟著 Herdr 更新
- 事實：main layout 的狀態列是一格 block，掛 31 行的 `workspace-status`——f 可在複製的 layout 裡把這格換成一列「內建 workspace-status＋自己的 speclink 計數 mod」，內建 mod 不必複製
- 事實：main layout 右側 `panel` 是 stack（files、changes，預設隱藏，368px）；每個窗格已有 30px 標頭，掛內建 `pane` mod
- 推論：f、h 都要複製 layout（h 另要複製 spaces mod），面板也可能要放進複製 layout 的右側 stack——複製版不會跟著 Herdr 更新
**Ruled out**: g 終端機右鍵「貼上變更名稱」——自訂右鍵選單會取代整個選單，內建項目要全部重做，面板的複製鈕已涵蓋這個需要
**Open**: (D) 技能列的呈現方式 (H) 複製 layout 的策略：一份 speclink 專用 layout 收齊面板、f、h，還是分開 (I) 切刀與先後

### Round 5 — interview (2026-11-03)

**Focus**: 技能入口要照 Claude Code 外掛的操作，還是照 Herdr 的方式設計
**Position**: 以 Herdr 的方式設計技能入口，不照搬 Claude Code 外掛；我提議「按技能＝開一件 agent 工作」，待使用者確認。
- 使用者先確認原技能列的操作（點技能、再點變更帶入名稱），接著明說不必照 Claude Code 外掛，應以 Herdr 思考
- 事實：Claude Code 外掛是「點技能填入 `/speclink-xxx `、點面板名稱換成參數」（integrations/claude-code/speclink-skills/hooks/skills.ts 的 withCommand、withArgument）
- 事實：Herdr mod 只能用 `hc send` 往終端機打字，讀不到也改不了輸入框現有內容；面板變更列的點擊已定為叫 Speclink 開詳情
- 事實：Herdr 內建指令面板（app.palette）沒有開放 mod 加項目；分頁列「+」選單（new-menu mod）只列 programs.json 有 `run` 的 agent，開分頁不帶第一則訊息
- 事實：Herdr 啟動工作的原生方式是 `tab.new {name, space, argv, task}`／`space.new {…, task}`——開分頁跑 agent 並送出第一則訊息，分頁列、agents 清單、blocked 通知自動追蹤
- 提議：按變更列上的技能 → 在該變更的 space（有 worktree 就是 worktree 的 space）開新分頁「<技能> · <變更>」，task 為完整指令，貼標籤 `speclink.change`、`speclink.skill`；面板列顯示這件工作的 working／blocked／done；全域技能（discuss、improve、config 等）由面板的「新工作」入口開，j 併入此入口
**Ruled out**: 照搬 Claude Code 外掛的「點技能、再點變更」兩步填字——Herdr mod 讀不到輸入框、只能接在游標後打字，變更列點擊已用於開詳情，且使用者要求以 Herdr 思考
**Open**: (D) 技能按下去的預設：開新分頁，還是交給該變更現有的 agent (H) 複製 layout 的策略 (I) 切刀與先後

### Round 6 — interview (2026-11-03)

**Focus**: 技能按下去的預設：開新分頁，還是交給現有的 agent
**Position**: 預設延續現有的 agent（在原本的視窗接著跑），新分頁只在找不到 agent 時才開。
- 使用者：平常是延續某個視窗跑，所以不要預設開新分頁
- 事實：`hc send <terminal> <text>` 會貼上文字並按 Enter，等使用者停手才送；`--if idle|working` 只在該狀態才送；agent 為 blocked 時預設不送（要 `--force`）；`hc wait` 可等 agent 停下
- 提議的送達順序（待確認）：1. 正在做這個變更的 agent（標籤 `speclink.change`、worktree 路徑，或 Claude Code 外掛設的終端機標題「<步驟> · <變更>」——Herdr 的 panes()／agents() 有 title）2. 目前分頁的 agent 3. 都沒有才開新分頁
- 提議：按鈕送出完整指令並按 Enter（`hc send --if idle`），送出後 `pane.focus` 到那個 agent；agent 忙碌或在等回答時不送，按鈕說明原因
**Ruled out**: 預設一律開新分頁——使用者平常在同一個視窗延續工作，每按一次就多一個分頁不符合習慣
**Open**: (D) 送達順序與忙碌時的處理 (H) 複製 layout 的策略 (I) 切刀與先後

### Round 7 — interview (2026-11-03)

**Focus**: 技能送達的順序與 agent 忙碌時的處理
**Position**: 照提議定案——技能送給「這個變更的 agent → 目前分頁的 agent → 新分頁」，用 `hc send --if idle` 送出完整指令並按 Enter，忙碌或等回答時不送。
- 送達順序：1. 正在做這個變更的 agent（`speclink.change` 標籤、worktree 路徑、Claude Code 外掛設的終端機標題，依序比對）2. 目前分頁的 agent 3. 都沒有才 `tab.new {argv, task}` 開新分頁
- 送出後 `pane.focus` 到那個 agent，畫面切過去
- agent 為 working 或 blocked 時不送，技能鈕旁寫出原因（「正在做事」「在等你回答」）
- 技能入口：變更列與討論列上的情境技能鈕（只列該階段用得到的，建議的下一步加粗外框）；和單一變更無關的技能放在面板頂端「＋ 新工作」，j 一句話開討論併入此入口；底部技能列不做
**Ruled out**: 忙碌時照樣送出——會打亂 agent 正在處理的輸入，且 blocked 時可能誤答它的對話框
**Open**: (H) 複製 layout 的策略 (I) 切刀與先後

### Round 8 — interview (2026-11-03)

**Focus**: 複製 layout 的策略
**Position**: 複製 main 做一份 `speclink` 專用 layout，三處改動集中在這一份；Herdr 原本的 main 不動。
- 右側：speclink 面板自成一欄，放在 Files／Changes 旁，以 `layout.toggle:speclink` 開關
- 底部：狀態列那格換成一列「內建 workspace-status＋自己的 speclink 計數 mod」（f），內建狀態列不複製、照樣跟著 Herdr 更新
- 左側：spaces 換成複製並加上待收尾徽章的 spaces mod（h）
- 三處改動寫成說明文件與 mod 放在一起，Herdr 更新 main 或 spaces 後據此重新複製、重套
- 其他整合不必改 layout：技能鈕在面板內；d、i 用 `pane.split {mod}` 開窗格；e 用 notify；b、j 用 space.new／tab.new
- 事實：分頁列 mod（mods/tabs/main.js）寫死只認 files、changes 兩個面板，並依 space 記住開關、切 space 時自動開關
**Ruled out**: 面板擠進 Files／Changes 的 stack——分頁列寫死兩個開關且依 space 自動開關，會和 speclink 面板互搶；不複製 layout——f、h 做不到
**Open**: (I) 切刀與先後

### Round 9 — interview (2026-11-03)

**Focus**: 切刀與先後
**Position**: 照提議切成 speclink 變更兩刀加 Herdr 端三批；兩刀互不依賴，可並行。
- speclink 變更：`desktop-deep-link`（桌面 app 註冊 `speclink://`）、`cli-board-status`（CLI 選擇性旗標輸出品質站狀態與討論 concluded／hold，章過期計算搬進引擎）
- Herdr 端維持 F：放 `integrations/herdr/`、不寫 capability 規格、以 `feat(herdr)` 提交
- 第 1 批：speclink layout（右側欄、f、h）＋面板＋c；第 2 批：a → 技能入口（含 j）→ b；第 3 批：d、e、i；`cli-board-status` 完成後面板補章與已轉出分區
- 結論以 `--hold` 寫入，兩刀分開提案，第二刀標最後一刀
**Ruled out**: Herdr 端也走 speclink 變更——使用者維持 F，與 Claude Code 外掛同樣處理

### Round 10 — interview (2026-11-03)

**Focus**: 左側 spaces 的 worktree 列和 agents 清單看似重複，留著的意義
**Position**: worktree space 保持 Herdr 原樣顯示；三處各管一件事——spaces 管地方與 git 狀態、agents 管 agent 狀態、speclink 面板管變更進度。
- 事實：Herdr 內建 spaces mod（mods/spaces/main.js 開頭註解）寫明「Agent state lives in the agents list, not here.」——spaces 列只顯示頭像、名稱，以及特別的 git 資訊（與名稱不同的分支、非 main／master 的分支、↑領先／↓落後 upstream）
- 事實：agents 清單可依 space 分組或依急迫度分區（mods/agents/main.js）
- worktree space 的意義：可切過去的地方（agent 以外的 shell、測試、dev server 分頁都在 worktree 裡）；顯示 worktree 的分支與 ahead／behind，合併前有用；縮排在所屬專案底下，和 Herdr 內建 worktree 功能「一個 worktree 一個 space」一致
- 先前示意圖在 worktree space 列畫了 blocked 狀態點，與 Herdr 實際行為不符，已修正 artifact
- 不影響結論：b 一鍵開工維持 `space.new`
**Ruled out**: 開工改成在 speclink space 內開分頁、不另開 space——看不到 worktree 的 git 狀態，且偏離 Herdr「worktree 就是 space」的慣例；在複製版 spaces 隱藏 speclink 開的 worktree space——要在該 worktree 開 shell 或跑測試時沒有地方可點

### Round 11 — interview (2026-11-03)

**Focus**: 波次地圖（i）的入口與呈現
**Position**: i 改成「路線圖」：像 ⌘K 一樣彈出的對話框，每一波一欄、每個變更一站、用線連出誰擋誰。
- 入口：快捷鍵（keys.json 綁 `mod.open`，再按一次或 Esc 關閉）、點面板列首的波次數字（彈出後選中那一站）、面板頂端「路線圖」鈕
- 呈現：站的樣子表示階段（實心綠＝已就緒、外圈弧線＝進行中且弧長是任務進度、空心＝提案中、虛線圈＝等前一波），站旁紅點＝agent 在等回答；滑鼠停留時該站的路線亮起、其他變淡；plan 的 `next` 標成「下一個」
- 操作：方向鍵移動、Enter 叫 Speclink 開詳情、Space 跳到 agent、⌘Enter 開工；b 的整波開工鈕放在每一欄標頭
- 事實：`openOverlay({kind: "dialog"})` 置中、背景變暗、Esc 或點旁邊關閉；gpui-kit 有 `window.paint_path`＋`PathBuilder`（cubic_bezier_to、stroke）可畫路線，站排在固定格線上即可算出座標
- 待查：快捷鍵選哪個不和 Herdr 衝突
**Ruled out**: 開成自己的分頁——每次多一個分頁；浮動視窗——一直開著會蓋住一塊畫面；卡片欄＋只用滑鼠停留標出依賴（上一版）——使用者偏好路線圖
**Open**: 使用者要求重新檢視每項功能對簡化、加快開發流程的意義

### Round 12 — interview (2026-11-03)

**Focus**: 每項功能對加快開發流程的意義，以及哪些該放進 speclink 本體
**Position**: 依使用者實際用法重排優先順序；我提議分工——Herdr 管「執行」（叫技能、帶名稱、送對 agent），speclink 本體管「規劃與閱讀」（路線圖、未轉出的刀、規格），c 連接兩邊；待確認。
- 使用者：最常用的是「技能＋選變更名稱」；其次是決定該做哪些變更、第幾波、依賴誰、下一波的刀是否已轉出，並問這些是否該改進到 speclink 本體；c 用得少，但一次做很多變更時要在終端機和 Speclink 之間切換看規格
- 數據（9/1 起封存的 104 個變更）：69 個當天建立當天封存；30 個宣告依賴；35 個工作日中 18 天同時有 3 個以上在實作（按天計，會高估）；品質關卡每站約 2.5～2.8 輪
- 事實：`speclink plan` 已有波次、blockedBy、next；桌面看板依 plan 排序並畫淡被擋的卡；「N 刀尚未轉出」只在桌面討論詳情頁，由前端 packages/ui/src/discussionModel.ts 解析結論文字得出，引擎與 CLI 不知道
- 價值評估：高＝技能鈕＋送達順序（含 a）；中＝面板、b、路線圖、c；低＝d、e、f、h、j、cut 2（Herdr 已通知 blocked／done、技能結尾已提示下一步、quality 每輪已在終端機列出發現）
**Open**: (J) 是否照「Herdr 管執行、本體管規劃」重排範圍與切刀

### Round 13 — interview (2026-11-03)

**Focus**: 依「Herdr 管執行、本體管規劃」重排範圍與切刀
**Position**: 照提議重排——Herdr 只做執行面，規劃面（尚未轉出的刀、路線圖）移進 speclink 本體，c 連接兩邊。
- Herdr 第 1 批：面板（tray 內容、跟著 space）＋技能鈕＋送達順序（含 a）＋「＋ 新工作」（不帶變更名稱的技能）
- Herdr 第 2 批：b 開工（單一與整波）＋點列開詳情（c）＋面板的「路線圖」鈕與波次數字改為叫 Speclink 開路線圖
- speclink 變更改為三刀：`desktop-deep-link`（不變）、`plan-unpromoted-cuts`（引擎解析結論的刀，plan 一起列出尚未轉出的刀；前端 discussionModel.ts 的解析改用引擎資料）、`desktop-roadmap`（桌面看板層級路線圖，含尚未轉出的刀與依賴線）
- `cli-board-status` 延後：面板先不顯示章，「已轉出」用 status 粗分
- 不做：d、e、f、g、h、Herdr 內的路線圖；f、h 拿掉後不再需要複製 layout（推翻第 8 輪），面板改用 Herdr 停靠或快捷鍵彈出，實作時查證
- 尚未轉出的刀之間不推算依賴，路線圖只標「屬於哪份討論的第幾刀」——結論的刀序不一定代表依賴
**Ruled out**: Herdr 內做路線圖——規劃資訊在桌面、CLI、Herdr 應一致，資料與畫面放本體、Herdr 用 c 跳過去即可；複製 `speclink` layout——只剩面板一個理由，不值得承擔跟著 Herdr 更新重套的維護

### Round 14 — interview (2026-11-03)

**Focus**: cut 2 一刀做完的範圍過大，要不要拆
**Position**: 拆成兩刀——cut 2 `plan-unpromoted-cuts` 只做引擎解析與 plan 輸出，桌面討論詳情頁改用引擎資料另切 cut 4 `discussion-cuts-from-engine`。
- 事實：`plan --json` 的形狀由 change-plan「plan 動詞輸出波次與阻擋清單」釘為四鍵，remote 臂（GET /plan）要求與本機同形——加鍵即動到引擎、CLI、server
- 事實：討論詳情頁（packages/ui 的 DiscussionDetailPage、ConclusionDoc）只有桌面 app 使用，但要顯示本機、remote、已封存三種來源的刀卡；改用引擎資料要另動桌面查詢、server 討論 API 與 remote 轉接
- 事實：前端判定刀是否轉出用 `stageOf(name) != null`，stageOf 取自討論的 promoted_to 衍生列（DiscussionDetailPage 的 rowStage），語意等同比對 promoted_to
- cut 4 落地前，前端解析暫時保留；cut 4 依賴 cut 2（引擎解析器）
**Ruled out**: 一刀做完——五層同時改動，任務超過 15 項、品質關卡審查面過大；永久保留前端解析——引擎與前端各解析一份，判定會漂移

## Conclusion

**Decision**: 分工為「Herdr 管執行、speclink 本體管規劃與閱讀」——在 Herdr 做一套精簡的 speclink mod（面板、技能入口、開工），規劃面（尚未轉出的刀、路線圖）與詳情入口放進 speclink 本體，切成四個 speclink 變更。
- **cut 1 `desktop-deep-link`**: 桌面 app 註冊 `speclink://` 網址，外部可叫它開變更或討論的詳情頁
  - 網址帶專案路徑與變更名（或討論 slug）；收到後先開該專案（還沒開就加成分頁），再開詳情頁，沿用 store 的 openProjectAt、openDetail、openDiscussion
  - macOS 用 Tauri deep-link 外掛；Windows、Linux 另加 single-instance，把網址轉給已開著的 app
  - 規格落在 `desktop-app`；網址確切格式在 propose／design 定
  - 與 cut 2 互不依賴
- **cut 2 `plan-unpromoted-cuts`**: 引擎解析討論結論的分刀列，`plan` 一起列出尚未轉出的刀（本機與 remote 的 GET /plan）
  - 每把刀帶所屬討論 slug、刀編號、變更名、範圍句與是否已轉出（比對 `promoted_to`）
  - 解析器放在引擎，供 cut 4 與 cut 3 共用；本刀不改桌面討論詳情頁
  - 不推算刀與刀之間的依賴：結論的刀序不一定代表依賴，只標「屬於哪份討論的第幾刀」
  - `plan --json` 既有鍵不變，尚未轉出的刀為新增的鍵；人眼輸出另加一段
- **cut 3 `desktop-roadmap`**: 桌面看板層級的路線圖，一張圖看全部進行中變更與尚未轉出的刀
  - 每一波一欄、每個變更一站、依賴畫成線；站的樣子表示階段（已就緒、進行中含任務進度、提案中、等前一波），尚未轉出的刀以桌面既有的「尚未轉出」橄欖色虛線呈現，可複製 `/speclink-propose --from-discussion <slug>`
  - 標出 plan 的 `next`；滑鼠停留時該站的路線亮起、其他變淡
  - `speclink://` 加一條開路線圖的網址，供 Herdr 跳過來
  - 依賴 cut 1（網址）與 cut 2（尚未轉出的刀資料）
- **cut 4 `discussion-cuts-from-engine`**: 桌面討論詳情頁的刀卡與「N 刀尚未轉出」改用引擎解析的資料，移除前端的分刀解析
  - 涵蓋本機、remote、已封存三種來源：桌面查詢與 server 的討論 API 一併帶出引擎解析的刀
  - 移除 packages/ui/src/discussionModel.ts 的分刀解析（parseCuts、cutPromotion 等），刀卡畫面不變
  - 依賴 cut 2（引擎解析器）
- Herdr 端（不走 speclink 變更）：程式放 `integrations/herdr/`，不寫 capability 規格，以 `feat(herdr)` 提交，與 Claude Code 外掛同樣處理
- 資料面：mod 只用 CLI `--json`（list、plan、discuss list），`watchFolder` 只當重抓訊號；階段、波次、依賴、尚未轉出的刀一律由引擎算
- Herdr 第 1 批：
  - 面板：照搬 macOS 系統匣面板的內容（三階段分區常駐、變更列的進度條／n/m／波次／被擋變淡、討論卡與已轉出卡、溢出摺疊、卡片收合、複製鈕）；專案跟著目前的 Herdr space；章先不顯示、「已轉出」先用 status 粗分；放置方式用 Herdr 停靠或快捷鍵彈出，不複製 layout
  - 技能入口：變更列與討論列上的情境技能鈕（只列該階段用得到的，建議的下一步加粗外框）；不帶變更名稱的技能（discuss、propose、improve、config 等）放在面板頂端「＋ 新工作」，選 discuss 時可直接輸入主題
  - 技能送達：依序送給「正在做這個變更的 agent（`speclink.change` 標籤、worktree 路徑、Claude Code 外掛設的終端機標題）→ 目前分頁的 agent → 都沒有才 `tab.new {argv, task}` 開新分頁」；以 `hc send --if idle` 送出完整指令並按 Enter，送出後 `pane.focus` 過去；agent 在 working 或 blocked 時不送，技能鈕旁寫出原因
  - a 變更和 agent 對應：面板列上顯示做這個變更的 agent 與狀態，點了 `agent.focus`
- Herdr 第 2 批：
  - b 一鍵開工與整波開工：`space.new` 帶 task 跑 `/speclink-apply-with-worktree <名稱>` 並貼 `speclink.change` 標籤；worktree space 保持 Herdr 原樣顯示（spaces 管地方與 git、agents 管 agent 狀態、面板管變更進度）；未開 worktree 政策時改在新分頁跑 `/speclink-apply`、不提供整波
  - c 點變更或討論列：`exec("open", ["speclink://…"])` 叫 Speclink 開詳情（等 cut 1）
  - 面板的「路線圖」鈕與列首波次數字：叫 Speclink 開路線圖（等 cut 3）
- 這套 mod 的檢查目標：一個變更從提案到提交，全程不用手打變更名稱、不用找視窗
**Rationale**: 使用者最常做的是「叫技能＋選變更名稱」，其次是規劃先後與確認刀是否轉出；前者發生在終端機與 agent 旁，適合放 Herdr，後者是退一步看全局，桌面、CLI、Herdr 都該看到同一份，所以資料進引擎、畫面進桌面，Herdr 只用 c 跳過去——功能集中在省最多步驟的地方，其餘和 Herdr 或技能本身已提供的東西重複者一律不做。
**Rejected alternatives**:
- 沿用 tray 程式碼（TrayPanel.tsx 等）— Herdr mod 不能用 HTML／web view
- 在 Herdr 做系統選單列圖示 — herdr.d.ts 沒有 tray／menubar API
- Herdr 內的文件窗格當詳情 — 使用者要詳情交給 Speclink app
- 面板保留專案分頁 — 專案由目前 space 決定
- mod 自己讀 `.meta.yaml` 判斷章 — 算不出章是否過期
- CLI 動詞 `speclink open` 或 `speclink-desktop` 啟動參數當詳情入口 — 仍要經網址或另一條通道通知 app，單獨存在只是多包一層
- 照搬 Claude Code 外掛「點技能、再點變更」填輸入框 — Herdr mod 讀不到輸入框，變更列點擊已用於開詳情
- 底部技能列與填輸入框的做法（S1–S4）— 技能改為交給 agent 的完整指令
- 技能預設一律開新分頁 — 使用者平常在同一個視窗延續工作
- agent 忙碌時照樣送出 — 會打亂輸入，blocked 時可能誤答對話框
- 開工改在 speclink space 內開分頁、或隱藏 worktree space — 看不到 worktree 的 git 狀態，且要開 shell 時沒有地方點
- Herdr 內的路線圖（分頁或彈出對話框）— 規劃資訊應在各處一致，畫面放桌面、Herdr 用 c 跳過去
- d 品質工單窗格 — quality 每輪停頓時終端機已列出發現並等決定
- e 下一步提示 — Herdr 已通知 blocked／done，技能結尾已提示下一步，技能鈕已加粗建議
- f 狀態列計數 — 與面板重複，且需複製 layout
- g 終端機右鍵「貼上變更名稱」— 自訂右鍵選單會取代整個選單
- h space 側欄徽章 — 只在多專案同時進行時有用，且要複製並維護 spaces mod
- 複製 `speclink` 專用 layout — f、h 不做後只剩面板一個理由，不值得承擔跟著 Herdr 更新重套的維護
- Herdr 端也走 speclink 變更 — 與 Claude Code 外掛相同處理
- cut 2 一刀做完（含桌面詳情頁改用引擎資料）— 一次動到引擎、CLI、server、桌面、共用 UI 五層，任務與審查面過大
- 永久保留前端的分刀解析 — 引擎與前端各解析一份，判定會漂移
**Deferred**:
- `cli-board-status`（CLI 輸出章與討論 concluded／hold，面板顯示章）— 價值低、要改四份規格，等面板用過再議
- 面板的放置方式（Herdr `place` 停靠或快捷鍵彈出）— 實作第 1 批時查證停靠在內建 layout 是否可行
- Codex 叫用技能的寫法 — 和 Claude Code 不同，第 1 批實作時查證
- 各階段對應哪些情境技能 — 第 1 批的對照表
- 路線圖網址與快捷鍵的確切形式 — cut 1、cut 3 提案時定
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion herdr-speclink-panel
