## Context

前刀 desktop-reading-experience 已把 markdown 字內排版交給 @tailwindcss/typography（prose 16px、remark-breaks、skipHtml），但三個文件檢視面仍以「單一連續 markdown 流」呈現：討論輪（DiscussionDrawer 討論過程分頁、ArchivedList 討論檢視）、變更文件四分頁（RichDetailDrawer）、已封存變更檢視（ArchivedList）。討論記錄與 delta 規格都有 CLI scaffold 保證的固定結構（Round 標題、粗體欄位前綴、delta 區段標題），前端可靠切分。文字牆的內容側成因在 discuss 技能的 add-round 範本——Position 慣性寫成單行數百字。

本刀無 Rust 程式邏輯變更：speclink-core／speclink-cli 的指令行為、人眼輸出與 --json 不動；speclink-core 只有 assets/skills/discuss.md 內容更新與 render golden 快照再生。

## Goals / Non-Goals

**Goals:**

- 討論輪以卡片呈現：輪與輪有視覺邊界，欄位有標籤，新舊記錄一體適用。
- 討論結論以欄位標籤呈現：conclude scaffold 的六欄位拆標籤區塊，與輪同型（實作驗收後追加）。
- 抽屜 markdown 文件有行寬上限與容器留白，全螢幕不失控。
- 變更抽屜與已封存檢視的規格分頁不直出 delta 機器標記，區段以色標呈現。
- add-round 範本引導 Position 列點多行，新記錄從源頭不再產生單行長文。

**Non-Goals:**

- requirement 逐條卡片與 scenario 展開（討論已否決——範圍膨脹）。
- 背景分頁結構化——context 是自由散文、無固定欄位可解析，文件容器已涵蓋。
- 任務分頁（desktop-task-interactions 刀）與左側導覽規格頁功能（desktop-specs-view 刀）。
- 回寫既有討論記錄；變更 CLI 任何指令行為或輸出。
- markdown 渲染引擎更換（react-markdown 續用）。

## Decisions

### D1 輪卡片切分：前端行掃描解析 scaffold，不動後端

沿用 splitDiscussionSections 的行掃描手法，在 packages/ui 新增輪切分 helper：以 scaffold 固定標題格式「### Round <N> — <mode> (<date>)」為卡片邊界，解析出輪次、mode、日期與卡身內文。任何一輪標題不合格式（手寫記錄、pre-scaffold 格式）即整篇退回單一 Markdown 檢視——與現行 sections fallback 同一層級、同一策略。

替代方案：
- react-markdown AST visitor（rehype plugin）切卡——耦合渲染管線、fallback 邏輯難做整篇退回，否決。
- Rust 端提供結構化 rounds JSON——跨層改動大、舊記錄仍需前端 fallback、且「呈現歸前端」是現行分工，否決。

### D2 欄位標籤區塊：行首粗體前綴解析，缺欄位不渲染

卡身內文按行首「**<Label>**:」前綴切欄位，Label 僅認 Focus／Position／Ruled out／Open 四詞（scaffold 固定詞彙）；一個欄位涵蓋其標籤行起至下一個標籤行（或輪結尾）的所有行——Position 底下的列點多行自然歸屬 Position 欄位。標籤渲染為小字大寫欄位標頭（muted、tracking-wider；後由姊妹刀 drawer-section-labels 依使用者比對裁定改為粗體大標題共用常數，本刀 spec 不約束字級），欄位內文交給共用 Markdown prose。來源缺某欄位（如無 Ruled out）就不渲染該標籤；出現非四詞集合的粗體前綴行時，該行按一般 prose 內文照排（不誤判為欄位）。

替代方案：把欄位解析做成通用「粗體前綴＝欄位」規則——會誤傷內文中任意粗體開頭的行，否決；固定四詞白名單才可靠。

### D3 文件容器：prose 行寬上限與容器留白

共用 Markdown 元件的容器移除 max-w-none，改設固定行寬上限（以 CJK 可讀行長約 35-40 全形字為準，約 70-78ch 區間定值），內容靠左、容器保留一致的側向留白。抽屜寬度（720px 常態、96vw 全螢幕）變化時行寬不隨之增長。表格維持既有 overflow-x 橫向捲動（.markdown table），寬表格在容器內捲動、不破版。TaskList 非 prose 流不在此容器內（任務分頁不入此刀）。

替代方案：
- 內容置中（Spectra 部分頁面手法）——抽屜是側欄形態、標題與 metadata 靠左，內容置中會產生左右不對齊的鋸齒感，否決，靠左對齊。
- 依抽屜寬度百分比設上限——全螢幕仍會失控，否決，用固定字元寬。

### D4 規格分頁色標區段：delta 標題切分、配色對齊 DeltaBadges

在 packages/ui 的 delta 模組（已有 specDeltaCounts 解析 delta 標題的前例）旁新增區段切分 helper：把 delta spec 文字按「## ADDED|MODIFIED|REMOVED|RENAMED Requirements」切區段。渲染層（RichDetailDrawer 規格分頁與 ArchivedList 規格分頁共用同一區段元件）為每區段畫一個色標區段標頭——彩色標記與中文區段名（新增／修改／移除／更名），配色對齊 DeltaBadges：ADDED 綠（emerald）、MODIFIED 琥珀（amber）、REMOVED 紅（red）、RENAMED 藍（sky）——區段內文交給共用 Markdown 照排；原始「## ADDED Requirements」標題行不再直出。無任何 delta 區段標題的文件整篇照現行渲染。

替代方案：在 Markdown 元件以 components override 攔 h2 判斷 delta 標題——Markdown 是全域共用元件，delta 語意只屬規格檢視，全域攔截會把一般文件裡碰巧同名的標題誤染色，否決；切分在規格分頁的呼叫端做。

### D5 skill 範本：assets 單一來源、乾淨樹 golden 再生

crates/speclink-core/assets/skills/discuss.md 的 add-round 範例區塊改為：Position 以一句總綰開頭、隨後以「- 」列點展開（每點一行），並在該節 Document rules 補一句「Position 超過一句時 SHALL 列點分行」等級的引導；Focus／Ruled out／Open 維持單行慣例。repo 技能實例（.claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md）由 speclink update 自 assets 再生同步，不手改。render golden 於乾淨樹執行 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生並逐行審視 diff（dirty 樹再生會把未提交狀態烙進 golden——本 repo 已發生過一次，備忘錄有案）。

替代方案：改 CLI 的 discuss new scaffold 註解（Document rules）強制格式——scaffold 是功能性錨點且屬 CLI 輸出面，動它牽動回歸對照，且引導對象是寫 round 的 agent（讀技能），否決。

### D7 結論欄位標籤化：六詞白名單共用欄位解析，背景分頁不動

（實作驗收後由使用者回饋追加。）conclude scaffold 與 add-round 同體質——固定粗體前綴欄位。把輪卡片的欄位解析泛化為共用 helper（輸入內文與標籤白名單、輸出 lead＋欄位對應），splitRounds 與新的結論切分共用同一實作；結論白名單為 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 六詞，渲染為標籤區塊（決定／理由／否決替代案／擱置／記錄去向／下一步），欄位內文交給共用 Markdown prose。無任何白名單欄位的結論（手寫自由格式）整篇單一 markdown 檢視退回；來源缺席的欄位不渲染空標籤。DiscussionDrawer 結論分頁與 ArchivedList 結論區共用同一元件；結論為空（僅 scaffold 註解）時的既有預設分頁邏輯不動。

背景分頁明確不做結構化：context 是自由散文、無固定欄位可解析，硬加視覺結構是無中生有；文件容器（行寬＋prose）已涵蓋。

替代方案：
- 結論獨立寫第二份欄位解析器——與輪欄位解析重複邏輯，否決，泛化共用。
- Capture to／Next 做成互動 chip（點擊跳轉）——超出「閱讀結構」範圍，否決，純文字標籤區塊即可。

### D6 測試策略：jsdom 結構驗證，視覺以真實視窗驗收

依 TDD 先寫失敗測試再實作。vitest（jsdom）驗結構性行為：輪卡片數量與卡頭內容、欄位標籤存在與缺欄位不渲染、非標準格式整篇退回、規格分頁色標區段標頭與機器標題不直出、Markdown 容器行寬 class。golden 再生屬 cargo test（render_golden）。間距、色彩、行長等視覺效果 jsdom 測不出——apply 尾聲以 release exe 真實視窗＋截圖驗收（機器備忘的 GUI 驗證程序）。

## Implementation Contract

**可觀察行為：**

- 討論抽屜結論分頁與已封存討論檢視結論區：scaffold 格式結論的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 以標籤區塊呈現（決定／理由／否決替代案／擱置／記錄去向／下一步），粗體前綴原文不出現在欄位內文；缺席欄位無空標籤；無任何白名單欄位時整篇以現行單一 markdown 檢視呈現。
- 討論抽屜討論過程分頁與已封存討論檢視：scaffold 格式記錄逐輪呈現為獨立卡片；卡頭含輪次（Round N）、mode（assumptions／interview）、日期；卡身 Focus／Position／Ruled out／Open 以標籤區塊呈現，「**Focus**:」等粗體前綴原文不再出現在內文；來源缺的欄位無空標籤；任一輪標題不合 scaffold 格式時整篇以現行單一 markdown 檢視呈現。
- 所有經共用 Markdown 元件渲染的文件（變更抽屜提案／設計／規格分頁、討論抽屜各分頁、已封存檢視）：內容行寬有固定上限，抽屜全螢幕（96vw）時行寬不隨之增長；表格超寬時於容器內橫向捲動。
- 變更抽屜規格分頁與已封存變更檢視規格分頁：delta 區段以色標標頭呈現（新增綠、修改琥珀、移除紅、更名藍），原始「## ADDED Requirements」等標題行不以 h2 文字直出；requirement 與 scenario 內文照 prose 排版；不含 delta 標題的規格文件整篇照現行渲染。
- discuss 技能（三處實例內容一致）：add-round 範例的 Position 為「一句總綰＋列點展開」形；render golden 快照與 assets 渲染結果一致。

**介面／資料形狀：** 輪切分 helper 輸入記錄 Rounds 區段全文、輸出輪陣列（輪次、mode、日期、欄位名→內文的對應）或 null（不合格式）；delta 區段切分 helper 輸入 delta spec 全文、輸出區段陣列（delta 種類、區段內文）。皆為 packages/ui 內部純函式，無 IPC、無新依賴。

**失敗模式：** 解析失敗一律靜默退回現行整篇 prose 渲染，不報錯、不留空白；來源檔案在任何路徑下位元不變。

**驗收：** npm test -w packages/ui 全綠（新增輪卡片、欄位標籤、fallback、色標區段、容器 class 測試）；乾淨樹 golden 再生後 cargo test -p speclink-core --lib 與 render_golden 綠；release exe 開啟本專案實際記錄（desktop-reading-and-tasks-ux 四輪）截圖確認卡片分界與行寬。

**範圍邊界：** in scope＝上述三個檢視面的呈現層（含討論結論分頁欄位標籤化）與 discuss 技能範本內容；out of scope＝背景分頁結構化、TaskList、SpecsView 功能、CLI 指令行為與輸出、既有記錄回寫、markdown 引擎更換。

## Risks / Trade-offs

- [dirty 樹再生 golden 把未提交狀態烙進快照，main 長期紅燈（已發生過）] → 任務明定：golden 再生前 git status 必須乾淨，再生後逐行審視 diff 僅含 discuss 範本措辭。
- [行寬上限用 ch 單位對 CJK 的實際行長不直觀] → 以全形字數驗算定值（35-40 全形字），真實視窗截圖驗收；值定案後寫入測試斷言的 class 名。
- [舊記錄的 Position 仍是單行長文，卡片內依然偏密] → 接受：卡片分界＋欄位標籤已是可讀性下限保證；內容側改善僅及新記錄（範本引導），不回寫舊檔。
- [輪標題格式解析對日期／mode 變體（未來新 mode）過緊導致整篇退回] → mode 欄位按字串透傳呈現、不設白名單；只有結構（### Round N — … (…)）不合才退回。
- [色標區段元件與 DeltaBadges 配色日後漂移] → 兩處共用同一組色彩 class 常數，測試斷言引用同一來源。
- [回歸對照] → CLI 人眼與 --json 輸出零變更，parity／color 套件不受影響；golden 屬刻意更新並經審視。跨平台無新面（純前端＋文件內容）。

## Open Questions

（無——行寬定值於實作時在 35-40 全形字區間敲定，屬工程細節非設計分歧。）
