#!/usr/bin/env node
// Release 說明開頭的下載指南（desktop-release spec「Release 說明含下載指南」；
// 設計 D7）。輸入 --tag vX.Y.Z，指南 markdown 輸出至 stdout；release job 落檔後
// 以 body_path 傳入 action-gh-release，自動 changelog 接續其後。
// 檔名清單即資產命名契約——與 Tauri bundler 的產出逐字一致，對錯由
// release-notes.test.mjs 看守。

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

const args = process.argv.slice(2);
const tagIndex = args.indexOf('--tag');
const tag = tagIndex >= 0 ? args[tagIndex + 1] : undefined;
if (!tag) fail('用法：release-notes.mjs --tag vX.Y.Z');
const match = /^v(\d+\.\d+\.\d+)$/.exec(tag);
if (!match) fail(`tag「${tag}」不符 vX.Y.Z 格式`);
const version = match[1];

process.stdout.write(`## 📦 我該下載哪個檔案？

| 你的環境 | 下載這個（在下方 Assets 區） |
| --- | --- |
| macOS（Apple Silicon，M 系列晶片） | \`Speclink_${version}_aarch64.dmg\` |
| macOS（Intel） | \`Speclink_${version}_x64.dmg\` |
| Windows（x64） | \`Speclink_${version}_x64-setup.exe\` |
| Linux（x64） | \`Speclink_${version}_amd64.AppImage\` 或 \`Speclink_${version}_amd64.deb\` |
| Linux（arm64） | \`Speclink_${version}_aarch64.AppImage\` 或 \`Speclink_${version}_arm64.deb\` |

**CLI 一行安裝**（毋須手動下載壓縮檔）：

\`\`\`sh
# macOS／Linux
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# Windows PowerShell
irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex

# Homebrew（macOS／Linux）
brew install MomoChenisMe/tap/speclink
\`\`\`

> 其餘檔案毋須手動下載：\`.app.tar.gz\` 與所有 \`.sig\`、\`latest.json\` 是桌面 App 自動更新機制用的；\`speclink-v${version}-*.tar.gz\`／\`.zip\` 是上面安裝腳本抓的 CLI 壓縮檔；\`SHA256SUMS.txt\` 是全部檔案的校驗碼。

---

`);
