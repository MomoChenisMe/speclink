---
topic: 內嵌宿主第八輪的三項需求
slug: embedded-host-needs-8
status: promoted
created: 2026-11-24
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: instructions-spec-language-and-tasks-pointer, render-git-history-option
---

# Discussion: 內嵌宿主第八輪的三項需求

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

內嵌宿主的 Agent 用真實模型照技能走完 discuss 與 propose 之後，回報三項需求：G1 規格的語言（`instructions specs` 的 `locale` 不看 `spec_locale`）、G2 任務指示指到 `dispatch` 拒絕的 `instructions --skill`、G3 `skills.render` 的 `checkout` 不能表示「讀得到程式碼、沒有 git」。輸入是一份文件，需求已經寫明現況、代價與要求，所以沒有先磨需求的階段，直接逐條對照程式碼分成「確認」「和程式碼不符」「要決定」三類。相關的正式規格：workflow-config、workflow-schemas、propose-skill、node-sdk、skill-routing、command-runtime；進行中的變更只有 rail-remote-connection-check，和這三項無關。
Source doc: ../speclink-embedded-host-needs-8.md
Prior discussions: embedded-host-needs, embedded-host-needs-7, tdd-switch-apply-wiring, skill-conversation-language, feature-provenance-skill

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-24)

**Focus**: 來源文件的三項主張對照程式碼後哪些成立，每一項要怎麼修
**Position**: 三項的問題都存在、都做，但 G1 的原因和文件寫的不同，G2 改文字就夠，G3 沒有觀察到的錯誤、只做最小的選項；使用者確認全部成立。
- 決策樹：G1（payload 的 `locale`、指示文字、技能文字）→ G2（括號的處理、其他指到 `dispatch` 不能跑的命令）→ G3（選項的形狀）→ 切法
- G1 確認：`instructions` 給每份產出物同一個 `locale`，規格也是（`crates/engine/speclink-core/src/workspace/instructions.rs` 的 `build_artifact`）；本機與內嵌是同一個函式，所以本機模式也不看 `spec_locale`
- G1 和文件不符：文件說指示只叫模型讀不存在的檔；實際上 `spec_locale` 解析成非英文時，引擎在指示最後補一段「This project sets `spec_locale: tw` — write spec prose in …」，從 v0.1.0 就有。用 SDK 的記憶體儲存實跑：`tw`、`auto`、`ja` 有這段，`en` 與沒設定時沒有；`auto` 時印的是解析後的代碼而不是 `auto`
- G1 的原因：同一份回應先講「English by default」、最後才講實際語言，技能文字又講一次「預設英文」，而 `locale` 欄位在 `locale: tw`、`spec_locale: en` 時和指示互相矛盾
- G1 決定：`instructions specs` 的 `locale` 改成規格的語言（`spec_locale` 是 tw／ja／en 用它，`auto` 跟著 locale，沒設定是英文），不另加欄位；指示拿掉 config.yaml 那段、一律寫出解析後的語言（英文也寫）；propose、ingest 技能拿掉「規格預設英文，除非 workflow-config show…」的例外。沒有程式讀這個欄位（搜尋過 apps、packages、crates），只有技能讀；舊技能配新引擎仍然正確
- G1 和正式規格衝突：propose-skill「artifact 的讀取與改寫一律經動詞」寫明規格語言指向 `workflow-config show --json` 的 `specLocale`，這一句要改；baseline 技能不改（沒有變更可以呼叫 `instructions`）
- G2 確認：任務指示在開了 tdd、audit 時寫「fetch … with `speclink instructions --skill …`」（`instructions.rs`），`dispatch` 以 `invalid_argv` 拒絕；拒絕寫在 node-sdk 的正式規格
- G2 和文件不符：訊息說文字「built into the CLI」，原文其實在引擎核心（`workspace/skills.rs` 的 `skill_body`）
- G2 決定：改寫那兩個括號，意思是「實作時 apply 會提供完整的紀律，寫任務不用去抓」；`dispatch` 不動。tdd.md、audit.md 是實作時的紀律，apply 技能自己會抓，而 apply 一定在有 CLI 的 checkout 執行
- G2 的盤點：宿主那六支技能（neutral、tool-call、specInCheckout false）全文沒有指到 `dispatch` 拒絕的五個命令；引擎訊息有一處，validate 在 delta 與正式規格衝突時每行結尾是「(see: speclink drift <change>)」（`lifecycle/archive.rs`）
- G3 確認：`checkout` 只接受布林值（node-sdk 正式規格）
- G3 和文件不符：六支技能裡只有 trace 有要執行的 git 步驟（前提、第 4 步、第 6 步）；文件記的錯誤（discuss 三次跳過讀程式碼）來自 `checkout: false` 這個不合適的值，改成不給 `checkout` 之後文件自己記錄沒有任何 git 呼叫，trace 沒有實際跑過；宿主的 Agent 沒有執行 git 的工具，所以 `git blame` 算錯 commit 在這個宿主不會發生
- G3 的後備：trace 第 4 步已有後備（改用討論與提案的內容）；第 6 步的關鍵字搜尋可以照做，`git log -S`、`git blame` 沒有替代，只能跳過並說明
- G3 決定：加選項 `gitHistory`（預設跟著 `checkout`；`checkout: false` 加 `gitHistory: true` 拋錯），false 時 `## Harness limits` 多一個項目，技能內文不改。這是推翻 embedded-host-needs-7「檔案搜尋與 git 歷史合成一個開關」：那時沒有宿主是「有程式碼、沒有 git」，現在有了
- 切法：兩個變更依序做，G1 加 G2 一個（都改 `instructions.rs`），G3 一個
**Ruled out**: G1 另加一個規格語言的欄位 — 模型仍要在兩個欄位之間選；G1 只改文字不改 `locale` — 同一份回應仍有兩個互相矛盾的語言；G2 直接刪掉括號 — 留著「audit checklist」卻不說來源，模型可能自己去找；G2 讓 `dispatch` 讀得到 `--skill` — 要改 node-sdk 正式規格、不經過 `beforeCommand`，換來的是寫任務用不到的文字，audit 原文還含獨立模式的步驟；G2 經 `dispatch` 時指示直接帶全文 — 引擎核心不知道呼叫的是誰，每次多約 18 KB；G3 `checkout: 'read-only'` — 重點是沒有 git 不是唯讀，而且要把布林改成聯合型別；G3 在 trace 內文加「不能跑 git 時」的判斷 — embedded-host-needs 已排除在技能裡做執行期判斷；G3 不做 — embedded-host-needs-7 排除「只補文件、宿主自己寫說明」的理由（用語和技能對不上）仍然成立
**Open**: 沒有待決的項目；G1、G2、G3 都在這一輪定案。validate 的「(see: speclink drift …)」這次不改，寫進結論的延後項目

## Conclusion

**Decision**: 三項都做，分成兩個變更依序做：G1 與 G2 合成一個（改 `instructions` 的回應），G3 一個（`skills.render` 多一個選項）。
- **cut 1 `instructions-spec-language-and-tasks-pointer`**: `instructions specs` 自己說清楚規格的語言，任務指示不再叫模型去抓 `dispatch` 拒絕的技能文字
  - `instructions specs` 的 `locale` 改成規格的語言：`spec_locale` 是 `tw`／`ja`／`en` 時用它，`auto` 時跟著 `locale`，沒有設定時是英文；其他產出物的 `locale` 不變；不另加欄位
  - 沿用現有的解析（`resolve_policy` 的 `spec_locale`，含環境變數 `SPECLINK_SPEC_LOCALE`），本機、遠端連線、內嵌宿主得到同一個結果
  - 規格的指示文字拿掉「English by default. If the project sets `spec_locale` in `speclink/config.yaml`…」那一段，改成一律寫出解析後的語言，英文也寫；「結構標記與 SHALL／MUST 維持英文」與中文模糊用語的提醒保留；不再印 `spec_locale: <代碼>`，所以 `auto` 時印出解析後代碼的小錯一併消失
  - propose、ingest 技能拿掉「規格預設英文，除非 `workflow-config show --json` 的 `specLocale`…」的例外，改成「`locale` 就是這份產出物的語言」；技能原稿有改，ASSET_VERSION、golden、assets.lock 一起更新
  - 要改寫 propose-skill「artifact 的讀取與改寫一律經動詞」裡「spec 的撰寫語言 SHALL 指向 speclink workflow-config show --json 的 specLocale」那一句與對應的情境；「技能全文不提及 config.yaml」維持
  - baseline 技能不改：它沒有變更可以呼叫 `instructions`，繼續讀 `workflow-config show --json` 的 `specLocale`
  - 相容性：沒有程式讀 `instructions` 的 `locale`，只有技能讀；舊技能配新引擎仍然正確，因為舊技能自己套用「規格預設英文」的例外
  - 任務指示在開了 tdd、audit 時的兩個括號（「fetch the full discipline with `speclink instructions --skill tdd`」「fetch it with `speclink instructions --skill audit`」）改寫成：實作時 apply 會提供完整的紀律，寫任務不用去抓；一句摘要保留；確切的英文用語提案時定
  - `dispatch` 照舊拒絕帶 `--skill` 的 `instructions`（node-sdk 的正式規格不改）；拒絕訊息「built into the CLI」不精確（原文在引擎核心），提案時決定要不要順手改
  - 例子（`locale: tw`）：`spec_locale` 沒設定時 `instructions specs --json` 的 `locale` 是 `"English"`、指示寫明用英文；`spec_locale: auto` 時是 `"Traditional Chinese (繁體中文)"`；`spec_locale: ja` 時是 `"Japanese (日本語)"`；`spec_locale: en` 時是 `"English"`；四種情況下 `instructions proposal --json` 的 `locale` 都是 `"Traditional Chinese (繁體中文)"`
- **cut 2 `render-git-history-option`**: `skills.render` 多一個選項 `gitHistory`，表示「讀得到程式碼、不能用 git」
  - `gitHistory` 是布林值；省略時跟著 `checkout`（`checkout` 是 true 時為 true，false 時為 false）；`checkout: false` 加 `gitHistory: true` 拋錯，和 `specInCheckout: true` 的規則一樣；不是布林值時拋錯
  - `gitHistory: false` 而 `checkout` 不是 false 時，`## Harness limits` 多一個項目：可以讀與搜尋程式碼，但沒有 git 歷史、不能執行 git；跳過執行 git 的步驟；步驟寫了其他做法時改用它；在回答裡說明跳過了什麼
  - `checkout: false` 的項目文字不變（它已經涵蓋 git）；技能內文不改，CLI 產生的技能檔逐位元不變，ASSET_VERSION 不動
  - 這是推翻 embedded-host-needs-7「檔案搜尋與 git 歷史合成 `checkout` 一個開關」：那時沒有「有程式碼、沒有 git」的宿主，現在有了，而 `checkout: false` 會讓模型連程式碼都不讀
  - 影響的技能：宿主常用的六支（discuss、trace、propose、ingest、analyze、config）裡只有 trace 有要執行的 git 步驟。第 4 步（evidence 是 null 時 `git log`）已有後備，改用討論與提案的內容；第 6 步的關鍵字搜尋照做，`git log -S`、`git blame` 沒有替代，跳過並說明只根據現在的程式碼回答
  - 要改 node-sdk 說明 `skills`、`checkout` 兩個選項的那條需求，以及 `index.d.ts`、npm 套件的 README 與 sdk-node 文件
  - 例子：`skills.render('trace', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, gitHistory: false, skills: ['discuss', 'trace'] })` 的 `## Harness limits` 有兩個項目（技能清單、不能用 git）；`{ checkout: false, gitHistory: true }` 拋錯；`{ checkout: false }` 的輸出和現在逐位元相同
- 順序：先 cut 1 再 cut 2；兩個彼此獨立，但都動技能相關的檔，不平行做
- 來源文件「這一輪確認足夠的部分」不要求改動，沒有逐條再驗證
**Rationale**: 取捨的主軸是「引擎說一次、說對」：G1 的錯不是缺資訊，而是同一份回應裡先講預設英文、最後才講實際語言，欄位又和指示矛盾，所以把解析後的結果直接放進欄位與指示，技能不再自己查。G2 的紀律全文是實作時用的，由一定有 CLI 的 apply 取得，所以改文字比打開 `dispatch` 小，也不動正式規格對 `dispatch` 的界線。G3 沒有觀察到的錯誤（錯誤來自選了不合適的 `checkout: false`），所以照上一輪的做法只加前言的一個項目，不改技能內文。
**Rejected alternatives**:
- G1 另加一個規格語言的欄位 — 模型仍要在兩個欄位之間選，這次的錯會留著
- G1 只改指示文字、不改 `locale` — `locale: tw`、`spec_locale: en` 時同一份回應仍有兩個互相矛盾的語言
- G1 技能繼續指向 `workflow-config show` — 它不套用環境變數，和指示可能給出不同的語言；tdd-switch-apply-wiring 已排除「技能自己查政策」
- G2 直接刪掉括號 — 留著「audit checklist」卻不說來源，模型可能自己去找
- G2 讓 `dispatch` 讀得到 `instructions --skill` — 要改 node-sdk 正式規格、不經過 `beforeCommand`，換來的是寫任務用不到的文字，audit 原文還含獨立模式的步驟
- G2 經 `dispatch` 時指示直接帶紀律全文 — 引擎核心不知道呼叫的是誰，每次任務指示多約 18 KB
- G3 `checkout: 'read-only'` 三種值 — 重點是沒有 git 不是唯讀，而且要把布林改成聯合型別
- G3 在 trace 內文加「不能跑 git 時」的判斷 — embedded-host-needs 已排除在技能裡做執行期判斷
- G3 不做、宿主繼續用自己的 system prompt — embedded-host-needs-7 排除「只補文件」的理由（用語和技能對不上）仍然成立
- 三項合成一個變更 — G3 改的是 node-sdk 的選項，和 `instructions` 的回應無關，放在一起會分散審查的焦點
**Deferred**:
- validate 在 delta 與正式規格衝突時每行結尾的「(see: speclink drift <change>)」經 `dispatch` 會被拒絕 — drift 的拒絕訊息已說明原因，修法（ingest）技能已經知道，目前沒有觀察到問題
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion embedded-host-needs-8
