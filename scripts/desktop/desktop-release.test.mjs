import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { desktopReleaseVersion } from './desktop-release-version.mjs';

const workflow = readFileSync(new URL('../../.github/workflows/desktop-build.yml', import.meta.url), 'utf8');
const root = fileURLToPath(new URL('../../', import.meta.url));

test('CI 版號檢查輸出可供後續工作使用，錯誤 Tag 不寫入輸出', (t) => {
  const temp = mkdtempSync(join(tmpdir(), 'speclink-desktop-release-'));
  t.after(() => rmSync(temp, { recursive: true, force: true }));
  const version = JSON.parse(readFileSync(new URL('../../apps/desktop/src-tauri/tauri.conf.json', import.meta.url), 'utf8')).version;
  for (const [event, tag, passes] of [['push', `desktop-v${version}`, true], ['push', 'desktop-v999.0.0', false], ['workflow_dispatch', 'main', true]]) {
    const output = join(temp, `${event}-${tag}.txt`);
    const result = spawnSync(process.execPath, ['scripts/desktop/desktop-release-version.mjs'], {
      cwd: root,
      env: { ...process.env, GITHUB_EVENT_NAME: event, GITHUB_REF_NAME: tag, GITHUB_OUTPUT: output },
      encoding: 'utf8',
    });
    assert.equal(result.status, passes ? 0 : 1, result.stderr);
    if (passes) assert.equal(readFileSync(output, 'utf8'), `version=${version}\n`);
    else assert.equal(existsSync(output), false);
  }
});

test('桌面 tag 與 App、CLI 同版才可發布', () => {
  assert.equal(desktopReleaseVersion('desktop-v0.9.0', '0.9.0', '0.9.0'), '0.9.0');
  for (const tag of ['v0.9.0', 'desktop-v01.9.0', 'desktop-v0.9', 'desktop-v0.9.0-', 'desktop-v0.9.0-rc..1', 'desktop-v0.9.0-rc.01', 'desktop-v0.9.0;echo bad']) {
    assert.throws(() => desktopReleaseVersion(tag, '0.9.0', '0.9.0'), /desktop-vX\.Y\.Z/);
  }
  assert.throws(() => desktopReleaseVersion('desktop-v0.9.1', '0.9.0', '0.9.1'), /App/);
  assert.throws(() => desktopReleaseVersion('desktop-v0.9.1', '0.9.1', '0.9.0'), /CLI/);
});

test('接受合法預先發行版，仍要求 App 與 CLI 的完整版本一致', () => {
  for (const version of ['0.9.0-build.m1', '0.9.0-rc.1']) {
    assert.equal(desktopReleaseVersion(`desktop-v${version}`, version, version), version);
    assert.throws(() => desktopReleaseVersion(`desktop-v${version}`, '0.9.0', version), /App/);
    assert.throws(() => desktopReleaseVersion(`desktop-v${version}`, version, '0.9.0'), /CLI/);
  }
  assert.ok(workflow.includes("prerelease: ${{ contains(needs.prepare.outputs.version, '-') }}"));
});

test('保留手動建置，僅桌面 tag 可發布，四平台全部成功才執行發布', () => {
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /tags:\s*\['desktop-v\*'\]/);
  assert.match(workflow, /desktop:\s*\n\s+needs: \[prepare\]/);
  const publish = workflow.slice(workflow.indexOf('\n  release:'));
  const desktop = workflow.slice(workflow.indexOf('\n  desktop:'), workflow.indexOf('\n  release:'));
  assert.match(desktop, /node scripts\/desktop\/windows-portable\.mjs/);
  assert.match(desktop, /name: Upload Windows portable ZIP/);
  assert.match(publish, /needs: \[prepare, desktop\]/);
  assert.match(publish, /if: github\.event_name == 'push' && startsWith\(github\.ref, 'refs\/tags\/desktop-v'\)/);
  assert.match(publish, /contents: write/);
  assert.doesNotMatch(workflow.slice(0, workflow.indexOf('\njobs:')), /contents: write/);
  assert.doesNotMatch(publish, /always\(\)|continue-on-error/);
});

test('只發布五種桌面附件，缺任一檔案失敗，保留免私鑰建置方式', () => {
  const publish = workflow.slice(workflow.indexOf('\n  release:'));
  for (const suffix of ['universal.dmg', 'x64-setup.exe', 'amd64.AppImage', 'aarch64.AppImage', 'x64-portable.zip']) {
    assert.ok(publish.includes(`_${suffix}`), suffix);
  }
  assert.match(publish, /fail_on_unmatched_files: true/);
  assert.match(publish, /make_latest: false/);
  assert.match(workflow, /createUpdaterArtifacts: false/);
  assert.match(workflow, /signingIdentity: '-'/);
  assert.doesNotMatch(workflow, /TAURI_SIGNING_PRIVATE_KEY/);
  const files = publish.split('          files: |\n')[1].trim().split('\n');
  assert.equal(files.length, 5);
  assert.doesNotMatch(files.join('\n'), /latest\.json|\.sig/);
  assert.doesNotMatch(publish, /npm publish|docker\/build-push/);
  for (const target of ['universal-apple-darwin', 'x86_64-pc-windows-msvc', 'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu']) {
    assert.ok(workflow.includes(target), target);
  }
});
