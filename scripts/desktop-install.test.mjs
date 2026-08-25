// 本機安裝入口的斷言鏈（desktop-release spec「本機安裝的新鮮度斷言」）。
// 覆蓋純函式部分：源碼版號讀取、--version 的引擎版號解析、版號斷言的失敗形狀，
// 以及 --install 的平台限定與簽章環境變數檢查。實際建置與安裝為手動驗證。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  assertSameEngineVersion,
  engineVersionOf,
  missingSigningEnv,
  preflight,
  sourceAssetVersion,
} from './desktop-install.mjs';

const SIGNED = {
  TAURI_SIGNING_PRIVATE_KEY: 'k',
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: 'p',
};

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('源碼版號取自 speclink-core 的 ASSET_VERSION 常數', () => {
  const text = readFileSync(path.join(root, 'crates/speclink-core/src/init.rs'), 'utf8');
  const version = sourceAssetVersion(text);
  assert.match(version, /^v\d+\.\d+\.\d+$/);
  assert.ok(text.includes(`"${version}"`), '讀出的版號須確實出現在源碼中');
});

test('讀不到 ASSET_VERSION 常數時明確失敗，不回空字串', () => {
  assert.throws(() => sourceAssetVersion('沒有常數的檔案\n'), /ASSET_VERSION/);
});

test('--version 輸出的引擎版號可解析', () => {
  assert.equal(engineVersionOf('speclink 0.1.0 (arm64, engine v1.14.0)\n'), 'v1.14.0');
});

test('--version 不含引擎版號時明確失敗（舊 binary 的形狀）', () => {
  assert.throws(() => engineVersionOf('speclink 0.1.0 (arm64)\n'), /engine/);
});

test('版號不符的斷言印出兩邊版號', () => {
  assert.throws(
    () => assertSameEngineVersion('bundle', 'v1.11.0', 'v1.14.0'),
    (error) => {
      assert.match(error.message, /v1\.11\.0/);
      assert.match(error.message, /v1\.14\.0/);
      assert.match(error.message, /bundle/);
      return true;
    },
  );
  assert.doesNotThrow(() => assertSameEngineVersion('bundle', 'v1.14.0', 'v1.14.0'));
});

test('簽章環境變數缺失時點名缺的變數，齊備時回空', () => {
  assert.deepEqual(missingSigningEnv({}), [
    'TAURI_SIGNING_PRIVATE_KEY',
    'TAURI_SIGNING_PRIVATE_KEY_PASSWORD',
  ]);
  assert.deepEqual(missingSigningEnv({ TAURI_SIGNING_PRIVATE_KEY: 'k' }), [
    'TAURI_SIGNING_PRIVATE_KEY_PASSWORD',
  ]);
  assert.deepEqual(
    missingSigningEnv({ TAURI_SIGNING_PRIVATE_KEY: 'k', TAURI_SIGNING_PRIVATE_KEY_PASSWORD: 'p' }),
    [],
  );
});

test('preflight：非 macOS 帶 --install 即拒絕，點名平台', () => {
  assert.throws(() => preflight({ env: SIGNED, platform: 'linux', install: true }), /macOS/);
  assert.throws(() => preflight({ env: SIGNED, platform: 'win32', install: true }), /win32/);
});

test('preflight：簽章環境變數缺失即拒絕並點名變數', () => {
  assert.throws(
    () => preflight({ env: {}, platform: 'darwin', install: false }),
    /TAURI_SIGNING_PRIVATE_KEY/,
  );
});

test('preflight：macOS 且簽章 env 齊備即通過（帶不帶 --install 皆然）', () => {
  assert.doesNotThrow(() => preflight({ env: SIGNED, platform: 'darwin', install: false }));
  assert.doesNotThrow(() => preflight({ env: SIGNED, platform: 'darwin', install: true }));
});

test('preflight：不帶 --install 時平台不設限（建置步驟平台中立）', () => {
  assert.doesNotThrow(() => preflight({ env: SIGNED, platform: 'linux', install: false }));
});
