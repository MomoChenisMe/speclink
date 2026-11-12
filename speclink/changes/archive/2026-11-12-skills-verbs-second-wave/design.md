## Context

**渲染的現況**

- 技能原稿在 `speclink-core` 的 assets/skills，渲染是純文字取代，沒有條件語法。第一波為了一個要分情況的步驟，用了一句執行期的條件文字（「規格資料夾在這個 checkout 時才做」）。
- 產生技能檔的入口是 `init`、`init_remote`、`update` 與幾個同步的函式，它們都經同一個計畫（`SyncPlan`）。遠端專案的規格資料夾名稱一律渲染成預設值，即使 checkout 裡沒有這個資料夾。
- 自訂工具用中性渲染：斜線指令的前綴換成 `speclink `（後面有空白），所以原稿的技能引用變成 `speclink apply <name>`；開頭的呼叫說明另外寫著「`speclink <動詞>` 就是呼叫工具」。技能名稱裡，`analyze`、`drift`、`trace`、`archive`、`config`、`discuss`、`review`、`verify` 和真的動詞同名；`apply`、`propose`、`ingest`、`commit`、`improve`、`quality` 等不是動詞。
- golden 快照有八份（五個內建工具、claude 加 worktree、中性的兩種呼叫形式）。

**守門的現況**

- `skill_verbization` 的通用測試只渲染 Claude，擋「read、glob、open 之後接規格資料夾的三類路徑或詞彙表」。第一波另外加了兩個只管三支技能的測試。
- `skill_authoring` 有五項機械檢查與一張逐檔的強調字上限表，多數檔的用量等於上限。

**盤點的結果**（逐行清單在這個變更的來源查證裡；這裡只列數量與類別）

| 原稿 | 處數 | 主要的類別 |
|---|---:|---|
| commit | 20 | 讀 evidence、規格檔的 git、比對與改寫 delta |
| archive | 14 | 讀任務檔、檢查與比對 delta、改寫 delta、封存衝突時搬資料夾、顯示路徑 |
| apply | 11 | 改 artifact、閒置的判斷用 git log、重讀設計與任務 |
| verify | 8 | 用路徑找 delta、搜尋正式規格 |
| apply-worktree-pre | 7 | commit 規格檔、cat evidence |
| review、quality、improve、apply-worktree-post | 各 2–3 | evidence、規格檔的 git、提案的讀取 |
| station-shared、analyze、drift、trace | 各 1 | 改 artifact、讀 artifact、閒置、已封存的提案 |

apply、review、verify 用的 `contextFiles`（指示 JSON 回傳的檔案清單）不在上表的問題裡：遠端專案的 CLI 會把這些文件放進 checkout 的唯讀投影，路徑是真的檔案。它們只在「沒有 checkout」的形態落空，而這幾支技能在那個形態本來就不適用。

限制：

- 討論的裁定：不在每支技能裡做「規格在不在 checkout」的執行期判斷；讀寫改走動詞，git 步驟在產生技能時決定。
- 原稿每檔不超過 500 行；強調字的上限多半已滿。
- 改原稿要同時前進資產版號、重新產生 golden 與 lock。

## Goals / Non-Goals

**Goals:**

- 遠端專案產生的技能檔，沒有任何一句叫 agent 去碰規格資料夾底下的檔案。
- 本機專案的技能，讀寫規格文件的步驟也都指名動詞；只有 git 與顯示路徑保留路徑。
- 中性渲染裡，交棒和動詞一眼分得開。
- 守門是通用的規則，之後新增技能或改句子時，漏掉的情況直接失敗。

**Non-Goals:**

- 依 Server 的流程設定決定產生哪些技能（remote-skill-delivery）。
- baseline、manual 在遠端可用（這個變更只替它們補上「不在」時的拒絕步驟）；內部技能。
- 新動詞。

## Decisions

### D1: 條件區塊

原稿新增兩種區塊記號，各自獨佔一行：

```
{{#SPEC_IN_CHECKOUT}}
…規格資料夾在這個 checkout 時才留下的內容…
{{/SPEC_IN_CHECKOUT}}

{{^SPEC_IN_CHECKOUT}}
…不在時才留下的內容…
{{/SPEC_IN_CHECKOUT}}
```

規則：

- 記號行本身不輸出。區塊可以包住任意行數，含程式碼區塊與清單項目。
- 不能巢狀；開了沒關、關了沒開、名稱不是 `SPEC_IN_CHECKOUT` 都是渲染錯誤，由測試在建置期抓到（每份原稿都被渲染一次）。
- 區塊的處理在其他取代之前。共用的片段（寫回的那一段、對話語言）也經同一個處理。
- `instructions --skill` 印出內部技能的原文時不處理區塊；內部技能這次不用區塊。

只有一個軸、一種名稱。不做一般化的條件語言。

- **否決：沿用第一波的執行期條件句** — 討論已否決在技能裡做執行期判斷；遠端的技能檔會留著一堆永遠不成立的步驟。
- **否決：為遠端另外寫一套原稿** — 兩套會各自漂移。

### D2: 軸的來源

`render_skill_file_for` 多一個參數：`location: SpecLocation`。`SpecLocation` 是兩個值的型別（`InCheckout`、`Elsewhere`），不用布林——呼叫端已經有一個「worktree 政策開不開」的布林，兩個並排的 `true`／`false` 看不出誰是誰（品質關卡第 1 輪的修正）。下表的「真」對應 `InCheckout`，「假」對應 `Elsewhere`。Node 的 `specInCheckout` 仍是布林，在綁定層轉換。

| 入口 | 值 |
|---|---|
| `init`（本機） | 真 |
| `init_remote` | 假 |
| `update` 與各同步函式 | `.speclink.yaml` 有 remote 區段 → 假；否則真 |
| Node 的 `skills.render` | 選項 `specInCheckout`，預設真 |
| 既有的技能檔探測（比對「現在會產生的內容」） | 和 `update` 相同的判斷 |

遠端專案的判斷用引擎既有的模式解析（讀 `.speclink.yaml`），不看規格資料夾存不存在：遠端專案的 checkout 可能殘留舊的規格資料夾，那不代表規格在這裡。

`{{SPEC_DIR}}` 在「不在」的渲染裡仍然會被取代，但 D4 的守門保證它不出現。

### D3: 各技能的改法

共通的對照：

| 原本的做法 | 改成 |
|---|---|
| 用路徑讀變更的 artifact | `speclink artifact cat <id> --change <name>`（已封存的變更也可以） |
| 讀 `.evidence.json` | `speclink evidence show <name> --json`，檔案清單取 `files`，逐任務取 `entries` |
| 用路徑檢查有沒有 delta、列出 capability | `speclink list --json` 該變更的 `deltaCapabilities` |
| 用路徑讀或搜尋正式規格 | `speclink list --specs --json` 找名稱，`speclink show <capability> --item-type spec` 讀內容 |
| 讀任務檔數勾選 | `speclink list --json` 的 `completedTasks` 與 `totalTasks` |
| 就地改寫、直接修正 artifact | 共用的寫回片段（第一波的 `artifact-write-back`），每支技能插入一次 |
| `git log` 看規格資料夾多久沒動 | `speclink list --json` 的 `created` 與 `modifiedAt` |
| 對規格檔的 git 指令、顯示規格路徑 | 留著，放進「在」的區塊 |

逐支的重點：

- **apply**：閒置的規則改成「`created` 超過 5 天，而且 `modifiedAt` 超過 3 天」。沒有 `modifiedAt` 時只用第一個條件，並在向使用者說明時寫明無法判斷最近有沒有動；沒有 `created` 時不算閒置。這兩個欄位要在步驟 2 把變更標成進行中「之前」讀取並留到步驟 3d 用：`in-progress add` 會寫入變更，本機的 `modifiedAt` 是變更資料夾裡最新的檔案時間，之後才讀就永遠是「剛剛」。apply-with-worktree 同理，在 P2（建立 worktree 之前）讀取，本體的 3d 用那一組值（品質關卡第 1 輪的修正）。重讀設計與任務改用 `artifact cat`（`contextFiles` 仍是第一次讀取的來源）。修正 artifact 用寫回片段。
- **drift**：同一條閒置的規則，同樣的寫法。兩處用同一個句子。
- **apply-worktree-pre**：「artifact 還沒 commit 就先 commit」的整段放進「在」的區塊；「不在」的區塊寫一句：規格文件在 Server 上，沒有東西要 commit。讀 evidence 改用動詞。
- **apply-worktree-post**：要 stage 的檔案 = `evidence show` 的 `files`；「在」的區塊另外加上這個變更的規格資料夾。對應的守則句子同樣分兩種。
- **commit**：檔案清單的來源改成 `evidence show`。本機舊位置的退路（`.speclink/touched/`）留著，它是 checkout 自己的狀態檔，不是規格文件。規格資料夾底下的髒檔、封存造成的檔案搬移、手冊資料夾的提醒，都放進「在」的區塊；「不在」的區塊寫一句：這次 commit 只含程式碼的檔案。任務進度與 delta 的比對照共通的對照。確認閘門顯示的內容在兩種情況各自正確。
- **archive**：讀任務、檢查與比對 delta、改寫 delta 照共通的對照。封存名稱衝突時「改名或刪除既有的封存」放進「在」的區塊；「不在」時停下來告訴使用者要在 Server 上處理。顯示「Archived to」的路徑改成顯示動詞回傳的封存名稱；路徑那一行放進「在」的區塊。收尾的 commit 提醒與手冊的提醒放進「在」的區塊。
- **improve**：讀進行中變更的提案用 `artifact cat`；已封存變更的 evidence 用 `evidence show <封存名稱>`。
- **verify**：delta 用 `deltaCapabilities` 加 `artifact cat specs/<capability>`；找相關的正式規格用清單與 `show`。
- **quality**：「列出變更資料夾裡未 commit 的檔案」放進「在」的區塊；「不在」的區塊改成和本機那一邊對稱：列出點名 artifact 的必修發現，問使用者上一個工作階段有沒有用寫回修過它們；有，就用 `speclink artifact cat` 讀出那些段落放進複驗的簡報，沒有，就不跑那一站（品質關卡第 1 輪的修正；原本的寫法在續跑時沒有出口）。
- **station-shared**：改 artifact 的措辭用寫回片段。
- **analyze**：讀 artifact 用 `artifact cat`。
- **trace**：已封存變更的提案用 `speclink artifact cat proposal --change <溯源結果裡的名稱>`；`archivedDir` 只當顯示用。
- **discuss、propose、ingest**：記錄檔的位置、詞彙表的路徑這些說明放進「在」的區塊（「不在」時說明記錄由 Server 保存）；「把封存記錄搬回來」從執行期條件句改成兩個區塊，`SPEC_FOLDER_CONDITION` 這個取代項移除。第一波釘住那個句子的測試同批改寫。

寫法上的約束：寫回片段在每支技能最多插入一次；不新增強調字（必須調整上限表時，在任務的完成說明寫明是哪一檔與原因）；每檔維持 500 行以內——commit 與 archive 最接近上限，區塊會增加行數，必要時把重複的說明收成一句。

### D4: 守門

**原稿層**（`skill_authoring` 的第六項）：對 assets/skills 下每一份原稿，`{{SPEC_DIR}}`、`.evidence.json`、`.meta.yaml` 這三種字樣只能出現在 `{{#SPEC_IN_CHECKOUT}}` 區塊裡。豁免的原稿列在測試的一張表，每筆附理由：

- `baseline.md`、`manual.md`：直接寫規格資料夾。以「不在」渲染時，兩支都在動手寫入之前先說明不支援並結束（這一段是稽核後補的，見「稽核結論」）。
- `sync.md`、`clarify.md`、`tdd.md`：內部技能，不產生技能檔。
- `config.md`：不在豁免表。那句禁止用的是「spec directory」這個詞，沒有佔位符，原稿不用改（見 Open Questions）。

表是封閉的：新原稿預設受檢。

**渲染層**（`skill_verbization`，取代既有的通用測試與第一波的兩個測試）：

1. 以「不在」、規格資料夾名稱 `openspec`、Claude 與中性 tool-call 兩種目標，渲染全部非豁免的技能。結果不得含 `openspec/`，也不得含 `.evidence.json`、`.meta.yaml`。
2. 以「在」渲染全部非豁免的技能。含 `openspec/` 的每一段（到下一個空白行為止的連續幾行），不得同時含這些字（不分大小寫、字詞邊界，含字尾變化如 reads、editing、rewritten）：read、open、glob、grep、cat、check for、compare against、edit、rewrite。也就是「在」的區塊只能放 git 步驟與顯示，不能藏讀寫。以段落為單位與字尾變化是稽核後加嚴的。品質關卡第 1 輪再補上：`write`（含 overwrite）、`check whether`、`compare … against` 中間隔幾個字的寫法，以及「以冒號結尾的引導句和緊接在後的那一段算同一段」。
3. 每一份渲染結果不得殘留 `{{`。
4. 第一波釘住的正面條件保留並擴大：用到寫回的技能都含 `--stdin --force` 與「不要改成直接編輯檔案」那一句；讀 evidence 的技能含 `speclink evidence show`。

**交棒**（同一個測試檔）：以中性的兩種呼叫形式渲染全部技能，對技能登錄表裡的每個名稱 N，結果不得含反引號裡以 `speclink N` 開頭、後面接空白或反引號的片段，除非 N 是真的動詞而且那一處在原稿裡本來就寫成動詞（原稿的技能引用與動詞寫法不同：技能引用是 `/speclink:N`，動詞是 `speclink N`）。做法是直接比原稿：原稿每一處 `/speclink:N`，在中性渲染裡必須是 `speclink-N`。

### D5: 中性渲染的交棒

自訂工具的斜線取代從 `speclink `（帶空白）改成 `speclink-`。所以原稿的 `/speclink:apply <change-name>` 渲染成 `speclink-apply <change-name>`，和技能資料夾的名稱相同。原本「把 `/speclink-` 改成 `speclink-`」的第二步因此不再需要，移除。

兩種呼叫形式的開頭說明各加一句（英文，和既有說明同一段）：寫成 `speclink-<name>` 的是另一支技能——載入並依它執行；它不是動詞，不要當成指令或工具的參數送出。tool-call 說明裡的 argv 範例同批從 `["apply", "add-auth"]` 改成 `["list", "--json"]`：`apply` 不是動詞，原本的範例正好示範了要避免的錯誤。

內建工具不變：它們的技能引用是各自的斜線或錢字號寫法，本來就和動詞不同。

- **否決：改成「the apply skill」這種散文寫法** — 原稿的引用後面常帶參數（變更名稱），散文化之後要逐句改寫。

### D6: golden 與版號

- 資產版號前進一版；lock 檔與八份既有快照重新產生。
- 新增兩份快照：Claude 的「不在」與中性 tool-call 的「不在」。內嵌宿主與遠端專案拿到的就是這兩種。
- lock 的指紋改成兩種情況都算進去：只改「不在」區塊裡的句子，也要前進版號。
- 用 diff 檢查：內建工具的五份快照裡，交棒的寫法沒有變；中性的兩份裡，交棒全部從 `speclink N` 變成 `speclink-N`。

### D7: Node 的 skills.render

`RenderOptions` 新增 `specInCheckout?: boolean`（預設真）。正式規格 node-sdk「渲染 API」同時更正兩處和實作不符的敘述：選項沒有 `store`，SDK 沒有 `instructions.render`。

## Implementation Contract

**行為**

- 對本機專案執行 `speclink update`：產生的技能檔裡，讀寫規格文件的步驟都指名動詞；規格路徑只出現在 git 步驟與顯示的句子。
- 對遠端專案執行 `speclink update`（或 `speclink init --store remote`）：產生的技能檔不含規格資料夾的路徑，不含 `.evidence.json` 與 `.meta.yaml`，不含對規格檔的 git 步驟。
- 自訂工具的技能檔：每一處交棒都寫成 `speclink-<名稱>`；開頭的說明有「這是另一支技能，不是動詞」的句子。
- `skills.render(name, { target: 'neutral', invocation: 'tool-call', specInCheckout: false })` 的輸出等於新增的中性「不在」快照裡對應的那一段。
- D4 的三組守門通過；故意在原稿加一句「cat 規格路徑」或一處區塊外的路徑時，測試失敗並指出檔名。

**介面與資料形狀**

- 原稿的區塊記號（D1）。
- `speclink_core` 的渲染函式多一個「規格在不在 checkout」的輸入。
- Node：`RenderOptions.specInCheckout`。
- 資產版號前進一版。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| 原稿的區塊沒有關、巢狀、名稱不認得 | 渲染回報錯誤；測試失敗並指出檔名與行 |
| 原稿在區塊外寫了規格路徑 | `skill_authoring` 失敗 |
| 「在」的區塊裡藏了讀寫的字 | `skill_verbization` 失敗 |
| 遠端專案殘留舊的規格資料夾 | 仍然以「不在」渲染 |

**驗收**

- `cargo test -p speclink-core --test it skill_authoring::`、`skill_verbization::`、`render_golden::` 通過。
- `cargo test -p speclink-core` 全部通過；`cargo test -p speclink-cli` 通過（技能檔內容的期望值同批更新）。
- Node 的渲染測試通過，新增 `specInCheckout` 的兩個情境。
- 技能評測的守門（每支技能都有評測）仍然通過：`node --test scripts/*.test.mjs scripts/*/*.test.mjs`。

**範圍**

- 在範圍內：渲染的軸與區塊、十七份原稿（含 baseline 與 manual 的拒絕步驟）、守門、中性渲染的交棒、Node 的選項。
- 不在範圍內：動詞與線上、依流程設定決定產生哪些技能、讓 baseline 與 manual 在遠端可用、`speclink link`／`unlink` 之後自動重新產生技能、桌面 app、使用者文件。

## 稽核結論

任務 6.1 依 sharp-edges 清單逐項檢查的結果。

| 檢查的項目 | 結果 | 處置 |
|---|---|---|
| 原稿能不能用換行繞過「在」的字詞比對 | 可以：原本逐行比對，把路徑放到讀寫字的下一行就逃得掉 | 改成以段落為單位（到下一個空白行為止）；補測試。discuss 的兩處說明因此各自獨立成段，quality 的三個字改寫 |
| 原稿能不能用拼字繞過 | 可以：`reads`、`editing`、`rewritten` 這些字尾變化不在原本的比對裡；另外，把資料夾名稱寫死（`speclink/changes/…`）可以繞過佔位符的檢查 | 字詞比對涵蓋字尾變化；原稿層新增「寫死的規格資料夾名稱」檢查；補測試 |
| 條件區塊的錯誤會不會被靜默吞掉 | 寫壞的區塊（沒關、巢狀、名稱不認得、記號不獨佔一行）渲染直接中止，而且渲染發生在任何寫入之前，不會寫出半份技能。有一個洞：多一個空白或小寫的記號（`{{ #SPEC_IN_CHECKOUT}}`）會被當成一般文字，兩種情況的內容都留下 | 原稿層新增檢查：提到 SPEC_IN_CHECKOUT 的行必須是三種記號之一；渲染結果不殘留 `{{` 的測試是第二道 |
| 豁免的每一筆是不是真的在 remote 專案先拒絕 | 不是：baseline 的原稿沒有拒絕的步驟；manual 靠執行時讀 `.speclink.yaml`，內嵌宿主沒有這個檔 | 兩份原稿各加一個「不在」區塊，開頭就說明不支援並結束；新增 baseline-skill 的需求、修改 manual-skill 的需求；補測試 |
| 「不在」的 commit 與 worktree 收尾會不會 stage 到不屬於這個變更的檔案 | 不會：檔案只取自 `evidence show` 的 `files` 加上使用者確認的檔案，逐檔 stage；checkout 裡殘留的舊規格資料夾若有髒檔，會列在「Unrelated Changes」而不納入 | 不用改 |
| 中性渲染裡有沒有殘留會被當成動詞的技能名稱 | 有一處：tool-call 開頭說明的 argv 範例是 `["apply", "add-auth"]` | 範例改成真的動詞；測試另外釘住「不是動詞的技能名稱不得以 `speclink <名稱>` 出現」 |
| 技能評測套件的檔案是否仍對得上每一支技能 | 是：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過 | 不用改 |
| `skills.render` 的 `specInCheckout` 傳非布林值 | 綁定層直接拋出錯誤，不會把字串 `"false"` 當成真 | 補測試釘住 |

守門測不到、留給審查的部分：

- 不帶資料夾名稱的相對路徑（例如只寫 `changes/<name>/tasks.md`）與不寫檔名的說法（「打開那份 evidence 檔」）。機械比對會誤判太多正常的句子，所以沒有加規則。
- 「在」的字詞清單是規格列的九個字。換成其他動詞（load、parse、inspect）不會被擋。
- 渲染對內嵌原稿的區塊錯誤採取中止（panic），不回傳錯誤：原稿是編譯期的常數，每一份都在測試裡以兩種情況渲染過，執行期不會發生。

## Risks / Trade-offs

- [回歸對照] 全部 golden 快照會變。→ 刻意的變更；D6 的 diff 檢查把「該變的」與「不該變的」分開看。
- [回歸對照] 技能的行為靠文字，改寫可能讓 agent 做法不同。→ 技能評測套件涵蓋每支技能；改完跑一次評測的守門。apply、commit、archive 三支另外在本 repo 實際走一遍（手動任務）。
- [行數與強調字] commit、archive 接近 500 行。→ D3 的約束；`skill_authoring` 會擋。
- [跨平台] 區塊的處理要對 CRLF 的原稿得到相同結果。→ 測試各放一份 LF 與 CRLF 的樣本。
- [閒置的判斷變寬鬆] 檔案的修改時間不等於 commit 時間。→ 它只決定要不要建議跑 drift，誤判的代價是多跑或少跑一次檢查；技能把它寫成提示。
- [遠端專案的品質關卡看不到 artifact 的未提交修改] 沒有 git 可以列出。→ 續跑時改問使用者，並用 `speclink artifact cat` 讀出被點名的段落（D3 的 quality）。
- [link／unlink 之後技能檔和專案的形態對不上] `speclink link` 與 `unlink` 只改 `.speclink.yaml` 的 remote 區段，不會重新產生技能；探測只比版號，所以也不提示。使用者要自己執行 `speclink update`。→ 已知缺口（品質關卡第 1 輪的發現，使用者決定接受），交給 remote-skill-delivery。桌面 app 綁定 checkout 時會同步，不受影響。
- [儲存解耦] 完成後，產生的技能不再假設規格是檔案；只有明確標成「在 checkout」的步驟才碰路徑。

## Migration Plan

使用者執行 `speclink update` 取得新的技能檔。要退回時還原這個變更的 commit 並重新執行 `speclink update`。

## Open Questions

- `config.md` 能不能在不提路徑的情況下保留那句禁止（D4）。**已定（任務 3.4）**：可以，而且原稿不用改。那一句是「do not read the spec directory's files for it」，用的是「spec directory」這個詞，沒有規格資料夾的佔位符，也沒有那兩個檔名，所以原稿層的第六項與渲染層的兩組規則對它都通過。`config.md` 不進豁免表；豁免表只有 baseline、manual（直接寫規格資料夾）與 sync、clarify、tdd（內部技能）五筆。
- commit 的確認閘門在遠端專案要顯示什麼來代表「規格這一邊的變動」。**已定（任務 3.2）**：commit 計畫的「Change Artifacts」一節留著，內容換成一行 `(none in this commit — the spec changes are already on the Server)`；先封存子流程重新顯示計畫時，同一節寫 `(none in this commit — the change is archived on the Server)`。和 commit-skill「commit 確認閘門所見即所簽」沒有牴觸：那條需求要的是「計畫與訊息先以可見文字輸出、確認問題不指涉沒輸出過的內容」，這一行本身就是計畫裡的可見文字；技能另外寫明「顯示的檔案就是步驟 8 會 stage 的檔案」。遠端專案沒有 evidence 時，步驟 5 的「沒有東西可 commit」先停下來，所以「只 commit artifact」的警告只留在「在」的區塊。
