// OS 簽章 secrets 的前置閘門（desktop-release spec「OS 程式碼簽章為可插鑰匙開關」，
// design D1：部分設定 fail-closed）。
//
// 用法：node scripts/signing-gate.mjs
//
// 輸入為各 secret 的環境變數，只讀「是否非空」，永不讀取或輸出其值。每一組 secrets
// 是全有全無：全無即該路徑跳過、workflow 照常全綠；全有即啟用；部分存在則以列出
// 缺項名稱的錯誤非零結束——「已簽章但未公證」對使用者與未簽章幾乎等價，卻讓維護者
// 誤以為已完成，因此視為設定錯誤而非合法中間態。
//
// 決策寫入 $GITHUB_ENV 供後續步驟以 env.<鍵> 判斷，不讓各步驟各自重推：
//   SPECLINK_MACOS_SIGNING=full|none
//   SPECLINK_WINDOWS_SIGNING=signpath|certificate|none
import { appendFileSync } from 'node:fs';

// macOS 的憑證半組與公證半組合為一組：只簽章不公證的產物照樣被 Gatekeeper 攔。
const MACOS_GROUP = {
  label: 'macOS 簽章與公證',
  names: [
    'APPLE_CERTIFICATE',
    'APPLE_CERTIFICATE_PASSWORD',
    'APPLE_SIGNING_IDENTITY',
    'APPLE_ID',
    'APPLE_PASSWORD',
    'APPLE_TEAM_ID',
  ],
};

const SIGNPATH_GROUP = {
  label: 'Windows SignPath 簽章',
  names: [
    'SIGNPATH_API_TOKEN',
    'SIGNPATH_ORGANIZATION_ID',
    'SIGNPATH_PROJECT_SLUG',
    'SIGNPATH_POLICY_SLUG',
  ],
};

const WINDOWS_CERT_GROUP = {
  label: 'Windows 本機憑證簽章',
  names: ['WINDOWS_CERTIFICATE', 'WINDOWS_CERTIFICATE_PASSWORD'],
};

/// 判斷一組 secrets 的狀態：'full'、'none'，或列出缺項的 'partial'。
/// 只看 trim 後是否非空——填了空白字元等同未設定，落在安全的一側（跳過而非半套簽章）。
function classify(group) {
  const missing = group.names.filter((name) => (process.env[name] ?? '').trim() === '');
  if (missing.length === 0) return { state: 'full' };
  if (missing.length === group.names.length) return { state: 'none' };
  return { state: 'partial', missing };
}

const groups = [MACOS_GROUP, SIGNPATH_GROUP, WINDOWS_CERT_GROUP].map((group) => ({
  group,
  result: classify(group),
}));

const partial = groups.filter(({ result }) => result.state === 'partial');
if (partial.length > 0) {
  for (const { group, result } of partial) {
    console.error(
      `::error::簽章組「${group.label}」設定不完整，缺少：${result.missing.join('、')}` +
        '——請補齊該組全部 secrets，或整組移除以產出未簽章安裝檔',
    );
  }
  process.exit(1);
}

const state = Object.fromEntries(groups.map(({ group, result }) => [group.label, result.state]));
const macos = state[MACOS_GROUP.label] === 'full' ? 'full' : 'none';
// SignPath 優先於本機憑證：兩者皆備時走服務簽章，本機憑證維持後備路徑。
const windows =
  state[SIGNPATH_GROUP.label] === 'full'
    ? 'signpath'
    : state[WINDOWS_CERT_GROUP.label] === 'full'
      ? 'certificate'
      : 'none';

console.log(`macOS 簽章：${macos}／Windows 簽章：${windows}`);

if (process.env.GITHUB_ENV) {
  appendFileSync(
    process.env.GITHUB_ENV,
    `SPECLINK_MACOS_SIGNING=${macos}\nSPECLINK_WINDOWS_SIGNING=${windows}\n`,
  );
}
