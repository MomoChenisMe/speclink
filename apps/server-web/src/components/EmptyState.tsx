import type { ComponentType, ReactNode } from "react";
import { Inbox, SearchX } from "lucide-react";
import { useI18n } from "@speclink/ui";

// 空狀態（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」）：
// 圖示、一句說明該資料用途的文字，以及該頁的 primary action——取代單行「尚無 X。」。
// 篩選後無結果時只說明「沒有符合的項目」而不給建立動作：該情況要改的是篩選，不是建立。
export function EmptyState({
  title,
  description,
  action,
  icon: Icon = Inbox,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
  /** 換一個更貼近該頁的圖示；預設是「空收件匣」。 */
  icon?: ComponentType<{ className?: string; "aria-hidden"?: boolean | "true" | "false" }>;
}) {
  return (
    <div className="flex flex-col items-center gap-2 rounded-md border border-dashed border-border px-6 py-10 text-center">
      <Icon aria-hidden="true" className="h-6 w-6 text-muted-foreground" />
      <p className="font-medium">{title}</p>
      {description && (
        <p className="max-w-prose text-sm text-muted-foreground">{description}</p>
      )}
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}

/** 篩選後沒有符合項目——訊息與圖示都刻意與「這份清單本來就空」區分開。 */
export function NoMatchState() {
  const { t } = useI18n();
  return (
    <EmptyState
      icon={SearchX}
      title={t("common.noMatchTitle")}
      description={t("common.noMatchHint")}
    />
  );
}
