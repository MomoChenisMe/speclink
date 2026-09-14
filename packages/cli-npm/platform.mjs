// @speclink/cli 的平台對映與 binary 定位（cli-distribution spec「CLI 以 npm 套件發布」；
// release-assets-trim 設計 D4）。shim（bin/speclink）與 postinstall 共用：兩者都以自身
// 位置往上找 node_modules 解析平台子套件，與 npm 實際安裝的佈局同一條路。
import { createRequire } from 'node:module';
import path from 'node:path';

const SCOPE = '@speclink';

/** 五個平台子套件（os-cpu），與 scripts/npm/npm-platform-package.mjs 的 TARGETS 同序。 */
export const SUPPORTED_PLATFORMS = ['darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64', 'win32-x64'];

/** os/cpu → 平台子套件名；無對應（未支援平台）回 null。 */
export function platformPackage(platform, arch) {
  const key = `${platform}-${arch}`;
  return SUPPORTED_PLATFORMS.includes(key) ? `${SCOPE}/cli-${key}` : null;
}

export function binaryName(platform) {
  return platform === 'win32' ? 'speclink.exe' : 'speclink';
}

/** 自 fromUrl（shim 或 postinstall 所在檔）解析對應子套件內 binary 的絕對路徑；
 * 平台不支援、或子套件未安裝（optionalDependencies 被略過）回 null。 */
export function resolveBinary(fromUrl, platform, arch) {
  const pkg = platformPackage(platform, arch);
  if (!pkg) return null;
  try {
    const pkgJson = createRequire(fromUrl).resolve(`${pkg}/package.json`);
    return path.join(path.dirname(pkgJson), binaryName(platform));
  } catch {
    return null;
  }
}
