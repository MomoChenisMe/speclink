#!/usr/bin/env node
// 一鍵 remote 開發編排：讀 repo root 的 .env（若存在）與 process env 合併，
// 插值生成 .dev/config.yaml 後同起 speclink-server 與 desktop 的 tauri dev。
// server 的組態 YAML 不做環境變數展開——插值只發生在這一層（同 deploy compose 的決策）。
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const VALID_DRIVERS = ['sqlite', 'serverfs', 'postgres', 'memory'];

/// .env 逐行解析：KEY=VALUE、跳過註解與空行；不支援多行值與變數展開。
export function parseDotenv(text) {
  const env = {};
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (line === '' || line.startsWith('#')) continue;
    const eq = line.indexOf('=');
    if (eq <= 0) continue;
    env[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
  }
  return env;
}

/// env → { configYaml, addr }。processEnv 蓋過 fileEnv（dotenv 慣例）。
/// 設定不合法（未知 driver、postgres 缺 URL）時直接 throw——啟動前 fail-closed。
export function buildDevConfig(fileEnv, processEnv) {
  const env = { ...fileEnv };
  for (const [key, value] of Object.entries(processEnv)) {
    if (value !== undefined) env[key] = value;
  }

  const driver = env.SPECLINK_STORE_DRIVER ?? 'sqlite';
  const port = env.SPECLINK_PORT ?? '8080';
  const publicUrl = env.SPECLINK_PUBLIC_URL ?? `http://localhost:${port}`;
  const identityPath = env.SPECLINK_IDENTITY_PATH ?? '.dev/identity.db';

  let storeSection;
  switch (driver) {
    case 'sqlite':
      storeSection = `  driver: sqlite\n  path: ${env.SPECLINK_STORE_PATH ?? '.dev/store.db'}`;
      break;
    case 'serverfs':
      storeSection = `  driver: serverfs\n  path: ${env.SPECLINK_STORE_PATH ?? '.dev/store'}`;
      break;
    case 'postgres': {
      const url = env.SPECLINK_POSTGRES_URL;
      if (!url) {
        throw new Error(
          'SPECLINK_STORE_DRIVER=postgres 需要設定 SPECLINK_POSTGRES_URL（連線 URL 可不含密碼，由 SPECLINK_POSTGRES_PASSWORD 補全）',
        );
      }
      storeSection = `  driver: postgres\n  url: ${url}`;
      break;
    }
    case 'memory':
      storeSection = '  driver: memory';
      break;
    default:
      throw new Error(
        `SPECLINK_STORE_DRIVER=${driver} 不是合法的 store driver（合法值：${VALID_DRIVERS.join('、')}）`,
      );
  }

  const configYaml = `# 本檔由 npm run dev 生成（scripts/dev.mjs），每次啟動整檔重寫——手改無效。
# 調整設定請編輯 repo root 的 .env（對照 .env.example）。
store:
${storeSection}
identity:
  driver: sqlite
  path: ${identityPath}
public_url: ${publicUrl}
`;

  return { configYaml, addr: `127.0.0.1:${port}` };
}

// --- 編排主流程 ---

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const IS_WINDOWS = process.platform === 'win32';

function main() {
  const devDir = path.join(ROOT, '.dev');

  // --reset：只刪 .dev/（不碰 .env 與 deploy/），對不存在的目錄冪等成功。
  // postgres 的資料在外部資料庫，不在 reset 範圍（見 .env.example）。
  if (process.argv.includes('--reset')) {
    rmSync(devDir, { recursive: true, force: true });
    console.log('speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。');
    return;
  }

  const envPath = path.join(ROOT, '.env');
  const fileEnv = existsSync(envPath) ? parseDotenv(readFileSync(envPath, 'utf8')) : {};

  let generated;
  try {
    generated = buildDevConfig(fileEnv, process.env);
  } catch (err) {
    console.error(`speclink dev: ${err.message}`);
    process.exit(1);
  }

  mkdirSync(devDir, { recursive: true });
  writeFileSync(path.join(devDir, 'config.yaml'), generated.configYaml);

  // tauri.conf.json 無 devUrl——tauri dev 直接載入 apps/desktop/dist。
  // 全新 checkout 沒有 dist（gitignored），先建一次前端資產。
  if (!existsSync(path.join(ROOT, 'apps/desktop/dist/index.html'))) {
    console.log('speclink dev: apps/desktop/dist 不存在，先建置前端資產…');
    const build = spawnSync('npm', ['run', 'build', '-w', 'apps/desktop'], {
      cwd: ROOT,
      stdio: 'inherit',
      shell: IS_WINDOWS,
    });
    if (build.status !== 0) process.exit(build.status ?? 1);
  }

  // detached：child 自成 process group，收束時整組終止——cargo/npm 的孫 process
  //（server binary、vite、tauri）不殘留。代價是終端 Ctrl+C 不會直達 child，
  // 由下方的 SIGINT/SIGTERM handler 轉送。
  const spawnChild = (cmd, args) =>
    spawn(cmd, args, {
      cwd: ROOT,
      stdio: 'inherit',
      shell: IS_WINDOWS && cmd === 'npm',
      detached: !IS_WINDOWS,
    });

  const children = [
    spawnChild('cargo', [
      'run', '-p', 'speclink-server', '--',
      '--config', '.dev/config.yaml', '--addr', generated.addr,
    ]),
    spawnChild('npm', ['run', 'tauri', '-w', 'apps/desktop', '--', 'dev']),
  ];

  let closing = false;
  let exitCode = 0;
  let alive = children.length;

  const killTree = (child) => {
    if (child.exitCode !== null || child.signalCode !== null) return;
    if (IS_WINDOWS) {
      spawnSync('taskkill', ['/pid', String(child.pid), '/T', '/F']);
    } else {
      try {
        process.kill(-child.pid, 'SIGTERM');
      } catch {}
    }
  };

  const beginShutdown = (code) => {
    if (!closing) {
      closing = true;
      exitCode = code;
    }
    for (const child of children) killTree(child);
  };

  for (const child of children) {
    child.on('exit', (code) => {
      alive -= 1;
      // 任一 child 先退出：終止另一個並以其 exit code 退出（避免半死狀態）。
      if (!closing) beginShutdown(code ?? 1);
      if (alive === 0) process.exit(exitCode);
    });
  }

  process.on('SIGINT', () => beginShutdown(0));
  process.on('SIGTERM', () => beginShutdown(0));
}

// node --test 匯入本模組時只取純函式，不觸發編排。
if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  main();
}
