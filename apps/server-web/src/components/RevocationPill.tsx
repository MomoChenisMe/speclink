import { StatusPill, useI18n } from "@speclink/ui";

/** 憑證與登入工作階段的狀態籤：有效為綠系、已撤銷為中性（撤銷不可逆，得一眼看出哪一把還活著）。 */
export function RevocationPill({ revokedAt }: { revokedAt: string | null }) {
  const { t } = useI18n();
  return (
    <StatusPill tone={revokedAt ? "neutral" : "success"}>
      {revokedAt ? t("common.revoked") : t("common.active")}
    </StatusPill>
  );
}
