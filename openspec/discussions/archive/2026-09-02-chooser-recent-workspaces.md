---
topic: 在開啟 workspace 時列出曾經開過的 workspace
slug: chooser-recent-workspaces
status: promoted
promoted_to: chooser-recent-workspaces
created: 2026-09-02
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 在開啟 workspace 時列出曾經開過的 workspace

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者截圖桌面 app 的「新增 Workspace」chooser 第一步，問能否在這裡列出曾經開過的 workspace。目標可驗證，不需 grill 階段，直接以假設清單開場。

現況：分頁列（tabs.ts，localStorage 鍵 speclink.projectTabs）就是唯一的「最近開啟」記憶，分頁一關即自清單移除；chooser 第一步只有「本機資料夾」與「Speclink Server」兩張來源卡。

相關正典與歷史：specs/desktop-config（分頁列需求：關閉分頁即自持久化清單移除）、specs/workspace-chooser（來源分流）、specs/workspace-session（WorkspaceLocator 身分與持久化 v2）；已封存討論「專案選擇對齊-spectra」（2026-07-06）曾否決「分頁之外另設最近清單」，理由是同一概念重複表達；已封存變更 desktop-config-multiproject（D8／D10）與 workspace-chooser-onboarding（2026-07-20 加入 chooser）。目前無進行中的變更。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-02)

**Focus**: 這個需求是分頁列已經涵蓋的東西，還是要推翻舊決定另設一份記憶
**Position**: 使用者裁定推翻 2026-07-06「分頁列即最近清單」的決定：分頁關掉可以忘記，但開過的 workspace 路徑要另外記住，形態如 VS Code 的最近開啟。
- 決策樹：A 是否推翻舊決定／B 清單放哪／C 記什麼／D 存哪／E 點了走哪條路與失效處理；A 由使用者定案
- B、C、E 以假設提出且未被反對，視為接受：清單放 chooser 第一步兩張來源卡下方（WorkspaceChooser.tsx 的 source 步驟目前只有兩張卡）；本機與 remote 都記、單位是 WorkspaceLocator（session.ts 已有 local／remote 兩種形狀，分頁持久化 v2 已存 locator＋顯示名）；點項目沿既有 onOpenLocal／onOpenRemote 開啟流程，本機仍先經 openProject 探測，失效項目沿 desktop-config 的錯誤態規則且可移除
- D 由使用者授權我決定：localStorage 獨立新鍵，沿 tabs.ts 的純函式模組模式。理由：與分頁同一耐久等級、零 IPC、唯一消費者是 chooser
- 正典衝突點：specs/desktop-config 行 123「關閉分頁 SHALL 將其自持久化清單移除」不變（分頁仍會忘），新增的是分頁之外的記憶；tabs.ts 檔頭「分頁列即最近開啟清單」的說法要改
**Ruled out**: 把清單塞進分頁持久化鍵（逼出 v3 遷移，兩個概念綁在一起）；Rust 側 app_config_dir 的 JSON 檔（要新增讀寫 command 與模組，卻沒有第二個消費者）；只記本機（用 server 的人得不到好處）
**Open**: 清單呈現細節待使用者對 mockup 確認（顯示欄位、每筆可移除、上限 20、首次啟動自既有分頁補種）；零分頁的空狀態引導頁與系統匣是否也列最近開啟（傾向延後）

### Round 2 — interview (2026-09-02)

**Focus**: mockup 與七條清單規則是否成立，以及使用者追加的條件
**Position**: 使用者接受 mockup 與七條規則，追加一條：已開啟且在分頁列上的 workspace 不顯示於最近開啟清單。
- 實作語意：記錄照存（locator 仍在 localStorage 清單裡），只在 chooser 顯示時以 locator key 過濾掉目前分頁列上的項目；分頁一關，該項目自然回到清單
- 過濾後清單為空時，整個「最近開啟」區段不顯示，不留空標題
- 其餘定案：本機項目顯示名稱＋路徑、remote 項目顯示 server／Project／Repo；點項目沿既有開啟流程（本機先探測）；資料夾不存在或連線已移除轉錯誤態；每筆可 ✕ 移除；上限 20；升級後首次啟動自既有分頁補種
**Ruled out**: 清單顯示已開著的分頁（與分頁列重複資訊，使用者裁定不顯示）；把「已開著」的項目從記錄裡刪掉（關分頁後就找不回來，違背記憶目的）
**Open**: 無，進結論

## Conclusion

**Decision**: 在「新增 Workspace」chooser 第一步的兩張來源卡下方加「最近開啟」清單。記錄存 localStorage 獨立新鍵（沿 tabs.ts 純函式模組模式）：每次成功開啟即把該 workspace（WorkspaceLocator＋顯示名）移到最前、同 locator key 去重、上限 20；分頁關閉不影響記錄。顯示時過濾掉目前分頁列上已開著的 workspace，過濾後為空則整段不顯示。本機項目顯示名稱與路徑、remote 項目顯示 server／Project／Repo。點項目沿既有 onOpenLocal／onOpenRemote 流程（本機先經 openProject 探測，未初始化仍走 init 確認）。資料夾不存在或連線已移除轉錯誤態、點擊只顯示原因；每筆滑過出現 ✕ 可移除。升級後首次啟動自既有分頁補種清單。
**Rationale**: 分頁列回答「現在開著什麼」，最近清單回答「以前開過什麼」；兩者分開才能同時做到分頁關掉即忘與路徑記憶（VS Code 模型）。localStorage 與分頁同一耐久等級、零 IPC、唯一讀者是 chooser。
**Rejected alternatives**: 維持 2026-07-06「分頁列即最近清單」（關掉即忘、無記憶，使用者裁定推翻）；把清單塞進分頁持久化鍵（逼出 v3 遷移、兩概念綁死）；Rust 側 app_config_dir JSON（要新增讀寫 command 卻無第二消費者）；只記本機（remote 使用者無益）；清單顯示已開著的分頁（與分頁列重複，使用者裁定不顯示）。
**Deferred**: 零分頁的空狀態引導頁與系統匣選單是否也列最近開啟（同一鍵可直接讀，另案）；跨機器或重裝後保留清單（未要求）。
**Capture to**: proposal（新變更）；specs/workspace-chooser 新增「最近開啟清單」需求；specs/desktop-config 分頁列需求措辭調整（分頁列不再是唯一最近清單，「關閉分頁即自清單移除」不變）；design（localStorage 鍵、純函式模組、顯示期過濾）
**Next**: /speclink-propose --from-discussion chooser-recent-workspaces
