// 組裝 Tauri updater 的 latest.json（desktop-release spec「更新描述檔隨 release 發布」，
// design D2：由 workflow 組裝、端點固定 GitHub Releases）。
//
// 用法：node scripts/release-latest-json.mjs --tag v0.2.0 --dir <artifacts> \
//         --repo owner/repo [--out latest.json]
//
// --dir 契約：每個平台鍵一個子目錄（darwin-aarch64／darwin-x86_64／windows-x86_64／
// linux-x86_64，linux-aarch64 可選），內含恰好一個更新包與其同名 .sig。
// 缺任一必要平台、缺更新包或缺簽章一律以非零結束（fail-closed）——寧可不發布，
// 不產出缺平台或無簽章的描述檔。
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

const REQUIRED_PLATFORMS = ['darwin-aarch64', 'darwin-x86_64', 'windows-x86_64', 'linux-x86_64'];
const OPTIONAL_PLATFORMS = ['linux-aarch64'];

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

/// 讀出平台子目錄裡的更新包與簽章；任何缺漏都丟出點名該平台的錯誤。
function readPlatformEntry(dir, key, tag, repo) {
  let files;
  try {
    files = readdirSync(path.join(dir, key));
  } catch {
    throw new Error(`缺少平台 ${key}：找不到子目錄 ${path.join(dir, key)}`);
  }
  const sigs = files.filter((name) => name.endsWith('.sig'));
  if (sigs.length !== 1) {
    throw new Error(`平台 ${key}：預期恰好一個 .sig 簽章檔，實際 ${sigs.length} 個`);
  }
  const pkg = sigs[0].slice(0, -'.sig'.length);
  if (!files.includes(pkg)) {
    throw new Error(`平台 ${key}：簽章 ${sigs[0]} 找不到對應更新包 ${pkg}`);
  }
  return {
    url: `https://github.com/${repo}/releases/download/${tag}/${pkg}`,
    signature: readFileSync(path.join(dir, key, sigs[0]), 'utf8').trim(),
  };
}

function main() {
  const { tag, dir, repo, out } = parseArgs(process.argv.slice(2));

  const platforms = {};
  for (const key of REQUIRED_PLATFORMS) {
    platforms[key] = readPlatformEntry(dir, key, tag, repo);
  }
  for (const key of OPTIONAL_PLATFORMS) {
    try {
      platforms[key] = readPlatformEntry(dir, key, tag, repo);
    } catch {
      // 可選平台缺席不擋發布。
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
