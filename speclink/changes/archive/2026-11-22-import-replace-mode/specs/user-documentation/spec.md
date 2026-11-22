## MODIFIED Requirements

### Requirement: SDK 文件說明伺服端匯入、釋放認領的掛鉤與蓋章指紋

<!-- BEFORE: 重新匯入的步驟是宿主先清空自己儲存裡這個範圍的全部文件再匯入 -->

Node SDK 文件（中英兩版）SHALL 有一段「在伺服端匯入」，接在匯出本機專案的說明之後，並 SHALL 寫明：後端只下載規格資料夾時，`createEngine` 的 `root` 指到它的上一層、`store.specDir` 給資料夾名稱，不給 `specDir` 時讀 `root/.speclink.yaml` 的 `spec_dir`；`root` 底下沒有規格資料夾時 `exportScope` 以 `no spec folder:` 開頭的訊息拒絕；`limits.importBytes` 也管程序內呼叫的 `host.importScope`（預設 32 MiB），匯出沒有上限；匯入是一次性的搬遷，不是同步；要重新匯入，宿主以取代模式呼叫 `host.importScope`（`mode: 'replace'`），先以 `dryRun: true` 試跑；文件 SHALL 寫明取代讓範圍的內容等於匯入包，會刪掉匯入後在宿主裡做的全部改動（變更、討論、蓋章），流程設定會變成 repo 的版本；文件 SHALL 提供一段範例：取代前以 `workflow-config show --json` 讀出目前的流程政策，試跑並把 `removed` 給管理者確認，取代，再以 `workflow-config set` 把宿主管理的政策鍵寫回去，並 SHALL 寫明「先調政策、再匯入」用同一段範例。文件 SHALL NOT 再要求宿主自己清空儲存。

Node SDK 文件的「匯出與匯入」一節 SHALL 說明 `mode` 與 `dryRun`、回報的 `added`、`changed`、`removed`、內容相同的文件不重寫、空的匯入包不能取代、取代期間另一方的寫入有哪些不會被察覺以及宿主怎麼擋住；匯入的原因代碼表 SHALL 有五列，含 `empty_bundle` 與 `scope_changed`。文件 SHALL 寫明取代模式只在程序內的呼叫提供，HTTP 入口的 `POST /import` 只收空的範圍；動詞參考文件（中英兩版）的匯入一節 SHALL 寫同一件事。Node SDK 文件的掛鉤段落 SHALL 寫明 `scope-import` 的參數是 `documents`、`bytes`、`mode`、`dryRun`。

Node SDK 文件的掛鉤段落 SHALL 在封存的範例之後提供「只讓認領的人與管理者釋放認領」的範例：命令名稱是 `claim-release`、變更名稱在 `params.name`；以 `read` 讀 `show --json` 的 `claimed`，有 `byId` 時比對身分的 `id`，沒有時比對名稱；`read` 以 `code` 為 `not_found` 拒絕時放行，由釋放本身回報找不到變更；`read` 的其他錯誤 SHALL 往外拋，這時掛鉤失敗，這次呼叫 SHALL NOT 放行，儲存 SHALL NOT 收到寫入；管理者由宿主自己的函式判斷。範例 SHALL 是可以單獨貼上的函式宣告（`async function beforeCommand(...)`），SHALL NOT 用只在物件字面值裡成立的方法寫法。文件 SHALL 寫明 Speclink 不限定誰可以釋放認領，並 SHALL 寫明掛鉤給的 `reason` 會出現在哪裡：CLI 原文印到 stderr；Speclink Desktop 沒有釋放認領的操作，認領被拒時失敗提示帶出原因。動詞參考文件（中英兩版）的釋放認領一節 SHALL 只保留命令名稱的說明並連到這個範例，SHALL NOT 另留一份範例程式。

Node SDK 文件的掛鉤段落 SHALL 說明如何在封存前比對蓋章時的檔案：以 `read(['show', <變更>, '--json', '--detail'])` 讀 `review.scope` 與 `verify.scope`，逐項比對宿主自己算出的指紋。指紋的算法 SHALL 寫成：能以 UTF-8 解讀的檔，把 CRLF 換成 LF 後取 SHA-256（小寫十六進位）；不能以 UTF-8 解讀的檔，取原始位元組的 SHA-256；開頭的 BOM 保留。文件 SHALL 寫明蓋章時已不存在的檔不在 `scope` 裡。文件裡算指紋的範例程式對同一個檔 SHALL 得到與引擎蓋章時相同的值。動詞參考文件（中英兩版）SHALL 列出 `show` 的 `--detail` 與 `scope` 的形狀。

#### Scenario: 整合者照文件在伺服端匯入

- **WHEN** 整合者照 Node SDK 文件的「在伺服端匯入」，把只含規格資料夾的下載內容交給 `createEngine` 與 `exportScope`，再交給 `host.importScope`
- **THEN** 文件提供 `root` 與 `specDir` 的寫法、放錯位置時的錯誤訊息開頭、大小上限與重新匯入的步驟，不需要翻正式規格

#### Scenario: 釋放範例在讀取失敗時不放行

- **WHEN** 宿主以 Node SDK 文件的釋放範例當 `beforeCommand`，以不是認領的人、也不是管理者的身分執行 `claim --release`，而範例裡的 `read` 因儲存出錯而失敗（錯誤代碼不是 `not_found`）
- **THEN** 這次呼叫失敗，認領沒有被釋放，儲存沒有收到寫入

##### Example: 釋放範例對各種情況的結果

| 情況 | 結果 |
| --- | --- |
| 認領的人釋放 | 釋放成功 |
| 管理者釋放別人的認領 | 釋放成功 |
| 不是認領的人，也不是管理者 | 以 `refused` 拒絕，訊息是範例給的 `reason` |
| 變更不存在 | 以 `not_found` 拒絕，由釋放本身回報 |
| 範例裡的 `read` 因儲存出錯而失敗 | 呼叫失敗，不釋放，儲存沒有收到寫入 |

#### Scenario: 釋放的範例只有一份

- **WHEN** 在中英兩版的動詞參考文件與 Node SDK 文件搜尋 `claim-release` 的範例程式
- **THEN** 範例程式只出現在 Node SDK 文件的掛鉤段落，動詞參考文件只有命令名稱的說明與連結

#### Scenario: 文件的指紋範例與引擎一致

- **WHEN** 以文件的指紋範例程式，對一個開頭有 BOM 的 UTF-8 檔、一個行尾是 CRLF 的檔與一個不是 UTF-8 的檔算出指紋，再以引擎蓋章時用的指紋規則對同樣三份內容算出指紋
- **THEN** 兩邊的三個值逐一相同

#### Scenario: 參考文件列出 show 的 --detail

- **WHEN** 在中英兩版的動詞參考文件搜尋 `show` 的 `--detail`
- **THEN** 兩版都說明它只能和 `--json` 一起用、`scope` 只在已蓋章的關卡出現，並附輸出範例

#### Scenario: 整合者照文件重新匯入並保住流程政策

- **WHEN** 整合者照 Node SDK 文件「在伺服端匯入」的重新匯入範例，對一個管理者設了 `tdd` 為 true 的範圍執行：讀出流程政策、試跑、取代、把 `tdd` 寫回去；匯入包的流程設定沒有 `tdd`
- **THEN** 試跑的 `changed` 含 `config.yaml`，範圍在試跑後不變；取代之後範圍的文件等於匯入包；範例跑完之後 `workflow-config show --json` 的 `tdd` 是 true

#### Scenario: 文件不再要求宿主清空儲存

- **WHEN** 在中英兩版的 Node SDK 文件搜尋重新匯入的步驟
- **THEN** 步驟是取代模式與試跑；文件沒有要求宿主在自己的儲存裡刪除這個範圍的文件

#### Scenario: 文件寫明取代模式不經 HTTP 入口

- **WHEN** 讀者在中英兩版的 Node SDK 文件與動詞參考文件查匯入
- **THEN** 兩份文件都寫明 `POST /import` 只收空的範圍，取代模式只在宿主的程式呼叫 `host.importScope` 時提供
