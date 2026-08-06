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
import { existsSync, readFileSync, rmSync } from 'node:fs';
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

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: ROOT, stdio: 'inherit', ...options });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} 以非零結束（${result.status}）`);
  }
}

function capture(command, args) {
  const result = spawnSync(command, args, { cwd: ROOT, encoding: 'utf8' });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} 以非零結束（${result.status}）`);
  }
  return result.stdout ?? '';
}

/// 某顆 CLI binary 自報的引擎版號。
function engineVersionOfBinary(binary) {
  if (!existsSync(binary)) throw new Error(`找不到 CLI：${binary}`);
  return engineVersionOf(capture(binary, ['--version']));
}

function main(argv) {
  const install = argv.includes('--install');
  // 平台限定先攔：非 macOS 帶 --install 不該先花十分鐘建置才發現裝不了。
  if (install && process.platform !== 'darwin') {
    throw new Error(`--install 僅支援 macOS，目前平台為 ${process.platform}`);
  }

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

  // (3) 前端建置與 bundle。簽章 env 缺失時 tauri 會在最後一步才失敗，白等整趟。
  const missing = missingSigningEnv(process.env);
  if (missing.length > 0) {
    throw new Error(`簽章環境變數未設定：${missing.join('、')}——bundle 會在簽章步驟失敗`);
  }
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

  // (5) 執行中的 app 不代關：使用者可能正在裡面工作。
  if (spawnSync('pgrep', ['-f', 'Speclink.app']).status === 0) {
    throw new Error('Speclink 正在執行——請先結束 app 再重跑（不代為結束程序）');
  }

  // (6) 覆蓋安裝。
  rmSync(APP, { recursive: true, force: true });
  run('cp', ['-R', BUNDLE_APP, APP]);

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
