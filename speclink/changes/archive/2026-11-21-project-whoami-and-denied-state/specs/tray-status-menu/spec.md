## ADDED Requirements

### Requirement: 系統匣標示存取被拒

remote 專案分頁已有 session 而處於存取被拒（依 remote-resilience「存取被拒的狀態跟著分頁」）時，系統匣 SHALL 標示它，且 SHALL NOT 只用顏色表意。

面板：該專案的 tab SHALL 以方塊內的警示三角形圖示與其下的狀態文字「沒有存取權限」辨識，狀態文字 SHALL 為錯誤的紅系。作用中的專案為存取被拒時，面板 SHALL 保留最後成功的生命週期與討論內容，並在內容上方呈現一張錯誤紅系的狀態卡，內容為警示三角形圖示、「沒有存取權限」與既有的 stale 說明；這張卡 SHALL NOT 有重新登入鈕，面板 SHALL NOT 改為無 session 的復原卡。

原生選單：該專案的項目 SHALL 以「<專案名> — 沒有存取權限」為標籤（與離線時的寫法相同），項目 SHALL 維持可點選切換，SHALL NOT 改為復原 submenu。

狀態解除後，tab、狀態卡與選單項目 SHALL 回到 ready 的呈現。文字 SHALL 跟隨介面語言：zh-TW 為「沒有存取權限」，en 為「Access denied」。connection 同時為 offline 或 needs-reauth 時 SHALL 依 remote-resilience 的優先序呈現那一種狀態，不呈現存取被拒。

#### Scenario: 面板標示作用中專案存取被拒

- **WHEN** 作用中的 remote session 已載入內容後進入存取被拒，使用者開啟面板
- **THEN** 該專案的 tab 顯示警示三角形與紅色的「沒有存取權限」；面板上方有一張紅色狀態卡，沒有重新登入鈕；生命週期與討論區塊仍是最後成功的內容

#### Scenario: 原生選單標示存取被拒

- **WHEN** 系統匣樣式為原生選單，一個已開啟的 remote 專案 demo 進入存取被拒
- **THEN** 選單專案區該項目的標籤為「demo — 沒有存取權限」，點選它仍可切換到該專案

#### Scenario: 解除後回到一般呈現

- **WHEN** 存取被拒的專案在下一次輪詢成功
- **THEN** 面板的 tab 與選單項目回到 ready 的呈現，紅色狀態卡消失
