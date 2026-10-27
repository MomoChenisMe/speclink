## MODIFIED Requirements

### Requirement: 中途轉出教學
<!-- BEFORE: 分期轉出教學規定最後一刀封存後教執行 speclink discuss archive <slug> 收尾，hold 只由不帶 --hold 的 conclude 或 discuss archive 解除 -->

技能檔 SHALL 規定：多需求討論中單項談定、使用者要先立案時，代理人 SHALL 教執行 speclink discuss promote 即刻轉出（引擎於無結論時以 topic 預填提案），討論 SHALL 繼續加輪談剩餘項，SHALL NOT 要求先寫結論。最終 conclude SHALL 照常執行——引擎保留 promoted 狀態、寫入結論、並將已轉出變更標為待重新反映；技能檔 SHALL 註明該標記與最終結論無關時僅需一次確認。技能檔 SHALL 另規定分期轉出：結論規劃「之後回同一份記錄再轉出一個或多個變更」（例：多刀依序立案）時，conclude SHALL 帶 --hold 一次；技能檔 SHALL 說明 hold 不因不帶 --last 的轉出而清除，只由不帶 --hold 的 conclude、speclink discuss archive，或帶 --last 的轉出解除；最後一刀 SHALL 由 /speclink-propose 於 speclink new change --from-discussion 帶 --last 轉出（判定規則在 propose 技能），其後最後一個轉出變更封存時記錄自動隨行封存，SHALL NOT 教執行 speclink discuss archive 作為常態收尾；技能檔 SHALL 註明忘了帶 --last 時記錄留在途、看板標「已轉出・保留中」，執行 speclink discuss archive <slug> 一次即可收尾；未帶 --hold 的記錄會在最後一個轉出變更封存時隨行封存，之後的刀 SHALL 走新討論。技能檔 SHALL NOT 含「旗標由下一次轉出清除」、「discard 後要重跑 conclude --hold」或「最後一刀封存後手動 discuss archive 收尾」作為常態步驟的規定。技能檔 SHALL 另註明救援路徑：討論被誤封存時（含在不是最後一刀的變更上帶了 --last），把記錄檔自 openspec/discussions/archive/ 搬回 openspec/discussions/ 即可續用（引擎對已封存討論的轉出錯誤訊息即指向此路）。conclude 指令範例 SHALL 標示 --hold 的用途。

#### Scenario: 單項談定即中途轉出

- **WHEN** 討論尚未結論，使用者要求先把已談定的需求轉為變更
- **THEN** 技能檔 SHALL 規定直接執行 promote 並繼續討論剩餘項，SHALL NOT 要求先 conclude 整份討論

#### Scenario: 中途轉出後補結論

- **WHEN** 中途轉出過的討論最終執行 conclude
- **THEN** 技能檔 SHALL 說明：狀態保持已轉出、結論照常寫入、先轉出的變更被標為待重新反映，與結論無關時一次確認即可

#### Scenario: 分期轉出帶 --hold

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的中途轉出段與 conclude 指令範例
- **THEN** 內容 SHALL 規定結論規劃之後回同一記錄再轉出時 conclude 帶 --hold 一次、hold 只由不帶 --hold 的 conclude、discuss archive 或帶 --last 的轉出解除、最後一刀由 propose 帶 --last 轉出且最後一個封存自動收尾、忘帶 --last 時執行 discuss archive 一次收尾、未帶 --hold 時後續刀走新討論；SHALL NOT 含「旗標由下一次轉出清除」、「discard 後重跑 conclude --hold」與「最後一刀封存後手動 discuss archive 收尾」的常態敘述；SHALL 含誤封存的搬回救援路徑；conclude 範例 SHALL 標示 --hold 的用途；claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映
