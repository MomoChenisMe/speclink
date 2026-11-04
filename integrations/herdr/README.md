# speclink：Speclink 的 Herdr mod

[English](README.en.md)

在 [Herdr](https://herdr.dev) 裡開一個面板，看目前專案的變更與討論，按一下就把技能交給對的 agent、開工，或叫 Speclink 開詳情。全程不用手打變更名稱、不用找視窗。

## 面板有什麼

面板有兩種放法。兩種都不搶鍵盤，專案都跟著目前焦點所在的終端機走，切 space 就換專案。

- **停靠**：用複製出來的 `speclink` 版面（見「安裝」），面板常駐在視窗最右邊一欄，和終端機並排。`⌘⇧S` 開關這一欄，拖動邊界可以改寬度。每個 space 各自記住開或關，切 space 會還原，還沒記過的 space 預設打開，所以用不到 Speclink 的專案關一次就好。
- **彈出**：其他版面按 `⌘⇧S` 開關面板，面板是一個可以拖動的浮動視窗。

面板的內容：

- **頂端**：專案名稱，下面一行寫「目前的 space · 分支」。
- **變更**：提案中、進行中、已就緒三張卡常駐，標題有階段顏色的圓點和數量。每一列有波次、名稱、worktree 標記、任務進度。被擋住的列會變淡，並寫出在等誰。
- **討論**：討論卡與已轉出卡。已轉出但還有刀沒轉出的討論留在討論卡，因為下一步還要 propose。
- **做這個變更的 agent**：列上顯示 agent 的位置與狀態，點了切過去。狀態和左邊 Agents 清單相同：做事中是轉動的圓弧，閒置是空心圈，做完是實心點，等你回答是一顆膠囊。比對順序是 `speclink.change` 標籤、worktree 路徑、Claude Code 外掛設的終端機標題。
- **技能鈕**：每列只留建議的下一步。`⋯` 開技能跳窗，並先選好這個技能和這一列。滑鼠停在鈕上會寫出會送給誰。
- **附加鈕**：名稱旁邊的對話圖示，把變更或討論的名稱接到目前分頁的輸入框，不送出。目前分頁沒有 agent，或 agent 在等你回答時，這顆鈕會變灰。
- **開工**：變更列上的鈕，為這個變更開一個專用的新 agent。見下面的「開工」。
- **技能**：頂端的鈕或 `⌘⇧N` 開跳窗，選技能和搭配的變更或討論。見下面的「技能跳窗」。
- **叫 Speclink 開頁面**：點變更列或討論列開它的詳情頁，點列首的波次數字或頂端的「路線圖」開路線圖。

每張卡可以收合，超過 5 列會摺疊成「還有 N 個…」，每一列都有複製鈕。

### 不是 Speclink 專案時

目前的 space 不是 Speclink 專案時，面板顯示一張提示卡，寫出專案資料夾和「初始化」鈕。沒有「路線圖」和「技能」，因為這些功能在這裡還用不了。

按「初始化」會開一個新分頁，在專案資料夾執行 `speclink init`。它會逐題問要產生哪些 agent 的檔案，答完才寫入。不想初始化就關掉那個分頁。初始化完成後，面板在 10 秒內換成變更清單。

只有「沒有初始化」會顯示這張卡。其他問題（例如找不到 `speclink`、設定檔壞掉）仍然顯示紅字。

### 各階段的技能

| 狀態 | 能用的技能（粗體是建議的下一步，也是列上顯示的那一顆） |
| --- | --- |
| 提案中，沒被擋 | **apply**、analyze、ingest |
| 提案中，被擋 | apply（灰掉，列上顯示這一顆）、analyze、ingest |
| 進行中 | **apply**、ingest、drift、verify |
| 已就緒，沒有 worktree | **quality**、review、verify、archive、commit |
| 已就緒，有 worktree | **quality**、review、verify、commit、worktree-merge |
| 討論重新定案，變更還沒更新 | 建議改成 **ingest** |
| 討論中 | **discuss** |
| 已定案，或還有刀沒轉出 | **propose --from-discussion**、discuss |

### 技能會送給誰

依序找，找到就停：

1. 正在做這個變更的 agent。
2. 目前分頁的 agent。
3. 都沒有就開新分頁，跑最常用且已安裝的 agent。

每一步都會檢查 agent 所在的資料夾。有 worktree 的變更要在 worktree 裡跑技能；archive、worktree-merge、apply-with-worktree 要在主資料夾跑。agent 在別的資料夾就換下一步。有 worktree 但裡面沒有 agent 時，apply 鈕會改送 apply-with-worktree，接續原本的 worktree。

agent 在做事、在等你回答或出錯時不送，按鈕變灰，滑鼠停在按鈕上會寫出原因。送出後畫面切到那個 agent。

Claude Code 與 GitHub Copilot 送 `/speclink-<技能>`，Codex 送 `$speclink-<技能>`。analyze 只有 Claude Code 有，送給其他 agent 時不顯示。其他 agent 沒有 speclink 技能，不會當送達對象。

### 開工

開工和技能鈕不同：技能鈕優先交給現有的 agent，開工一定開新的 agent。

| 工作流設定的 `worktree` | 按下開工 |
| --- | --- |
| 開啟 | 開一個新 space，跑 `/speclink-apply-with-worktree <變更>`。space 的名稱維持 Herdr 原樣，分頁名稱會寫出變更 |
| 關閉 | 在主資料夾的 space 開新分頁，跑 `/speclink-apply <變更>` |

沒被擋、任務還沒做完、也沒有 agent 在做的變更才有開工鈕。討論重新定案的變更要先 ingest，所以也沒有。

**整波開工**：`worktree` 開啟，而且最前面那一波有兩個以上能開工的變更時，面板頂端會多一列，寫出會開哪些變更。按一次就全部開工，不再多問。波次取自 `speclink plan`。

### 技能跳窗

按頂端的「技能」或 `⌘⇧N` 開跳窗，再按一次或按 `Esc` 關閉。所有版面都能用，面板那一欄收起來也一樣。

- **左邊**：所有技能，分成規劃、實作、品質、收尾、專案五組。用 `↑` `↓` 或滑鼠選。右邊的數字是現在有幾個變更或討論可以搭配，沒有變更可搭配的技能會變淡。「專案」組用在整個專案，不搭配東西，所以沒有數字。
- **右邊**：這個技能能搭配的變更或討論，規則和技能鈕相同。用 `←` `→` 或滑鼠選，也可以不指定。不搭配變更的技能（discuss、propose 和專案組）有輸入框，可以打主題或補充說明。
- **底部**：寫出會打進去的指令，或為什麼現在不能用。

按 Enter 有兩種結果，由「開新分頁」決定（`⌘O` 或滑鼠勾選，每次開跳窗都是沒勾）：

- **沒勾：放到輸入框。** 把指令放進目前分頁 agent 的輸入框，不送出。你可以接著打字，再自己按 Enter。agent 在做事也可以放；在等你回答時不放，因為打字可能答掉對話框。
- **有勾：開新分頁並開工。** 做法和 `herdr-dispatch` 技能相同：用 `hc run` 在目前的 space 開新分頁，分頁名稱是「技能 · 變更」，畫面留在原分頁。

有勾而且技能搭配變更時，可以一次開多個，一個變更一個分頁。用 `←` `→` 移動、`Space` 勾選，`⌘A` 或「全選」勾全部。這些變更不能勾，那一列會寫原因：已經有 agent、被別的變更擋住（apply）、主資料夾已經有 apply 在跑、專案沒開 worktree 流程（apply-with-worktree）。同時勾兩個都要在主資料夾跑的 apply 時，底部會請你改選 apply-with-worktree。

### 叫 Speclink 開頁面

面板用 `open` 送出 `speclink://` 網址，桌面 app 會切到這個專案再開頁面。app 沒開時會自動啟動。網址只能開啟與檢視，不會寫入任何檔案。

## 需求

- Herdr 0.1.12 以上（以這一版測試）。
- `speclink` CLI 在 PATH 上，版本要有 `plan --json` 的 `unpromotedCuts`。面板只讀 `speclink list`、`speclink plan`、`speclink discuss list`、`speclink workflow-config show` 的 `--json` 輸出。
- 專案執行過 `speclink init`。沒有的話，面板會給「初始化」鈕。
- 要叫 Speclink 開頁面時，桌面 app 要認得 `speclink://` 網址（v0.11.0 之後的版本）。
- 要停靠時需要 Node.js，用來執行複製版面的腳本。

## 安裝

在 repo 根目錄操作，Herdr 要開著：

1. 複製 mod。Herdr 會自動載入。

   ```bash
   cp -R integrations/herdr/mods/speclink integrations/herdr/mods/speclink-new ~/.config/herdr-app/mods/
   ```

2. 在 `~/.config/herdr-app/keys.json` 的 `global` 加上前兩行，再加上 `layout:speclink` 這一段（內容和 [keys.json](keys.json) 相同）。`⌘⇧N` 開技能跳窗。在 `speclink` 版面，`⌘⇧S` 開關右邊那一欄；在其他版面，`⌘⇧S` 彈出浮動視窗。

   ```json
   "cmd-shift-s": "mod.open:speclink",
   "cmd-shift-n": "mod.open:speclink-new"
   ```

   ```json
   "layout:speclink": { "cmd-shift-s": "layout.toggle:speclink" }
   ```

3. 要停靠就執行這一行。它把 Herdr 的 `main` 版面複製成 `speclink` 版面，在最右邊加一欄面板，然後切過去。不想停靠就跳過這一步，用 `⌘⇧S` 彈出。

   ```bash
   node scripts/herdr/dock-panel.mjs
   ```

用 `⌘⇧L` 切換版面。更新 mod 時再執行一次 `cp`，Herdr 會立刻重新載入。

不要用符號連結安裝：Herdr 載入得到程式，但畫不出連結裡的 SVG 圖示，存檔也不會自動重新載入。

### Herdr 更新後重做停靠

複製版不會跟著 Herdr 更新：Herdr 的 `main` 版面有新東西時，`speclink` 版面不會有。`~/.config/herdr-app/layouts/speclink.json` 的 `cloned_from.version` 是複製時的 Herdr 版本，和 `herdr-app --version` 不同時就該重做。重做是再執行一次同一行：

```bash
node scripts/herdr/dock-panel.mjs
```

它會做這些事：

1. 切回 `main`。
2. 把舊的複製版改名成 `layouts/speclink.json.old-<時間>`。
3. 用新版的 `main` 重新複製，加上面板，切到新的 `speclink` 版面。

重做後，面板那一欄回到預設寬度，其他欄寬和開關狀態和 `main` 相同，每個 space 記住的開關不變。你手改過舊版的話，內容還在 `.old` 檔裡，要自己搬過來。確認沒問題後可以刪掉 `.old` 檔。

### 移除

1. 用 `⌘⇧L` 切回 `main`。
2. 刪掉 `~/.config/herdr-app/layouts/speclink.json`、`~/.config/herdr-app/mods/speclink` 和 `~/.config/herdr-app/mods/speclink-new`。
3. 刪掉 `keys.json` 裡加的那幾行。

### 為什麼用複製的版面

討論 `herdr-speclink-panel` 的結論否決過「複製 speclink 專用版面」，理由是複製版不會跟著 Herdr 更新。2026-11-04 看過實際畫面後推翻這個否決，接受複製版的維護代價：浮動視窗蓋住終端機，和定案的右邊一欄差太多。

Herdr 內建的 `main` 版面不能停靠：`mod.json` 寫 `"place": "right"` 沒有作用，對 `main` 跑 `layout.set` 也會被拒。

## 已知限制

- **複製版不會跟著 Herdr 更新**：`speclink` 版面停在複製當下的 `main`。Herdr 更新後要重做，方法見「Herdr 更新後重做停靠」。
- **不顯示 review、verify 的章**：CLI 的輸出沒有蓋章狀態。已就緒的變更蓋完章後，建議仍停在 quality，archive 要自己按。
- **「已轉出」是粗分**：看 `discuss list` 的 `status`，以及 `plan` 有沒有未轉出的刀。
- **Codex 只查證了寫法**：叫用寫法取自 speclink 的文件與技能輸出，還沒有實際送給 Codex 測過。
- **放到輸入框靠兩種做法**：Herdr 沒有「只貼上不按 Enter」的指令。英數指令用 `hc keys` 一個字一個字打。有中文或大寫時用 `hc send` 貼上，並在結尾加 `\`：Claude Code 會把 `\` 加 Enter 當成換行，不會送出，所以輸入框最後會多一個空行。按 Enter 一樣會送出，指令也一樣能跑。如果 Claude Code 改掉這個行為，這段文字會被直接送出。
- **Codex 的分頁放不進技能指令**：`hc keys` 打不出 Codex 的 `$`，上面的反斜線做法也沒有在 Codex 驗證過。Copilot 只能放不含中文的指令。這些情況要勾選「開新分頁」。附加鈕只放名稱，沒有 `$`，所以 Codex 也能用，但中文名稱一樣放不進去。
- **一次開多個分頁可能撞 git 鎖**：和整波開工相同，撞到時那個 agent 會停下來回報。
- **按鈕和資料夾的名稱不一樣**：按鈕叫「技能」，資料夾還是 `speclink-new`，這樣 `keys.json` 不用改。
- **技能鈕不看專案裝了哪些技能**：例如專案沒有 Codex 的技能檔時，鈕還是會送 `$speclink-…`。
- **開頁面只有 macOS**：面板用 `open`，其他系統會跳通知說開不了。專案在別台機器的 space 不能開，所以那些列點了沒有反應，也沒有「路線圖」鈕。
- **不看 `SPECLINK_WORKTREE` 環境變數**：面板只讀工作流設定。環境變數把 worktree 關掉時，開工一樣開新 space，技能會自己停下來並說明。
- **開工 space 可能還在主資料夾**：agent 用 `cd` 進 worktree，終端機回報的資料夾不一定跟著變。這時 space 的名稱是專案名，也不會縮排在專案底下，要看分頁名稱才分得出變更。
- **第一次開的資料夾會先問信任**：Claude Code 問要不要信任資料夾時還不算 agent，面板看不到它，15 秒後開工鈕會再出現。信任過的資料夾不會問。
- **整波開工可能撞 git 鎖**：每個 agent 都可能在主資料夾 commit 規格或建 worktree。撞到時那個 agent 會停下來回報，再按一次技能鈕即可。
- **畫面只有繁體中文**。

## 開發

```bash
node --test scripts/herdr/panel-model.test.mjs
herdr-app check ~/.config/herdr-app/mods/speclink
herdr-app check ~/.config/herdr-app/mods/speclink-new
```

`model.js` 是純邏輯：資料合併、技能表、送達決策，測試直接跑它。`main.js` 是面板畫面與 Herdr API，`skill-picker.js` 是技能跳窗（選擇的畫面，放到輸入框和開新分頁的規則在 `model.js`），`column-sync.js` 讓右邊那一欄每個 space 各自記住開關。它放在 `dock-panel.mjs` 加的隱形浮動區塊裡，因為那一欄收起來後面板不會重畫。`speclink-new` 只有設定：一個 mod 只能設定一種 overlay，Herdr 也不讓 mod 引用別的資料夾的程式，所以用它開出 `speclink` mod 的跳窗畫面。`scripts/herdr/panel-model.test.mjs` 也拿技能表對照 `.claude/skills/speclink-*`：speclink 新增或移除技能時，這個測試會提醒你同步。
