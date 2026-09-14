// 由 CLI npm 發布產出的 tgz 校驗清單產生 Homebrew formula（cli-distribution spec
// 「Homebrew formula 產生器」，design D4／release-assets-trim D5：binary 的唯一來源是
// npm，formula 的 url 指向 registry.npmjs.org 的平台子套件 tgz——homebrew-core 對
// Node CLI 的既有寫法）。
//
// 用法：node scripts/release/homebrew-formula.mjs --tag v0.5.0 --sums cli-npm-sums.txt
//
// --sums 是 cli-npm-publish job 在發布並等 registry 可見後，對自 registry 取回的 tgz
// （npm pack <name>@<version>，拿到的是 registry 原 bytes）跑 sha256sum 的輸出（digest、
// 兩空白、檔名），所以 sha256 對 registry 上的檔成立——重跑時已上架的同版被跳過也一樣。
// 輸出到 stdout，貼進 tap repo 的 Formula/speclink.rb 即可。四組平台資產
// 缺任一即以非零結束——checksum 每版都變，手抄或漏抄要到使用者 brew install 失敗
// 才會發現，因此寧可不產出也不產出半套。
import { readFileSync } from 'node:fs';

const REPO = 'MomoChenisMe/speclink';
const SCOPE = '@speclink';
const BIN = 'speclink';

// brew 只用得到這四個；win32 子套件不在 formula 範圍。
const TARGETS = [
  { key: 'macosArm', platform: 'darwin-arm64' },
  { key: 'macosIntel', platform: 'darwin-x64' },
  { key: 'linuxArm', platform: 'linux-arm64' },
  { key: 'linuxIntel', platform: 'linux-x64' },
];

/// npm pack 的產物名：@speclink/cli-darwin-arm64@0.5.0 → speclink-cli-darwin-arm64-0.5.0.tgz。
const packName = (platform, version) => `${SCOPE.slice(1)}-cli-${platform}-${version}.tgz`;
/// registry 上該 tgz 的下載網址（registry 的檔名不帶 scope）。
const registryUrl = (platform, version) =>
  `https://registry.npmjs.org/${SCOPE}/cli-${platform}/-/cli-${platform}-${version}.tgz`;

function fail(message) {
  console.error(`homebrew-formula: ${message}`);
  process.exit(1);
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i];
    const value = argv[i + 1];
    if (!/^--(tag|sums)$/.test(flag) || value === undefined) {
      fail(`無法解析的參數：${flag}——用法：--tag <v0.5.0> --sums <cli-npm-sums.txt>`);
    }
    args[flag.slice(2)] = value;
  }
  for (const required of ['tag', 'sums']) {
    if (!args[required]) fail(`缺少必要參數 --${required}`);
  }
  return args;
}

const { tag, sums } = parseArgs(process.argv.slice(2));

let sumsText;
try {
  sumsText = readFileSync(sums, 'utf8');
} catch {
  fail(`讀不到校驗清單：${sums}`);
}

const version = tag.replace(/^v/, '');

/// 校驗清單的每一行是「digest<空白>檔名」。以完整檔名（含版本）比對：版本不符的
/// 清單等同缺條目，不會把上一版的 sha256 套到這一版。
const digests = new Map();
for (const line of sumsText.split('\n')) {
  const match = line.trim().match(/^([0-9a-fA-F]{64})\s+(\S+)$/);
  if (match) digests.set(match[2], match[1].toLowerCase());
}

const assets = [];
const missing = [];
for (const { key, platform } of TARGETS) {
  const digest = digests.get(packName(platform, version));
  if (!digest) {
    missing.push(platform);
    continue;
  }
  assets.push({ key, platform, digest, url: registryUrl(platform, version) });
}

if (missing.length > 0) {
  fail(
    `校驗清單缺少下列平台的 ${BIN} tgz 條目：${missing.join('、')}` +
      `——確認 tag（${tag}）與 cli-npm-publish 產出的清單同版後重試`,
  );
}

const byKey = Object.fromEntries(assets.map((a) => [a.key, a]));

process.stdout.write(`# 由 scripts/release/homebrew-formula.mjs 產生，請勿手改；改版時重新產生。
class Speclink < Formula
  desc "Spec-Driven Development engine and toolkit"
  homepage "https://github.com/${REPO}"
  version "${version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "${byKey.macosArm.url}"
      sha256 "${byKey.macosArm.digest}"
    else
      url "${byKey.macosIntel.url}"
      sha256 "${byKey.macosIntel.digest}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "${byKey.linuxArm.url}"
      sha256 "${byKey.linuxArm.digest}"
    else
      url "${byKey.linuxIntel.url}"
      sha256 "${byKey.linuxIntel.digest}"
    end
  end

  def install
    bin.install "${BIN}"
  end

  test do
    assert_match "${version}", shell_output("#{bin}/${BIN} --version")
  end
end
`);
