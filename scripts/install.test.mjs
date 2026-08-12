// CLI 安裝腳本的單元測試（cli-distribution spec「安裝腳本一行安裝對應平台 CLI」，
// design D3）。以子行程執行 scripts/install.sh。
//
// 平台偵測與下載都靠外部指令，因此測試在 PATH 前置假的 uname 與 curl——腳本本身
// 不必為了可測而開任何測試專用旗標，被驗的就是正式路徑。checksum 用真的
// shasum／sha256sum 對 fixture 實算，驗的是真正的比對行為而非樁。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts/install.sh');

const REPO = 'MomoChenisMe/speclink';
const TAG = 'v0.1.0';

/// 假 uname：依環境變數回答 -s／-m，讓五組平台矩陣都能在單一機器上驗。
const FAKE_UNAME = `#!/bin/sh
case "$1" in
  -s) echo "$FAKE_UNAME_S" ;;
  -m) echo "$FAKE_UNAME_M" ;;
  *) echo "$FAKE_UNAME_S" ;;
esac
`;

/// 假 curl：記錄每次呼叫，並以 fixture 目錄中與 URL 檔名同名的檔案作為回應。
/// 找不到對應 fixture 時以 22 結束（curl 對 HTTP 錯誤的慣用碼），讓腳本的
/// 失敗路徑也走得到。
const FAKE_CURL = `#!/bin/sh
echo "$@" >> "$FAKE_CURL_LOG"
out=""
url=""
while [ $# -gt 0 ]; do
  case "$1" in
    -o) out="$2"; shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
  esac
done
name=$(basename "$url")
src="$FAKE_FIXTURES/$name"
[ -f "$src" ] || exit 22
if [ -n "$out" ]; then cp "$src" "$out"; else cat "$src"; fi
`;

function sha256(buffer) {
  return createHash('sha256').update(buffer).digest('hex');
}

/// 佈置一次執行所需的沙盒：假指令目錄、fixture 目錄、安裝目錄與 curl 呼叫紀錄。
function sandbox(t) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'speclink-install-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));

  const binDir = path.join(dir, 'fakebin');
  const fixtures = path.join(dir, 'fixtures');
  const installDir = path.join(dir, 'install');
  mkdirSync(binDir);
  mkdirSync(fixtures);
  mkdirSync(installDir);

  for (const [name, body] of [['uname', FAKE_UNAME], ['curl', FAKE_CURL]]) {
    const file = path.join(binDir, name);
    writeFileSync(file, body);
    chmodSync(file, 0o755);
  }

  return { dir, binDir, fixtures, installDir, curlLog: path.join(dir, 'curl.log') };
}

/// 造出一份可安裝的壓縮檔 fixture 與對應的 SHA256SUMS.txt。
/// badChecksum 時故意寫入錯的 digest，用來驗「不符即中止且不落檔」。
function stageRelease(box, { target, tag = TAG, badChecksum = false } = {}) {
  const assetName = `speclink-${tag}-${target}.tar.gz`;
  const payloadDir = path.join(box.dir, 'payload');
  mkdirSync(payloadDir, { recursive: true });
  // 內容任意但可辨識——安裝後以此斷言確實是解出來的那一份。
  writeFileSync(path.join(payloadDir, 'speclink'), '#!/bin/sh\necho speclink-fixture\n');
  chmodSync(path.join(payloadDir, 'speclink'), 0o755);

  const archive = path.join(box.fixtures, assetName);
  const tarResult = spawnSync('tar', ['czf', archive, '-C', payloadDir, 'speclink'], { encoding: 'utf8' });
  assert.equal(tarResult.status, 0, `建立 fixture 壓縮檔失敗：${tarResult.stderr}`);

  const digest = badChecksum ? '0'.repeat(64) : sha256(readFileSync(archive));
  writeFileSync(path.join(box.fixtures, 'SHA256SUMS.txt'), `${digest}  ${assetName}\n`);
  writeFileSync(path.join(box.fixtures, 'latest'), JSON.stringify({ tag_name: tag }));

  return { assetName };
}

function runInstall(box, { unameS, unameM, args = [], env = {} } = {}) {
  const result = spawnSync('sh', [script, ...args], {
    encoding: 'utf8',
    env: {
      ...process.env,
      PATH: `${box.binDir}:${process.env.PATH}`,
      HOME: box.dir,
      FAKE_UNAME_S: unameS,
      FAKE_UNAME_M: unameM,
      FAKE_CURL_LOG: box.curlLog,
      FAKE_FIXTURES: box.fixtures,
      SPECLINK_INSTALL_REPO: REPO,
      ...env,
    },
  });
  const curlCalls = existsSync(box.curlLog)
    ? readFileSync(box.curlLog, 'utf8').split('\n').filter(Boolean)
    : [];
  return { ...result, curlCalls };
}

// --- 平台對映矩陣（dry-run，不碰網路） ---

const MATRIX = [
  { unameS: 'Darwin', unameM: 'arm64', target: 'aarch64-apple-darwin' },
  { unameS: 'Darwin', unameM: 'x86_64', target: 'x86_64-apple-darwin' },
  { unameS: 'Linux', unameM: 'x86_64', target: 'x86_64-unknown-linux-gnu' },
  { unameS: 'Linux', unameM: 'aarch64', target: 'aarch64-unknown-linux-gnu' },
];

for (const { unameS, unameM, target } of MATRIX) {
  test(`dry-run 對映 ${unameS}/${unameM} 為 ${target} 並組出 Release 資產 URL`, (t) => {
    const box = sandbox(t);

    const result = runInstall(box, {
      unameS,
      unameM,
      args: ['--dry-run'],
      env: { SPECLINK_INSTALL_VERSION: TAG },
    });

    assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
    assert.match(result.stdout, new RegExp(target), `輸出應含 target ${target}`);
    assert.match(
      result.stdout,
      new RegExp(`https://github\\.com/${REPO}/releases/download/${TAG}/speclink-${TAG}-${target}\\.tar\\.gz`),
      '輸出應含指向該 target 的 Release 資產 URL',
    );
  });
}

test('不支援的平台以非零結束並說明', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, { unameS: 'Linux', unameM: 'i686', args: ['--dry-run'] });

  assert.notEqual(result.status, 0, '不支援的架構應以非零結束');
  assert.match(result.stderr, /i686/, '錯誤訊息應點名偵測到的架構');
});

test('Windows 上以非零結束並導向 PowerShell 版腳本', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, { unameS: 'MINGW64_NT-10.0', unameM: 'x86_64', args: ['--dry-run'] });

  assert.notEqual(result.status, 0, 'Windows 應以非零結束');
  assert.match(result.stderr, /install\.ps1/, '錯誤訊息應導向 PowerShell 版腳本');
});

// --- dry-run 的兩條保證 ---

test('dry-run 不發出任何網路請求也不寫入檔案', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    args: ['--dry-run'],
    env: { SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
  assert.deepEqual(result.curlCalls, [], 'dry-run 不得呼叫 curl');
  assert.deepEqual(readdirSync(box.installDir), [], 'dry-run 不得寫入安裝目錄');
});

test('dry-run 未釘選版本時標示將於安裝時查詢，仍不呼叫 curl', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, { unameS: 'Linux', unameM: 'x86_64', args: ['--dry-run'] });

  assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
  assert.deepEqual(result.curlCalls, [], 'dry-run 不得呼叫 curl');
  assert.match(result.stdout, /latest/, '未釘選版本時輸出應標示為 latest');
});

// --- 環境變數覆寫 ---

test('SPECLINK_INSTALL_DIR 覆寫安裝目錄', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    args: ['--dry-run'],
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0);
  assert.match(result.stdout, new RegExp(box.installDir.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
});

test('未覆寫時安裝目錄預設為 ~/.local/bin', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    args: ['--dry-run'],
    env: { SPECLINK_INSTALL_VERSION: TAG },
  });

  assert.equal(result.status, 0);
  assert.match(result.stdout, /\.local\/bin/, '預設安裝目錄應為 ~/.local/bin');
});

test('釘選版本時不查詢 latest API', (t) => {
  const box = sandbox(t);
  stageRelease(box, { target: 'aarch64-apple-darwin' });

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  assert.equal(
    result.curlCalls.some((call) => call.includes('api.github.com')),
    false,
    '版本已釘選時不應查詢 latest API',
  );
});

// --- 實際安裝路徑 ---

test('安裝完成後 binary 落在安裝目錄且可執行', (t) => {
  const box = sandbox(t);
  stageRelease(box, { target: 'x86_64-unknown-linux-gnu' });

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  const installed = path.join(box.installDir, 'speclink');
  assert.ok(existsSync(installed), 'speclink 應存在於安裝目錄');
  const run = spawnSync(installed, { encoding: 'utf8' });
  assert.match(run.stdout, /speclink-fixture/, '安裝的應是壓縮檔中解出的那一份');
});

test('未釘選版本時經 latest API 解析出版本再下載', (t) => {
  const box = sandbox(t);
  stageRelease(box, { target: 'aarch64-apple-darwin' });

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    env: { SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  assert.ok(
    result.curlCalls.some((call) => call.includes('api.github.com')),
    '未釘選版本時應查詢 latest API',
  );
  assert.ok(existsSync(path.join(box.installDir, 'speclink')));
});

// --- checksum 驗證 ---

test('checksum 不符時以非零結束且安裝目錄不留任何檔案', (t) => {
  const box = sandbox(t);
  stageRelease(box, { target: 'x86_64-unknown-linux-gnu', badChecksum: true });

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.notEqual(result.status, 0, 'checksum 不符應以非零結束');
  assert.match(result.stderr, /checksum|校驗/i, '錯誤訊息應指出 checksum 不符');
  assert.deepEqual(readdirSync(box.installDir), [], 'checksum 不符時安裝目錄不得留下任何檔案');
});

test('SHA256SUMS.txt 缺少該資產條目時以非零結束', (t) => {
  const box = sandbox(t);
  stageRelease(box, { target: 'x86_64-unknown-linux-gnu' });
  // 覆寫成只含別的平台條目，模擬資產缺漏。
  writeFileSync(
    path.join(box.fixtures, 'SHA256SUMS.txt'),
    `${'a'.repeat(64)}  speclink-${TAG}-aarch64-apple-darwin.tar.gz\n`,
  );

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.notEqual(result.status, 0, '缺少條目應以非零結束');
  assert.deepEqual(readdirSync(box.installDir), [], '缺少條目時安裝目錄不得留下任何檔案');
});

// --- PATH 提示 ---

test('安裝目錄不在 PATH 時提示使用者', (t) => {
  const box = sandbox(t);
  stageRelease(box, { target: 'x86_64-unknown-linux-gnu' });

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  assert.match(
    `${result.stdout}${result.stderr}`,
    new RegExp('PATH'),
    '安裝目錄不在 PATH 時應提示',
  );
});

// --- PowerShell 版（install.ps1）---
//
// 契約與 install.sh 相同，只是平台偵測與解壓走 .NET API。無 pwsh 的環境自動跳過；
// Windows CI 的 runner 一定有，該面在那裡才真的跑得到。
const ps1 = path.join(root, 'scripts/install.ps1');
const pwsh = ['pwsh', 'powershell'].find(
  (candidate) => spawnSync(candidate, ['-NoProfile', '-Command', 'exit 0']).status === 0,
);

function runInstallPs1(box, { args = [], env = {} } = {}) {
  return spawnSync(pwsh, ['-NoProfile', '-File', ps1, ...args], {
    encoding: 'utf8',
    env: {
      ...process.env,
      SPECLINK_INSTALL_REPO: REPO,
      ...env,
    },
  });
}

test('install.ps1 的 dry-run 印出 Windows target 與 zip 資產 URL', { skip: !pwsh && '本機無 pwsh' }, (t) => {
  const box = sandbox(t);

  const result = runInstallPs1(box, {
    args: ['-DryRun'],
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
  assert.match(result.stdout, /x86_64-pc-windows-msvc/, '輸出應含 Windows target');
  assert.match(
    result.stdout,
    new RegExp(`https://github\\.com/${REPO}/releases/download/${TAG}/speclink-${TAG}-x86_64-pc-windows-msvc\\.zip`),
    '輸出應含 zip 資產 URL',
  );
});

test('install.ps1 的 dry-run 不寫入安裝目錄', { skip: !pwsh && '本機無 pwsh' }, (t) => {
  const box = sandbox(t);

  const result = runInstallPs1(box, {
    args: ['-DryRun'],
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0);
  assert.deepEqual(readdirSync(box.installDir), [], 'dry-run 不得寫入安裝目錄');
});

test('install.ps1 支援 SPECLINK_INSTALL_DIR 覆寫，未設時落在使用者層級目錄', { skip: !pwsh && '本機無 pwsh' }, (t) => {
  const box = sandbox(t);

  const overridden = runInstallPs1(box, {
    args: ['-DryRun'],
    env: { SPECLINK_INSTALL_VERSION: TAG, SPECLINK_INSTALL_DIR: box.installDir },
  });
  assert.match(overridden.stdout, new RegExp(box.installDir.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));

  const defaulted = runInstallPs1(box, {
    args: ['-DryRun'],
    env: { SPECLINK_INSTALL_VERSION: TAG },
  });
  assert.match(defaulted.stdout, /Speclink/, '未覆寫時應落在使用者層級的 Speclink 目錄');
});

test('install.ps1 未釘選版本時標示為 latest', { skip: !pwsh && '本機無 pwsh' }, (t) => {
  const box = sandbox(t);

  const result = runInstallPs1(box, { args: ['-DryRun'] });

  assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
  assert.match(result.stdout, /latest/, '未釘選版本時輸出應標示為 latest');
});
