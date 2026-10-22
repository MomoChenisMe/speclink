# 封存保留驗證

- 三類開關的八種組合，完成紀錄按設定保留；carry 的 review/verify 工單不受影響。先觀察未實作裁切時失敗，再通過。
- FsStore 先複製、篩選、驗證並同步封存副本，再移走活動提案。注入 staging、publish、rollback 故障驗證活動原文仍在；未提交副本不出現在封存讀取與清單。journal 支援再次操作回復，連結不跟隨。
- BridgeStore 與既有封存命令共用同次 UOW；index revision 衝突不會先刪活動紀錄。
- required 的最小 checks/acceptance.json 保留兩份報告 digest、政策／知識庫／來源指紋、裁決時間與風險接受理由；不含程式片段。完整安全報告全略過時仍可透過安全歷史與 Desktop 文件樹唯讀檢視摘要。摘要不作為新一輪驗收證據。
- CLI 與 Desktop 成功訊息顯示三類保留／略過數及摘要是否保留。

驗證：Core lib 1071 passed / 1 ignored；FS 44 unit + 1 integration + 21 store tests passed / 1 ignored；Host 160 unit + 5 bridge + 1 security integration passed；Desktop checkRecords/store 189 passed。Host 本機測試埠須於沙箱外執行。CLI/Server/Desktop-core cargo check 通過；git diff --check 通過。
