## Summary

把桌面 app（apps/desktop）的兩種常駐提示搬到新外殼上該屬的層：專案層的「技能檔過期／缺失／較新」提示從主區頂部橫幅改為專案欄底部、專案設定上方的琥珀淡底提示卡（一句標題、一句說明、›），點開才是帶三列細框卡的確認框（專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數；兩個動作鈕依狀態）；全域的「有新版本」從視窗頂端橫幅改為圖示列底部、齒輪上方的圖示鈕（下載箭頭＋琥珀點、tooltip 帶版號），點開「設定」頁的軟體更新卡，卡內多一列「下載與安裝」依狀態換控制（下載／進度＋取消／安裝並重新啟動），啟動時發現新版本另發一次帶動作鈕的 toast。這是討論 desktop-ui-redesign 的 cut 2 外殼刀的第二塊，站在 desktop-shell-redesign 的圖示列與專案欄上。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境是開著舊專案或等新版本時的每一個畫面。兩個問題：

- 技能檔提示是橫跨主區頂部的橫幅：看板被往下推、各視圖要為它扣高度（spec 還為此寫了欄高算式與捲動釘選），而它其實是「這個專案」的事，離它所屬的層（專案欄）很遠；切到其他專案時又要重新算一次。
- 版本更新通知列橫跨整個視窗最上方，與原生標題列、頂欄疊成三條；提示是全域的事，卻壓在每個專案的畫面上；下載中與待重啟的狀態也沒有地方可以回去看。

外殼刀第一塊把專案欄與圖示列立好之後，兩種提示才有自己的落點；放進同一塊會讓外殼刀超過 25 個任務，所以拆開、緊接著做。設定模式的「更新」頁（版本卡、下載與安裝卡、更新日誌卡、自動檢查卡）是 cut 4 的範圍，本塊只把橫幅的動作搬進既有「軟體更新」卡，cut 4 再把它拆成獨立頁。

## Proposed Solution

1. **技能檔提示卡**：`AssetUpdatePrompt` 不再掛在主區頂部，改為專案欄底部、專案設定項上方的 `AssetNoticeCard`——琥珀淡底、琥珀細框、12px 圓角，琥珀圖示＋一句標題（過期：「技能檔是舊版」；缺失：「還沒安裝技能檔」；較新：「技能檔比 Speclink 新」）＋一句說明（過期／缺失：「先更新再改寫，否則 N 個檔案會被換回舊內容」／「安裝會新建 N 個檔案」；較新：「更新 Speclink 才能安全改寫」）＋右端 ›；整張可點，開 `AssetNoticeDialog` 確認框：琥珀圖示＋標題＋說明、一張細框卡三列（這個專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數），按鈕依狀態——過期：「稍後」「更新技能檔」、缺失：「稍後」「安裝技能檔」、較新：「保留現狀」「更新 Speclink」（開設定頁軟體更新卡）。「稍後」只關對話框、卡片留著；「保留現狀」寫入既有的略過記憶、卡片消失。更新失敗時錯誤顯示在對話框內原位、可重試。提示卡跨頁常駐（看板、規格、手冊、已封存、專案設定都在）、不佔主區、不隨主區捲動；remote 專案不探測、無卡。其他專案同狀況時，該專案的圖示列方塊角落亮琥珀點（與 remote 離線點同一顆、同色）。
2. **「有新版本」圖示鈕**：`UpdateBanner` 移除；圖示列底部、齒輪上方新增 `UpdateRailButton`——下載箭頭圖示＋右上琥珀點，tooltip「有新版本 X.Y.Z — 到設定 › 軟體更新」，只在更新狀態為 available／downloading／restartPending／error 時出現（idle、checking、upToDate、checkFailed 不出現；下載中圖示改轉圈、待重啟圖示改重啟箭頭、錯誤改紅點）；點擊進設定頁並捲到軟體更新卡。
3. **軟體更新卡加一列**：既有「軟體更新」卡（檢查更新、更新日誌、目前版本、行內狀態）下方加「下載與安裝」列，依狀態換控制：available＝「有新版本 X.Y.Z」＋「下載」鈕；downloading＝不確定進度列＋「取消」（既有狀態機沒有取消事件：取消＝`dismissed` 回 idle，下載結果到達時忽略）；restartPending＝「更新已就緒」＋「安裝並重新啟動」；error＝錯誤訊息＋「重試」（重新檢查）；idle／upToDate／checkFailed／checking＝該列不出現。徵得同意才下載、簽章失敗拒裝、檢查失敗靜默等既有語意不變。
4. **啟動 toast**：啟動或前景重檢發現新版本時發一次 sonner toast「有新版本 X.Y.Z」＋「查看」動作鈕（進設定頁更新卡）＋關閉；同一版本只發一次（執行期記憶、不持久化）；手動檢查不發 toast（結果在卡內行內呈現）。
5. **spec 字面**：「指令檔過期提示」的呈現改為專案欄提示卡＋確認框、刪除欄高算式與「只佔自身高度」條款（提示不在主區）；「指令檔過期提示捲動釘選」改為「提示卡常駐於專案欄、不隨主區捲動」；「桌面自動更新」的呈現改為圖示鈕＋設定卡＋啟動 toast。

不新增或變更任何 CLI 指令、技能或 Agent 指令。設定欄位：無。持久化：略過記憶鍵 `speclink.instructionSkips` 格式不變；toast 的「已提示版本」只存執行期記憶體。相容性：`--json` 與 CLI 輸出不變。

## Non-Goals

- 不建立設定模式的「更新」頁（版本卡、下載與安裝卡、更新日誌卡、自動檢查卡）——cut 4 `desktop-settings-mode`。
- 不改更新狀態機的事件集（不加真正的下載取消；取消＝放棄本次同意）。
- 不改技能檔探測時機與略過記憶的規則，只改呈現與入口。
- 不動 remote stale 橫幅（連線離線／需重新登入）——它是連線層提示，歸屬另議。
- 不動系統匣面板。

## Alternatives Considered

- 維持頂部橫幅提示——看板被推下去，且提示離它所屬的層很遠（討論否決）。
- 只用 toast 提示——會消失，提示必須常駐到使用者處理（討論否決）。
- 先把「更新」頁做出來再放圖示鈕——設定模式整組是 cut 4，等它會讓橫幅多活一刀；既有軟體更新卡加一列即可承接橫幅的全部動作。

## Impact

- Affected specs: `desktop-app`（MODIFIED「指令檔過期提示」——呈現改專案欄提示卡＋確認框、刪欄高條款；MODIFIED「指令檔過期提示捲動釘選」——改為常駐專案欄不隨捲動；MODIFIED「桌面自動更新」——呈現改圖示鈕、設定卡「下載與安裝」列與啟動 toast）
- Affected code:
  - Modified: `apps/desktop/src/App.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/components/ProjectRail.tsx`、`apps/desktop/src/components/ProjectColumn.tsx`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/appSettingsView.test.tsx`、`apps/desktop/src/__tests__/projectRail.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`
  - New: `apps/desktop/src/components/AssetNoticeCard.tsx`、`apps/desktop/src/components/AssetNoticeDialog.tsx`、`apps/desktop/src/components/UpdateRailButton.tsx`、`apps/desktop/src/components/UpdateInstallRow.tsx`、`apps/desktop/src/__tests__/assetNotice.test.tsx`、`apps/desktop/src/__tests__/updateRailButton.test.tsx`、`apps/desktop/src/__tests__/updateInstallRow.test.tsx`
  - Removed: `apps/desktop/src/components/AssetUpdatePrompt.tsx`、`apps/desktop/src/components/UpdateBanner.tsx`、`apps/desktop/src/__tests__/assetUpdatePrompt.test.tsx`、`apps/desktop/src/__tests__/updateBanner.test.tsx`（案例搬到新測試）
