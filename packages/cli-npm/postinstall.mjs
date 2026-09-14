// @speclink/cli 的 postinstall（cli-distribution spec「CLI 以 npm 套件發布」；release-assets-trim
// 設計 D4，esbuild 的做法）：macOS／Linux 上把 bin/speclink 這個 JS shim 原地換成本機平台的
// 原生 binary——npm 建好的全域 symlink 直接指到原生執行檔，之後每次呼叫零 Node 啟動成本。
//
// 不置換的情況一律保留 shim、以 0 結束：Windows（npm 的 .cmd 殼以 node 執行 bin，換成
// 原生檔會壞）、Yarn（Yarn Berry 一律以 node 執行套件的 bin，非 JS 的 bin 跑不起來；
// esbuild 同款守門，依 npm_config_user_agent 判斷）、找不到平台 binary（optionalDependencies
// 被略過或平台不支援——錯誤留到執行時由 shim 報，安裝本身不該失敗）。
import { chmodSync, copyFileSync, realpathSync, renameSync, rmSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { resolveBinary } from './platform.mjs';

/** 置換決策與執行：回傳 {replaced, binary} 或 {replaced: false, reason}。userAgent 是
 * 套件管理器設的 npm_config_user_agent（直接執行時未設定）。 */
export function replaceShim({ platform, arch, shimPath, userAgent = '' }) {
  if (platform === 'win32') return { replaced: false, reason: 'win32' };
  if (userAgent.startsWith('yarn/')) return { replaced: false, reason: 'yarn' };
  const binary = resolveBinary(pathToFileURL(shimPath).href, platform, arch);
  if (!binary) return { replaced: false, reason: 'binary-missing' };

  // 同目錄暫存名寫好、設權限，再 rename 覆蓋——rename 是原子的，不會留下半個 shim。
  const staging = path.join(path.dirname(shimPath), `.speclink-${process.pid}`);
  try {
    copyFileSync(binary, staging);
    chmodSync(staging, 0o755);
    renameSync(staging, shimPath);
  } catch (error) {
    rmSync(staging, { force: true });
    throw error;
  }
  return { replaced: true, binary };
}

// 入口判定先 realpath 再比對（同 packages/server-npm/bin/speclink-server.mjs）：argv[1]
// 可能是經 symlink 的路徑，import.meta.url 則是 node 解析後的實體路徑，不 realpath 會
// 靜默不跑 main、shim 留著——安裝看起來成功，每次呼叫卻多付一次 Node 啟動。
const invokedAs = (() => {
  if (!process.argv[1]) return null;
  try {
    return pathToFileURL(realpathSync(process.argv[1])).href;
  } catch {
    return null;
  }
})();
if (invokedAs === import.meta.url) {
  const shimPath = path.join(path.dirname(fileURLToPath(import.meta.url)), 'bin', 'speclink');
  try {
    const outcome = replaceShim({
      platform: process.platform,
      arch: process.arch,
      shimPath,
      userAgent: process.env.npm_config_user_agent,
    });
    if (!outcome.replaced) {
      console.log(`speclink: 保留 JS shim（${outcome.reason}）`);
    }
  } catch (error) {
    // 置換失敗不擋安裝：shim 仍可用，只是多一次 Node 啟動。
    console.warn(`speclink: 置換 shim 失敗，保留 JS shim：${error.message}`);
  }
}
