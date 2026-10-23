# 實作期間的安全邊界檢查

以三個精簡觀點核對新入口：惡意輸入者（跨提案／路徑／秘密）、省略步驟的開發者（缺知識庫／缺新鮮度／一般 accept）、混淆設定的開發者（錯型別／舊 server／參考與 required）。未啟動額外 agent。

已重現並修正：

- 空目標家族原可空集合 PASS；現在須有實際理由、其他授權目標證據及 NOT_APPLICABLE，沒有任何目標仍不完整。
- 深度報告的標準狀態原一直停在 awaiting_deep_review；現在從精確控制項 check 推導，缺項不當成功。
- 原始碼深度清單原漏讀 .env/.env.*、大寫副檔名與常見設定文件；擴充深度輸入，原基本掃描支援不變。
- 基本報告外來 prose 原會保留 password 賦值；寫入／回應前統一遮罩。
- checks 文件的普通讀取原跟隨檔案或父目錄連結；現在限合法檢查路徑、64MiB，使用既有 no-follow I/O。active 與 archived 均驗證不讀出外部 sentinel。
- 標準引用原可讓 A01 指到 A02 的文件；現在 itemId 文件只允許同一項，整體 XML/CSV/JSON 表仍可精確查表。
- 規格／程式一併審查時，基本 coverage 的 rules_recorded 未驗證即使已逐項深度複核仍會阻擋；現在要求實際 coverage 複核 ID、目標證據及理由後才採用，不是清除原始發現。
- required 封存的 --mark-tasks-complete 原可在驗收後修改已審 tasks.md；現在在任何寫入前回 security_report_stale，須先完成任務再重審。
- 封存回復原可能與另一筆進行中的封存相撞；現在同提案的回復、發佈、清理共用固定檔案鎖，busy 零效果。

保留既有 CAS、來源／decision 綁定、嚴格設定型別、舊 server unsupported_version、只讀選定本機知識庫與接受理由遮罩測試。沒有新增通用交易或掃描抽象。

驗證：Core lib 1073 passed / 1 ignored；FS 45 unit + 1 integration + 22 store passed / 1 ignored；Host 160 unit + 5 bridge + 1 paired Remote integration passed；Protocol 76 passed；Desktop TypeScript noEmit 通過。新增缺陷均先觀察失敗；修正後重新執行受影響 package。
