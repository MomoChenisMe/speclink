## MODIFIED Requirements

### Requirement: UI 介面語言支援 zh-TW 與 en

app 的 UI 介面語言 SHALL 支援 zh-TW 與 en:未設定偏好時 SHALL 跟隨系統語言(系統語言以 zh 開頭判為 zh-TW,其餘判為 en);設定頁 SHALL 提供「跟隨系統／zh-TW／en」三選,切換 SHALL 即時對全介面生效並持久化於 app 本機;macOS 的原生選單列 SHALL 同樣跟隨 UI 語言,切換時 SHALL 立即重建。UI 語言偏好與 config.yaml 的 locale(AI artifacts 產出語言)SHALL 互不影響。兩語言字典的 key 集合 SHALL 相等;查無 key 時 SHALL 顯示 key 本身而非另一語言的字串。

#### Scenario: 未設定偏好時跟隨系統語言

- **WHEN** app 於 UI 語言偏好未設定的狀態下啟動
- **THEN** 系統語言以 zh 開頭時全介面(含 macOS 選單列)呈現 zh-TW,否則呈現 en

##### Example: 系統語言判定

| 系統語言 | UI 語言 |
| -------- | ------- |
| zh-TW | zh-TW |
| zh-CN | zh-TW |
| en-US | en |
| ja-JP | en |

#### Scenario: 手動切換即時生效並持久化

- **WHEN** 使用者於設定頁將 UI 語言由 zh-TW 切為 en
- **THEN** 全介面(標題列、圖示列、專案欄、看板、對話框、macOS 選單列)即時改為英文,重啟 app 後仍為英文,且 config.yaml 內容未被此操作改動

#### Scenario: UI 語言與 artifacts 產出語言互不影響

- **WHEN** config.yaml 設定 locale: tw,使用者將 UI 語言切為 en
- **THEN** UI 呈現英文,而 config.yaml 的 locale 仍為 tw(引擎產出 artifacts 的語言政策不受 UI 語言影響)
