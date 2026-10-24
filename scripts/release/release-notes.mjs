#!/usr/bin/env node
// Release 說明開頭的下載指南（desktop-release spec「Release 說明含下載指南」）。輸入
// --tag vX.Y.Z，雙語說明輸出至 stdout：英文指南接該版英文更新日誌片段在前，其後是預設
// 收合的「繁體中文」區塊（繁中指南接繁中片段）。片段由 release-notes.json 經同一份渲染
// 邏輯畫出（與兩份更新日誌共用）；release job 落檔後以 body_path 傳入 action-gh-release，
// 自動產生的比較連結接續於最後。
// 檔名清單即資產命名契約——與 Tauri bundler 的產出逐字一致，兩種語言共用同一份，對錯由
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

const assets = {
  dmg: `Speclink_${version}_universal.dmg`,
  exe: `Speclink_${version}_x64-setup.exe`,
  appImageX64: `Speclink_${version}_amd64.AppImage`,
  appImageArm64: `Speclink_${version}_aarch64.AppImage`,
  dockerImage: `ghcr.io/mosutw/speclink-server:${version}`,
};
const NPM_INSTALL = 'npm i -g @speclink/cli';
const SH_INSTALL = 'curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh';
const BREW_INSTALL = 'brew install MomoChenisMe/tap/speclink';
const NPX_SERVER = 'npx @speclink/server';
const DOCKER_SERVER = `docker run -d -p 8080:8080 -v speclink-data:/data ${assets.dockerImage}`;

const guideEn = `## 📦 Which file should I download?

| Your system | Download this (in Assets below) |
| --- | --- |
| macOS (Apple Silicon and Intel use the same file) | \`${assets.dmg}\` |
| Windows (x64) | \`${assets.exe}\` |
| Linux desktop (x64) | \`${assets.appImageX64}\` |
| Linux desktop (arm64) | \`${assets.appImageArm64}\` |
| Linux server or no graphical interface | No download — use the one-line CLI install below |

**One-line CLI install** (for people who want only the CLI, without the desktop app; pick one of three):

\`\`\`sh
# With Node.js (any platform)
${NPM_INSTALL}

# macOS/Linux without Node.js (servers, WSL, CI)
${SH_INSTALL}

# Homebrew (macOS/Linux)
${BREW_INSTALL}
\`\`\`

**One-line server start** (npx to try it locally, Docker to deploy):

\`\`\`sh
# Node.js 18+ starts a local server (SQLite by default, data in ./speclink-data, prints a /setup link)
${NPX_SERVER}

# Deploy with Docker (the deployment docs show the SQLite/PostgreSQL compose files)
${DOCKER_SERVER}
\`\`\`

> The desktop app uses \`.app.tar.gz\` and \`latest.json\` for automatic updates. You do not need to download them.
`;

const guideZh = `## 📦 我該下載哪個檔案？

| 你的環境 | 下載這個（在下方 Assets 區） |
| --- | --- |
| macOS（Apple Silicon 與 Intel 同一檔） | \`${assets.dmg}\` |
| Windows（x64） | \`${assets.exe}\` |
| Linux 桌面機（x64） | \`${assets.appImageX64}\` |
| Linux 桌面機（arm64） | \`${assets.appImageArm64}\` |
| Linux 伺服器或無圖形介面 | 不用下載——用下方的 CLI 一行安裝 |

**CLI 一行安裝**（只要 CLI、不裝桌面 app 的人；三種擇一）：

\`\`\`sh
# 有 Node.js（任何平台）
${NPM_INSTALL}

# 沒有 Node.js 的 macOS／Linux（伺服器、WSL、CI）
${SH_INSTALL}

# Homebrew（macOS／Linux）
${BREW_INSTALL}
\`\`\`

**Server 一行啟動**（本機試用 npx、部署用 Docker）：

\`\`\`sh
# 有 Node.js 18+ 就能起本機 server（預設 SQLite、資料在 ./speclink-data、印出 /setup 連結）
${NPX_SERVER}

# 部署走 Docker（SQLite／PostgreSQL compose 見部署文件）
${DOCKER_SERVER}
\`\`\`

> \`.app.tar.gz\` 與 \`latest.json\` 是桌面 App 自動更新機制用的，毋須手動下載。
`;

// <details> 裡的 Markdown 前後都要空一行，GitHub 才會渲染。
process.stdout.write(`${guideEn}
---

${renderEntry(entry, 'en')}
<details>
<summary>繁體中文</summary>

${guideZh}
---

${renderEntry(entry, 'zh-TW')}
</details>
`);
