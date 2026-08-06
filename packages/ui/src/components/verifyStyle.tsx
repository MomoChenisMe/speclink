import type { ReactElement } from "react";
import { Shield, ShieldAlert, ShieldCheck, ShieldX } from "lucide-react";

import { REVIEW_TONE } from "./reviewStyle";

/* 驗證標示配色（spec desktop-app「看板卡片的驗證標示」「詳情抽屜的驗證資訊列」
   「已封存側的驗證標示」共用；design D5）：色承載狀態、圖示形狀承載站別。
   四個色階直接引用審查章樣式表的同值常數——不複製色階字面，否則兩站日後只會
   一邊改、一邊留。 */
export const VERIFY_TONE = {
  inVerify: REVIEW_TONE.inReview,
  verified: REVIEW_TONE.reviewed,
  verifiedStale: REVIEW_TONE.reviewedStale,
  verifiedNotPassed: REVIEW_TONE.reviewedNotPassed,
} as const;

/** 驗證狀態 → i18n 詞條 key（active 四態與 archived 三態同一張表）。`none` 與
   缺席不入表——查不到即不渲染任何驗證元素。 */
export const VERIFY_LABEL_KEY = {
  inVerify: "verify.inVerify",
  verified: "verify.verified",
  verifiedStale: "verify.verifiedStale",
  verifiedNotPassed: "verify.notPassed",
} as const;

/** 有標示的驗證狀態——`VERIFY_TONE`／`VERIFY_LABEL_KEY` 的共同鍵集。 */
export type VerifyBadgeStatus = keyof typeof VERIFY_TONE;

/** 驗證狀態 → 行內小章圖示：盾牌系（審查站是徽章系），同色不同形即可一眼分站。 */
export const VERIFY_ICON: Record<VerifyBadgeStatus, ReactElement> = {
  inVerify: <Shield className="h-3.5 w-3.5" />,
  verified: <ShieldCheck className="h-3.5 w-3.5" />,
  verifiedStale: <ShieldAlert className="h-3.5 w-3.5" />,
  verifiedNotPassed: <ShieldX className="h-3.5 w-3.5" />,
};
