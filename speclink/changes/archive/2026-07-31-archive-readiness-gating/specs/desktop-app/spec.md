## MODIFIED Requirements

### Requirement: 拖曳封存落點以浮層呈現

拖曳看板卡片時的封存落點 SHALL 以浮層呈現:疊於看板欄列右緣上方、不參與欄列佈局——落點浮現與消失時各欄寬度 SHALL 維持不變。落點 SHALL 僅於拖曳**已就緒**變更卡時浮現;拖曳非已就緒變更卡或討論卡時 SHALL NOT 浮現,且 SHALL NOT 造成任何佈局變動——非已就緒變更卡的拖曳僅得欄內排序。拖曳靠近落點時 SHALL 呈現可放開的視覺回饋。已就緒卡拖至落點放開的封存確認流程與拖排語意 SHALL 維持 board-card-order 規格所定行為不變。

#### Scenario: 拖曳已就緒變更卡時浮層浮現且欄寬不變

- **WHEN** 使用者開始拖曳一張已就緒的變更卡
- **THEN** 封存落點以浮層疊於看板右緣浮現,各欄(含討論欄)寬度與拖曳前一致;放開或取消拖曳後浮層消失、欄寬仍不變

#### Scenario: 拖曳非已就緒變更卡時落點不浮現

- **WHEN** 使用者開始拖曳一張提案中或進行中的變更卡
- **THEN** 封存落點不浮現,看板佈局無任何變動;放開時僅依欄內拖排語意處理

#### Scenario: 拖曳討論卡時落點不浮現

- **WHEN** 使用者開始拖曳討論卡
- **THEN** 封存落點不浮現,看板佈局無任何變動

#### Scenario: 已就緒卡拖至浮層落點放開觸發既有封存流程

- **WHEN** 使用者拖曳已就緒變更卡至浮層落點放開
- **THEN** 觸發既有封存確認流程,行為與未改版前一致

## ADDED Requirements

### Requirement: 詳情抽屜的封存與刪除依階段守門

詳情抽屜的封存鈕 SHALL 僅於該 change 派生階段為已就緒時可按;非已就緒時 SHALL 呈現 disabled 並以 tooltip 說明原因(載明任務進度與「完成後才能封存」的出路)。刪除鈕 SHALL 僅於派生階段為提案中時可按;非提案中時 SHALL 呈現 disabled 並以 tooltip 說明原因(載明已有開工痕跡與「先退回提案中」的出路)。remote session 下對應能力缺失時,能力缺失原因 SHALL 優先於階段原因呈現。守門為呈現層過濾:可按期間階段已於外部改變(併發寫入)時,引擎拒絕 SHALL 為最終裁決,依既有失敗 toast 語意呈現。桌面 SHALL NOT 提供任何略過守門的強制通道。

#### Scenario: 非已就緒的封存鈕停用附原因

- **WHEN** 使用者開啟提案中(0/11)與進行中(5/19)change 的詳情抽屜
- **THEN** 兩者封存鈕皆 disabled,tooltip 各載明任務進度與完成後才能封存;已就緒 change 的封存鈕照常可按並走既有確認流程

#### Scenario: 非提案中的刪除鈕停用附原因

- **WHEN** 使用者開啟進行中 change 的詳情抽屜
- **THEN** 刪除鈕 disabled,tooltip 載明已有開工痕跡並指向先退回提案中;提案中 change 的刪除鈕照常可按

#### Scenario: 併發階段變化由引擎拒絕保底

- **WHEN** 封存鈕可按期間另一 session 於 tasks.md 取消勾選一個任務,使用者隨後確認封存
- **THEN** 引擎拒絕封存,app 依既有失敗 toast 語意呈現拒絕訊息,change 仍在看板

### Requirement: 桌面刪除變更走 discard 全語意

桌面 app 的本地刪除變更 SHALL 執行引擎 discard 全語意(不帶強制):開工痕跡守門、來源討論解鏈與 touched 紀錄清理——SHALL NOT 直接刪除目錄繞過任一環節。刪除一個由討論轉出的 change 後,來源討論的已轉出清單 SHALL 同步移除該 change 名,清單空時討論狀態 SHALL 回復。對有開工痕跡(meta 含 started_at 或任一任務已勾)的 change,刪除 SHALL 被引擎拒絕且無任何檔案改動。remote session 的刪除 SHALL 以不帶強制的語意呼叫 server 刪除端點,已開工 change 的刪除由 server 拒絕,依既有失敗 toast 語意呈現。

#### Scenario: 刪除轉出 change 連帶解鏈來源討論

- **WHEN** 使用者刪除一個由討論轉出、無開工痕跡的提案中 change(該討論僅轉出此一 change)
- **THEN** change 目錄消失、其 touched 紀錄清除;來源討論的已轉出清單移除該名稱且狀態回復,看板重載後討論卡如實呈現

#### Scenario: 有開工痕跡的本地刪除被拒

- **WHEN** 使用者對 meta 含 started_at 的 change 觸發刪除並確認(UI 停用未及反應的併發情境)
- **THEN** 引擎拒絕,openspec/ 下任何檔案逐位元不變,app 依既有失敗 toast 語意呈現拒絕訊息

#### Scenario: remote 刪除已開工 change 被 server 拒絕

- **WHEN** 使用者於 remote 分頁對已勾任務的 change 觸發刪除並確認
- **THEN** server 以需要強制的拒絕語意回應,scope 內容零改動,app 依既有失敗 toast 語意呈現
