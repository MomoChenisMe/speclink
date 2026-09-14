// 桌面建置前置：把 speclink CLI binary 佈到 Tauri externalBin 要求的
// target-triple 命名位置（desktop-release spec「桌面安裝檔內含同版 CLI」，design D5；
// dev-harness spec「dev 啟動自動佈署當前 checkout 的 sidecar」）。
//
// 用法：node scripts/desktop/desktop-sidecar.mjs [--profile <debug|release>] [--target <triple>]
//   --profile：debug｜release，無旗標預設 release（本機安裝與 CI 的既有呼叫形狀）
//   有 --target：cargo build -p speclink-cli --target <triple>（交叉編譯）
//   --target universal-apple-darwin：對 aarch64 與 x86_64 兩個 Apple target 各建一次，
//     佈三份檔（release-assets-trim 設計 D2）：兩份 per-triple 複本給 tauri-build——
//     `tauri build --target universal-apple-darwin` 對兩個 triple 各跑一次 cargo build，
//     build script 各以 cargo 的 TARGET 找 speclink-<triple>，缺即失敗；再以 lipo 合成
//     一份 speclink-universal-apple-darwin 給 bundler（它要求 universal 建置的 external
//     binary 也是 universal，且命名帶 -universal-apple-darwin）
//   無 --target：host 編譯，triple 取自 rustc -vV 的 host
// 產出：apps/desktop/src-tauri/binaries/speclink-<triple>[.exe]
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

/// cargo 的兩個 profile；dev 佈 debug（與 npm run cli 驗證所用同一顆），
/// 其餘呼叫者維持 release。
export const PROFILES = ['debug', 'release'];

/// spawnSync 的失敗形狀收攏（同 scripts/desktop/desktop-install.mjs 的 checkSpawn）：spawn 本身
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

/// Tauri 的 universal 目標名與其展開的兩個 rustup target。
const UNIVERSAL_TRIPLE = 'universal-apple-darwin';
const UNIVERSAL_TARGETS = ['aarch64-apple-darwin', 'x86_64-apple-darwin'];

/// 建置計畫（純函式，可測）：跑哪幾次 cargo（null＝host 編譯）、產出哪幾個檔——每個輸出
/// 列出來源與目的檔，來源兩個以上即以 lipo 合成。universal 以外的 target 維持既有的
/// 單次建置、單一輸出形狀。
export function sidecarPlan(rootDir, { profile, target, triple }) {
  const destFor = (name) => path.join(rootDir, 'apps/desktop/src-tauri/binaries', `speclink-${name}${exeSuffix(name)}`);
  if (target === UNIVERSAL_TRIPLE) {
    const built = UNIVERSAL_TARGETS.map((t) => builtBinaryPath(rootDir, { profile, target: t, triple: t }));
    return {
      cargoTargets: UNIVERSAL_TARGETS,
      outputs: [
        ...UNIVERSAL_TARGETS.map((t, i) => ({ sources: [built[i]], dest: destFor(t) })),
        { sources: built, dest: destFor(UNIVERSAL_TRIPLE) },
      ],
    };
  }
  return {
    cargoTargets: [target],
    outputs: [{ sources: [builtBinaryPath(rootDir, { profile, target, triple })], dest: destFor(triple) }],
  };
}

/// 內容相同即跳過（決策三的防抖）：binaries/speclink-<triple> 在 cargo 的
/// rerun-if-changed 清單內，無條件覆蓋會更新 mtime，使每次 dev 啟動都多付一輪
/// speclink-desktop 重編。來源檔缺失時報錯而非回 false——那不是「內容相同」。
export function shouldCopy(source, dest) {
  if (!existsSync(source)) throw new Error(`找不到建置產物：${source}`);
  if (!existsSync(dest)) return true;
  return !readFileSync(source).equals(readFileSync(dest));
}

/// 佈一個輸出：多來源先 lipo 到暫存檔，再走同一套「內容相同即跳過」——合成結果才是
/// 要比對的東西。暫存目錄在 lipo 之前建立、finally 收掉，lipo 失敗也不留殘檔。
function deploy(output, profile) {
  const [built] = output.sources;
  const shown = path.relative(root, output.dest);
  const scratch = output.sources.length > 1 ? mkdtempSync(path.join(os.tmpdir(), 'speclink-lipo-')) : null;
  try {
    let source = built;
    if (scratch) {
      source = path.join(scratch, 'speclink');
      run('lipo', ['-create', ...output.sources, '-output', source]);
    }
    mkdirSync(path.dirname(output.dest), { recursive: true });
    if (!shouldCopy(source, output.dest)) {
      console.log(`sidecar 內容未變，跳過複製：${shown}`);
      return;
    }
    copyFileSync(source, output.dest);
    console.log(`sidecar 佈署完成（${profile}${scratch ? '，universal' : ''}）：${shown}`);
  } finally {
    if (scratch) rmSync(scratch, { recursive: true, force: true });
  }
}

function main(argv) {
  const { profile, target } = parseSidecarArgs(argv);
  const triple = target ?? hostTriple();
  const plan = sidecarPlan(root, { profile, target, triple });

  for (const cargoTarget of plan.cargoTargets) {
    run('cargo', [
      'build',
      ...(profile === 'release' ? ['--release'] : []),
      '-p',
      'speclink-cli',
      ...(cargoTarget ? ['--target', cargoTarget] : []),
    ]);
  }

  for (const output of plan.outputs) deploy(output, profile);
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
