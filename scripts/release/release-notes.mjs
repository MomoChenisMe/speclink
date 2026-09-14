#!/usr/bin/env node
// Release 說明開頭的下載指南（desktop-release spec「Release 說明含下載指南」；
// 設計 D7）。輸入 --tag vX.Y.Z，指南 markdown 輸出至 stdout，其後接該版的更新日誌
// 片段（release-notes.json 為真相、渲染邏輯與 CHANGELOG.md 共用）；release job 落檔後
// 以 body_path 傳入 action-gh-release，自動產生的比較連結接續於最後。
// 檔名清單即資產命名契約——與 Tauri bundler 的產出逐字一致，對錯由
// release-notes.test.mjs 看守。
import { readReleaseNotes, renderEntry } from './release-notes-render.mjs';

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

// 先找條目再輸出：JSON 缺該版時 stdout 保持空，release job 不會拿到半份說明。
let entries;
try {
  entries = readReleaseNotes();
} catch (error) {
  fail(error.message);
}
const entry = entries.find((candidate) => candidate.version === version);
if (!entry) fail(`release-notes.json 沒有 ${version} 的條目——先執行 /release 技能補上更新日誌再發 tag`);

process.stdout.write(`## 📦 我該下載哪個檔案？

| 你的環境 | 下載這個（在下方 Assets 區） |
| --- | --- |
| macOS（Apple Silicon 與 Intel 同一檔） | \`Speclink_${version}_universal.dmg\` |
| Windows（x64） | \`Speclink_${version}_x64-setup.exe\` |
| Linux 桌面機（x64） | \`Speclink_${version}_amd64.AppImage\` |
| Linux 桌面機（arm64） | \`Speclink_${version}_aarch64.AppImage\` |
| Linux 伺服器或無圖形介面 | 不用下載——用下方的 CLI 一行安裝 |

**CLI 一行安裝**（只要 CLI、不裝桌面 app 的人；三種擇一）：

\`\`\`sh
# 有 Node.js（任何平台）
npm i -g @speclink/cli

# 沒有 Node.js 的 macOS／Linux（伺服器、WSL、CI）
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# Homebrew（macOS／Linux）
brew install MomoChenisMe/tap/speclink
\`\`\`

**Server 一行啟動**（本機試用 npx、部署用 Docker）：

\`\`\`sh
# 有 Node.js 18+ 就能起本機 server（預設 SQLite、資料在 ./speclink-data、印出 /setup 連結）
npx @speclink/server

# 部署走 Docker（SQLite／PostgreSQL compose 見部署文件）
docker run -d -p 8080:8080 -v speclink-data:/data ghcr.io/momochenisme/speclink-server:${version}
\`\`\`

> \`.app.tar.gz\` 與 \`latest.json\` 是桌面 App 自動更新機制用的，毋須手動下載。

---

${renderEntry(entry)}`);
