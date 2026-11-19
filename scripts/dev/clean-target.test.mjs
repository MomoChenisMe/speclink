import assert from 'node:assert/strict';
import test from 'node:test';

import { busyBuildProcesses, formatSize } from './clean-target.mjs';

test('cargo、rustc、tauri 在跑時算有編譯進行，帶路徑也認得', () => {
  const ps = ['/sbin/launchd', '/Users/dev/.cargo/bin/cargo', 'rustc', '/opt/homebrew/bin/tauri', ''].join('\n');
  assert.deepEqual(busyBuildProcesses(ps), ['/Users/dev/.cargo/bin/cargo', 'rustc', '/opt/homebrew/bin/tauri']);
});

test('rust-analyzer 與名稱只是含 cargo 的程序不算', () => {
  const ps = ['/Users/dev/.cargo/bin/rust-analyzer', 'cargo-watch', '/Applications/Cargo Helper.app/Contents/MacOS/Cargo Helper', 'node'].join('\n');
  assert.deepEqual(busyBuildProcesses(ps), []);
});

test('大小以 GB 或 MB 顯示', () => {
  assert.equal(formatSize(51 * 1024 * 1024), '51.0 GB');
  assert.equal(formatSize(1536 * 1024), '1.5 GB');
  assert.equal(formatSize(488 * 1024), '488 MB');
  assert.equal(formatSize(0), '0 MB');
});
