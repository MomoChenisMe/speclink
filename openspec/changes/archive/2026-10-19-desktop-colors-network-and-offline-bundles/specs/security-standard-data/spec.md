## MODIFIED Requirements

### Requirement: 受限 ZIP 與安全 XML 匯入
<!-- BEFORE: 僅定義formatVersion=1單版本ZIP，尚無集合包。 -->

單一版本離線包 SHALL 為 formatVersion=1 的 ZIP，根 manifest.json 列 standardId／version／sourceRevision／授權與 entries 的 name／sha256，內容僅列明 raw 資料及 LICENSE。SHALL 在寫入前檢查全部 entry，拒絕絕對／點點／盤符／反斜線路徑、symlink、executable、重名／大小寫碰撞、未列檔案與指紋不符，不任意 extract。XML SHALL 拒絕 DTD、外部與非內建實體，不取外部 schema。匯入 SHALL 標 imported，包內自填 checksum 不提供官方可信來源證明。

#### Scenario: Zip Slip 與實體攻擊

- **WHEN** 包內有 ../outside、C:\\secret、symlink、重名或 XML 外部實體
- **THEN** 回 invalid_package，不在應用程式受控暫存外寫檔、不讀外部實體、不取網路，舊資料不變

#### Scenario: 離線匯入合法資料

- **WHEN** 無網路時匯入版本、指紋、結構及授權皆有效的包
- **THEN** 安裝成功、來源種類為 imported、可供離線查詢，清單與 metadata 顯示正確版本

官方 CWE 內層封裝若將 regular XML 設為可執行權限（已核實4.20為100750），專用解析路徑 SHALL 只接受單一、名稱符合所選版本的 regular XML，以有界 bytes 解析且不沿用原權限；所有連結／特殊檔仍拒絕，外層離線包的可執行項目限制不變。SHALL NOT 解壓至任意路徑或執行 XML 內容。

#### Scenario: 官方 CWE XML 帶執行權限位元

- **WHEN** 固定版本的官方 CWE ZIP 僅有 cwec_v4.20.xml、regular mode 100750，內容通過4.20根版本及結構驗證
- **THEN** 只將 XML 當資料保存，檔案不帶封裝執行權限；包內不是XML、其他名稱、多檔、連結及特殊檔一律拒絕

formatVersion=2集合包 SHALL 依「離線集合包匯入與逐版本結果」處理，內含單包仍逐一遵守本需求全部限制，不將外層格式放寬套用到單包。


### Requirement: 資料管理 CLI 輸入輸出
<!-- BEFORE: 資料管理只有list／refresh／install／單包import／remove，import固定回單一receipt。 -->

security data SHALL 提供 list [--json]、refresh [--json]、install STANDARD --version VERSION [--reinstall] [--json]、import PATH [--json]、remove STANDARD --version VERSION [--json]；無 stdin、支援 --no-color，成功0、錯誤1、取消130、clap 語法錯誤2。list／refresh stdout JSON SHALL 為 {standards:[{standardId:string,versions:[{version:string,status:string,sourceUrl:string,licenseName:string,licenseUrl:string,attribution:string,sourceFormat:string}]}]}。install／單包import SHALL 回 {standardId,version,installedStatus,sha256,sourceRevision,integrityKind} 字串欄位；remove 回 {standardId:string,version:string,removed:boolean}。人眼 SHALL 顯示同資訊，進度與錯誤至 stderr，--json stdout 不混進度；installedStatus SHALL 區分 missing／installed／invalid。

#### Scenario: 清單與安裝 JSON

- **WHEN** 執行 list --json --no-color，然後明確 install owasp_top10 --version 2025 --json
- **THEN** stdout 各為單一符合指定型別的 JSON，無 ANSI，進度在 stderr；安裝成功為 installed 且有內容指紋

#### Scenario: 不存在或壞包

- **WHEN** import 指定不存在的檔案或 install 指定不支援版本
- **THEN** exit 1，stderr 指出原因，stdout 沒有成功 payload，現有清單及已安裝資料不變

CLI SHALL 新增「離線資料包CLI匯出與集合匯入」規定的export入口，既有import PATH亦接受集合包並依該需求輸出逐項結果；單包人眼與JSON輸出保持既有位元級契約，不強制舊包轉換。


## ADDED Requirements

### Requirement: 已安裝標準資料的離線匯出

Desktop SHALL 在安全標準資料頁提供每個有效已安裝標準版本的「匯出」及頁級「全部匯出」，以原生儲存選擇器選路徑。單包 SHALL 沿用formatVersion=1，含manifest.json、已列raw資料與LICENSE及來源／授權／署名／sourceRevision／SHA資訊；SHALL NOT 包含parsed.json、鎖、generation、專案內容或把自算指紋當官方認證。匯出 SHALL 完全離線，不安裝、不補下載、不更動已安裝資料與current。

全部匯出 SHALL 凍結起始清單，按standardId與version字串順序選全部有效installed版本，invalid列入skipped並顯示原因，missing不補裝。沒有有效版本 SHALL 回missing_data且不輸出空包；已選版本讀取／指紋驗證失敗 SHALL 整次失敗，不少包冒稱全部。結果 SHALL 為{path:string,formatVersion:number,bytes:number,packages:[{standardId:string,version:string,sha256:string}],skipped:[{standardId:string,version:string,reason:string}]}。

匯出 SHALL 保留完整同代資料，寫同目錄暫存、完成並sync後無覆寫原子發佈。已存在回file_exists；取消或寫入／容量失敗 SHALL 清理暫存、不覆寫既有目的檔、不留完成包；不支援原子無覆寫發佈的檔案系統 SHALL 明確失敗。SHALL 提供進度及取消，不在UI主執行緒組包。

#### Scenario: 單一ASVS版本離線交換
- **WHEN** 在離線電腦匯出已安裝的ASVS 5.0.0，再於另一台離線電腦匯入
- **THEN** 得到可依既有v1匯入的ZIP，另一台可查詢相同版本，來源授權與raw指紋一致，匯入來源標imported；原電腦current與專案檔案不變

#### Scenario: 全部選取與無資料
- **WHEN** 本機有兩個有效已安裝版本及一個invalid版本而選全部匯出，或完全沒有有效版本
- **THEN** 前者包內精確含兩個版本，invalid逐項顯示在skipped；後者顯示missing_data，不輸出空包、不發網路請求

#### Scenario: 取消及目的檔保留
- **WHEN** 使用者取消儲存、取消組包，或目的檔已存在／磁碟不足
- **THEN** 不顯示匯出成功，原檔及已安裝資料保持，暫存清理，錯誤或取消明示

### Requirement: 離線集合包匯入與逐版本結果

全部匯出 SHALL 為集合ZIP，根manifest.json為{formatVersion:2,packages:[{standardId:string,version:string,name:string,sha256:string}]}，name精確為packages/<standardId>/<version>.zip，每個內包為v1。SHALL 最多32個內包、33個outer entries、64KiB manifest、outer及內包總bytes最多512MiB；每個v1維持64MiB壓縮／256MiB展開／128entries限制，全部內包raw累計最多1GiB。SHALL 按實際讀取核對，拒絕重複版本、遞迴集合、未列entry、指紋不符、encrypted、ZIP64、連結、可執行項、路徑穿越及大小寫碰撞，不任意extract。

既有匯入入口 SHALL 自動辨識v1或v2。v2 SHALL 預檢所有包的格式、SHA、來源、授權及可解析性，通過前不得提交任一版本；通過後依manifest順序逐版本原子提交。任一提交失敗或取消 SHALL 停止，已成功版本保留、當前失敗版本保舊、後續notAttempted；清理警告不撤銷已提交。SHALL 明確呈現逐項結果，不能用單一成功訊息掩蓋partial。所有匯入來源皆標imported，不因匯出機器原本下載來源而升級信任。既有單包輸出與保舊行為不變。

#### Scenario: 全部匯出一次匯入
- **WHEN** 使用者將含OWASP Top 10 2025及ASVS 5.0.0的集合包帶到無網路裝置匯入
- **THEN** 同一入口安裝兩版本並列逐項成功，資料皆可離線查詢、來源均imported，未更動專案政策

#### Scenario: 內包損壞預檢不寫入
- **WHEN** 第二個內包SHA錯誤、缺授權或含../路徑
- **THEN** 整包預檢失敗，第一個版本也沒有提交，兩版本既有current與資料不變

#### Scenario: 中途提交失敗與取消
- **WHEN** 三包預檢通過、第一包完成、第二包遇權限錯誤或取消
- **THEN** 第一包保留成功，第二包舊current保持，第三包未執行，頁面顯示partial或cancelled及逐項結果

### Requirement: 離線資料包CLI匯出與集合匯入

CLI SHALL 提供security data export [STANDARD --version VERSION | --all] --output PATH [--json]，STANDARD與--all互斥、--version只搭STANDARD、--output必要，無stdin，支援global --no-color。成功exit0、操作失敗1、取消130、參數錯誤2；人眼stdout列路徑／版本／跳過項，JSON stdout為「已安裝標準資料的離線匯出」規定的單物件；進度與錯誤至stderr，失敗無成功payload。--no-color SHALL 不含ANSI，不改JSON欄位。

security data import PATH [--json] SHALL 維持v1人眼及JSON位元級輸出；v2在提交開始後回{formatVersion:2,status:"installed"|"partial"|"failed"|"cancelled",results:[{standardId:string,version:string,status:"installed"|"failed"|"notAttempted",sha256:string|null,errorCode:string|null}]}。JSON結果至stdout，人眼stdout列同資訊；全部完成exit0、提交失敗exit1、取消exit130，進度與錯誤至stderr；預檢失敗stdout空且exit1，預檢取消stdout空且exit130。SHALL 不接受stdin、不修改專案文件。新CLI人眼文案維持英文，JSON鍵camelCase不翻譯；tw／ja／en或未設政策及中文弱語言偵測行為不變。Remote專案離線或認證失效不妨礙這些裝置資料操作，不發revision請求。

#### Scenario: 人眼與JSON單包匯出
- **WHEN** 離線執行security data export owasp_asvs --version 5.0.0 --output /tmp/asvs.zip --json --no-color，或省略--json使用另一個未存在路徑
- **THEN** exit0，前者stdout為含path／formatVersion／bytes／packages／skipped且型別正確的單JSON、無ANSI與進度，後者stdout列相同資訊；進度只在stderr

#### Scenario: CLI參數及目的檔錯誤
- **WHEN** 同時指定STANDARD與--all，或匯出到已存在目的檔
- **THEN** 前者exit2、後者exit1且stderr指出file_exists，stdout沒有成功payload，目的檔保持原內容

#### Scenario: 集合匯入部分成功JSON
- **WHEN** 以--json匯入三包，預檢通過後第一包成功、第二包寫入失敗
- **THEN** exit1，stdout單JSON的status為partial、results依序installed／failed／notAttempted，sha256成功項為string而其餘為null，錯誤項errorCode為string；stderr呈錯誤，無ANSI或混入進度
