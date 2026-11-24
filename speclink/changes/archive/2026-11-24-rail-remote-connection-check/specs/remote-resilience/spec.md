## ADDED Requirements

### Requirement: 背景遠端分頁的連線定期檢查

Desktop SHALL 對圖示列上至少有一個遠端分頁的每一條 connection 做連線檢查，時機為 app 還原分頁列之後、主視窗取得焦點時，以及之後每 60 秒。同一條 connection 有多個遠端分頁時，每個時機 SHALL 只檢查一次；同一條 connection 的檢查 SHALL NOT 重疊；圖示列沒有遠端分頁時 SHALL NOT 送出任何檢查請求。

一次檢查 SHALL 只送不需要專案範疇的身分查詢（`/api/speclink/v1/whoami`；Server 回 404 或 405 時改問根層 `/auth/whoami`），SHALL NOT 讀取專案資料、SHALL NOT 建立 session、SHALL NOT 發出 handshake。檢查的結果 SHALL 經「離線狀態機單一真相且明確呈現」的同一套判定與事件廣播，計入失敗與歸零的規則相同；TS 層 SHALL NOT 由檢查請求的成敗自行推斷連線狀態；檢查命令 SHALL 回傳 Rust 判定後的目前狀態，前端以它補回沒有再轉換、因而沒有事件的狀態（例如主視窗重建之後）。檢查失敗、該失敗計入離線、而 connection 仍為 online 時，Desktop SHALL 在數秒內重試，直到成功、connection 轉為 offline 或送出次數達到離線閾值為止；connection 已是 offline 時一次檢查 SHALL 只送一個請求；connection 為 needs-reauth 時 SHALL NOT 送出請求。檢查失敗 SHALL NOT 彈出訊息。

遠端分頁 SHALL 以所屬 connection 由 Rust 判定的狀態呈現，不論該分頁有沒有 session、是不是作用中分頁：沒有 session 也沒有復原狀態（連線中、錯誤）的遠端分頁，connection 為 offline 或 needs-reauth 時，圖示列方塊角落 SHALL 為琥珀點，提示 SHALL 附狀態文字：offline 時為「離線」且 SHALL NOT 提到最後資料（這類分頁沒有資料），needs-reauth 時沿用既有的需要登入文字；系統匣面板的專案 tab 與原生選單的專案項 SHALL 呈現同一個狀態；分頁的復原狀態 SHALL 優先於 connection 狀態。connection 回到 online 時 SHALL 拿掉琥珀點，SHALL NOT 自動 handshake、SHALL NOT 建立 session；使用者點選該分頁時才開始連線。

#### Scenario: 沒點過的遠端分頁在 Server 停止時亮琥珀點

- **WHEN** 圖示列有本機專案 A，與同一台 Server 上這次啟動後還沒點過的遠端專案 B、C；Server 停止中，使用者啟動 app，作用中專案為 A
- **THEN** 約 3 秒內 B、C 的方塊角落亮琥珀點，提示為「離線」且沒有提到最後資料，A 沒有標示；Server 端在這段時間只收到一組（最多 3 個）身分查詢，而不是 B、C 各一組；沒有發出 handshake

#### Scenario: Server 恢復後琥珀點消失且不自動連線

- **WHEN** B、C 已呈現離線，之後 Server 恢復，使用者把主視窗切回前景
- **THEN** 一次檢查成功，B、C 的琥珀點消失，沒有對 B、C 發出 handshake；使用者點選 C 時才開始連線

#### Scenario: 點過但在背景的遠端分頁也跟著更新

- **WHEN** 使用者先點開遠端專案 B（已有 session），再切到本機專案 A，之後 Server 停止
- **THEN** 最遲一個檢查週期（60 秒）加上重試的數秒內，B 的方塊亮琥珀點，提示照舊為「離線，顯示最後資料」，使用者不必切回 B

#### Scenario: 單次失敗在重試成功後不亮燈

- **WHEN** 一次檢查的第一個請求連不上 Server，1 秒後的重試成功
- **THEN** connection 維持 online，沒有廣播 offline，方塊沒有琥珀點

#### Scenario: 登入失效時提示重新登入

- **WHEN** 遠端專案 B 所屬 connection 的登入已失效：身分查詢回 401，換發後仍為 401
- **THEN** connection 廣播 needs-reauth，B 的方塊亮琥珀點，提示附需要登入的狀態文字；之後的定期檢查不再送出請求，直到使用者重新登入

#### Scenario: Server 有回應但拒絕不算離線

- **WHEN** 檢查時 Server 對身分查詢回應 403
- **THEN** 這次檢查只送一個請求，connection 維持或回到 online，方塊沒有琥珀點

#### Scenario: 只有本機專案時不檢查

- **WHEN** 圖示列只有本機專案，使用者啟動 app、把主視窗切回前景，並等待超過 60 秒
- **THEN** Desktop 沒有對任何 Server 送出身分查詢

#### Scenario: 系統匣呈現同一個狀態

- **WHEN** 沒點過的遠端專案 B 所屬 connection 為 offline
- **THEN** 系統匣面板的專案 tab B 的狀態文字為「離線」，原生選單的 B 項附同一段文字，兩處都沒有提到最後資料，與圖示列一致

#### Scenario: 復原狀態優先於連線狀態

- **WHEN** 使用者點選遠端專案 C、握手失敗，C 進入錯誤復原頁；之後 connection 為 offline
- **THEN** C 的方塊維持錯誤態（紅點），不改為琥珀點
