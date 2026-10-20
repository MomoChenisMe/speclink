# 換機接續開發

- 分支：`feat/desktop-workflow-offline-security-review`。
- 變更：`desktop-workflow-and-offline-security-review`，保留原始九項需求、安全驗收 checkbox 與所有專案提案預設條列顯示。
- 程式任務 42／42 完成；全部任務 42／43。唯一未完成為 task 40（12.3），公司 Proxy 的 Desktop 實機驗收。不要代勾。
- 尚未執行此變更的正式審查、驗證或封存。需要時接續 `speclink-review`／`speclink-verify`，不因換機先封存。
- 驗證結果與已知限制見同目錄 `delivery-regression.md`；公司實機紀錄填入 `company-proxy.md`。

## 取得分支

```bash
git fetch origin
git switch --track origin/feat/desktop-workflow-offline-security-review
```

若本機已有同名分支，改用 `git switch feat/desktop-workflow-offline-security-review`，再執行 `git pull --ff-only`。

## 本機準備與審查基準

Git 不包含 node_modules、target、安裝的安全標準資料、憑證、裝置設定及 `.speclink/` 本機快取。新電腦依專案既有安裝方式準備依賴；要做安全審查時，先離線匯入／安裝所選版本的完整資料，再準備 `.security_kb/`，不要在審查期間下載資源。專案目前未開啟安全審查政策，不因換機自行開啟。

這次 Apply 的原始基準是 `faefda7dabde01b8d90974aea63f4f7a8bdd5c24`，開始時沒有髒檔。host-local 的 review baseline 不隨 Git 搬移；新主機若提示缺少基準，可核對此 commit 後明確指定它作為 `review resolve --base` 的基準，不要把換機後的 HEAD 當成原始開工基準。

不要提交 `.checks.lock`；它是執行時的本機鎖檔，不是提案紀錄。
