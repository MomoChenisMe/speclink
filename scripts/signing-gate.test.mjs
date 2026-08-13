// OS 簽章 secrets 閘門的單元測試（desktop-release spec「OS 程式碼簽章為可插鑰匙開關」，
// design D1）。以子行程執行 scripts/signing-gate.mjs，模擬三種 secrets 組合：
// 全無（跳過簽章、綠燈）、全有（啟用對應路徑）、缺一（fail-closed 並點名缺項）。
//
// 閘門只讀「是否非空」，永不輸出 secret 值——洩漏面由專門案例守住。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts/signing-gate.mjs');

/// 回傳 needle 在 haystack 的起始索引，找不到時以帶標籤的訊息 fail——把「缺片段」
/// 與「順序錯」兩種失敗分開，避免 indexOf 回 -1 讓順序斷言假性通過。
function requireIndex(haystack, needle, label) {
  const i = haystack.indexOf(needle);
  assert.notEqual(i, -1, `${label}: 找不到片段 ${JSON.stringify(needle)}`);
  return i;
}

const MACOS_NAMES = [
  'APPLE_CERTIFICATE',
  'APPLE_CERTIFICATE_PASSWORD',
  'APPLE_SIGNING_IDENTITY',
  'APPLE_ID',
  'APPLE_PASSWORD',
  'APPLE_TEAM_ID',
];
// SIGNPATH_*：裁定不採用後不再受管，保留清單只為抹除環境與「被忽略」斷言。
const SIGNPATH_NAMES = [
  'SIGNPATH_API_TOKEN',
  'SIGNPATH_ORGANIZATION_ID',
  'SIGNPATH_PROJECT_SLUG',
  'SIGNPATH_POLICY_SLUG',
];
const WINDOWS_CERT_NAMES = ['WINDOWS_CERTIFICATE', 'WINDOWS_CERTIFICATE_PASSWORD'];
const ALL_NAMES = [...MACOS_NAMES, ...SIGNPATH_NAMES, ...WINDOWS_CERT_NAMES];

// 可辨識的假值：一旦出現在 stdout／stderr／GITHUB_ENV，即為洩漏。
const SECRET_VALUE = 'ZmFrZS1zZWNyZXQtdmFsdWU=';

/// 把一組 secret 名稱全部填成假值。
function filled(names) {
  return Object.fromEntries(names.map((name) => [name, SECRET_VALUE]));
}

/// 以乾淨環境執行閘門：先抹掉全部受管名稱，再套用本案例的 overrides。
/// 回傳結果附上 GITHUB_ENV 檔的內容（未寫入時為 null），供決策斷言使用。
function runGate(overrides = {}, t) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'signing-gate-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const envFile = path.join(dir, 'github_env');

  const env = { ...process.env };
  for (const name of ALL_NAMES) delete env[name];
  env.GITHUB_ENV = envFile;
  Object.assign(env, overrides);

  const result = spawnSync(process.execPath, [script], { encoding: 'utf8', env });
  const githubEnv = existsSync(envFile) ? readFileSync(envFile, 'utf8') : null;
  return { ...result, githubEnv };
}

/// 從 GITHUB_ENV 內容取出某個鍵的值（`KEY=value` 逐行格式）。
function envValue(content, key) {
  const line = (content ?? '').split('\n').find((row) => row.startsWith(`${key}=`));
  return line === undefined ? undefined : line.slice(key.length + 1);
}

// --- 全無：跳過簽章、workflow 照常全綠 ---

test('三組 secrets 全無時以零結束，兩平台決策皆為 none', (t) => {
  const result = runGate({}, t);

  assert.equal(result.status, 0, `全無 secrets 應成功結束\nstderr: ${result.stderr}`);
  assert.equal(envValue(result.githubEnv, 'SPECLINK_MACOS_SIGNING'), 'none');
  assert.equal(envValue(result.githubEnv, 'SPECLINK_WINDOWS_SIGNING'), 'none');
});

// --- 全有：啟用對應簽章路徑 ---

test('macOS 六項齊備時決策為 full（憑證半組＋公證半組）', (t) => {
  const result = runGate(filled(MACOS_NAMES), t);

  assert.equal(result.status, 0, `六項齊備應成功結束\nstderr: ${result.stderr}`);
  assert.equal(envValue(result.githubEnv, 'SPECLINK_MACOS_SIGNING'), 'full');
});

// SignPath 裁定不採用（design D2）：SIGNPATH_* 不再是受管簽章組——存在、部分
// 存在都不影響決策，也不觸發 fail-closed。
test('SIGNPATH_* 即使齊備也不影響決策（裁定不採用）', (t) => {
  const result = runGate(filled(SIGNPATH_NAMES), t);

  assert.equal(result.status, 0, `SIGNPATH_* 不應觸發任何閘門行為\nstderr: ${result.stderr}`);
  assert.equal(envValue(result.githubEnv, 'SPECLINK_WINDOWS_SIGNING'), 'none');
});

test('SIGNPATH_* 與本機憑證並存時決策仍為 certificate', (t) => {
  const result = runGate({ ...filled(SIGNPATH_NAMES), ...filled(WINDOWS_CERT_NAMES) }, t);

  assert.equal(result.status, 0, `stderr: ${result.stderr}`);
  assert.equal(envValue(result.githubEnv, 'SPECLINK_WINDOWS_SIGNING'), 'certificate');
});

test('僅本機憑證兩項齊備時 Windows 決策為 certificate', (t) => {
  const result = runGate(filled(WINDOWS_CERT_NAMES), t);

  assert.equal(result.status, 0, `兩項齊備應成功結束\nstderr: ${result.stderr}`);
  assert.equal(envValue(result.githubEnv, 'SPECLINK_WINDOWS_SIGNING'), 'certificate');
});

// --- 缺一：fail-closed 並點名缺項 ---

test('macOS 憑證半組齊而公證半組缺一項時非零結束並點名該項', (t) => {
  const overrides = filled(MACOS_NAMES);
  delete overrides.APPLE_TEAM_ID;

  const result = runGate(overrides, t);

  assert.notEqual(result.status, 0, 'macOS 簽章組部分存在時應非零結束');
  assert.match(result.stderr, /APPLE_TEAM_ID/, '錯誤訊息應點名缺少的 APPLE_TEAM_ID');
});

test('macOS 只設公證半組時非零結束並列出憑證半組三項缺項', (t) => {
  const result = runGate(
    filled(['APPLE_ID', 'APPLE_PASSWORD', 'APPLE_TEAM_ID']),
    t,
  );

  assert.notEqual(result.status, 0, 'macOS 簽章組部分存在時應非零結束');
  for (const name of ['APPLE_CERTIFICATE', 'APPLE_CERTIFICATE_PASSWORD', 'APPLE_SIGNING_IDENTITY']) {
    assert.match(result.stderr, new RegExp(name), `錯誤訊息應點名缺少的 ${name}`);
  }
});

test('本機憑證只設憑證而缺密碼時非零結束並點名密碼', (t) => {
  const result = runGate(filled(['WINDOWS_CERTIFICATE']), t);

  assert.notEqual(result.status, 0, 'Windows 憑證組部分存在時應非零結束');
  assert.match(
    result.stderr,
    /WINDOWS_CERTIFICATE_PASSWORD/,
    '錯誤訊息應點名缺少的 WINDOWS_CERTIFICATE_PASSWORD',
  );
});

test('只填空白字元視為未設定，與全無同義', (t) => {
  const result = runGate({ APPLE_CERTIFICATE: '   ' }, t);

  assert.equal(result.status, 0, '空白值應視為未設定而非部分存在');
  assert.equal(envValue(result.githubEnv, 'SPECLINK_MACOS_SIGNING'), 'none');
});

// --- fail-closed 的兩個附帶保證 ---

test('缺項失敗時不寫入任何決策，避免下游沿用半套設定', (t) => {
  const overrides = filled(MACOS_NAMES);
  delete overrides.APPLE_TEAM_ID;

  const result = runGate(overrides, t);

  assert.notEqual(result.status, 0);
  assert.equal(
    envValue(result.githubEnv, 'SPECLINK_MACOS_SIGNING'),
    undefined,
    '失敗時不得寫出 macOS 決策',
  );
  assert.equal(
    envValue(result.githubEnv, 'SPECLINK_WINDOWS_SIGNING'),
    undefined,
    '失敗時不得寫出 Windows 決策',
  );
});

test('輸出只含 secret 名稱，任何路徑都不得帶出 secret 值', (t) => {
  const overrides = filled(MACOS_NAMES);
  delete overrides.APPLE_TEAM_ID;

  const result = runGate(overrides, t);

  assert.doesNotMatch(result.stderr, new RegExp(SECRET_VALUE), 'stderr 洩漏了 secret 值');
  assert.doesNotMatch(result.stdout, new RegExp(SECRET_VALUE), 'stdout 洩漏了 secret 值');
  assert.doesNotMatch(result.githubEnv ?? '', new RegExp(SECRET_VALUE), 'GITHUB_ENV 洩漏了 secret 值');
});

// --- release.yml 接線的靜態契約 ---
//
// 閘門本身綠燈不代表 workflow 有用它。以下斷言守住接線：閘門排在昂貴的桌面建置
// 之前、簽章步驟改以閘門決策為條件、公證三項確實注入建置環境。

test('release.yml 的 desktop job 在 Tauri 建置前執行簽章閘門', () => {
  const release = readFileSync(path.join(root, '.github/workflows/release.yml'), 'utf8');

  const gate = requireIndex(release, 'node scripts/signing-gate.mjs', 'release.yml');
  const build = requireIndex(release, 'tauri -- build', 'release.yml');
  assert.ok(gate < build, 'release.yml：簽章閘門必須排在 Tauri 建置之前');
});

test('release.yml 的簽章步驟以閘門決策為條件，不再自行推導 secrets 是否存在', () => {
  const release = readFileSync(path.join(root, '.github/workflows/release.yml'), 'utf8');

  assert.match(
    release,
    /env\.SPECLINK_MACOS_SIGNING == 'full'/,
    'release.yml：macOS 簽章步驟應以閘門的 SPECLINK_MACOS_SIGNING 決策為條件',
  );
  assert.match(
    release,
    /env\.SPECLINK_WINDOWS_SIGNING == 'certificate'/,
    'release.yml：Windows 憑證簽章步驟應以閘門的 SPECLINK_WINDOWS_SIGNING 決策為條件',
  );
  assert.doesNotMatch(
    release,
    /HAS_APPLE_CERT|HAS_WINDOWS_CERT/,
    'release.yml：舊的 secrets 存在旗標應由閘門決策取代，避免兩處各自推導',
  );
});

test('release.yml 在 macOS 簽章啟用時注入公證三項，供 Tauri 完成公證與 staple', () => {
  const release = readFileSync(path.join(root, '.github/workflows/release.yml'), 'utf8');

  for (const name of ['APPLE_ID', 'APPLE_PASSWORD', 'APPLE_TEAM_ID']) {
    assert.match(
      release,
      new RegExp(`${name}=\\$\\{\\{ secrets\\.${name} \\}\\}`),
      `release.yml：缺少公證所需的 ${name} 注入`,
    );
  }
});
