#!/usr/bin/env node
// speclink-server 的 npm launcher（server-release spec「npm 套件一行啟動 server」；
// 設計 D9）。binary 只吃單一 YAML 且 fail closed；快速啟動仿 compose 的外掛法——
// 把環境變數插值成 YAML 落地資料目錄，再帶 --config spawn 對應平台子套件裡的
// binary。帶參數或 SPECLINK_CONFIG 時純透傳，行為與直接執行 binary 一致。

import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath, pathToFileURL } from 'node:url';

// 快速啟動只開放持久化 driver；memory 是測試組態，不在此列。
const QUICKSTART_DRIVERS = ['sqlite', 'serverfs', 'postgres'];

/** os/cpu → 平台子套件名尾碼；無對應（未支援平台）回 null。 */
export function platformPackageSuffix(platform, arch) {
  const supported = {
    'darwin-arm64': true,
    'darwin-x64': true,
    'linux-x64': true,
    'linux-arm64': true,
    'win32-x64': true,
  };
  const key = `${platform}-${arch}`;
  return supported[key] ? `server-${key}` : null;
}

/** YAML 雙引號 scalar：JSON 字串是合法 YAML，URL 帶冒號也安全。 */
const yamlString = (value) => JSON.stringify(String(value));

/** 環境變數 → 快速啟動計畫（組態 YAML、資料目錄、綁定位址）；錯誤以 error 欄位回報。 */
export function buildQuickstart(env) {
  const driver = env.SPECLINK_STORE || 'sqlite';
  if (!QUICKSTART_DRIVERS.includes(driver)) {
    return {
      error: `SPECLINK_STORE「${driver}」不支援；可用值：${QUICKSTART_DRIVERS.join('、')}`,
    };
  }
  const dataDir = path.resolve(env.SPECLINK_DATA_DIR || './speclink-data');
  const port = env.SPECLINK_PORT || '8080';
  const publicUrl = env.SPECLINK_PUBLIC_URL || `http://localhost:${port}`;

  let storeBlock;
  if (driver === 'postgres') {
    const url = env.SPECLINK_POSTGRES_URL;
    if (!url) {
      return {
        error:
          'SPECLINK_STORE=postgres 需要 SPECLINK_POSTGRES_URL（連線 URL；密碼可另拆 SPECLINK_POSTGRES_PASSWORD）',
      };
    }
    storeBlock = `store:\n  driver: ${yamlString('postgres')}\n  url: ${yamlString(url)}\n`;
  } else {
    const storePath =
      driver === 'sqlite' ? path.join(dataDir, 'store.db') : path.join(dataDir, 'store');
    storeBlock = `store:\n  driver: ${yamlString(driver)}\n  path: ${yamlString(storePath)}\n`;
  }

  const configYaml =
    `# 由 npm launcher 依環境變數產生（speclink-server 只吃單一 YAML；設計 D9）。\n` +
    storeBlock +
    `identity:\n  driver: ${yamlString('sqlite')}\n  path: ${yamlString(path.join(dataDir, 'identity.db'))}\n` +
    `public_url: ${yamlString(publicUrl)}\n`;

  return {
    dataDir,
    configPath: path.join(dataDir, 'config.yaml'),
    configYaml,
    addr: `127.0.0.1:${port}`,
  };
}

/** 參數與環境 → spawn 模式：零參數且無 SPECLINK_CONFIG 走快速啟動，其餘透傳。 */
export function decideSpawnArgs(args, env) {
  if (env.SPECLINK_CONFIG && !args.includes('--config')) {
    return { mode: 'passthrough', args: ['--config', env.SPECLINK_CONFIG, ...args] };
  }
  if (args.length === 0 && !env.SPECLINK_CONFIG) {
    return { mode: 'quickstart' };
  }
  return { mode: 'passthrough', args };
}

function resolveBinary() {
  const suffix = platformPackageSuffix(process.platform, process.arch);
  if (!suffix) {
    throw new Error(
      `此平台（${process.platform}/${process.arch}）沒有對應的 speclink-server 套件；` +
        `支援：macOS（arm64／x64）、Linux（x64／arm64）、Windows（x64）`,
    );
  }
  const require = createRequire(import.meta.url);
  const ownScope = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  ).name.split('/')[0];
  const binaryName = process.platform === 'win32' ? 'speclink-server.exe' : 'speclink-server';
  try {
    const pkgJson = require.resolve(`${ownScope}/${suffix}/package.json`);
    return path.join(path.dirname(pkgJson), binaryName);
  } catch {
    throw new Error(
      `找不到平台子套件 ${ownScope}/${suffix}——npm 安裝時的 optionalDependencies 可能被略過，` +
        `重新安裝並確認未帶 --no-optional`,
    );
  }
}

function main() {
  const args = process.argv.slice(2);
  let binary;
  try {
    binary = resolveBinary();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exit(1);
  }

  const decision = decideSpawnArgs(args, process.env);
  let spawnArgs;
  if (decision.mode === 'quickstart') {
    const plan = buildQuickstart(process.env);
    if (plan.error) {
      process.stderr.write(`${plan.error}\n`);
      process.exit(1);
    }
    mkdirSync(plan.dataDir, { recursive: true });
    writeFileSync(plan.configPath, plan.configYaml);
    spawnArgs = ['--config', plan.configPath, '--addr', plan.addr];
  } else {
    spawnArgs = decision.args;
  }

  const child = spawn(binary, spawnArgs, { stdio: 'inherit' });
  for (const signal of ['SIGINT', 'SIGTERM']) {
    process.on(signal, () => child.kill(signal));
  }
  child.on('error', (error) => {
    process.stderr.write(`${error.message}\n`);
    process.exit(1);
  });
  child.on('exit', (code, signal) => {
    process.exit(signal ? 1 : (code ?? 1));
  });
}

// 測試直接 import 本檔取用上面的純函式；只有作為執行入口時才 spawn。
// npm 的 bin shim 是 symlink：argv[1] 是 symlink 路徑、import.meta.url 是 node
// 解析後的實體路徑，必須先 realpath 再比對——比對原始 argv[1] 會讓 npx 呼叫
// 靜默不執行 main()。
const invokedAs = (() => {
  if (!process.argv[1]) return null;
  try {
    return pathToFileURL(realpathSync(process.argv[1])).href;
  } catch {
    return null;
  }
})();
if (invokedAs === import.meta.url) {
  main();
}
