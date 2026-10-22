# quality-skill Specification

## Purpose

/speclink-quality 技能的內容與行為：技能檔的生成與正典化，以及審查與驗證兩站的時序編排。本 capability 保證技能只承載時序、不重述兩站各自的檢查與蓋章語意——兩站先都跑完檢查而不蓋章，findings 一起回報後每輪暫停，修什麼、何時蓋章由使用者決定。

## Requirements

### Requirement: 品質關卡技能的生成與正典化

`speclink update` SHALL 生成 `/speclink-quality` 技能檔至已啟用工具（claude／codex）的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 使生成之 CLAUDE.md／AGENTS.md 的 workflow 行含 quality 入口，並於技能使用清單加入 quality 條目，敘明觸發時機：事前已知 review 與 verify 兩站都要跑時使用；只跑一站直接呼叫該站技能。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** 已啟用工具的技能目錄出現 speclink-quality 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行與技能清單含 quality 條目

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 quality 入口，技能使用清單含 quality 條目與其觸發時機（兩站都跑時使用；單站直接呼叫該站技能）


<!-- @trace
source: quality-skill-canonicalization
updated: 2026-08-07
-->

---
### Requirement: 兩站時序的編排行為

技能 SHALL 只承載兩站時序與子代理的派出時機;唯一的例外是「蓋章前的預防提醒」要求的第一輪前主分支比對(含它帶來的停頓、rebase 後的 commit 確認與起點重設)與收尾補蓋選項的 CI 附註,兩者以該需求為準。技能 SHALL NOT 重述兩站的檢查內容、簡報規則、工單與蓋章語意(由 review/verify 技能正典各自承載)。前提為 change 寫碼任務全數完成(非 `[M]` 任務全數勾選;僅餘 `[M]` 手動測試任務未勾時前提成立,技能於報告點名尚餘手測);寫碼任務未完成時 SHALL 依兩站既有前提行為呈現(拒絕或中途盤點,依該站正典),技能不另設守門。

時序 SHALL 為每輪暫停制:

- 檢查輪:技能 SHALL 先依序完成兩站的準備步驟——審查站的選定 change、守門、查工單、鎖定範圍、讀 artifact 與專案審查規則;驗證站的選定 change、讀狀態、讀 artifact、分流與鎖定範圍——再於同一則訊息派出審查 Standards 軸、審查 Correctness 軸與驗證檢查三個唯讀子代理,三份簡報各依該站正典組成。三者回報後依序完成審查站(呈現、分類、寫入工單)與驗證站(報告、分類、寫入工單)的收尾,兩站皆以「先不蓋章」離場 → 彙整兩站 findings 向使用者報告並停下詢問下一步;SHALL NOT 未經使用者裁示即開始修正。harness 無法同時派出三個子代理時,SHALL 依兩站既有的「無法派子代理即依序自跑」退路執行。
- 彙整報告 SHALL 依站分組、必修與其餘分開,SHALL 保留審查站呈現開頭標明套用規則的那一行,並 SHALL 把修正會動到 artifact(規格、design、tasks)的 finding 單獨標出,讓使用者先決定。
- 停下時提供的選項 SHALL 至少涵蓋:全修、挑選部分修正、不修就停(兩站工單與凍結快照留存、不蓋章離場)。必修尚未淨空時 SHALL NOT 提供補蓋選項——站內「必修淨空才蓋章」的正典不被繞過。
- 修正輪:使用者裁示的修正統一落地(一律回主線、依專案 TDD 慣例)並通過兩站的修復迴圈驗證門後,以與檢查輪相同的「先準備、同訊息派出」方式重跑 review 複驗與 verify 複驗;每輪兩站複驗完成後 SHALL 再次停下詢問,直到使用者裁示收尾或停止。
- 乾淨輪:兩站皆零 findings,或必修淨空且該輪零新修正時,SHALL 同樣停下報告兩站皆綠,由使用者決定是否進入收尾補蓋;SHALL NOT 自動補蓋。
- 收尾:使用者裁示補蓋後,review 章與 verify 章接連補蓋;補蓋時技能 SHALL 向兩站明示本次為收尾補蓋呼叫——兩站據此關閉其禁蓋例外;未宣告即補蓋不合本規格。票內仍留有使用者裁示不修的必修級 findings 時,該站之章為帶保留章——技能 SHALL 於補蓋選項中預先載明,並以停下時的使用者裁示為明示授權;SHALL NOT 未經明示逕自蓋保留章。使用者於「進入收尾補蓋」或「挑選部分修正」留下的必修項含 CRITICAL 級時,技能 SHALL 於停下處逐條列出這些 CRITICAL 的原文並取得使用者明確確認一次,之後呼叫審查站時 SHALL 明示「CRITICAL 已逐條確認」——審查站據此不再詢問(見 review-skill「quality 時序轉交的 CRITICAL 確認」)。僅 SUGGESTION 級殘留 SHALL NOT 構成保留——該站收尾補蓋落乾淨章,無需保留授權。兩章 SHALL 接連落、中間零編輯,使兩章至封存皆不出現「其後有變動」狀態。封存 SHALL 以建議形式提出、由使用者執行;change 尚餘 `[M]` 任務未勾時,封存建議 SHALL 提醒手動測試完成並勾選後才可封存。

#### Scenario: 完整兩站時序

- **WHEN** 使用者對寫碼任務全數完成的 change 要求兩站都跑(或執行 /speclink-quality)
- **THEN** 技能先完成兩站的準備步驟,於同一則訊息派出三個子代理,回報後依序寫入審查工單與驗證工單並皆先不蓋章,彙整兩站 findings 停下待裁示;裁示的修正統一落地後兩站複驗、每輪複驗完成再停;必修淨空且零新修正的輪停下報告兩站皆綠,使用者裁示後以明示的收尾補蓋呼叫讓兩章接連補蓋,且到封存皆維持已完成狀態、無「其後有變動」

#### Scenario: 檢查輪後暫停等待裁示

- **WHEN** 兩站檢查完成且合計揪出至少一筆 finding
- **THEN** 技能彙整兩站 findings 停下詢問(全修/挑選部分修正/不修就停),使用者裁示前不落任何修正編輯

#### Scenario: 複驗引出新修正時延後補蓋

- **WHEN** 統一修正後某站複驗仍揪出必修
- **THEN** 此時兩章皆尚未蓋下;技能停下回報並待使用者裁示,裁示的修正落地後重跑兩站複驗,迴圈至一輪完整複驗零新修正且使用者裁示補蓋,才接連補蓋兩章,全程不出現已蓋章又被後續修正打黃的狀態

#### Scenario: 乾淨輪停下由使用者決定補蓋

- **WHEN** 某輪兩站皆零 findings,或必修淨空且該輪零新修正
- **THEN** 技能報告兩站皆綠並停下;使用者裁示後才以明示的收尾補蓋呼叫接連補蓋兩章、中間零編輯,使用者未裁示前不蓋任何章

#### Scenario: 僅 SUGGESTION 殘留的收尾補蓋落乾淨章

- **WHEN** 使用者裁示補蓋時某站票內僅殘留 SUGGESTION 級 findings、無使用者裁示不修的必修
- **THEN** 該站之章為乾淨章(無該站的 --accept),補蓋選項無需載明保留

#### Scenario: 使用者選擇不修就停

- **WHEN** 檢查輪或複驗輪停下時使用者選擇不修就停
- **THEN** 技能以兩站「先不蓋章」出口離場收尾,工單與凍結快照留存,不蓋任何章、不自行封存

#### Scenario: 事後變卦加跑第二站

- **WHEN** 一站已蓋章後使用者才要求加跑另一站
- **THEN** 技能 SHALL NOT 重做已蓋章的站,照跑新站;新站各輪零碼改動時前章維持有效、封存照行;新站的修正落地時前章即失效,封存的章失效守門拒絕至前章之站重跑補章(見 change-lifecycle「封存的章失效守門」)——重跑即「蓋章後再修碼」的代價,兩站已知都要跑時走本技能的雙章保留時序即可避免

#### Scenario: 單站請求不經編排

- **WHEN** 使用者只要求跑單一品質關卡
- **THEN** 直接呼叫該站技能,quality 不啟動,該站維持修完即蓋章的預設行為與既有的子代理派法(審查站兩軸同派、驗證站一個子代理)

#### Scenario: 前提未完成時依站內前提行為呈現

- **WHEN** change 尚有未完成寫碼任務即啟動兩站時序
- **THEN** 依兩站既有前提行為呈現(review 站拒絕並停止;verify 站以中途盤點報告收場),quality 不另設守門、不吞錯

#### Scenario: 僅餘手動任務時兩站照跑

- **WHEN** 對寫碼任務全勾、僅餘 `[M]` 任務未勾的 change 執行 /speclink-quality
- **THEN** 兩站時序照常啟動,報告點名尚餘的手動測試任務;收尾補蓋照常可行,封存建議提醒手測完成後才可封存

#### Scenario: 三個子代理同一則訊息派出

- **WHEN** 檢查輪或修正輪的兩站準備步驟都完成,且兩站的鎖定範圍皆為 resolved
- **THEN** 技能於同一則訊息派出審查 Standards 軸、審查 Correctness 軸與驗證檢查三個子代理,SHALL NOT 等審查站寫完工單才派驗證站;三者回報後工單仍先寫審查、再寫驗證

#### Scenario: harness 無法同時派出時依序自跑

- **WHEN** 執行環境無法派出子代理
- **THEN** 技能依兩站既有退路依序自行執行三份檢查並各自分開報告,其餘時序(先不蓋章、彙整後停下)不變

#### Scenario: 停下時標出會動到 artifact 的 finding

- **WHEN** 某輪 findings 中有一筆的修正需改規格的 Scenario 文字
- **THEN** 彙整報告把該筆單獨列為「會動到 artifact」,與只改程式碼的 finding 分開呈現,再提供停下選項

#### Scenario: 留下 CRITICAL 時於停頓處逐條確認並轉交

- **WHEN** 使用者選擇「進入收尾補蓋」,而審查工單仍留有一筆使用者裁示不修的 CRITICAL finding
- **THEN** 技能先逐條列出該 CRITICAL 原文並取得使用者明確確認一次;補蓋呼叫審查站時明示「CRITICAL 已逐條確認」,審查站以帶保留章蓋下且不再詢問


<!-- @trace
source: stamp-order-rules
updated: 2026-10-22T22:36:29+08:00
-->

---
### Requirement: 中斷後接續

session 於品質關卡的修正途中被關閉時,使用者 SHALL 能以再次執行 /speclink-quality 接續:兩站工單在變更資料夾、鎖定快照在 host 本機的 `.speclink/review-scopes/<change>/`,兩站的鎖定範圍看到工單仍有必修項目時即判為 validation,關閉前落地的修正都在驗證 patch 內。接續時技能 SHALL 另守三條:

- 派出子代理之前 SHALL 先通過兩站的修復迴圈驗證門——前一個 session 的測試狀態未知。
- 某站的鎖定範圍為 validation 且驗證 patch 無任何檔案、該站末輪仍有必修項目時,技能 SHALL 先列出變更資料夾內未提交與未追蹤的檔案(兩站工單不計)——驗證 patch 不含規格目錄,只改 artifact 的修正只能從簡報判定,而 git 分不出這些修改發生在上一輪之前或之後。沒有列出任何檔案時,該站本輪 SHALL NOT 派出子代理、SHALL NOT 寫入新輪;兩站皆如此時技能 SHALL 直接回到停下詢問,以末輪 findings 讓使用者裁示——照常記一輪會被判「沒有進展」而以 failed 結束。有列出檔案時,技能 SHALL 把清單交給使用者,詢問前一個 session 是否以修改這些 artifact 修正 findings:回答是,該站本輪照常派出複驗子代理,簡報 SHALL 附上改過的段落原文;回答否,依「沒有列出任何檔案」處理。
- 技能 SHALL 提醒使用者:關閉後、接續前不得 commit 修正——驗證 patch 以內容移動與 dirty 判定收錄修正,提交後未被快照收錄的新檔會離開驗證面。

使用者前一輪的修／不修裁示不保存;接續後的停下詢問 SHALL 只列仍未解的 findings 讓使用者再裁示一次。

#### Scenario: 關閉後重跑即接續複驗

- **WHEN** 第 1 輪有三筆必修,使用者修好其中兩筆後 session 被關閉,新 session 執行 /speclink-quality
- **THEN** 技能先跑驗證門,兩站以 validation 鎖定含兩筆修正的驗證 patch,子代理判定兩筆已解、一筆未解,停下詢問時只列那一筆

#### Scenario: 複驗差異為空的站不記一輪

- **WHEN** session 在任何修正落地前被關閉,新 session 執行 /speclink-quality,兩站的驗證 patch 皆無檔案,變更資料夾也沒有未提交或未追蹤的檔案
- **THEN** 技能不派子代理、兩站工單皆不新增輪次,直接以末輪 findings 停下詢問,工單位元級不變

#### Scenario: 只改 artifact 的修正於接續後照常複驗

- **WHEN** 第 1 輪的必修項以修改 delta spec 字面修正後 session 被關閉,新 session 執行 /speclink-quality,兩站的驗證 patch 皆無檔案,但變更資料夾的 delta spec 有未提交的修改
- **THEN** 技能列出該 delta spec 並詢問使用者;使用者確認這是前一個 session 的修正後,技能照常派出三個子代理,簡報附上改過的段落原文,兩站各寫入一輪

#### Scenario: 未提交的 artifact 不屬於修正時不記一輪

- **WHEN** 接續時兩站的驗證 patch 皆無檔案,變更資料夾只有 apply 期間留下、未提交的 tasks.md 勾選,使用者回答前一個 session 沒有修改 artifact
- **THEN** 技能不派子代理、兩站工單皆不新增輪次,直接以末輪 findings 停下詢問

#### Scenario: 只有一站的差異為空

- **WHEN** 接續時審查站的驗證 patch 有檔案、驗證站的驗證 patch 無檔案且驗證工單末輪仍有必修,變更資料夾也沒有未提交或未追蹤的檔案
- **THEN** 技能只派審查站的兩個子代理並寫入審查工單,驗證工單不新增輪次,彙整報告註明驗證站本輪無修正可判

<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->

---
### Requirement: 蓋章前的預防提醒

quality 技能 SHALL 在兩條會讓章失效的情境上，於事前提醒使用者：

- **跑關卡前比對主分支**：準備兩道關卡、凍結第一輪範圍之前，若目前分支不是 repo 的主分支（例如 worktree 或功能分支），技能 SHALL 以 git 找出主分支在分岔點之後改過的檔案，並與本分支自分岔點以來改過的檔案取交集。交集不為空時，技能 SHALL 停下來列出這些檔案，說明之後再 rebase 會讓章失效、重蓋要從首輪重跑兩道關卡，並建議先 commit 本變更的檔（含新檔）再 rebase（或合併）主分支，等使用者回答後才繼續。使用者 rebase 或合併之後，技能 SHALL 先確認本變更的檔（含新檔）都已 commit——沒 commit 的檔會被記成開工前就有的修改，讓第一輪範圍停下——仍有未 commit 的檔時請使用者 commit，或經使用者同意代為 commit；再以 `speclink review prepare --base <新的分岔點>` 把兩站的起點移到新的分岔點，否則第一輪會把主分支的修改算成本變更的；這個指令失敗時 SHALL 回報錯誤，排除原因後在任何關卡開跑前重跑。交集為空、目前就在主分支上、或找不到主分支時，SHALL 直接繼續，不停頓。
- **收尾蓋章前提醒 CI**：每輪停下來詢問時，若 repo 有 CI 設定（例如 `.github/workflows/` 下有工作流程檔），「進入收尾蓋章」選項的說明 SHALL 附註「建議先 commit 並 push，再確認該 commit 的 CI 已在所有平台通過——蓋章後為了 CI 再修會讓章失效」；技能 SHALL NOT 為此新增問題，也 SHALL NOT 自動 push 或查詢 CI 狀態。

技能 SHALL NOT 自動執行 rebase、合併或 push。

#### Scenario: 主分支改過本變更的檔時先停下

- **WHEN** 在 worktree 分支上對變更執行 `/speclink-quality`，主分支在分岔點之後改過 `crates/engine/speclink-core/src/util.rs`，而本分支也改過這個檔
- **THEN** 技能在凍結第一輪範圍之前停下，列出 `crates/engine/speclink-core/src/util.rs`，建議先 rebase 主分支並說明原因；使用者回答前沒有任何工單或範圍被建立

#### Scenario: 使用者 rebase 後先確認 commit 再重設起點

- **WHEN** 技能停下建議 rebase 後，使用者 rebase 了主分支，而本變更還有一個新檔沒 commit
- **THEN** 技能先請使用者 commit 這個檔（或經使用者同意代為 commit），再以 `speclink review prepare --base <新的分岔點>` 移動兩站的起點，之後才凍結第一輪範圍

#### Scenario: 沒有交集時直接開始

- **WHEN** 在功能分支上執行 `/speclink-quality`，主分支在分岔點之後只改過本分支沒碰的檔
- **THEN** 技能不停頓，直接凍結第一輪範圍並送出檢查

#### Scenario: 在主分支上不比對

- **WHEN** 目前就在主分支上執行 `/speclink-quality`
- **THEN** 技能不做分支比對，直接凍結第一輪範圍

#### Scenario: 有 CI 設定時收尾選項附註

- **WHEN** repo 有 `.github/workflows/ci.yml`，某一輪兩道關卡的必修 finding 皆為零，技能停下來詢問
- **THEN** 「進入收尾蓋章」選項的說明附註先確認 CI 已在所有平台通過的建議，選項數量與沒有 CI 設定時相同

#### Scenario: 沒有 CI 設定時不附註

- **WHEN** repo 沒有任何 CI 設定檔，某一輪兩道關卡的必修 finding 皆為零，技能停下來詢問
- **THEN** 「進入收尾蓋章」選項的說明沒有 CI 附註

<!-- @trace
source: stamp-order-rules
updated: 2026-10-22T22:36:29+08:00
-->