// 桌面建置前置：把 speclink CLI binary 佈到 Tauri externalBin 要求的
// target-triple 命名位置（desktop-release spec「桌面安裝檔內含同版 CLI」，design D5；
// dev-harness spec「dev 啟動自動佈署當前 checkout 的 sidecar」）。
//
// 用法：node scripts/desktop-sidecar.mjs [--profile <debug|release>] [--target <triple>]
//   --profile：debug｜release，無旗標預設 release（本機安裝與 CI 的既有呼叫形狀）
//   有 --target：cargo build -p speclink-cli --target <triple>（交叉編譯）
//   無 --target：host 編譯，triple 取自 rustc -vV 的 host
// 產出：apps/desktop/src-tauri/binaries/speclink-<triple>[.exe]
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/// cargo 的兩個 profile；dev 佈 debug（與 npm run cli 驗證所用同一顆），
/// 其餘呼叫者維持 release。
export const PROFILES = ['debug', 'release'];

/// spawnSync 的失敗形狀收攏（同 scripts/desktop-install.mjs 的 checkSpawn）：spawn 本身
/// 失敗（ENOENT、權限）與被訊號收束時 status 同為 null，一律報成「非零結束（null）」會
/// 丟掉真正的原因——predev 讓這支腳本每次 dev 啟動都跑，這訊息是缺 cargo 時唯一的線索。
export function checkSpawn(result, command, args) {
  if (result.error) {
    throw new Error(`${command} ${args.join(' ')} 無法執行：${result.error.message}`);
  }
  if (result.status !== 0) {
    const how = result.status === null ? `signal ${result.signal}` : result.status;
    throw new Error(`${command} ${args.join(' ')} 以非零結束（${how}）`);
  }
}

function run(command, args) {
  checkSpawn(spawnSync(command, args, { cwd: root, stdio: 'inherit' }), command, args);
}

function hostTriple() {
  const args = ['-vV'];
  const result = spawnSync('rustc', args, { encoding: 'utf8' });
  checkSpawn(result, 'rustc', args);
  const host = result.stdout?.match(/^host: (\S+)$/m)?.[1];
  if (!host) throw new Error('無法從 rustc -vV 取得 host triple');
  return host;
}

/// 某旗標後面接的值，沒給該旗標時為 null；旗標存在卻沒接值（或下一個 token 是
/// 另一個旗標）即失敗，不靜默當成「沒給」而落回預設。
function flagValue(argv, flag, expects) {
  const index = argv.indexOf(flag);
  if (index === -1) return null;
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) throw new Error(`${flag} 後必須接 ${expects}`);
  return value;
}

/// argv → {profile, target}。白名單外的 profile 值不靜默吞掉：設定選項要大聲失敗，
/// 否則 predev 被改成錯誤的值時只會表現為「dev 佈了不該佈的東西」。
export function parseSidecarArgs(argv) {
  const target = flagValue(argv, '--target', 'triple');
  const profile = flagValue(argv, '--profile', PROFILES.join(' 或 '));
  if (profile !== null && !PROFILES.includes(profile)) {
    throw new Error(`--profile 的值 ${profile} 不合法，合法值為 ${PROFILES.join('、')}`);
  }

  return { profile: profile ?? 'release', target };
}

function exeSuffix(triple) {
  return triple.includes('windows') ? '.exe' : '';
}

/// cargo 把產物放在哪：有 --target 時多一層 triple 目錄。
export function builtBinaryPath(rootDir, { profile, target, triple }) {
  return path.join(
    rootDir,
    'target',
    ...(target ? [target] : []),
    profile,
    `speclink${exeSuffix(triple)}`,
  );
}

/// 內容相同即跳過（決策三的防抖）：binaries/speclink-<triple> 在 cargo 的
/// rerun-if-changed 清單內，無條件覆蓋會更新 mtime，使每次 dev 啟動都多付一輪
/// speclink-desktop 重編。來源檔缺失時報錯而非回 false——那不是「內容相同」。
export function shouldCopy(source, dest) {
  if (!existsSync(source)) throw new Error(`找不到建置產物：${source}`);
  if (!existsSync(dest)) return true;
  return !readFileSync(source).equals(readFileSync(dest));
}

function main(argv) {
  const { profile, target } = parseSidecarArgs(argv);
  const triple = target ?? hostTriple();
  run('cargo', [
    'build',
    ...(profile === 'release' ? ['--release'] : []),
    '-p',
    'speclink-cli',
    ...(target ? ['--target', target] : []),
  ]);

  const built = builtBinaryPath(root, { profile, target, triple });
  const destDir = path.join(root, 'apps/desktop/src-tauri/binaries');
  mkdirSync(destDir, { recursive: true });
  const dest = path.join(destDir, `speclink-${triple}${exeSuffix(triple)}`);
  const shown = path.relative(root, dest);

  if (!shouldCopy(built, dest)) {
    console.log(`sidecar 內容未變，跳過複製：${shown}`);
    return;
  }
  copyFileSync(built, dest);
  console.log(`sidecar 佈署完成（${profile}）：${shown}`);
}

// node --test 匯入本模組時只取函式，不執行佈署。
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`desktop-sidecar: ${error.message}`);
    process.exit(1);
  }
}
