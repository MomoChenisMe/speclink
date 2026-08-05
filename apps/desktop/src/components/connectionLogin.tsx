// 連線登入的共用回饋元件（design 決策三）：等待授權面與 PAT 輸入由伺服器頁籤
// 的連線列與工作區選擇器的新增並登入區共用——登入回饋跟著發起登入的介面走，
// 兩處無重複實作。credential 不經此層：PAT 僅單次過境 onSubmit。
import { useEffect, useState } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Button, Input, SEMANTIC_TONE, useI18n } from "@speclink/ui";

import type { ConnectionPhase } from "../store";

/** 複製鈕：沿用系統匣複製 slug 的語彙——寫剪貼簿（Rust 端外掛）、失敗靜默，
 * 成功後短暫顯示已複製。 */
function CopyButton({ label, value }: { label: string; value: string }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);
  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      aria-label={label}
      onClick={() => {
        void writeText(value).catch(() => {});
        setCopied(true);
        setTimeout(() => setCopied(false), 1200);
      }}
    >
      {copied ? t("servers.copied") : t("servers.copy")}
    </Button>
  );
}

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

/** 等待授權面（規格「device login 預設與 PAT fallback」）：裝置碼與驗證網址
 * 各附複製、倒數與取消——使用者無需依賴已開啟的分頁即可換裝置核准。 */
export function AwaitingApproval({
  origin,
  phase,
  onCancel,
}: {
  origin: string;
  phase: Extract<ConnectionPhase, { kind: "awaitingApproval" }>;
  onCancel?: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="flex flex-col gap-1.5" data-testid={`awaiting-approval-${origin}`}>
      <div className="text-xs font-medium">{t("servers.awaitingTitle")}</div>
      <p className="m-0 text-xs text-muted-foreground">{t("servers.awaitingHint")}</p>
      <div className="flex items-center gap-1.5">
        <span className="text-xs text-muted-foreground">{t("servers.deviceCode")}</span>
        <code className="font-mono text-sm tracking-widest">{phase.userCode}</code>
        <CopyButton label={t("servers.copyDeviceCode")} value={phase.userCode} />
      </div>
      <div className="flex items-center gap-1.5">
        <span className="text-xs text-muted-foreground">{t("servers.verificationUri")}</span>
        <span className="min-w-0 truncate font-mono text-xs">{phase.verificationUri}</span>
        <CopyButton label={t("servers.copyVerificationUri")} value={phase.verificationUri} />
      </div>
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
