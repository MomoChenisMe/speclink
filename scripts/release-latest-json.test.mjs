// latest.json 組裝腳本的單元測試（desktop-release spec「更新描述檔隨 release 發布」，
// design D2）。以子行程執行 scripts/release-latest-json.mjs：給定 tag 與各平台更新包
// ＋簽章檔，驗證輸出欄位；缺任一必要平台輸入時必須以非零結束（fail-closed）。
//
// 目錄契約：--dir 之下每個平台鍵一個子目錄（darwin-aarch64 等），內含恰好一個
// 更新包與其同名 .sig——由 release workflow 的 artifact 下載步驟佈置。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts/release-latest-json.mjs');

const REPO = 'MomoChenisMe/speclink';
const TAG = 'v0.2.0';

// 必要四平台與各自的更新包檔名（命名含架構，避免 Release asset 撞名）。
const REQUIRED_PLATFORMS = {
  'darwin-aarch64': 'Speclink_0.2.0_aarch64.app.tar.gz',
  'darwin-x86_64': 'Speclink_0.2.0_x64.app.tar.gz',
  'windows-x86_64': 'Speclink_0.2.0_x64-setup.exe',
  'linux-x86_64': 'Speclink_0.2.0_amd64.AppImage',
};

/// 佈置 --dir 目錄：每平台一個子目錄，放更新包與 .sig（內容為平台專屬字串）。
function layoutArtifacts(dir, platforms) {
  for (const [key, fileName] of Object.entries(platforms)) {
    const platformDir = path.join(dir, key);
    mkdirSync(platformDir, { recursive: true });
    writeFileSync(path.join(platformDir, fileName), `binary-${key}`);
    writeFileSync(path.join(platformDir, `${fileName}.sig`), `sig-${key}\n`);
  }
}

function runScript(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

test('組裝出的 latest.json：version 去 v 前綴、四平台 url 指向 Release asset、signature 為簽章內容', (t) => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'latest-json-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  layoutArtifacts(dir, REQUIRED_PLATFORMS);
  const out = path.join(dir, 'latest.json');

  const result = runScript(['--tag', TAG, '--dir', dir, '--repo', REPO, '--out', out]);
  assert.equal(result.status, 0, `腳本應成功結束\nstderr: ${result.stderr}`);

  const manifest = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(manifest.version, '0.2.0', 'version 必須是 tag 去除 v 前綴');
  assert.ok(
    !Number.isNaN(Date.parse(manifest.pub_date)),
    `pub_date 必須是可解析的時間，實際 ${manifest.pub_date}`,
  );

  for (const [key, fileName] of Object.entries(REQUIRED_PLATFORMS)) {
    const entry = manifest.platforms?.[key];
    assert.ok(entry, `platforms 缺少 ${key}`);
    assert.equal(
      entry.url,
      `https://github.com/${REPO}/releases/download/${TAG}/${fileName}`,
      `${key} 的 url 必須指向該 Release asset 下載路徑`,
    );
    assert.equal(entry.signature, `sig-${key}`, `${key} 的 signature 必須是簽章檔內容（去尾端空白）`);
  }
});

test('額外平台（linux-aarch64）存在時一併收錄', (t) => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'latest-json-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  layoutArtifacts(dir, {
    ...REQUIRED_PLATFORMS,
    'linux-aarch64': 'Speclink_0.2.0_aarch64.AppImage',
  });
  const out = path.join(dir, 'latest.json');

  const result = runScript(['--tag', TAG, '--dir', dir, '--repo', REPO, '--out', out]);
  assert.equal(result.status, 0, `腳本應成功結束\nstderr: ${result.stderr}`);

  const manifest = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(
    manifest.platforms['linux-aarch64']?.url,
    `https://github.com/${REPO}/releases/download/${TAG}/Speclink_0.2.0_aarch64.AppImage`,
  );
  assert.equal(manifest.platforms['linux-aarch64']?.signature, 'sig-linux-aarch64');
});

test('缺任一必要平台輸入時以非零結束並點名缺失平台（fail-closed）', (t) => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'latest-json-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const partial = { ...REQUIRED_PLATFORMS };
  delete partial['windows-x86_64'];
  layoutArtifacts(dir, partial);
  const out = path.join(dir, 'latest.json');

  const result = runScript(['--tag', TAG, '--dir', dir, '--repo', REPO, '--out', out]);
  assert.notEqual(result.status, 0, '缺 windows-x86_64 時必須以非零結束');
  assert.match(result.stderr, /windows-x86_64/, 'stderr 必須點名缺失的平台');
});

test('更新包存在但缺 .sig 時以非零結束（簽章 fail-closed）', (t) => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'latest-json-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  layoutArtifacts(dir, REQUIRED_PLATFORMS);
  rmSync(path.join(dir, 'darwin-aarch64', `${REQUIRED_PLATFORMS['darwin-aarch64']}.sig`));
  const out = path.join(dir, 'latest.json');

  const result = runScript(['--tag', TAG, '--dir', dir, '--repo', REPO, '--out', out]);
  assert.notEqual(result.status, 0, '缺簽章檔時必須以非零結束');
  assert.match(result.stderr, /darwin-aarch64/, 'stderr 必須點名缺簽章的平台');
});
