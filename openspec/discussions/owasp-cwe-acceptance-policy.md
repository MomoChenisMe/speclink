---
topic: 增加一個 checkbox 設定是否納入驗收條件，還是只是做參考文件，不影響專案驗收
slug: owasp-cwe-acceptance-policy
status: promoted
created: 2026-10-20
created_by: mosutw <luke1974@gmail.com>
promoted_to: desktop-workflow-and-offline-security-review
---

# Discussion: 增加一個 checkbox 設定是否納入驗收條件，還是只是做參考文件，不影響專案驗收

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 desktop-workflow-and-offline-security-review 結論後追加需求，附上既有安全審查設定畫面：新增 checkbox，控制 OWASP／CWE 安全審查是否納入驗收條件，或只作參考文件而不影響專案驗收。這是對先前「安全報告只作資訊、不新增封存門檻」方向的明確擴充，不影響原討論其餘九項需求、基本掃描保留、以 skill 執行深度審查、資料離線與提案內紀錄保存的結論。前討論已 concluded，本討論獨立接續，不重寫先前輪次。
相關正式規格為 security-review-report、security-review、desktop-config、workflow-config、verify-station 與 change-lifecycle，目前沒有進行中的變更。security-review-report 明訂報告不預設阻擋 Archive；verify-station 的 stamp 要求沒有未解必修項，change-lifecycle 現有封存規則則允許沒有驗證章，故僅在 skill 中文字提醒或只擋 verify stamp，未必能落實必須驗收的政策。若納入驗收，要由引擎共用判定涵蓋 CLI／Desktop／Remote 的相關流程，不能僅隱藏 UI 按鈕。詞彙表「初步安全審查」目前也寫明不新增封存守門，後續提案需同步釐清參考模式與納入驗收模式，不把驗收通過描述成正式資安認證。
已有事實：安全結果含風險、未驗證、缺資料與未完成；skill 規劃使用 HIGH／MEDIUM／LOW，基本掃描模型另含 critical。既有審查／驗證的必修分類是 CRITICAL／WARNING，不能不經定義就把兩套等級視為相同。尚須決定勾選後的通過標準與風險接受方式。
Prior discussions: desktop-workflow-and-offline-security-review, owasp-cwe-preliminary-security-review

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-20)

**Focus**: OWASP／CWE 審查的驗收參與方式與待決的通過條件。
**Position**: 使用者已要求增加「納入驗收條件」checkbox，參考模式與驗收模式須有實際行為差異，勾選後的風險門檻尚待確認。
- 已定：在既有「啟用 OWASP / CWE 安全審查」設定區新增 checkbox；未勾選時報告作參考，不因安全結果阻擋專案驗收；勾選時安全審查成為驗收要求。兩種模式皆沿用提案內紀錄及 Desktop 查詢要求。
- 建議：checkbox 放在主開關下方，預設未勾選以維持既有行為；只在主開關啟用時生效。UI 儲存與其他安全政策共用既有 workflow-config，不另存裝置偏好；此預設及位置尚屬設計建議。
- 證據：security-review-report 原本明定資訊性且不預設阻擋封存，verify-station 的驗證章檢查必修項，change-lifecycle 的封存則允許無章。勾選後若要確實影響驗收，不能只靠 skill 提醒，應由現有引擎流程共用政策及報告有效性的判定；介面仍須呈現未完成／不通過原因。
- 待確認建議：勾選後，CRITICAL（若基本掃描有此級）／HIGH／MEDIUM 的未解風險須修復或經明確接受並留下理由，LOW 保留為建議；未執行、必要資料缺失、審查未完成或必要項未驗證不得自動當作通過。這些門檻不是使用者已指定的要求。
- 具體差異例：同一份有未處理 HIGH 風險的報告，在未勾選時不影響原驗收；勾選時依確認後的通過規則阻擋，報告仍能在提案內查閱。不得因切換此設定而把歷史報告內容改寫成通過。
**Ruled out**: checkbox 只改顯示文字、不影響驗收判定——不符合使用者要選擇是否納入驗收的要求；無條件維持所有安全報告皆不影響驗收——使用者已明確新增可選模式。
**Open**: 勾選後的通過標準與是否允許附理由接受風險；驗收與封存的確切接點及避免略過驗收的做法；基本掃描與深度 skill 哪些啟用結果納入判定；checkbox 預設、主開關依賴、政策變更及報告過期語意。

### Round 2 — interview (2026-10-20)

**Focus**: 勾選「納入驗收條件」後的安全審查通過標準。
**Position**: 使用者確認採用建議標準：中、高及嚴重風險需修復或明確接受，低風險列建議，缺少必要審查資訊不得通過。
- 使用者回覆「採用這個標準」，確認 CRITICAL／HIGH／MEDIUM 的未解風險須修復，或明確記錄接受風險的理由；LOW 不影響驗收，只作建議。
- 未審查、必要資料缺失、審查未完成或必要項目未驗證，不能算通過；接受某筆風險不能冒充已完成尚未執行的檢查。
- checkbox 沿前輪方案放在安全審查主開關下方，預設不勾選；未勾選時安全報告供參考，不影響原驗收，勾選時依上述標準參與驗收。主開關與此選項的停用／再啟用細節由提案明訂。
- 兩種模式皆保留提案內紀錄供 Desktop 查詢，風險接受理由也須可查，不藉切換 checkbox 改寫歷史結果。
- 驗收例：僅有 LOW 建議且必要檢查完成，可以通過；有未處理 HIGH 時不能通過，修復複核或記錄接受理由後才可依其餘條件判定；缺必要資料時即使沒有 findings 仍不能通過。未勾選則上述安全結果不額外阻擋原驗收。
**Ruled out**: 所有等級風險都必須處理——使用者採用 LOW 只作建議的標準；一律禁止接受風險——使用者採用可明確記錄接受理由的標準；沒有發現就視為通過——缺資料及未驗證仍須擋下。
**Open**: 驗收／封存的引擎接點、基本掃描與 skill 結果的彙整範圍、風險接受的欄位及操作、主開關依賴、政策變更與報告過期的精確語意，交提案設計。

## Conclusion

**Decision**: 在 OWASP／CWE 安全審查設定新增「納入驗收條件」checkbox，採用使用者確認的分級風險與審查完整性標準。
- checkbox 放在「啟用 OWASP / CWE 安全審查」主開關下方，預設不勾選，與既有專案安全政策一起保存。
- 未勾選時，安全報告僅供參考，不因其結果額外阻擋專案驗收；勾選時，安全審查須符合通過條件才能完成驗收。
- CRITICAL／HIGH／MEDIUM（嚴重／高／中）風險須修復，或明確記錄接受風險的理由；仍未處理的項目不得通過。
- LOW（低）風險保留為建議，不影響驗收，不要求為此清除全部低風險項目。
- 未審查、必要資料缺失、審查未完成或必要項目未驗證，均不能判定通過；接受已知風險不等於豁免未完成的必要檢查。
- 兩種模式都須在提案內保留可由 Desktop 查詢的審查紀錄；風險接受理由一併留存，沿用原討論的蓋章後保存及封存保留要求。切換模式不得改寫歷史報告的結果、證據或當時設定。
- 內建基本掃描保留，OWASP／CWE 深度審查採 skill、只使用完整本機資料且遵守所選標準版本；原始 desktop-workflow-and-offline-security-review 的其餘需求持續有效。
- 驗收例：必要檢查完整而僅有 LOW 建議時可通過；存在未處理 HIGH 時不可通過，修復複核或記錄接受理由後依其餘條件判定；缺必要資料且零發現時仍不可通過。未勾選時，這些安全結果不額外影響既有驗收。
- 此方向擴充既有「報告為資訊性且不預設阻擋」契約為可選驗收政策；共用引擎須落實判定，避免只由 UI 或 skill 文字提醒。驗收／封存的具體接點在設計定義，不能讓勾選模式僅變成報告標籤。
**Rationale**: 讓專案可選擇將安全結果作參考或正式驗收要求；中高風險有明確處置、低風險不造成不必要阻擋，同時避免把未執行或缺資料誤當安全通過。附理由的風險接受保留人工裁決及後續查詢依據。
**Rejected alternatives**:
- 一律將安全報告作參考 — 使用者要求可勾選納入驗收。
- checkbox 只改畫面、不影響驗收判定 — 無法落實所選政策。
- 連 LOW 也必須全部處理 — 使用者已確認低風險只列建議。
- 中高風險永不允許接受 — 使用者確認可明確記錄接受理由。
- 零發現、缺資料或未驗證即通過 — 使用者確認必要檢查完整性也是通過前提。
- 切換模式或接受風險後改寫舊報告為無風險 — 會失去當時結果與裁決依據。
**Deferred**:
- 驗收、驗證蓋章與封存的精確引擎接點，及 CLI／Desktop／Remote 的共同行為 — 交 proposal／design 對齊現有生命週期，避免未跑驗收仍能繞過政策。
- 基本掃描與深度 skill 哪些啟用結果、審查目標及必要項目納入判定 — 交 design 定義報告彙整與標準適用性，不擅自把停用或不適用項算必要。
- 接受風險的操作與欄位、複核證據、可歸屬的接受紀錄 — 交 design 定義最小可查詢格式，不能只留無理由的勾選。
- 主開關停用時 checkbox 的顯示、保存與生效方式，以及全部標準關閉的處理 — 交 design 避免設定互相矛盾或未配置被當通過。
- 政策或程式碼變更後報告的適用性與過期判定 — 交 design 定義可測規則，保留當時政策與來源資訊，不改寫歷史。
- 正式規格與詞彙表的調整 — 提案同步更新 security-review-report、security-review、desktop-config、workflow-config、verify-station、change-lifecycle 及 LANGUAGE.md 的適用範圍，驗收通過仍不得描述為正式資安認證。
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md
**Next**: $speclink-propose --from-discussion owasp-cwe-acceptance-policy；規劃時一併讀取 desktop-workflow-and-offline-security-review 的結論，保留原始九項需求。
