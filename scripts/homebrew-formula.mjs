// 由 Release 的 SHA256SUMS.txt 產生 Homebrew formula（cli-distribution spec
// 「Homebrew formula 產生器」，design D4：產生器產出、tap repo 手動維護）。
//
// 用法：node scripts/homebrew-formula.mjs --tag v0.1.0 --sums SHA256SUMS.txt
//
// 輸出到 stdout，貼進 tap repo 的 Formula/speclink.rb 即可。四組平台資產缺任一
// 即以非零結束——checksum 每版都變，手抄或漏抄要到使用者 brew install 失敗才會
// 發現，因此寧可不產出也不產出半套。
import { readFileSync } from 'node:fs';

const REPO = 'MomoChenisMe/speclink';
const BIN = 'speclink';

// brew 只用得到這四個；Windows 的 msvc 資產不在 formula 範圍。
const TARGETS = [
  { key: 'macosArm', triple: 'aarch64-apple-darwin' },
  { key: 'macosIntel', triple: 'x86_64-apple-darwin' },
  { key: 'linuxArm', triple: 'aarch64-unknown-linux-gnu' },
  { key: 'linuxIntel', triple: 'x86_64-unknown-linux-gnu' },
];

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
      fail(`無法解析的參數：${flag}——用法：--tag <v0.1.0> --sums <SHA256SUMS.txt>`);
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
  fail(`讀不到 SHA256SUMS.txt：${sums}`);
}

/// SHA256SUMS.txt 的每一行是「digest<空白>檔名」。以完整檔名比對，避免
/// speclink-server-<tag>-<triple>.tar.gz 被當成 CLI 的資產取走。
const digests = new Map();
for (const line of sumsText.split('\n')) {
  const match = line.trim().match(/^([0-9a-fA-F]{64})\s+(\S+)$/);
  if (match) digests.set(match[2], match[1].toLowerCase());
}

const assets = [];
const missing = [];
for (const { key, triple } of TARGETS) {
  const name = `${BIN}-${tag}-${triple}.tar.gz`;
  const digest = digests.get(name);
  if (!digest) {
    missing.push(triple);
    continue;
  }
  assets.push({ key, triple, digest, url: `https://github.com/${REPO}/releases/download/${tag}/${name}` });
}

if (missing.length > 0) {
  fail(
    `SHA256SUMS.txt 缺少下列平台的 ${BIN} 條目：${missing.join('、')}` +
      `——確認 tag（${tag}）與該次 Release 的資產一致後重試`,
  );
}

const byKey = Object.fromEntries(assets.map((a) => [a.key, a]));
const version = tag.replace(/^v/, '');

process.stdout.write(`# 由 scripts/homebrew-formula.mjs 產生，請勿手改；改版時重新產生。
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
