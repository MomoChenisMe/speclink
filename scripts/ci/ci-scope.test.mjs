import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { requiresFullCI } from './ci-scope.mjs';

test('純 Desktop 發布設定與文件只跑設定檢查，混入任何程式碼就跑完整 CI', () => {
  const release = ['.github/workflows/desktop-build.yml', 'scripts/desktop/desktop-release-version.mjs', 'docs/development.md'];
  assert.equal(requiresFullCI(release), false);
  assert.equal(requiresFullCI(['.github/workflows/ci.yml', 'scripts/ci/ci-scope.mjs', 'scripts/ci/ci-scope.test.mjs']), false);
  for (const file of ['apps/server-web/src/main.tsx', 'apps/desktop/src/App.tsx', 'crates/engine/speclink-core/src/lib.rs', 'Cargo.lock', 'package.json', '.github/workflows/release.yml', 'scripts/desktop/desktop-sidecar.mjs', 'unknown.txt']) {
    assert.equal(requiresFullCI([...release, file]), true, file);
  }
  assert.equal(requiresFullCI([]), true, '沒有變更資訊時保守執行完整 CI');
});

test('CI 始終跑設定檢查，四個昂貴工作共用同一個範圍判斷', () => {
  const ci = readFileSync(new URL('../../.github/workflows/ci.yml', import.meta.url), 'utf8');
  const sections = [...ci.matchAll(/^  ([a-z0-9-]+):\n([\s\S]*?)(?=^  [a-z0-9-]+:|$(?![\s\S]))/gm)];
  const scope = sections.find(([, name]) => name === 'changes')?.[2];
  assert.ok(scope);
  assert.doesNotMatch(scope, /\n    if:/);
  assert.match(scope, /fetch-depth: 0/);
  assert.match(scope, /github\.event\.before/);
  assert.match(scope, /github\.event\.pull_request\.base\.sha/);
  assert.match(scope, /node scripts\/ci\/ci-scope\.mjs/);
  assert.match(scope, /shell: bash\n\s+run: node --test scripts\/\*\.test\.mjs scripts\/\*\/\*\.test\.mjs/);
  for (const name of ['build-and-smoke', 'phase3-e2e', 'docker-smoke', 'postgres-store']) {
    const body = sections.find(([, key]) => key === name)?.[2];
    assert.match(body, /needs: \[changes\]/, name);
    assert.match(body, /if: needs\.changes\.outputs\.full == 'true'/, name);
  }
});

test('實際 Git diff：push／PR 的發布變更可略過，程式碼、重新命名及未知比較維持完整 CI', (t) => {
  const root = mkdtempSync(join(tmpdir(), 'speclink-ci-scope-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const git = (...args) => execFileSync('git', ['-c', 'user.name=CI', '-c', 'user.email=ci@example.com', '-c', 'commit.gpgsign=false', ...args], { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  const write = (file, text) => {
    mkdirSync(dirname(join(root, file)), { recursive: true });
    writeFileSync(join(root, file), text);
  };
  const commit = () => { git('add', '.'); git('commit', '-qm', 'fixture'); return git('rev-parse', 'HEAD'); };
  git('init', '-q');
  write('docs/development.md', 'first');
  const base = commit();
  write('docs/development.md', 'release instructions');
  write('.github/workflows/desktop-build.yml', 'workflow');
  const release = commit();
  write('apps/desktop/src/App.tsx', 'runtime');
  renameSync(join(root, 'docs/development.md'), join(root, 'apps/desktop/src/moved.md'));
  const runtime = commit();
  const script = fileURLToPath(new URL('./ci-scope.mjs', import.meta.url));
  let index = 0;
  for (const [event, before, head, expected] of [
    ['push', base, release, false], ['pull_request', base, release, false],
    ['push', release, runtime, true], ['pull_request', release, runtime, true],
    ['push', '0'.repeat(40), runtime, true], ['push', runtime, runtime, true],
    ['push', 'missing-commit', runtime, true],
  ]) {
    const output = join(root, `output-${index++}`);
    const result = spawnSync(process.execPath, [script], {
      cwd: root,
      env: { ...process.env, GITHUB_EVENT_NAME: event, BASE_SHA: before, HEAD_SHA: head, GITHUB_OUTPUT: output },
      encoding: 'utf8',
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(readFileSync(output, 'utf8'), `full=${expected}\n`, `${event}: ${before} → ${head}`);
  }
});
