#!/usr/bin/env node
// 本機建置安裝入口（desktop-release spec「本機安裝的新鮮度斷言」，決策 7）：
// 把 sidecar 佈署 → 前端建置 → tauri bundle → 版號斷言收成單一指令，以機械斷言
// 取代「剛建的應該是新的」這種信任——2026-08-05 就是這份信任讓 v1.11 的 app 裝了
// 進來，橫幅把 v1.14 的檔案標成舊版、按下更新靜默降級 30 檔。
//
// 用法：node scripts/desktop-install.mjs [--install]
//   無旗標：建置並斷言 bundle 內 CLI 的引擎版號等於源碼版號
//   --install：續行覆蓋安裝 /Applications/Speclink.app 並再斷言一次（僅 macOS）
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, renameSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const APP = '/Applications/Speclink.app';
const BUNDLE_APP = path.join(ROOT, 'target/release/bundle/macos/Speclink.app');
/// bundle 內的 sidecar CLI（Tauri externalBin 在 macOS 落到 Contents/MacOS/）。
const SIDECAR_IN_APP = 'Contents/MacOS/speclink';

/// 源碼的產物層版號：speclink-core 的 MARKER_VERSION 常數是唯一真相。
export function sourceMarkerVersion(initRs) {
  const version = initRs.match(/MARKER_VERSION:\s*&str\s*=\s*"([^"]+)"/)?.[1];
  if (!version) throw new Error('crates/speclink-core/src/init.rs 讀不到 MARKER_VERSION 常數');
  return version;
}

/// 從 `speclink --version` 的輸出取引擎版號。舊 binary 不含 engine 段——那本身
/// 就是「這顆 CLI 比守門還舊」的證據，明確失敗而非當成通過。
export function engineVersionOf(versionOutput) {
  const version = versionOutput.match(/engine (\S+?)\)?\s*$/m)?.[1];
  if (!version) {
    throw new Error(`--version 輸出不含 engine 版號（過期的 binary？）：${versionOutput.trim()}`);
  }
  return version;
}

/// 版號斷言：不等即帶兩邊版號失敗（差在哪一眼可見）。
export function assertSameEngineVersion(label, actual, expected) {
  if (actual !== expected) {
    throw new Error(`${label} 的引擎版號是 ${actual}，源碼是 ${expected}——建置鏈沿用了過期 binary`);
  }
}

/// tauri build 簽章需要的環境變數中缺了哪些（見 desktop-installer-and-updater：
/// 只認 TAURI_SIGNING_PRIVATE_KEY，_PATH 會在最後簽章步驟才失敗）。
export function missingSigningEnv(env) {
  return ['TAURI_SIGNING_PRIVATE_KEY', 'TAURI_SIGNING_PRIVATE_KEY_PASSWORD'].filter(
    (key) => !env[key],
  );
}

/// 進入任何建置之前的一次性把關：非 macOS 帶 --install 不該先花十分鐘建置才發現
/// 裝不了；簽章 env 缺失時 tauri 會在最後一步才失敗，同樣白等整趟。
export function preflight({ env, platform, install }) {
  if (install && platform !== 'darwin') {
    throw new Error(`--install 僅支援 macOS，目前平台為 ${platform}`);
  }
  const missing = missingSigningEnv(env);
  if (missing.length > 0) {
    throw new Error(`簽章環境變數未設定：${missing.join('、')}——bundle 會在簽章步驟失敗`);
  }
}

/// spawnSync 的失敗形狀收攏：spawn 本身失敗（ENOENT、權限）與被訊號收束時
/// status 為 null，不能一律報成「非零結束（null）」而丟掉真正的原因。
function checkSpawn(result, command, args) {
  if (result.error) {
    throw new Error(`${command} ${args.join(' ')} 無法執行：${result.error.message}`);
  }
  if (result.status !== 0) {
    const how = result.status === null ? `signal ${result.signal}` : result.status;
    throw new Error(`${command} ${args.join(' ')} 以非零結束（${how}）`);
  }
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: ROOT, stdio: 'inherit', ...options });
  checkSpawn(result, command, args);
}

function capture(command, args) {
  const result = spawnSync(command, args, { cwd: ROOT, encoding: 'utf8' });
  checkSpawn(result, command, args);
  return result.stdout ?? '';
}

/// 某顆 CLI binary 自報的引擎版號。
function engineVersionOfBinary(binary) {
  if (!existsSync(binary)) throw new Error(`找不到 CLI：${binary}`);
  return engineVersionOf(capture(binary, ['--version']));
}

function main(argv) {
  const install = argv.includes('--install');
  // (0) 進任何建置之前把關：平台限定與簽章 env 一次驗完。
  preflight({ env: process.env, platform: process.platform, install });

  // (1) 這棵樹是什麼——安裝版只保證等於這棵樹，不保證等於 origin 最新。
  const head = capture('git', ['rev-parse', '--short', 'HEAD']).trim();
  const branch = capture('git', ['rev-parse', '--abbrev-ref', 'HEAD']).trim();
  const dirty = capture('git', ['status', '--porcelain']).trim() !== '';
  const expected = sourceMarkerVersion(
    readFileSync(path.join(ROOT, 'crates/speclink-core/src/init.rs'), 'utf8'),
  );
  console.log(`來源：${branch} @ ${head}${dirty ? '（工作樹有未提交變更）' : '（工作樹乾淨）'}`);
  console.log(`源碼引擎版號：${expected}`);

  // (2) sidecar 永遠重建：src-tauri/binaries/ 殘留的舊 CLI 會被 externalBin 靜默
  // 打包，那正是 2026-08-05 裝進 v1.11 引擎的路徑。
  run('node', ['scripts/desktop-sidecar.mjs']);

  // (3) 前端建置與 bundle（簽章 env 已於 preflight 驗過）。
  run('npm', ['run', 'build', '-w', 'apps/desktop']);
  run('npm', ['run', '-w', 'apps/desktop', 'tauri', '--', 'build', '--bundles', 'app']);

  // (4) 第一道斷言：打包進去的 CLI 是不是這棵樹的。
  assertSameEngineVersion(
    'bundle 內的 CLI',
    engineVersionOfBinary(path.join(BUNDLE_APP, SIDECAR_IN_APP)),
    expected,
  );
  console.log(`✓ bundle 內的 CLI 引擎版號為 ${expected}`);

  if (!install) {
    console.log(`建置完成：${path.relative(ROOT, BUNDLE_APP)}（加 --install 覆蓋安裝）`);
    return;
  }

  // (5) 執行中的 app 不代關：使用者可能正在裡面工作。pattern 錨定安裝版的
  // 執行檔路徑，避免 argv 恰含「Speclink.app」的無關程序（tail、grep）誤攔。
  // pgrep 的 exit code 語意：0＝有符合、1＝沒有符合、其餘＝pgrep 本身失敗——
  // 失敗不能當成「沒在跑」。
  const pg = spawnSync('pgrep', ['-f', `${APP}/Contents/MacOS/speclink-desktop`]);
  if (pg.error || (pg.status !== 0 && pg.status !== 1)) {
    throw new Error(`無法確認 app 是否執行中（pgrep：${pg.error?.message ?? `exit ${pg.status}`}）`);
  }
  if (pg.status === 0) {
    throw new Error('Speclink 正在執行——請先結束 app 再重跑（不代為結束程序）');
  }

  // (6) 覆蓋安裝：先整份拷到暫存路徑，cp 失敗時既有的 app 原封不動；成功後
  // 才刪舊換新，把「app 消失」的視窗縮到 rm＋rename 兩步。
  const staging = `${APP}.new`;
  rmSync(staging, { recursive: true, force: true });
  run('cp', ['-R', BUNDLE_APP, staging]);
  rmSync(APP, { recursive: true, force: true });
  try {
    renameSync(staging, APP);
  } catch (error) {
    throw new Error(
      `舊版已移除但改名失敗（${error.message}）——新版完整保留在 ${staging}，手動改名為 ${APP} 即可完成安裝`,
    );
  }

  // (7) 第二道斷言：裝進去的才算數。
  assertSameEngineVersion(
    '安裝版的 CLI',
    engineVersionOfBinary(path.join(APP, SIDECAR_IN_APP)),
    expected,
  );
  console.log(`✓ 已安裝至 ${APP}，引擎版號為 ${expected}`);
}

// node --test 匯入本模組時只取函式，不執行安裝。
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`desktop-install: ${error.message}`);
    process.exit(1);
  }
}
