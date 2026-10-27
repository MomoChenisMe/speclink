---
topic: 讓 discuss 與 propose 能以使用者提供的 plan／docs 文件為輸入進行討論與建立提案
slug: discuss-propose-from-docs
status: promoted
promoted_to: discuss-propose-from-docs
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 讓 discuss 與 propose 能以使用者提供的 plan／docs 文件為輸入進行討論與建立提案

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者看到 mattpocock 的 grill-with-docs，想讓 discuss 能讀取 plan／docs 文件進行討論。事實查核發現 grill-with-docs 的本體是「輸出側」紙本痕跡（CONTEXT.md 詞彙表＋ADR），speclink 已有對應物（討論記錄、LANGUAGE.md、design.md）；真正的缺口是「輸入側」——使用者自寫的 markdown 或 plan mode 產出的規劃文件，交給 discuss 對 codebase 逐條評估可行性與漏洞。討論中範圍擴大：propose 也要能讀文件（from-discussion 跟隨記錄中引用的文件；以及不經 discuss 直接指定文件建立提案）。模式選 assumptions：三處 discuss skill 實例、propose skill 的 plan 檔邏輯（步驟 1c）、discuss.rs 脈絡足夠。前案 discuss-decision-tree-interview（已封存）確立的決策樹紀律是本次的基礎。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: grill-with-docs 可借鏡什麼？文件在 discuss 中的角色（樹的根節點）
**Position**: 輸入側才是缺口，文件＝「別人寫的 assumptions 清單」走預填樹逐條 stress-test——使用者確認並擴大範圍至 propose：
- 事實查核：grill-with-docs 本體是輸出側（詞彙表＋ADR），speclink 已有對應（討論記錄／LANGUAGE.md／design.md）；輸入側在其原文僅一句「codebase 查得到的自己查」
- 文件角色（使用者確認）：自寫 markdown 或 plan mode 文件，discuss 依 codebase 逐條評估可行性與漏洞——萃取主張當樹節點，分診成證實（附程式碼證據）／牴觸（文件說 X 程式碼是 Y）／真決策（送使用者裁定）
- 記錄面（使用者確認）：討論時明確引用文件位置；討論記錄只存討論結果，不內嵌整份規劃文件
- 入口：topic 自由指定路徑（plan 檔、repo docs、任意可讀路徑），不加旗標
- 落地：純 skill 文字，引擎零改動
- 範圍擴大（使用者本輪提出）：propose 側兩需求——from-discussion 時若記錄引用了文件，propose 須讀原始文件再疊加討論決策；discuss 非必要，propose 也要能不經討論直接指定文件建立提案
**Ruled out**: 文件僅作背景素材開場讀一次——文件與程式碼的矛盾不會被逐條抓出，stress-test 效果大減；採用 inline 詞彙落檔——LANGUAGE.md 是正典詞彙，討論中途落檔會把未收斂的詞燒進正典，輪記錄本來就留有痕跡；新旗標／引擎記 provenance——沿用引擎零改動原則
**Open**: propose 疊加語意（文件為底層、討論決策為勝出層？Ruled out 是否抑制文件內容）；propose 直接吃文件的入口語法；討論記錄中文件引用的固定慣例（讓 propose 可識別）；一個 change 還是拆兩個

### Round 2 — assumptions (2026-07-31)

**Focus**: propose 的疊加語意（P1）與三個次要節點（P2 入口、P3 記錄慣例、P4 範圍）
**Position**: 文件為底層、討論為勝出層——使用者以具體例確認三條規則：
- 討論有決定的 → 討論贏（較新、經 stress-test）
- 討論沒碰到的 → 文件內容照用（「不用改原始文件」因此成立）
- 討論 Ruled out 的 → propose 不得復活（例：plan 寫 SSE＋錯誤重試三次；討論否決 SSE 改 WebSocket、未碰重試 → 提案＝WebSocket＋重試三次、SSE 不出現）
- P2（無異議通過）：propose 認得 --from-doc <路徑> 引數慣例，仿 --from-discussion 的 skill 文字約定、非引擎旗標；與既有 ~/.claude/plans/ plan 檔偵測（步驟 1c）並存
- P3（無異議通過）：discuss 的 Context 固定寫一行 Source doc: <路徑>，輪 Evidence 引段落標題或短句、絕不內嵌整份文件；propose --from-discussion 以此行機械識別「有原始文件要讀」
- P4（無異議通過）：一個 change 落地——P3 寫與 P1 讀是同一條慣例，拆開會分家漂移
**Open**: none——樹走完

## Conclusion

**Decision**: discuss 與 propose 兩技能擴充「文件輸入」能力，純 skill 文字、引擎零改動。（1）discuss：topic 可自由指定文件（自寫 markdown、plan mode 產出、repo docs、任意可讀路徑）；文件視為「別人寫的 assumptions 清單」走預填樹——萃取主張當樹節點，對 codebase 逐條分診成證實（附程式碼證據）／牴觸（文件說 X 程式碼是 Y）／真決策（送使用者裁定）；記錄面：Context 固定寫一行 Source doc: <路徑>，輪 Evidence 引用段落標題或短句，記錄只存討論結果、不內嵌整份規劃文件。（2）propose --from-discussion：記錄含 Source doc 行時 SHALL 讀取原始文件，疊加語意＝文件為底層、討論為勝出層——討論有決定的以討論為準、討論未碰的文件內容補位、討論 Ruled out 的內容不得復活。（3）propose 直接文件入口：認得 --from-doc <路徑> 引數慣例（仿 --from-discussion 的 skill 文字約定），供不經 discuss 直接以自備 plan 文件建立提案；與既有 ~/.claude/plans/ plan 檔偵測並存。
**Rationale**: grill-with-docs 的本體是輸出側紙本痕跡（詞彙表＋ADR），speclink 已有對應（討論記錄／LANGUAGE.md／design.md）；真缺口是輸入側。文件必須當預填樹來源而非背景素材，才能逐條抓出文件與 codebase 的矛盾；「底層／勝出層」疊加語意讓討論不必回改原始文件——記錄只存決策差分，合成是 propose 的責任。
**Rejected alternatives**: 文件僅作背景素材開場讀一次——矛盾不被逐條抓出，stress-test 失效；inline 詞彙落檔（grill-with-docs 的做法）——未收斂的詞會燒進正典 LANGUAGE.md，輪記錄本已留痕；引擎旗標或 frontmatter 記 provenance——純文字約定即可達成，維持引擎零改動；拆成兩個 change——P3 寫方與 P1 讀方是同一條慣例，分家會漂移。
**Deferred**: none
**Capture to**: 兩技能檔各三處實例（crates/speclink-core/assets/skills/discuss.md、crates/speclink-core/assets/skills/propose.md、.claude/skills/speclink-discuss/SKILL.md、.claude/skills/speclink-propose/SKILL.md、.agents/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-propose/SKILL.md）＋乾淨樹再生 render golden——經變更落地
**Next**: speclink discuss promote discuss-propose-from-docs
