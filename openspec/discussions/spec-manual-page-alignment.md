---
topic: 規格詳情頁與手冊頁是否也改成詳情頁的手冊式設計：文件樹、閱讀卡、數字磚、提示框與本頁目錄
slug: spec-manual-page-alignment
status: promoted
created: 2026-10-12
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: desktop-spec-manual-pages
---

# Discussion: 規格詳情頁與手冊頁是否也改成詳情頁的手冊式設計：文件樹、閱讀卡、數字磚、提示框與本頁目錄

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 detail-content-redesign 的三刀排定後提出：規格詳情頁與手冊頁希望也符合目前詳情頁的設計模式。前次討論把這兩頁列為非目標，並擱置「手冊頁要不要同步加數字磚」；本討論重開這一項。

現況：規格詳情頁（packages/ui/src/components/SpecDetailPage.tsx）仍是第一刀之前的舊骨架——DetailHeader 標頭＋卡片分頁列（規格 N／溯源 M）＋需求目錄在 768px 欄左側空白處；手冊頁（packages/ui/src/components/ManualPage.tsx）本來就是三欄（240px 目錄樹、閱讀卡、200px 本頁目錄）加固定底列（出處籤、上一頁／下一頁），差別在閱讀卡內部：頁標題固定在卡外、無出處行、無數字磚、內文 16px、無「開啟 ▾」。DocumentRef 只有變更、討論、規格、已封存四種，沒有手冊頁。

Related specs: desktop-app（桌面 app 呈現 change 與 spec 的清單與內容、markdown 內容保留文件結構呈現、markdown 文件內容行寬有上限、詳情頁的文件樹、閱讀卡與本頁目錄）；desktop-manual-page（手冊頁的側欄樹與閱讀序、內頁渲染與出處跳規格、可能過期與未入冊的標示）。
Related changes: desktop-change-doc-views（在途、apply 中）會改寫 markdown 的兩條需求並明寫規格詳情頁維持 16px／96ch；desktop-detail-doc-tree（已封存）定義了三欄骨架與閱讀卡零件；desktop-discussion-doc-views（cut 3，尚未建）。
Prior discussions: detail-content-redesign（非目標：規格詳情頁與手冊頁維持現狀；擱置：手冊頁要不要同步加數字磚）, desktop-ui-redesign（第 18 輪否決手冊頁尾放在內文末端——底列固定）, desktop-readonly-detail-pages（規格詳情頁的需求目錄與需求段）
Source doc: none

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-12)

**Focus**: 規格詳情頁與手冊頁要不要、以及怎麼對齊詳情頁的手冊式設計（三欄、閱讀卡、數字磚、提示框、本頁目錄）
**Position**: 使用者確認六項假設全對：兩頁都做、份量不同；原型已加進 https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底一列兩張畫板，等使用者看過。
- 規格詳情頁整個骨架換成 DetailShell：左欄身分區（能力名、需求數與 scenario 數、最近更新、「溯源 N」關聯列）、「文件」節點 規格／溯源（溯源節點取代原本的分頁）、「狀態」列；閱讀卡標題「規格」＋搜尋需求＋「開啟 ▾」，出處行為規格檔路徑，數字磚為需求、Scenario、溯源變更、最近更新，Purpose 成引言段，需求段沿用現有畫法（第一條展開、其餘收合為一列，右端來源變更與日期）；需求目錄從左側空白處搬到右欄本頁目錄
- 規格詳情頁的提示框只放推得出的事實：Purpose 為封存佔位文字時「Purpose 待補」、有需求缺 @trace 時「N 條需求沒有來源變更」；真實資料（tray-status-menu）兩者皆無，畫板以 Tweaks 切換
- 手冊頁三欄與固定底列（出處、上一頁／下一頁）不動；只對齊閱讀卡：頁標題與「產生於」從卡外搬進卡的標題區，「可能過期」「未入冊」籤進籤槽，出處行為手冊檔路徑、生成時間與出處規格籤，數字磚為章節、小節、關鍵字、出處規格，可能過期時 warning 提示框（真實：〈系統匣選單〉09-17 產生、出處 10-12 更新），章節 16px、內文 15px，左樹節點改用文件樹同款並在過期的頁標「過期」
- 手冊頁的「開啟 ▾」要宿主與 Rust 多認一種文件：DocumentRef 只有變更、討論、規格、已封存四種
- 一刀落地，排在 desktop-change-doc-views 封存之後：它正在 apply，其 delta 把規格詳情頁排除在 15px／卡欄上限之外，本討論落地時要再改「markdown 內容保留文件結構呈現」「markdown 文件內容行寬有上限」同兩條
- 這是新討論、不重開 detail-content-redesign：那份已轉出兩刀且 cut 2 在途，重新 conclude 會讓在途 change 標待重新反映
**Ruled out**: 重開 detail-content-redesign 加輪——在途 change 會被標待重新反映；規格詳情頁只換字級不換骨架——它是唯一還長得不一樣的詳情頁；手冊頁拿掉底列——desktop-ui-redesign 第 18 輪裁定底列固定、手冊有閱讀順序
**Open**: 使用者看原型後的三件裁定（規格詳情頁的目錄搬右欄與溯源成文件節點；手冊頁標題進卡、底列保留；手冊頁數字磚四格內容）；規格詳情頁的搜尋框位置（閱讀卡標題列）是否成立；「開啟 ▾」在手冊頁要不要做（決定這刀碰不碰 Rust）；落地時序（等 cut 2 封存）與是否與 cut 3 平行

### Round 2 — interview (2026-10-12)

**Focus**: 原型裁定——使用者看過「手冊式詳情頁 C」兩張畫板
**Position**: 使用者選 C，規格詳情頁與手冊頁兩張畫板全部成立。
- 規格詳情頁：目錄搬到右欄本頁目錄、溯源成文件節點、搜尋需求放閱讀卡標題列，成立
- 手冊頁：頁標題與「產生於」進閱讀卡、固定底列保留、數字磚四格（章節、小節、關鍵字、出處規格）照畫板，成立
- 手冊頁的「開啟 ▾」照畫板做：DocumentRef 多一種手冊頁、宿主的 locate 認 openspec/manual/<slug>.md；這刀因此碰到 Rust desktop core 一小段
**Ruled out**: 手冊頁不做「開啟 ▾」以避開 Rust——畫板已含且使用者整張接受，少它手冊頁就與其他頁的閱讀卡不同款
**Open**: 無——寫結論

## Conclusion

**Decision**: 規格詳情頁與手冊頁對齊詳情頁的手冊式設計：規格詳情頁整個換成三欄骨架，手冊頁只對齊閱讀卡；原始規格與手冊 Markdown 不動；一個變更落地，排在 desktop-change-doc-views 封存之後。
- 規格詳情頁改用詳情頁共用的三欄骨架（文件樹、閱讀卡、本頁目錄）：左欄身分區為能力名（等寬＋複製）、「正式規格」籤、「N 條需求 · M 個 scenario」與「最近更新 日期」出身行、「溯源」關聯列（首顆來源變更籤＋「+N」浮層，點擊切到溯源文件）；「文件」節點為規格（需求數徽章）與溯源（來源變更數徽章），溯源節點取代原本的「溯源 M」分頁；「狀態」列為需求、溯源、最近更新三列
- 規格詳情頁的閱讀卡：標題「規格」＋需求數籤＋「搜尋需求」輸入（規則不變：子字串比對需求標題、只呈現命中段、Purpose 恆在、不持久化）＋「開啟 ▾」（DocumentRef 既有的 spec 種類）；出處行為 `openspec/specs/<capability>/spec.md` · 正式規格 · 最近更新（變更）；數字磚為需求、Scenario、溯源變更、最近更新；Purpose 成灰底引言段（標題 Purpose 進本頁目錄）；需求段沿用規格詳情頁現有畫法並加上 cut 2 的收合規則（第一條展開、其餘收合為一列：Requirement 小標、標題、來源變更與日期、Scenario 數籤）；需求目錄從左側空白處搬到右欄本頁目錄（Purpose＋各需求標題），≥1200px 才顯示
- 規格詳情頁的溯源文件：出處行寫明自 @trace 推導、不是檔案；數字磚為來源變更數、最早、最近、有來源的需求數／需求數；列規則與現在的溯源分頁相同（封存清單命中者在前依封存日期升冪、可點開已封存詳情頁，未命中者「無封存記錄」不可點）；無來源時空態文案不變
- 規格詳情頁的提示框只放推得出的事實：Purpose 為封存流程佔位文字時 warning「Purpose 待補」（指引直接編輯正式規格的 Purpose 段）；有需求缺 @trace 時 note「N 條需求沒有來源變更」列出名稱；兩者皆無時不出現
- 規格詳情頁標題列右端只有 ‹ n／m ›，無變更層動作；導覽、返回、宿主維持（規格頁或手冊頁）規則不變
- 手冊頁三欄與固定底列（出處籤、上一頁／下一頁）不動；頁標題與「產生於」從卡外搬進閱讀卡標題區（24px 標題、分區籤、「可能過期」「未入冊」籤、「開啟 ▾」），出處行為 `openspec/manual/<slug>.md` · 產生於 時間 · 出處規格籤（可點開規格詳情頁，規則同底列）；數字磚為章節（h2 數）、小節（h3 數）、關鍵字（frontmatter keywords 數）、出處規格（sources 去重數）；可能過期時 warning 提示框點名出處規格、更新日期與來源變更、本頁產生時間，並指引用 /speclink-manual 重生；章節 16px 粗體、內文 15px 行高 1.75；左樹節點改用文件樹同款（30px 列、主色淡底作用中），可能過期的頁在節點尾標「過期」籤
- 手冊頁的「開啟 ▾」：DocumentRef 新增手冊頁種類（slug），宿主 locate 解析為 openspec/manual/<slug>.md（Rust desktop core 同一套解析），選單項與規格詳情頁相同；remote 無 checkout 時只剩「複製內容」
- 規格頁清單、已封存清單與其他頁不動；規格詳情頁的 DetailHeader／DetailPageFrame／DetailContentCard 失去最後消費者即退場
- 受影響規格：desktop-app MODIFIED「桌面 app 呈現 change 與 spec 的清單與內容」（規格詳情頁段落改寫為三欄骨架、溯源文件、數字磚、提示框）、「markdown 內容保留文件結構呈現」與「markdown 文件內容行寬有上限」（規格詳情頁與手冊頁納入 15px 與卡欄上限，不再例外）、「詳情頁的文件動作」（手冊頁納入「開啟 ▾」）；desktop-manual-page MODIFIED「內頁渲染與出處跳規格」（標題進卡、出處行、數字磚、15px、節點款式）、「可能過期與未入冊的標示」（籤進籤槽、節點「過期」籤、提示框）；不改「詳情頁的文件樹、閱讀卡與本頁目錄」（規格詳情頁以引用方式共用骨架，避免與 cut 3 同名重疊）
- 時序：等 desktop-change-doc-views 封存後再 propose（同兩條 markdown 需求、且沿用它的需求段收合與 DocParts 零件）；與 desktop-discussion-doc-views（cut 3）無同名需求，可平行
**Rationale**: 規格詳情頁是四種詳情頁裡唯一還是舊骨架（標頭＋卡片分頁列＋左側目錄）的頁，換成同一套骨架使用者學一次就通；手冊頁是三欄模式的源頭，只需閱讀卡內部對齊，底列因手冊有閱讀順序而保留；數字磚與提示框全由現有結構（需求段、@trace、frontmatter、過期判定）推出，規格與手冊的 Markdown 都不必動；「開啟 ▾」補上手冊頁才讓四種閱讀卡同款，代價只是 DocumentRef 多一種與 locate 多一條分支。
**Rejected alternatives**:
- 重開 detail-content-redesign 加輪 — 那份已轉出兩刀且 cut 2 在途，重新 conclude 會讓在途 change 標待重新反映
- 規格詳情頁只換字級、不換骨架 — 它是唯一長得不一樣的詳情頁，換字級解不了目錄在左、分頁列在卡頭的差異
- 手冊頁拿掉固定底列、出處與翻頁進閱讀卡 — desktop-ui-redesign 第 18 輪裁定底列固定，手冊有閱讀順序
- 規格詳情頁的目錄留在左側空白處 — 與其他三種詳情頁相反，左欄管頁與頁之間、右欄管頁內的分工就斷了
- 手冊頁不做「開啟 ▾」以避開 Rust — 畫板已含且使用者整張接受，少它手冊頁就與其他頁的閱讀卡不同款
- 修改「詳情頁的文件樹、閱讀卡與本頁目錄」把規格詳情頁寫進去 — cut 3 大概率要改同一條（討論文件的出處行），兩刀會同名重疊；以引用共用骨架即可
**Deferred**:
- 規格詳情頁的「搜尋需求」要不要改成同時過濾右欄本頁目錄 — 等目錄上線後看實際使用
- 手冊頁節點「過期」籤的英文文案與未入冊頁在樹上的標法 — propose 時對照 desktop-manual-page「可能過期與未入冊的標示」現文決定
- 手冊頁的提示框是否納入 GitHub Alert 之外的第二種推導（如出處規格不存在） — 現有「不存在的出處不可點」已覆蓋，先不加
**Capture to**: proposal、design、spec（desktop-app MODIFIED、desktop-manual-page MODIFIED）、tasks
**Next**: desktop-change-doc-views 封存後跑 /speclink-propose --from-discussion spec-manual-page-alignment（單一變更，不帶 --hold 與 --last）；原型在 https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底一列
