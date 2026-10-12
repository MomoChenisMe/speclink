import { Fragment, type HTMLAttributes, type ReactNode } from "react";

import type { Stage } from "../stage";
import { CopyButton } from "./CopyButton";
import { READING_COLUMN_CLS } from "./Markdown";
import { Card } from "./ui/card";
import type { StatusPillTone } from "./ui/status-pill";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";

/** 變更階段籤的 tone（看板欄色相）：變更詳情頁身分區與討論頁的衍生變更籤共用；已封存中性、已刪除警示由呼叫端另判。 */
export const STAGE_PILL_TONE: Record<Stage, StatusPillTone> = {
  proposed: "primary",
  "in-progress": "progress",
  ready: "success",
};

/** 詳情頁分頁對應的文件（design D4「開啟 ▾」）：change 的 artifact（輸出路徑，如
 * `proposal.md`、`specs/<capability>/spec.md`）、一筆討論記錄、一份正式規格，或封存變更
 * 的 artifact（dated name 定址）。磁碟位置由宿主解析（worktree 副本、spec 目錄名、已封存
 * 討論的日期檔名），頁不拼路徑。 */
export type DocumentRef =
  | { change: string; artifact: string }
  | { discussion: string }
  | { spec: string }
  | { archived: string; artifact: string };

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
 * 前段由缺席變存在，後段不得因位移而重掛。詳情頁標頭、身分區與閱讀卡出處行共用。 */
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

export interface DetailHeaderProps {
  /** 等寬 22px 標題（變更名或討論 slug），單行截斷。 */
  title: string;
  /** 標題旁複製鈕的無障礙標籤。 */
  copyLabel: string;
  /** 複製鈕寫入的值；缺席＝標題本身（封存變更以變更名為題、複製含日期前綴的封存目錄名）。 */
  copyValue?: string;
  /** 標題之後的章籤／狀態籤槽。 */
  badges?: ReactNode;
  /** 同列右端的動作區。 */
  actions?: ReactNode;
  /** 標題下方一行（討論的 topic）。 */
  subtitle?: ReactNode;
  /** 出身列的各段；缺席（falsy）的段整段略過，其餘以「·」分隔。 */
  provenance?: ReactNode[];
}

/**
 * 詳情頁標頭骨架（design D6／D7）：標題列（標題＋複製＋章籤槽＋右端動作區）、
 * 選用的副標行、灰字出身列。標題列與出身列恆定單行——不折行、不撐寬頁面。
 */
export function DetailHeader({
  title,
  copyLabel,
  copyValue,
  badges,
  actions,
  subtitle,
  provenance,
}: DetailHeaderProps) {
  const hasProvenance = (provenance ?? []).some(Boolean);
  return (
    <header className="flex min-w-0 flex-col gap-1.5">
      <div data-title-row className="flex min-w-0 items-center gap-2 whitespace-nowrap">
        <h1 className="min-w-0 truncate font-mono text-[22px] font-semibold leading-tight">{title}</h1>
        <CopyButton value={copyValue ?? title} label={copyLabel} />
        {badges && <div className="flex shrink-0 items-center gap-1.5">{badges}</div>}
        {actions && <div className="ml-auto flex shrink-0 items-center gap-1.5 pl-2">{actions}</div>}
      </div>
      {subtitle && <p className="truncate text-[15px] text-foreground/80">{subtitle}</p>}
      {hasProvenance && (
        <div
          data-provenance-row
          className="flex min-w-0 items-center gap-1.5 overflow-hidden whitespace-nowrap text-xs text-muted-foreground"
        >
          <DotSeparated segments={provenance ?? []} />
        </div>
      )}
    </header>
  );
}

/** 詳情頁內容欄：置中 768px（design D6）。資料屬性（如 `data-change-detail`）原樣傳入。 */
export function DetailPageFrame({ children, ...rest }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div {...rest} className="mx-auto flex w-full min-w-0 max-w-[768px] flex-col gap-5 px-6 pt-7 pb-10">
      {children}
    </div>
  );
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

/** 內容卡：接在卡片式分頁列下方，共用置中閱讀欄包住分頁全部內容（區段標籤、清單與內文同欄）。 */
export function DetailContentCard({ children }: { children: ReactNode }) {
  return (
    <Card className="rounded-t-none border-t-0 px-6 py-5">
      <div data-reading-column className={READING_COLUMN_CLS}>
        {children}
      </div>
    </Card>
  );
}
