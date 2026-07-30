#!/usr/bin/env node
// npm run cli -- <args>：固定執行目前 checkout 的 debug CLI。
// 存在的理由是版本正確性——PATH 上可能沒有 speclink，或裝的是另一個 checkout 的
// 版本；本 wrapper 讓「啟動環境」與「以同版 CLI 驗證」落在同一份原始碼上。
// binary 不存在時先自動於 checkout root 建置再執行——仍絕不 fallback 到 PATH。
// 需要純 machine-readable stdout 時用 npm run --silent cli -- <args>：wrapper
// 本身不寫任何 stdout，剩下的雜訊只有 npm 的 lifecycle 訊息。
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/// 目前 checkout 的 debug CLI binary 絕對路徑；Windows 需要 .exe。
/// 固定回傳 checkout 內路徑——不查 PATH，也不 fallback 到已安裝的 speclink。
export function checkoutCliPath(platform = process.platform) {
  return path.join(ROOT, 'target', 'debug', platform === 'win32' ? 'speclink.exe' : 'speclink');
}

/// 執行 checkout CLI 並回傳它的 exit code。<args> 不解析、不改寫、原序轉送；
/// stdio 全 inherit（互動輸入與既有 --json／--no-color 輸出直通終端）；環境原樣
/// 繼承。child cwd 優先取 npm 的 INIT_CWD，讓 npm --prefix <checkout> run cli
/// 仍作用於呼叫者所在的測試 repo；該值不存在時退回 wrapper 的 process.cwd()。
/// 不設 shell：<args> 必須只被 CLI 解析一次。
export function runCheckoutCli({
  args,
  env = process.env,
  platform = process.platform,
  fallbackCwd = process.cwd(),
  exists = existsSync,
  runSync = spawnSync,
  logError = console.error,
}) {
  const binary = checkoutCliPath(platform);

  // binary 不存在：先於 checkout root 自動建置（不受呼叫端 cwd 影響），
  // 建置失敗即收場——絕不 fallback 到 PATH 上的 speclink。build 的 stdout
  // 導向 stderr，保證 --json 等 machine-readable 輸出不被建置進度污染。
  if (!exists(binary)) {
    const build = runSync('cargo', ['build', '-p', 'speclink-cli'], {
      cwd: ROOT,
      env,
      stdio: ['inherit', 2, 'inherit'],
    });
    if (build.error) {
      logError(`speclink cli: 無法自動建置 speclink-cli：${build.error.message}`);
      return 1;
    }
    if (build.status !== 0) {
      logError('speclink cli: 自動建置 speclink-cli 失敗（見上方 cargo 輸出），未執行任何 CLI。');
      return build.status ?? 1;
    }
  }

  // 空的 INIT_CWD（殘留的 export、外部工具設空值）語意上就是「沒提供」；
  // 原樣傳給 spawn 只會換來一個看起來像 binary 不存在的 ENOENT。
  const cwd = env.INIT_CWD || fallbackCwd;
  const result = runSync(binary, args, { cwd, env, stdio: 'inherit' });

  // binary 缺失、不可執行或 spawn 失敗：stdio inherit 不會留下任何線索，必須
  // 自己點名，並且明確失敗——絕不靜默改用 PATH 上的 speclink。
  // ENOENT 只會點名 binary，但 cwd 不存在時形狀一模一樣，因此兩者都印出來。
  if (result.error) {
    logError(`speclink cli: 無法執行 checkout CLI ${binary}（cwd ${cwd}）：${result.error.message}`);
    // 自動建置已確保 binary 存在，走到這裡多半是 cwd 不存在或權限問題。
    logError('speclink cli: 請確認 cwd 存在且 binary 具執行權限。');
    return 1;
  }

  // 被 signal 收束時 status 為 null，不能當成成功。
  if (result.status === null) {
    logError(`speclink cli: checkout CLI 因 ${result.signal} 結束。`);
    return 1;
  }

  return result.status;
}

// node --test 匯入本模組時只取函式，不執行 CLI。
if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  process.exit(runCheckoutCli({ args: process.argv.slice(2) }));
}
