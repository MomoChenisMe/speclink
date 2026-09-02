---
title: 工作流政策與設定
section: SDD 工作流
order: 210
keywords: [config.yaml, 政策, locale, tdd, audit, 專案說明, 產出規則]
sources: [workflow-config, config-skill]
generated: 2026-09-02
---
# 工作流政策與設定

工作流政策存在 `openspec/config.yaml`。這個檔有三類內容：五個政策欄位、專案說明，以及產出規則。你可以用 `speclink workflow-config` 指令讀寫，也可以請 `/speclink-config` 技能幫你整理專案說明與產出規則。

## 五個政策欄位

| 欄位 | 合法值 | 未設定時 |
| --- | --- | --- |
| locale | tw、ja、en | English |
| spec_locale | tw、ja、en、auto | 未設定 |
| tdd | true、false | false |
| audit | true、false | false |
| worktree | true、false | false |

值的比對區分大小寫。TW、Auto 都不合法。顯示名稱（例如「繁體中文」）也不合法，要寫代碼 tw。

`speclink init` 產生的 config.yaml 範本已含這五個欄位的註解示例，並列出對應的環境變數名。`.speclink.yaml` 不放政策鍵。

### 有效值怎麼決定

有效值依下列順序解析，先命中者勝：

1. 環境變數：SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE。
2. `openspec/config.yaml` 的值。
3. 內建預設。

布林環境變數只接受 true 或 false。其他值視為未設定，落到下一層，不報錯。`.speclink.yaml` 裡的同名鍵一律不生效，也不會有警告。

> [!WARNING]
> `openspec/config.yaml` 存在但無法解析（YAML 語法錯誤或型別不符）時，所有讀政策的指令都會失敗。錯誤訊息指出檔案路徑與解析原因。指令不會退回預設值繼續跑，設環境變數也繞不過。檔案不存在時才用內建預設。

## 讀設定：workflow-config show

```
speclink workflow-config show
```

顯示 config.yaml 裡的正典內容：五個政策欄位（未設定的會標示未設定與其預設語意）、專案說明的有無與行數、各產出規則節的條數。它顯示的是檔案裡的值，不套用環境變數覆寫。remote 模式下讀的是 server 上的設定，輸出形狀相同。

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

## 寫專案說明與產出規則

```
speclink workflow-config context --stdin
speclink workflow-config rules <artifact> --stdin
```

- context 以 stdin 全文設定專案說明。stdin 只有空白時移除該鍵。
- rules 整節代換指定 artifact 的規則清單：stdin 一行一條，空行忽略。stdin 為空時移除該節。artifact 只能是目前產出流程裡的 artifact id，例如 proposal、specs、design、tasks。未知 id 被拒絕。

兩個指令沒帶 `--stdin` 時失敗並說明用法。保留其他行、`--dry-run`、remote 的版本檢查、壞檔拒寫等行為，與 set 相同。

## 讓技能幫你整理：/speclink-config

`/speclink-config` 技能從固定的來源推導專案說明與產出規則：workspace 清單與相依 manifest、README、docs 索引、既有的 config.yaml，以及共用詞彙（若有）。它不掃整個 repo 的原始碼。

技能寫內容時守四條判準：

1. 引擎已自動注入的內容，以及品質關卡技能已承載的標準，不重述。判定方式是實際比對指令輸出與生成的技能檔，不憑印象。
2. 只對單一 artifact 有用的內容歸產出規則，不入專案說明。
3. 會過時的內容不寫，例如版本號、計數、統計數字。
4. 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行都核實。核實只用靜態手段，不執行被引用的測試或建置指令。

既有的規則只在兩種情況下被刪：不過四條判準，或你在政策詢問中明確撤回。「無法從固定來源導出」不是刪除理由。

技能會逐項問你五件事，不自己推斷：locale、spec_locale、tdd、audit，以及任務驗證步驟要跑全量測試還是只跑受影響面。你用自然語言回答（例如「繁體中文」），技能換成代碼（tw）再寫入。第五問答「只跑受影響面」時，技能依 repo 的相依 manifest 組出這個專案專用的測試對應規則，寫進 tasks 的產出規則。答「全量」時不寫任何測試範圍規則；既有的測試範圍規則視為撤回而被移除。

> [!TIP]
> 技能一律先用 `--dry-run` 產出差異給你看，你確認後才寫入。對同一份沒動過的程式碼連跑兩次，第二次的差異應該是空的。第二次還有差異，表示判準執行不當，技能會回查判準而不是落檔。

你也可以帶範圍提示呼叫技能。此時前三條判準只重審範圍內的 artifacts，第四條的引用核實仍掃全文件。

**出處**：`workflow-config`、`config-skill`
