// CLI 安裝腳本的單元測試（cli-distribution spec「安裝腳本一行安裝對應平台 CLI」，
// design D3／release-assets-trim D6：binary 來源改為 npm registry 的平台子套件 tgz）。
// 以子行程執行 scripts/install.sh。
//
// 平台偵測與下載都靠外部指令，因此測試在 PATH 前置假的 uname 與 curl——腳本本身
// 不必為了可測而開任何測試專用旗標，被驗的就是正式路徑。integrity 用真的 openssl
// 對 fixture 實算，驗的是真正的比對行為而非樁。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts/install.sh');

const VERSION = '0.5.0';
const REGISTRY = 'https://registry.npmjs.org';
const tgzUrl = (platform, version, registry = REGISTRY) =>
  `${registry}/@speclink/cli-${platform}/-/cli-${platform}-${version}.tgz`;

/// 假 uname：依環境變數回答 -s／-m，讓五組平台矩陣都能在單一機器上驗。
const FAKE_UNAME = `#!/bin/sh
case "$1" in
  -s) echo "$FAKE_UNAME_S" ;;
  -m) echo "$FAKE_UNAME_M" ;;
  *) echo "$FAKE_UNAME_S" ;;
esac
`;

/// 假 curl：記錄每次呼叫，並以 fixture 目錄中與 URL 最後一段同名的檔案作為回應
/// （registry 的三種請求：.../cli/latest、.../cli-<平台>/<版本>、.../-/<tgz>）。
/// 找不到對應 fixture 時以 22 結束（curl 對 HTTP 錯誤的慣用碼），讓腳本的失敗路徑也走得到。
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

function integrityOf(buffer) {
  return `sha512-${createHash('sha512').update(buffer).digest('base64')}`;
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

/// 造出 registry 的三份回應：主套件 latest、平台子套件的版本清單（含 dist.integrity）、
/// 平台子套件的 tgz（npm 佈局：package/speclink）。badIntegrity 時故意寫錯 integrity，
/// noIntegrity 時版本清單缺 dist.integrity。
function stageRegistry(box, { platform, version = VERSION, badIntegrity = false, noIntegrity = false } = {}) {
  const payloadDir = path.join(box.dir, 'payload', 'package');
  mkdirSync(payloadDir, { recursive: true });
  // 內容可辨識——安裝後以此斷言確實是解出來的那一份；--version 回該版，對齊 spec
  // 「安裝完成後版本可驗」的 THEN（真 binary 的版本比對留給發版後的手動驗證）。
  writeFileSync(
    path.join(payloadDir, 'speclink'),
    `#!/bin/sh\ncase "$1" in --version) echo "speclink ${version}" ;; *) echo speclink-fixture ;; esac\n`,
  );
  chmodSync(path.join(payloadDir, 'speclink'), 0o755);

  const tgzName = `cli-${platform}-${version}.tgz`;
  const archive = path.join(box.fixtures, tgzName);
  const tarResult = spawnSync('tar', ['czf', archive, '-C', path.dirname(payloadDir), 'package'], { encoding: 'utf8' });
  assert.equal(tarResult.status, 0, `建立 fixture tgz 失敗：${tarResult.stderr}`);

  const integrity = badIntegrity ? `sha512-${'A'.repeat(86)}==` : integrityOf(readFileSync(archive));
  const manifest = {
    name: `@speclink/cli-${platform}`,
    version,
    dist: noIntegrity ? { tarball: tgzUrl(platform, version) } : { integrity, tarball: tgzUrl(platform, version) },
  };
  writeFileSync(path.join(box.fixtures, version), JSON.stringify(manifest));
  writeFileSync(path.join(box.fixtures, 'latest'), JSON.stringify({ name: '@speclink/cli', version }));

  return { tgzName };
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
      ...env,
    },
  });
  const curlCalls = existsSync(box.curlLog)
    ? readFileSync(box.curlLog, 'utf8').split('\n').filter(Boolean)
    : [];
  return { ...result, curlCalls };
}

// install.sh 只出貨給 macOS 與 Linux；Windows 走 npm 或桌面安裝器。Git Bash 下以假的
// curl／uname 驗 sh 版並不對應任何出貨路徑（Windows 不會解析無副檔名的假指令），
// 故 Windows 上整組跳過。
const isWindows = process.platform === 'win32';
const shTest = isWindows ? test.skip : test;

// --- 平台對映矩陣（dry-run，不碰網路） ---

const MATRIX = [
  { unameS: 'Darwin', unameM: 'arm64', platform: 'darwin-arm64' },
  { unameS: 'Darwin', unameM: 'x86_64', platform: 'darwin-x64' },
  { unameS: 'Linux', unameM: 'x86_64', platform: 'linux-x64' },
  { unameS: 'Linux', unameM: 'aarch64', platform: 'linux-arm64' },
];

for (const { unameS, unameM, platform } of MATRIX) {
  shTest(`dry-run 對映 ${unameS}/${unameM} 為 ${platform} 並組出 registry 的 tgz URL`, (t) => {
    const box = sandbox(t);

    const result = runInstall(box, {
      unameS,
      unameM,
      args: ['--dry-run'],
      env: { SPECLINK_INSTALL_VERSION: VERSION },
    });

    assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
    assert.match(result.stdout, new RegExp(platform), `輸出應含平台 ${platform}`);
    assert.ok(result.stdout.includes(tgzUrl(platform, VERSION)), '輸出應含指向該平台子套件 tgz 的 registry URL');
    assert.deepEqual(result.curlCalls, [], 'dry-run 不得呼叫 curl');
  });
}

shTest('不支援的平台以非零結束並說明', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, { unameS: 'Linux', unameM: 'i686', args: ['--dry-run'] });

  assert.notEqual(result.status, 0, '不支援的架構應以非零結束');
  assert.match(result.stderr, /i686/, '錯誤訊息應點名偵測到的架構');
});

shTest('Windows 上以非零結束並導向 npm 與桌面安裝器，不再提 PowerShell 腳本', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, { unameS: 'MINGW64_NT-10.0', unameM: 'x86_64', args: ['--dry-run'] });

  assert.notEqual(result.status, 0, 'Windows 應以非零結束');
  assert.match(result.stderr, /npm i -g @speclink\/cli/, '錯誤訊息應導向 npm 全域安裝');
  assert.match(result.stderr, /setup\.exe/, '錯誤訊息應導向桌面安裝器');
  assert.doesNotMatch(result.stderr, /install\.ps1/, 'PowerShell 腳本已退役，不得再導向它');
});

// --- dry-run 的兩條保證 ---

shTest('dry-run 不發出任何網路請求也不寫入檔案', (t) => {
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

shTest('dry-run 未釘選版本時標示將於安裝時查詢 registry，仍不呼叫 curl', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, { unameS: 'Linux', unameM: 'x86_64', args: ['--dry-run'] });

  assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
  assert.deepEqual(result.curlCalls, [], 'dry-run 不得呼叫 curl');
  assert.match(result.stdout, /latest/, '未釘選版本時輸出應標示為 latest');
});

// --- 環境變數覆寫 ---

shTest('SPECLINK_INSTALL_VERSION 帶或不帶 v 前綴都對映到同一個 tgz URL', (t) => {
  const box = sandbox(t);
  const outputs = ['v0.5.0', '0.5.0'].map((pin) => {
    const result = runInstall(box, {
      unameS: 'Darwin',
      unameM: 'arm64',
      args: ['--dry-run'],
      env: { SPECLINK_INSTALL_VERSION: pin },
    });
    assert.equal(result.status, 0, `dry-run 應成功結束（${pin}）\nstderr: ${result.stderr}`);
    return result.stdout;
  });
  assert.equal(outputs[0], outputs[1], '兩種寫法的 dry-run 輸出應完全相同');
  assert.ok(outputs[0].includes(tgzUrl('darwin-arm64', VERSION)), 'URL 的版本段應為去 v 前綴的 0.5.0');
});

shTest('SPECLINK_INSTALL_REGISTRY 覆寫 registry 位址', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'aarch64',
    args: ['--dry-run'],
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_REGISTRY: 'https://npm.example.test' },
  });

  assert.equal(result.status, 0, `dry-run 應成功結束\nstderr: ${result.stderr}`);
  assert.ok(result.stdout.includes(tgzUrl('linux-arm64', VERSION, 'https://npm.example.test')), 'URL 應以覆寫的 registry 為底');
  assert.ok(!result.stdout.includes('registry.npmjs.org'), '覆寫後不得殘留預設 registry');
});

shTest('SPECLINK_INSTALL_DIR 覆寫安裝目錄', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    args: ['--dry-run'],
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0);
  assert.match(result.stdout, new RegExp(box.installDir.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
});

shTest('未覆寫時安裝目錄預設為 ~/.local/bin', (t) => {
  const box = sandbox(t);

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    args: ['--dry-run'],
    env: { SPECLINK_INSTALL_VERSION: VERSION },
  });

  assert.equal(result.status, 0);
  assert.match(result.stdout, /\.local\/bin/, '預設安裝目錄應為 ~/.local/bin');
});

shTest('釘選版本時不查詢 latest', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'darwin-arm64' });

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  assert.equal(
    result.curlCalls.some((call) => call.includes('/@speclink/cli/latest')),
    false,
    '版本已釘選時不應查詢 latest',
  );
});

// --- 實際安裝路徑 ---

shTest('安裝完成後 binary 落在安裝目錄且可執行', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64' });

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  const installed = path.join(box.installDir, 'speclink');
  assert.ok(existsSync(installed), 'speclink 應存在於安裝目錄');
  assert.deepEqual(readdirSync(box.installDir), ['speclink'], '安裝目錄只該多出 speclink 一個檔');
  const run = spawnSync(installed, { encoding: 'utf8' });
  assert.match(run.stdout, /speclink-fixture/, '安裝的應是 tgz 中 package/speclink 解出的那一份');
  const version = spawnSync(installed, ['--version'], { encoding: 'utf8' });
  assert.match(version.stdout, new RegExp(VERSION.replace(/\./g, '\\.')), 'speclink --version 應印出安裝的版本');
});

shTest('未釘選版本時經 registry 的 latest 解析出版本再下載', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'darwin-arm64' });

  const result = runInstall(box, {
    unameS: 'Darwin',
    unameM: 'arm64',
    env: { SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  assert.ok(
    result.curlCalls.some((call) => call.includes('/@speclink/cli/latest')),
    '未釘選版本時應查詢 registry 的 latest',
  );
  assert.ok(result.curlCalls.some((call) => call.includes(tgzUrl('darwin-arm64', VERSION))), '應下載解析出版本的 tgz');
  assert.ok(existsSync(path.join(box.installDir, 'speclink')));
});

// --- integrity 驗證 ---

shTest('integrity 不符時以非零結束且安裝目錄不留任何檔案', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64', badIntegrity: true });

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.notEqual(result.status, 0, 'integrity 不符應以非零結束');
  assert.match(result.stderr, /integrity|校驗/i, '錯誤訊息應指出 integrity 不符');
  assert.deepEqual(readdirSync(box.installDir), [], 'integrity 不符時安裝目錄不得留下任何檔案');
});

shTest('registry 的版本清單缺 integrity 時以非零結束', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64', noIntegrity: true });

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.notEqual(result.status, 0, '缺 integrity 應以非零結束');
  assert.match(result.stderr, /integrity/i, '錯誤訊息應指出缺少 integrity');
  assert.deepEqual(readdirSync(box.installDir), [], '缺 integrity 時安裝目錄不得留下任何檔案');
});

// --- 參數面的尖角（sharp-edges 稽核） ---

shTest('SPECLINK_INSTALL_VERSION 不合 X.Y.Z 形即非零結束，不組 URL 也不連網', (t) => {
  const box = sandbox(t);
  for (const bad of ['latest', '1.2', '../../etc', 'v0.5.0/../x', '0.5.0;rm -rf /']) {
    const result = runInstall(box, {
      unameS: 'Linux',
      unameM: 'x86_64',
      env: { SPECLINK_INSTALL_VERSION: bad, SPECLINK_INSTALL_DIR: box.installDir },
    });
    assert.notEqual(result.status, 0, `版本「${bad}」不應被接受`);
    assert.match(result.stderr, /X\.Y\.Z/, `錯誤訊息應說明版本格式（${bad}）`);
    assert.deepEqual(result.curlCalls, [], `不合形的版本不得連網（${bad}）`);
  }
});

shTest('SPECLINK_INSTALL_REGISTRY 非 https 即非零結束（integrity 也來自同一個 registry，明文等於沒驗）', (t) => {
  const box = sandbox(t);
  for (const bad of ['http://registry.npmjs.org', 'ftp://x', 'registry.npmjs.org']) {
    const result = runInstall(box, {
      unameS: 'Darwin',
      unameM: 'arm64',
      args: ['--dry-run'],
      env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_REGISTRY: bad },
    });
    assert.notEqual(result.status, 0, `registry「${bad}」不應被接受`);
    assert.match(result.stderr, /https/, '錯誤訊息應說明只接受 https');
  }
});

shTest('安裝目錄已有指向別處的 symlink 時，換掉連結本身而不寫穿到目標', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64' });
  // 模擬桌面 app 留下的 symlink：~/.local/bin/speclink → app bundle 內的 CLI。
  const bundled = path.join(box.dir, 'bundle-cli');
  writeFileSync(bundled, '#!/bin/sh\necho bundled\n');
  chmodSync(bundled, 0o755);
  symlinkSync(bundled, path.join(box.installDir, 'speclink'));

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.equal(result.status, 0, `安裝應成功\nstderr: ${result.stderr}`);
  const installed = path.join(box.installDir, 'speclink');
  assert.equal(lstatSync(installed).isSymbolicLink(), false, '安裝後應是一般檔，不再是 symlink');
  assert.match(spawnSync(installed, { encoding: 'utf8' }).stdout, /speclink-fixture/);
  assert.equal(readFileSync(bundled, 'utf8'), '#!/bin/sh\necho bundled\n', 'symlink 原本指到的檔不得被改寫');
  assert.deepEqual(readdirSync(box.installDir), ['speclink'], '不得留下暫存檔');
});

// --- 失敗路徑不留殘檔 ---

/// 只含指定工具的 PATH 目錄：把當前 PATH 找得到的工具以 symlink 收進來，漏掉的那個
/// 就是要模擬「機器上沒裝」的工具。
function toolboxWithout(box, missing) {
  const toolbox = path.join(box.dir, 'toolbox');
  mkdirSync(toolbox);
  const tools = ['sh', 'grep', 'head', 'sed', 'mktemp', 'rm', 'tar', 'mkdir', 'cp', 'chmod', 'mv', 'basename', 'cat'];
  for (const tool of tools.filter((name) => name !== missing)) {
    const found = spawnSync('sh', ['-c', `command -v ${tool}`], { encoding: 'utf8' }).stdout.trim();
    assert.ok(found, `測試機缺 ${tool}`);
    symlinkSync(found, path.join(toolbox, tool));
  }
  return toolbox;
}

shTest('找不到 openssl 時以非零結束、不下載也不寫入安裝目錄（integrity 無法驗就不裝）', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64' });
  const toolbox = toolboxWithout(box, 'openssl');

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir, PATH: `${box.binDir}:${toolbox}` },
  });

  assert.notEqual(result.status, 0, '缺 openssl 應以非零結束');
  assert.match(result.stderr, /openssl/, '錯誤訊息應點名 openssl');
  assert.deepEqual(result.curlCalls, [], '缺 openssl 時不該先下載');
  assert.deepEqual(readdirSync(box.installDir), [], '缺 openssl 時安裝目錄不得留下任何檔案');
});

shTest('落檔途中失敗（chmod 非零）時暫存名的檔一併清掉，安裝目錄不留隱藏檔', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64' });
  // 假 chmod 一律失敗：cp 到暫存名之後、mv 換名之前的那一步。
  const fakeChmod = path.join(box.binDir, 'chmod');
  writeFileSync(fakeChmod, '#!/bin/sh\nexit 1\n');
  chmodSync(fakeChmod, 0o755);

  const result = runInstall(box, {
    unameS: 'Linux',
    unameM: 'x86_64',
    env: { SPECLINK_INSTALL_VERSION: VERSION, SPECLINK_INSTALL_DIR: box.installDir },
  });

  assert.notEqual(result.status, 0, 'chmod 失敗應以非零結束');
  assert.deepEqual(readdirSync(box.installDir), [], '失敗後安裝目錄不得留下 .speclink.tmp.* 暫存檔');
});

// --- PATH 提示 ---

shTest('安裝目錄不在 PATH 時提示使用者', (t) => {
  const box = sandbox(t);
  stageRegistry(box, { platform: 'linux-x64' });

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
