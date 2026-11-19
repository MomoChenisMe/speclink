---
title: 備份與還原
section: Remote 模式
order: 510
keywords: [backup, verify-backup, restore, 匯出, 資料結構遷移, 災難演練, 備份格式版本, 舊版備份]
sources: [server-backup, "user-documentation#示範用 server 的升級步驟有說明"]
generated: 2026-11-19T16:40:41+08:00
---

# 備份與還原

server 提供三個子命令：backup、verify-backup、restore。它們在 server 主機上執行；Docker 部署時在容器內執行。管理面另外提供匯出與備份資訊。

## 備份：backup

- 產生單一個備份檔。
- 檔內有：manifest（備份格式版本、UTC 建立時間、引擎與 API 版本、儲存的種類、資料結構版本、scope 清單）、每個 scope 的匯出包、identity 資料庫的時點快照、每一個成員的 digest。
- 這個版本產生的備份，備份格式版本是 2。
- 匯出包由 server 逐 scope 匯出產生，不是直接複製儲存的資料庫檔或資料夾。
- 備份檔不含任何憑證明文。

> [!WARNING]
> 在沒有寫入的條件下備份：server 未運行，或部署層的維護窗口。寫入中的快照不保證一致。

## 驗證：verify-backup

- 只讀備份檔，不動任何資料。
- 檢查三件事：manifest 與每個成員的 digest 相符、匯出包的結構可解析、備份格式版本已知。
- 已知的備份格式版本有兩個：1（0.12.0 以前的版本產生，含 0.12.0；匯出包以文件種類標示文件）與 2（以鍵標示文件）。舊版的備份檔照樣通過檢查，報告的備份格式版本是 1。
- 全部通過：exit code 0。
- 任一成員 digest 不符、結構不可解析、格式版本未知：非零 exit code，並指出不符的成員。改動備份檔的任何一個位元都會被抓到。
- restore 的第一步就是這個驗證。

## 還原：restore

還原只能進空的目標。

- 目標的儲存後端與 identity 都必須是空的。任一邊非空：拒絕，並輸出既有內容的摘要。目標內容不變。
- 沒有覆蓋既有資料的旗標。

還原依序做四件事：

1. 驗證備份檔完整性。
2. 把 identity 快照放到位。
3. 逐 scope 匯入。每個 scope 一次提交，只進空的範圍。
4. 還原後驗證。逐 scope 重讀，比對匯出包的 digest 與文件數。比對 identity 的使用者、專案與儲存庫、稽核紀錄的計數，以及資料結構版本是否與 manifest 一致。

結果：

- 還原輸出逐項報告。
- 任一項不符：非零 exit code，明示差異項，並明示目標不可投產。
- manifest 宣告的格式版本未知：非零 exit code，原因指出版本不相容，目標不寫入。
- 備份格式版本 1 的備份檔可以還原到這個版本。還原會先把每份文件的種類換成鍵，而且在放 identity 快照之前就對全部 scope 換完。有任何一份文件換不成鍵：非零 exit code，訊息指出是哪個 scope 的哪份文件，identity 與儲存都不寫入。
- 還原成功後啟動 server：成員用備份前的存取金鑰照常通行，CLI 查詢的輸出與備份前一致，稽核歷史完整在列。舊版的備份也一樣：CLI 的 list、show 與 plan 輸出的變更、規格與順序與備份前一致。

從 0.12.0 以前的版本升級 server，做法是用舊版備份、再用新版還原，步驟見 [啟動 server](server-start.md)。反過來不行：舊版讀不了備份格式版本 2 的備份檔。

## 管理面的資料操作

管理面的系統頁提供兩個操作。兩者都走 admin 門禁，各記一筆稽核紀錄。

- 匯出 scope：即時產生該 scope 的匯出包供下載。匯出包可解析，digest 與儲存後端內容一致。稽核動作記為 scope-exported。未知的 scope 回 404。
- 備份資訊：顯示最近一次備份與驗證的 manifest 摘要與結果。稽核動作記為 backup-recorded。

管理面不提供資料結構遷移。server 不再產生 store-migrated 的稽核紀錄；以前留下的 store-migrated 紀錄照常列在稽核頁。

門禁與稽核的規則見 [管理面與稽核](server-admin.md)。系統頁的畫面見 [瀏覽器後台](web-console.md)。

**出處**：`server-backup`、`user-documentation`
