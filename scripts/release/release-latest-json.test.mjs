// latest.json 組裝腳本的單元測試（desktop-release spec「更新描述檔隨 release 發布」，
// design D2／release-assets-trim D3）。以子行程執行 scripts/release/release-latest-json.mjs：
// 給定 tag 與各更新包目錄＋簽章檔，驗證輸出欄位；缺任一必要目錄輸入時必須以非零結束
// （fail-closed）。
//
// 目錄契約：--dir 之下每個「更新包目錄」對應一或多個平台鍵——darwin-universal 一個
// 目錄餵 darwin-aarch64 與 darwin-x86_64 兩鍵（universal 更新包），windows-x86_64 與
// linux-x86_64 各對應同名鍵，linux-aarch64 可選；每個目錄內含恰好一個更新包與其同名
// .sig——由 release workflow 的 artifact 下載步驟佈置。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const script = path.join(root, 'scripts/release/release-latest-json.mjs');

const REPO = 'MomoChenisMe/speclink';
const TAG = 'v0.5.0';

// 必要三個更新包目錄與各自的更新包檔名。
const REQUIRED_DIRS = {
  'darwin-universal': 'Speclink_0.5.0_universal.app.tar.gz',
  'windows-x86_64': 'Speclink_0.5.0_x64-setup.exe',
  'linux-x86_64': 'Speclink_0.5.0_amd64.AppImage',
};

/// 佈置 --dir 目錄：每個更新包目錄放更新包與 .sig（內容為目錄專屬字串）。
function layoutArtifacts(dir, dirs) {
  for (const [key, fileName] of Object.entries(dirs)) {
    const packageDir = path.join(dir, key);
    mkdirSync(packageDir, { recursive: true });
    writeFileSync(path.join(packageDir, fileName), `binary-${key}`);
    writeFileSync(path.join(packageDir, `${fileName}.sig`), `sig-${key}\n`);
  }
}

function runScript(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

/// 佈置一組更新包目錄到暫存 --dir，回傳目錄與預期的輸出路徑。
function layout(t, dirs) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'latest-json-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  layoutArtifacts(dir, dirs);
  return { dir, out: path.join(dir, 'latest.json') };
}

function assemble(t, dirs) {
  const { dir, out } = layout(t, dirs);
  const result = runScript(['--tag', TAG, '--dir', dir, '--repo', REPO, '--out', out]);
  return { dir, out, result };
}

test('組裝出的 latest.json：version 去 v 前綴、四個必要平台鍵齊備、url 指向 Release asset、signature 為簽章內容', (t) => {
  const { out, result } = assemble(t, REQUIRED_DIRS);
  assert.equal(result.status, 0, `腳本應成功結束\nstderr: ${result.stderr}`);

  const manifest = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(manifest.version, '0.5.0', 'version 必須是 tag 去除 v 前綴');
  assert.ok(
    !Number.isNaN(Date.parse(manifest.pub_date)),
    `pub_date 必須是可解析的時間，實際 ${manifest.pub_date}`,
  );

  // Tauri updater 依執行機器查 darwin-aarch64／darwin-x86_64／windows-x86_64／linux-x86_64，
  // 鍵名是 updater 的契約，四鍵缺一不可。
  const expected = {
    'darwin-aarch64': REQUIRED_DIRS['darwin-universal'],
    'darwin-x86_64': REQUIRED_DIRS['darwin-universal'],
    'windows-x86_64': REQUIRED_DIRS['windows-x86_64'],
    'linux-x86_64': REQUIRED_DIRS['linux-x86_64'],
  };
  for (const [key, fileName] of Object.entries(expected)) {
    const entry = manifest.platforms?.[key];
    assert.ok(entry, `platforms 缺少 ${key}`);
    assert.equal(
      entry.url,
      `https://github.com/${REPO}/releases/download/${TAG}/${fileName}`,
      `${key} 的 url 必須指向該 Release asset 下載路徑`,
    );
  }
  assert.equal(manifest.platforms['windows-x86_64'].signature, 'sig-windows-x86_64', 'signature 必須是簽章檔內容（去尾端空白）');
  assert.equal(manifest.platforms['linux-x86_64'].signature, 'sig-linux-x86_64');
});

test('darwin-aarch64 與 darwin-x86_64 兩鍵共用 universal 更新包：url 與 signature 相同', (t) => {
  const { out, result } = assemble(t, REQUIRED_DIRS);
  assert.equal(result.status, 0, `腳本應成功結束\nstderr: ${result.stderr}`);

  const { platforms } = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(
    platforms['darwin-aarch64'].url,
    `https://github.com/${REPO}/releases/download/${TAG}/Speclink_0.5.0_universal.app.tar.gz`,
  );
  assert.deepEqual(platforms['darwin-x86_64'], platforms['darwin-aarch64'], 'darwin 兩鍵必須指向同一檔、同一簽章');
  assert.equal(platforms['darwin-aarch64'].signature, 'sig-darwin-universal');
  assert.equal(platforms['darwin-universal'], undefined, 'darwin-universal 是目錄名，不是 updater 的平台鍵');
});

test('額外目錄（linux-aarch64）存在時一併收錄', (t) => {
  const { out, result } = assemble(t, {
    ...REQUIRED_DIRS,
    'linux-aarch64': 'Speclink_0.5.0_aarch64.AppImage',
  });
  assert.equal(result.status, 0, `腳本應成功結束\nstderr: ${result.stderr}`);

  const manifest = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(
    manifest.platforms['linux-aarch64']?.url,
    `https://github.com/${REPO}/releases/download/${TAG}/Speclink_0.5.0_aarch64.AppImage`,
  );
  assert.equal(manifest.platforms['linux-aarch64']?.signature, 'sig-linux-aarch64');
});

test('缺任一必要目錄時以非零結束並點名（fail-closed），不寫出描述檔', (t) => {
  for (const missing of Object.keys(REQUIRED_DIRS)) {
    const partial = { ...REQUIRED_DIRS };
    delete partial[missing];
    const { out, result } = assemble(t, partial);
    assert.notEqual(result.status, 0, `缺 ${missing} 時必須以非零結束`);
    assert.match(result.stderr, new RegExp(missing), `stderr 必須點名缺失的目錄 ${missing}`);
    assert.throws(() => readFileSync(out), `缺 ${missing} 時不得寫出 latest.json`);
  }
});

test('更新包存在但缺 .sig 時以非零結束（簽章 fail-closed）', (t) => {
  const { dir, out } = layout(t, REQUIRED_DIRS);
  rmSync(path.join(dir, 'darwin-universal', `${REQUIRED_DIRS['darwin-universal']}.sig`));

  const result = runScript(['--tag', TAG, '--dir', dir, '--repo', REPO, '--out', out]);
  assert.notEqual(result.status, 0, '缺簽章檔時必須以非零結束');
  assert.match(result.stderr, /darwin-universal/, 'stderr 必須點名缺簽章的目錄');
  assert.throws(() => readFileSync(out), '缺簽章時不得寫出 latest.json');
});
