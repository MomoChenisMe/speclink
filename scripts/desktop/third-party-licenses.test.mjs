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

const METADATA = {
  workspace_members: ['path+file:///repo/crates/core#speclink-core@0.8.0'],
  packages: [
    { id: 'registry+https://github.com/rust-lang/crates.io-index#serde@1.0.200', name: 'serde', version: '1.0.200', license: 'MIT OR Apache-2.0' },
    { id: 'path+file:///repo/crates/core#speclink-core@0.8.0', name: 'speclink-core', version: '0.8.0', license: 'MIT' },
    { id: 'registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.80', name: 'anyhow', version: '1.0.80', license: 'MIT OR Apache-2.0' },
    { id: 'registry+https://github.com/rust-lang/crates.io-index#weird@0.1.0', name: 'weird', version: '0.1.0', license: null },
  ],
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

test('cargo 節排除 workspace 成員並依名稱排序', () => {
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

test('cargo 成功時 readCargoMetadata 解析 stdout 的 JSON', () => {
  const ok = () => ({ status: 0, stdout: JSON.stringify(METADATA) });
  assert.deepEqual(readCargoMetadata({ spawn: ok, warn: () => assert.fail('不應警告') }), METADATA);
});
