#!/usr/bin/env node
// 一鍵 remote 開發編排：讀 repo root 的 .env（若存在）與 process env 合併，
// 插值生成 .dev/config.yaml 後同起 speclink-server 與 desktop 的 tauri dev。
// --server／--desktop 只起單邊（dev:server、dev:desktop），設定驗證三模式共用。
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

/// detached：child 自成 process group，收束時整組終止——cargo/npm 的孫 process
///（server binary、vite、tauri）不殘留。代價是終端 Ctrl+C 不會直達 child，
/// 由 main() 的 SIGINT/SIGTERM handler 轉送。
function spawnDevChild(cmd, args) {
  return spawn(cmd, args, {
    cwd: ROOT,
    stdio: 'inherit',
    shell: IS_WINDOWS && cmd === 'npm',
    detached: !IS_WINDOWS,
  });
}

/// argv → 啟動模式。--server／--desktop 對應 package.json 的 dev:server 與
/// dev:desktop，無旗標即整套編排（npm run dev 行為不變）。
export function parseDevMode(argv) {
  if (argv.includes('--server')) return 'server';
  if (argv.includes('--desktop')) return 'desktop';
  return 'full';
}

/// 長時間程序啟動前的同步前置條件：只有目前 checkout 的 CLI——測試者用
/// npm run cli 驗證的就是這顆 binary。server 與 desktop 單獨模式皆不含前置
/// 條件（規格「單獨啟動 server」「單獨啟動 desktop」）。
///
/// 此處不建 Desktop 前端：前端由 tauri dev 依 tauri.conf.json 的
/// beforeDevCommand 起 vite dev server 供應（devUrl）。在此預先建置只是白費一趟
/// ——dev 視窗載入的是 dev server，不是 apps/desktop/dist。
function devPrerequisites(mode) {
  if (mode !== 'full') return [];
  return [
    {
      message: 'speclink dev: 建置當前 checkout 的 speclink-cli…',
      failure: 'speclink dev: 無法建置當前 checkout 的 speclink-cli',
      cmd: 'cargo',
      args: ['build', '-p', 'speclink-cli'],
      // cargo 是真 binary，不需要 shell。
      shell: false,
    },
  ];
}

/// prerequisite → 長時間 child 的啟動編排。任一 prerequisite 以非零狀態結束或
/// 無法啟動時回傳其狀態（無可用狀態時為 1）且 children 為空——不留「環境看似
/// 成功、CLI 尚不可用」的半完成狀態。全部成功時 status 為 null。
export function startDevEnvironment({
  addr,
  mode = 'full',
  runSync = spawnSync,
  spawnChild = spawnDevChild,
  log = console.log,
  logError = console.error,
}) {
  for (const step of devPrerequisites(mode)) {
    log(step.message);
    const result = runSync(step.cmd, step.args, {
      cwd: ROOT,
      stdio: 'inherit',
      shell: step.shell,
    });
    if (result.error || result.status !== 0) {
      // spawn 失敗時 stdio: 'inherit' 不會留下任何線索，必須自己點名。
      if (result.error) logError(`${step.failure}：${result.error.message}`);
      return { status: result.status ?? 1, children: [] };
    }
  }

  const children = [];
  if (mode !== 'desktop') {
    children.push(
      spawnChild('cargo', [
        'run', '-p', 'speclink-server', '--',
        '--config', '.dev/config.yaml', '--addr', addr,
      ]),
    );
  }
  if (mode !== 'server') {
    children.push(spawnChild('npm', ['run', 'tauri', '-w', 'apps/desktop', '--', 'dev']));
  }
  return { status: null, children };
}

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
  const mode = parseDevMode(process.argv);

  // 設定驗證三模式共用——壞掉的 .env 在任何入口都應立即被點名，
  // 而不是等到唯一會用到它的入口才爆。
  let generated;
  try {
    generated = buildDevConfig(fileEnv, process.env);
  } catch (err) {
    console.error(`speclink dev: ${err.message}`);
    process.exit(1);
  }

  // desktop 單獨模式沒有 server 會讀 config，不落地 .dev/。
  if (mode !== 'desktop') {
    mkdirSync(devDir, { recursive: true });
    writeFileSync(path.join(devDir, 'config.yaml'), generated.configYaml);
  }

  const started = startDevEnvironment({ addr: generated.addr, mode });
  if (started.status !== null) process.exit(started.status);
  const children = started.children;

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
