## MODIFIED Requirements

### Requirement: 指令檔過期提示
<!-- BEFORE: 只對本地專案探測；remote 專案不觸發；確認框三列 -->
<!-- REMOVED-SCENARIO: remote 分頁不觸發 -->

桌面 app SHALL 於本地專案成為作用中時執行指令檔過期探測,並於更新動作完成後與 workspace-changed 事件到達時重新探測。有 checkout 的 remote 專案成為作用中時 SHALL 同樣探測,基準改為 Server 提供的技能:Server 宣告技能時,以 Server 回應的技能摘要與 checkout 裡各工具的技能檔比對——任一工具沒有任何技能檔為缺失,任一技能的內容不同、缺少或多出為過期,全部相同為現版;Server 對某個工具的回應不含任何技能時,該工具 SHALL NOT 參與比對——不算缺失、也不算過期,它既有的技能檔不列入會改寫的檔案;每個工具都是如此時為現版。Server 沒有宣告技能、無法連線或沒有 checkout 時 SHALL 無任何提示,且 SHALL NOT 記入略過。有 checkout 的 remote 專案為作用中時,app SHALL 監看 checkout 裡各工具的技能目錄與 `.speclink.yaml`,它們變動時 SHALL 重新探測,不需要切換分頁;這個監看 SHALL NOT 觸發專案資料的重新載入,切到其他分頁後 SHALL 停止。remote 專案的探測 SHALL NOT 下載技能全文。探測回報過期、缺失或較新、且該專案未略過當前產物層版號時,SHALL 於專案欄底部、專案設定項上方呈現一張琥珀淡底的提示卡(琥珀圖示、一句標題、一句說明、右端 ›);提示卡 SHALL 於該專案的所有頁面(變更、已封存、規格、手冊、專案設定)常駐,SHALL NOT 佔用主區、SHALL NOT 以 modal 阻斷操作,SHALL NOT 於沒有 checkout 的 remote 專案觸發探測。探測回報現版或無法判定時 SHALL 無任何提示;無法判定 SHALL NOT 記入略過、SHALL NOT 視同現版。其他開著的本地專案上次探測為過期、缺失或較新時,其圖示列方塊角落 SHALL 亮琥珀點,提示附「技能檔需要處理」;背景專案 SHALL NOT 主動探測。

點提示卡 SHALL 開啟確認框:琥珀圖示、標題、說明,一張細框卡三列(這個專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數——較新態第三列為會被換回舊內容的檔案數),按鈕依探測狀態:過期為「稍後」「保留現狀」「更新技能檔」、缺失為「稍後」「保留現狀」「安裝技能檔」(從未安裝的專案不以「更新」稱之)、較新為「保留現狀」「更新 Speclink」。較新時 SHALL 以「app 本體是舊版、專案檔案較新」語意呈現,SHALL NOT 提供任何改寫專案檔案的動作,「更新 Speclink」SHALL 開啟應用程式設定的更新頁。「稍後」SHALL 只關閉確認框、提示卡維持。remote 專案的確認框 SHALL 多一列「Server 提供的技能版本」;「更新技能檔」與「安裝技能檔」SHALL 向 Server 取得技能全文並寫入 checkout,寫入後的技能檔 SHALL 等於 Server 的回應。app 內建的技能版號舊於 Server 宣告的版號時,remote 專案的確認框 SHALL 多一句說明與「更新 Speclink」按鈕,原有的安裝或更新按鈕 SHALL 仍然可用。remote 專案的略過記憶 SHALL 以該專案與 Server 提供的技能版本為鍵。

「更新技能檔」與「安裝技能檔」SHALL 經引擎既有的指令檔再生入口整套再生受管檔,成功後提示卡與確認框消失;失敗時錯誤 SHALL 呈現於確認框內原位且可重試,SHALL NOT 阻斷專案其他功能。「保留現狀」SHALL 將該專案與當前產物層版號記入 app 本機持久化,SHALL NOT 寫入專案內任何檔案;同版不再提示(缺失、過期與較新共用同一略過記憶),產物層版號變動後 SHALL 重新提示。提示文案 SHALL 遵循 speclink/LANGUAGE.md 詞彙原則,SHALL NOT 出現工程詞。主區各視圖的高度 SHALL NOT 因提示存在而改變。

#### Scenario: 開啟過期專案出現提示並更新

- **WHEN** 以新版 desktop 開啟指令檔為舊版的本地專案,點專案欄底部的提示卡,於確認框點「更新技能檔」
- **THEN** 提示卡先於專案欄底部呈現(標題「技能檔是舊版」、說明含將被改寫的檔案數);確認框三列顯示專案技能檔版本、Speclink 版本與檔案數;更新後受管檔再生為現版、確認框與提示卡消失

#### Scenario: 檔案領先 app 時不提供改寫動作

- **WHEN** 以舊版 desktop(引擎產物層版號舊於專案指令檔標記版號)開啟該本地專案並點提示卡
- **THEN** 提示卡以「技能檔比 Speclink 新」呈現;確認框說明以「app 本體是舊版」語意引導更新 app,按鈕只有「保留現狀」與「更新 Speclink」,無「更新技能檔」「安裝技能檔」;「更新 Speclink」開設定的更新頁;專案內無任何檔案因此變動

#### Scenario: 較新提示保留現狀後同版不再問

- **WHEN** 使用者於較新態的確認框點「保留現狀」,其後關閉並重新開啟同一專案
- **THEN** 提示卡不再出現;app 升版使產物層版號變動後重新探測、依新狀態重新裁決

#### Scenario: 保留現狀同版不再問

- **WHEN** 使用者於過期態的確認框點「稍後」再點提示卡改點「保留現狀」,其後關閉並重新開啟同一專案
- **THEN** 「稍後」後提示卡仍在;「保留現狀」後提示卡消失且重開不再出現,專案內無任何檔案因此變動

#### Scenario: 產物層版號變動後重新提示

- **WHEN** 曾保留現狀的專案,在 desktop 升版且產物層版號變動後再次開啟
- **THEN** 提示卡重新出現

#### Scenario: 標記移除不提示

- **WHEN** 開啟指令檔已整塊移除 SPECLINK 標記的本地專案
- **THEN** 無任何提示卡

#### Scenario: 從未安裝的專案提示安裝

- **WHEN** 開啟 tools 清單宣告 claude 但 CLAUDE.md 不存在(如 clone 後指令檔未進版控)的本地專案,點提示卡並於確認框點「安裝技能檔」
- **THEN** 提示卡以「還沒安裝技能檔」呈現、說明含將被新建的檔案數;安裝後受管檔生成為現版、提示卡消失

#### Scenario: 更新失敗原位呈現錯誤

- **WHEN** 更新動作因受管檔無法寫入而失敗
- **THEN** 錯誤訊息呈現於確認框內容卡下方、動作鍵恢復可按可重試,專案其餘功能不受影響

#### Scenario: 外部更新自然消提示

- **WHEN** 提示卡顯示期間,使用者於終端執行 speclink update 使受管檔更新為現版
- **THEN** workspace-changed 事件觸發重新探測後提示卡消失,無需重開專案

#### Scenario: 提示卡跨頁常駐且主區不扣高度

- **WHEN** 技能檔提示卡存在時依序切到規格頁、手冊頁、已封存頁、專案設定頁
- **THEN** 提示卡始終在專案欄底部同一位置;主區各視圖的高度與無提示時相同,看板欄底緣仍貼齊主區底緣

#### Scenario: 其他專案的狀況以琥珀點提示

- **WHEN** 開著專案 A(技能檔舊版)與 B(現版),作用中切到 B
- **THEN** A 的圖示列方塊角落亮琥珀點、提示附「技能檔需要處理」;B 無點;專案欄無提示卡;切回 A 後重新探測並顯示提示卡

#### Scenario: remote 專案的技能與 Server 不同時提示並更新

- **WHEN** 有 checkout 的 remote 專案成為作用中，checkout 的一支技能內容與 Server 提供的不同
- **THEN** 出現提示卡；確認框有「Server 提供的技能版本」一列與「更新技能檔」按鈕；按下後 checkout 的技能檔等於 Server 的回應，提示卡消失

#### Scenario: remote 專案沒有技能檔時提示安裝

- **WHEN** 有 checkout 的 remote 專案成為作用中，checkout 沒有任何 speclink- 技能檔
- **THEN** 確認框的主要按鈕是「安裝技能檔」

#### Scenario: app 比 Server 舊

- **WHEN** app 內建的技能版號舊於 Server 宣告的版號，remote 專案的技能需要更新
- **THEN** 確認框有一句說明 app 較舊，並同時有「更新技能檔」與「更新 Speclink」兩個按鈕

#### Scenario: Server 對某個工具不提供任何技能時不提示

- **WHEN** 有 checkout 的 remote 專案的 tools 為 claude 與 codex;claude 的技能與 Server 提供的相同,Server 對 codex 回應空的清單,而 checkout 的 codex 技能目錄有既有的技能檔(或沒有任何技能檔)
- **THEN** 沒有提示卡;codex 的技能檔不列入會改寫的檔案

#### Scenario: remote 專案的技能檔被外部改動後提示自動更新

- **WHEN** 有 checkout 的 remote 專案為作用中且技能與 Server 相同,使用者在終端機改動 checkout 裡的一個技能檔
- **THEN** 不需要切換分頁,提示卡出現;把那個檔案改回 Server 的內容後,提示卡消失;過程中專案的變更清單沒有重新載入

#### Scenario: 沒有 checkout 的 remote 分頁不觸發

- **WHEN** 只看規格、沒有綁定 checkout 的 remote 專案成為作用中
- **THEN** 不執行探測，沒有提示

#### Scenario: Server 沒有宣告技能時不提示

- **WHEN** 有 checkout 的 remote 專案連到一台握手沒有宣告技能的 Server
- **THEN** 沒有提示，也沒有任何略過被記下

#### Scenario: remote 探測失敗不提示

- **WHEN** 探測時 Server 不可達
- **THEN** 沒有提示；連線恢復後下一次探測照常進行
