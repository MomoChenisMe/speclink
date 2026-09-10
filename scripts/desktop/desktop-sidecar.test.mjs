// sidecar 佈署腳本的純函式層（dev-harness spec「dev 啟動自動佈署當前 checkout 的
// sidecar」，決策二～四）。覆蓋 --profile 白名單、建置產物來源路徑推導、以及
// 「內容相同即跳過複製」的防抖判定；實際 cargo 建置與 dev 啟動為手動驗證。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { builtBinaryPath, checkSpawn, parseSidecarArgs, shouldCopy } from './desktop-sidecar.mjs';

const ROOT = '/repo';

/// 每個測試自己的暫存目錄，離開時整份刪掉。
function tempDir(t) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'sidecar-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

// --- 參數解析（決策二：dev 佈 debug，無參數維持 release） ---

test('無旗標時 profile 預設 release、target 為 null（既有呼叫者的行為不變）', () => {
  assert.deepEqual(parseSidecarArgs([]), { profile: 'release', target: null });
});

test('--profile debug 與 --profile release 皆為合法值', () => {
  assert.deepEqual(parseSidecarArgs(['--profile', 'debug']), { profile: 'debug', target: null });
  assert.deepEqual(parseSidecarArgs(['--profile', 'release']), { profile: 'release', target: null });
});

test('--profile 與 --target 正交，可同時給（CI 交叉編譯的形狀）', () => {
  assert.deepEqual(parseSidecarArgs(['--target', 'x86_64-pc-windows-msvc']), {
    profile: 'release',
    target: 'x86_64-pc-windows-msvc',
  });
  assert.deepEqual(parseSidecarArgs(['--profile', 'debug', '--target', 'aarch64-apple-darwin']), {
    profile: 'debug',
    target: 'aarch64-apple-darwin',
  });
});

test('白名單外的 profile 值即拒絕，錯誤點名該值與兩個合法值', () => {
  assert.throws(
    () => parseSidecarArgs(['--profile', 'bogus']),
    (error) => {
      assert.match(error.message, /bogus/);
      assert.match(error.message, /debug/);
      assert.match(error.message, /release/);
      return true;
    },
  );
});

test('--profile 後沒接值時明確失敗，不靜默回退成 release', () => {
  assert.throws(() => parseSidecarArgs(['--profile']), /--profile/);
  assert.throws(() => parseSidecarArgs(['--profile', '--target', 'aarch64-apple-darwin']), /--profile/);
});

test('--target 後沒接值時明確失敗（既有行為）', () => {
  assert.throws(() => parseSidecarArgs(['--target']), /--target/);
});

// --- 子程序失敗形狀（predev 讓這支腳本每次 dev 啟動都跑，訊息是唯一的線索） ---

test('spawn 本身失敗時點名真正的原因，不報成「非零結束（null）」', () => {
  assert.throws(
    () => checkSpawn({ error: new Error('spawn cargo ENOENT'), status: null }, 'cargo', ['build']),
    (error) => {
      assert.match(error.message, /cargo build/);
      assert.match(error.message, /ENOENT/);
      assert.doesNotMatch(error.message, /null/);
      return true;
    },
  );
});

test('被訊號收束時點名該訊號（status 為 null 但沒有 error）', () => {
  assert.throws(
    () => checkSpawn({ status: null, signal: 'SIGKILL' }, 'cargo', ['build']),
    /signal SIGKILL/,
  );
});

test('非零狀態碼原樣點名', () => {
  assert.throws(() => checkSpawn({ status: 101 }, 'cargo', ['build']), /101/);
});

test('status 0 即通過', () => {
  assert.doesNotThrow(() => checkSpawn({ status: 0 }, 'cargo', ['build']));
});

// --- 來源路徑推導（profile × --target × Windows .exe） ---

test('無 --target：路徑為 target/<profile>/speclink', () => {
  assert.equal(
    builtBinaryPath(ROOT, { profile: 'debug', target: null, triple: 'aarch64-apple-darwin' }),
    path.join(ROOT, 'target/debug/speclink'),
  );
  assert.equal(
    builtBinaryPath(ROOT, { profile: 'release', target: null, triple: 'aarch64-apple-darwin' }),
    path.join(ROOT, 'target/release/speclink'),
  );
});

test('有 --target：路徑多一層 triple，為 target/<triple>/<profile>/speclink', () => {
  assert.equal(
    builtBinaryPath(ROOT, {
      profile: 'release',
      target: 'x86_64-unknown-linux-gnu',
      triple: 'x86_64-unknown-linux-gnu',
    }),
    path.join(ROOT, 'target/x86_64-unknown-linux-gnu/release/speclink'),
  );
});

test('Windows triple 帶 .exe 後綴（交叉編譯與 host 編譯皆然）', () => {
  assert.equal(
    builtBinaryPath(ROOT, {
      profile: 'release',
      target: 'x86_64-pc-windows-msvc',
      triple: 'x86_64-pc-windows-msvc',
    }),
    path.join(ROOT, 'target/x86_64-pc-windows-msvc/release/speclink.exe'),
  );
  assert.equal(
    builtBinaryPath(ROOT, { profile: 'debug', target: null, triple: 'x86_64-pc-windows-msvc' }),
    path.join(ROOT, 'target/debug/speclink.exe'),
  );
});

// --- 跳過複製判定（決策三：binaries/ 在 cargo 重編觸發清單內，無謂改寫要避免） ---

test('目的檔不存在即複製', (t) => {
  const dir = tempDir(t);
  const source = path.join(dir, 'source');
  writeFileSync(source, 'built');
  assert.equal(shouldCopy(source, path.join(dir, 'dest')), true);
});

test('目的檔內容與來源相同即跳過（不觸碰檔案，避免觸發重編）', (t) => {
  const dir = tempDir(t);
  const source = path.join(dir, 'source');
  const dest = path.join(dir, 'dest');
  writeFileSync(source, 'built');
  writeFileSync(dest, 'built');
  assert.equal(shouldCopy(source, dest), false);
});

test('目的檔內容與來源相異即複製（過期 sidecar 照常覆蓋）', (t) => {
  const dir = tempDir(t);
  const source = path.join(dir, 'source');
  const dest = path.join(dir, 'dest');
  writeFileSync(source, 'built');
  writeFileSync(dest, 'stale');
  assert.equal(shouldCopy(source, dest), true);
});

test('來源檔缺失時明確報錯點名該路徑，不當成「內容相同」靜默跳過', (t) => {
  const dir = tempDir(t);
  const source = path.join(dir, 'missing');
  assert.throws(() => shouldCopy(source, path.join(dir, 'dest')), (error) => {
    assert.match(error.message, /missing/);
    return true;
  });
});
