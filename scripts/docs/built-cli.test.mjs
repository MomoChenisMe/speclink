// 文件守門測試共用的 CLI 定位：只認這個 repo 建出來的 binary，兩種 profile 都在時取較新的。
// 測試只在暫存目錄裡造假的 target/ 結構，不碰真正的 target/。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, utimesSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { builtCli } from './built-cli.mjs';

const exe = process.platform === 'win32' ? 'speclink.exe' : 'speclink';

function fakeRoot(profiles) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'speclink-built-cli-'));
  for (const [profile, seconds] of Object.entries(profiles)) {
    const file = path.join(root, 'target', profile, exe);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, '');
    utimesSync(file, seconds, seconds);
  }
  return root;
}

test('兩種 profile 都在時取較新的那個（本機的 cargo test 會把 debug 換回舊產物）', () => {
  const root = fakeRoot({ debug: 1_000, release: 2_000 });
  try {
    assert.equal(builtCli(root), path.join(root, 'target', 'release', exe));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('只有一種 profile 時就用它；都沒有時回 null，呼叫端據此 skip', () => {
  const debugOnly = fakeRoot({ debug: 1_000 });
  const none = fakeRoot({});
  try {
    assert.equal(builtCli(debugOnly), path.join(debugOnly, 'target', 'debug', exe));
    assert.equal(builtCli(none), null);
  } finally {
    rmSync(debugOnly, { recursive: true, force: true });
    rmSync(none, { recursive: true, force: true });
  }
});
