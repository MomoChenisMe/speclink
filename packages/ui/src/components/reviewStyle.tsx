import type { ReactElement } from "react";
import { BadgeAlert, BadgeCheck, BadgeX, Stamp } from "lucide-react";

/* 審查標示配色（spec desktop-app「看板卡片的審查標示」「詳情抽屜的審查資訊列」
   「已封存側的審查標示」共用）：四態各給可辨識的色，不落回灰階。
   審查中＝進行中的藍；已審查＝主色青綠（實心，與完成語意一致）；
   其後有變動＝沿用專案的琥珀警示；曾審查未通過＝紅（永久結局，警示層級最高）。 */
export const REVIEW_TONE = {
  inReview: "text-sky-600 dark:text-sky-400",
  reviewed: "text-primary",
  reviewedStale: "text-amber-600 dark:text-amber-500",
  reviewedNotPassed: "text-rose-600 dark:text-rose-400",
} as const;

/** 審查狀態 → i18n 詞條 key（active 四態與 archived 三態同一張表）：狀態詞在
   卡片、詳情抽屜與已封存兩處各出現一次，對照只此一份。`none` 與缺席不入表
   ——查不到即不渲染任何審查元素。 */
export const REVIEW_LABEL_KEY = {
  inReview: "review.inReview",
  reviewed: "review.reviewed",
  reviewedStale: "review.reviewedStale",
  reviewedNotPassed: "review.notPassed",
} as const;

/** 有標示的審查狀態——`REVIEW_TONE`／`REVIEW_LABEL_KEY` 的共同鍵集。 */
export type ReviewBadgeStatus = keyof typeof REVIEW_TONE;

/** 審查狀態 → 行內小章圖示：進行中的印章、已審查的實心徽章、其後有變動的
   警示徽章、未通過的叉號徽章。與 tone／label 同一組鍵，三處標示共用。 */
export const REVIEW_ICON: Record<ReviewBadgeStatus, ReactElement> = {
  inReview: <Stamp className="h-3.5 w-3.5" />,
  reviewed: <BadgeCheck className="h-3.5 w-3.5" />,
  reviewedStale: <BadgeAlert className="h-3.5 w-3.5" />,
  reviewedNotPassed: <BadgeX className="h-3.5 w-3.5" />,
};
