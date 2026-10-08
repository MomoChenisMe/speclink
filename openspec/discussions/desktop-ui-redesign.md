---
topic: 重新設計整個 Desktop 介面：白底中性灰、單一點綴色、扁平細框的桌面工具風格，含共用元件整理、原生選單、標題列與設定頁
slug: desktop-ui-redesign
status: promoted
created: 2026-10-08
created_by: MomoChen <momochenisme@gmail.com>
hold: true
promoted_to: fix-drawer-open-dismisses-overlays, desktop-design-foundation
---

# Discussion: 重新設計整個 Desktop 介面：白底中性灰、單一點綴色、扁平細框的桌面工具風格，含共用元件整理、原生選單、標題列與設定頁

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要把 Speclink 桌面 app 的整個介面重新設計成一種「白底中性灰、單一點綴色、扁平細框分層、圓角分層、浮層才有陰影、短而安靜的動態、隱藏原生標題列」的桌面工具風格，並在換皮之外一併整理散落各處的共用元件、把 macOS 原生選單中文化並補功能、消掉標題列的大片空白（Windows／Linux 一併考慮）、設定頁改成同風格。目標可對照一份具體風格描述驗收，不需要 grill 階段。偵察命中的正典規格：desktop-app（79 條，約 16 條約束視覺）、ui-copy-vocabulary、tray-status-menu、desktop-config、workspace-chooser、server-web-console（外殼尺寸須與 Desktop 相等）；在途變更 add-copilot-tool: desktop-config、add-copilot-tool: workspace-chooser（設定頁需求與 checkout 綁定）。相關已封存變更：card-name-single-line-fade、change-drawer-header-redesign、desktop-導覽與專案首頁重構、tray-panel-card-design。
Prior discussions: card-drawer-header-colors, desktop-導覽與專案首頁重構, web-service-navigation-redesign, tray-panel-card-design, admin-console-redesign

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-08)

**Focus**: 重新設計的範圍邊界——只換皮、連外殼，還是連資訊架構一起改
**Position**: 範圍定為五項：換皮、共用元件整理、macOS 原生選單中文化與功能、三平台標題列、設定頁風格；資訊架構（看板四欄、四種抽屜、六個導覽頁、專案分頁列）不動。
- 底層不換：現有 React 19＋Tailwind v4＋shadcn 式 Radix 原語＋lucide＋sonner 與目標風格同款；重新設計＝改 token 值、元件外觀、視窗殼（apps/desktop/package.json、packages/ui/src/theme.css、packages/ui/src/components/ui/）
- 目標風格：白底主區、淺灰側欄、細框分層不靠陰影；單一點綴色只用於品牌標記、選取項、主要操作、進度，不做大面積底色；選取＝點綴色淡底深字、hover＝半透明黑；圓角四階（按鈕 6px、清單列 8px、面板 12px、卡片 16px，標籤與搜尋框全圓）；陰影只給 tooltip／選單／對話框；外殼字 11–13px、頂列 48px、閱讀區 15px／行高 28px；側欄約 275px 可拖寬、欄間細線 hover 變點綴色；動態 150–200ms、位移 4px、ease-out、動作鈕 hover 才出現
- 現況對照（apps/desktop/src/App.tsx、packages/ui/src/theme.css）：原生標題列＋48px header 兩層、側欄 200px 白底、選中整塊填滿主色、Card 帶 shadow-sm、hover 加 shadow-md、單一圓角 10px、字級大量 9–11px arbitrary 值
- 未被更正、視為成立：深色模式保留並補一套同風格深色值；token 機制不動（theme.css 的 CSS 變數＋tone.ts／stage.ts TS 常數表，theme.test.ts 快照同步更新）；既有裁定保留（省略號截斷不改淡出、Noto Sans TC 打包、卡片三列骨架、三層語意色、抽屜四層標頭、側欄六項順序）；系統匣面板維持毛玻璃、只對齊 hover／選取語意與圓角
- 正典對照：server-web-console 要求 header 高度、側欄寬、主內距與 Desktop 相等（apps/server-web/src/layouts/ConsoleLayout.tsx 的 h-12／w-[200px]／p-5），改殼要同步；desktop-config 的設定頁需求正被 add-copilot-tool 改寫，設定頁改版排在其後
- 重開先前裁定：desktop-導覽與專案首頁重構 曾以「使用者要保持自家設計」否決照抄別家視覺；本次使用者明確改換視覺方向，舊理由不再成立
**Ruled out**: 連資訊架構一起改（看板欄位、抽屜、導覽頁重排）——desktop-app 其餘 63 條行為需求與 70 個 vitest 檔綁著現有結構，工程量差一個量級且不是使用者要的
**Open**: 點綴色留 teal 還是換紫（紫為品質站蓋章專屬）；共用元件整理的範圍與守門方式；macOS 原生選單中文化與要補哪些功能；三平台標題列作法；設定頁採「設定模式取代側欄」還是保留六項側欄只換皮；切刀

### Round 2 — assumptions (2026-10-08)

**Focus**: 點綴色的裁定，與第二輪五項（共用元件、原生選單、標題列、設定頁、Logo）是否成立
**Position**: 點綴色留 teal；五項全數成立。
- 點綴色：teal 留任品牌／互動／主要操作／進度色，紫色維持品質站蓋章專屬；風格一樣、色相不同，Speclink 身分靠色相分辨
- 共用元件：packages/ui 為唯一的家，兩層——components/ui 原語＋領域元件；新增原語 dropdown-menu、switch、search-input、status-pill、kbd、confirm-dialog（包 AlertDialog 四件組）、empty-state、copy-button、section-header、page-header、settings-card／settings-row；apps/* 不得定義與 @speclink/ui 匯出同名的元件（新守門測試掃 apps/*/src）；openspec/config.yaml 產出規則加「優先用 @speclink/ui 既有元件、呼叫端不覆蓋變體顏色」；現況：CopyButton 定義 4 次（TrayPanel.tsx:93、connectionLogin.tsx:12、ArchivedList.tsx:22、server-web CopyButton.tsx）、App.tsx 13 個 AlertDialogContent、EmptyState／NavItem 在 App.tsx 本地、SectionHeader 在 TrayPanel.tsx 與 DocumentTree.tsx 各一份、server-web 自有 Field／DataList／ListToolbar／DetailSheet
- 原生選單（macOS 限定）：src-tauri 現無選單程式碼、顯示 Tauri 預設英文選單；改為 Rust MenuBuilder 自建、跟 UI 語言偏好（packages/ui/src/locale.ts）、語言切換時 app.set_menu 重建（tauri 2.11.5 app.rs:961；menu/predefined.rs 各預設項接受自訂文字）；結構：Speclink（關於、檢查更新…、設定… ⌘,、服務、隱藏、結束）、檔案（開啟專案… ⌘O、關閉分頁 ⌘W、安裝 CLI…）、編輯（復原／重做／剪下／複製／貼上／全選，預設項必留否則輸入框快捷鍵失效）、檢視（變更／已封存／規格／手冊 ⌘1–4、專案設定、搜尋看板 ⌘F、重新整理 ⌘R、下一個分頁 ⌃Tab）、視窗（縮到最小／縮放／全部移到最前）、說明（手冊、更新日誌、GitHub、回報問題）；Windows／Linux 不做選單列
- 標題列：macOS 用 titleBarStyle Overlay＋hiddenTitle＋trafficLightPosition {x:16,y:17}、decorations 維持 true，側欄首列 48px 為拖曳區、左讓 88px 給紅綠燈；Windows 用 decorations false＋自繪 48px 拖曳列與三顆視窗鈕、shadow true 保留 Win11 圓角（代價：失去貼齊版面浮窗）；Linux 維持原生標題列；側欄改為全高淺灰欄、專案分頁列移到主區頂列；server-web 的 ConsoleLayout.tsx 同步外殼尺寸（server-web-console 要求相等）
- 設定頁：採「設定模式」——進入設定後側欄換成返回／搜尋設定／分組清單（應用程式：一般、語言與外觀、更新、CLI、伺服器；專案：專案說明、產出規則、產出流程、.speclink.yaml），主區 max-w-3xl 置中、2xl 大標、16px 圓角卡片、每列標籤＋說明在左／控制項在右；「專案設定」與「設定」合為一個入口；需 MODIFIED desktop-app「側欄導覽結構」與 desktop-config 設定頁需求，排在 add-copilot-tool 之後
- Logo：packages/ui 新增 BrandMark（圖標）與 Wordmark（圖標＋字）兩個元件、資產改 SVG 放一處；Wordmark 只出現在側欄頂部一處，系統匣面板與關於視窗只用 BrandMark；使用者之後依新風格重畫 Logo 時只換這兩個 SVG；現況為 PNG，渲染於 App.tsx:559–564 與 server-web Wordmark.tsx，資產重複於 apps/desktop/public、apps/server-web/src/assets、docs/assets/brand 三處
**Ruled out**: 點綴色換紫——蓋章要另找零佔用色相、改 reviewStyle／verifyStyle／theme.test 白名單與 spec 字面，且與參考風格的來源撞色；設定頁只換皮不動導覽——與使用者要的形狀不符
**Open**: 左側導覽形態（Codex 桌面版式的圖示列，是否加第二欄）；系統匣面板底色偏白與「已轉出」收進討論區塊下方並記憶收合狀態；多色彩原則（主色 teal＋白底之外，看板欄與計數等可帶不同色相）；切刀

### Round 3 — interview (2026-10-08)

**Focus**: 左側導覽形態——Codex 桌面版式的「圖示列＋一欄」要怎麼套到 Speclink 的專案脈絡模型
**Position**: 上輪的 A（只做圖示列）與 B（第二欄放專案清單）都被否決；使用者澄清模型——專案（分頁）是頂層脈絡，變更、已封存、規格、手冊與專案設定都屬於每個專案，只有「設定」是全域；左側要照這個層級設計。系統匣面板（底色偏白、已轉出收進討論區塊並記憶）與多色彩原則（teal＋白底為主，生命週期四欄各一色相：討論 fuchsia、提案中 teal、進行中 sky、已就緒 emerald；amber／red／violet 保留給警示／錯誤／蓋章）兩條假設未被反對，視為成立。
- 現況層級證據：ProjectTabs 在頂欄（desktop-config「專案分頁列存於 app 本機」）、側欄六項中四頁＋專案設定讀的是作用中專案、「設定」不依賴任何分頁（desktop-app「側欄導覽結構」）
- 使用者另新增一項：抽屜（變更、討論、規格、已封存）內部的排版也要調整
**Ruled out**: A 圖示列只放四頁導覽、分頁留頂列——沒有表達「頁面屬於專案」的層級；B 第二欄放專案清單——專案通常只有 2–5 個，第二欄多為空白
**Open**: 依專案脈絡模型的左側設計（圖示列＝專案＋全域設定、第二欄＝該專案的導覽）；抽屜內部排版；切刀

### Round 4 — assumptions (2026-10-08)

**Focus**: 左側導覽 C 案是否定案，與抽屜內部排版的假設是否成立
**Position**: C 案定案；抽屜內部排版的假設未被反對、視為成立；使用者另加三項需求（任務列整列可點、系統匣開啟項目撞上既有浮層、markdown 渲染套件）。
- C 案：圖示列（約 56px，全域）＝頂 BrandMark、每個專案一個首字母方塊（作用中 teal 底白字）、「＋」新增 Workspace、底「設定」齒輪；專案欄（200px，每專案）＝頂專案名＋▾（關閉／在 Finder 顯示／複製路徑）、變更／已封存（計數）／規格／手冊、底專案設定；標題列 ⊟ 把專案欄收成 48px 只剩圖示、狀態記憶；主區頂列 48px＝頁名在左、頁級動作在右；零專案時圖示列只有 BrandMark、＋、設定，主區為「開啟專案」空狀態（放 Wordmark）
- 設定模式改回兩個入口：全域「設定」→ 專案欄換成應用程式設定分類（一般、語言與外觀、更新、CLI、伺服器）；「專案設定」→ 專案欄換成專案設定分類（專案說明、產出規則、產出流程、.speclink.yaml）＋「← 返回」；兩者共用同一套設定版面
- 專案方塊：hover tooltip 全名、右鍵選單、路徑失效＝角落紅點、⌃Tab 與 ⌃1–9 不變、不顯示計數徽章；與系統匣面板的專案 tab 同形
- 抽屜骨架（變更、討論、規格、已封存四種共用）：標題列（名稱＋複製＋章籤靠右）、出身列（灰字，email 收進提示）、進度列、工具列（「分析」次要鈕＋依階段的主要鈕＋⋯選單，刪除為選單底部紅字項）、膠囊式分頁（h-8、圓角 8px、作用中淺灰底、計數徽章）、閱讀欄 max-w 72ch 置中留白、15–16px 行高 1.7、h2 降為 16px；寬度維持 720px／42vw 與放大鈕；遮罩 40%→20%；可拖曳改寬先不做
- spec 影響：desktop-config「專案分頁列存於 app 本機」MODIFIED（頂欄分頁列改左側專案列，持久化／上限 10／去重／關閉／錯誤態／快捷鍵全保留）；desktop-app「側欄導覽結構」MODIFIED（六項拆成專案欄五項＋全域一項）；「變更詳情抽屜標頭的四層結構」MODIFIED（標題列、出身列、進度列、工具列）；「抽屜與浮層的開關動畫」只改遮罩濃度
- Logo 位置修正：BrandMark 在圖示列頂、Wordmark 只在空狀態與關於視窗
**Ruled out**: 抽屜改成固定在版面內的右側面板、不蓋遮罩——720px 面板會把看板壓到約 500px 寬；第二輪的「專案設定與設定合成一個入口」——與使用者的「專案設定屬專案、設定屬全域」模型不符
**Open**: 任務列整列可點的做法；系統匣開啟項目時主視窗既有浮層（新增 Workspace 等）的互斥；markdown 渲染套件（現用 react-markdown，Streamdown 曾被 desktop-reading-and-tasks-ux 否決）；切刀

### Round 5 — interview (2026-10-08)

**Focus**: markdown 渲染引擎要不要從 react-markdown 換成 Streamdown
**Position**: 換成 Streamdown（static 模式），任務列整列可點、浮層互斥修正、切刀全數成立。
- 重開先前裁定：desktop-reading-and-tasks-ux 否決 Streamdown 的理由是「串流場景特化，靜態文件無需求」；Streamdown 2.6.0 已有 mode="static"，且使用者要的是它內建的程式碼區塊介面（Shiki 上色、複製鈕、統一樣式），舊理由不再成立
- 可行性證據（讀套件本體）：peer React 18／19；匯出 defaultRemarkPlugins／defaultRehypePlugins 可附加自家 plugin；props 含 remarkPlugins、rehypePlugins、components、controls（code copy／download、table、mermaid 可關）、translations、icons（可接 lucide）、linkSafety；程式碼上色在選配套件 @streamdown/code；元件 class 用 shadcn 語意 token（bg-muted、text-primary、border-border），packages/ui/src/theme.css 已全部定義，不必補 token；深色靠 dark: 變體，與現行 media-query 深色一致；Tailwind 要加 @source 指到 node_modules/streamdown/dist
- 要搬的四個行為（packages/ui/src/components/Markdown.tsx）：remarkGithubAlerts（自寫 plugin，產出帶 class 的 div；Streamdown 走 rehype-sanitize，傳自訂 rehypePlugins 後不再自動擴充 schema，alert 的 class 要加進 sanitize schema）、remark-breaks（附加到 defaultRemarkPlugins 之後）、skipHtml（改由 rehype-raw＋sanitize 承擔，HTML 註解不進畫面）、prose max-w-[96ch]（行寬上限保留）；apps/desktop/src/index.css 的 11 條 .markdown 覆寫逐條對照 Streamdown 預設後刪減
- 影響面：Markdown 元件有 8 個消費者（ManualPage、SectionedDoc、四種抽屜、DeltaBadges、index.ts），單點替換；markdownAlerts.test.tsx 要改；spec「markdown 內容保留文件結構呈現」「raw HTML 不以原文呈現」「markdown 文件內容行寬有上限」「Markdown 的 GitHub Alert 提示框」行為不變、零 delta；歸在地基刀，取代「react-markdown 加 Shiki」
**Ruled out**: react-markdown 加 @shikijs/rehype——使用者裁定要 Streamdown 的整套程式碼區塊介面，不只上色
**Open**: 寫結論（含切刀與 Workspace／專案同物兩名的詞彙漂移）

### Round 6 — interview (2026-10-08)

**Focus**: 色彩分三層（theme.css 基底 token、TS 語意色常數表、元件上的 class）要不要統整
**Position**: 統整成「色值只在 theme.css、對照只在 TS 表、元件只寫 token class」。
- 現況三層：packages/ui/src/theme.css 16 個基底 token（白、灰階、teal、紅、邊框、焦點環，淺深各一套）；tone.ts／stage.ts／reviewStyle.tsx／verifyStyle.tsx／DeltaBadges.tsx 以 Tailwind 原生色階寫語意色，每個用法都要 -600 與 dark:-400 成對；17 個原語與約 33 個領域元件自己寫陰影、圓角、填滿色
- 統整後：theme.css 新增語意 token（status-progress＝sky、status-success＝emerald、status-warning＝amber、status-danger＝red、stamp＝violet）、生命週期 token（stage-discussion＝fuchsia、stage-proposed＝teal、stage-in-progress＝sky、stage-ready＝emerald）與面 token（sidebar、code-bg），淺深各寫一次；TS 表縮成「狀態→token class」一行對照；元件不出現原生色階；透明度寫法沿用 Tailwind v4 的 token 修飾（bg-status-progress/10，與現有 bg-primary/8 同機制）
- 守門：theme.test.ts 從「白名單檔案以外不准原生色階」改為「theme.css 以外全 repo 不准原生色階」，白名單取消
- 淺色基底的中性灰已與目標風格同值（#f5f5f5／#e5e5e5／#737373＝neutral-100／200／500），token 層真正要改的只有：加 sidebar、圓角四階、深色底去藍調（色相 260→中性）、code-bg
- 重開先前裁定：card-drawer-header-colors 否決「CSS token 落地」的理由是「現行慣例是 TS 表，加 token 多一套機制」；本次三層全部重寫，token 讓 TS 表縮成一行對照、深色不必成對寫，舊理由不成立
**Ruled out**: 維持三層只改值——深色成對寫法與白名單守門繼續存在；語意色全搬進 TS 表——深色仍要寫兩遍且 server-web、系統匣無法共用一個來源
**Open**: 寫結論

### Round 7 — assumptions (2026-10-08)

**Focus**: 未談的四個畫面（新增專案對話框、專案設定的編輯、設定頁內容、抽屜切換）與詳情的呈現形態
**Position**: 詳情改為「詳情頁」取代四種抽屜；專案說明與產出規則都用 CodeMirror；設定頁與專案設定照五個零件排；新增專案對話框換皮不換流程；所有頁面對齊同一套版面規則；系統匣面板同語言；分頁用卡片標頭式。使用者在互動原型（八張畫板）上逐一確認。
- 詳情頁：點卡片後主區整個換成該項目的頁；頂端 48px 麵包屑（speclink / 變更 / 名稱）右端 ‹ n／m ›（順序＝看板呈現順序含搜尋過濾，到頭不循環，快捷鍵 ⌥↑／⌥↓，焦點在文字輸入時不作用）；內容欄置中 768px：22px 等寬名稱＋複製＋章籤、灰字出身列（email 收進提示）、右側「分析」「依階段的主要鈕」「⋯（封存、刪除）」；「狀態」卡三列（任務進度、品質站、排程）；分頁列是內容卡的標頭（底線式、全寬、作用中 teal 底線），內文在同一張卡裡；任務分頁每個群組一張卡、整列可點勾選；Esc 或麵包屑回看板，看板捲動與搜尋保留；規格、已封存、討論同樣各自成頁、在各自清單內 ‹ ›；系統匣「開啟此變更」直接落到頁
- 所有頁面的版面規則：頂端 48px 是標題列——圖示列放紅綠燈、專案欄放「專案名 ▾」與收合鈕（設定模式放「設定」、專案設定模式放「← 專案設定」）、主區放麵包屑；內容欄置中 768px、上緣 28px、24px 一般字重頁標題＋灰字說明＋右側頁級動作；區段＝14px 小標＋16px 圓角卡，卡內一列一筆細線分隔、左標籤說明右控制項；內文在卡裡 15px 行高 1.75
- 設定頁零件（五個）：settings-page-header、settings-card、settings-row、settings-choice（列右側下拉）、settings-group-header（圖示＋標籤＋計數）；全域設定分類：一般（顯示語言、外觀：淺色／深色／跟隨系統）、語言與外觀、更新（目前版本、檢查更新、更新日誌三列）、CLI（已安裝版本＋重新安裝）、伺服器、關於；專案設定分類：專案說明、產出規則、產出流程、.speclink.yaml（唯讀列）
- 專案說明：CodeMirror 6 markdown 模式（行號、標題與行內 code 上色、自動換行）、頁標題右側「編輯」→「編輯／預覽」膠囊切換＋取消＋儲存（⌘S）、預覽走 Streamdown；存回保留原有縮排與檔尾換行
- 產出規則：不是 markdown、是 YAML 字串清單——每個產出物一個 CodeMirror 純文字區，一行一條、行序＝注入順序、空行略過、儲存整份寫回 rules；與專案說明共用同一個編輯器元件
- 新增專案對話框：四步與步驟條不動；12px 圓角、p-5、30% 中性遮罩；來源改可選卡片（細框、hover 框變深、選中 teal 淡底）；最近開啟列 8px 圓角、hover 灰底、× hover 才現；取消為 ghost；標題「新增專案」
- 系統匣面板：底色偏白；每個分區是白底細框 12px 圓角卡，分區標題 12px 粗體＋色相圖示＋計數，列以細線分隔、hover 灰底；已轉出收在討論卡底部一列；專案 tab 列與主視窗圖示列同形
- 分頁樣式四選一（膠囊、填滿、底線、卡片標頭）在原型上以 Tweaks 比較，裁定卡片標頭；共用 tabs 原語改為此樣式（卡片標頭式底線）
- Logo 在原型以 teal 方塊「S」佔位，等使用者重畫
**Ruled out**: D 非模態抽屜加 ‹ ›——使用者看過原型後選詳情頁；TipTap 所見即所得編輯專案說明——要自扛 Markdown 往返契約，餵 AI 的設定文字需逐位元保真；產出規則改清單卡逐條編輯——使用者偏好與專案說明同一個編輯器、一行一條；膠囊／填滿／底線分頁——七個帶徽章的分頁需要全寬，且與「內容在卡裡」的規則不如卡片標頭貼合；專案欄頂端只放收合鈕——留下 48px 空白
**Open**: 無——補寫結論

### Round 8 — interview (2026-10-08)

**Focus**: 專案欄要不要有收合鈕
**Position**: 不收合——拿掉收合鈕，專案欄固定 200px。
- 收合成 48px 只剩圖示時，會與左邊 56px 的圖示列並排成兩條圖示欄，形狀怪且分不清哪條是專案、哪條是頁面
- 專案欄只有五個項目，收合省下約 150px，看板欄寬的收益小；少一個要持久化的狀態
- 專案欄頂端 48px 只放「專案名 ▾」；原型三張畫板（看板、詳情頁、新增專案）已拿掉收合鈕
**Ruled out**: 收合成圖示欄——兩條圖示欄並排；收合時整欄隱藏——四頁導覽會消失、圖示列放的是專案不是頁面
**Open**: 無

### Round 9 — interview (2026-10-08)

**Focus**: macOS 紅綠燈要放哪裡——56px 的圖示列塞不下三顆燈
**Position**: 左側頂端 48px 改成一條橫跨圖示列與專案欄的連續標題列。
- 圖示列與專案欄共用側欄底色，頂端 48px 內沒有分隔線；兩欄的分隔線從 48px 以下才開始
- 紅綠燈在 x=16（trafficLightPosition {x:16,y:17}）；標題列內容從約 88px 起：看板與詳情頁放「專案名 ▾」，設定模式放「設定」，專案設定模式放「← 專案設定」＋右端等寬小字專案名
- BrandMark 從標題列下方的圖示列第一格開始；主區頂端 48px 仍是麵包屑
- Windows：沒有紅綠燈，左側標題列內容從 12px 起，自繪的三顆視窗鈕放主區頂列右端；Linux：原生標題列在上方，下面同一套
- 三顆燈需要約 68px，56px 圖示列塞不下；原型六張畫板已改
**Ruled out**: 紅綠燈塞在 56px 圖示列內——擠不下、BrandMark 被壓到燈下；圖示列加寬到 72px——為三顆燈多出 16px 全高的空白
**Open**: 無

### Round 10 — interview (2026-10-08)

**Focus**: 左側標題列的「專案名 ▾」下拉要放什麼——會不會和圖示列的專案方塊打架
**Position**: 會打架，拿掉「▾」：專案名改成純文字標籤，專案動作改放標題列右端的「⋯」選單。
- 名稱旁的「▾」會被讀成「切換專案」，而切換專案已經是圖示列的事；同一件事兩個入口，使用者會猜哪個是正的
- 專案名純文字、hover 顯示完整路徑；右端「⋯」開專案動作選單：在 Finder 顯示、複製路徑、重新整理、關閉專案；圖示列方塊的右鍵選單是同一份
- 原型三張畫板（看板、詳情頁、新增專案）已改
**Ruled out**: 「▾」作切換專案選單——與圖示列重複；「▾」作專案動作選單——符號語意是切換，要學才知道；只靠圖示列右鍵——Windows／Linux 使用者較難發現，標題列的「⋯」便宜且看得見
**Open**: 無

### Round 11 — assumptions (2026-10-08)

**Focus**: 檔案系統動作——專案與文件要補哪些「在 Finder 顯示」這類常用功能
**Position**: 補兩層：專案層在標題列「⋯」，文件層在詳情頁內容卡標頭右端的「開啟 ▾」。
- 專案層「⋯」（與圖示列方塊右鍵同一份）：在 Finder 顯示、在終端機開啟（專案根目錄；macOS 用系統 Terminal、Windows 用 Windows Terminal 退回 cmd、Linux 用系統預設終端）、以編輯器開啟（交給作業系統對資料夾的預設 App）、複製路徑、重新整理、關閉專案
- 文件層「開啟 ▾」：對目前分頁的檔案（proposal.md、design.md、tasks.md、specs/<capability>/spec.md、討論記錄 .md、正典 spec.md、已封存的同名檔）提供：以預設 App 開啟、在 Finder 顯示、複製路徑；沒有對應檔案的分頁（排程）不顯示
- 標籤隨平台：macOS「在 Finder 顯示」、Windows「在檔案總管顯示」、Linux「在檔案管理員顯示」
- 遠端專案（server）沒有本機檔案：專案層只剩重新整理與關閉，文件層只剩「複製內容」
- 落點：apps/desktop/src-tauri 既有 tauri-plugin-opener（reveal 與 open path），終端機開啟需各平台一條指令，歸 apps/desktop/core
- 原型：看板與詳情頁標題列的「⋯」提示列出六項；詳情頁內容卡標頭右端加「開啟 ▾」
**Ruled out**: 另存為——檔案本來就在 repo 裡，複製一份只會製造第二真相；文件層放到頁標題動作列——那一列是變更的動詞（分析、退回、封存），檔案動作屬於目前看的那一份文件、放卡標頭才對位
**Open**: 無

### Round 12 — interview (2026-10-08)

**Focus**: 看板欄與系統匣面板的容器風格不一致——主畫面是灰底欄＋白卡，面板是白底細框卡＋細線列，要對齊哪一邊
**Position**: 對齊面板：看板欄改成白底細框 16px 圓角卡，卡片縮一階為 12px 圓角、hover 框變深底微灰。
- 一條規則貫穿全 app：「容器＝白底細框卡；項目＝容器內的列（細線分隔）或小卡（圓角小一階）」——設定頁的卡與列、面板的分區卡與列、看板的欄與卡，三處同一套
- 看板欄：3px 色相頂條、標頭列（圖示＋名稱＋計數）下有細線、欄底「已轉出」收合列以細線分隔；頁面底色維持白，欄靠邊框分層
- 不反過來把面板改成灰底分區＋白列卡：320px 寬下列不能再包一層卡，密度掉太多
- 原型看板、新增專案背景與詳情頁已更新；兩個檔案系統動作選單（專案層「⋯」、文件層「開啟 ▾」）畫成開啟狀態：12px 圓角面板、8px 圓角項目、hover 灰底、分隔線內縮、關閉專案為紅字、複製路徑與重新整理帶右側提示
**Ruled out**: 看板欄維持灰底——與面板和設定頁的容器語言相反；欄不帶底色只留卡片——四欄的邊界靠空白分辨，標頭與欄底收合列沒有歸屬
**Open**: 無

### Round 13 — assumptions (2026-10-08)

**Focus**: 兩個橫跨頂部的提示條要搬去哪——「技能檔比 Speclink 新」與「有新版本可更新」
**Position**: 依「提示屬於哪一層就放哪一層」拆開：專案層提示放專案欄底部的提示卡，全域更新放圖示列底部的「有新版本」圖示鈕；頂部不再有橫跨主區的 banner。
- 技能檔較新（專案層）：專案欄底部、專案設定上方一張琥珀淡底提示卡——一句標題「技能檔比 Speclink 新」、一句說明、右端 ›；點開確認框（完整說明＋「更新 Speclink」「保留現狀」兩鍵）；跨變更／規格／手冊頁常駐可見、不佔主區高度、不隨頁面捲動；其他專案有同狀況時圖示列方塊角落亮琥珀點
- 有新版本（全域）：圖示列底部、設定齒輪上方一顆「有新版本」圖示鈕（下載箭頭＋琥珀點，tooltip 帶版號）→ 開「設定 › 更新」頁：目前版本、新版本與更新日誌、下載進度、安裝並重新啟動；下載與套用仍先徵得同意；啟動時發現新版本另發一次帶動作鈕的 toast；沒有新版本時圖示鈕不出現
- 兩者都不再推開看板：主區只放目前頁面的內容
- 落點：專案提示卡歸外殼刀，更新頁歸設定刀；spec desktop-app「指令檔過期提示」「指令檔過期提示捲動釘選」「桌面自動更新」MODIFIED
**Ruled out**: 維持頂部橫幅——每次開專案先看到一條警告，看板被推下去，而且提示離它所屬的層（專案／全域）很遠；塞進主區 48px 麵包屑列——放不下說明文字；只用 toast——會消失，提示必須常駐到使用者處理
**Open**: 無

### Round 14 — assumptions (2026-10-08)

**Focus**: 補齊三個畫面——技能檔提示確認框、「設定 › 更新」頁、新增專案對話框對齊新規則
**Position**: 三個畫面都畫進原型並照同一套容器與列的規則。
- 技能檔提示確認框（點專案欄提示卡後）：琥珀圖示＋15px 標題＋灰字說明；一張細框卡三列（專案的技能檔版本、你的 Speclink 版本、會改寫的檔案 37 個：.claude 19、.agents 18）；兩個情境——專案較新：「保留現狀」「更新 Speclink」；Speclink 較新：「稍後」「更新技能檔」；原型以「情境」tweak 切換
- 「設定 › 更新」頁：頁標題右側「檢查更新」；「版本」卡兩列（目前版本＋安裝日期、最新版本＋上次檢查時間與狀態籤）；「下載與安裝」卡一列（版號、大小、發布日、安裝後會重新啟動），依狀態換右側控制——有新版本：下載；下載中：進度列＋取消；已下載：安裝並重新啟動；已是最新：整張卡不出現、圖示列更新鈕也不出現；「更新日誌」卡（最新版本的條目直接展開、舊版本一列一個「檢視」、底部「全部更新日誌」）；「自動檢查」卡一列開關（只檢查、不自動下載）；原型以「狀態」tweak 切換
- 新增專案對話框對齊：「最近開啟」改成 14px 小標＋計數，列放進 12px 細框卡、細線分隔、hover 灰底、× hover 才現；遠端項目帶「遠端」籤；說明補一句「本機資料夾沒有 openspec/ 時會先問你要不要初始化」；頁尾「取消」＋主要鈕「選擇資料夾…」
**Ruled out**: 更新日誌另開彈窗——已有「更新」頁，彈窗是第二個入口；技能檔確認框只有一顆按鈕——兩個情境都有「不做」與「做」兩條路，兩鍵才對稱
**Open**: 無

### Round 15 — assumptions (2026-10-08)

**Focus**: 看板頁的白底層次（頁、欄、卡三層全白會糊在一起）與看板頁內容要寫進規格的細目
**Position**: 欄的卡片區鋪側欄同款的淡灰（neutral-50），欄標頭與欄底收合列維持白底；看板頁的內容細目進結論供各刀 design 參照。
- 層次：頁白 → 欄白底細框 16px 卡（標頭列白、卡片區淡灰 neutral-50、底部收合列白）→ 卡片白底細框 12px；同一條規則套到容器內有「可拖曳項目」的區域：卡片區鋪淡灰，項目白卡才浮得出來；列式容器（設定卡、面板分區）維持白底細線，因為列不是獨立物件
- 看板頁內容細目：麵包屑「專案名 / 變更」；頁標題「變更」＋說明「依生命週期分欄；拖曳卡片調整順序，點卡片開詳情頁」；頁級動作＝搜尋（全圓、280px）＋篩選圖示鈕；四欄等寬 grid、間距 12px；欄＝3px 色相頂條、標頭（色相圖示、名稱、色相淡底計數）、卡片區、討論欄底「已轉出 N ›」收合列；卡片三列骨架不變（等寬名稱＋複製鈕＋右端 meta 圖示與頭像、描述一行截斷、meta 列＝波次章＋進度條＋n/m；討論卡＝狀態 chip、輪數、日期）；卡片 hover 框變深底微灰；拖曳中落點與不合法落點灰化沿用既有規格、只換色成 token；空欄文字「沒有已就緒的變更」置中灰字
**Ruled out**: 三層全白——欄與卡靠同一條細框分不出層；欄整個鋪灰——標頭與收合列沒有「卡」的感覺、回到舊看板
**Open**: 尚未畫的畫面——討論詳情頁、規格頁與規格詳情頁、已封存頁與已封存詳情頁、手冊頁、零專案空狀態、新增專案步驟 2–4 與初始化／遷移／遠端衝突對話框、遠端工作區復原頁、設定的語言與外觀／CLI／伺服器／關於、專案設定的產出流程與 .speclink.yaml、工單三選項框與退回受阻框、看板拖曳中狀態與篩選面板、審查／驗證分頁內容、Windows 標題列變體

### Round 16 — assumptions (2026-10-08)

**Focus**: 補齊九個尚未畫的畫面，並把各自的版面決定寫下
**Position**: 九張都畫進原型（共 19 張畫板），每張沿用「頂端 48px 標題列、置中內容欄、頁標題、容器與列」規則，各自的決定如下。
- 討論詳情頁：麵包屑「專案 / 變更 / slug」＋‹ ›；標題列＝等寬 slug＋複製＋狀態 chip；topic 15px 一行、灰字出身列（頭像、輪數、建立日、更新）；右側「封存」（保留在途時 disabled 並說明由最後一刀自動封存）＋「⋯」；「狀態」卡三列（階段步驟列 討論中→已結論→封存、保留在途說明、衍生變更籤）；卡片標頭分頁 結論／討論過程 N／背景／衍生變更 N＋「開啟 ▾」；結論以欄位標籤（決定、理由、否決的替代方案、延後、落點）呈現
- 規格頁：列式容器——一列一個 capability（等寬名稱、Purpose 首句一行截斷、需求數與變更數籤、更新日、›），頁級動作＝搜尋（比對名稱、Purpose 與需求標題）＋排序下拉，底部換頁（第 N／M 頁、每頁 N 個、‹ ›）
- 規格詳情頁：‹ › 在清單內切；標題列＝等寬名稱＋複製；灰字 meta（需求數、來源變更數、最近更新與來源變更）；右側搜尋需求；卡片標頭分頁 規格 N／溯源 N＋「開啟 ▾」；內容卡＝Purpose 段，之後每條需求一段：Requirement 標籤＋標題＋右端「來自 <變更> · 日期」錨點、需求文（SHALL 等關鍵字以等寬灰底標示）、Scenario 以左側細線縮排區塊呈現；寬於 1200px 時左側目錄列出全部需求
- 已封存頁：頁級搜尋；卡片標頭分頁 變更 N／討論 N；列＝等寬名稱＋描述一行、蓋章籤（已審查／已驗證紫、曾審查未通過琥珀）、任務 n/n 綠籤、觸及規格數、封存日、›；底部換頁；點列開已封存詳情頁（與變更詳情頁同骨架、唯讀）
- 手冊頁：主區分三欄——左 240px 手冊目錄（搜尋＋分組樹，作用中節點 teal 淡底）、中 768px 閱讀卡（24px 頁標題、「可能過期」琥珀籤、出處與時戳行、GitHub Alert 提示框）、右 200px 本頁目錄（細線錨點、作用中 teal）；麵包屑「專案 / 手冊 / 頁名」右端「開啟 ▾」
- 零專案空狀態：圖示列只有 BrandMark、＋、設定，沒有專案欄；主區置中 420px 直欄——Wordmark、24px 標題「開啟一個專案開始」、灰字說明、「新增專案」主要鈕＋「連線 Server」次要鈕、「最近開啟」細框卡列、CLI 提示一行
- 新增專案步驟 2–4：步驟 2 選擇 Server（列＝單選圓點、名稱、網址、登入狀態籤或「重新登入」、底列「新增連線…」）；步驟 3 選擇專案與儲存庫（兩層列：專案列帶代號與儲存庫數，儲存庫列縮排單選）；步驟 4 綁定本機 checkout（路徑輸入＋「選擇…」、驗證結果一行、「記住這個綁定」勾選）；頁尾「上一步」靠左，「取消」「略過，只看規格」「下一步／完成」靠右；原型以「步驟」tweak 切換
- 伺服器設定頁：頁標題右側「新增連線」淡色主色鈕；「連線」卡一列一個 server（名稱＋狀態籤、網址、使用者與到期、右側「登出」或琥珀「重新登入」＋「⋯」）；「預設」卡（新增遠端專案時預設使用、登入過期時提醒）；「登入方式」卡（裝置登入為預設、存取金鑰）
- Windows 標題列：視窗 1px 邊框；左側標題列沒有紅綠燈、內容從 12px 起；主區頂列右端三顆自繪視窗鈕（最小化、最大化、關閉）各 46×48、無底色、hover 灰底、關閉 hover 紅底白字；其餘版面與 macOS 相同
**Ruled out**: 規格頁維持卡片格——規格不是可拖曳物件，列式容器密度較高且與已封存頁同形；手冊目錄併入專案欄——專案欄是頁面導覽，手冊目錄是頁內導覽，混在一起層級不清
**Open**: 無——跑 ingest

### Round 17 — interview (2026-10-08)

**Focus**: 原型三處修正——系統匣面板底色、手冊頁尾、清單頁填滿畫面
**Position**: 三處都改，寫進結論。
- 系統匣面板底色對標桌面：補光層的底色 token 與主視窗同一個（--background），淺色下接近不透明的白，深色下同主視窗深色；分區卡用 --card；不再是獨立調出來的灰
- 手冊頁尾保留既有兩項：閱讀卡底部細線之下「出處」列（來源 capability 的等寬 teal 淡底籤，點了開規格詳情頁）與「上一頁／下一頁」兩顆框線鈕（帶小字標籤與頁名）；目錄樹節點右端維持「可能過期」琥珀小字
- 有換頁的清單頁（規格、已封存）填滿畫面：內容區改全寬，列表卡以 flex 撐到底、卡內捲動，換頁列固定在底部不隨捲動；每頁筆數依視窗高度自動決定（頁尾顯示「每頁依視窗高度自動（現在 N 個）」）
**Ruled out**: 清單頁維持置中 960px 短卡——卡下面一片空白，換頁列浮在中間
**Open**: 無

### Round 18 — interview (2026-10-08)

**Focus**: 原型再修三處——手冊頁尾固定、換頁要能直選頁數、系統匣面板的層次
**Position**: 三處都改，寫進結論。
- 手冊頁尾固定：出處籤與上一頁／下一頁不在閱讀卡裡捲動，而是閱讀欄底部一條固定列（白底、上緣細線、內容對齊 768px 閱讀欄），內文在它上方捲動
- 換頁工具列：左側「第 a–b 筆，共 N 筆」＋「每頁 N 個」下拉；右側 ‹ 1 2 3 4 … M › 頁碼鈕（作用中 teal 淡底）＋「跳到 __ 頁」數字輸入；規格頁與已封存頁共用同一個 pager 元件
- 系統匣面板層次比照看板：面板底用側欄淡灰（--sidebar），專案 tab 列、各分區、底部動作列各是一張白底細框 12px 卡，列以細線分隔——與主視窗「容器白卡 on 淡灰底」同一條規則；之前面板白底加白卡是三層全白
**Ruled out**: 手冊頁尾放在內文末端——長頁要捲到底才看得到出處與翻頁；只有 ‹ › 的換頁——39 頁的已封存清單無法直達
**Open**: 無

### Round 19 — assumptions (2026-10-08)

**Focus**: 新 Logo 做好了——「接合 S」標記、青綠圓角底板 app 圖示、橫式與直式鎖版、深色變體、SVG 原稿——各個表面怎麼擺
**Position**: 依表面分四種用法：圖示列頂放裸標記、空狀態與關於放橫式鎖版、Dock／安裝檔用底板版 app 圖示、選單列用單色 template；資產以 docs/assets/brand/svg 為原稿、介面只收三個 SVG。
- 圖示列頂：裸標記（speclink-logo-mark.svg，無底板），高 26px、置中、標題列下方 8px、下方留 14px；用 currentColor 取 text-primary，淺色＝主色 teal、深色＝淺青綠；不用 app 圖示的「青綠底板＋白 S」——那和作用中專案方塊（teal 底白字母）同形，會被讀成一個專案
- 零專案空狀態與「關於 Speclink」：橫式鎖版（speclink-logo-horizontal.svg，深色用 -dark），高 40px；介面其他地方不再出現字標
- 系統匣面板：不放品牌標記——選單列圖示已經是識別，面板頂端是專案 tab 列
- 選單列（系統匣）圖示：由標記輪廓產生的單色 template（18／36px、含 alpha），macOS 以 template 渲染隨深淺色；與 app 圖示分開產
- App 圖示（Dock、Finder、Windows 安裝檔、工作列）：speclink-app-icon.svg（1024、青綠圓角底板＋白 S、Windows 圓角存在圖檔內）經 tauri icon 一次產出全部尺寸與 icns／ico；已重生
- 資產落點：docs/assets/brand/svg 是原稿（文字已轉路徑）；packages/ui/src/assets 收 logo-mark.svg、logo-horizontal.svg、logo-horizontal-dark.svg 三個（cut 1 的 BrandMark／Wordmark 讀這裡，標記以 currentColor 內嵌、鎖版以 img）；apps/desktop/public 只留 favicon 用的 logo-mark.png；apps/server-web/src/assets 的副本在 cut 1 刪除改引用 ui；README 與文件用 docs/assets/brand/transparent 的 PNG
- 品牌固定色（#167873／#4bb9b3）與介面 token 分開管理，介面標記跟 token 走；cut 1 的 --primary 以品牌青綠的 oklch 值校準一次，讓兩者肉眼一致
- speclink-brand-kit.zip（656KB 二進位）建議不進 repo，改放發版資產或文件站
**Ruled out**: 圖示列頂放 app 圖示底板版——與作用中專案方塊同形；面板頂放標記——重複識別、佔面板高度；每個表面各自存一份 PNG——換 Logo 要改多處，正是這次要收掉的
**Open**: 無

### Round 20 — interview (2026-10-08)

**Focus**: 「關於 Speclink」頁的內容，以及原型補齊品牌擺放後的狀態
**Position**: 關於頁是設定模式「關於」分類下唯一一頁，也是橫式鎖版的第二個表面（第一個是零專案空狀態）；內容四段，全部用設定頁共用零件：
- 品牌卡：橫式鎖版 40px（深色自動換深色版）＋一列灰字版本資訊（app 版號 · 引擎版號 · 平台 · 更新狀態：綠勾「已是最新版」或警示「有新版本」）＋一句產品描述；不再另放裸標記，鎖版已含標記
- 版本：檢查更新（說明列顯示上次檢查時間）、更新日誌（檢視）、複製版本資訊（app、引擎、平台、技能檔版本，回報問題時貼上）；檢查更新與更新日誌和「更新」頁共用同一份狀態與元件，只是多一個入口
- 連結：原始碼（github.com/MomoChenisMe/speclink）、說明文件、回報問題（開 GitHub issue，版本資訊自動帶入）；整列可點、右端外開圖示
- 授權：授權條款（MIT License · © 2026 MomoChen，檢視）、第三方授權（檢視清單）
- macOS 原生選單「Speclink › 關於 Speclink」開這一頁，不用系統預設的關於面板
- 原型同步：第 20 張畫板「設定：關於 Speclink」；17 張有圖示列的畫面頂端改成裸標記 26px（跟主色，深色可切）；零專案空狀態改成真正的橫式鎖版；token 一覽加「品牌」段（裸標記、鎖版、App 圖示底板版、選單列 18px 單色版並排）並把分頁示意改成卡片標頭式；範例專案名改成中性名稱
**Ruled out**: 用系統預設的關於面板——放不下鎖版與連結，也不跟 UI 語言走；關於頁再放一次裸標記——鎖版已含標記，重複
**Open**: 無

## Conclusion

**Decision**: Speclink 桌面 app 整個介面重新設計為「白底中性灰、teal 單一點綴色加生命週期四色、扁平細框分層、圓角分層、隱藏原生標題列」的桌面工具風格，左側依「專案為頂層脈絡」重排，詳情以頁取代抽屜，色彩統整成兩層，分七刀落地。
- 底層不換：React 19＋Tailwind v4＋shadcn 式 Radix 原語＋lucide＋sonner 維持；重新設計＝改 token、元件外觀、視窗殼與頁面結構；看板四欄與變更／已封存／規格／手冊四頁的資訊架構不動
- 風格原則：白底主區、淺灰專案欄、細框分層不靠陰影；teal 只用於品牌、連結、主要操作、互動回饋、進度，不做大面積底色；選取＝teal 淡底深字、hover＝半透明黑；圓角四階（按鈕 6px、清單列 8px、面板 12px、卡片 16px，標籤與搜尋框全圓，取 Tailwind 預設值、移除自訂 --radius）；陰影只給 tooltip／選單／對話框；外殼字 11–13px、頂列 48px、閱讀區 15–16px 行高 1.7；動態 150–200ms、位移 4px、ease-out，動作鈕 hover 才出現；prefers-reduced-motion 時停用
- 所有頁面的版面規則：頂端 48px 是標題列——左側（圖示列＋專案欄）是一條連續標題列，共用側欄底色、頂端 48px 內無分隔線，紅綠燈在 x=16，內容從約 88px 起放純文字專案名（hover 顯示完整路徑）＋右端「⋯」專案動作選單（在 Finder 顯示、在終端機開啟、以編輯器開啟、複製路徑、重新整理、關閉專案；與圖示列方塊的右鍵選單同一份），設定模式放「設定」，專案設定模式放「← 專案設定」＋右端等寬小字專案名，Windows／Linux 無紅綠燈時內容從 12px 起；主區放麵包屑（speclink / 變更、speclink / 變更 / 名稱、設定 / 一般），Windows 自繪的三顆視窗鈕在主區頂列右端；內容欄置中 768px、上緣 28px、24px 一般字重頁標題＋灰字說明＋右側頁級動作；區段＝14px 小標＋16px 圓角卡，卡內一列一筆細線分隔、左標籤說明右控制項；內文在卡裡 15px 行高 1.75；容器規則：容器＝白底細框卡，項目＝容器內的列（細線分隔）或小卡（圓角小一階）——設定頁的卡與列、系統匣面板的分區卡與列、看板的欄與卡同一套；容器內放可拖曳小卡的區域鋪側欄同款淡灰（neutral-50）讓白卡浮出，列式容器維持白底；看板、詳情頁、設定、專案設定全部照此
- 多色原則：生命週期四欄各一色相——討論 fuchsia、提案中 teal、進行中 sky、已就緒 emerald；單一來源 stage.ts，看板欄頂條、欄計數徽章、卡片波次章、系統匣分區圖示與計數共用；狀態語意色不變（sky 進行中、emerald 成功、amber 警示、red 錯誤、violet 品質站蓋章專屬、indigo 改進標示）；amber／red／violet 不給欄位；導覽圖示與靜態 meta 中性；每個色相只有一個意思
- 色彩統整成兩層：色值只在 packages/ui/src/theme.css——基底 16 個 token 之外新增語意 token（status-progress＝sky、status-success＝emerald、status-warning＝amber、stamp＝violet、improve＝indigo）、生命週期 token（stage-discussion、stage-proposed、stage-in-progress、stage-ready）與面 token（sidebar），淺色深色各寫一次、原生色階值逐字取自 Tailwind 的 theme.css；深色基底去藍調改純中性；tone.ts／stage.ts／reviewStyle.tsx／verifyStyle.tsx／improveStyle.tsx／DeltaBadges.tsx 縮成「狀態→token class」對照；元件只寫 token class、不出現原生色階（含中性色階）、透明度用 token 修飾；theme.test.ts 改為「theme.css 以外全掃描面不准原生色階」、白名單取消
- 深色模式保留並補同風格深色值；設定頁新增「外觀」列（淺色／深色／跟隨系統，存 app 本機）
- 不動的既有裁定：省略號截斷、Noto Sans TC 打包、卡片三列骨架、三層語意色角色、側欄四頁順序、產出規則不做逐項輸入框
- 左側導覽：圖示列（約 56px，全域）自標題列下方起由上而下為 BrandMark、每個專案一個首字母方塊（作用中 teal 底白字、其餘灰底；hover tooltip 全名；右鍵選單：關閉、在 Finder 顯示、複製路徑；路徑失效＝角落紅點；不顯示計數徽章；⌃Tab 與 ⌃1–9 不變）、「＋」新增專案、底部全域「設定」；專案欄（固定 200px、不收合，每專案）自標題列下方起為變更、已封存（計數）、規格、手冊，底部專案設定；零專案時圖示列只有 BrandMark、＋、設定，主區為「開啟專案」空狀態放 Wordmark；專案方塊與系統匣面板的專案 tab 同形
- 詳情頁取代四種抽屜：點卡片後主區整個換成該項目的頁；麵包屑右端 ‹ n／m ›（順序＝看板呈現順序含搜尋過濾，到頭不循環，快捷鍵 ⌥↑／⌥↓，焦點在文字輸入時不作用）；內容欄 22px 等寬名稱＋複製＋章籤、灰字出身列（email 收進提示）、右側「分析」＋依階段的主要鈕＋「⋯」（封存、刪除為選單底部紅字項）；「狀態」卡三列（任務進度、品質站、排程）；分頁列是內容卡的標頭（底線式、全寬、作用中 teal 底線，計數徽章跟著），內文在同一張卡裡；任務分頁每個群組一張卡、整列為勾選框的 label（拖曳把手與連結除外、唯讀與批次寫回中不變）；Esc 或麵包屑回看板，看板捲動與搜尋保留；規格、已封存、討論同樣各自成頁、在各自清單內 ‹ ›；系統匣「開啟此變更」「開啟此討論」直接落到頁；主區沒有浮層，對話框只剩真正的對話框
- 提示的歸屬：頂部不再有橫跨主區的 banner；專案層提示（技能檔比 Speclink 新）放專案欄底部、專案設定上方的琥珀淡底提示卡（一句標題、一句說明、›），點開確認框（琥珀圖示＋標題＋說明、一張細框卡三列：專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數；專案較新時「保留現狀」「更新 Speclink」，Speclink 較新時「稍後」「更新技能檔」），跨頁常駐、不佔主區、不隨捲動，其他專案同狀況時圖示列方塊角落亮琥珀點；全域更新放圖示列底部、設定齒輪上方的「有新版本」圖示鈕（下載箭頭＋琥珀點、tooltip 帶版號），開「設定 › 更新」頁：頁標題右側「檢查更新」，「版本」卡兩列（目前版本＋安裝日期、最新版本＋上次檢查與狀態籤），「下載與安裝」卡一列依狀態換控制（有新版本：下載；下載中：進度列＋取消；已下載：安裝並重新啟動；已是最新：整張卡不出現），「更新日誌」卡（最新版本條目展開、舊版本一列一個檢視、底部全部更新日誌），「自動檢查」卡一列開關；下載與套用仍先徵得同意，啟動時發現新版本另發一次帶動作鈕的 toast，沒有新版本時圖示鈕不出現
- 檔案系統動作：專案層在標題列「⋯」（在 Finder 顯示、在終端機開啟——macOS 系統 Terminal、Windows 用 Windows Terminal 退回 cmd、Linux 系統預設終端——、以編輯器開啟、複製路徑、重新整理、關閉專案）；文件層在詳情頁內容卡標頭右端的「開啟 ▾」（對目前分頁的檔案：以預設 App 開啟、在 Finder 顯示、複製路徑；沒有對應檔案的分頁不顯示）；標籤隨平台（Finder／檔案總管／檔案管理員）；遠端專案沒有本機檔案時專案層只剩重新整理與關閉、文件層只剩「複製內容」；落點 tauri-plugin-opener 與 apps/desktop/core 的終端機指令
- 共用 tabs 原語改為卡片標頭式底線分頁（取代膠囊式）
- 設定模式：點全域「設定」→ 專案欄換成分類清單＋搜尋（應用程式：一般、語言與外觀、更新、CLI、伺服器；關於）；點「專案設定」→ 專案欄換成「← 專案設定」＋分類（專案說明、產出規則、產出流程、.speclink.yaml 唯讀列）；兩者共用五個設定零件：settings-page-header、settings-card、settings-row、settings-choice（列右側下拉）、settings-group-header（圖示＋標籤＋計數）；一般頁內容：顯示語言與外觀兩列下拉、更新三列（目前版本、檢查更新、更新日誌）、CLI 一列（已安裝版本＋重新安裝）；關於頁內容：品牌卡（橫式鎖版 40px＋灰字版本列「app 版號 · 引擎版號 · 平台 · 更新狀態」＋一句產品描述，不另放裸標記）、版本三列（檢查更新含上次檢查時間、更新日誌、複製版本資訊——前兩列與更新頁共用同一份狀態）、連結三列（原始碼、說明文件、回報問題；整列可點＋外開圖示）、授權兩列（授權條款 MIT、第三方授權）；原生選單「Speclink › 關於 Speclink」開此頁
- 專案說明編輯：CodeMirror 6 markdown 模式（行號、標題與行內 code 上色、自動換行），頁標題右側「編輯」→「編輯／預覽」膠囊切換＋取消＋儲存（⌘S），預覽走 Streamdown；存回保留原有縮排與檔尾換行
- 產出規則編輯：每個產出物一個 CodeMirror 純文字區，一行一條、行序＝注入順序、空行略過、儲存整份寫回 rules；與專案說明共用同一個編輯器元件
- 新增專案對話框：四步與步驟條不動；12px 圓角、p-5、30% 中性遮罩；15px 標題＋灰字說明（本機資料夾沒有 openspec/ 時會先問要不要初始化）；來源改可選卡片；「最近開啟」為 14px 小標＋計數，列放進 12px 細框卡、細線分隔、hover 灰底、× hover 才現、遠端項目帶「遠端」籤；頁尾「取消」ghost＋主要鈕「選擇資料夾…」；標題「新增專案」
- 標題列：macOS 用 titleBarStyle Overlay＋hiddenTitle＋trafficLightPosition {x:16,y:17}、decorations 維持 true；Windows 用 decorations false＋自繪 48px 拖曳列與三顆視窗鈕、shadow true 保留 Win11 圓角，代價是失去貼齊版面浮窗；Linux 維持原生標題列、下方同一套版面
- macOS 原生選單：Rust MenuBuilder 自建、跟 UI 語言偏好、語言切換時 app.set_menu 重建；Speclink（關於、檢查更新…、設定… ⌘,、服務、隱藏、結束）、檔案（開啟專案… ⌘O、關閉分頁 ⌘W、安裝 CLI…）、編輯（復原、重做、剪下、複製、貼上、全選——預設項必留）、檢視（變更／已封存／規格／手冊 ⌘1–4、專案設定、搜尋看板 ⌘F、重新整理 ⌘R、下一個分頁 ⌃Tab）、視窗（縮到最小、縮放、全部移到最前）、說明（手冊、更新日誌、GitHub、回報問題）；Windows／Linux 不做選單列
- 共用元件：packages/ui 為唯一的家，components/ui 原語＋領域元件兩層；只建有消費者的元件、各自隨第一個消費者所在的刀落地（地基刀：ConfirmDialog、CopyButton、EmptyState、SectionHeader、BrandMark、Wordmark；外殼刀：dropdown-menu、kbd、page-header；設定刀：settings 五零件、switch、search-input；詳情刀：status-pill）；desktop 與 server-web 的本地重複改 import；新守門測試：apps/*/src 不得定義與 @speclink/ui 匯出同名的元件；openspec/config.yaml 產出規則加「優先用 @speclink/ui 既有元件、呼叫端不覆蓋變體的顏色／圓角／陰影」
- Markdown：react-markdown 換成 Streamdown（static 模式，選配 @streamdown/code 做 Shiki 上色、內建複製鈕接 lucide 圖示、關閉 table 與 mermaid 控制）；搬四個行為——remarkGithubAlerts 附加到 defaultRemarkPlugins 並把 alert 的 class 加進 sanitize schema、remark-breaks 附加、skipHtml 改由 rehype-raw＋sanitize 承擔、行寬上限保留；移除 @tailwindcss/typography；相關 spec 行為不變
- 浮層互斥：openDetail／openDiscussion 在狀態層同時清掉所有可取消浮層（新增專案、更新日誌、未確認的初始化／啟用／封存確認框）；遷移與遠端衝突進行中不清、只帶主視窗到前景；所有入口走同一條動作；詳情頁落地後同一規則適用於開頁
- 系統匣面板：層次比照看板——面板底用側欄淡灰（--sidebar，淺色下補光層約 90% 不透明；HudWindow 材質不換），專案 tab 列、各分區、底部動作列各是一張白底（--card）細框 12px 圓角卡；分區標題 12px 粗體＋色相圖示＋計數，列以細線分隔、hover 灰底；「已轉出 N ›」收在討論卡底部一列、預設收合、展開狀態存 app 本機；非 macOS 原生選單維持獨立分區；看板討論欄底的「已轉出 N」收合列同樣加記憶
- Logo（新標記「接合 S」已完成）：packages/ui/src/assets 收 logo-mark.svg、logo-horizontal.svg、logo-horizontal-dark.svg 三個 SVG（原稿在 docs/assets/brand/svg，文字已轉路徑）；BrandMark＝裸標記以 currentColor 內嵌、高 26px、置於圖示列頂（標題列下 8px、下留 14px）、取 text-primary 隨深淺色，不用 app 圖示的底板版以免與作用中專案方塊同形；Wordmark＝橫式鎖版（深色用 -dark）高 40px，只在零專案空狀態與「關於 Speclink」；系統匣面板不放品牌標記；選單列圖示為標記輪廓的單色 template（18／36px）；App 圖示由 speclink-app-icon.svg 經 tauri icon 產出全部尺寸與 icns／ico（已重生）；apps/desktop/public 只留 favicon 用的 logo-mark.png，server-web 的副本於 cut 1 刪除改引用 ui；品牌固定色與介面 token 分開管理，cut 1 以品牌青綠的 oklch 值校準 --primary 一次；brand-kit.zip 不進 repo
- server-web：共用 theme.css 自動吃到新 token；ConsoleLayout.tsx 的外殼尺寸隨外殼刀同步
- 詞彙漂移：「新增 Workspace」「加入專案」「專案分頁」三處指同一件事，LANGUAGE.md 定「專案」為正典、「Workspace」列 avoid，於圖示列「＋」落地時收斂
- 其餘頁面的版面：規格頁與已封存頁為列式容器（一列一筆、籤與日期靠右、›），內容區全寬、列表卡以 flex 撐滿畫面並在卡內捲動，底部固定換頁工具列（左：第 a–b 筆共 N 筆＋每頁 N 個下拉；右：‹ 頁碼 1 2 3 … M › 作用中 teal 淡底＋「跳到 __ 頁」數字輸入；規格頁與已封存頁共用同一個 pager 元件），頁級搜尋；已封存頁以卡片標頭分頁分變更／討論；規格詳情頁與討論詳情頁與變更詳情頁同骨架（標題列、meta、狀態卡、卡片標頭分頁、內容卡、‹ ›），規格詳情的需求以「Requirement 標籤＋標題＋來源錨點、需求文、左線縮排的 Scenario」分段、寬於 1200px 時左側列需求目錄，討論詳情的狀態卡含階段步驟列與衍生變更籤、結論以欄位標籤呈現；手冊頁主區三欄（240px 目錄樹——節點右端「可能過期」琥珀小字、768px 閱讀卡、200px 本頁目錄），閱讀欄底部一條固定列（白底、上緣細線、對齊 768px）放「出處」capability 籤（點開規格詳情頁）與「上一頁／下一頁」框線鈕，內文在其上方捲動；零專案空狀態無專案欄、主區置中直欄（Wordmark、標題、說明、新增專案＋連線 Server、最近開啟卡）；新增專案步驟 2–4 為單選列（Server、兩層專案／儲存庫、checkout 路徑與記住綁定）；伺服器設定頁為連線卡（狀態籤、登出／重新登入、⋯）＋預設卡＋登入方式卡；Windows 視窗 1px 邊框、左側標題列內容從 12px 起、主區頂列右端三顆自繪視窗鈕（各 46×48、hover 灰底、關閉 hover 紅底白字）
- 互動原型：二十張畫板（看板、詳情頁、新增專案、設定一般頁、專案說明、產出規則、系統匣面板、token、原語與品牌一覽、技能檔提示確認框、設定更新頁、討論詳情頁、規格頁、規格詳情頁、已封存頁、手冊頁、零專案空狀態、新增專案步驟 2–4、伺服器設定頁、Windows 標題列、設定關於頁）記錄於 https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3，各刀 design 以它為視覺基準
- **cut 0 `fix-drawer-open-dismisses-overlays`**: 獨立 bug 修，不等換皮（已轉出）
  - openDetail／openDiscussion 清掉可取消浮層、遷移與遠端衝突進行中不開
  - desktop-app「detail 抽屜互斥」MODIFIED
- **cut 1 `desktop-design-foundation`**: 地基（已轉出；tabs 卡片標頭式與 Logo 三項皆已 ingest）
  - theme.css 新 token、深色去藍調、圓角改 Tailwind 預設；TS 表縮成對照；theme.test.ts 全掃描面無白名單
  - 17 個原語扁平化；六個有消費者的共用元件；三處本地重複改 import；同名本地定義守門；config.yaml 產出規則
  - Markdown 換 Streamdown
  - desktop-app「介面狀態語意色分層」MODIFIED、ADDED「共用元件唯一來源」
- **cut 2 `desktop-shell-redesign`**: 外殼
  - 三平台標題列（左側連續標題列承載紅綠燈與專案名、主區麵包屑）、圖示列＋固定寬專案欄（不收合）、專案層「⋯」檔案系統動作、專案欄底部提示卡與圖示列「有新版本」鈕、頁標題區、零專案空狀態；desktop-app「指令檔過期提示」「指令檔過期提示捲動釘選」「桌面自動更新」MODIFIED
  - 看板頁套用版面規則（24px 標題＋說明、搜尋與篩選為頁級動作）
  - 新增專案對話框換皮與改名（含步驟 2–4 的單選列）、零專案空狀態、Windows 自繪視窗鈕
  - macOS 原生選單中文化與六組功能
  - 新原語 dropdown-menu、kbd、page-header
  - server-web ConsoleLayout 同步
  - desktop-config「專案分頁列存於 app 本機」MODIFIED（頂欄分頁列改左側專案列，持久化／上限 10／去重／關閉／錯誤態／快捷鍵全保留）；desktop-app「側欄導覽結構」MODIFIED；workspace-chooser 外觀相關需求 MODIFIED；LANGUAGE.md 收斂 Workspace
- **cut 3a `desktop-board-reskin`**: 看板
  - 規格頁、已封存頁改列式容器＋換頁；手冊頁三欄版面
  - 看板頁內容：麵包屑「專案名 / 變更」；頁標題「變更」＋說明「依生命週期分欄；拖曳卡片調整順序，點卡片開詳情頁」；頁級動作＝全圓搜尋（280px）＋篩選圖示鈕；四欄等寬 grid、間距 12px
  - 看板欄改白底細框 16px 圓角卡：3px 色相頂條、白底標頭（色相圖示、名稱、色相淡底計數）下細線、卡片區鋪 neutral-50、討論欄底白底「已轉出 N ›」收合列細線分隔並記憶；空欄置中灰字（如「沒有已就緒的變更」）
  - 卡片 12px 圓角白底細框無陰影、三列骨架不變（等寬名稱＋複製鈕＋右端 meta 圖示與頭像、描述一行截斷、meta 列＝波次章＋進度條＋n/m；討論卡＝狀態 chip、輪數、日期）、hover 框變深底微灰；拖曳中落點與不合法落點灰化沿用既有規格、只換成 token 色
  - desktop-app 看板卡片與欄位相關需求的外觀字面 MODIFIED
- **cut 3b `desktop-detail-page`**: 詳情頁取代抽屜
  - 變更、討論、規格、已封存四種詳情頁；‹ › 與快捷鍵；狀態卡（討論頁含階段步驟列與衍生變更籤）；卡片標頭分頁與「開啟 ▾」文件動作；規格詳情的需求分段與需求目錄；任務列整列可點；Esc／麵包屑返回保留看板狀態
  - 新原語 status-pill；Sheet 原語退場
  - desktop-app 約 15 條含「抽屜」的需求 MODIFIED 或 RENAMED 為詳情頁語意（互斥、底層落回看板、標頭結構、開關動畫、skeleton、審查／驗證資訊列、排程與工單分頁、已封存抽屜）；tray-status-menu「變更子選單動作」「討論列表」的開啟語意 MODIFIED；「detail 抽屜互斥」在 cut 0 之後再 MODIFIED 為開頁語意（封存順序在 cut 0 之後）
- **cut 4 `desktop-settings-mode`**: 設定模式（add-copilot-tool 已封存，可隨時開始）
  - 全域設定與專案設定兩種分類欄、五個設定零件、switch 與 search-input 原語、外觀切換列、「更新」頁（新版本、更新日誌、下載進度、安裝並重新啟動）、「伺服器」頁（連線卡、預設卡、登入方式卡）
  - 專案說明 CodeMirror markdown 編輯器（編輯／預覽、⌘S、保真寫回）、產出規則 CodeMirror 純文字一行一條
  - desktop-config 設定頁需求 MODIFIED
- **cut 5 `tray-panel-restyle`**: 系統匣面板，可與 3a、3b、4 並行
  - 淺色補光偏白、白底細框分區卡與細線列、已轉出收進討論卡並記憶、hover／選取語意與圓角對齊
  - tray-status-menu「面板樣式（macOS）」與「討論列表」MODIFIED
**Rationale**: 兩邊技術底層同款，改版的工作量不在色值（淺色中性灰已同值、主色不動）而在元件層的質感、外殼與頁面結構；把色彩收成「值在 theme.css、對照在 TS、元件不寫色」讓深色不必成對寫、換色只改一行、守門不用白名單；左側依「專案為頂層脈絡」排列才與每個專案各有四頁加專案設定、只有設定是全域的資料模型對齊；詳情改成頁讓「主區＝目前在看的東西」從看板、手冊、設定一路貫到詳情，疊層問題從根拔掉，代價是約 15 條需求的字面與一刀的拆分；所有頁面共用一套版面規則（頂端 48px 標題列、置中 768px、24px 標題、區段卡、列）讓新畫面不必再各自設計；依「地基→外殼→看板／詳情／設定／系統匣」切刀，每刀各自跑兩站封存，spec delta 互不重疊，cut 0 的 bug 修不必等換皮。
**Rejected alternatives**:
- 點綴色換紫 — 紫為品質站蓋章專屬，換色要另找零佔用色相並改四處守門與 spec 字面，且與參考風格的來源撞色
- 連看板資訊架構一起改 — 63 條行為需求與 70 個 vitest 檔綁著現有結構，工程量差一個量級
- 圖示列只放頁面導覽、專案分頁留頂列 — 沒有表達「頁面屬於專案」的層級
- 第二欄放專案清單 — 專案通常只有 2–5 個，第二欄多為空白
- 專案欄頂端只放收合鈕 — 留下 48px 空白
- 維持頂部橫幅提示 — 看板被推下去，且提示離它所屬的層很遠；只用 toast 提示 — 會消失，提示必須常駐到使用者處理
- 頁、欄、卡三層全白 — 欄與卡靠同一條細框分不出層
- 看板欄維持灰底 — 與面板和設定頁的容器語言相反；欄不帶底色只留卡片 — 標頭與欄底收合列沒有歸屬
- 文件「另存為」 — 檔案本來就在 repo 裡，複製一份只會製造第二真相
- 標題列的專案名帶「▾」下拉 — 會被讀成切換專案，與圖示列重複；專案動作改放右端「⋯」
- 專案欄可收合成圖示欄 — 與 56px 圖示列並排成兩條圖示欄，分不清專案與頁面；五個項目省 150px 不值，還多一個持久化狀態
- 紅綠燈塞在 56px 圖示列內或把圖示列加寬到 72px — 三顆燈需約 68px，前者擠不下、後者為三顆燈多出全高空白
- 抽屜改成版面內右側面板、不蓋遮罩 — 720px 面板把看板壓到約 500px 寬
- 非模態抽屜加上一頁下一頁 — 使用者看過原型後選詳情頁；疊層問題仍要靠互斥規則
- 專案設定與設定合成一個入口 — 與「專案設定屬專案、設定屬全域」模型不符
- TipTap 所見即所得編輯專案說明 — 要自扛 Markdown 往返契約，餵 AI 的設定文字需逐位元保真；參考做法自己也在解析失敗時退回原始碼模式
- 產出規則改清單卡逐條編輯 — 使用者偏好與專案說明共用同一個編輯器、一行一條；且既有裁定否決逐項輸入框
- 膠囊、填滿、底線三種分頁樣式 — 七個帶徽章的分頁需要全寬，且與「內容在卡裡」的規則不如卡片標頭貼合
- react-markdown 加 @shikijs/rehype — 使用者要 Streamdown 整套程式碼區塊介面，不只上色
- Windows 維持原生標題列 — 空白條在 Windows 還在
- 維持色彩三層只改值 — 深色成對寫法與白名單守門繼續存在
- 語意色全搬進 TS 表 — 深色仍寫兩遍，server-web 與系統匣無共用來源
- 一次建好所有原語 — 沒有消費者的原語是死碼，等不到第一個消費者前無法驗證形狀
- 砍深色模式 — spec 要求淺深一致取自 token，server-web 共用同一份 theme
- 省略號截斷改淡出遮罩 — 既有裁定，重開要改 spec 字面與測試，這次不碰
- 直接寫色階 class 不經 CSS 變數 — 三處守門全紅、深色沒有落點
- Windows／Linux 做選單列 — 隱藏標題列後的行為無機器可查
- 直接以舊 topic 字面記錄 — 參考風格來自私人專案，記錄改用中性風格描述
**Deferred**:
- Windows 隱藏標題列後選單列的行為 — 無 Windows 機器可查，cut 2 design 期實測
- 詳情頁的瀏覽歷史（返回鍵是否經 history、看板捲動保留的實作方式） — cut 3b design 期定
- Streamdown 的數學與 mermaid 外掛 — 規格文件沒有用到
- 四欄色相與語意 token 的確切色階 — design 期在真實畫面調
- 產出規則單條寫回動詞 — 現以整份寫回，單條增刪改另議
**Capture to**: proposal、design、spec（desktop-app、desktop-config、workspace-chooser、tray-status-menu MODIFIED）、tasks、LANGUAGE.md
**Next**: /speclink-propose --from-discussion desktop-ui-redesign（cut 0、1 已轉出；依序 cut 2 → 3a → 3b；cut 4 與 5 可與 3a、3b 並行；最後一刀帶 --last）
