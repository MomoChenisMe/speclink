## MODIFIED Requirements

### Requirement: 外部變更即時反映

<!-- BEFORE: 抽屜「亦同步」語意模糊，實作僅同步標頭計數；內容（任務勾選、文件原文、meta）須重開抽屜才更新；無討論抽屜與互動讓路語意。 -->

桌面 app SHALL 監看目前專案的 openspec/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情抽屜、討論抽屜與已封存頁 SHALL 在短時間內（秒級）自動更新呈現；已開啟檢視中「已載入的內容」——任務清單勾選狀態、proposal／design／specs 文件原文、meta 開工歸屬、討論記錄各分頁——SHALL 重載至磁碟現況，SHALL NOT 要求重啟、重開抽屜或任何 app 內操作。使用者互動進行中（任務勾選寫回、拖曳排序）時外部觸發的內容重載 SHALL 讓路，互動結束後 SHALL 補一次重載至磁碟現況——SHALL NOT 打斷或蓋掉進行中的操作；重載回應交錯時 SHALL 以最新一次為準，較舊回應 SHALL NOT 覆蓋較新內容。app 內操作（勾任務、拖曳、動詞）後 SHALL 重載受影響的已載入內容——含任務清單與 meta。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

#### Scenario: 外部勾選任務後看板自動更新

- **WHEN** 桌面 app 執行中，於外部終端執行 speclink task done 勾掉某 change 的一項任務
- **THEN** 數秒內該 change 的看板卡片任務數與進度條更新；抽屜若開啟，標頭計數與任務清單中該項的核取方塊皆同步至磁碟狀態，全程無任何 app 內操作

#### Scenario: 外部蓋開工章後抽屜出現開工歸屬

- **WHEN** 某 change 的詳情抽屜開啟中，於外部終端執行 speclink in-progress add 該 change
- **THEN** 數秒內抽屜出現開工者與開工日，無需重開抽屜

#### Scenario: 外部推進討論後抽屜內容更新

- **WHEN** 某討論的抽屜開啟中，於外部終端執行 speclink discuss add-round 該討論
- **THEN** 數秒內抽屜的回合分頁出現新回合內容，標頭回合數與其一致

#### Scenario: 互動進行中外部重載讓路

- **WHEN** 使用者正在拖曳某 change 的任務（尚未放開），外部寫者同時修改該 change 的文件
- **THEN** 拖曳互動不被打斷、拖曳視覺不重置；放開完成後數秒內，抽屜內容重載至磁碟現況

#### Scenario: 外部新增與歸檔反映到清單

- **WHEN** 於外部以 CLI 建立新 change，隨後將另一 change 歸檔
- **THEN** 數秒內看板出現新 change 卡片，被歸檔者自看板消失並出現於已封存頁

#### Scenario: 監看不可用時功能照常

- **WHEN** 檔案監看因環境因素無法建立
- **THEN** app 啟動與所有查詢、操作照常運作，錯誤僅記錄於日誌，畫面無錯誤彈窗堆疊
