---
topic: 目前的quality的速度（review跟verify）感覺上有點慢，有沒有什麼優化的建議？
slug: quality-gate-speed
status: promoted
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: quality-gate-speedup
---

# Discussion: 目前的quality的速度（review跟verify）感覺上有點慢，有沒有什麼優化的建議？

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者覺得 /speclink-quality（審查＋驗證兩站）每輪太慢，想要優化建議。需求可量測（每輪耗時），未進追問階段，直接列假設。scout 盤點耗時分布：CLI 動詞約 0.03 秒（非瓶頸）；首輪審查兩軸 4–6.5 分、驗證排在審查後再跑 4–6.5 分；修完跑全量 cargo test --workspace 約 40 分；複驗輪窄簡報 1–1.5 分；子代理卡住每次 600 秒；finding 路徑寫錯要到下一輪才報錯、只能 discard 重跑首輪。相關正式規格：quality-skill、review-skill、verify-skill、review-station、verify-station；在途變更（manual-callout-unchecked-only、desktop-swipe-back-finger-lift、server-web-list-pages）皆不碰這些 capability。
Prior discussions: code-review-convergence-boundary, quality-skill-pause-and-ui-polish, change-scoped-test-policy, quality-skill-canonicalization, slow-global-test-suite

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: 品質關卡的時間花在哪裡、哪些能砍
**Position**: 使用者確認四條假設全部成立——慢的主因是每輪全量測試與兩站排隊，CLI 不是瓶頸。
- 決策樹：A 每輪固定成本（A1 測試門、A2 子代理派法）／B 卡住與重來（B1 子代理卡 600 秒、B2 路徑錯要整輪重來）／C 刻意不動（每輪停頓、輪數上限）
- A1 測試門改「照專案寫好的測試政策跑受影響範圍，專案沒寫政策才跑全量」，全量交 CI——與 review-skill／verify-skill「修復迴圈的驗證門」（依 repo 慣例的全量測試指令）衝突，改正式規格；證據 openspec/config.yaml:82 已規定全量只由 CI 守門；每輪省約 40 分；蓋章前 CI 綠的既有順序會擋下範圍外回歸
- A2 審查 Standards 軸、審查 Correctness 軸、驗證檢查（一個子代理跑三維度）三個子代理同一則訊息派出，工單仍先寫審查再寫驗證——與 quality-skill「完整兩站時序」情境的「依序完成」字面衝突，但順序只為蓋章不失效（quality SKILL.md:26）；兩站鎖定範圍各寫各的 snapshot 命名空間（crates/host/speclink-host/src/change_diff.rs:414-417），同時跑無干擾；首輪省 4–6.5 分
- B1 子代理簡報寫法定進兩站技能：小 patch 直接貼、大 patch 給路徑與行號範圍、列函式名、限工具次數、沒查完就直說；複驗輪限得嚴、首輪放寬——正式規格未提
- B2 add-round 寫入時擋 finding 路徑不在範圍內或帶 :行號——現行 crates/engine/speclink-core/src/quality/station.rs:240-293 只驗 Phase／Patch 格式與輪次順序；正式規格未提；要改引擎、成本高於前三項
**Ruled out**: 拿掉每輪停頓——quality-skill-pause-and-ui-polish 已裁定使用者要握每輪決定權；設輪數上限——code-review-convergence-boundary 已否決，會截斷仍在改善的修正；子代理改用較小模型——複驗輪本來就只要 1–1.5 分，省得少還可能漏問題
**Open**: 四項怎麼切成變更（一刀或分刀）；B2 的比對基準（凍結快照在 host 端，remote 模式的 add-round 在 server 端看不到）；B1 首輪與複驗輪的工具次數上限數字；驗證站一個子代理跑三維度要不要也拆開平行

### Round 2 — interview (2026-10-14)

**Focus**: 四項要做成一個變更還是拆成兩個
**Position**: 使用者選一個變更，四項（A1 測試門、A2 三子代理同派、B1 簡報寫法、B2 路徑檢查）一起落地。
- 理由：拆成兩個就要跑兩次品質關卡，與本討論要省的時間相反
- B2 的比對基準定為「工單所有輪次 `**Scope**:` 的聯集加上本輪 Scope」，並拒絕帶 `:行號` 的路徑——引擎已有 `scope_union`（crates/engine/speclink-core/src/quality/station.rs:455，蓋章指紋範圍在 :531 用它），只改引擎核心、不讀 host 端快照，remote 模式 server 端的 add-round 也能檢查；續輪沿用的舊 finding 落在早輪 Scope 內，聯集涵蓋得到
- 澄清：A2 只改 /speclink-quality；單獨跑 /speclink-review 本來就兩軸同派、單獨跑 /speclink-verify 維持一個子代理；A1、B1 改在兩站技能檔，單站執行也生效；寫工單、每輪停頓、蓋章順序（先審查後驗證）都不變
**Ruled out**: 拆成「技能三項」與「引擎檢查」兩個變更——要多跑一次品質關卡，換來的只是審查範圍較小；B2 比對 host 端凍結快照——remote 模式的 add-round 在 server 端讀不到 host 本機的快照
**Open**: A1 的「受影響範圍」從哪裡判定（fallback 順序）；B1 首輪與複驗輪的工具次數上限數字；驗證站一個子代理跑三維度要不要也拆開平行

### Round 3 — interview (2026-10-14)

**Focus**: A1 修完後的「受影響範圍」怎麼判定
**Position**: 使用者同意三段判斷順序：先看變更任務寫的驗證指令，再看專案寫好的測試政策，都沒有才跑全量。
- 第 1 段：修補改到的檔案屬於哪個任務的範圍，就跑該任務說明寫的驗證指令——`speclink instructions apply` 的 `tasks` 已帶每個任務的說明，本專案的任務照 openspec/config.yaml:82 寫了 `cargo test -p <package>` 等指令
- 第 2 段：任務沒涵蓋的檔案，照專案寫好的測試政策（CLAUDE.md／AGENTS.md／workflow config 的產出規則）
- 第 3 段：專案什麼都沒寫就跑全量，與現行相同——該專案不變快、也不變差
- 固定規則：建置照跑不跳過（編譯錯會讓下一輪子代理白看）；停頓報告寫明跑了哪些、略過哪些，略過的交 CI
**Open**: 修補若要動到 artifact（規格、design、tasks）要不要先走 /speclink-ingest、怎麼不拉長時間（使用者本輪新提）；B1 首輪與複驗輪的工具次數上限數字；驗證站一個子代理跑三維度要不要也拆開平行

### Round 4 — interview (2026-10-14)

**Focus**: 修補動到 artifact 時要不要先走 /speclink-ingest
**Position**: 不走完整 ingest，依修補大小分三種情況處理，作為同一變更的第五項；使用者同意。
- 不走 ingest 的理由：ingest 為「做到一半、需求有新來源」設計，含自我檢查、analyze、依賴重判、已完成任務重檢，對品質關卡常見的單句規格修正太重；且 ingest 可能新增任務或退回已勾任務，寫碼任務一旦未全完成，驗證站 add-round 被引擎拒絕（crates/engine/speclink-core/src/quality/station.rs:252-260）、審查站開頭守門即停（review SKILL.md:34），品質迴圈會卡死
- 情況 1 改程式碼符合規格：不碰 artifact
- 情況 2 規格字面要修、意圖不變：修補步驟由主對話直接改 artifact，接著跑 `speclink validate`，停頓報告列出改了哪幾段
- 情況 3 意圖或範圍改變：走兩站既有的「範圍改變」出口——該站不蓋章結束，先 ingest 再重跑品質關卡
- 2 與 3 的界線：修補需不需要新增任務，或改變某條 Requirement 的意圖
- 第 3 步停頓時，會動到 artifact 的 finding 先單獨標出讓使用者決定
- 情況 2 改過的 artifact 段落放進下一輪驗證站簡報——鎖定差異排除 openspec/（crates/host/speclink-host/src/change_diff.rs:248、:267），不放進簡報複驗看不到修正，該 finding 會被判未修好、白跑一輪
**Ruled out**: 品質迴圈內一律走完整 /speclink-ingest——重且可能退回任務讓兩站守門拒收
**Open**: session 在修補中途被關閉時，工單與修補進度怎麼接續（使用者本輪新提）；B1 首輪與複驗輪的工具次數上限數字；驗證站一個子代理跑三維度要不要也拆開平行

### Round 5 — interview (2026-10-14)

**Focus**: session 在修補中途被關閉時怎麼接續
**Position**: 在 quality 技能加一段「中斷後接續」作為同一變更的第六項，不改引擎、不加檔案格式；使用者同意。
- 已有的備援：工單在變更資料夾、鎖定快照在 `.speclink/review-scopes/<change>/` 兩站各一資料夾；新 session 的 scope 看到工單仍有必修項目即自動判為複驗，以目前檔案比對上次快照（crates/host/speclink-host/src/change_diff.rs:678 起），關閉前的修補都會被抓到；哪些修好由複驗子代理逐條重算
- 接續步驟：再跑一次 /speclink-quality → 先跑 A1 測試門（前一個 session 的測試狀態未知）→ 複驗差異為空（什麼都還沒修）時不記這一輪、直接回第 3 步拿上一輪 findings 問使用者 → 使用者上一輪的修／不修決定不保存，複驗後停頓時對剩下的項目再答一次
- 補的缺口：空差異照跑一輪會被判「沒有進展 → 未通過」白記一輪失敗；修到一半的程式碼未經測試門就派子代理
- 注意事項寫進技能：關閉後、接續前不要 commit——複驗範圍外新增的檔案靠 `git status` 的 dirty 判定進入複驗（change_diff.rs 約 768-795），commit 後會悄悄掉出範圍
**Ruled out**: 新增動詞把使用者的修／接受決定存進工單或 sidecar——要改引擎、工單格式、remote 與 desktop，省下的只是重答一題
**Open**: B1 首輪與複驗輪的工具次數上限與貼入簡報的 patch 大小門檻；驗證站一個子代理跑三維度要不要也拆開平行

### Round 6 — interview (2026-10-14)

**Focus**: B1 子代理簡報的上限數字
**Position**: 首輪不設工具呼叫次數上限，改為禁止兩種會卡住的查法；複驗輪上限 5 次；使用者同意。
- 改口原因：使用者質疑 15 次太少——最近 25 個封存變更的 delta 規格平均 7.8 條 Requirement、45 個 Scenario（最大 54 條、200 個），驗證站逐條查 15 次查不完；9/17 的 15 次只是一個中型變更的數據
- 卡住的真因是一次讀幾萬字的大檔與沒有目標的遞迴搜尋，不是次數——600 秒中止看的是多久沒有進度，連續小查詢都算進度
- 首輪：不設次數上限；禁止整份讀大檔（用行號範圍）、禁止沒有指定目標的遞迴搜尋（只搜簡報列出的函式名與檔案）
- 複驗輪：上限 5 次——複驗本來就只看修補，上限同時防止變成重新盤點
- 兩輪共通：差異 25 KB 以內直接貼進簡報；自動產生的檔案（golden、重新產生的 SKILL.md）只給路徑、最多搜尋一次；簡報結尾「沒查完就直說，不要猜」
- 澄清：輪數上限仍否決（code-review-convergence-boundary）；工具呼叫上限限制的是單一子代理一次檢查的查詢次數，碰到上限回報未查完而不是中斷修補迴圈
**Ruled out**: 首輪固定 15 次上限——對 Scenario 多的變更驗證站查不完；首輪改用依 Requirement 數縮放的公式——卡住真因不在次數，公式是多餘的彈性
**Open**: 審查要不要再縮到「規格動到的程式碼範圍」、驗證站的查找範圍（使用者本輪新提）；驗證站一個子代理跑三維度要不要也拆開平行

### Round 7 — interview (2026-10-14)

**Focus**: 審查要不要再縮到「規格動到的程式碼範圍」，驗證站的查找範圍
**Position**: 審查維持現狀；驗證站改為先在鎖定差異裡找證據、找不到才往外搜尋，併入 B1；使用者同意。
- 審查現況已只判鎖定差異的段落加上判斷直接影響所需的呼叫端與測試，檔案其餘部分只當背景（review SKILL.md:76）
- 驗證站的矛盾：第 6 步說程式碼證據是鎖定差異加呼叫端／測試（verify SKILL.md:76），第 7 步卻要每條 Requirement「在整個 codebase 搜尋相關關鍵字」（verify SKILL.md:98）
- 新規則：驗證站先在鎖定差異裡找每條 Requirement 的實作證據；差異裡完全找不到時（例如由變更前既有程式碼滿足）才往外搜尋，且只搜具體函式名或檔名
- 澄清：B1 的「禁止整份讀大檔」指子代理為了取得背景一次讀進幾萬字的檔案，不代表審查以整份檔案為審查面
**Ruled out**: 審查只查和規格有關的差異段落——bug 不看規格，順手改的共用函式、測試工具、設定檔會變成沒有任何一站在看；判斷哪段和規格有關本身就要多做一輪
**Open**: 驗證站一個子代理跑三維度要不要也拆開平行

### Round 8 — interview (2026-10-14)

**Focus**: 驗證站一個子代理跑三維度要不要也拆開平行
**Position**: 驗證站維持一個子代理，上線後依實際數據再決定；使用者同意。
- 現況：驗證站只派一個子代理依序做完完整、正確、一致（verify SKILL.md:74）；審查站依軸派兩個（Standards、Correctness）——兩軸標準完全不同，分開才不互相干擾
- 不拆的理由：三子代理同派後驗證站已與審查站重疊，9/17 實測驗證子代理 4–6.5 分回來、與審查兩軸相當；三維度都讀同一份 delta 規格與同一份差異，拆開等於讀三遍；「先查差異」規則已收緊最耗時的整個 codebase 搜尋；三份報告合成一張工單要主對話去重
- 重議條件：上線後驗證站經常比審查兩軸晚很多才回來（例如超過兩倍時間）就拆，依 capability 拆（一份 delta 規格一個子代理），不依維度
- 澄清：兩站看同一份差異，但標準與問題不同——審查問寫得好不好、有沒有 bug，artifact 只用來理解意圖；驗證問規格答應的事有沒有做到
**Ruled out**: 現在就依維度拆成三個子代理——同樣輸入讀三遍、主對話多一道合併，且沒有驗證站拖慢的數據

### Round 9 — interview (2026-10-14)

**Focus**: worktree 流程下品質關卡的基準遺失與一律「先在 worktree 跑」的指引（使用者結論後新提）
**Position**: 合併時把 Apply 基準重新記到主 tree，作為同一變更的第七項；使用者同意，並要求針對這個機制再優化。
- 根因：evidence（openspec/changes/<name>/.evidence.json）與工單都隨 commit 進主 tree，只有 Apply 基準存在 worktree 的 `.speclink/review-scopes/<change>/baseline.json`、不進 git，worktree 移除即消失；主 tree 補跑時缺的只有那個 commit SHA，即每次被問的 `--base`
- 指引在遷就這件事：apply-worktree-post.md:32「Apply baseline 就在這個 worktree 裡，離開就沒有了」、worktree-merge.md:112 把主 tree 補跑稱為「降級路徑」
- 正確基準在合併當下已知：worktree-merge 先 rebase 再 fast-forward（worktree-merge.md:49-62），合併前主分支 HEAD＝變更第一個 commit 的父 commit；合併技能要求主 tree 乾淨，無其他未提交修改混入；rebase 失敗改走一般 merge 時，從合併前 HEAD 比對到合併節點也恰為變更帶進的內容（含解衝突）
- 引擎：`speclink review prepare` 加 `--base <rev>`，以指定 commit 寫正式（initial）基準，該 commit 必須是 HEAD 的祖先；驗證站共用同一份基準（crates/adapters/speclink-cli/src/verbs/station.rs:196）
- 合併技能：合併前記下主分支 HEAD，合併成功後執行 `speclink review prepare <change> --base <該值>`，拿掉「降級路徑」說法
- 交棒指引依情況建議：沒有手動任務→worktree 內跑品質關卡→蓋章→合併；有未勾手動任務→先合併→主 tree 跑品質關卡→手動測試→封存；手動任務數由 `instructions apply` 的 `remaining - codeRemaining` 取得
**Ruled out**: 把 worktree 的 baseline.json 原樣複製到主 tree——其 base 是 rebase 前的舊分岔點，主分支在那之後若有其他變更改到同檔，比對會混入別人的修改；開工時把基準寫進 commit 的 metadata——同樣在 rebase 後過期
**Open**: 機制的進一步優化：未跑完的品質迴圈跨合併接續（快照一起帶過去）、worktree 路徑蓋章後合併讓章過期、CI 只跑主分支對路徑選擇的影響

### Round 10 — interview (2026-10-14)

**Focus**: worktree 基準機制的進一步優化
**Position**: 三項全部加進同一變更（7a 快照跨合併、7b worktree 路徑先 rebase 再跑、7c 指引提 CI）；使用者同意。
- 7a 合併時把兩站的鎖定快照一起帶到主 tree：快照與基準同在 worktree 的 `.speclink/review-scopes/<change>/`，未跑完就合併會讓主 tree 複驗報「frozen snapshot is missing」（crates/host/speclink-host/src/change_diff.rs:692）只能 discard 重跑首輪；快照存檔案內容與雜湊、不綁資料夾（change_diff.rs:1209-1224），搬過去即可用，第六項的中斷接續因此跨合併也成立
- 7b worktree 路徑在跑品質關卡之前先 rebase 到主分支最新位置，再以 `prepare --base` 把基準改到新分岔點：章記整份檔案的內容指紋，跑站期間主分支改到同檔時合併的 rebase 會讓章過期、兩站整套重跑；先 rebase 後蓋章時的內容即合併後內容，之後合併只是 fast-forward
- 7c 交棒選擇表加「想在蓋章前看到 CI 結果→先合併」：worktree 分支 push 不觸發 CI，只有主分支與 PR 會

### Round 11 — interview (2026-10-14)

**Focus**: 與在途變更 review-rules-config 的先後順序與規格衝突（使用者要求調配）
**Position**: 順序定為 review-rules-config → quality-gate-speedup → custom-review-rules 第 2 刀；本變更現在可以 propose，apply 等 review-rules-config 封存後才開始。
- 事實：review-rules-config（已 propose、0/20 任務）的 review-skill delta 只 MODIFIED「Standards 軸的 smell baseline」「審查結果的裁量分類」，ADDED「專案審查規則的讀取與套用」「接受 CRITICAL 必修項的逐條確認」「審查呈現標明套用的規則」；不改 quality-skill、verify-skill、review-station
- 規格層：本變更的 review-skill delta 只改「修復迴圈的驗證門」「審查流程的技能行為」「審查後的迴圈與收尾」等，SHALL NOT 改 review-rules-config 的兩條 MODIFIED——Requirement 層零重疊；propose 後以 `speclink plan --json` 的 requirementOverlap 確認
- 程式層必撞：兩邊都改 crates/engine/speclink-core/assets/skills/review.md 的同兩段（第 5 步子代理簡報、第 9 步選項 1），review-rules-config 的任務 3.3 把 ASSET_VERSION 由 v1.44.0 升到 v1.45.0 並再生 golden／assets.lock——本變更排在後面、以 v1.45.0 為底升 v1.46.0，衍生物重生不挑邊
- 排後面的理由：(1) review-rules-config 已 propose 且在 plan 第一波；(2) 它刻意不改 quality-skill，語意對齊放在本變更做最自然，反過來要 ingest 回它；(3) 等它封存後才 apply，review.md 兩段零文字衝突、本變更的品質關卡也不會因主分支改到同檔讓章過期；(4) 它的品質關卡跑舊技能的代價小——使用者已要求品質迴圈只跑受影響測試
- 語意對齊兩處（併入本變更）：(a) review-rules-config 要求「任何接受途徑接受的必修項含 CRITICAL 時逐條列出並確認一次」，而 quality-skill 現寫「停下時的使用者裁示即明示授權」——quality 第 3 步的「收尾補蓋」與「挑著修」留下 CRITICAL 時，在停頓處逐條列出原文確認一次，並把這次確認交給審查站當授權，審查站不再問第二次；(b) quality 第 3 步的合併報告保留審查站「標明套用的規則」那一行
- A2 同派時，審查站的準備步驟含 review-rules-config 新增的 `speclink workflow-config show --json` 讀規則，準備完才派子代理
- custom-review-rules 第 2 刀（工單記錄首輪套用的規則）可能改 review-station「審查工單的建立與追加」，與本變更 B2 同一條——第 2 刀排在本變更之後轉出，以本變更封存後的正式規格為底；若兩者同時在途，由 plan 的 requirementOverlap 抓出
- 本變更 propose 收尾以 `speclink change depends quality-gate-speedup --on review-rules-config` 寫入軟依賴
**Ruled out**: 本變更排在 review-rules-config 之前——它已 propose 且明言不改 quality-skill，要它 ingest 語意對齊不划算；兩者平行 apply——review.md 同兩段文字衝突、版號與 golden 對撞，後合併者的章會因主分支改到同檔而過期

## Conclusion

**Decision**: 以一個變更 `quality-gate-speedup` 加速品質關卡，共七項，外加與在途變更 review-rules-config 的語意對齊；順序排在 review-rules-config 之後、custom-review-rules 第 2 刀之前；每輪停頓、輪數不設上限、蓋章順序都不變。
- **A1 修完後的測試門改跑受影響範圍**：兩站技能的驗證門從「全量建置與測試」改為三段判斷——(1) 修補改到的檔案屬於哪個任務的範圍，就跑該任務說明寫的驗證指令（`speclink instructions apply` 的 `tasks` 已帶）；(2) 任務沒涵蓋的檔案照專案寫好的測試政策（CLAUDE.md／AGENTS.md／workflow config 產出規則）；(3) 專案什麼都沒寫才跑全量，與現行相同
  - 建置照跑不跳過；停頓報告寫明跑了哪些、略過哪些，略過的交 CI
  - 改 review-skill「修復迴圈的驗證門」（現寫「依 repo 慣例的全量測試指令」）；verify 的對應字面目前只在技能檔第 13 步，正式規格無對應 Requirement，由 propose 決定補在哪條
- **A2 /speclink-quality 三個子代理同一則訊息派出**：每一輪先做完兩站的準備步驟（審查站、驗證站各自鎖定範圍），再把審查 Standards 軸、審查 Correctness 軸、驗證檢查三個子代理同時派出，回來後先寫審查工單、再寫驗證工單，然後照舊停頓
  - 只改 quality 技能；單獨跑 /speclink-review 本來就兩軸同派、單獨跑 /speclink-verify 維持一個子代理
  - 兩站鎖定範圍各寫各的 snapshot 命名空間（crates/host/speclink-host/src/change_diff.rs:414-417），同時跑無干擾
  - 審查站的準備步驟含 review-rules-config 新增的 `speclink workflow-config show --json` 讀專案審查規則，準備完才派子代理
  - 改 quality-skill「兩站時序的編排行為」——「完整兩站時序」情境的「依序完成」字面要改；先後順序只為蓋章不失效，與檢查無關
- **B1 子代理簡報寫法定進兩站技能**：
  - 首輪不設工具呼叫次數上限；禁止整份讀大檔（用行號範圍）、禁止沒有指定目標的遞迴搜尋（只搜簡報列出的函式名與檔案）
  - 複驗輪工具呼叫上限 5 次
  - 差異 25 KB 以內直接貼進簡報；超過只給路徑與行號範圍；自動產生的檔案（golden、重新產生的 SKILL.md）只給路徑、最多搜尋一次
  - 簡報結尾一律加「沒查完就直說，不要猜」
  - 驗證站先在鎖定差異裡找每條 Requirement 的實作證據；差異裡完全找不到時（例如由變更前既有程式碼滿足）才往外搜尋，且只搜具體函式名或檔名——消除 verify 技能第 6 步（差異為證據）與第 7 步（每條 Requirement 搜整個 codebase）的矛盾
  - 審查範圍維持現狀：只判鎖定差異加判斷直接影響所需的呼叫端與測試
- **B2 add-round 寫入時檢查 finding 路徑**：每條 finding 的路徑必須落在「工單既有各輪 `**Scope**:` 的聯集加上本輪 Scope」之內，且不得帶 `:行號`；違者非零拒絕、工單零寫入
  - 比對基準用引擎既有的 `scope_union`（crates/engine/speclink-core/src/quality/station.rs:455，蓋章指紋範圍在 :531 已用它），只改引擎核心、不讀 host 端快照，remote 模式 server 端的 add-round 也能檢查
  - 改 review-station「審查工單的建立與追加」與 verify-station「驗證工單的建立與追加」
- **C1 修補動到 artifact 的三種情況**（寫進兩站與 quality 技能）：
  - 情況 1 改程式碼符合規格：不碰 artifact
  - 情況 2 規格字面要修、意圖不變：修補步驟由主對話直接改 artifact，接著跑 `speclink validate`，停頓報告列出改了哪幾段；改過的 artifact 段落放進下一輪驗證站簡報（鎖定差異排除 openspec/，change_diff.rs:248、:267，不放進簡報複驗看不到）
  - 情況 3 意圖或範圍改變：走兩站既有的「範圍改變」出口，該站不蓋章結束，先 /speclink-ingest 再重跑品質關卡
  - 2 與 3 的界線：修補需不需要新增任務，或改變某條 Requirement 的意圖
  - quality 第 3 步停頓時，會動到 artifact 的 finding 先單獨標出讓使用者決定
- **C2 session 中途關閉後的接續**（寫進 quality 技能）：
  - 再跑一次 /speclink-quality 即可——工單在變更資料夾、鎖定快照在 `.speclink/review-scopes/<change>/`，scope 看到工單仍有必修項目即自動判為複驗，關閉前的修補都會被抓到
  - 派子代理前先跑 A1 測試門（前一個 session 的測試狀態未知）
  - 複驗差異為空（什麼都還沒修）時不記這一輪，直接回第 3 步拿上一輪 findings 問使用者——照跑會被判「沒有進展 → 未通過」
  - 使用者上一輪的修／不修決定不保存，複驗後停頓時對剩下的項目再答一次
  - 關閉後、接續前不要 commit：複驗範圍外新增的檔案靠 `git status` 的 dirty 判定進入複驗（change_diff.rs 約 768-795），commit 後會悄悄掉出範圍
- **D1 worktree 流程的 Apply 基準跨合併存活**：
  - 根因：evidence（openspec/changes/<name>/.evidence.json）與工單隨 commit 進主 tree，只有 Apply 基準存在 worktree 的 `.speclink/review-scopes/<change>/baseline.json`、不進 git，worktree 移除即消失；主 tree 補跑時每次被問的 `--base` 就是在補這個值
  - 引擎：`speclink review prepare` 加 `--base <rev>`，以指定 commit 寫正式（initial）基準；該 commit 必須是 HEAD 的祖先；任一站有未結工單時拒絕改寫基準（快照綁舊基準）；驗證站共用同一份基準（crates/adapters/speclink-cli/src/verbs/station.rs:196）
  - 合併技能：合併前記下主分支 HEAD（worktree-merge 先 rebase 再 fast-forward，worktree-merge.md:49-62，此值＝變更第一個 commit 的父 commit；主 tree 必須乾淨，無其他未提交修改混入；rebase 失敗改走一般 merge 時，從此值比對到合併節點也恰為變更帶進的內容），合併成功後以 `prepare --base <該值>` 在主 tree 寫入基準；拿掉 worktree-merge.md:112「降級路徑」與 apply-worktree-post.md:32「離開就沒有了」的說法
  - 7a 合併時把兩站的鎖定快照一起帶到主 tree：未跑完就合併時主 tree 複驗不再報「frozen snapshot is missing」（change_diff.rs:692），C2 的接續跨合併也成立；快照存檔案內容與雜湊、不綁資料夾（change_diff.rs:1209-1224）；搬移由引擎動詞負責（例如 prepare 帶來源 worktree 路徑），技能不直接複製 `.speclink` 內部檔案，確切旗標由 design 定
  - 7b worktree 路徑在跑品質關卡之前先 rebase 到主分支最新位置，再以 `prepare --base` 把基準改到新分岔點——蓋章時的內容即合併後內容，之後合併只是 fast-forward、章不過期；rebase 有衝突時照合併技能的規則不自行解，abort 後回報由使用者決定（可改走先合併的路徑）；已有未結工單時不在迴圈中途 rebase
  - 交棒指引（apply-worktree-post 的 W3）改為依情況建議，不再一律叫使用者在 worktree 跑：沒有未勾手動任務→worktree 內 rebase、跑品質關卡、蓋章、合併；有未勾手動任務，或 7c 想在蓋章前看到 CI 結果（worktree 分支 push 不觸發 CI，只有主分支與 PR 會）→先合併、主 tree 跑品質關卡、手動測試、封存；手動任務數由 `instructions apply` 的 `remaining - codeRemaining` 取得
- **E1 與 review-rules-config 的語意對齊**（它刻意不改 quality-skill，對齊由本變更承擔）：
  - review-rules-config 新增「接受 CRITICAL 必修項的逐條確認」（任何接受途徑接受的必修項含 CRITICAL 時逐條列出原文、確認一次），而 quality-skill 現寫「停下時的使用者裁示即明示授權」——quality 第 3 步的「收尾補蓋」與「挑著修」留下 CRITICAL 時，在停頓處逐條列出原文確認一次，並把這次確認交給審查站當授權，審查站不再問第二次
  - quality 第 3 步的合併報告保留審查站「審查呈現標明套用的規則」那一行（預設清單模式與三個清單的條數）
- **順序與規格調配**：
  - 順序：review-rules-config → quality-gate-speedup → custom-review-rules 第 2 刀；本變更現在可以 propose，apply 等 review-rules-config 封存後才開始；propose 收尾以 `speclink change depends quality-gate-speedup --on review-rules-config` 寫入軟依賴
  - 規格層零重疊：review-rules-config 的 review-skill delta 只 MODIFIED「Standards 軸的 smell baseline」「審查結果的裁量分類」並 ADDED 三條，不改 quality-skill、verify-skill、review-station；本變更 SHALL NOT 改那兩條 MODIFIED，只改「修復迴圈的驗證門」「審查流程的技能行為」「審查後的迴圈與收尾」等；propose 後以 `speclink plan --json` 的 requirementOverlap 確認無衝突
  - 程式層必撞、靠先後消除：兩邊都改 crates/engine/speclink-core/assets/skills/review.md 的第 5 步子代理簡報與第 9 步選項 1；review-rules-config 的任務 3.3 把 ASSET_VERSION 由 v1.44.0 升到 v1.45.0 並再生 golden／assets.lock——本變更以 v1.45.0 為底升 v1.46.0，衍生物重生不挑邊；等它封存後才 apply，兩段零文字衝突，本變更的章也不會因主分支改到同檔而過期
  - custom-review-rules 第 2 刀（工單記錄首輪套用的規則）可能改 review-station「審查工單的建立與追加」，與 B2 同一條——第 2 刀排在本變更之後轉出、以本變更封存後的正式規格為底；若兩者同時在途，由 plan 的 requirementOverlap 抓出
- 落地範圍：技能資產 crates/engine/speclink-core/assets/skills/{quality,review,verify,apply-worktree-post,worktree-merge}.md（走 ASSET_VERSION／golden／assets.lock 三連動與 speclink update 再生）；引擎 crates/engine/speclink-core/src/quality/station.rs、crates/host/speclink-host/src/change_diff.rs、crates/adapters/speclink-cli/src/verbs/station.rs；正式規格 quality-skill、review-skill、verify-skill、review-station、verify-station、change-diff-scope、worktree-merge-skill、worktree-apply-skill、skill-routing
- 例：一個中型變更的首輪，原本審查兩軸 4–6.5 分回來後才派驗證、再等 4–6.5 分，合計約 8–13 分；改後三個子代理同時派，約 4–6.5 分全部回來。修完一輪只動到 speclink-core，原本跑 `cargo test --workspace` 約 40 分；改後跑任務寫的 `cargo test -p speclink-core` 加秒級 node 守門，停頓報告列出略過 speclink-desktop 等未受影響的 package、交 CI 守門。帶 `[M]` 任務的 worktree 變更合併後，在主 tree 跑 /speclink-quality 直接進入首輪，不再被問 `--base`
**Rationale**: 時間主要耗在每輪約 40 分的全量測試與兩站排隊的子代理，CLI 動詞只要約 0.03 秒；全量測試早已由 CI 守門（openspec/config.yaml:82），而蓋章前 CI 綠的既有順序會擋下範圍外回歸，所以本機每輪付的全量成本沒有換到額外保障；兩站排隊只為蓋章順序，檢查本身彼此獨立。其餘各項處理會讓整輪白跑或重來的情況（子代理卡 600 秒、路徑錯到下一輪才炸、修補動到 artifact 時誤走重流程、session 中斷後空跑一輪、合併後基準與快照遺失、合併讓章過期），能在源頭消掉的就在源頭消掉——基準在合併當下就已知，記下來比事後每次問使用者準確也省事。排在 review-rules-config 之後，是因為兩者規格不重疊、只在 review.md 與版號上相撞，依序做就能讓衝突消失，而語意對齊放在本變更最省事。
**Rejected alternatives**:
- 拿掉每輪停頓 — quality-skill-pause-and-ui-polish 已裁定使用者要握每輪決定權
- 設輪數上限 — code-review-convergence-boundary 已否決，會截斷仍在改善的修正
- 子代理改用較小模型 — 複驗輪本來就只要 1–1.5 分，省得少還可能漏問題
- 拆成多個變更 — 每多一個變更就要多跑一次品質關卡，換來的只是審查範圍較小
- B2 比對 host 端凍結快照 — remote 模式的 add-round 在 server 端讀不到 host 本機的快照
- 品質迴圈內一律走完整 /speclink-ingest — 太重，且可能新增或退回任務，讓兩站的寫碼任務守門拒收、迴圈卡死
- 新增動詞把使用者的修／接受決定存起來 — 要改引擎、工單格式、remote 與 desktop，省下的只是重答一題
- 首輪固定 15 次工具呼叫上限 — 最近 25 個封存變更平均 45 個 Scenario，驗證站查不完
- 首輪依 Requirement 數縮放上限的公式 — 卡住真因是讀大檔與無目標搜尋，不是次數，公式是多餘的彈性
- 審查只查和規格有關的差異段落 — bug 不看規格，順手改的共用函式、測試工具、設定檔會變成沒有任何一站在看
- 現在就把驗證站依維度拆成三個子代理 — 同樣輸入讀三遍、主對話多一道合併，且沒有驗證站拖慢的數據
- 把 worktree 的 baseline.json 原樣複製到主 tree — 其 base 是 rebase 前的舊分岔點，主分支在那之後若有其他變更改到同檔，比對會混入別人的修改
- 開工時把基準寫進會 commit 的 metadata — 同樣在 rebase 後過期
- 主 tree 補跑時由技能自動推算 `--base`（找變更第一個 commit 的父 commit）— 哪些 commit 屬於變更要靠判斷，合併當下記下的值才是確定的
- 本變更排在 review-rules-config 之前 — 它已 propose 且明言不改 quality-skill，要它 ingest 語意對齊不划算；它跑舊技能的代價也小，品質迴圈已實際只跑受影響測試
- 與 review-rules-config 平行 apply — review.md 同兩段文字衝突、版號與 golden 對撞，後合併者的章會因主分支改到同檔而過期
**Deferred**:
- 驗證站是否依 capability 拆成多個子代理 — 等本變更上線後的實際數據；驗證站經常比審查兩軸晚超過約兩倍時間才回來時重議
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion quality-gate-speed（現在即可；收尾寫入對 review-rules-config 的軟依賴，apply 等它封存後開始）
