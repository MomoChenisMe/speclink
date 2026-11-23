// 本機安裝入口的斷言鏈（desktop-release spec「本機安裝的新鮮度斷言」）。
// 覆蓋純函式部分：源碼版號讀取、--version 的引擎版號解析、版號斷言的失敗形狀，
// --install 的平台限定與簽章環境變數檢查，以及程式碼簽章身分的選擇與簽章結果的判斷。
// 實際建置、安裝與 security／codesign 的呼叫為手動驗證。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  assertSameEngineVersion,
  chooseSigningIdentity,
  developerIdIdentity,
  engineVersionOf,
  missingSigningEnv,
  preflight,
  signatureProblem,
  sourceAssetVersion,
} from './desktop-install.mjs';

const SIGNED = {
  TAURI_SIGNING_PRIVATE_KEY: 'k',
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: 'p',
};

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

test('源碼版號取自 speclink-core 的 ASSET_VERSION 常數', () => {
  const text = readFileSync(path.join(root, 'crates/engine/speclink-core/src/workspace/init.rs'), 'utf8');
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

// security find-identity -v -p codesigning 的輸出形狀（身分名稱為虛構）。
const DEVELOPER_ID = 'Developer ID Application: Acme Inc (A1B2C3D4E5)';
const findIdentity = (...names) =>
  [
    ...names.map((name, i) => `  ${i + 1}) ${String(i + 1).repeat(40)} "${name}"`),
    `     ${names.length} valid identities found`,
    '',
  ].join('\n');

test('developerIdIdentity：只有一張 Developer ID 時回傳它', () => {
  assert.equal(developerIdIdentity(findIdentity(DEVELOPER_ID)), DEVELOPER_ID);
});

test('developerIdIdentity：兩張 Developer ID 時回傳第一張', () => {
  const second = 'Developer ID Application: Globex LLC (Z9Y8X7W6V5)';
  assert.equal(developerIdIdentity(findIdentity(DEVELOPER_ID, second)), DEVELOPER_ID);
});

test('developerIdIdentity：只有 Apple Development 身分時回傳 null', () => {
  assert.equal(
    developerIdIdentity(findIdentity('Apple Development: dev@example.com (Q1W2E3R4T5)')),
    null,
  );
});

test('developerIdIdentity：0 valid identities found 與空字串回傳 null', () => {
  assert.equal(developerIdIdentity('     0 valid identities found\n'), null);
  assert.equal(developerIdIdentity(''), null);
});

const AD_HOC_NOTE = '找不到 Developer ID Application 憑證，這次以 ad-hoc 簽章建置；重裝後鑰匙圈會重新詢問';

/// 記錄是否被呼叫的假 lookup。
function fakeLookup(result) {
  const lookup = () => {
    lookup.called = true;
    if (result instanceof Error) throw result;
    return result;
  };
  lookup.called = false;
  return lookup;
}

test('chooseSigningIdentity：非 macOS 不查找，也沒有說明', () => {
  const lookup = fakeLookup(findIdentity(DEVELOPER_ID));
  assert.deepEqual(chooseSigningIdentity({ env: {}, platform: 'linux', lookup }), {
    identity: null,
    note: null,
  });
  assert.equal(lookup.called, false);
});

test('chooseSigningIdentity：APPLE_SIGNING_IDENTITY 有值時用它，不查找鑰匙圈', () => {
  const lookup = fakeLookup(findIdentity(DEVELOPER_ID));
  const env = { APPLE_SIGNING_IDENTITY: 'Developer ID Application: Globex LLC (Z9Y8X7W6V5)' };
  assert.deepEqual(chooseSigningIdentity({ env, platform: 'darwin', lookup }), {
    identity: env.APPLE_SIGNING_IDENTITY,
    note: null,
  });
  assert.equal(lookup.called, false);
});

test('chooseSigningIdentity：查找到 Developer ID 時用它', () => {
  const lookup = fakeLookup(findIdentity(DEVELOPER_ID));
  assert.deepEqual(chooseSigningIdentity({ env: {}, platform: 'darwin', lookup }), {
    identity: DEVELOPER_ID,
    note: null,
  });
  assert.equal(lookup.called, true);
});

test('chooseSigningIdentity：沒有 Developer ID 時以 ad-hoc 建置並說明', () => {
  const lookup = fakeLookup('     0 valid identities found\n');
  assert.deepEqual(chooseSigningIdentity({ env: {}, platform: 'darwin', lookup }), {
    identity: null,
    note: AD_HOC_NOTE,
  });
});

test('chooseSigningIdentity：查找丟錯時以 ad-hoc 建置，說明附上錯誤訊息', () => {
  const lookup = fakeLookup(new Error('security find-identity -v -p codesigning 以非零結束（1）'));
  const { identity, note } = chooseSigningIdentity({ env: {}, platform: 'darwin', lookup });
  assert.equal(identity, null);
  assert.ok(note.startsWith(AD_HOC_NOTE), note);
  assert.match(note, /以非零結束（1）/);
});

// codesign -dv 印在 stderr 的形狀（身分名稱為虛構）。
const SIGNED_CLI = [
  'Executable=/Applications/Speclink.app/Contents/MacOS/speclink',
  'Identifier=speclink',
  'Format=Mach-O thin (arm64)',
  'CodeDirectory v=20500 size=96536 flags=0x10000(runtime) hashes=3013+7 location=embedded',
  'Signature size=9046',
  `Authority=${DEVELOPER_ID}`,
  'Authority=Developer ID Certification Authority',
  'Authority=Apple Root CA',
  'Info.plist=not bound',
  'TeamIdentifier=A1B2C3D4E5',
  'Sealed Resources=none',
  '',
].join('\n');
const AD_HOC_CLI = [
  'Executable=/Applications/Speclink.app/Contents/MacOS/speclink',
  'Identifier=speclink-9973c961f6bb5a30',
  'Format=Mach-O thin (arm64)',
  'CodeDirectory v=20400 size=96536 flags=0x20002(adhoc,linker-signed) hashes=3013+0 location=embedded',
  'Signature=adhoc',
  'Info.plist=not bound',
  'TeamIdentifier=not set',
  'Sealed Resources=none',
  '',
].join('\n');

test('signatureProblem：帶 Team ID 的 Developer ID 簽章回傳 null', () => {
  assert.equal(signatureProblem(SIGNED_CLI), null);
});

test('signatureProblem：ad-hoc 且 TeamIdentifier=not set 時回傳原因與原始輸出', () => {
  const problem = signatureProblem(AD_HOC_CLI);
  assert.notEqual(problem, null);
  assert.ok(problem.includes('Identifier=speclink-9973c961f6bb5a30'), problem);
});

test('signatureProblem：有 Team ID 但同時是 ad-hoc 時不通過', () => {
  assert.notEqual(signatureProblem(`${SIGNED_CLI}Signature=adhoc\n`), null);
});

test('signatureProblem：沒有 TeamIdentifier 行時不通過', () => {
  const output = 'path/to/speclink: code object is not signed at all\n';
  const problem = signatureProblem(output);
  assert.notEqual(problem, null);
  assert.ok(problem.includes('code object is not signed at all'), problem);
});
