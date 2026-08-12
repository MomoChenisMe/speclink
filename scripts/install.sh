#!/bin/sh
# Speclink CLI 安裝腳本（cli-distribution spec「安裝腳本一行安裝對應平台 CLI」，
# design D3）。偵測平台、取得對應的 GitHub Release 壓縮檔、驗證 SHA-256 後安裝。
#
# 用法：
#   curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh
#   sh scripts/install.sh --dry-run
#
# 環境變數：
#   SPECLINK_INSTALL_VERSION  釘選版本（如 v0.1.0）；未設定時查詢最新 Release
#   SPECLINK_INSTALL_DIR      安裝目錄；預設 ~/.local/bin
#   SPECLINK_INSTALL_REPO     來源 repo；預設 MomoChenisMe/speclink
#
# 一律先驗 checksum 再落檔：驗證在暫存目錄完成，不符即中止，安裝目錄不會出現半份
# 或損毀的 binary。Windows 請改用同目錄的 install.ps1。
set -eu

REPO="${SPECLINK_INSTALL_REPO:-MomoChenisMe/speclink}"
INSTALL_DIR="${SPECLINK_INSTALL_DIR:-$HOME/.local/bin}"
BIN=speclink

die() {
  echo "install.sh: $1" >&2
  exit 1
}

usage() {
  cat <<'USAGE'
用法：sh install.sh [--dry-run]

  --dry-run   只印出解析結果（平台、版本、下載網址、安裝目錄），不連網、不寫檔
  --help      顯示本說明

環境變數：SPECLINK_INSTALL_VERSION／SPECLINK_INSTALL_DIR／SPECLINK_INSTALL_REPO
USAGE
}

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    --help|-h) usage; exit 0 ;;
    *) die "未知參數：$arg" ;;
  esac
done

# --- 平台偵測 ---
#
# 對映到 release 管線建置的五個 target triple；資產命名與其 Package 步驟一致。
detect_target() {
  uname_s="$(uname -s)"
  uname_m="$(uname -m)"
  case "$uname_s" in
    Darwin)
      case "$uname_m" in
        arm64|aarch64) echo aarch64-apple-darwin ;;
        x86_64) echo x86_64-apple-darwin ;;
        *) die "macOS 上不支援的架構：$uname_m" ;;
      esac
      ;;
    Linux)
      case "$uname_m" in
        x86_64|amd64) echo x86_64-unknown-linux-gnu ;;
        aarch64|arm64) echo aarch64-unknown-linux-gnu ;;
        *) die "Linux 上不支援的架構：$uname_m" ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows*)
      die "此腳本支援 macOS 與 Linux；Windows 請改用 install.ps1"
      ;;
    *)
      die "不支援的作業系統：$uname_s"
      ;;
  esac
}

TARGET="$(detect_target)"

# --- 版本解析 ---
#
# dry-run 不連網，因此未釘選版本時只標示為 latest 並在網址中保留佔位，不去查 API。
VERSION="${SPECLINK_INSTALL_VERSION:-}"
if [ "$dry_run" -eq 1 ]; then
  if [ -z "$VERSION" ]; then
    version_label="latest（安裝時查詢 GitHub API）"
    version_in_url="<version>"
  else
    version_label="$VERSION"
    version_in_url="$VERSION"
  fi
  asset="${BIN}-${version_in_url}-${TARGET}.tar.gz"
  cat <<EOF
平台：      $TARGET
版本：      $version_label
資產：      $asset
下載網址：  https://github.com/${REPO}/releases/download/${version_in_url}/${asset}
安裝目錄：  $INSTALL_DIR
EOF
  exit 0
fi

command -v curl >/dev/null 2>&1 || die "找不到 curl，請先安裝後重試"

if [ -z "$VERSION" ]; then
  api="https://api.github.com/repos/${REPO}/releases/latest"
  body="$(curl -fsSL "$api")" || die "查詢最新 Release 失敗：$api"
  VERSION="$(printf '%s' "$body" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
  [ -n "$VERSION" ] || die "無法從 $api 的回應解析出 tag_name"
fi

ASSET="${BIN}-${VERSION}-${TARGET}.tar.gz"
BASE="https://github.com/${REPO}/releases/download/${VERSION}"

# --- 下載與驗證 ---
#
# 全程在暫存目錄進行；trap 涵蓋正常結束與中斷，不留殘檔。
tmp="$(mktemp -d)"
# 明確把離開碼帶過 trap：只寫 rm 的話，清理指令的成功狀態會蓋掉失敗的離開碼，
# 讓 checksum 不符這類中止對呼叫端看起來像成功。
cleanup() {
  rc=$?
  rm -rf "$tmp"
  exit "$rc"
}
trap cleanup EXIT INT TERM

echo "下載 ${ASSET}（${VERSION}）…"
curl -fsSL -o "$tmp/$ASSET" "${BASE}/${ASSET}" || die "下載失敗：${BASE}/${ASSET}"
curl -fsSL -o "$tmp/SHA256SUMS.txt" "${BASE}/SHA256SUMS.txt" || die "下載 SHA256SUMS.txt 失敗"

expected="$(awk -v name="$ASSET" '$2 == name { print $1 }' "$tmp/SHA256SUMS.txt")"
[ -n "$expected" ] || die "SHA256SUMS.txt 中找不到 $ASSET 的條目"

if command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "$tmp/$ASSET" | awk '{ print $1 }')"
elif command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$tmp/$ASSET" | awk '{ print $1 }')"
else
  die "找不到 shasum 或 sha256sum，無法驗證 checksum"
fi

[ "$actual" = "$expected" ] || die "checksum 不符（預期 ${expected}，實得 ${actual}），已中止安裝"

# --- 安裝 ---
#
# 驗證通過才碰安裝目錄——失敗路徑不會在此留下任何檔案。
mkdir -p "$tmp/extract"
tar xzf "$tmp/$ASSET" -C "$tmp/extract" || die "解壓 $ASSET 失敗"
[ -f "$tmp/extract/$BIN" ] || die "壓縮檔中找不到 $BIN"

mkdir -p "$INSTALL_DIR"
cp "$tmp/extract/$BIN" "$INSTALL_DIR/$BIN"
chmod 755 "$INSTALL_DIR/$BIN"

echo "已安裝 $BIN $VERSION 至 $INSTALL_DIR/$BIN"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *) echo "提醒：$INSTALL_DIR 不在 PATH 中，請將它加入 shell 設定檔後重開終端機" ;;
esac
