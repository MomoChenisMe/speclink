## MODIFIED Requirements

### Requirement: 遷移入口雙路且不經並存合併

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

遷移流程 SHALL 可自兩處進入：chooser 本機路徑開到含 speclink/ 的專案時的次要動作，與並存衝突對話。SHALL NOT 提供「上傳合併至非空 scope」的任何路徑——CreateNew 為唯一遷移語意。

#### Scenario: chooser 次要動作進入遷移

- **WHEN** 於 chooser 本機路徑選到含 speclink/ 的專案並選擇遷移動作
- **THEN** 進入同一遷移流程（選 connection 與空 scope、確認、上傳、轉換）
