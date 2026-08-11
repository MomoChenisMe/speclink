## Context

現況(已探查):`[P]` 有解析(tasks 模組前綴剝除迴圈)、有上 wire(instructions/query 任務 payload 的 parallel 欄位、server 與 CLI 兩處搬運),但全正典唯一出處是起草指引的翻譯保留規則,無指引教何時加、node SDK 與 UI 與 desktop 零消費;封存區三個舊 change 的 tasks.md 實際帶 `[P]`。desktop 任務列由 packages/ui 的任務解析(僅剝行尾 stable-ID 註解)渲染,`[M]`/`[P]` 字面照印;看板卡片 meta 列只有全量 completedTasks/totalTasks 進度;解剖學正典明定變更卡不帶狀態 chip,但「待重新反映徽章」與「審查/驗證標示」(行內小章、不增加文字列)是合法先例,且審查/驗證標示本就 local-only(remote 卡片無章)。前一 change manual-task-marker-gates(11/11 完成、未封存)已落 `[M]` 解析、寫碼預測子與 codeTotal/codeComplete/codeRemaining 的 apply payload 欄位。

## Goals / Non-Goals

**Goals:**

- `[P]` 語意與 payload 欄位移除,解析容忍舊檔(認得但不承載)
- 起草指引翻譯保留規則改點名 `[M]`
- 任務列:`[M]` 前綴剝離+行尾右對齊「✋ 手動測試」徽章,編號排版零變動
- 看板變更卡:「待手測」行內小章,寫碼收工且手測未勾時浮現
- desktop 協定變更清單項增三個寫碼進度欄
- 「手動測試」「待手測」入共用詞彙

**Non-Goals:**

- `[P]` 啟用(需先有 apply 平行執行機制,獨立案)
- remote 看板的待手測章與 remote 變更摘要 payload(沿審查標示 local-only 先例)
- CLI list --json 的寫碼進度欄位(無消費者)
- 勾選互動加料(高亮/彈窗/分組)
- 徽章 lucide 圖示選型細節(實作時定,Hand 或同義)

## Decisions

**D1 `[P]` 移除深度=認得但不承載。** 解析器的前綴剝除迴圈保留對 `[P] ` 的剝離(順序不敏感、至多一次的既有結構不動),但不再落任何旗標——Task 結構的 parallel 欄位刪除。效果:封存區與外部 repo 的舊檔在所有顯示面(引擎 payload 描述、desktop 原文渲染經 UI 剝離後)維持乾淨,新檔寫 `[P]` 則等同無效前綴、被靜默剝掉。
　替代案:完全移除解析——舊檔字面前綴滲進描述,rejected;保留欄位僅停用——欄位本身就是假承諾的載體,rejected。

**D2 wire 欄位移除與版本偏斜。** instructions/query 任務 payload 的 parallel 欄位、server task_entry 與 CLI remote 回程轉接的搬運全數刪除;fixture(protocol roundtrip、remote_read_path、typed_client、manual_task_gates)同步。版本偏斜姿態與 manual 加欄同一條:新 client 讀舊 server 的多餘欄位由 serde 靜默忽略;舊 client 讀新 server 的缺欄位落 loud error——隨版本一起出貨、不做相容層。verb-contract 的形狀凍結契約以 delta 更新基線(移除 parallel,僅此一欄)。

**D3 UI 端剝離落在 packages/ui 的任務解析。** 行首標記剝離(`[M] `/`[P] `,順序不敏感、各至多一次)加進既有的任務行解析,與引擎剝除邏輯同構;任務項增 manual 旗標供徽章渲染。剝離只影響顯示——勾選與拖放的寫回是行級改寫(僅翻 checkbox 記號),前綴在檔案裡原樣保留,與行尾 stable-ID 註解的既有處理同一姿態。
　替代案:改走引擎解析後的結構化 payload——任務分頁的資料源是 tasks.md 原文(worktree 現值語意),換資料源是獨立大改,rejected。

**D4 任務列徽章:描述上方獨立一行、與描述左緣切齊。** 任務列的排版核心是「checkbox 後直接接編號」的縱向對齊欄;徽章獨佔描述正上方那一行,左緣與描述同欄(checkbox 之後),描述左緣不因徽章位移、換行時徽章不動。樣式:✋ 圖示+「手動測試」文字的 chip,配色取語意色票(非 muted 灰階)——滿頁灰階任務文字裡,灰底灰字的 chip 讀不出來。勾完後任務文字劃線、徽章保留不劃線;`[P]` 舊標記剝離後不顯示任何徽章(不留空行)。文案走 i18n 詞條(tw「手動測試」,en 對應詞)。
　此決策取代原「行尾右對齊」版本:2026-08-11 的實測(任務 4.2)顯示行尾灰 chip 在長描述列裡幾乎看不見——右緣掃視欄的假設要求視線先跳到行尾,但讀者的視線起點在描述左緣。改置描述上方後徽章落在視線起點,且與編號欄零衝突。
　替代案:維持行尾僅加深配色(視線起點問題未解,rejected——使用者實測後裁定移位)、徽章橫跨整列最左(比 checkbox 還前面,多一層縮排層級且與勾選欄爭位,rejected)、徽章置編號前或編號後內嵌(破壞編號對齊/打斷閱讀流,原即 rejected)、獨立圖示欄(全列讓位且退化純符號,rejected)。

**D5 卡片待手測章:沿審查標示家族,不動解剖學。** 呈現為行內小章(不增加文字列),與審查/驗證標示同家族樣式與位置慣例,附 tooltip 載明剩餘項數(「待手測·剩 N 項」);浮現條件:codeTotal 大於 0、codeRemaining 為 0 且 remaining 大於 0(此時剩餘任務必為 `[M]`;codeTotal=0 的全手測變更是「寫碼全完成」的空真值,不浮現)。判定收斂於 stage.ts 的 awaitingManualCount 單一入口(與 changeStage 同住階段派生模組),卡片元件只讀結果——品質站審查裁定,消除 JSX 內重複計算與派生規則散落。其他狀態(寫碼未完、全完成)卡片逐位元不變;章與看板欄位派生正交(不影響欄位歸屬)。remote 資料源缺欄位時章缺席——與審查標示的 local-only 行為同構。
　替代案:進度條下獨立 chip 列——觸犯解剖學「變更卡不帶狀態 chip」且增加文字列,rejected。

**D6 desktop 協定清單項加欄。** 變更清單 payload(桌面看板查詢路徑的 Rust 序列化,tauriDataSource 傳遞,packages/ui 渲染——與待重新反映徽章同一資料管道)增 codeTotal/codeComplete/codeRemaining 三欄,命名沿 apply payload;計數取自引擎的任務雙組計數單一入口,與任務錨/守門同源。CLI list --json 不含此三欄(相容釘住句式沿審查狀態欄位條文)。
　替代案:只加布林 awaitingManual——tooltip 的剩餘項數還是要數字,且三欄與 apply payload 同名可減少一套詞彙,rejected。

**D7 起草指引與衍生物。** tasks.instruction.md 與 fork.schema.yaml 的翻譯保留規則「`[P]` markers」改為「`[M]` markers」(規則本身保留——標記 token 不得翻譯的約束對 `[M]` 同樣必要)。asset 內文變動走三連動慣例:MARKER_VERSION 提升、golden 再生、assets.lock 更新——以 render_golden 測試紅綠為準(指紋未涵蓋 schema assets 時三連動自然免除,綠燈即證)。

**D8 詞彙條目。** openspec/LANGUAGE.md 增兩條:「手動測試」(定義:agent 無法代行、由使用者實際操作驗證的任務,tasks.md 以 `[M]` 前綴標記;avoid:人工測試、手工測試、手動驗證(標記語境))與「待手測」(定義:寫碼任務全完成、僅餘手動測試任務未勾的變更狀態;avoid:等待驗收、待人工)。

## Implementation Contract

**Behavior(可觀察行為):**

1. 解析:含「- [ ] [P] 描述」的 tasks.md,引擎 payload 的該任務描述不含 `[P]` 且任務項無 parallel 欄位;`[M]` 行為與前一 change 落地版完全不變(manual 欄位、code 計數)。
2. wire:instructions apply --json 的任務項欄位集合為 id/description/done/manual(parallel 消失);本機與 remote 兩模式同形。
3. 任務列:載入含 `[M]` 任務的 tasks.md,該列於描述正上方獨立一行顯示「✋ 手動測試」徽章、徽章左緣與描述左緣切齊;編號起始欄與無徽章列同位,描述左緣不因徽章位移;描述換多行時徽章仍獨佔上方那一行;徽章配色取語意色票而非 muted 灰階;勾選後文字劃線、徽章保留;含 `[P]` 的舊檔任務列顯示剝離後文字、無徽章行(不留空行);勾選寫回後 tasks.md 該行前綴原樣保留。
4. 卡片:codeTotal>0、codeRemaining=0 且 remaining>0 的變更卡出現「待手測」行內小章(tooltip 含剩餘項數);codeTotal=0(全手測變更)不浮現;判定經 stage.ts 的 awaitingManualCount 單一入口;寫碼未完成或全完成的卡片與現況逐位元一致;remote 模式卡片無此章。
5. 詞彙:LANGUAGE.md 含 D8 兩條目。

**Interface / data shape:**

- Task 結構與 TaskJson/TaskEntry:parallel 欄位移除;packages/ui 任務項型別增 manual 旗標
- desktop 協定變更清單項:增 codeTotal/codeComplete/codeRemaining(camelCase、加欄不改名)
- i18n:packages/ui/src/i18n.tsx 增手動測試/待手測詞條(tw/en)
- 起草指引兩檔的翻譯保留規則點名 `[M]`

**Verification targets:**

- cargo:tasks 解析([P] 剝離不承載、[M] 不變)、payload 形狀(parallel 消失)、fixture 全綠——cargo test -p speclink-core -p speclink-protocol -p speclink-cli -p speclink-remote
- UI:packages/ui 測試——taskList(徽章位於描述上方且左緣切齊、剝離、編號對齊結構、勾選保留前綴)與卡片(待手測章浮現條件三態)——pnpm --filter ui test
- golden:cargo test -p speclink-core --test it 全綠(render_golden 與 assets.lock)
- 全套:workspace 測試與 lint 全綠
