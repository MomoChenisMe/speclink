#!/bin/sh
# Speclink CLI 安裝腳本（cli-distribution spec「安裝腳本一行安裝對應平台 CLI」，
# design D3／release-assets-trim D6）。偵測平台、自 npm registry 取得對應平台子套件的
# tgz、驗證 sha512 integrity 後安裝——服務沒有 Node、沒有 brew 的機器（伺服器、WSL、CI）。
# binary 的唯一來源是 npm：這裡抓的 tgz 就是 npm i -g @speclink/cli 會裝到的那一份。
#
# 用法：
#   curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh
#   sh scripts/install.sh --dry-run
#
# 環境變數：
#   SPECLINK_INSTALL_VERSION   釘選版本（0.5.0 或 v0.5.0）；未設定時查 registry 的 latest
#   SPECLINK_INSTALL_DIR       安裝目錄；預設 ~/.local/bin
#   SPECLINK_INSTALL_REGISTRY  npm registry；預設 https://registry.npmjs.org
#
# 一律先驗 integrity 再落檔：驗證在暫存目錄完成，不符即中止，安裝目錄不會出現半份
# 或損毀的 binary。Windows 沒有腳本：有圖形介面就用桌面安裝器（內含同版 CLI 並管 PATH），
# 有 Node 就 npm i -g @speclink/cli。
set -eu

REGISTRY="${SPECLINK_INSTALL_REGISTRY:-https://registry.npmjs.org}"
REGISTRY="${REGISTRY%/}"
INSTALL_DIR="${SPECLINK_INSTALL_DIR:-$HOME/.local/bin}"
BIN=speclink
SCOPE=@speclink

die() {
  echo "install.sh: $1" >&2
  exit 1
}

# integrity 也是向同一個 registry 問來的：走明文等於沒驗，所以只接受 https。
case "$REGISTRY" in
  https://*) ;;
  *) die "SPECLINK_INSTALL_REGISTRY 必須是 https:// 開頭（目前為 ${REGISTRY}）" ;;
esac

usage() {
  cat <<'USAGE'
用法：sh install.sh [--dry-run]

  --dry-run   只印出解析結果（平台、版本、下載網址、安裝目錄），不連網、不寫檔
  --help      顯示本說明

環境變數：SPECLINK_INSTALL_VERSION／SPECLINK_INSTALL_DIR／SPECLINK_INSTALL_REGISTRY
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
# 對映到 npm 平台子套件名（os-cpu），與 scripts/npm/npm-platform-package.mjs 的 TARGETS 一致。
detect_platform() {
  uname_s="$(uname -s)"
  uname_m="$(uname -m)"
  case "$uname_s" in
    Darwin)
      case "$uname_m" in
        arm64|aarch64) echo darwin-arm64 ;;
        x86_64) echo darwin-x64 ;;
        *) die "macOS 上不支援的架構：$uname_m" ;;
      esac
      ;;
    Linux)
      case "$uname_m" in
        x86_64|amd64) echo linux-x64 ;;
        aarch64|arm64) echo linux-arm64 ;;
        *) die "Linux 上不支援的架構：$uname_m" ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows*)
      die "此腳本支援 macOS 與 Linux；Windows 請執行 npm i -g @speclink/cli，或安裝桌面版 Speclink_<版本>_x64-setup.exe（內含 CLI）"
      ;;
    *)
      die "不支援的作業系統：$uname_s"
      ;;
  esac
}

PLATFORM="$(detect_platform)"
PKG="${SCOPE}/cli-${PLATFORM}"

# registry 的 tgz 路徑：<registry>/@scope/name/-/name-<版本>.tgz（檔名不帶 scope）。
tgz_url() {
  echo "${REGISTRY}/${PKG}/-/cli-${PLATFORM}-$1.tgz"
}

# --- 版本解析 ---
#
# 釘選值接受帶或不帶 v 前綴，去前綴後必須是 X.Y.Z（可帶 -pre 尾）——版本會拼進網址
# 與暫存檔名，不合形的值（例如帶 ../）不能往下走。dry-run 不連網，未釘選時只標示為
# latest 並在網址中保留佔位。
VERSION="${SPECLINK_INSTALL_VERSION:-}"
VERSION="${VERSION#v}"
if [ -n "$VERSION" ]; then
  printf '%s' "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$' \
    || die "SPECLINK_INSTALL_VERSION 必須是 X.Y.Z（可帶 v 前綴），目前為 $VERSION"
fi
if [ "$dry_run" -eq 1 ]; then
  if [ -z "$VERSION" ]; then
    version_label="latest（安裝時查詢 ${REGISTRY}/${SCOPE}/cli/latest）"
    version_in_url="<version>"
  else
    version_label="$VERSION"
    version_in_url="$VERSION"
  fi
  cat <<EOF
平台：      $PLATFORM
套件：      $PKG
版本：      $version_label
下載網址：  $(tgz_url "$version_in_url")
安裝目錄：  $INSTALL_DIR
EOF
  exit 0
fi

command -v curl >/dev/null 2>&1 || die "找不到 curl，請先安裝後重試"
command -v openssl >/dev/null 2>&1 || die "找不到 openssl，無法驗證 integrity；請先安裝後重試"

# 取 JSON 裡第一個 "<key>":"<value>"——registry 的回應是單行 JSON，用第一個出現的頂層鍵。
json_field() {
  grep -o "\"$1\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" | head -n 1 | sed 's/.*:[[:space:]]*"\(.*\)"$/\1/'
}

if [ -z "$VERSION" ]; then
  latest="${REGISTRY}/${SCOPE}/cli/latest"
  body="$(curl -fsSL "$latest")" || die "查詢最新版本失敗：$latest"
  VERSION="$(printf '%s' "$body" | json_field version)"
  [ -n "$VERSION" ] || die "無法從 $latest 的回應解析出 version"
fi

TGZ_URL="$(tgz_url "$VERSION")"
TGZ_NAME="cli-${PLATFORM}-${VERSION}.tgz"

# --- 下載與驗證 ---
#
# 全程在暫存目錄進行；trap 涵蓋正常結束與中斷，不留殘檔——包括落檔階段寫在安裝目錄
# 的暫存名檔（chmod／mv 失敗或中途 Ctrl-C 時它會留成一個隱藏檔）。
tmp="$(mktemp -d)"
staging=""
# 明確把離開碼帶過 trap：只寫 rm 的話，清理指令的成功狀態會蓋掉失敗的離開碼，
# 讓 integrity 不符這類中止對呼叫端看起來像成功。
cleanup() {
  rc=$?
  rm -rf "$tmp"
  if [ -n "$staging" ]; then rm -f "$staging"; fi
  exit "$rc"
}
trap cleanup EXIT INT TERM

echo "下載 ${PKG}@${VERSION}…"
manifest_url="${REGISTRY}/${PKG}/${VERSION}"
manifest="$(curl -fsSL "$manifest_url")" || die "查詢 ${PKG}@${VERSION} 失敗：$manifest_url"
expected="$(printf '%s' "$manifest" | json_field integrity)"
case "$expected" in
  sha512-*) ;;
  *) die "registry 未提供 ${PKG}@${VERSION} 的 sha512 integrity，已中止安裝" ;;
esac

curl -fsSL -o "$tmp/$TGZ_NAME" "$TGZ_URL" || die "下載失敗：$TGZ_URL"

actual="sha512-$(openssl dgst -sha512 -binary "$tmp/$TGZ_NAME" | openssl base64 -A)"
[ "$actual" = "$expected" ] || die "integrity 不符（預期 ${expected}，實得 ${actual}），已中止安裝"

# --- 安裝 ---
#
# npm 的 tgz 內容在 package/ 之下；驗證通過才碰安裝目錄——失敗路徑不會在此留下任何檔案。
mkdir -p "$tmp/extract"
tar xzf "$tmp/$TGZ_NAME" -C "$tmp/extract" --strip-components=1 "package/$BIN" || die "解壓 $TGZ_NAME 失敗"
[ -f "$tmp/extract/$BIN" ] || die "tgz 中找不到 package/$BIN"

# 先寫到同目錄暫存名再 mv 覆蓋：安裝目錄裡可能是桌面 app 留下的 symlink（macOS 指向
# app bundle 內的 CLI），cp 會寫穿到它指的檔；mv 換掉的是連結本身。
mkdir -p "$INSTALL_DIR"
staging="$INSTALL_DIR/.$BIN.tmp.$$"
cp "$tmp/extract/$BIN" "$staging"
chmod 755 "$staging"
mv -f "$staging" "$INSTALL_DIR/$BIN"

echo "已安裝 $BIN $VERSION 至 $INSTALL_DIR/$BIN"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *) echo "提醒：$INSTALL_DIR 不在 PATH 中，請將它加入 shell 設定檔後重開終端機" ;;
esac
