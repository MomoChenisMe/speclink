import { Button, useI18n } from "@speclink/ui";

/** 撤銷鈕（server-web-list-pages design D8）：存取金鑰與裝置登入列上滑過才出現。顯隱只用不透明度——
 *  隱藏期間仍在 Tab 順序內、螢幕閱讀器讀得到；鍵盤聚焦時出現；沒有滑鼠移入能力的裝置恆常顯示。
 *  所在列要帶 `group`（ListRow、SettingsRow 都有）。 */
export function RevokeButton({
  label,
  disabled,
  onClick,
}: {
  /** 無障礙名稱，指名要撤銷的對象。 */
  label: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  const { t } = useI18n();
  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className="opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100 [@media(hover:none)]:opacity-100"
    >
      {t("common.revoke")}
    </Button>
  );
}
