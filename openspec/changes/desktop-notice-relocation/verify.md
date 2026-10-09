# Verify — desktop-notice-relocation

## Round 1

**Phase**: discovery
**Patch**: sha256:ff1f6cfc40f21d42f5adc2de99dbf5362590261dcbf980f4962fe45c9526ff68
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/__tests__/assetNotice.test.tsx, apps/desktop/src/__tests__/assetUpdatePrompt.test.tsx, apps/desktop/src/__tests__/projectRail.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/__tests__/updateBanner.test.tsx, apps/desktop/src/__tests__/updateInstallRow.test.tsx, apps/desktop/src/__tests__/updateRailButton.test.tsx, apps/desktop/src/__tests__/updater.test.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/components/AssetNoticeCard.tsx, apps/desktop/src/components/AssetNoticeDialog.tsx, apps/desktop/src/components/AssetUpdatePrompt.tsx, apps/desktop/src/components/ProjectColumn.tsx, apps/desktop/src/components/ProjectRail.tsx, apps/desktop/src/components/UpdateBanner.tsx, apps/desktop/src/components/UpdateInstallRow.tsx, apps/desktop/src/components/UpdateRailButton.tsx, apps/desktop/src/core/updater.ts, apps/desktop/src/i18n/messages.ts, apps/desktop/src/store.ts, apps/desktop/src/views/AppSettingsView.tsx
- [WARNING] apps/desktop/src/core/updater.ts — Correctness: 按「取消」只把狀態改回閒置（core/updater.ts:64-69）。`acceptUpdate`（store.ts:1849-1866）還在等 `downloadAndInstall()`，外掛下載完照樣會安裝；Windows 上還會啟動安裝程式並結束 app。spec 寫「取消即放棄本次同意、回到閒置」「SHALL NOT 靜默安裝」，這個行為和它的本意不合；design 的 Risks 只寫到「外掛仍在背景下載」，沒提安裝也會落地。建議：adapter 改成先 `download()`、再 `install()`（plugin-updater 2.x 兩支都有），下載完成時如果狀態已經不是 downloading 就不呼叫 `install()`；或者在 spec／design 明寫「取消後安裝仍會發生」。
- [WARNING] apps/desktop/src/__tests__/updater.test.ts — Correctness: Example「開著三小時後切回」的 THEN 這次加了「圖示列更新鈕提示顯示 0.5.1、toast 顯示 0.5.1」，但 updater.test.ts:275-296 只斷言狀態是 available 0.5.1，沒驗 toast 和更新鈕。store.test.ts 的 toast 案例是直接呼叫 `checkForUpdates(false)`，沒走 12:00 前景重檢那條路。建議：這個案例 mock sonner，並斷言 toast 帶 0.5.1。
- [SUGGESTION] apps/desktop/src/store.ts — Correctness: Scenario「背景檢查的 toast 只發一次」寫的是「關掉 toast 但未下載、一小時後前景重檢」。可是待同意狀態下 `focusRecheckAllowed` 會擋掉前景重檢，所以這條路實際走不到；store.test.ts:1833-1843 是直接呼叫 `checkForUpdates(false)` 繞過去的。建議 scenario 改成「取消下載回閒置後再前景重檢」，測試改走 `dismissUpdate` → `recheckOnFocus`。
- [SUGGESTION] apps/desktop/src/core/updater.ts — Coherence: design D3 寫「在 `acceptUpdate` 內以 phase 檢查實作」，實際做法是讓 reducer 的 `dismissed` 也接受 downloading，`acceptUpdate` 沒有改。行為一樣、事件集也沒變。建議改 design D3 的字面，對齊實作。
- [SUGGESTION] apps/desktop/src/components/UpdateRailButton.tsx — Correctness: spec 寫「提示文字 SHALL 帶目標版本與『到設定 › 軟體更新』」，只有待同意態符合。下載中、待重啟、失敗三態照 design D2 用了別的文案，其中失敗態「更新失敗 — 到設定查看」沒有版號（失敗狀態本來就不帶版號）。建議把 spec 這句限定為待同意態，或改成逐態列出文案。
- [SUGGESTION] apps/desktop/src/i18n/messages.ts — Correctness: 過期態卡片的說明是「先更新再改寫，否則 {count} 個檔案會被換回舊內容」。「換回舊內容」看起來描述的是較新態的風險，和 scenario「說明含將被改寫的檔案數」的意思不合（字面是照 proposal 抄的）。建議確認原意；如果確實不對，改成類似「更新會改寫 {count} 個檔案」，並同步改 proposal。
- [SUGGESTION] apps/desktop/src/components/UpdateInstallRow.tsx — Coherence: 失敗態只有「重試」，沒有關閉鈕。但 Example「節流邊界」的備註還寫「錯誤訊息留到使用者關閉」，現在已經沒有對應的控制，圖示列的紅色警示只能靠重試清掉。建議把備註改成「留到使用者重試」，或加一顆關閉鈕。
- [SUGGESTION] apps/desktop/src/__tests__/App.test.tsx — Coherence: Implementation Contract 的驗收寫 `grep -rn "update-banner\|asset-prompt\"" apps/desktop/src` 要零命中，實跑會命中 App.test.tsx:1362 的反向斷言 `queryByTestId("asset-prompt")`（tasks 1.3 用的是比較窄的 grep，那支會過）。建議刪掉這行斷言（緊接的 `main().querySelector` 已經涵蓋），或把 design 的 grep 改成 tasks 1.3 的版本。
- [SUGGESTION] apps/desktop/src/__tests__/App.test.tsx — Correctness: Scenario「提示卡跨頁常駐且主區不扣高度」要依序切四頁：規格、手冊、已封存、專案設定。App.test.tsx:1366-1378 的迴圈少了手冊頁，而舊實作在手冊頁還有一段特例處理。建議把手冊頁加進迴圈。
- [SUGGESTION] apps/desktop/src/components/AssetNoticeDialog.tsx — Coherence: 確認框改成三顆按鈕以後，artifact 還留著「兩鍵」：spec scenario「更新失敗原位呈現錯誤」寫「兩鍵恢復可按」，design D5 寫「依狀態的兩鍵」，Failure modes 也有，tasks 1.2 寫「busy 兩鍵 disabled」，3.1 寫「三列與兩鍵」。實作和測試其實是三顆都會停用、也都會恢復。建議把這些地方統一改成「動作鍵」。

## Round 2

**Phase**: validation
**Patch**: sha256:b91e9f612a493b1131a4c92f53bff87898394e1bf195751d0ac07da7fb41bd48
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/assetNotice.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/__tests__/updateRailButton.test.tsx, apps/desktop/src/__tests__/updater.test.ts, apps/desktop/src/adapter/updater.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/components/AssetNoticeDialog.tsx, apps/desktop/src/components/ProjectRail.tsx, apps/desktop/src/components/UpdateInstallRow.tsx, apps/desktop/src/components/UpdateRailButton.tsx, apps/desktop/src/core/updater.ts, apps/desktop/src/i18n/messages.ts, apps/desktop/src/store.ts, apps/desktop/src/views/AppSettingsView.tsx
- [SUGGESTION] apps/desktop/src/assetPrompt.ts — Coherence: 這次修補把 `projectVersion` 的取法改了：較新態取領先的那個工具、過期態取落後的那個工具、缺失態一律是 null。但 design.md D1 第二點和 tasks.md 1.1 都沒跟著改，還寫「自 `AssetProbeResult.tools[].workspaceVersion` 取第一個非 null 值填入」，和現在的程式對不上。建議把這兩處改成「取觸發該狀態的工具的標記版號」。
- [SUGGESTION] apps/desktop/src/App.tsx — Coherence: 這次修補從 store 拿掉了 `assetPrompt` 欄位，改用 `selectAssetPrompt(state)` 推算。design.md D1 有一點已經同步，但同一節的「落點」那點還寫 `App.tsx` 傳 `<AssetNoticeCard prompt={s.assetPrompt} …/>`，tasks.md 1.1 也還寫「`assetPrompt` 改為作用中分頁的派生值」。兩處都在講已經不存在的欄位，和同一份 design 裡更新過的那點互相矛盾。建議統一改成 `selectAssetPrompt(s)`。

## Round 3

**Phase**: validation
**Patch**: sha256:a156458dcae908d7f4ca9bccaf0f5b887e57700ebd89efa5c4613942aa4630da
**Scope**: apps/desktop/src/store.ts
