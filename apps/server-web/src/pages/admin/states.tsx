import { Button, useI18n } from "@speclink/ui";

// 管理六頁共用的載入／錯誤狀態（D6：一致且可恢復）。標題屬頁面外殼、始終呈現，
// 資料層則以這兩個狀態呈現載入與可重試錯誤。
export function AdminLoading() {
  const { t } = useI18n();
  return (
    <p role="status" aria-live="polite" className="text-muted-foreground">
      {t("common.loading")}
    </p>
  );
}

export function AdminError({ onRetry }: { onRetry: () => void }) {
  const { t } = useI18n();
  return (
    <div role="alert" className="space-y-2">
      <p className="text-destructive">{t("common.loadFailed")}</p>
      <Button type="button" variant="outline" size="sm" onClick={onRetry}>
        {t("common.retry")}
      </Button>
    </div>
  );
}
