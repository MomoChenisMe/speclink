// 連線登入的共用回饋元件（design 決策三）：等待授權面與 PAT 輸入由伺服器頁籤
// 的連線列與工作區選擇器的新增並登入區共用——登入回饋跟著發起登入的介面走，
// 兩處無重複實作。credential 不經此層：PAT 僅單次過境 onSubmit。
import { useEffect, useState } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Button, CopyButton, Input, SEMANTIC_TONE, useI18n } from "@speclink/ui";

import type { LoginPhase } from "../adapter/connections";

/** 授權有效期限的剩餘時間（分:秒）：以截止時刻減現在時間計算、每秒更新，
 * 不依賴輪詢節奏（design 決策三）；歸零即停在 0:00。 */
function Countdown({ expiresAt }: { expiresAt: number }) {
  const { t } = useI18n();
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);
  const left = Math.max(0, Math.floor((expiresAt - now) / 1000));
  const time = `${Math.floor(left / 60)}:${String(left % 60).padStart(2, "0")}`;
  return (
    <span className="text-xs text-muted-foreground">
      {t("servers.timeLeft").replace("{time}", time)}
    </span>
  );
}

/** 等待面裡的一列：標籤、內容與複製鈕。裝置碼要人照著輸入，字放大並拉開字距。 */
function CopyRow({
  label,
  value,
  copyLabel,
  code = false,
}: {
  label: string;
  value: string;
  copyLabel: string;
  code?: boolean;
}) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-xs text-muted-foreground">{label}</span>
      {code ? (
        <code className="font-mono text-sm tracking-widest">{value}</code>
      ) : (
        <span className="min-w-0 truncate font-mono text-xs">{value}</span>
      )}
      <CopyButton size="sm" label={copyLabel} value={value} onCopy={writeText} />
    </div>
  );
}

/** 登入等待面（規格「device login 預設與 PAT fallback」；client-browser-login D9）：
 * 標題、一句說明、可複製的內容、倒數與取消。等待裝置核准時是裝置碼與驗證網址
 * ——使用者無需依賴已開啟的分頁即可換裝置核准；等待瀏覽器登入時是授權網址——
 * 瀏覽器沒有自動開啟時自己複製去開（網址只帶 state 與 challenge，不含任何
 * credential）。伺服器頁的連線列與工作區選擇器共用這一個元件。 */
export function LoginWaiting({
  origin,
  phase,
  onCancel,
}: {
  origin: string;
  phase: LoginPhase;
  onCancel?: () => void;
}) {
  const { t } = useI18n();
  const browser = phase.kind === "awaitingBrowser";
  return (
    <div
      className="flex flex-col gap-1.5"
      data-testid={`${browser ? "awaiting-browser" : "awaiting-approval"}-${origin}`}
    >
      <div className="text-xs font-medium">
        {t(browser ? "servers.awaitingBrowserTitle" : "servers.awaitingTitle")}
      </div>
      <p className="m-0 text-xs text-muted-foreground">
        {t(browser ? "servers.awaitingBrowserHint" : "servers.awaitingHint")}
      </p>
      {phase.kind === "awaitingBrowser" ? (
        <CopyRow
          label={t("servers.authorizationUrl")}
          value={phase.url}
          copyLabel={t("servers.copyAuthorizationUrl")}
        />
      ) : (
        <>
          <CopyRow
            code
            label={t("servers.deviceCode")}
            value={phase.userCode}
            copyLabel={t("servers.copyDeviceCode")}
          />
          <CopyRow
            label={t("servers.verificationUri")}
            value={phase.verificationUri}
            copyLabel={t("servers.copyVerificationUri")}
          />
        </>
      )}
      <div className="flex items-center gap-3">
        <Countdown expiresAt={phase.expiresAt} />
        <Button type="button" variant="outline" size="sm" onClick={onCancel}>
          {t("servers.cancelLogin")}
        </Button>
      </div>
    </div>
  );
}

/** PAT 輸入（規格 PAT fallback）：草稿只存在提交前的元件狀態，提交即清空
 * ——單次過境，不留任何拷貝。 */
export function PatLoginInput({
  error,
  onSubmit,
}: {
  error: string | null;
  onSubmit: (pat: string) => void;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState("");

  function submit() {
    const pat = draft.trim();
    if (!pat) return;
    setDraft("");
    onSubmit(pat);
  }

  return (
    <div className="flex flex-col gap-1.5">
      <p className="m-0 text-xs text-muted-foreground">{t("servers.patHint")}</p>
      <div className="flex items-center gap-1.5">
        <Input
          type="password"
          placeholder={t("servers.patPlaceholder")}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
        />
        <Button type="button" size="sm" onClick={submit}>
          {t("servers.patSubmit")}
        </Button>
      </div>
      {error && (
        <span role="alert" className={`text-xs ${SEMANTIC_TONE.danger}`}>
          {error}
        </span>
      )}
    </div>
  );
}
