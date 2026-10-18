import { execFileSync } from 'node:child_process';
import { setTimeout } from 'node:timers/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Only the latest push CI for the tagged commit on the default branch can admit a build.
export async function waitForCI(
  { repository, sha, branch },
  listRuns = (args) => JSON.parse(execFileSync('gh', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] })),
  pause = setTimeout,
) {
  if (!repository || !/^[a-f0-9]{40}$/.test(sha ?? '') || !branch) {
    throw new Error('缺少有效的 CI 執行環境：repository、commit SHA 與主分支皆為必要欄位');
  }
  const args = ['run', 'list', '--repo', repository, '--workflow', 'ci.yml', '--branch', branch, '--commit', sha,
    '--event', 'push', '--limit', '1', '--json', 'status,conclusion,headSha,url'];
  for (let attempt = 0; attempt <= 80; attempt++) {
    const [run] = await listRuns(args);
    if (run) {
      if (run.headSha !== sha) throw new Error('CI 的 commit SHA 與建置目標不一致，拒絕建置');
      if (run.status === 'completed') {
        if (run.conclusion === 'success') return run;
        throw new Error(`主 CI 未通過（${run.conclusion}），拒絕桌面建置：${run.url}`);
      }
    }
    if (attempt < 80) await pause(30_000);
  }
  throw new Error(`40 分鐘內未取得同一 SHA 的成功主 CI，拒絕桌面建置：${sha}`);
}

if (resolve(process.argv[1] ?? '') === fileURLToPath(import.meta.url)) {
  console.log(`等候主分支 CI 通過：${process.env.GITHUB_SHA}（最多 40 分鐘）`);
  try {
    const run = await waitForCI({ repository: process.env.GITHUB_REPOSITORY, sha: process.env.GITHUB_SHA, branch: process.env.CI_BRANCH });
    console.log(`主 CI 通過，允許桌面建置：${run.url}`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
