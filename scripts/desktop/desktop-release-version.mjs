import { appendFileSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function desktopReleaseVersion(tag, appVersion, cliVersion) {
  const match = /^desktop-v((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*))$/.exec(tag);
  if (!match) throw new Error('桌面發布 Tag 必須使用 desktop-vX.Y.Z，例如 desktop-v0.9.0');
  const version = match[1];
  if (appVersion !== version) throw new Error(`App 版本 ${appVersion} 與 Tag 版本 ${version} 不一致`);
  if (cliVersion !== version) throw new Error(`CLI 版本 ${cliVersion} 與 Tag 版本 ${version} 不一致`);
  return version;
}

if (resolve(process.argv[1] ?? '') === fileURLToPath(import.meta.url)) {
  try {
    const app = JSON.parse(readFileSync('apps/desktop/src-tauri/tauri.conf.json', 'utf8'));
    const cargo = readFileSync('Cargo.toml', 'utf8');
    const workspace = cargo.match(/\[workspace\.package\]([\s\S]*?)(?=\n\[|$)/)?.[1];
    const cliVersion = workspace?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
    const version = process.env.GITHUB_EVENT_NAME === 'push'
      ? desktopReleaseVersion(process.env.GITHUB_REF_NAME ?? '', app.version, cliVersion)
      : app.version;
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `version=${version}\n`);
    console.log(`Desktop version: ${version}`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
