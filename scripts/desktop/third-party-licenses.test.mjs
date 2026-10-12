// 第三方授權清單腳本的純函式層（desktop-about-page design D3）：以 fixture lock 與假的
// cargo metadata 驗證輸出形狀與排序，不碰真 cargo。
import { test } from 'node:test';
import assert from 'node:assert/strict';

import { buildLicenseList, cargoLicenses, npmLicenses, readCargoMetadata } from './third-party-licenses.mjs';

/** 兩個工作區：desktop 依賴 ui（workspace 連結）與 zeta；ui 依賴 alpha；dev 相依各一。 */
const LOCK = {
  lockfileVersion: 3,
  packages: {
    '': { workspaces: ['packages/ui', 'apps/desktop'] },
    'apps/desktop': {
      name: '@speclink/desktop',
      dependencies: { '@speclink/ui': '*', zeta: '^1.0.0', shared: '^2.0.0' },
      devDependencies: { vitest: '^3.0.0' },
    },
    'packages/ui': {
      name: '@speclink/ui',
      dependencies: { alpha: '^1.0.0', shared: '^1.0.0' },
      devDependencies: { jsdom: '^25.0.0' },
    },
    'node_modules/@speclink/ui': { resolved: 'packages/ui', link: true },
    'node_modules/zeta': { version: '1.2.0', license: 'MIT', dependencies: { beta: '^0.1.0' } },
    'node_modules/beta': { version: '0.1.3' },
    'node_modules/alpha': { version: '1.0.0', license: 'ISC' },
    'node_modules/shared': { version: '1.5.0', license: 'MIT' },
    'apps/desktop/node_modules/shared': { version: '2.0.1', license: 'Apache-2.0' },
    'node_modules/vitest': { version: '3.0.0', license: 'MIT', dependencies: { devonly: '^1.0.0' } },
    'node_modules/devonly': { version: '1.0.0', license: 'MIT' },
    'node_modules/jsdom': { version: '25.0.0', license: 'MIT' },
  },
};

const crate = (name, version, license, source = 'registry+https://github.com/rust-lang/crates.io-index') => ({
  id: `${source}#${name}@${version}`,
  name,
  version,
  license,
});
const member = (name) => crate(name, '0.8.0', 'MIT', `path+file:///repo/${name}`);
const DESKTOP = member('speclink-desktop');
const CLI = member('speclink-cli');
const CORE = member('speclink-core');
const SERVER = member('speclink-server');
const SERDE = crate('serde', '1.0.200', 'MIT OR Apache-2.0');
const ANYHOW = crate('anyhow', '1.0.80', 'MIT OR Apache-2.0');
const WEIRD = crate('weird', '0.1.0', null);
const POSTGRES = crate('tokio-postgres', '0.7.0', 'MIT OR Apache-2.0');
const INSTA = crate('insta', '1.40.0', 'Apache-2.0');
const CC = crate('cc', '1.1.0', 'MIT OR Apache-2.0');
const NORMAL = [{ kind: null, target: null }];
const dep = (pkg, dep_kinds = NORMAL) => ({ name: pkg.name, pkg: pkg.id, dep_kinds });

/** desktop 與 cli 為根：經 core 遞迴到 weird；server 自己的 tokio-postgres、desktop 的 dev／build 相依不列入。 */
const METADATA = {
  workspace_members: [DESKTOP.id, CLI.id, CORE.id, SERVER.id],
  packages: [SERDE, DESKTOP, CLI, CORE, SERVER, ANYHOW, WEIRD, POSTGRES, INSTA, CC],
  resolve: {
    nodes: [
      {
        id: DESKTOP.id,
        deps: [dep(CORE), dep(SERDE), dep(INSTA, [{ kind: 'dev', target: null }]), dep(CC, [{ kind: 'build', target: null }])],
      },
      { id: CLI.id, deps: [dep(CORE), dep(ANYHOW)] },
      { id: CORE.id, deps: [dep(WEIRD), dep(SERDE)] },
      { id: SERVER.id, deps: [dep(CORE), dep(POSTGRES)] },
      { id: SERDE.id, deps: [] },
      { id: ANYHOW.id, deps: [] },
      { id: WEIRD.id, deps: [] },
      { id: POSTGRES.id, deps: [] },
      { id: INSTA.id, deps: [] },
      { id: CC.id, deps: [] },
    ],
  },
};

test('npm 節只含兩個工作區的生產相依（遞迴），不含 dev 相依與工作區本身', () => {
  const names = npmLicenses(LOCK).map((entry) => `${entry.name}@${entry.version}`);
  assert.deepEqual(names, ['alpha@1.0.0', 'beta@0.1.3', 'shared@1.5.0', 'shared@2.0.1', 'zeta@1.2.0']);
});

test('npm 相依依 node_modules 巢狀規則解析：工作區自己的 node_modules 先於根目錄', () => {
  const shared = npmLicenses(LOCK).filter((entry) => entry.name === 'shared');
  assert.deepEqual(shared, [
    { name: 'shared', version: '1.5.0', license: 'MIT' },
    { name: 'shared', version: '2.0.1', license: 'Apache-2.0' },
  ]);
});

test('授權欄缺席時為 null（UI 顯示為「未標示」）', () => {
  assert.equal(npmLicenses(LOCK).find((entry) => entry.name === 'beta').license, null);
  assert.equal(cargoLicenses(METADATA).find((entry) => entry.name === 'weird').license, null);
});

test('cargo 節只含桌面 app 與 CLI 的正式相依（遞迴穿過 workspace 成員），不含成員本身、dev／build 相依與其他成員的相依，依名稱排序', () => {
  assert.deepEqual(cargoLicenses(METADATA), [
    { name: 'anyhow', version: '1.0.80', license: 'MIT OR Apache-2.0' },
    { name: 'serde', version: '1.0.200', license: 'MIT OR Apache-2.0' },
    { name: 'weird', version: '0.1.0', license: null },
  ]);
});

test('cargo 缺席時仍輸出 npm 節，cargo 節為空陣列', () => {
  const list = buildLicenseList({ lock: LOCK, metadata: null });
  assert.equal(list.npm.length, 5);
  assert.deepEqual(list.cargo, []);
});

test('cargo 無法執行或非零結束時 readCargoMetadata 回 null 並警告', () => {
  const warnings = [];
  const warn = (message) => warnings.push(message);
  const missing = () => ({ error: new Error('spawn cargo ENOENT'), status: null });
  assert.equal(readCargoMetadata({ spawn: missing, warn }), null);
  const failed = () => ({ status: 101, stdout: '', stderr: 'error: failed to load manifest' });
  assert.equal(readCargoMetadata({ spawn: failed, warn }), null);
  assert.equal(warnings.length, 2);
  assert.match(warnings[0], /cargo/);
});

test('cargo 成功時 readCargoMetadata 解析 stdout 的 JSON；帶 --locked，不改寫過期的 Cargo.lock', () => {
  let args;
  const ok = (_command, spawnArgs) => {
    args = spawnArgs;
    return { status: 0, stdout: JSON.stringify(METADATA) };
  };
  assert.deepEqual(readCargoMetadata({ spawn: ok, warn: () => assert.fail('不應警告') }), METADATA);
  assert.ok(args.includes('--locked'), args.join(' '));
});
