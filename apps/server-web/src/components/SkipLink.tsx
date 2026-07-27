// 每頁第一個可聚焦元素（D6）。平時視覺隱藏，聚焦時顯示；跳至 <main id>。
import { useI18n } from "@speclink/ui";

export function SkipLink() {
  const { t } = useI18n();
  return (
    <a
      href="#main-content"
      className="sr-only rounded-md bg-primary px-3 py-2 text-primary-foreground focus:not-sr-only focus:absolute focus:left-2 focus:top-2 focus:z-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {t("shell.skipToContent")}
    </a>
  );
}
