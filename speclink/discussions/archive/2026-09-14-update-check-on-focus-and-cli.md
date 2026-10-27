---
topic: 桌面 app 視窗回到前景時重檢更新；只裝 CLI 是否需要更新檢查機制
slug: update-check-on-focus-and-cli
status: promoted
created: 2026-09-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: update-recheck-on-focus
---

# Discussion: 桌面 app 視窗回到前景時重檢更新；只裝 CLI 是否需要更新檢查機制

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者先問「自動檢查更新的時機點是不是第一次開啟 app」。查證結果：desktop 每次啟動在背景檢查一次（App.tsx 掛載時呼叫 checkForUpdates(false)），沒有定時重檢；中途出新版要靠設定頁手動「檢查更新」；單獨安裝的 CLI（install.sh／brew）完全沒有更新檢查，`speclink update` 是回寫技能檔不是自我更新；桌面 app 佈署到 ~/.local/bin 的那份 CLI 在 app 啟動時版本不符會自動重佈。使用者接著問兩個決定：(1) 視窗回到前景時重檢做不做得到；(2) 只裝 CLI 的話要不要檢查機制。需求已夠清楚（可驗證的目標），不需 grill 階段，直接以假設清單開場。
相關規格：desktop-app（「桌面自動更新」需求，只寫「啟動後在背景檢查」，未限制次數）、desktop-release（updater 簽章、latest.json）、cli-distribution（只寫安裝，未寫更新檢查）。
相關變更：release-assets-trim（進行中）——把 CLI 通路改為 npm 唯一來源、brew 改指 npm 檔案，直接影響「CLI 要不要內建檢查」的取捨。
Prior discussions: release-assets-trim

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-14)

**Focus**: 前景重檢是否可行、要不要做；只裝 CLI 要不要更新檢查
**Position**: 四個假設全數成立（使用者「好OK可以」）：前景重檢做、CLI 檢查不做。
- 前景重檢可行且改動小：Tauri 視窗 API 有 `onFocusChanged`（node_modules/@tauri-apps/api/window.d.ts:1299），app 尚未使用；`checkForUpdates(false)` 已是自動模式（失敗靜默、下載中／待重啟自動擋重檢，core/updater.ts reducer），前景重檢只需在 App.tsx 多掛一個監聽呼叫同一支函式
- 節流：回到前景時距上次檢查超過 1 小時才再查；時間戳放 store 記憶體，不寫檔；app 重啟本就檢查一次
- 只裝 CLI 不加檢查機制：release-assets-trim 把通路改成 npm 唯一來源（brew 也指 npm 檔），npm／brew 自帶升級流程；CLI 主要由 agent 以 --json 呼叫、一個 session 幾十次，提示會混進輸出且要加快取／離線／CI 偵測；舊 CLI 的真正傷害已有守門（`speclink update` 對比引擎新的工作區拒絕回寫，需 --allow-downgrade；桌面佈署的 CLI 啟動時自動換新）
- 前景重檢立獨立小變更，不併進 release-assets-trim（後者改發版產物與通路，前者只動 desktop 前端）
- 正典對照：desktop-app「桌面自動更新」只寫「啟動後在背景檢查」，不與前景重檢衝突，補一個 scenario 即可
**Ruled out**: 定時輪詢（setInterval）——前景事件已覆蓋「使用者回來看」的時機，常駐計時器多打端點無益；CLI 內建更新提示——與 npm／brew 重複、污染 agent 輸出、複雜度不成比例；併入 release-assets-trim——驗證面混雜
**Open**: 節流時長 1 小時是猜測值，propose 期可定；macOS 系統匣面板是否觸發主視窗 focus（propose 期驗）

## Conclusion

**Decision**: 兩件事各一個結論。(1) 桌面 app 新增「視窗回到前景時重檢更新」：以 Tauri 視窗 `onFocusChanged` 為訊號，focused 為 true 且距上次檢查超過 1 小時才呼叫既有的 `checkForUpdates(false)`（自動模式：失敗靜默、下載中／待重啟不重檢）；時間戳放 store 記憶體；立獨立小變更，只動 desktop 前端（App.tsx 掛監聽、store 記上次檢查時間），canon delta 為 desktop-app「桌面自動更新」補一個「回到前景重檢」scenario。例：app 開著 3 小時未關，期間發布新版；使用者切回 app 視窗，1 秒內背景檢查一次，提示浮出目標版號；同一小時內再切回不會再查。(2) 只裝 CLI 的使用者不加任何更新檢查機制——不立變更。
**Rationale**: 前景事件恰好對應「使用者回來看」的時機，重用既有自動模式的狀態機，成本是一個監聽加一個時間戳；CLI 的通路（npm／brew）自帶升級流程，CLI 主要由 agent 呼叫，內建提示會污染輸出且要加快取／離線／CI 偵測，而舊 CLI 的真正傷害已有 `speclink update` 的降級守門與桌面啟動自動重佈。
**Rejected alternatives**: 定時輪詢——常駐計時器多打端點，前景事件已覆蓋需求；CLI 內建更新提示——與通路重複、污染 agent 輸出、複雜度不成比例；併入 release-assets-trim——驗證面混雜；只在 `speclink update` 動詞內一天查一次——留作折衷，目前無舊 CLI 回報 bug 的實證，不做。
**Deferred**: 節流 1 小時為預設值，propose 期可改；macOS 系統匣面板是否觸發主視窗 focus 事件，propose 期實測；若日後常有舊 CLI 回報 bug，再開討論談 `speclink update` 內的一日一查。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion update-check-on-focus-and-cli
