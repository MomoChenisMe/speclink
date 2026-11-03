## MODIFIED Requirements

### Requirement: 以 speclink 網址開啟詳情頁

<!-- BEFORE: host 只接受 change 與 discussion，沒有開路線圖的網址 -->

桌面 app SHALL 註冊 `speclink` 網址格式，接受三種網址：`speclink://change/<變更名稱>?project=<專案資料夾絕對路徑>` 開啟變更詳情頁，`speclink://discussion/<討論 slug>?project=<專案資料夾絕對路徑>` 開啟討論詳情頁，`speclink://roadmap?project=<專案資料夾絕對路徑>` 開啟該專案的路線圖（依 desktop-roadmap「看板頁的路線圖檢視」）；名稱與路徑 SHALL 以百分比編碼傳遞並在處理前解碼。網址的 host SHALL 只接受 `change`、`discussion` 與 `roadmap`；`change` 與 `discussion` 的 path SHALL 恰為一段非空名稱，`roadmap` 的 path SHALL 為空（可帶單一 `/`），`project` SHALL 為絕對路徑（POSIX 以 `/` 開頭，Windows 以磁碟代號加 `:\` 或 `:/` 開頭）；不符者 SHALL 視為格式錯誤。

處理一個網址時，桌面 app SHALL 依序：顯示主視窗並取得焦點（自最小化亦還原）；以與「開啟專案」相同的探測流程開啟 `project`——已是分頁的專案成為活躍分頁，未開過的加成分頁；探測結果進入專案且該專案的資料載入完成後，`change` 與 `discussion` 開啟目標的詳情頁，詳情頁的互斥與宿主規則與系統匣的「開啟此變更」「開啟此討論」相同（依「詳情頁互斥」「變更與討論詳情頁的宿主為看板」）；`roadmap` SHALL 關閉開著的詳情頁並把主區切到該專案看板頁的路線圖檢視，沒有找不到目標的情形。探測結果為未初始化、未啟用或帶 remote 標記的資料夾時，SHALL 沿用「開啟專案」的既有對話框或選擇器，SHALL NOT 開啟詳情頁，確認前 SHALL NOT 寫入任何檔案。

網址 SHALL 只能開啟與檢視：處理流程 SHALL NOT 觸發任何寫入動詞（勾任務、封存、轉出、刪除等）。格式錯誤時 SHALL 顯示主視窗並以 toast 說明網址無法開啟，SHALL NOT 切換專案；目標不在該專案已載入的清單內時 SHALL 停在已切換的專案並以 toast 說明找不到該變更或討論。

app 未執行時，網址 SHALL 啟動 app，並在分頁列還原完成後才處理；還原前收到的網址 SHALL 依到達順序排隊，逐一處理。app 已執行時，網址 SHALL 由執行中的 app 處理，SHALL NOT 出現第二個 app。於 Windows 與 Linux，不帶網址再次啟動 app 時 SHALL 把既有主視窗帶到前景並取得焦點，SHALL NOT 開啟第二個 app。

#### Scenario: 以網址開啟已開專案的變更詳情

- **WHEN** app 執行中且專案 /Users/dev/speclink 已是分頁，外部程式送出 `speclink://change/desktop-deep-link?project=%2FUsers%2Fdev%2Fspeclink`
- **THEN** 主視窗顯示於前景並取得焦點，活躍分頁為 speclink，主區為看板且開著變更 desktop-deep-link 的詳情頁

#### Scenario: 以網址開啟討論詳情

- **WHEN** app 執行中，外部程式送出 `speclink://discussion/herdr-speclink-panel?project=%2FUsers%2Fdev%2Fspeclink`
- **THEN** 主視窗顯示於前景並取得焦點，主區為看板且開著討論 herdr-speclink-panel 的詳情頁

#### Scenario: 專案尚未開啟時加成分頁再開詳情

- **WHEN** 分頁列沒有 /Users/dev/other，外部程式送出 `speclink://change/add-search?project=%2FUsers%2Fdev%2Fother`，該資料夾是已啟用的專案且含變更 add-search
- **THEN** 分頁列多出 other 並成為活躍分頁，資料載入完成後開著變更 add-search 的詳情頁

#### Scenario: app 未執行時由網址啟動

- **WHEN** app 未執行，外部程式送出 `speclink://change/desktop-deep-link?project=%2FUsers%2Fdev%2Fspeclink`
- **THEN** app 啟動並還原上次的分頁列，之後切到 speclink 並開著變更 desktop-deep-link 的詳情頁

#### Scenario: 執行中再收到網址不另開 app

- **WHEN** app 執行中，外部程式先後送出兩個合法網址
- **THEN** 兩個網址都由同一個 app 依序處理，最後開著第二個網址的詳情頁，畫面上沒有第二個 app 視窗、Dock 或工作列沒有第二個圖示

#### Scenario: 格式錯誤的網址

- **WHEN** 外部程式送出 `speclink://spec/desktop-app?project=%2FUsers%2Fdev%2Fspeclink`
- **THEN** 主視窗顯示並取得焦點，toast 說明網址無法開啟，活躍分頁維持原本的專案

##### Example: 網址解析

| 網址 | 結果 |
| ---- | ---- |
| `speclink://change/add-search?project=%2FUsers%2Fdev%2Fspeclink` | 變更 add-search，專案 /Users/dev/speclink |
| `speclink://discussion/board-search-bar?project=C%3A%5Cwork%5Capp` | 討論 board-search-bar，專案 C:\work\app |
| `speclink://change/add-search?project=C%3A%2Fwork%2Fapp` | 變更 add-search，專案 C:/work/app |
| `speclink://roadmap?project=%2FUsers%2Fdev%2Fspeclink` | 路線圖，專案 /Users/dev/speclink |
| `speclink://roadmap/?project=%2Fa` | 路線圖，專案 /a |
| `speclink://roadmap/waves?project=%2Fa` | 格式錯誤（路線圖不帶名稱） |
| `speclink://spec/desktop-app?project=%2Fa` | 格式錯誤（未知種類） |
| `speclink://change/a/b?project=%2Fa` | 格式錯誤（名稱多於一段） |
| `speclink://change/?project=%2Fa` | 格式錯誤（名稱為空） |
| `speclink://change/add-search` | 格式錯誤（缺 project） |
| `speclink://change/add-search?project=work%2Fapp` | 格式錯誤（project 不是絕對路徑） |
| `https://change/add-search?project=%2Fa` | 格式錯誤（不是 speclink 網址） |

#### Scenario: 以網址開啟路線圖

- **WHEN** app 執行中、主區開著某變更的詳情頁，外部程式送出 `speclink://roadmap?project=%2FUsers%2Fdev%2Fspeclink`
- **THEN** 主視窗顯示於前景並取得焦點，活躍分頁為 speclink，詳情頁已關閉，主區為看板頁的路線圖檢視，沒有出現 toast

#### Scenario: 找不到目標變更

- **WHEN** 外部程式送出 `speclink://change/no-such-change?project=%2FUsers%2Fdev%2Fspeclink`，speclink 專案沒有這個進行中的變更
- **THEN** 活躍分頁切到 speclink、主區為看板且未開任何詳情頁，toast 說明在 speclink 找不到變更 no-such-change

#### Scenario: 未初始化的資料夾沿用既有確認流程

- **WHEN** 外部程式送出 `speclink://change/x?project=%2FUsers%2Fdev%2Fplain`，該資料夾尚未初始化 speclink
- **THEN** 主視窗出現「開啟專案」流程的既有初始化確認框，未開任何詳情頁，使用者確認前該資料夾沒有新增或修改任何檔案

#### Scenario: 不可取消的浮層開著時不開詳情

- **WHEN** 主視窗開著不可取消的浮層，外部程式送出指向已開專案某變更的合法網址
- **THEN** 主視窗顯示於前景，浮層與目前畫面維持不變，未開啟該變更的詳情頁

#### Scenario: Windows 與 Linux 再次啟動時帶到前景

- **WHEN** 於 Windows 或 Linux，app 執行中且主視窗已最小化，使用者再次從開始選單或啟動器啟動 Speclink
- **THEN** 既有主視窗還原、顯示於前景並取得焦點，沒有出現第二個 app
