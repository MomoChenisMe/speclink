---
title: 安裝 CLI 與桌面 app
section: 開始使用
order: 20
keywords: [安裝, CLI, 桌面 app, Homebrew, 安裝腳本, 未簽章]
sources: [cli-distribution, user-documentation]
generated: 2026-09-02
---

# 安裝 CLI 與桌面 app

Speclink 有兩個可以安裝的東西：命令列工具 `speclink`（下稱 CLI），以及桌面 app。兩者可以只裝一個，也可以都裝。這一頁說明每條安裝通路會做什麼、裝完怎麼確認，以及兩者同時安裝時要注意的事。

## 安裝通路一覽

| 要裝什麼 | 通路 | 平台 |
| --- | --- | --- |
| CLI | 安裝腳本（一行指令） | macOS、Linux（sh 腳本）；Windows（PowerShell 腳本） |
| CLI | Homebrew tap | macOS、Linux（arm64 與 x86_64） |
| 桌面 app | 安裝檔下載 | macOS（dmg）、Windows（安裝器）、Linux（AppImage 與 deb） |
| 從原始碼建置 | 開發者導向 | 全平台 |

> [!NOTE]
> 規格只規定通路與行為，沒有載明安裝指令本文與下載網址。實際的一行指令與下載入口，請看 README 的安裝區塊。

## 用安裝腳本裝 CLI

安裝腳本會依序做這些事：

1. 偵測你的作業系統與 CPU 架構，對應到正確的發行版本。
2. 向 GitHub Releases 查最新版本。你可以用環境變數釘住特定版本。
3. 下載 CLI 壓縮檔與 checksum 清單（SHA256SUMS.txt），核對 checksum。
4. 把 `speclink` 可執行檔解壓到安裝目錄。Unix 預設是 `~/.local/bin`，Windows 預設是使用者層級的程式目錄。環境變數可以改安裝目錄。
5. 檢查安裝目錄是否在 PATH 裡。不在的話，腳本會提示你。

checksum 對不上時，腳本以錯誤結束。錯誤訊息指出 checksum 不符，而且不會留下任何已下載的檔案。

腳本另有 dry-run 模式：只印出解析到的平台、資產網址與安裝目錄，不連網、不寫檔。想先看看會裝到哪裡，可以用這個模式。

## 用 Homebrew 裝 CLI

Homebrew formula 涵蓋 macOS 與 Linux 各自的 arm64 與 x86_64。每次發版後，管線會自動更新 tap 裡的 formula。

## 確認 CLI 裝好了

執行：

```
speclink --version
```

輸出會顯示你安裝的版本號。

## 裝桌面 app

桌面 app 的安裝檔分三個平台：

- macOS：dmg
- Windows：安裝器
- Linux：AppImage 與 deb

### 未簽章安裝檔的放行步驟

安裝檔沒有作業系統的程式碼簽章時，系統會擋下它。放行方法：

- macOS：開啟「系統設定 > 隱私權與安全性」，對該 app 選「強制打開」。
- Windows：SmartScreen 出現時，點「其他資訊」，再點「仍要執行」。

## 桌面 app 與 CLI 同時安裝時的覆蓋行為

桌面 app 與 CLI 共用同一個佈署位置。兩者都裝的時候，桌面 app 會動到這個位置：

| 平台 | 行為 |
| --- | --- |
| macOS | 桌面 app 每次啟動，會把該位置換成指向內建 CLI 的 symlink，並刪除原有檔案 |
| Linux AppImage | 只在版本不符時覆蓋 |
| Windows、Linux deb | 由安裝器與套件管理器管理，不動該位置 |

> [!WARNING]
> 在 macOS 上先用腳本或 Homebrew 裝了 CLI，再裝桌面 app，桌面 app 啟動後你的 CLI 會被換成 app 內建的版本。你釘住的版本也會一併失效。

要保留自己安裝的 CLI，把它裝到另一個目錄，並把該目錄放在 PATH 順序的前面。

## Node SDK

`@speclink/engine` 以 `npm install @speclink/engine` 為主要安裝路徑。實際可安裝的時點，以第一個帶 engine 的 release 為準；在那之前只能從 repo 建置。

下一步：[建立工作區與指令檔](init-workspace.md)。

**出處**：`cli-distribution`、`user-documentation`
