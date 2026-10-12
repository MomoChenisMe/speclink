import {
  createContext,
  forwardRef,
  useContext,
  useLayoutEffect,
  useRef,
  type ReactNode,
  type RefObject,
} from "react";
import { createPortal } from "react-dom";

import { useI18n } from "../i18n";
import { CopyButton } from "./CopyButton";
import { DotSeparated } from "./DetailHeader";
import { ManualToc } from "./ManualToc";

/** 文件 → 閱讀卡標題（與文件節點同一詞條）：變更、討論與已封存三頁共用。 */
export const DOC_TITLE_KEYS: Record<string, string> = {
  proposal: "common.tabProposal",
  design: "common.tabDesign",
  tasks: "common.tabTasks",
  specs: "common.tabSpecs",
  plan: "drawer.tab.plan",
  review: "drawer.tab.review",
  verify: "drawer.tab.verify",
  conclusion: "ddrawer.tabConclusion",
  rounds: "ddrawer.tabRounds",
  context: "ddrawer.tabContext",
  promote: "ddrawer.tabPromote",
  record: "ddrawer.tabRecord",
};

/** 變更文件的本頁目錄重掃鍵：規格取全部 capability、工單取輪次，其餘取該文件內容；任一變就重掃錨點。
 * 變更詳情頁與已封存變更頁共用。 */
export function changeDocTocKey(
  tab: string,
  docs: { content: unknown; specs: unknown; tickets: Record<"review" | "verify", unknown> },
): string {
  const source = tab === "specs" ? docs.specs : tab === "review" || tab === "verify" ? docs.tickets[tab] : docs.content;
  return `${tab}\u0000${JSON.stringify(source)}`;
}

/** 本頁目錄收的標題：渲染器標出的區段標題，加上 Markdown 內文的 h2／h3。 */
const DOC_HEADINGS = "[data-doc-heading], .markdown h2, .markdown h3";

export interface DetailShellProps {
  /** 外框上的頁標記（`data-change-detail` 等），供宿主與測試定位。 */
  [mark: `data-${string}`]: string | boolean | undefined;
  /** 左欄文件樹的內容（身分區、「文件」節點、「狀態」列）。 */
  tree: ReactNode;
  /** 目前文件（含項目識別）；一變閱讀區就捲回頂端。 */
  activeDoc: string;
  /** 本頁目錄的重掃鍵：文件或其內容一變就重掃錨點。 */
  tocKey: string;
  /** 指向閱讀卡的 `DocBody`——本頁目錄自它渲染後的 DOM 推導錨點。 */
  bodyRef: RefObject<HTMLElement | null>;
  children: ReactNode;
}

/**
 * 詳情頁三欄骨架（design D1；spec「詳情頁的文件樹、閱讀卡與本頁目錄」）：左欄 240px 文件樹
 * （獨立捲動）、中欄閱讀區（置中 768px，自己捲動）、右欄 200px 本頁目錄。右欄以 container
 * query 量主區寬，≥1200px 才顯示；沒有錨點時目錄不渲染、右欄留空。無底部固定列。
 */
export function DetailShell({ tree, activeDoc, tocKey, bodyRef, children, ...marks }: DetailShellProps) {
  const { locale } = useI18n();
  const scrollerRef = useRef<HTMLDivElement>(null);
  // 換文件：閱讀區自頂端呈現（畫面畫出前就歸零，不閃前一份的捲動位置）；左欄捲動位置不受影響。
  useLayoutEffect(() => {
    if (scrollerRef.current) scrollerRef.current.scrollTop = 0;
  }, [activeDoc]);
  return (
    <div {...marks} data-detail-shell className="@container flex h-full min-h-0 w-full">
      <aside
        data-detail-tree
        className="flex w-[240px] shrink-0 flex-col overflow-y-auto border-r border-border px-2 pt-3.5 pb-3"
      >
        {tree}
      </aside>
      <div data-detail-reading ref={scrollerRef} className="min-h-0 min-w-0 flex-1 overflow-y-auto px-6 py-7">
        <div className="mx-auto flex w-full max-w-[768px] flex-col gap-5">{children}</div>
      </div>
      <aside data-detail-toc className="hidden w-[200px] shrink-0 @min-[1200px]:flex">
        {/* 錨點文字多為譯文（章節標籤、欄位標籤）：語系一換也重掃。 */}
        <ManualToc
          bodyRef={bodyRef}
          markdownBody={`${locale}\u0000${tocKey}`}
          headings={DOC_HEADINGS}
          className="flex border-l-0"
        />
      </aside>
    </div>
  );
}

/** 身分區的一列關聯：標籤＋內容（內容可折行），`data-relation-<mark>` 供測試定位。 */
export interface DetailRelation {
  mark: string;
  label: string;
  children: ReactNode;
}

export interface DetailIdentityProps {
  /** 名稱或 slug：等寬 14px，過長折行、不撐寬左欄。 */
  title: string;
  copyLabel: string;
  /** 複製鈕寫入的值；缺席＝標題本身。 */
  copyValue?: string;
  /** 階段籤、章籤等籤列。 */
  badges?: ReactNode;
  /** 討論 topic：最多三行、溢出截斷。 */
  subtitle?: ReactNode;
  /** 出身行的各段：單行灰字、缺席段略過、溢出裁切。 */
  provenance?: ReactNode[];
  /** 關聯列，各自一行；空陣列＝無關聯列。 */
  relations?: DetailRelation[];
}

/** 左欄身分區（design D1）：承接原標題列與出身列。 */
export function DetailIdentity({
  title,
  copyLabel,
  copyValue,
  badges,
  subtitle,
  provenance,
  relations,
}: DetailIdentityProps) {
  return (
    <section data-detail-identity className="flex min-w-0 flex-col gap-2 px-2 pb-1">
      <div className="flex min-w-0 items-start gap-1">
        <h1 className="min-w-0 break-all pt-0.5 font-mono text-sm font-semibold leading-snug">{title}</h1>
        <CopyButton value={copyValue ?? title} label={copyLabel} />
      </div>
      {badges && <div className="flex min-w-0 flex-wrap items-center gap-1.5">{badges}</div>}
      {subtitle && <p className="line-clamp-3 text-[13px] text-foreground/80">{subtitle}</p>}
      {(provenance ?? []).some(Boolean) && (
        <div
          data-identity-provenance
          className="flex min-w-0 items-center gap-1.5 overflow-hidden whitespace-nowrap text-[11px] text-muted-foreground"
        >
          <DotSeparated segments={provenance ?? []} />
        </div>
      )}
      {(relations ?? []).map((r) => (
        <div key={r.mark} {...{ [`data-relation-${r.mark}`]: "" }} className="flex min-w-0 items-start gap-2 text-xs">
          <span className="min-w-8 shrink-0 whitespace-nowrap leading-6 text-muted-foreground">{r.label}</span>
          <div className="flex min-h-6 min-w-0 flex-1 flex-wrap items-center gap-1">{r.children}</div>
        </div>
      ))}
    </section>
  );
}

/** 左欄小標（「文件」「狀態」）：與手冊頁側欄樹的分組標題同款。 */
export function DetailTreeLabel({ children }: { children: ReactNode }) {
  return (
    <div className="px-2.5 pt-4 pb-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
      {children}
    </div>
  );
}

/** 左欄「狀態」列：承接原「狀態」卡的內容（`DetailHeader` 的 `StatusRow` 清單，細線分隔）。 */
export function DetailStatus({ children }: { children: ReactNode }) {
  return (
    <section data-status-card className="flex flex-col divide-y divide-border">
      {children}
    </section>
  );
}

export interface DocHeaderProps {
  /** 文件名（24px 一般字重）。 */
  title: ReactNode;
  badges?: ReactNode;
  /** 同列右端（「開啟 ▾」）。 */
  actions?: ReactNode;
  /** 出處行：13px 灰字、「·」分隔、可折行。 */
  provenance?: ReactNode[];
}

/** 閱讀卡標題區（design D1）。 */
export function DocHeader({ title, badges, actions, provenance }: DocHeaderProps) {
  return (
    <header data-doc-header className="flex min-w-0 flex-col gap-1.5">
      <div className="flex min-w-0 items-center gap-2">
        <h2 className="min-w-0 truncate text-2xl font-normal leading-tight">{title}</h2>
        {badges && (
          <div data-doc-badges className="flex shrink-0 items-center gap-1.5">
            {badges}
          </div>
        )}
        {actions && <div className="ml-auto flex shrink-0 items-center">{actions}</div>}
      </div>
      {(provenance ?? []).some(Boolean) && (
        <div data-doc-provenance className="flex min-w-0 flex-wrap items-center gap-x-1.5 text-[13px] text-muted-foreground">
          <DotSeparated segments={provenance ?? []} />
        </div>
      )}
    </header>
  );
}

/** 一格數字磚：值與標籤。 */
export interface DocTile {
  value: ReactNode;
  label: string;
}

/** 閱讀卡標題區的推導結果：籤槽、數字磚、提示框（缺席時為空）。 */
export interface DocSummary {
  badges: ReactNode;
  tiles: DocTile[];
  callout: ReactNode;
}

/** 數字磚：閱讀卡標題下由文件結構推出的計數格，最多四格；空陣列整列缺席。 */
export function DocTiles({ tiles }: { tiles: DocTile[] }) {
  if (tiles.length === 0) return null;
  return (
    <div data-doc-tiles className="grid grid-cols-4 gap-2.5">
      {tiles.map((tile, i) => (
        <div key={i} data-doc-tile className="flex min-w-0 flex-col gap-0.5 rounded-xl border border-border px-3 py-2.5">
          <span className="text-xl font-semibold tabular-nums">{tile.value}</span>
          <span className="truncate text-[11px] text-muted-foreground">{tile.label}</span>
        </div>
      ))}
    </div>
  );
}

/** 閱讀卡內文：包住各文件渲染器的輸出，本頁目錄自它推導錨點。內文 15px／行高 1.75，行寬以卡欄
 * （768px 扣內距）為上限（design D10；規格詳情頁與手冊頁不經這裡）。index.css 的 `.markdown` 基準字級
 * 不在 cascade layer、一般宣告就壓過 Tailwind utilities 層，字級與行高的覆蓋因此帶 important（`!`）；
 * Markdown 自帶的 96ch 同在 utilities 層，後代選擇器的特異性就夠。 */
const DOC_BODY_CLS =
  "w-full text-[15px] leading-[1.75] [&_.markdown]:max-w-none [&_.markdown]:text-[15px]! [&_.markdown]:leading-[1.75]!";

export const DocBody = forwardRef<HTMLDivElement, { children: ReactNode }>(({ children }, ref) => (
  <div ref={ref} data-doc-body className={DOC_BODY_CLS}>
    {children}
  </div>
));
DocBody.displayName = "DocBody";

/** 標題列動作插槽：undefined＝沒有宿主（原地渲染）、null＝宿主的插槽尚未掛上。 */
const ActionsSlotContext = createContext<HTMLElement | null | undefined>(undefined);

/** 宿主提供插槽節點（design D3）：value 為宿主以 callback ref 取得的 [`DetailActionsSlot`] 節點。 */
export const DetailActionsSlotProvider = ActionsSlotContext.Provider;

/** 宿主標題列上的空插槽。 */
export const DetailActionsSlot = forwardRef<HTMLDivElement>((_, ref) => (
  <div ref={ref} data-detail-actions className="flex shrink-0 items-center gap-1.5 pr-3" />
));
DetailActionsSlot.displayName = "DetailActionsSlot";

/**
 * 頁的變更層動作（design D3）：有宿主時以 portal 送進標題列插槽，最前面帶一條細線與
 * ‹ n／m › 分隔；沒有宿主時原地渲染。
 */
export function DetailActions({ children }: { children: ReactNode }) {
  const slot = useContext(ActionsSlotContext);
  if (slot === undefined) {
    return (
      <div data-detail-actions className="flex items-center justify-end gap-1.5">
        {children}
      </div>
    );
  }
  if (!slot) return null;
  return createPortal(
    <>
      <span aria-hidden="true" className="mx-1.5 h-5 w-px bg-border" />
      {children}
    </>,
    slot,
  );
}
