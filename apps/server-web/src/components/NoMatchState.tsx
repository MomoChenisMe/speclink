import { SearchX } from "lucide-react";
import { EmptyState, useI18n } from "@speclink/ui";

/** 篩選後沒有符合項目——訊息與圖示都刻意與「這份清單本來就空」區分開，且不給建立動作：
 *  該情況要改的是篩選，不是建立（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」）。 */
export function NoMatchState() {
  const { t } = useI18n();
  return <EmptyState icon={SearchX} title={t("common.noMatchTitle")} description={t("common.noMatchHint")} />;
}
