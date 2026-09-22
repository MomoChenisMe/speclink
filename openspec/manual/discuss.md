---
title: 討論：需求還模糊時
section: SDD 工作流
order: 100
keywords: [討論, discuss, 決策樹, slug, 結論, 條列, 轉為變更, 分期轉出, hold, 最後一刀, 改進討論, 搜尋舊討論, search]
sources: [discuss-skill, discussion-docs, improve-skill, user-documentation]
generated: 2026-09-22T15:37:29+08:00
---

# 討論：需求還模糊時

需求還有取捨、還要辯論時，先開一場討論。你呼叫 `/speclink-discuss`，agent 帶著你把決策一個一個解掉，過程寫進一份討論記錄。結論出來後，記錄可以轉成新變更、連到既有變更，或封存起來留下推理。

只是想理解問題、沒有待決事項時，直接問答就好，不用開討論。

## 討論怎麼進行

### 開場：先查、再問

agent 開場先做一輪淺掃，順序固定是三段：正式規格 → 舊討論 → 程式碼。

1. 先查正式規格（`speclink list --specs`）。候選最多 5 個：名稱命中關鍵字多的排前面，同數依指令列出的順序。依這個順序最多讀 3 份 Purpose，只有主題直接動到的 capability 才讀全文。零命中就靜默跳過。
   命中 capability 之後，agent 再看這些 capability 有沒有「進行中的 delta」：用開場已經跑過的 `speclink list --json`，比對每個未封存變更的 delta capability 清單（不看變更的狀態，因為未封存的 delta 都還沒併進正式規格；也不逐一執行 `speclink show`）。命中的記下變更名，用 `speclink artifact cat specs/<capability> --change <變更名>` 只讀該 delta 的需求標題與 ADDED／MODIFIED／REMOVED／RENAMED 區段標記，最多 3 份、不讀全文；變更在 worktree 裡時到它的 worktree 路徑下讀。正式規格零命中、沒有進行中變更、或沒有變更的 delta 動到命中的 capability 時，這一小步靜默跳過。進行中的 delta 屬於規格這一段，不另立一段。
2. 再查舊討論。agent 用你題目的關鍵字，加上正式規格掃描轉譯出來的英文詞，執行 `speclink discuss search`（見下方「搜尋舊討論的定案」）。命中的決定行全部列出；整份結論最多讀 3 份，topic 命中的優先。這一步不分討論種類，一般討論與改進討論一起查。
3. 最後掃原始碼，最多讀 5 個檔。主題已經點名具體檔名或符號時，程式碼這條線直接開跑。

淺掃的用途是接地與判斷需求清不清楚。深入查證留到決策樹逐節點進行。

> [!NOTE]
> 規格對開場淺掃有兩種說法：較早的版本是「正式規格 → 程式碼」兩段，較晚的版本加入舊討論查核成為三段。本頁依較晚的版本撰寫，詳見[本手冊的來源](about.md)。淺掃的時間盒是：正式規格最多 3 份 Purpose、進行中 delta 最多 3 份且只讀標題與區段標記、原始碼最多 5 個檔。

### 需求鈍或利，決定要不要先磨

- 需求鈍（沒有可驗證的目標、沒有門檻、用「改進一下」這類措辭）：agent 先進入磨需求階段，一次一題，把目標、範圍、門檻、成功判準問清楚。
- 需求利：磨需求階段縮成零題，直接進入假設清單。

### 決策樹：一次一題、上游先解

agent 先攤開決策空間：根節點是「這題到底在決定什麼」，往下展開子決策與它們之間的依賴。提問依依賴順序進行，一次只問一題，上游的決策先解。

每一題都不會空白提問：

- 磨需求的題目，附上現況或正式規格證據，或最佳猜測的建議。
- 假設清單裡證據撐不起立場的節點，附上 agent 的建議答案與 Evidence（檔案路徑或查證結果）。

你只需要同意或修正。

### 事實自己查，決策才問你

每個節點解決前 agent 先分診：程式碼、檔案系統、工具查得到的事實，agent 自己查證後附為 Evidence，不拿來問你、也不憑印象作答。只有真正要你裁定的決策才會問你。

### 正式規格是證據，不是裁決

淺掃命中相關正式規格或舊討論時，假設清單會對你的需求逐項對照，分成四類：

- 正式規格已涵蓋：附規格證據。
- 與正式規格衝突：指出衝突內容並附證據，由你決定改正式規格還是改需求。agent 不會因此擋下討論方向。

  前兩類的 capability 有進行中的 delta 時，該條假設的類別標籤後面多一個標記「（進行中變更 <變更名> 將改動：<需求名>）」，一個變更一個標記，需求名取自那份 delta 的標題、逗號分隔。超出時間盒沒讀到的命中寫成「（進行中變更 <變更名> 將改動）」，不猜需求名。Evidence 同時列正式規格與那份 delta（寫法 `<變更名>: <capability>`）。這個標記只是提醒，agent 不會拿它擋下討論方向；對照表也不會為進行中的 delta 另開一類。例如：需求對應正式規格 client-protocol 的既有承諾，而 add-change-plan-remote 的 delta 動到它的「討論資訊 payload」需求，該條假設就寫成「Covered by canon（進行中變更 add-change-plan-remote 將改動：討論資訊 payload）」。
- 正式規格沒講：新地盤。agent 順帶檢查 capability 命名是否貼近既有規格。
- 舊討論已定案：某份舊討論已經對這件事下過決定。再細分三種：
  - 曾否決：附上當時的理由。你要重開這個方向時，agent 會請你說明當時的理由為什麼已經失效。
  - 曾延後：可以直接接手。
  - 已落地：正式規格會照出來，不重列。

偏離正式規格是允許的結論方向，但會記進討論記錄，成為有意識的決定。舊討論的定案同樣只是證據，agent 不會拿它擋下你的方向。

### 什麼時候停

停止由你主導。agent 最多輕推一次；你喊停時，還沒走的分支記進結論的 Deferred 欄位。沒有「所有分支都解完才能收斂」的規定。

### 多個需求一起談

一份討論載有多個需求時，首輪的 Open 列出全部需求，之後每輪的 Open 復述剩餘未談的項目。已定案項的去向寫在該輪 Position 的第一句。

續用一份還開著的討論時，agent 先給你恢復摘要：逐輪的 Focus 與 Position 第一句，加上最後一輪的 Open 邊界，然後才接續討論。

### 用文件當輸入

主題可以直接給一個文件路徑：你自己寫的 markdown、plan mode 的產出、repo 裡的 docs，任何讀得到的路徑都行。agent 讀取文件、把裡面的主張萃取成決策樹節點，逐條對 codebase 分診成三類：

- 證實：附程式碼證據。
- 牴觸：逐條指出文件內容與程式碼實況的差異，附證據。不會籠統帶過。
- 真決策：送你裁定。

記錄的 Context 會多一行 `Source doc: <路徑>`。輪的 Evidence 引用文件時用段落標題或短句。記錄只存討論結果，不內嵌整份文件。agent 不會修改你的原始文件。

## 討論記錄

### 什麼時候建檔

記錄在第一輪實質往返時才建立：你對假設清單給出確認或修正、或回答了第一個問題的那一刻。在你回覆之前，磁碟上不會有記錄檔。誤觸發或一輪就答完的主題，零檔案離場。

### 檔名（slug）

建檔時 agent 從主題衍生一個英文 kebab-case 的 slug，並以 `--slug` 傳入；主題保留你的語言原文。

```
speclink discuss new "看板搜尋列" --slug board-search-bar
```

slug 只接受純 ASCII 的 kebab-case：小寫英文字母與數字的段落，以單一連字號串接。

| slug | 結果 | 原因 |
| --- | --- | --- |
| Board-Search | 拒絕 | 含大寫字母 |
| 看板搜尋 | 拒絕 | 含非 ASCII 字元 |
| board_search | 拒絕 | 含底線 |
| board search | 拒絕 | 含空白 |
| -board | 拒絕 | 以連字號開頭 |
| board--search | 拒絕 | 連續連字號 |
| （空字串） | 拒絕 | 不得為空 |
| board-search-2 | 接受 | 合法 |

不合法時指令以錯誤結束、不建檔。slug 與既有討論同名時，指令報該討論已存在，不覆寫。

沒帶 `--slug` 時，slug 從主題衍生：英數字轉小寫、空白與底線轉連字號、ASCII 標點去掉、中文等非 ASCII 字母原樣保留。例如「config context 與 rules GUI 編輯」會變成 `config-context-與-rules-gui-編輯`。衍生結果為空（例如主題只有標點）時報錯。

### 記錄的結構

每份記錄有 Context、Rounds、Conclusion 三個區段。每一輪有 Focus、Position、Ruled out、Open 四個欄位。首輪的 Position 攤開初始決策空間（可以含 ASCII 樹），之後每輪聚焦解掉一個節點，中途發現的新分支記進該輪的 Open。

Context 固定有一行 `Prior discussions: <slug 清單>`，列出開場淺掃查到的舊討論；零命中時寫 none。Context 既有的「相關變更與規格」那一句，會一併列出淺掃命中的進行中 delta，寫法 `<變更名>: <capability>`，多筆逗號分隔；零命中時那一句只列正式規格與變更名，不加任何空標記。記錄的骨架不變，不新增獨立的標記行，舊記錄不用遷移。

輪只能往後追加。這三個指令寫內容：

| 指令 | 行為 |
| --- | --- |
| `speclink discuss context <slug>` | 寫 Context。重跑會覆寫 |
| `speclink discuss add-round <slug>` | 追加一輪。不能改寫既有的輪 |
| `speclink discuss conclude <slug>` | 寫結論。重跑會覆寫 |

內容去掉前後空白後是空的，指令會中止。錯誤訊息指出內容為空，並提醒可能漏帶 `--stdin`。conclude 遇到空內容不會把狀態翻成 concluded。以管線送內容時，不帶 `--stdin` 也會被讀進去。

輪內文裡出現「## Conclusion」這類與結構標題同名的行時，寫入時會自動跳脫。它不會截斷區段，也不會讓輪數膨脹。

### 結論怎麼寫

結論有六個欄位：Decision（決定）、Rationale（理由）、Rejected alternatives（否決替代案）、Deferred（擱置）、Capture to（記錄去向）、Next（下一步）。agent 寫結論時遵守一條條列規則，與輪內「Position 用條列不用散文」對稱：

| 欄位 | 寫法 |
| --- | --- |
| Decision | 先用一句定論起頭；內容超過一句時，其後以 `- ` 一點一行條列 |
| Rejected alternatives | 標頭後不接文字，直接一點一行，每行寫「方案——落敗理由」 |
| Deferred | 標頭後不接文字，直接一點一行，每行寫「問題——為何現在不解」；沒有擱置項時單寫 none |
| Rationale、Capture to、Next | 維持單段 |

Decision 保留全部定案細節，不為了縮短而刪減；各點（含縮排子項）不回指「第幾輪」。結論規劃分幾刀轉出時，Decision 每刀一個 bullet，開頭寫 ``**cut N `change-name`**：`` 加一句範圍，該刀的細節放在縮排子項、一子項一件事。

```
**Decision**: 新版 Server 與團隊管理分三刀轉出。
- **cut 0 `redesign-settings-page`**：設定頁與帳號選單
  - 側欄底部單一帳號列：頭像＋名字＋方案
  - 選單項目：使用情況／設定／登出
- **cut 1 `add-team-server-foundation`**：身分、專案、成員與管理畫面
  - Server：Fastify 5＋PostgreSQL＋Drizzle
- **cut 2 `add-shared-byok-gateway`**：共用模型閘道
  - 路由 /api/model-gateway/azure：驗 Bearer→查權限→換真 key→串流轉發
**Rationale**: 登入是所有團隊功能的前提，所以 Server 第一刀只做身分與專案。
**Rejected alternatives**:
- 大場景 SVG 插圖——舊版做法，與現行系統風格不合
- 第二顆實心按鈕——次要動作一律底線文字鈕
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion team-server-and-admin
```

規則只在欄位超過一句時才要求條列：單刀結論的 Decision 只有一句、Rejected alternatives 只有一項時，兩欄維持單行即可。規則只約束新寫入的結論；規則落地前寫的舊記錄（Decision 是整段長句的）一個位元都不改，桌面 app 的結論分頁照常顯示、不報錯。

在桌面 app 的討論詳情面板裡，結論分頁把這六個欄位顯示成標籤區塊，條列的內容在標籤區塊內顯示為清單，見[規格、討論、已封存與搜尋](desktop-browse.md)。

## 搜尋舊討論的定案

想知道某件事以前有沒有討論過、當時怎麼決定的，用 search：

```
speclink discuss search <關鍵字> [<關鍵字> ...]
```

至少要給一個關鍵字。可以加 `--json` 或 `--no-color`。search 只讀不寫，不會動任何檔案。

**比對什麼**：不分大小寫的子字串比對，多個關鍵字任一命中就算命中。比對範圍只有記錄的 topic、slug，以及四種決定行：

| 決定行 | 在哪個區段 | 結果裡的標示 |
| --- | --- | --- |
| Ruled out（否決） | 每一輪 | ruled-out |
| Decision（決定） | 結論 | decision |
| Rejected alternatives（否決替代案） | 結論 | rejected |
| Deferred（擱置） | 結論 | deferred |

決定行後面緊接的條列項也算進去，每個命中的條列項各算一筆。Focus、Position、Open、Evidence 與一般散文都不比對，所以只在 Evidence 提到的字搜不到。

**搜哪裡**：進行中與已封存的記錄一起搜，沒有旗標可以縮小範圍。記錄還沒有任何輪或結論時，只拿 topic 與 slug 比對，不會讓查詢失敗。

**結果順序**：topic 或 slug 命中的記錄排前面，其餘排後面。兩群裡面各依建立日期新到舊，同日依 slug 字典序。同一份記錄裡的命中依文件順序。

**輸出長相**：有命中時第一行是標題，之後每份記錄一行，底下每筆命中各一行縮排：

```
Discussions matching "<關鍵字>":
  • <slug> [<status>, archived|live] (<created>) — <topic>
      <where> <kind>: <text>
```

`<where>` 是命中的位置：frontmatter（topic 或 slug）、round-N（第 N 輪）或 conclusion（結論）。`<kind>` 是上表的標示，加上 topic 與 slug 兩種。`<text>` 是那一行的原文。

零命中時印一行 `No discussions match "<關鍵字>".`，仍算成功。沒帶關鍵字時指令以錯誤結束，stderr 說明用法。輸出是英文，與其他 discuss 指令一致，不隨介面語言變動。

## 結論之後怎麼走

討論結論後至少有四條路：

| 你的決定 | 做法 |
| --- | --- |
| 開一個新變更 | `/speclink-propose --from-discussion <slug>`。直接跑完整的產物流程，完成後可以交給 apply |
| 修正或補充一個既有變更 | 依序：`speclink discuss link <slug> <變更名>` → `/speclink-ingest` 反映內容 → `speclink discuss seal <slug> <變更名>` |
| 決定不做 | 封存討論，保留推理記錄。不要建空變更，也不要讓已結論的討論無限期停在待收尾 |
| 討論沒有實質內容 | `speclink discuss discard <slug>`。有輪的討論要加 `--force` |

討論技能結尾只會建議第一條的 propose 入口。中途轉出（見下文）是討論還沒結束時用的工具，不是結論後的建議路徑。

> [!NOTE]
> 一份討論可以轉出多個變更。最後一個存活的變更封存時，討論會一起封存，前提是結論已經寫入，而且記錄沒有「保留在途」旗標（見下方「分期轉出」）。

### 中途轉出

多需求的討論裡，某一項已經談定、你想先立案時，不用先寫結論。直接：

```
speclink discuss promote <slug>
```

引擎會建立變更骨架，提案的 Why 以結論預填；還沒有結論時以主題預填。討論繼續加輪談剩下的項目。promote 只建立骨架，還要由 propose 補齊必要產物，不能直接 apply。

最後照常 conclude。引擎保留「已轉出」狀態、寫入結論，並把先轉出的變更標為待重新反映。這個標記與最終結論無關時，一次確認即可。

### 分期轉出：一份討論分幾刀立案

結論裡規劃「之後還要回到同一份記錄再轉出一個或多個變更」（例如把一件事切成幾刀依序立案）時，conclude 帶一次 `--hold`：

```
speclink discuss conclude <slug> --hold
```

- 結論照常寫入，記錄的 frontmatter 多一行 `hold: true`；狀態轉換與不帶旗標時相同。stdout 多一行告知記錄保留在途，這次 conclude 不會順手封存。
- 之後不帶 `--last` 的轉出（promote、`speclink new change --from-discussion`、seal）都不會清掉這個旗標。所以中間每一刀的變更封存時，這份討論都留在原地，封存輸出也不列它。
- 最後一刀在轉出時帶 `--last`。正常情況下這一步由 `/speclink-propose --from-discussion` 代勞：agent 讀結論 Decision 段的刀清單與記錄的轉出清單，判定這次立的是最後一刀就帶上，判定規則見[提案：建立變更與產物](propose.md)。帶 `--last` 的轉出在同一次寫入移除 `hold:` 行；變更名已經在轉出清單裡（重複轉出、重新反映後的 seal）也一樣移除；記錄本來就沒有 hold 行時什麼都不改。轉出指令的輸出不會提到 hold。
- 旗標只由三件事解除：不帶 `--hold` 的 conclude（會移除任何 `hold:` 行，手改的 `hold: false` 也一樣）、帶 `--last` 的轉出，或 `speclink discuss archive <slug>`。archive 無視旗標，照常封存。
- 帶過 `--last` 之後，最後一個轉出變更封存時（不論封存順序），記錄自動隨行封存，並列在封存輸出裡。不用再手動執行 `speclink discuss archive`。
- 忘了帶 `--last` 時，記錄留在途，看板的討論卡標「已轉出・保留中」。執行一次 `speclink discuss archive <slug>` 就收尾。
- 沒帶 `--hold` 的記錄，會在最後一個轉出變更封存時隨行封存；之後還要再切的刀，走新討論。

`speclink new change` 的 `--last` 必須與 `--from-discussion` 一起用。單獨帶 `--last` 時指令以錯誤結束，stderr 說明 `--last` 需要 `--from-discussion`，不建立任何檔案。

| 步驟 | 記錄的變化 |
| --- | --- |
| `conclude --hold` | 結論寫入，`hold: true` 出現 |
| `promote --name cut-a`，封存 cut-a | 轉出清單多 cut-a；旗標保留；記錄留在原地，封存輸出不列它 |
| `promote --name cut-b`，封存 cut-b | 轉出清單多 cut-b；旗標保留；記錄留在原地 |
| `promote --name cut-c --last` | 轉出清單多 cut-c；`hold:` 行消失 |
| 封存 cut-c | 記錄移入 `discussions/archive/`，封存輸出列它。全程不用執行 `speclink discuss archive` |

沒有 frontmatter 的舊記錄不能帶 `--hold`：指令以錯誤結束、記錄不動；不帶 `--hold` 照常結論。remote 模式的 `--hold`、`--last` 與旗標的行為都與本機相同。

> [!TIP]
> 討論被誤封存時，把記錄檔從 `openspec/discussions/archive/` 搬回 `openspec/discussions/` 就能繼續用。在不是最後一刀的變更上帶了 `--last`，記錄會在最後一個在途變更封存時被收走，也走這條路救回。引擎對已封存討論的轉出錯誤訊息也會指向這條路。

### link 與 seal 的細節

`speclink discuss link <slug> <變更名>`：

- 只寫變更側的來源鏈。討論記錄一個字都不動，保留旗標也照舊。
- open 與 concluded 的討論都可以連；已封存的討論不行。
- 同一組合重跑直接成功。
- 討論不存在、討論已封存、變更不存在時拒絕，兩邊檔案都不動。

`speclink discuss seal <slug> <變更名>`：

- 前提是鏈已經存在（經 link、promote 或建立變更時鑄成）。鏈不存在時拒絕，stderr 說明鏈未存在。
- 通過時討論狀態標記為 promoted，並記下這個變更名。
- 同時清掉該變更上對應這份討論的「待重新反映」標記。
- 帶 `--last` 時，同一次寫入移除記錄的 `hold:` 行（見上方「分期轉出」）。輸出與不帶時相同。

| 情境 | seal 的結果 |
| --- | --- |
| slug 沒有對應的討論 | 拒絕：討論不存在 |
| 討論已封存 | 拒絕：討論已封存 |
| 變更不存在 | 拒絕：變更不存在 |
| 變更沒有連到這份討論 | 拒絕：鏈未鑄妥 |
| 已經 seal 過 | 直接成功，不改檔 |
| 已經 seal 過，這次帶 `--last` | 成功；轉出清單不變，`hold:` 行移除 |

### 重新下結論

已轉出的討論再次 conclude 時，引擎把它轉出的每個進行中變更標為「待重新反映」，stdout 列出被標記的變更；已封存的變更跳過。這些變更要重新 ingest，再 seal 清掉標記。

### conclude 順手封存

conclude 時，如果這份討論轉出的變更全部已經封存、沒有任何進行中的變更引用它、而且這次寫入後記錄沒有 `hold: true`，引擎會在寫完結論後把記錄移到 `discussions/archive/`，stdout 多一行告知。帶 `--hold` 的 conclude 一律不封存。封存那一步失敗時，結論已經寫入、不回滾，指令以錯誤結束；之後執行 `speclink discuss archive <slug>` 收尾。判定「沒有進行中的變更引用」時，任何一個進行中變更的狀態檔壞掉，都當成它仍在引用，記錄留在原地。

## 改進討論：讓 agent 找題目

`/speclink-improve` 是反過來的討論：不是你帶題目，而是 agent 掃描 codebase 提出改進候選。只有你能發起，agent 不會自己觸發，過程中也不會實作程式碼。

流程六步：

1. 載入詞彙。
2. 防重提檢查：用範圍關鍵字執行 `speclink discuss search`，取得進行中與已封存討論裡 Ruled out 與結論的命中；閱讀時把同範圍的舊改進討論排在前面。已否決的方案不再提，除非說明當時的否決理由已經失效；讀進行中的變更，與它們重疊的區域不提。
3. 範圍收斂：你點名方向（模組、子系統、痛點）就直接採用；沒點名時用 git log 熱點推斷，近期常變的區域加權，並參考已封存變更的觸及記錄。熱點分散時放寬範圍。
4. 掃描：找六種摩擦訊號——理解一個概念要跳多個小模組、介面複雜度逼近實作、為了測試抽出純函式但 bug 藏在呼叫端、緊耦合跨界洩漏、難以透過現行介面測試，以及分組只靠檔名猜（同一概念的檔案散在平鋪目錄、讀者靠前綴或字母序重拼分組，或目錄與既有設計文件的分層對不上）。有機探索，不逐條打勾。前五種訊號的候選准入判準是刪除測試：刪掉之後複雜度集中才算，只是搬家不算。第六種改問讀者預測測試：第一次打開那個目錄的人，能不能不靠搜尋就猜到功能住哪、哪些檔案是一組。第六種的候選還要同時滿足三條件：分組有客觀依據（模組引用方向、既有設計文件的分層、對外連結）、對呼叫者不可見（既有路徑經根層 re-export 維持；無法 re-export 的面以守門測試斷言沒有殘留舊路徑）、搬移本身就是整個變更（每個呼叫端都要改路徑的只是搬家，不列）。一個候選同時命中前五種與第六種時，先用刪除測試裁決：複雜度會集中的算程式碼候選，第六種只涵蓋原樣搬移。
5. 建記錄：用 `--kind improve` 與 `--slug improve-<範圍>` 建討論，候選寫在 Round 1。每個候選有 Files、Problem、Solution、Wins、建議強度五欄；建議強度分三級：強烈建議、值得探索、尚屬臆測。結尾給首選建議，並問你想深入哪一個。
6. 收斂：沿用一次一題、提案帶證據的紀律，對每個被挑中的候選做介面深度檢查。第六種訊號的候選，四問改成：分組邊界的客觀來源是什麼、既有路徑靠什麼維持不變、搬移前盤點過哪些路徑風險（內嵌路徑的公開網址、依 tag 觸發或依路徑篩選的 CI workflow、以相對路徑寫的建置期檔案引入、跨套件的路徑相依）、回到平鋪目錄讀者會失去什麼。並附兩條做法：測試跟著原始碼的分組一起搬（內嵌測試超過行數門檻時搬到同名子檔）；切刀依路徑相依排序，先動目錄本身的刀、再動目錄內分組的刀，不用 worktree 平行做。

收斂走 conclude，再經 promote 或 link 轉成變更。結論的寫法與一般討論相同（見上方「結論怎麼寫」）：收斂轉出時，Decision 一句定論起頭、其後條列，分幾刀就每刀一個 bullet；Rejected alternatives 每個候選一行「方案——落敗理由」。結論規劃分期立案（先立一刀、封存後再回同一份記錄轉出下一刀）時，conclude 帶一次 `--hold`；最後一刀由 propose 轉出時帶 `--last`，之後最後一個轉出變更封存時記錄自動隨行封存；忘了帶 `--last` 就執行一次 `speclink discuss archive <slug>` 收尾；沒帶旗標的記錄在最後一個轉出變更封存時隨行封存，之後的刀走新討論（見上方「分期轉出」）。你全數否決時，仍然要 conclude 並封存，不能 discard：Decision 寫一句「不做」的定論即可、不強制條列，Rejected alternatives 每個候選一行寫方案與落敗理由。

## Remote 模式

`speclink discuss new` 的 `--slug`、`discard`（含 `--force`）、`link` 與 `seal` 在 remote 模式都可用，語意與本地相同。連到還沒升級的舊 server 時，這些指令會以錯誤訊息結束。

`speclink discuss search` 在 remote 模式也可用，結果的順序與每筆命中和本地相同。斷線、登入失效時的錯誤訊息，與其他 remote 讀取指令一致。

下一步：[提案：建立變更與產物](propose.md)。

**出處**：`discuss-skill`、`discussion-docs`、`improve-skill`、`user-documentation`
