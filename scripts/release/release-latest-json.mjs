// 組裝 Tauri updater 的 latest.json（desktop-release spec「更新描述檔隨 release 發布」，
// design D2：由 workflow 組裝、端點固定 GitHub Releases）。
//
// 用法：node scripts/release/release-latest-json.mjs --tag v0.2.0 --dir <artifacts> \
//         --repo owner/repo [--out latest.json]
//
// --dir 契約：每個「更新包目錄」對應一或多個 updater 平台鍵——darwin-universal 一個
// 目錄餵 darwin-aarch64 與 darwin-x86_64 兩鍵（macOS 出 universal，設計 D2／D3），
// windows-x86_64 與 linux-x86_64 各對應同名鍵，linux-aarch64 可選；每個目錄內含恰好
// 一個更新包與其同名 .sig。鍵名是 Tauri updater 依執行機器查表的契約，目錄名是我們
// 的佈置契約，兩者刻意分開。缺任一必要目錄、缺更新包或缺簽章一律以非零結束
// （fail-closed）——寧可不發布，不產出缺平台或無簽章的描述檔。
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

// 更新包目錄 → 該目錄餵的平台鍵。
const REQUIRED_DIRS = {
  'darwin-universal': ['darwin-aarch64', 'darwin-x86_64'],
  'windows-x86_64': ['windows-x86_64'],
  'linux-x86_64': ['linux-x86_64'],
};
const OPTIONAL_DIRS = { 'linux-aarch64': ['linux-aarch64'] };

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i];
    const value = argv[i + 1];
    if (!/^--(tag|dir|repo|out)$/.test(flag) || value === undefined) {
      throw new Error(`無法解析的參數：${flag}`);
    }
    args[flag.slice(2)] = value;
  }
  for (const required of ['tag', 'dir', 'repo']) {
    if (!args[required]) throw new Error(`缺少必要參數 --${required}`);
  }
  return args;
}

/// 讀出更新包目錄裡的更新包與簽章；任何缺漏都丟出點名該目錄的錯誤。
function readPackageEntry(dir, dirName, tag, repo) {
  let files;
  try {
    files = readdirSync(path.join(dir, dirName));
  } catch {
    throw new Error(`缺少更新包目錄 ${dirName}：找不到子目錄 ${path.join(dir, dirName)}`);
  }
  const sigs = files.filter((name) => name.endsWith('.sig'));
  if (sigs.length !== 1) {
    throw new Error(`更新包目錄 ${dirName}：預期恰好一個 .sig 簽章檔，實際 ${sigs.length} 個`);
  }
  const pkg = sigs[0].slice(0, -'.sig'.length);
  if (!files.includes(pkg)) {
    throw new Error(`更新包目錄 ${dirName}：簽章 ${sigs[0]} 找不到對應更新包 ${pkg}`);
  }
  return {
    url: `https://github.com/${repo}/releases/download/${tag}/${pkg}`,
    signature: readFileSync(path.join(dir, dirName, sigs[0]), 'utf8').trim(),
  };
}

function main() {
  const { tag, dir, repo, out } = parseArgs(process.argv.slice(2));

  const platforms = {};
  for (const [dirName, platformKeys] of Object.entries(REQUIRED_DIRS)) {
    const entry = readPackageEntry(dir, dirName, tag, repo);
    for (const platformKey of platformKeys) platforms[platformKey] = entry;
  }
  for (const [dirName, platformKeys] of Object.entries(OPTIONAL_DIRS)) {
    try {
      const entry = readPackageEntry(dir, dirName, tag, repo);
      for (const platformKey of platformKeys) platforms[platformKey] = entry;
    } catch {
      // 可選目錄缺席不擋發布。
    }
  }

  const manifest = {
    version: tag.replace(/^v/, ''),
    pub_date: new Date().toISOString(),
    platforms,
  };

  const json = `${JSON.stringify(manifest, null, 2)}\n`;
  if (out) {
    writeFileSync(out, json);
  } else {
    process.stdout.write(json);
  }
}

try {
  main();
} catch (error) {
  console.error(`release-latest-json: ${error.message}`);
  process.exit(1);
}
