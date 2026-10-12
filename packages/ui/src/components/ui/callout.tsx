import type { ReactNode } from "react";

import { SEMANTIC_SURFACE, SEMANTIC_TONE, type SemanticTone } from "../../tone";

/** 提示框四型（與 GitHub Alert 同名）。 */
export type CalloutTone = "note" | "tip" | "warning" | "caution";

/** 型 → 介面狀態語意色（資訊、成功、警告、危險；不佔主色）。 */
export const CALLOUT_TONE: Record<CalloutTone, SemanticTone> = {
  note: "inProgress",
  tip: "success",
  warning: "warning",
  caution: "danger",
};

/** 外框與標題列款式——Markdown 的 `> [!NOTE]` 提示框與 sanitize 白名單也由這兩串展開。 */
export const CALLOUT_BOX_CLS = "my-4 rounded-r-md border-l-4 px-4 py-2 [&>p]:my-1 [&>ul]:my-1 [&>ol]:my-1";
export const CALLOUT_TITLE_CLS = "markdown-alert-title mb-1 text-sm font-semibold";

export interface CalloutProps {
  tone: CalloutTone;
  title: ReactNode;
  icon?: ReactNode;
  children?: ReactNode;
}

/** 提示框（design D6）：左粗線＋語意色淡底、語意色標題列。與 Markdown 的 GitHub Alert 同款。 */
export function Callout({ tone, title, icon, children }: CalloutProps) {
  const semantic = CALLOUT_TONE[tone];
  return (
    <div className={`${CALLOUT_BOX_CLS} ${SEMANTIC_SURFACE[semantic]}`}>
      <div className={`${CALLOUT_TITLE_CLS} ${SEMANTIC_TONE[semantic]}${icon ? " flex items-center gap-1.5" : ""}`}>
        {icon}
        {title}
      </div>
      {children}
    </div>
  );
}
