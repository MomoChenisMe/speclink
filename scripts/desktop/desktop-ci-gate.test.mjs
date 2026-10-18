import { test } from 'node:test';
import assert from 'node:assert/strict';
import { waitForCI } from './desktop-ci-gate.mjs';

const target = { repository: 'mosutw/speclink', sha: 'a'.repeat(40), branch: 'main' };
const run = (status, conclusion = '') => ({ status, conclusion, headSha: target.sha, url: 'https://github.com/mosutw/speclink/actions/runs/1' });

test('只查指定 repository、主分支、同一 SHA 的最新 push CI，不以 PR 或其他工作取代', async () => {
  let args;
  const result = await waitForCI(target, (values) => {
    args = values;
    return [run('completed', 'success')];
  }, () => assert.fail('已通過的 CI 不需等待'));
  for (const [flag, value] of [['--repo', target.repository], ['--workflow', 'ci.yml'], ['--branch', 'main'], ['--commit', target.sha], ['--event', 'push'], ['--limit', '1']]) {
    assert.equal(args[args.indexOf(flag) + 1], value);
  }
  assert.equal(result.conclusion, 'success');
});

test('CI 尚未出現、排隊或執行中時等待，成功才放行', async () => {
  const responses = [[], [run('queued')], [run('in_progress')], [run('completed', 'success')]];
  const waits = [];
  const result = await waitForCI(target, () => responses.shift(), (ms) => waits.push(ms));
  assert.equal(result.conclusion, 'success');
  assert.deepEqual(waits, [30_000, 30_000, 30_000]);
});

for (const conclusion of ['failure', 'cancelled', 'timed_out', 'skipped', 'neutral', 'action_required', 'startup_failure', 'stale']) {
  test(`最新 CI 為 ${conclusion} 時拒絕建置，不採更早的成功結果`, async () => {
    await assert.rejects(waitForCI(target, () => [run('completed', conclusion), run('completed', 'success')], () => assert.fail('已失敗不應繼續等待')), new RegExp(conclusion));
  });
}

test('SHA 不符、讀取失敗及缺少執行環境都不放行', async () => {
  await assert.rejects(waitForCI(target, () => [{ ...run('completed', 'success'), headSha: 'b'.repeat(40) }]), /SHA/);
  await assert.rejects(waitForCI(target, () => { throw new Error('GitHub API unavailable'); }), /GitHub API unavailable/);
  await assert.rejects(waitForCI({ ...target, sha: '' }, () => assert.fail('參數不完整不能查詢')), /執行環境/);
});

test('找不到 CI 或 CI 持續未完成，最多等 40 分鐘後拒絕建置', async () => {
  for (const response of [[], [run('in_progress')]]) {
    let calls = 0;
    let waits = 0;
    await assert.rejects(waitForCI(target, () => { calls++; return response; }, () => { waits++; }), /40 分鐘/);
    assert.equal(calls, 81);
    assert.equal(waits, 80);
  }
});
