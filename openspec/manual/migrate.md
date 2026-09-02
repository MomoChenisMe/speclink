---
title: 本地工作區遷移到 remote
section: Remote 模式
order: 480
keywords: [遷移, 匯入, 備份, 空 scope, remote]
sources: [workspace-migration]
generated: 2026-09-02
---
# 本地工作區遷移到 remote

已經有本地工作區的專案，可以整個搬到 server 上的一個空 scope。遷移是一次性的整批動作：全部成功才轉換本地，任一步失敗本地分毫不動。

## 從哪裡進入

遷移在桌面 app 操作，有兩個入口：

- 新增工作區時選本機路徑，開到含 `openspec/` 的專案，選擇次要動作「遷移」。
- 本地 `openspec/` 與 remote 連接並存時跳出的衝突對話。

兩個入口進的是同一條流程：選 connection 與空 scope、確認、上傳、轉換。沒有「上傳合併到非空 scope」這種路徑。

## 上傳什麼

桌面 app 用引擎讀取本地工作區的全部內容打包上傳：

- 進行中變更的中介資料與全部產物；
- 正典規格；
- 進行中與已封存的討論；
- 已封存變更的全部文件；
- 工作流設定與共用詞彙文件。

任一來源檔解析失敗，打包中止並點名該檔。不會只搬一部分。

## server 端怎麼收

server 只接受「建立新的」這一種匯入語意：

- 目標 scope 已持有任何文件時，server 拒絕，並且不寫入任何內容。錯誤訊息帶 create-new 語意。
- 你的角色是 reader 時，server 拒絕（403）。
- 格式版本未知或缺欄位時，server 拒收。

成功時整包以單一原子提交落庫。不存在「寫了一半」的狀態。

## 成功後本地會怎樣

1. 本地的 `openspec/` 改名為帶日期的備份目錄。同名已存在時加序號。沒有任何路徑會刪除本地內容。
2. 桌面 app 寫入 remote 連接標記。
3. 原分頁原地轉為 remote 分頁，這個資料夾成為它的 checkout。
4. 之後 CLI 在這個資料夾以 remote 模式運作。

遷移前的確認對話會指出目標 Project／Repo，以及本地備份的行為。

> [!NOTE]
> 匯入失敗時（scope 非空、網路、驗證），本地 `openspec/` 原樣存在、沒有連接標記、分頁維持本地，錯誤原樣呈現在畫面上。

遷移完成後的工作方式見 [remote 模式總覽](remote-overview.md)。

**出處**：`workspace-migration`
