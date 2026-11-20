#!/usr/bin/env node
// engine 主套件的使用端檢查（node-sdk-release spec「打包好的主套件通過使用端檢查」；
// 設計決策一到四）。把打包好的 tarball 當成使用端的相依：解開成暫存資料夾裡的
// node_modules，用 Node 載入主套件取得執行期的匯出名稱，再產生具名匯入全部名稱的
// TypeScript 檔，以 tsc 編譯（不輸出、不開 skipLibCheck）。
//
// 不用 npm install：release 觸發時 optionalDependencies 釘的是還沒發佈的版號，安裝
// 會去 registry 解析它們。解開 tarball 不連網、也不執行套件的安裝腳本。

import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { parseArgs } from 'node:util';

const USAGE = '用法：node scripts/npm/npm-engine-consumer-check.mjs --main <主套件 tgz> [--platform <平台子套件 tgz>]';

// npm 的套件名稱格式（可帶 @scope/）；不允許 .. 之類的路徑跳脫，name 只能落在 node_modules 底下。
const PACKAGE_NAME = /^(?:@[a-z0-9-~][a-z0-9-._~]*\/)?[a-z0-9-~][a-z0-9-._~]*$/;

// 每個子程序（tar、載入、tsc）的上限。正常都在幾秒內結束；卡住時由腳本自己停下並刪掉
// 暫存資料夾，不必等到被外部強制結束。
const STEP_TIMEOUT_MS = 120_000;

class CheckFailure extends Error {}

function parse() {
  let values;
  try {
    // multiple：同一個參數給兩次時明確拒絕，parseArgs 預設會默默取最後一個。
    ({ values } = parseArgs({
      options: { main: { type: 'string', multiple: true }, platform: { type: 'string', multiple: true } },
    }));
  } catch (e) {
    throw new CheckFailure(`${e.message}\n${USAGE}`);
  }
  const args = {};
  for (const key of ['main', 'platform']) {
    const given = values[key] ?? [];
    if (given.length > 1) throw new CheckFailure(`--${key} 只能給一次\n${USAGE}`);
    const file = given[0];
    if (file !== undefined && !(existsSync(file) && statSync(file).isFile())) {
      throw new CheckFailure(`--${key} 指定的 tarball 不存在：${file}`);
    }
    args[key] = file;
  }
  if (!args.main) throw new CheckFailure(`缺少 --main\n${USAGE}`);
  return args;
}

/// 從執行當下的工作目錄解析 tsc 的入口，之後以目前的 Node 執行它（不經 .cmd 轉接檔）。
function resolveTsc() {
  try {
    const pkgFile = createRequire(path.join(process.cwd(), 'noop.js')).resolve('typescript/package.json');
    return path.join(path.dirname(pkgFile), JSON.parse(readFileSync(pkgFile, 'utf8')).bin.tsc);
  } catch {
    throw new CheckFailure(
      `從 ${process.cwd()} 解析不到 typescript；請在有安裝 typescript 的資料夾執行（例如 repo 根或 crates/adapters/speclink-node）`,
    );
  }
}

/// 在使用端資料夾裡執行一個子程序，參數以陣列傳入（不經 shell）。
function runIn(consumer, command, args) {
  return spawnSync(command, args, { cwd: consumer, encoding: 'utf8', timeout: STEP_TIMEOUT_MS });
}

/// 把 tarball 複製進使用端資料夾、以相對檔名解開（Windows 上 GNU tar 會把含磁碟機代號
/// 的路徑當成遠端主機），再把 package/ 搬到 node_modules 下由 name 決定的位置。
function unpackIntoNodeModules(consumer, tgz) {
  copyFileSync(tgz, path.join(consumer, 'incoming.tgz'));
  const tar = runIn(consumer, 'tar', ['-xzf', 'incoming.tgz']);
  if (tar.status !== 0) throw new CheckFailure(`解不開 ${tgz}：\n${tar.error?.message ?? tar.stderr}`);
  rmSync(path.join(consumer, 'incoming.tgz'));

  const extracted = path.join(consumer, 'package');
  const pkgFile = path.join(extracted, 'package.json');
  if (!existsSync(pkgFile)) throw new CheckFailure(`${tgz} 裡沒有 package/package.json`);
  let name;
  try {
    ({ name } = JSON.parse(readFileSync(pkgFile, 'utf8')));
  } catch (e) {
    throw new CheckFailure(`${tgz} 的 package/package.json 不是合法的 JSON：${e.message}`);
  }
  if (typeof name !== 'string' || !PACKAGE_NAME.test(name)) {
    throw new CheckFailure(`${tgz} 的 package.json 的 name 不是合法的套件名稱：${JSON.stringify(name)}`);
  }
  const target = path.join(consumer, 'node_modules', name);
  mkdirSync(path.dirname(target), { recursive: true });
  renameSync(extracted, target);
  return name;
}

function check(consumer, args, tsc) {
  const name = unpackIntoNodeModules(consumer, args.main);
  if (args.platform !== undefined) unpackIntoNodeModules(consumer, args.platform);

  // 載入在子程序裡做。套件名稱與結果檔的路徑以參數傳入，不拼進程式碼；-- 讓以 - 開頭的
  // 名稱不被 Node 當成自己的選項。匯出名稱寫進檔案而不是 stdout：套件載入時自己印的字
  // 不影響結果。
  const namesFile = path.join(consumer, 'export-names.json');
  const load = runIn(consumer, process.execPath, [
    '-e',
    "require('node:fs').writeFileSync(process.argv[2], JSON.stringify(Object.keys(require(process.argv[1]))))",
    '--',
    name,
    namesFile,
  ]);
  if (load.status !== 0) {
    throw new CheckFailure(`load failed: ${name} 載入時拋出錯誤\n${load.error?.message ?? load.stderr}`);
  }
  const exportNames = JSON.parse(readFileSync(namesFile, 'utf8'));
  // 沒有名稱可以比對時，下面的匯入那一行是空的、編譯一定通過，等於什麼都沒檢查。
  if (exportNames.length === 0) {
    throw new CheckFailure(`load failed: ${name} 載入後沒有任何執行期匯出的名稱，沒有東西可以比對`);
  }

  // 一行具名匯入全部名稱、一行放進匯出的陣列，避免未使用的匯入被略過。匯入名稱寫成字串
  // 形式並各配一個別名（TypeScript 5.6 起支援）：default 或含連字號的名稱照樣比對。
  const aliases = exportNames.map((_, index) => `e${index}`);
  const specifiers = exportNames.map((exportName, index) => `${JSON.stringify(exportName)} as ${aliases[index]}`);
  writeFileSync(
    path.join(consumer, 'consumer.mts'),
    `import { ${specifiers.join(', ')} } from ${JSON.stringify(name)};\nexport const imported = [${aliases.join(', ')}];\n`,
  );
  writeFileSync(
    path.join(consumer, 'tsconfig.json'),
    `${JSON.stringify(
      {
        compilerOptions: {
          module: 'nodenext',
          moduleResolution: 'nodenext',
          strict: true,
          noEmit: true,
          skipLibCheck: false,
          types: [],
        },
        files: ['consumer.mts'],
      },
      null,
      2,
    )}\n`,
  );
  const compile = runIn(consumer, process.execPath, [tsc, '-p', 'tsconfig.json', '--pretty', 'false']);
  if (compile.status !== 0) {
    throw new CheckFailure(
      `type check failed: ${name} 的型別定義檔編譯不過\n${compile.error?.message ?? `${compile.stdout}${compile.stderr}`}`,
    );
  }
  return `通過使用端檢查：${name} 的 ${exportNames.length} 個匯出名稱都能載入並從型別定義檔具名匯入`;
}

function main() {
  const args = parse();
  const tsc = resolveTsc();
  const consumer = mkdtempSync(path.join(os.tmpdir(), 'speclink-consumer-check-'));
  try {
    return check(consumer, args, tsc);
  } finally {
    rmSync(consumer, { recursive: true, force: true });
  }
}

// 註冊處理函式，Node 才不會在收到 SIGINT／SIGTERM 的當下結束（預設會立即結束、跳過
// finally）。main 全程同步，處理函式要等 main 跑完、暫存資料夾刪掉之後才執行：它把結束碼
// 改成慣例的 128＋訊號編號，被中斷的檢查因此不會以 0 結束。
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => {
    process.stderr.write(`interrupted: 收到 ${signal}\n`);
    process.exitCode = 128 + os.constants.signals[signal];
  });
}

try {
  process.stdout.write(`${main()}\n`);
} catch (e) {
  if (!(e instanceof CheckFailure)) throw e;
  process.stderr.write(`${e.message}\n`);
  process.exitCode = 1;
}

// 訊號的處理函式不會讓事件迴圈繼續跑。多排一輪，main 執行期間收到的訊號才處理得到。
// 用 50 毫秒的計時器而不是 setImmediate：Node 20 要多轉幾輪才讀得到訊號，setImmediate 或
// 0 毫秒的計時器那一輪常常還沒讀到就結束。事件迴圈在這段時間裡等著，訊號一到就處理。
setTimeout(() => {}, 50);
