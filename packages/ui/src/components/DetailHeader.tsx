import { Fragment, type HTMLAttributes, type ReactNode } from "react";

import { useI18n } from "../i18n";
import { CopyButton } from "./CopyButton";
import { READING_COLUMN_CLS } from "./Markdown";
import { SectionHeader } from "./SectionHeader";
import { Card } from "./ui/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";

/** 詳情頁分頁對應的文件（design D4「開啟 ▾」）：change 的 artifact（輸出路徑，如
 * `proposal.md`、`specs/<capability>/spec.md`）或一筆討論記錄。磁碟位置由宿主解析
 * （worktree 副本、spec 目錄名、已封存討論的日期檔名），頁不拼路徑。 */
export type DocumentRef = { change: string; artifact: string } | { discussion: string };

/** 「開啟 ▾」的對象：文件與該分頁已載入的原文（未載完為 null）。分頁不載原文時
 * （審查／驗證工單只載解析後的回合）`content` 缺席。 */
export interface DocumentTarget {
  doc: DocumentRef;
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

export interface DetailHeaderProps {
  /** 等寬 22px 標題（變更名或討論 slug），單行截斷。 */
  title: string;
  /** 標題旁複製鈕的無障礙標籤。 */
  copyLabel: string;
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
export function DetailHeader({ title, copyLabel, badges, actions, subtitle, provenance }: DetailHeaderProps) {
  // 以原始槽位為 key：資料陸續抵達時前段由缺席變存在，後段不得因位移而重掛。
  const segments = (provenance ?? []).flatMap((node, slot) => (node ? [{ node, slot }] : []));
  return (
    <header className="flex min-w-0 flex-col gap-1.5">
      <div data-title-row className="flex min-w-0 items-center gap-2 whitespace-nowrap">
        <h1 className="min-w-0 truncate font-mono text-[22px] font-semibold leading-tight">{title}</h1>
        <CopyButton value={title} label={copyLabel} />
        {badges && <div className="flex shrink-0 items-center gap-1.5">{badges}</div>}
        {actions && <div className="ml-auto flex shrink-0 items-center gap-1.5 pl-2">{actions}</div>}
      </div>
      {subtitle && <p className="truncate text-[15px] text-foreground/80">{subtitle}</p>}
      {segments.length > 0 && (
        <div
          data-provenance-row
          className="flex min-w-0 items-center gap-1.5 overflow-hidden whitespace-nowrap text-xs text-muted-foreground"
        >
          {segments.map(({ node, slot }, i) => (
            <Fragment key={slot}>
              {i > 0 && (
                <span aria-hidden="true" className="shrink-0 opacity-50">
                  ·
                </span>
              )}
              {node}
            </Fragment>
          ))}
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

/** 「狀態」卡：區段標頭＋細線分隔的列（內容為 [`StatusRow`]）。 */
export function StatusCard({ children }: { children: ReactNode }) {
  const { t } = useI18n();
  return (
    <section data-status-card className="flex flex-col gap-2">
      <SectionHeader label={t("detail.status")} />
      <Card className="divide-y divide-border">{children}</Card>
    </section>
  );
}

/** 狀態卡的一列：左標籤右內容，`data-status-<mark>` 供測試定位。內容預設單行不撐寬；
 * `wrap` 讓多顆籤（衍生變更）折行。 */
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
    <div {...{ [`data-status-${mark}`]: "" }} className="flex min-w-0 items-center gap-4 px-4 py-2.5 text-sm">
      <span className="w-20 shrink-0 text-xs text-muted-foreground">{label}</span>
      <div
        className={
          wrap
            ? "flex min-w-0 flex-1 flex-wrap items-center gap-1.5"
            : "flex min-w-0 flex-1 items-center gap-2 overflow-hidden whitespace-nowrap"
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
