// @speclink/cli 主套件的 shim 與 postinstall（cli-distribution spec「CLI 以 npm 套件發布」；
// release-assets-trim 設計 D4）。純函式（平台對映、binary 定位、置換決策）直接匯入測；
// shim 的 spawn 與 exit code 轉發以子行程對假 binary 驗——只在非 Windows 跑（假 binary
// 是 sh 腳本，Windows 的 shim 路徑本來就走 spawn，不置換）。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, statSync, symlinkSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const pkgSource = path.join(root, 'packages/cli-npm');
const { platformPackage, binaryName, resolveBinary, supportedPlatforms } = await import(
  pathToFileURL(path.join(pkgSource, 'platform.mjs')).href
);
const { replaceShim } = await import(pathToFileURL(path.join(pkgSource, 'postinstall.mjs')).href);

const isWindows = process.platform === 'win32';
const posixTest = isWindows ? test.skip : test;

// --- 平台對映 ---

test('os/cpu 對映到五個平台子套件，之外的平台回 null', () => {
  const matrix = [
    ['darwin', 'arm64', '@speclink/cli-darwin-arm64'],
    ['darwin', 'x64', '@speclink/cli-darwin-x64'],
    ['linux', 'x64', '@speclink/cli-linux-x64'],
    ['linux', 'arm64', '@speclink/cli-linux-arm64'],
    ['win32', 'x64', '@speclink/cli-win32-x64'],
    ['freebsd', 'x64', null],
    ['win32', 'arm64', null],
  ];
  for (const [platform, arch, expected] of matrix) {
    assert.equal(platformPackage(platform, arch), expected, `${platform}/${arch}`);
  }
  assert.deepEqual(supportedPlatforms(), ['darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64', 'win32-x64']);
  assert.equal(binaryName('win32'), 'speclink.exe');
  assert.equal(binaryName('linux'), 'speclink');
});

// --- 沙盒：模擬 npm i -g 之後的目錄佈局 ---
//
//   <sandbox>/node_modules/@speclink/cli/{package.json,bin/speclink,postinstall.mjs,platform.mjs}
//   <sandbox>/node_modules/@speclink/cli-<os>-<cpu>/{package.json,speclink[.exe]}
//
// shim 與 postinstall 以自身位置往上找 node_modules 解析子套件，與真實安裝同一條路。
function sandbox(t, { withPlatformPackage = true, platform = process.platform, arch = process.arch, exitCode = 0, signal = null } = {}) {
  // realpath：macOS 的 os.tmpdir() 經 /var → /private/var symlink，require.resolve 回的是真實路徑。
  const dir = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'speclink-cli-npm-')));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const mainDir = path.join(dir, 'node_modules/@speclink/cli');
  mkdirSync(path.join(mainDir, 'bin'), { recursive: true });
  for (const file of ['package.json', 'bin/speclink', 'postinstall.mjs', 'platform.mjs']) {
    cpSync(path.join(pkgSource, file), path.join(mainDir, file));
  }
  chmodSync(path.join(mainDir, 'bin/speclink'), 0o755);

  let binary = null;
  if (withPlatformPackage) {
    const subDir = path.join(dir, 'node_modules', platformPackage(platform, arch));
    mkdirSync(subDir, { recursive: true });
    writeFileSync(path.join(subDir, 'package.json'), JSON.stringify({ name: platformPackage(platform, arch), version: '0.5.0' }));
    binary = path.join(subDir, binaryName(platform));
    // 假 binary：印出收到的每個參數（一行一個），以指定碼結束；指定 signal 時改對自己送該訊號。
    const ending = signal ? `kill -${signal} $$` : `exit ${exitCode}`;
    writeFileSync(binary, `#!/bin/sh\nfor a in "$@"; do echo "arg:$a"; done\n${ending}\n`);
    chmodSync(binary, 0o755);
  }
  return { dir, mainDir, shim: path.join(mainDir, 'bin/speclink'), binary };
}

// --- binary 定位 ---

test('resolveBinary 自主套件位置解析出對應子套件內的 binary；未安裝或不支援回 null', (t) => {
  const box = sandbox(t);
  const fromUrl = pathToFileURL(box.shim).href;
  assert.equal(resolveBinary(fromUrl, process.platform, process.arch), box.binary);
  assert.equal(resolveBinary(fromUrl, 'freebsd', 'x64'), null, '不支援的平台回 null');

  const empty = sandbox(t, { withPlatformPackage: false });
  assert.equal(resolveBinary(pathToFileURL(empty.shim).href, process.platform, process.arch), null, '子套件未安裝回 null');
});

// --- shim：spawn、參數透傳、exit code 轉發 ---

posixTest('shim 以 spawn 執行平台 binary、全部參數原樣透傳、exit 0 轉發', (t) => {
  const box = sandbox(t);
  const result = spawnSync(process.execPath, [box.shim, 'status', '--json', '--', '--weird', ''], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, 'arg:status\narg:--json\narg:--\narg:--weird\narg:\n');
});

posixTest('shim 把 binary 的非零 exit code 原樣帶回', (t) => {
  const box = sandbox(t, { exitCode: 3 });
  const result = spawnSync(process.execPath, [box.shim, 'validate'], { encoding: 'utf8' });
  assert.equal(result.status, 3);
});

posixTest('binary 被訊號收束時 shim 對自己送同一個訊號：呼叫端看到的是 SIGTERM，不是 exit code', (t) => {
  const box = sandbox(t, { signal: 'TERM' });
  const result = spawnSync(process.execPath, [box.shim, 'serve'], { encoding: 'utf8' });
  assert.equal(result.signal, 'SIGTERM', 'shim 應以同一個訊號終止');
  assert.equal(result.status, null);
  assert.equal(result.stdout, 'arg:serve\n', '訊號之前的輸出仍透傳');
});

posixTest('找不到對應子套件時 shim 於 stderr 列出支援組合並以 exit 1 結束', (t) => {
  const box = sandbox(t, { withPlatformPackage: false });
  const result = spawnSync(process.execPath, [box.shim, '--version'], { encoding: 'utf8' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, new RegExp(`${process.platform}[-/]${process.arch}`), 'stderr 點名目前的 os/cpu');
  for (const key of supportedPlatforms()) {
    assert.ok(result.stderr.includes(key), `stderr 應列出支援組合 ${key}`);
  }
  assert.equal(result.stdout, '');
});

// --- postinstall：原生 binary 原地置換 shim ---

posixTest('darwin／linux 找到 binary 時把 bin/speclink 原地換成該 binary（內容相同、0755）', (t) => {
  const box = sandbox(t);
  const outcome = replaceShim({ platform: process.platform, arch: process.arch, shimPath: box.shim });
  assert.deepEqual(outcome, { replaced: true, binary: box.binary });
  assert.equal(readFileSync(box.shim, 'utf8'), readFileSync(box.binary, 'utf8'), '置換後 shim 檔內容應等於原生 binary');
  assert.equal(statSync(box.shim).mode & 0o777, 0o755);
  // 置換後就是原生檔本身，直接執行仍轉發 exit code。
  const result = spawnSync(box.shim, ['x'], { encoding: 'utf8' });
  assert.equal(result.stdout, 'arg:x\n');
});

test('win32 不置換：npm 的 .cmd 殼以 node 執行 bin，換成原生檔會壞', (t) => {
  const box = sandbox(t, { platform: 'win32', arch: 'x64' });
  const before = readFileSync(box.shim, 'utf8');
  const outcome = replaceShim({ platform: 'win32', arch: 'x64', shimPath: box.shim });
  assert.deepEqual(outcome, { replaced: false, reason: 'win32' });
  assert.equal(readFileSync(box.shim, 'utf8'), before);
});

test('找不到平台 binary 時保留 shim 且不視為錯誤（--ignore-scripts 或子套件缺席的退路）', (t) => {
  const box = sandbox(t, { withPlatformPackage: false });
  const before = readFileSync(box.shim, 'utf8');
  // 平台固定給 linux/x64：win32 在找 binary 之前就先以 reason win32 保留 shim（上一條測試），
  // 用 host 平台會讓這條在 Windows runner 上驗到的是另一個分支。
  const outcome = replaceShim({ platform: 'linux', arch: 'x64', shimPath: box.shim });
  assert.deepEqual(outcome, { replaced: false, reason: 'binary-missing' });
  assert.equal(readFileSync(box.shim, 'utf8'), before);

  // 以子行程跑 postinstall 本體：exit 0、shim 未動。
  const result = spawnSync(process.execPath, [path.join(box.mainDir, 'postinstall.mjs')], { encoding: 'utf8', cwd: box.mainDir });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(box.shim, 'utf8'), before);
  assert.equal(existsSync(path.join(box.mainDir, 'bin/speclink')), true);
});

posixTest('postinstall 本體經 symlink 路徑被呼叫時仍執行置換（npm prefix 走 symlink 的機器）', (t) => {
  const box = sandbox(t);
  // 套件目錄透過一個 symlink 抵達：argv[1] 是 symlink 路徑，import.meta.url 是 node 解析後的
  // 實體路徑——入口判定沒 realpath 就會靜默不跑 main，shim 留著、每次呼叫多付 Node 啟動。
  const link = path.join(box.dir, 'linked-cli');
  symlinkSync(box.mainDir, link);
  const result = spawnSync(process.execPath, [path.join(link, 'postinstall.mjs')], { encoding: 'utf8', cwd: link });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(box.shim, 'utf8'), readFileSync(box.binary, 'utf8'), '經 symlink 呼叫也要完成置換');
  assert.equal(statSync(box.shim).mode & 0o777, 0o755);
});
