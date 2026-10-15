import type { ButtonHTMLAttributes, ReactNode } from "react";
import { ChevronDown, ChevronRight, FolderOpen } from "lucide-react";

import { listItemsOf, parseListItems, type H2Section, type ListBlock, type ListItem } from "../docModel";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { CopyButton } from "./CopyButton";
import { Markdown } from "./Markdown";

// 變更文件渲染器的共用零件（desktop-change-doc-views design D2）：全部只用 token class，ui/ 原語不動。

/** 章節標題款式（16px 粗體）：提案／設計章節、任務群組卡頭、delta 需求標題、排程卡頭、工單輪卡頭同源。 */
export const SECTION_HEADING_CLS = "text-base font-semibold";
/** 次級標籤（Capabilities 內的新增／修改能力、Goals 兩欄小標）。 */
export const SUB_LABEL_CLS = "text-sm font-semibold";

/** 列內 Markdown：段落不留上下距（列本身已有間距）。 */
export const ROW_MD_CLS = "min-w-0 flex-1 [&_p]:my-0";

/** 一個二級標題段：`h2` 帶本頁目錄錨點、右端計數籤，段間細線（末段無線）。 */
export function DocSection({
  sectionKey,
  title,
  count,
  children,
}: {
  /** 白名單 key；白名單外為 null（`data-section="custom"`）。 */
  sectionKey: string | null;
  title: string;
  count?: number;
  children?: ReactNode;
}) {
  return (
    <section data-section={sectionKey ?? "custom"} className="border-b border-border/70 py-5 first:pt-0 last:border-b-0 last:pb-0">
      <div className="mb-2.5 flex items-center gap-2">
        <h2 data-doc-heading className={SECTION_HEADING_CLS}>
          {title}
        </h2>
        {count !== undefined && (
          <Badge variant="secondary" className="ml-auto tabular-nums">
            {count}
          </Badge>
        )}
      </div>
      {children}
    </section>
  );
}

/** 列式章節：列點群交給 `renderList`、prose 照排，標題右端為頂層列點數（零則不出）。 */
export function ListSection({
  section,
  title,
  renderList,
}: {
  section: H2Section;
  title: string;
  renderList: (items: ListItem[]) => ReactNode;
}) {
  const blocks = parseListItems(section.content);
  const count = listItemsOf(blocks).length;
  return (
    <DocSection sectionKey={section.key} title={title} count={count > 0 ? count : undefined}>
      <ListBlocks blocks={blocks} renderList={renderList} />
    </DocSection>
  );
}

/** 依序渲染列點群與 prose 段：列點群交給 `renderList`，prose 照 Markdown。 */
export function ListBlocks({ blocks, renderList }: { blocks: ListBlock[]; renderList: (items: ListItem[]) => ReactNode }) {
  return (
    <div className="flex flex-col gap-3">
      {blocks.map((b, i) =>
        b.kind === "prose" ? <Markdown key={i} content={b.text} /> : <div key={i}>{renderList(b.items)}</div>,
      )}
    </div>
  );
}

/** 細線列：每列一段 Markdown；`icon` 放在列首（非目標的禁止圖示）。 */
export function RowList({ items, icon }: { items: string[]; icon?: ReactNode }) {
  return (
    <ul data-row-list className="divide-y divide-border/70">
      {items.map((md, i) => (
        <li key={i} className="flex items-start gap-2.5 py-2 first:pt-0 last:pb-0">
          {icon && <span className="mt-1 shrink-0 text-muted-foreground">{icon}</span>}
          {md && <Markdown content={md} className={ROW_MD_CLS} />}
        </li>
      ))}
    </ul>
  );
}

/** 編號步驟：序號圓標＋內文。 */
export function NumberedSteps({ items }: { items: string[] }) {
  return (
    <ol data-numbered-steps className="flex flex-col gap-2.5">
      {items.map((md, i) => (
        <li key={i} className="flex items-start gap-3">
          <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold tabular-nums">
            {i + 1}
          </span>
          <Markdown content={md} className={ROW_MD_CLS} />
        </li>
      ))}
    </ol>
  );
}

/** 替代方案列：方案一行、落敗理由灰字一行；理由為 null 時只有方案。`icon` 放在列首（擱置的時鐘）。 */
export function OptionReasonRows({
  items,
  icon,
}: {
  items: { option: string; reason: string | null }[];
  icon?: ReactNode;
}) {
  const lines = (it: { option: string; reason: string | null }) => (
    <>
      <div data-alternative-option className="font-medium">
        <Markdown content={it.option} className={ROW_MD_CLS} />
      </div>
      {it.reason && (
        <div data-alternative-reason className="text-muted-foreground">
          <Markdown content={it.reason} className={ROW_MD_CLS} />
        </div>
      )}
    </>
  );
  return (
    <ul className="divide-y divide-border/70">
      {items.map((it, i) =>
        icon ? (
          <li key={i} data-alternative className="flex items-start gap-2.5 py-2 first:pt-0 last:pb-0">
            <span className="mt-1 shrink-0 text-muted-foreground">{icon}</span>
            <div className="flex min-w-0 flex-1 flex-col gap-1">{lines(it)}</div>
          </li>
        ) : (
          <li key={i} data-alternative className="flex flex-col gap-1 py-2 first:pt-0 last:pb-0">
            {lines(it)}
          </li>
        ),
      )}
    </ul>
  );
}

/** 鍵值列：左鍵灰字、右值 Markdown。 */
export function KeyValueRows({ rows }: { rows: { key: string; value: string }[] }) {
  return (
    <dl className="divide-y divide-border/70">
      {rows.map((r) => (
        <div key={r.key} className="flex items-start gap-4 py-2 first:pt-0 last:pb-0">
          <dt className="w-28 shrink-0 text-sm text-muted-foreground">{r.key}</dt>
          <dd className="min-w-0 flex-1">{r.value && <Markdown content={r.value} className={ROW_MD_CLS} />}</dd>
        </div>
      ))}
    </dl>
  );
}

/** 等寬名稱籤的款式：能力籤、決策 D 籤、任務 D 籤共用。 */
export const NAME_CHIP_CLS = "inline-flex shrink-0 items-center rounded bg-muted px-1.5 py-0.5 font-mono text-xs";

/** 能力名籤：`onClick` 有時為可點的 button。 */
export function NameChip({ name, onClick }: { name: string; onClick?: () => void }) {
  if (!onClick) {
    return (
      <span data-capability-chip className={NAME_CHIP_CLS}>
        {name}
      </span>
    );
  }
  return (
    <button
      type="button"
      data-capability-chip
      className={`${NAME_CHIP_CLS} text-primary underline-offset-2 hover:bg-accent hover:underline`}
      onClick={onClick}
    >
      {name}
    </button>
  );
}

/** 指令籤：等寬指令原文＋複製鈕。結論的「下一步」與未轉出提示框共用。 */
export function CommandChip({ command }: { command: string }) {
  const { t } = useI18n();
  return (
    <span data-command-chip className={`${NAME_CHIP_CLS} gap-1`}>
      {command}
      <CopyButton value={command} label={t("conclusion.copyCommand")} className="h-5 w-5" />
    </span>
  );
}

/** hover 籤時才浮出的動作鈕（鍵盤聚焦時也顯現）。 */
const REVEAL_ON_CHIP_HOVER = "opacity-0 transition-opacity group-hover/path:opacity-100 focus-visible:opacity-100 data-[copied]:opacity-100";

/** 宿主的本機路徑動作：鈕的文案（依平台，如 macOS「在 Finder 顯示」）與動作本身。 */
export interface PathReveal {
  label: string;
  reveal: (path: string) => void;
}

/** 等寬路徑籤：hover 出複製鈕；宿主提供本機路徑動作時另出顯示鈕。 */
export function PathChip({ path, note, reveal }: { path: string; note?: string | null; reveal?: PathReveal }) {
  const { t } = useI18n();
  return (
    <span className="inline-flex min-w-0 max-w-full items-center gap-1.5">
      <span data-path-chip className={`group/path ${NAME_CHIP_CLS} min-w-0 gap-1`}>
        <span className="min-w-0 break-all">{path}</span>
        <CopyButton value={path} label={t("common.copy")} className={REVEAL_ON_CHIP_HOVER} />
        {reveal && (
          <Button
            type="button"
            variant="subtle"
            size="icon-xs"
            aria-label={reveal.label}
            title={reveal.label}
            className={`h-5 w-5 shrink-0 ${REVEAL_ON_CHIP_HOVER}`}
            onClick={() => reveal.reveal(path)}
          >
            <FolderOpen className="h-3.5 w-3.5" />
          </Button>
        )}
      </span>
      {note && <span className="text-xs text-muted-foreground">{note}</span>}
    </span>
  );
}

/** 「展開其餘 N 段」「還有 N 個」：點了就全部展開、鈕消失。 */
export function MoreToggle({ label, onExpand }: { label: string; onExpand: () => void }) {
  return (
    <Button type="button" variant="ghost" size="sm" aria-expanded={false} className="text-muted-foreground" onClick={onExpand}>
      {label}
    </Button>
  );
}

/** 整列可點的展開切換鈕，左側 chevron 標示開合（設計決策卡、工單輪卡與套用規則列同一種）；
 * 內距、間距與換行由呼叫端給。 */
export function ExpandToggle({
  open,
  onToggle,
  className,
  children,
  ...rest
}: { open: boolean; onToggle: () => void } & Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type" | "onClick">) {
  return (
    <button
      type="button"
      aria-expanded={open}
      onClick={onToggle}
      {...rest}
      className={cn(
        "flex w-full cursor-(--cursor-interaction) items-center text-left transition-colors hover:bg-foreground/5",
        className,
      )}
    >
      {open ? (
        <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
      ) : (
        <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
      )}
      {children}
    </button>
  );
}

/** 單行標題的行內標記：程式碼成等寬籤、粗體成粗體，其餘文字原樣（任務標題、手動提示框、決策與風險標題）。 */
export function InlineMarks({ text }: { text: string }) {
  return (
    <>
      {text.split(/(`[^`]+`|\*\*[^*]+\*\*)/).map((part, i) =>
        /^`[^`]+`$/.test(part) ? (
          <code key={i} className="rounded bg-muted px-1 py-px font-mono text-[0.86em]">
            {part.slice(1, -1)}
          </code>
        ) : /^\*\*[^*]+\*\*$/.test(part) ? (
          <strong key={i}>{part.slice(2, -2)}</strong>
        ) : (
          part
        ),
      )}
    </>
  );
}

/** 灰底引言段（Summary）。 */
export function Lead({ children }: { children: ReactNode }) {
  return <div className="rounded-xl bg-muted px-4 py-3.5">{children}</div>;
}
