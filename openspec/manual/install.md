---
title: 安裝 CLI 與桌面 app
section: 開始使用
order: 20
keywords: [安裝, CLI, 桌面 app, npm, Homebrew, 安裝腳本, 未簽章, universal, AppImage]
sources: [cli-distribution, user-documentation]
generated: 2026-09-17T16:05:41+08:00
---

# 安裝 CLI 與桌面 app

Speclink 有兩個可以安裝的東西：命令列工具 `speclink`（下稱 CLI），以及桌面 app。兩者可以只裝一個，也可以都裝。這一頁說明每條安裝通路會做什麼、裝完怎麼確認，以及兩者同時安裝時要注意的事。

## 安裝通路一覽

| 要裝什麼 | 通路 | 平台 |
| --- | --- | --- |
| CLI | npm 全域安裝（一行指令） | macOS、Linux、Windows |
| CLI | 安裝腳本（curl 一行指令） | macOS、Linux |
| CLI | Homebrew tap | macOS、Linux（arm64 與 x86_64） |
| 桌面 app | 安裝檔下載 | macOS（單一 universal dmg，兩種晶片同一檔）、Windows（安裝器）、Linux（AppImage，依架構各一） |
| 從原始碼建置 | 開發者導向 | 全平台 |

Windows 沒有安裝腳本：走 npm 全域安裝，或裝桌面 app 由安裝器處理。沒有圖形介面的 Linux 主機走 CLI 的一行安裝。Linux 不再提供 deb 套件；已經用 deb 裝過的人，移除 deb 套件後改裝 AppImage。

> [!NOTE]
> 規格只規定通路與行為，沒有載明每條指令的本文與下載網址。實際的一行指令與下載入口，請看 README 的安裝區塊。

## 用 npm 裝 CLI

CLI 以 npm 套件 `@speclink/cli` 發布，指令名是 `speclink`。主套件底下有五個平台子套件（macOS arm64、macOS x64、Linux x64、Linux arm64、Windows x64），各只含該平台的執行檔；安裝時 npm 只抓你這個平台的那一個。

在 macOS 與 Linux 上，安裝後全域的 `speclink` 直接就是原生執行檔。在 Windows 上、用 `--ignore-scripts` 安裝、或用 Yarn 安裝時，全域的 `speclink` 是一個 JS 的轉接檔：它找出你這個平台的子套件並執行裡面的執行檔，結束碼與訊號原樣帶回，用起來沒有差別。你的作業系統與 CPU 組合沒有對應子套件時，執行會在 stderr 說明不支援並列出支援的組合，以錯誤結束。

## 用安裝腳本裝 CLI

安裝腳本只支援 macOS 與 Linux（POSIX sh）。在 Windows 的 shell 環境執行時，腳本以錯誤結束，並指向 npm 與桌面安裝器。腳本會依序做這些事：

1. 偵測你的作業系統與 CPU 架構，對應到 npm 的平台子套件（macOS arm64、macOS x64、Linux x64、Linux arm64 四種）。
2. 向 npm registry 查 `@speclink/cli` 的最新版本。要釘住特定版本，設環境變數 `SPECLINK_INSTALL_VERSION`；帶不帶 v 前綴都接受，`v0.5.0` 與 `0.5.0` 結果相同。
3. 下載對應平台子套件該版的 tgz。
4. 取 registry 回報的該版校驗值（sha512 integrity），與本機算出來的比對。
5. 通過後，把 tgz 裡的 `speclink` 執行檔解到安裝目錄。預設是 `~/.local/bin`，環境變數 `SPECLINK_INSTALL_DIR` 可以改。
6. 檢查安裝目錄是否在 PATH 裡。不在的話，腳本會提示你。

registry 位址預設是 `https://registry.npmjs.org`，環境變數 `SPECLINK_INSTALL_REGISTRY` 可以覆寫。校驗值對不上、或本機找不到算校驗值的工具時，腳本以錯誤結束，錯誤訊息指出 integrity 不符，而且不會留下任何已下載的檔案。

腳本另有 dry-run 模式：只印出解析到的平台、版本、tgz 網址與安裝目錄，不連網、不寫檔。想先看看會裝到哪裡，可以用這個模式。

| 你的機器 | 對應的平台子套件 |
| --- | --- |
| macOS、Apple 晶片 | `@speclink/cli-darwin-arm64` |
| macOS、Intel | `@speclink/cli-darwin-x64` |
| Linux、x86_64 | `@speclink/cli-linux-x64` |
| Linux、aarch64 | `@speclink/cli-linux-arm64` |

## 用 Homebrew 裝 CLI

Homebrew formula 涵蓋 macOS 與 Linux 各自的 arm64 與 x86_64，四組資產都指向 npm registry 上該版平台子套件的 tgz，不含 Windows。每次發版、CLI 的 npm 套件發布完成後，管線會自動更新 tap 裡的 formula；那一版沒有發到 npm 時，formula 就不更新。

## 確認 CLI 裝好了

執行：

```
speclink --version
```

輸出會顯示你安裝的版本號。用 npm 裝在 macOS 或 Linux 上時，PATH 上解析到的 `speclink` 是原生執行檔，不是 JS 檔。

## 裝桌面 app

桌面 app 的安裝檔分三個平台：

- macOS：單一 universal dmg，Apple 晶片與 Intel 都用同一個檔。
- Windows：安裝器。
- Linux：AppImage，依 CPU 架構各一個。

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
| Windows | 由安裝器管理，不動該位置 |

> [!WARNING]
> 在 macOS 上先用 npm、腳本或 Homebrew 裝了 CLI，再裝桌面 app，桌面 app 啟動後你的 CLI 會被換成 app 內建的版本。你釘住的版本也會一併失效。

要保留自己安裝的 CLI，把它裝到另一個目錄，並把該目錄放在 PATH 順序的前面。桌面 app 這一側的細節見[自動更新、安裝 CLI 與指令檔過期](desktop-update.md)。

## Node SDK

`@speclink/engine` 以 `npm install @speclink/engine` 為主要安裝路徑。實際可安裝的時點，以第一個帶 engine 的 release 為準；在那之前只能從 repo 建置。

下一步：[建立工作區與指令檔](init-workspace.md)。

**出處**：`cli-distribution`、`user-documentation`
