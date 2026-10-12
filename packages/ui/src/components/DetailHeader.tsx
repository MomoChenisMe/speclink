import { Fragment, type ReactNode } from "react";

import type { Stage } from "../stage";
import type { StatusPillTone } from "./ui/status-pill";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";

/** 變更階段籤的 tone（看板欄色相）：變更詳情頁身分區與討論頁的衍生變更籤共用；已封存中性、已刪除警示由呼叫端另判。 */
export const STAGE_PILL_TONE: Record<Stage, StatusPillTone> = {
  proposed: "primary",
  "in-progress": "progress",
  ready: "success",
};

/** 詳情頁分頁對應的文件（design D4「開啟 ▾」）：change 的 artifact（輸出路徑，如
 * `proposal.md`、`specs/<capability>/spec.md`）、一筆討論記錄、一份正式規格、封存變更
 * 的 artifact（dated name 定址），或一頁手冊（slug 定址）。磁碟位置由宿主解析（worktree
 * 副本、spec 目錄名、已封存討論的日期檔名），頁不拼路徑。 */
export type DocumentRef =
  | { change: string; artifact: string }
  | { discussion: string }
  | { spec: string }
  | { archived: string; artifact: string }
  | { manual: string };

/** 對象的 artifact 路徑（相對變更目錄，閱讀卡出處行用）；討論、正式規格與沒有檔案時為 null。 */
export function documentArtifact(target: DocumentTarget | null): string | null {
  return target?.doc && "artifact" in target.doc ? target.doc.artifact : null;
}

/** 「開啟 ▾」的對象：文件與該分頁已載入的原文（未載完為 null）。分頁不載原文時
 * （審查／驗證工單只載解析後的回合）`content` 缺席；`doc` 為 null＝沒有可定位的檔案
 * （已封存討論），只能複製原文。 */
export interface DocumentTarget {
  doc: DocumentRef | null;
  content?: string | null;
}

/** 出身列僅顯示名字：去除 email 尖括號段（無尖括號時整串直出），完整識別由提示承載。
 * 變更、討論與已封存詳情頁的出身列共用。 */
export function displayName(identity: string): string {
  const name = identity.replace(/\s*<[^>]*>\s*/g, " ").trim();
  return name || identity;
}

/** 出身列的人：首字母圓標（中性）＋名字，含 email 的完整識別收進提示。
 * 需在 TooltipProvider 內渲染。 */
export function IdentityName({ identity }: { identity: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex shrink-0 items-center gap-1">
          <span className="inline-flex h-4 w-4 items-center justify-center rounded-full bg-muted text-[9px] font-bold text-muted-foreground">
            {identity.charAt(0).toUpperCase()}
          </span>
          {displayName(identity)}
        </span>
      </TooltipTrigger>
      <TooltipContent>{identity}</TooltipContent>
    </Tooltip>
  );
}

/** 以「·」分隔的資訊段：缺席（falsy）的段整段略過。以原始槽位為 key——資料陸續抵達時
 * 前段由缺席變存在，後段不得因位移而重掛。身分區與閱讀卡出處行共用。 */
export function DotSeparated({ segments }: { segments: ReactNode[] }) {
  return segments
    .flatMap((node, slot) => (node ? [{ node, slot }] : []))
    .map(({ node, slot }, i) => (
      <Fragment key={slot}>
        {i > 0 && (
          <span aria-hidden="true" className="shrink-0 opacity-50">
            ·
          </span>
        )}
        {node}
      </Fragment>
    ));
}

/** 「狀態」列的一列（文件樹左欄，design D1）：左標籤右內容，`data-status-<mark>` 供測試定位。
 * 內容預設單行不撐寬；`wrap` 讓多顆籤（衍生變更、階段步驟）折行。 */
export function StatusRow({
  mark,
  label,
  wrap = false,
  children,
}: {
  mark: string;
  label: string;
  wrap?: boolean;
  children: ReactNode;
}) {
  return (
    <div {...{ [`data-status-${mark}`]: "" }} className="flex min-w-0 items-start gap-2 px-2 py-2 text-xs">
      <span className="w-14 shrink-0 leading-5 text-muted-foreground">{label}</span>
      <div
        className={
          wrap
            ? "flex min-h-5 min-w-0 flex-1 flex-wrap items-center gap-1.5"
            : "flex min-h-5 min-w-0 flex-1 items-center gap-2 overflow-hidden whitespace-nowrap"
        }
      >
        {children}
      </div>
    </div>
  );
}
