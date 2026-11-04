---
title: 用 speclink 網址開啟畫面
section: 桌面 app
order: 370
keywords: [speclink 網址, deep link, 開啟詳情頁, 路線圖, 切專案, 帶到前景, 第二個 app, 格式錯誤, 找不到]
sources: ["desktop-app#以 speclink 網址開啟詳情頁"]
generated: 2026-11-04T17:42:55+08:00
---

# 用 speclink 網址開啟畫面

桌面 app 註冊了 `speclink` 網址格式。外部程式送出這種網址，app 就會到前景，打開指定專案的變更詳情頁、討論詳情頁或路線圖，或單純切到那個專案的看板。

網址只能開啟與檢視。處理過程不會觸發任何寫入動作，例如勾任務、封存、轉出或刪除。

## 四種網址

| 網址 | 開啟 |
| --- | --- |
| `speclink://change/<變更名稱>?project=<專案資料夾絕對路徑>` | 變更詳情頁 |
| `speclink://discussion/<討論 slug>?project=<專案資料夾絕對路徑>` | 討論詳情頁 |
| `speclink://roadmap?project=<專案資料夾絕對路徑>` | 該專案的[路線圖](desktop-roadmap.md) |
| `speclink://project?project=<專案資料夾絕對路徑>` | 切到該專案並顯示看板 |

規則：

- 名稱與路徑以百分比編碼傳遞（例如 `/` 寫成 `%2F`），app 處理前會解碼。
- 種類只有 `change`、`discussion`、`roadmap`、`project` 四個。
- `change` 與 `discussion` 後面恰好一段非空的名稱。`roadmap` 與 `project` 後面不帶名稱，可以多一個 `/`。
- `?project=` 的值必須是絕對路徑：POSIX 以 `/` 開頭，Windows 以磁碟代號加 `:\` 或 `:/` 開頭。

不符合的都算格式錯誤：

| 網址 | 結果 |
| --- | --- |
| `speclink://change/add-search?project=%2FUsers%2Fdev%2Fspeclink` | 變更 add-search，專案 /Users/dev/speclink |
| `speclink://discussion/board-search-bar?project=C%3A%5Cwork%5Capp` | 討論 board-search-bar，專案 C:\work\app |
| `speclink://change/add-search?project=C%3A%2Fwork%2Fapp` | 變更 add-search，專案 C:/work/app |
| `speclink://roadmap?project=%2FUsers%2Fdev%2Fspeclink` | 路線圖，專案 /Users/dev/speclink |
| `speclink://roadmap/?project=%2Fa` | 路線圖，專案 /a |
| `speclink://project?project=%2FUsers%2Fdev%2Fspeclink` | 切到專案 /Users/dev/speclink |
| `speclink://project/?project=%2Fa` | 切到專案 /a |
| `speclink://roadmap/waves?project=%2Fa` | 格式錯誤（路線圖不帶名稱） |
| `speclink://project/board?project=%2Fa` | 格式錯誤（專案網址不帶名稱） |
| `speclink://project` | 格式錯誤（缺 project） |
| `speclink://spec/desktop-app?project=%2Fa` | 格式錯誤（未知種類） |
| `speclink://change/a/b?project=%2Fa` | 格式錯誤（名稱多於一段） |
| `speclink://change/?project=%2Fa` | 格式錯誤（名稱為空） |
| `speclink://change/add-search` | 格式錯誤（缺 project） |
| `speclink://change/add-search?project=work%2Fapp` | 格式錯誤（project 不是絕對路徑） |
| `https://change/add-search?project=%2Fa` | 格式錯誤（不是 speclink 網址） |

## 開啟時發生什麼事

1. 主視窗顯示並取得焦點。視窗最小化時也會還原。
2. app 用和「開啟專案」相同的探測流程開啟 `?project=` 指定的資料夾。已經是分頁的專案成為活躍分頁，沒開過的加成新分頁。
3. 探測結果進入專案、資料載入完成後，app 開啟目標：
   - 變更與討論：開啟目標的詳情頁。互斥與宿主規則和系統匣的「開啟此變更」「開啟此討論」相同，見[認識桌面 app](desktop-overview.md)。
   - 路線圖：關閉開著的詳情頁，主區切到該專案看板頁的路線圖檢視。路線圖沒有找不到目標的情形。
   - 專案：關閉開著的詳情頁與可取消的浮層，主區切到該專案的看板（卡片檢視），上次選的是路線圖時也一樣。不會開詳情頁或路線圖，也沒有找不到目標的情形。

| 情況 | 結果 |
| --- | --- |
| app 執行中，專案已是分頁，送出變更的網址 | 主視窗到前景，活躍分頁是該專案，主區是看板且開著該變更的詳情頁 |
| app 執行中，送出討論的網址 | 主視窗到前景，主區是看板且開著該討論的詳情頁 |
| 分頁列沒有該專案；資料夾是已啟用的專案且含該變更 | 分頁列多出該專案並成為活躍分頁，資料載入完成後開著該變更的詳情頁 |
| 主區開著某變更的詳情頁，送出路線圖的網址 | 主視窗到前景，詳情頁關閉，主區是路線圖，沒有 toast |
| 活躍分頁是別的專案、主區開著某變更的詳情頁、看板頁上次選的是路線圖，送出專案的網址 | 主視窗到前景，活躍分頁是該專案，詳情頁關閉，主區是看板（卡片檢視），沒開詳情頁或路線圖，沒有 toast |
| 分頁列沒有該專案，資料夾是已啟用的專案，送出專案的網址 | 分頁列多出該專案並成為活躍分頁，資料載入完成後主區是看板（卡片檢視） |

## 打不開時

| 情況 | 結果 |
| --- | --- |
| 網址格式錯誤 | 主視窗顯示並取得焦點，toast 說明網址無法開啟；活躍分頁維持原本的專案 |
| 變更或討論不在該專案已載入的清單內 | 停在已切換的專案，主區是看板且沒開詳情頁；toast 說明在該專案找不到該變更或討論 |
| 資料夾未初始化、未啟用或帶 remote 標記 | 沿用「開啟專案」的既有對話框或選擇器，不開詳情頁；路線圖與專案的網址也不切換主區，例如主區是規格頁時仍停在規格頁；你確認前不寫入任何檔案 |
| 不可取消的浮層開著，送出指向已開專案某變更的網址 | 主視窗到前景，浮層與目前畫面不變，不開詳情頁 |
| 不可取消的浮層開著，送出專案的網址 | 主視窗到前景，浮層、活躍分頁與目前畫面都不變，沒有 toast |

## app 沒開或已開

- **app 沒在執行**：網址會啟動 app，等分頁列還原完才處理。還原前收到的網址依到達順序排隊，逐一處理。
- **app 在執行**：由執行中的 app 處理，不會出現第二個 app，Dock 或工作列也不會多一個圖示。連續送出兩個合法網址時，兩個都依序處理，最後開著第二個網址的詳情頁。
- **Windows 與 Linux 再次啟動**：不帶網址再次啟動 app（例如從開始選單或啟動器）時，既有主視窗會還原、到前景並取得焦點，不會開第二個 app。

**出處**：`desktop-app`
