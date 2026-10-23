---
title: 工作流政策與設定
section: SDD 工作流
order: 210
keywords: [config.yaml, 政策, locale, tdd, audit, worktree, 專案說明, 產出規則, 審查規則, review, 確認表, 提醒分流, dry-run, 對話語言]
sources: ["skill-authoring#技能共用的對話語言段落", "workflow-config", "config-skill#技能規定固定輸入來源與四條內容判準", "config-skill#技能規定 diff 先行與收斂驗收", "config-skill#技能規定政策語系欄位寫入代碼", "config-skill#技能規定任務驗證測試範圍的第五問", "config-skill#技能規定審查規則的第六問與規則接收", "config-skill#技能規定政策欄位一次確認現值", "config-skill#技能規定提醒的分流"]
generated: 2026-10-23T14:19:42+08:00
---
# 工作流政策與設定

工作流政策存在 `openspec/config.yaml`。這個檔包含五個既有政策欄位、安全審查政策、專案說明、產出規則，以及審查規則。你可以用 `speclink workflow-config` 指令讀寫，也可以請 `/speclink-config` 技能幫你整理。桌面 app 的設定頁怎麼編輯這些內容，見 [專案、新增專案與設定頁](desktop-projects.md)。

## 五個政策欄位

| 欄位 | 合法值 | 未設定時 |
| --- | --- | --- |
| locale | tw、ja、en | 系統語言 |
| spec_locale | tw、ja、en、auto | 系統語言 |
| tdd | true、false | false |
| audit | true、false | false |
| worktree | true、false | false |

值的比對區分大小寫。TW、Auto 都不合法。顯示名稱（例如「繁體中文」）也不合法，要寫代碼 tw。

`speclink init` 產生的 config.yaml 範本已含這五個欄位的註解示例，並列出對應的環境變數名。`.speclink.yaml` 不放政策鍵。

### locale 管文件，不管對話

locale 決定 AI 產出的文件與報告用哪種語言。技能替自己的產出指定了語言時，就照 locale 寫；例如審查站的回合呈現與提問也照 locale。除此之外，agent 跟你說的話（進度、提問、摘要）都用你跟它說話的語言。

- 技能檔本身是英文寫的，裡面的回報範例只示範格式。agent 會把範例的文字換成你的語言；指令、檔案路徑、程式碼，以及嚴重度標籤、狀態詞這類固定字樣維持原樣。
- agent 修改一份既有文件、而且沒有步驟指定語言時，沿用那份文件原本的語言。
- 技能在看不到對話的獨立情境中執行時，用收到的請求所用的語言。

這段規則每一個技能都有，內容與 locale 的值無關。

### 有效值怎麼決定

有效值依下列順序解析，先命中者勝：

1. 環境變數：SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE。
2. `openspec/config.yaml` 的值。
3. 內建預設。

布林環境變數只接受 true 或 false。其他值視為未設定，落到下一層，不報錯。`.speclink.yaml` 裡的同名鍵一律不生效，也不會有警告。

> [!WARNING]
> `openspec/config.yaml` 存在但無法解析（YAML 語法錯誤或型別不符）時，所有讀政策的指令都會失敗。錯誤訊息指出檔案路徑與解析原因。指令不會退回預設值繼續跑，設環境變數也繞不過。檔案不存在時才用內建預設。review 區塊寫錯也算無法解析，見下方「審查規則」。

## 讀設定：workflow-config show

```
speclink workflow-config show
```

顯示 config.yaml 裡的正典內容：五個政策欄位（未設定的會標示未設定與其預設語意）、專案說明的有無與行數、各產出規則節的條數，以及審查規則與安全審查政策的摘要。安全審查摘要接在審查規則之後；未設定時顯示停用。它顯示的是檔案裡的值，不套用環境變數覆寫。remote 模式下讀的是 server 上的設定，輸出形狀相同。

審查規則那一行接在產出規則那一行之後：

| config.yaml 的 review 區塊 | review 行的值 |
| --- | --- |
| 沒寫 | none |
| critical 兩條、warning 一條、suggestion 一條，沒寫模式 | extend, critical 2, warning 1, suggestion 1 |
| 模式為 replace、三個清單皆空 | replace, critical 0, warning 0, suggestion 0 |

## 寫政策：workflow-config set

```
speclink workflow-config set <key> <value>
```

key 只能是五個政策欄位之一。其他 key 被拒絕，不動檔案。值不合法時指令失敗，錯誤訊息指出欄位名、收到的值與合法代碼集合。

- tdd、audit、worktree 設 false，或 locale、spec_locale 設空字串，會移除該鍵，回到未設定＝預設。鍵上方的註解行保留。
- 寫入只動目標鍵所在的行。其他行逐位元保留，包括註解、空行與你自己加的內容。
- 加 `--dry-run` 只印出前後差異，不寫檔。沒有變更時差異為空。值不合法時同樣拒絕，不印差異。

remote 模式下，指令先讀 server 上的現行設定，套用同樣的改寫再寫回。寫回時發現別人已先改過，指令失敗並提示你重新執行，不會蓋掉別人的寫入。離線或認證失效時指令失敗，不會暫存或排隊。

### worktree 欄位的額外效果

`set worktree true` 寫入成功後會同步技能，範圍同 `speclink update`。各工具的技能目錄多出兩顆 worktree 技能。

`set worktree false` 時，若還有活躍的 worktree 掛著，指令拒絕寫入，列出每個 worktree 的變更名、分支與路徑，並提示先收尾。

設定寫入成功但技能同步失敗時，設定仍成立。錯誤訊息提示重跑 `speclink update` 重建技能。詳見 [平行實作與合回：worktree](worktree.md)。

## OWASP / CWE 初步安全審查政策

舊專案沒有安全設定時，主開關預設關閉，原有 audit、TDD 與 worktree 不受影響。設定入口在桌面 app 的專案設定 → 產出流程 → 產出政策，audit 下方的「啟用 OWASP / CWE 安全審查」。

安全主開關下另有預設未勾選的「納入驗收條件」。勾選後，安全審查結果會影響驗收：CRITICAL／HIGH／MEDIUM 需修復或逐項記錄接受理由，LOW 只列建議；必要檢查未完成時不能通過。儲存這個選項不會自動啟動審查。主開關關閉時，checkbox 暫時停用但保留勾選值，重新啟用後才生效。

產出流程也能分別設定封存時是否保留審查、驗證及安全審查的完整紀錄；三項預設都保留。取消某項只影響之後的封存，不會刪除既有歷史，安全驗收摘要仍會保留。

### 先選範圍，再選標準

| 範圍 | 用途 |
| --- | --- |
| 關閉／不指定範圍 | 不自動限定 Web 或 API；不等於關閉主開關。目標資訊不足時保留未驗證 |
| Web 應用程式 + API | 同時涵蓋 Web 與 API |
| 僅 API / 後端服務 | 著重 API 與後端邊界 |
| 僅 Web 應用程式 | 著重 Web 輸出與瀏覽器端風險 |

| 標準 | 可選版本 |
| --- | --- |
| OWASP Top 10 | 關閉、2021、2025 |
| OWASP API Security Top 10 | 關閉、2019、2023 |
| OWASP ASVS | 關閉、4.0.3、5.0.0；啟用後選 L1、L2 或 L3 |
| MITRE CWE | 關閉，或從已安裝資料選版本 |

每個標準各自開關，可以只啟用 Top 10 2025，不必同時選 ASVS 或 CWE。主開關開著但四個標準全關時，政策是已啟用但未配置；標準化審查不執行，另外啟用的自有基本規則仍可執行。

規格、原始碼、API 與參數邊界三種審查分別選用；也能停用個別規則、調整根相對的排除目錄。關閉主開關或某個標準會保留先前選值，日後重新開啟仍可還原。版本、型別或 ASVS 等級不合法時拒絕載入，不會靜默改成停用。

### 從 CLI 局部更新

```sh
speclink workflow-config security --stdin --dry-run
speclink workflow-config security --stdin
```

從標準輸入提供單一 YAML 部分更新，最多64 KiB。第一條只看差異；第二條才寫入。它只改提供的項目，保留未動的設定、未知欄位及註解。空白、非設定物件、未知輸入鍵、非法值，或無法安全改寫的複雜 YAML 都會被拒絕，原檔不變。這個區塊不使用新的環境變數或另一份設定檔。

Remote 仍需要 editor 權限並核對讀取時的版次；衝突、離線或認證失效不會排隊寫入或強制覆蓋。儲存合法政策不要求 server 已安裝標準資料，也不會順便下載。接著到[安全標準資料](security-data.md)明確安裝所選版本，再[執行審查](security-review.md)。

## 寫專案說明與產出規則

```
speclink workflow-config context --stdin
speclink workflow-config rules <artifact> --stdin
```

- context 以 stdin 全文設定專案說明。stdin 只有空白時移除該鍵。
- rules 整節代換指定 artifact 的規則清單：stdin 一行一條，空行忽略。stdin 為空時移除該節。artifact 只能是目前產出流程裡的 artifact id，例如 proposal、specs、design、tasks。未知 id 被拒絕。

兩個指令沒帶 `--stdin` 時失敗並說明用法。保留其他行、`--dry-run`、remote 的版本檢查、壞檔拒寫等行為，與 set 相同。

## 審查規則：review 區塊

審查規則給審查站的 Standards 軸用，怎麼套用見 [審查站](review.md)。它是 config.yaml 的頂層 review 區塊，子鍵只有四個：

- **baseline**：預設清單模式，extend 或 replace（區分大小寫）。省略等於 extend：照常檢查內建的 12 條 code smell，再加上專案規則。replace 只檢查專案規則。
- **critical、warning、suggestion**：三個規則清單，每條一個字串。省略等於空清單。

整個 review 區塊不寫，等於 extend 加三個空清單。審查規則不是政策欄位：沒有環境變數可以覆寫，`.speclink.yaml` 的同名鍵也不讀。

```yaml
review:
  baseline: replace
  critical:
    - 所有 SQL 一律用參數化查詢，不得用字串拼接
  warning:
    - 新增的公開 API 要有對應的測試
```

> [!CAUTION]
> review 區塊寫錯，整份 config.yaml 就當成無法解析：出現不認得的子鍵（例如把 critical 拼成 critcal）、baseline 不是 extend 或 replace（例如寫成 Replace）、或某個清單不是字串陣列時，所有讀設定的指令（含 `speclink workflow-config show` 與 `speclink instructions`）都會失敗，錯誤訊息點名出錯的鍵或值，不會退回預設。remote 模式下 server 用同一套規則拒收，server 上的內容與版本都不變。

### 寫審查規則：workflow-config review

```
speclink workflow-config review <critical|warning|suggestion> --stdin
speclink workflow-config review baseline <extend|replace>
```

- **清單**：stdin 整段代換該清單，一行一條，頭尾空白去掉，空行忽略。stdin 為空或只有空白時移除該清單。沒帶 `--stdin` 時失敗並說明用法，不動檔案。
- **baseline**：寫 replace 會寫入 `baseline: replace`；寫 extend 會移除 baseline 鍵（省略就是預設）。
- 寫完後 baseline 不在、三個清單也都空時，整個 review 區塊會被移除。
- 第一個參數不是這四個之一，或 baseline 的值不是 extend、replace 時拒絕，列出合法值，不動檔案。
- 只代換 review 區塊：其他鍵、註解與空行逐位元保留；review 區塊內的註解不保留。以 YAML 保留字元（反引號、@ 等）開頭的條目會自動加引號，讀回時一字不差。
- `--dry-run`、remote 的讀改寫與版本衝突處理、離線與認證失效、壞檔拒寫，行為都與 set 相同。

成功時印一行訊息：

| 寫入前 | 指令 | 寫入後 | 訊息 |
| --- | --- | --- | --- |
| 沒有 review | review warning --stdin（一行） | warning 一條 | ✓ review.warning = 1 entries |
| warning 一條 | review warning --stdin（空） | review 區塊被移除 | ✓ review.warning removed |
| critical 一條 | review baseline replace | baseline: replace、critical 一條 | ✓ review.baseline = replace |
| baseline: replace、critical 一條 | review baseline extend | critical 一條，沒有 baseline 鍵 | ✓ review.baseline = extend (default) |
| baseline: replace、三個清單皆空 | review baseline extend | review 區塊被移除 | ✓ review.baseline = extend (default) |

### 寫完提醒技能檔過期

審查規則是審查技能自己去讀的，舊版的審查技能可能不會套用。所以 `workflow-config review` 實際寫入成功後，CLI 會檢查本機工作區的技能檔；有工具的技能檔比引擎舊時，stderr 多印一行提醒，列出過期的工具與它的技能版號、引擎版號，請你執行 `speclink update`：

```
speclink: warning: skill files are older than this engine (claude v1.44.0; engine v1.45.0) — an older review skill may not apply these rules; run `speclink update`
```

這行只是提醒：寫入照樣成功、不會回滾，exit code 仍是 0，stdout 仍是那一行成功訊息。一個工具缺技能檔、另一個過期時，只列過期的那個。以下情況不提醒：

- 技能檔都是現版，或整組缺失。
- 工作區的技能檔比引擎新（執行 update 會降級），或判斷不出來。
- 帶 `--dry-run`（沒有寫入，也不檢查）。
- set、context、rules 的寫入。這些內容由 CLI 在下指令時注入，技能檔新舊不影響生效。

remote 模式寫入成功後，用本機工作區的技能檔判斷，條件相同。檢查本身失敗時不影響寫入結果。

## 先看差異：--dry-run

所有寫入子指令（set、context、rules、review、security）帶 `--dry-run` 時，只印差異、不寫檔。差異是單一段的 unified diff：`--- a/…`、`+++ b/…` 兩行開頭，接一行 `@@` 標頭，前後各帶至多三行沒改的內容。

- 沒改動、順序也沒變的行以空白開頭顯示，不會成對出現 `-` 與 `+`。
- 只有被刪、新增或換了位置的行才帶 `-`／`+`；同一處有刪有增時，`-` 在前。
- 沒有變更時輸出空白。

例如 tasks 的產出規則原本依序是「甲」「乙」「丙」，用「甲二、乙、丙、丁」整節代換時，差異裡只有「甲」標 `-`、「甲二」與「丁」標 `+`，「乙」「丙」照原樣當上下文。

## 讓技能幫你整理：/speclink-config

`/speclink-config` 技能從固定的來源推導專案說明與產出規則：workspace 清單與相依 manifest、README、docs 索引、既有的 config.yaml，以及共用詞彙（若有）。它不掃整個 repo 的原始碼。它也管理審查規則，但審查規則只來自你，不從 repo 推導（見「第六問」）。

技能寫內容時守四條判準：

1. 引擎已自動注入的內容，以及品質關卡技能已承載的標準，不重述。判定方式是實際比對指令輸出與生成的技能檔，不憑印象。
2. 只對單一 artifact 有用的內容歸產出規則，不入專案說明。
3. 會過時的內容不寫，例如版本號、計數、統計數字。
4. 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行都核實。核實只用靜態手段，不執行被引用的測試或建置指令。

既有的規則只在兩種情況下被刪：不過四條判準，或你在政策詢問中明確撤回。「無法從固定來源導出」不是刪除理由。

### 政策先列確認表，一題確認

技能先列一張確認表，每列是一個項目的現值，共七項：locale、spec_locale、tdd、audit、worktree、任務驗證的測試範圍（第五問）、審查規則的模式（第六問）。沒設定的項目標成「未設定，預設 X」。

| 項目 | config 現況 | 確認表顯示 |
| --- | --- | --- |
| locale | tw | tw |
| spec_locale | 未設定 | 未設定，預設系統語言 |
| tdd | true | true |
| audit | 未設定 | 未設定，預設 false |
| worktree | 未設定 | 未設定，預設 false |
| 測試範圍 | 產出規則沒有測試範圍規則 | 未設定，預設完整測試 |
| 審查規則 | 沒有 review 區塊 | 未設定，預設不設定（只用內建 12 條 code smell） |

接著只問一題：全部沿用，還是要改其中幾項？

- 選全部沿用：不再逐項問，不改任何政策欄位，review 區塊也不因這題而動。
- 選要改：只逐項問你點名的項目，每題帶出現值。例如你說要改 tdd 與 worktree，就只問這兩題。
- 第一次執行（config 還沒有任何政策欄位）也一樣，先攤開確認表、問一題。

技能不從 repo 推斷任何政策值，例如有測試目錄不代表 tdd 是 true。你用自然語言回答語言（例如「繁體中文」）時，技能換成代碼（tw）再寫入，不會把顯示名稱寫進去。互動提問工具一題放不下所有項目時，「要改哪幾項」改用純文字問；沒有互動提問工具時，整個確認都用純文字。

worktree 的改動同樣走 `speclink workflow-config set worktree <true|false>` 的 `--dry-run` 核准流程。本機模式下寫入會同步技能檔，有活躍 worktree 時關閉會被拒，技能只把 CLI 列出的活躍 worktree 轉告你，不會自己收尾或移除 worktree。remote 模式下這個寫入只改 server 上的設定，不同步技能檔、也不檢查活躍 worktree，技能會告訴你這點。

### 寫入的內容用哪種語言

技能起草或改寫的專案說明與產出規則，用確認表確認後的 locale 撰寫：tw 是繁體中文，ja 是日文，en 是英文；未設定時採主機系統語言（Remote 採 Server 語言）。你在確認表改了 locale 時，技能先把已起草的行改寫成新語言，再顯示差異。

- 沒改動的既有行維持原文，不會為了統一語言而翻譯。
- 審查規則照你的原話記錄，不翻譯。
- 指令、檔案路徑、測試名、設定鍵與程式碼識別字維持原樣。

### 第五問：任務驗證的測試範圍

任務清單的驗證步驟要跑完整測試，還是只跑受影響面的測試？現行產出規則裡已有測試範圍的規則時，確認表以它帶出現值；沒有時標為未設定、預設完整測試。

- **沿用現值**：不因這一題改動測試範圍規則。那條規則仍照四條判準檢查，例如引用失效時依判準四移除。
- **改成只跑受影響面**：技能依已讀取的相依 manifest，按專案的組件型態對應各自的測試指令，組出這個專案專用的測試對應規則，寫進 tasks 的產出規則。
- **改成完整測試**：不寫任何測試範圍規則。既有的測試範圍規則視為你明確撤回，經同一個 `--dry-run` 核准流程移除；本來就沒有的話維持原樣。

### 第六問：審查規則的模式

一題三選，預設「不設定」：

- **不設定**：技能不動 review 區塊，不寫模式、不加規則。只適合當審查規則的提醒不會寫進去，最後報告列為「屬審查規則，未寫入」；技能也不會建議把產出規則的句子搬去審查規則。
- **附加**（extend）：審查站照舊檢查內建的 12 條 code smell，再加專案審查規則。
- **覆蓋**（replace）：審查站只檢查專案審查規則；三個清單都空時，Standards 軸只依 repo 文件判斷，這是正當的選擇。

config 已有 review 區塊時，確認表帶出模式與三個清單各自的條數。已有審查規則而你選不設定或沿用時，review 區塊維持原狀，既有規則不會因此被刪（刪除只經你明確撤回），但仍要過下面「會過時的內容」與「引用核實」兩項檢查。

技能不會從 repo 文件（CLAUDE.md、AGENTS.md、CONTRIBUTING、lint 設定等）整理出審查規則。審查站的 Standards 軸本來就會讀這些文件，抄一份會變成第二份正典。審查規則只來自你口述，或你提供的提醒清單。

模式為附加或覆蓋時，技能會問你要不要新增或修改審查規則，並用「有沒有正當例外」建議放哪個清單：違反一定錯、沒有正當例外的建議 critical；通常要修、偶有正當例外的建議 warning；屬偏好的建議 suggestion。「不得」「一律」「要」「宜」「偏好」這類措辭只當提示，不單獨決定分格，最後放哪一格由你確認或改過才寫入。

| 你的規則 | 有沒有正當例外 | 技能建議 |
| --- | --- | --- |
| 公開函式在錯誤路徑不得 panic，一律回傳錯誤 | 沒有 | critical |
| 元件一律使用設計 token 的顏色 | 有：圖表配色需要原始色碼 | warning（措辭雖為「一律」） |
| 新增的公開 API 要有對應的測試 | 有：純 re-export 不需要 | warning |
| 錯誤訊息偏好寫成完整句子 | 屬偏好 | suggestion |

每條既有與新收的審查規則都要過這幾項檢查：

- **重述 smell**：模式為 extend（含未設定）時，和內建 12 條 smell 同義的規則淘汰，例如「變數名稱要能說明用途」就是 Mysterious Name；replace 時不做這項。
- **已有守門**：已經有測試或 lint 守住的規矩不收。核實只用針對性的靜態查找（在 lint 設定裡找該規則、用文字搜尋找釘住該行為的測試），不通讀原始碼；查不到守門才照常收。
- **屬於其他關卡**：audit 開啟時，安全類規矩歸 audit；要求「程式照 artifact 做」的規矩歸驗證站。audit 以本次確認表確認後的值為準；audit 由關改開時，既有的安全類審查規則會在差異裡提出移除，你確認後才寫入。
- **會過時的內容**：版本號、計數、統計數字不寫。
- **引用核實**：引用的路徑、測試名、指令要以靜態手段核實存在。
- **刪除理由**：與產出規則相同，只因不過上面這些檢查，或你明確撤回。

審查規則與模式的寫入，同樣先用 `speclink workflow-config review` 的 `--dry-run` 產出差異給你看，你確認後才不帶 `--dry-run` 重跑。

### 提醒的分流

你可以把希望專案遵守的規矩（提醒）口述給技能，或給一份提醒清單：貼在對話裡，或給一個檔案路徑讓技能讀。提醒清單是你的輸入，不算固定來源的掃描範圍。技能不會自己去翻任何工具的對話紀錄找提醒。

每條提醒依三題判斷去處：違反的證據在程式還是文件？審查員只看差異能不能判斷？太晚發現的代價大不大？結果分到四個去處之一：審查規則、某個 artifact 的產出規則、兩邊都放、或不收。

| 提醒 | 判斷 | 模式為附加時 | 模式為不設定時 |
| --- | --- | --- | --- |
| 呼叫端不得用 className 覆蓋共用元件的顏色 | 證據在程式，只看差異就能判斷 | 審查規則 | 報告列為「屬審查規則，未寫入」 |
| design 要列出這次沿用的既有元件 | 證據在文件 | design 的產出規則 | design 的產出規則 |
| 共通行為要在共用層實作一次 | 計畫要寫出落點、程式要檢查，兩個階段各自有用 | 兩邊都放，兩句描述不同對象 | 只寫 design 的產出規則 |

- **審查規則的出口只在選用時開放**：模式為附加或覆蓋時四個去處都開放；模式為不設定時，分到審查規則的提醒不寫入，分到兩邊都放的只寫產出規則那一句。
- **兩邊都放的條件**：只有兩個階段各自有用時才兩邊放，其餘選一邊，因為產出規則每次產出該 artifact 都會注入。兩句描述不同對象：產出規則要求計畫寫出什麼，審查規則檢查程式本身。
- **審查規則不寫驗證站的工作**：不寫成「程式要照 artifact 做」。
- **搬移建議**：模式為附加或覆蓋時，既有產出規則裡如果有一句其實在檢查程式本身，技能會建議拆到審查規則；你確認才搬，拒絕就原句保留。技能不記被拒絕的建議，下次執行會再提一次，這是預期行為。模式為不設定時不提搬移建議。
- **時機**：提醒依賴的行為還沒進程式（例如討論已定案、對應的變更還沒實作）時先不寫，技能會告訴你並在最後報告列出。查討論或變更的狀態只用 `speclink discuss list --json` 或 `speclink list --json`，不直接讀 openspec/ 目錄。
- **各自的判準**：分到產出規則的提醒照四條判準檢查；分到審查規則的照「第六問」的檢查。

最後報告逐條列出每條提醒的去處；不收或沒寫入的附上理由：不過判準、重述 smell、已有測試或 lint 守門、屬其他關卡、尚未進程式、屬審查規則未寫入。

> [!TIP]
> 技能一律先用 `--dry-run` 產出差異給你看，你確認後才寫入。對同一份沒動過的程式碼連跑兩次，第二次的差異應該是空的，審查規則也算在內。第二次還有差異，表示判準執行不當，技能會回查判準而不是落檔。被拒絕的搬移建議不是差異，下次再提一次不影響這項驗收。

你也可以帶範圍提示呼叫技能。此時前三條判準只重審範圍內的 artifacts，第四條的引用核實仍掃全文件。



### Fork 的語言預設與查詢

locale 與 spec_locale 未設定時各自採主機 OS 語言：中文 tw、日文 ja，其餘 en；只有舊 auto 跟隨 locale。Remote 使用 Server 語言，UI 偏好不參與解析；不回寫設定。`speclink workflow-config languages --json` 只讀查詢正典設定與 OS 預設；show 仍顯示正典 null，舊 Server 缺語言資料時只有 languages 明示要求升級。

**出處**：`workflow-config`、`config-skill`、`skill-authoring`
