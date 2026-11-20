// engine 主套件的使用端檢查（node-sdk-release spec「打包好的主套件通過使用端檢查」；
// 設計決策一到四）。以假套件的 tarball 驅動 npm-engine-consumer-check.mjs，涵蓋
// 通過、型別定義檔語法錯誤、執行期匯出缺宣告、載入失敗四種結果與四種用法錯誤，以及
// sharp-edges 稽核補上的案例（參數給兩次、name 含路徑跳脫、name 以 - 開頭、中斷）。
//
// 每個案例給腳本一個自己的系統暫存資料夾（TMPDIR／TEMP／TMP），案例並行時才能各自
// 斷言「腳本沒有留下使用端資料夾」，不被別的案例同時建立的資料夾干擾。
import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { describe, test } from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const script = path.join(root, 'scripts', 'npm', 'npm-engine-consumer-check.mjs');

const GOOD_JS = `'use strict'
module.exports = { createEngine() {}, agentTool: { name: 'speclink' } }
`;

const GOOD_DTS = `export interface Host {
  /** Example: express.raw({ type: 'application/octet-stream' }) */
  handleRequest(): void
}
export function createEngine(): Host
export const agentTool: { name: string }
`;

function packageJson(extra = {}) {
  return `${JSON.stringify({ name: '@speclink/engine', version: '0.0.0', main: 'index.js', types: 'index.d.ts', ...extra }, null, 2)}\n`;
}

/// 在 src 底下的 sub 資料夾組出假套件並壓成 tgz，回傳 tgz 路徑。預設是完好的
/// @speclink/engine；overrides 只寫和完好套件不同的檔，值為 undefined 時不寫那個檔。
/// tar 的工作目錄設在套件的上一層、只帶相對檔名（和腳本解開時同一個理由）。
function fakeTarball(src, sub, overrides = {}) {
  const dir = path.join(src, sub);
  const pkgDir = path.join(dir, 'package');
  mkdirSync(pkgDir, { recursive: true });
  const files = { 'package.json': packageJson(), 'index.js': GOOD_JS, 'index.d.ts': GOOD_DTS, ...overrides };
  for (const [name, content] of Object.entries(files)) {
    if (content !== undefined) writeFileSync(path.join(pkgDir, name), content);
  }
  execFileSync('tar', ['-czf', 'engine.tgz', 'package'], { cwd: dir });
  return path.join(dir, 'engine.tgz');
}

/// 非同步執行腳本，案例之間才真的並行。工作目錄預設是 repo 根（typescript 從那裡解析）。
function start(args, tmpDir, options = {}) {
  const child = spawn(process.execPath, [script, ...args], {
    cwd: root,
    env: { ...process.env, TMPDIR: tmpDir, TEMP: tmpDir, TMP: tmpDir },
    ...options,
  });
  let stdout = '';
  let stderr = '';
  child.stdout.setEncoding('utf8').on('data', (chunk) => (stdout += chunk));
  child.stderr.setEncoding('utf8').on('data', (chunk) => (stderr += chunk));
  const done = new Promise((resolve, reject) => {
    child.on('error', reject);
    child.on('close', (status, signal) => resolve({ status, signal, stdout, stderr }));
  });
  return { child, done };
}

/// 每個案例一個工作區：src/ 放假套件與 tgz，tmp/ 是腳本看到的系統暫存資料夾。兩個資料夾
/// 的名稱都含空白與非 ASCII 字元，守住「tarball 路徑與暫存路徑不受字元影響」。
/// 案例跑完斷言 tmp/ 裡沒有腳本留下的東西，再整個刪掉工作區。tsc 會呼叫
/// module.enableCompileCache()，在系統暫存資料夾留下 Node 共用的編譯快取，不算在內。
async function withCase(body) {
  const base = mkdtempSync(path.join(os.tmpdir(), 'speclink-consumer-fixture-'));
  const src = path.join(base, 'src 規格 ü');
  const tmp = path.join(base, 'tmp 暫存');
  mkdirSync(src);
  mkdirSync(tmp);
  try {
    await body({
      base,
      src,
      run: (args, options) => start(args, tmp, options).done,
      start: (args, options) => start(args, tmp, options),
    });
    const leftovers = readdirSync(tmp).filter((name) => name !== 'node-compile-cache');
    assert.deepEqual(leftovers, [], '腳本結束後不得在系統暫存資料夾留下使用端資料夾');
  } finally {
    rmSync(base, { recursive: true, force: true });
  }
}

/// 輪詢等檔案出現（中斷案例用來確認腳本已經進到載入那一步）。
async function waitForFile(file) {
  for (let i = 0; i < 200 && !existsSync(file); i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.ok(existsSync(file), `等不到 ${file}`);
}

/// 對一個「載入時寫下記號、再睡 sleepMs」的套件執行腳本，看到記號（使用端資料夾已經建立、
/// 正在載入）才以 send 送出訊號，回傳腳本的結果。腳本放在獨立的程序群組，send 才能選擇
/// 只送給腳本或送給整組。
async function interruptDuringLoad({ base, src, start }, sleepMs, send) {
  const marker = path.join(base, 'loading');
  const js = `'use strict'\nrequire('node:fs').writeFileSync(${JSON.stringify(marker)}, '')\nAtomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ${sleepMs})\n${GOOD_JS}`;
  const tgz = fakeTarball(src, 'slow', { 'index.js': js });
  const { child, done } = start(['--main', tgz], { detached: true });
  await waitForFile(marker);
  send(child);
  return done;
}

function firstLine(text) {
  return text.split(/\r?\n/, 1)[0];
}

const NO_POSIX_SIGNALS = process.platform === 'win32' && 'Windows 沒有可以攔截的 SIGINT／SIGTERM';

describe('npm-engine-consumer-check.mjs', { concurrency: true }, () => {
  test('完好的套件通過：主套件載入平台子套件，optionalDependencies 指向未發佈的版本也不連線，兩個 tgz 位元組不變', () =>
    withCase(async ({ src, run }) => {
      const platform = fakeTarball(src, 'platform', { 'package.json': packageJson({ name: '@speclink/engine-fake-platform' }) });
      // 主套件的匯出來自平台子套件：子套件沒被放到主套件載入得到的位置，載入就會失敗。
      const main = fakeTarball(src, 'main', {
        'package.json': packageJson({ optionalDependencies: { '@speclink/engine-fake-platform': '0.0.0-never-published' } }),
        'index.js': `'use strict'\nmodule.exports = require('@speclink/engine-fake-platform')\n`,
      });
      const before = [readFileSync(main), readFileSync(platform)];
      const result = await run(['--main', main, '--platform', platform]);
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, /@speclink\/engine/, 'stdout 須含主套件名稱');
      assert.match(result.stdout, /2 個匯出/, 'stdout 須含檢查到的匯出名稱個數');
      assert.ok(readFileSync(main).equals(before[0]), '檢查不得改動傳入的主套件 tarball');
      assert.ok(readFileSync(platform).equals(before[1]), '檢查不得改動傳入的平台子套件 tarball');
    }));

  test('套件載入時往 stdout 印字不影響結果', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'noisy', { 'index.js': `${GOOD_JS}console.log('native module loaded')\n` });
      const result = await run(['--main', tgz]);
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, /2 個匯出/);
    }));

  test("型別定義檔的區塊註解含 '*/*' 時失敗，stderr 帶 TS1131 與 index.d.ts", () =>
    withCase(async ({ src, run }) => {
      const brokenDts = GOOD_DTS.replace("type: 'application/octet-stream'", "type: '*/*', limit: '4mb'");
      const tgz = fakeTarball(src, 'broken-dts', { 'index.d.ts': brokenDts });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.match(firstLine(result.stderr), /^type check failed/, result.stderr);
      assert.match(result.stderr, /TS1131/);
      assert.match(result.stderr, /index\.d\.ts/);
    }));

  test('執行期多匯出 commandNames 而型別定義檔沒有宣告時失敗，stderr 點名 commandNames', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'undeclared', { 'index.js': `${GOOD_JS}module.exports.commandNames = ['list']\n` });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.match(firstLine(result.stderr), /^type check failed/, result.stderr);
      assert.match(result.stderr, /commandNames/);
    }));

  test('index.js 載入時拋錯時以 load failed 失敗，不執行編譯', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'unloadable', { 'index.js': `'use strict'\nrequire('./binding.js')\n` });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.match(firstLine(result.stderr), /^load failed/, result.stderr);
      assert.doesNotMatch(result.stderr, /error TS/, '載入失敗時不得執行編譯');
    }));

  // --- 沒有匯出、名稱不是合法識別字（design 決策三的後兩段） ---

  for (const [label, js] of [
    ['空物件', `'use strict'\nmodule.exports = {}\n`],
    ['沒有屬性的函式', `'use strict'\nmodule.exports = function () {}\n`],
  ]) {
    test(`index.js 匯出的是${label}時以 load failed 失敗，不執行編譯`, () =>
      withCase(async ({ src, run }) => {
        const tgz = fakeTarball(src, 'no-exports', { 'index.js': js });
        const result = await run(['--main', tgz]);
        assert.notEqual(result.status, 0, '沒有名稱可以比對時不得通過');
        assert.match(firstLine(result.stderr), /^load failed/, result.stderr);
        assert.match(result.stderr, /沒有任何執行期匯出/);
        assert.doesNotMatch(result.stderr, /error TS/, '沒有匯出時不得執行編譯');
      }));
  }

  test('執行期匯出 a-b 與 default、型別定義檔宣告了 a-b 時通過，匯出個數包含這兩個名稱', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'odd-names', {
        'index.js': `${GOOD_JS}module.exports['a-b'] = 1\nmodule.exports.default = 2\n`,
        'index.d.ts': `${GOOD_DTS}declare const dashed: number\nexport { dashed as "a-b" }\n`,
      });
      const result = await run(['--main', tgz]);
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, /4 個匯出/);
    }));

  test('執行期匯出 x-y 而型別定義檔沒有宣告時失敗，stderr 點名 x-y，不是語法錯誤', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'odd-undeclared', { 'index.js': `${GOOD_JS}module.exports['x-y'] = 1\n` });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.match(firstLine(result.stderr), /^type check failed/, result.stderr);
      assert.match(result.stderr, /TS2305/);
      assert.match(result.stderr, /x-y/);
      assert.doesNotMatch(result.stderr, /error TS1\d{3}/, '名稱的寫法不得變成語法錯誤');
    }));

  // --- 用法錯誤（design 的 Implementation Contract 列的四種） ---

  test('不給 --main 時以用法錯誤結束，stderr 點名 --main', () =>
    withCase(async ({ run }) => {
      const result = await run([]);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /--main/);
    }));

  test('--main 指定的 tarball 不存在時失敗，stderr 點名參數與那個路徑', () =>
    withCase(async ({ src, run }) => {
      const missing = path.join(src, 'missing.tgz');
      const result = await run(['--main', missing]);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /--main/);
      assert.ok(result.stderr.includes(missing), `stderr 須點名 ${missing}：\n${result.stderr}`);
    }));

  test('tgz 裡沒有 package/package.json 時失敗，stderr 點名那個 tgz', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'no-manifest', { 'package.json': undefined });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.ok(result.stderr.includes(tgz), `stderr 須點名 ${tgz}：\n${result.stderr}`);
      assert.match(result.stderr, /package\/package\.json/);
    }));

  test('package/package.json 不是合法的 JSON 時失敗，stderr 點名那個 tgz 而不是印出堆疊', () =>
    withCase(async ({ src, run }) => {
      const tgz = fakeTarball(src, 'bad-manifest', { 'package.json': '{' });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.ok(result.stderr.includes(tgz), `stderr 須點名 ${tgz}：\n${result.stderr}`);
      assert.doesNotMatch(result.stderr, /SyntaxError|^\s+at /m, '用法錯誤不得以未攔截的例外結束');
    }));

  test('從解析不到 typescript 的資料夾執行時失敗，stderr 點名那個資料夾', () =>
    withCase(async ({ base, src, run }) => {
      const tgz = fakeTarball(src, 'good');
      // 案例的工作區在系統暫存資料夾底下，往上找不到 typescript。
      const result = await run(['--main', tgz], { cwd: base });
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /typescript/);
      assert.ok(result.stderr.includes(path.basename(base)), `stderr 須點名執行的資料夾：\n${result.stderr}`);
    }));

  // --- sharp-edges 稽核（任務 1.3）補上的案例 ---

  test('同一個參數給兩次時以用法錯誤結束，不默默取最後一個', () =>
    withCase(async ({ src, run }) => {
      const main = fakeTarball(src, 'main');
      const platform = fakeTarball(src, 'platform', { 'package.json': packageJson({ name: '@speclink/engine-fake-platform' }) });

      const twiceMain = await run(['--main', main, '--main', main]);
      assert.notEqual(twiceMain.status, 0, '--main 給兩次不得通過');
      assert.match(twiceMain.stderr, /--main/);

      const twicePlatform = await run(['--main', main, '--platform', platform, '--platform', platform]);
      assert.notEqual(twicePlatform.status, 0, '--platform 給兩次不得通過');
      assert.match(twicePlatform.stderr, /--platform/);
    }));

  test('package.json 的 name 含路徑跳脫時失敗，不寫到使用端資料夾之外', () =>
    withCase(async ({ src, run }) => {
      // 使用端資料夾在 tmp/ 底下：../../escaped 會落到 tmp/escaped，withCase 的殘留斷言抓得到。
      const tgz = fakeTarball(src, 'escape', { 'package.json': packageJson({ name: '../../escaped' }) });
      const result = await run(['--main', tgz]);
      assert.notEqual(result.status, 0);
      assert.ok(result.stderr.includes('../../escaped'), `stderr 須點名那個 name：\n${result.stderr}`);
      assert.doesNotMatch(firstLine(result.stderr), /^load failed/, '名稱不合法要在解開時擋下，不是到載入才失敗');
    }));

  test('名稱以 - 開頭的套件照樣被當成套件名稱載入，不被 Node 當成自己的選項', () =>
    withCase(async ({ src, run }) => {
      // node -e <程式> 之後的參數若以 - 開頭會被 Node 解析成選項；-r 是 --require。
      const tgz = fakeTarball(src, 'dash', { 'package.json': packageJson({ name: '-r' }) });
      const result = await run(['--main', tgz]);
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, /2 個匯出/);
    }));

  test('Ctrl-C（整個程序群組收到 SIGINT）時刪掉暫存資料夾，並以 130 結束', { skip: NO_POSIX_SIGNALS }, () =>
    withCase(async (workspace) => {
      // 載入的子程序也收到訊號而結束，所以睡多久都不必等完。
      const result = await interruptDuringLoad(workspace, 10000, (child) => process.kill(-child.pid, 'SIGINT'));
      // 腳本要自己結束（刪完暫存資料夾才結束），不是被訊號直接殺掉；殘留由 withCase 斷言。
      assert.equal(result.signal, null, `腳本不得被訊號直接結束：${JSON.stringify(result)}`);
      assert.equal(result.status, 130, JSON.stringify(result));
    }));

  test('只有腳本收到 SIGTERM 時刪掉暫存資料夾，並以 143 結束，不回報成功', { skip: NO_POSIX_SIGNALS }, () =>
    withCase(async (workspace) => {
      // 子程序沒收到訊號，腳本要等它載入完（1.5 秒）；之後不得以 0 結束。
      const result = await interruptDuringLoad(workspace, 1500, (child) => child.kill('SIGTERM'));
      assert.equal(result.signal, null, `腳本不得被訊號直接結束：${JSON.stringify(result)}`);
      assert.equal(result.status, 143, JSON.stringify(result));
      assert.match(result.stderr, /SIGTERM/);
    }));
});
