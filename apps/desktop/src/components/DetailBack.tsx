// 詳情頁標題列左側的返回鈕與返回快捷鍵判斷（desktop-readonly-detail-pages design D8）。
// 右端的 ‹ n／m › 已搬進 @speclink/ui 的 DetailNav（server-web-detail-pages design D3）。
import { ArrowLeft } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";

import type { Platform } from "../platform";

/** 返回快捷鍵（design D8）：macOS 為 ⌘[ 與 ⌘←、其他平台 Alt+←。以實體按鍵（e.code）判定——
 * 中文輸入法啟用時 `[` 鍵送進頁面的 e.key 不一定是「[」；e.key 留作沒有 code 的事件後備。 */
export function isBackShortcut(
  e: Pick<KeyboardEvent, "key" | "code" | "metaKey" | "altKey" | "ctrlKey" | "shiftKey">,
  platform: Platform,
): boolean {
  if (e.shiftKey || e.ctrlKey) return false;
  const leftArrow = e.code === "ArrowLeft" || e.key === "ArrowLeft";
  if (platform === "macos") {
    return e.metaKey && !e.altKey && (leftArrow || e.code === "BracketLeft" || e.key === "[");
  }
  return e.altKey && !e.metaKey && leftArrow;
}

/** 返回鈕：回宿主清單頁（與 Esc、麵包屑中段同一個動作），提示標示 Esc 與返回快捷鍵。 */
export function DetailBack({
  page,
  keys,
  onBack,
}: {
  /** 宿主頁名（規格、已封存、變更、手冊）。 */
  page: string;
  /** 依平台的返回快捷鍵（⌘[ 或 Alt+←）。 */
  keys: string;
  onBack: () => void;
}) {
  const { t } = useI18n();
  const label = t("detail.back").replace("{page}", page);
  return (
    <Button
      variant="subtle"
      size="icon-xs"
      aria-label={label}
      title={t("detail.backHint").replace("{label}", label).replace("{keys}", keys)}
      className="-ml-1.5 mr-1.5 shrink-0"
      onClick={onBack}
    >
      <ArrowLeft className="h-4 w-4" />
    </Button>
  );
}
