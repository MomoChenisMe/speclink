import type { KeyboardEvent, ReactNode, SyntheticEvent } from "react";

import { splitLead, type ContextModel } from "../discussionModel";
import { useI18n } from "../i18n";
import { CopyButton } from "./CopyButton";
import { displayName } from "./DetailHeader";
import { DocSection, Lead, NameChip, ROW_MD_CLS } from "./DocParts";
import { isImproveKind } from "./improveStyle";
import { Markdown, type NameChip as NameChipRef } from "./Markdown";

export interface ContextDocProps {
  /** 背景的推導（`contextModel`）：收尾行、關聯與比對所用的名稱清單（本討論的 slug 為 `names.self`）。 */
  model: ContextModel;
  empty?: string;
  created?: string | null;
  createdBy?: string | null;
  kind?: string | null;
  /** 記錄檔路徑；已封存討論頁為 null（記錄列不出檔案列）。 */
  recordPath: string | null;
  onOpenSpec?: (capability: string) => void;
  onOpenChange?: (name: string) => void;
  /** 開已封存變更（以變更名；宿主自己換成封存目錄名）。 */
  onOpenArchivedChange?: (name: string) => void;
  onOpenDiscussion?: (slug: string) => void;
}

type ChipKind = "spec" | "change" | "archived" | "discussion";

/** 內文名稱籤的款式：由容器以任意變體給（籤是 Markdown 外掛產生的 span）。 */
const NAME_CHIP_HOST_CLS =
  "[&_.name-chip]:cursor-(--cursor-interaction) [&_.name-chip]:rounded [&_.name-chip]:border [&_.name-chip]:border-border [&_.name-chip]:bg-background [&_.name-chip]:px-1 [&_.name-chip]:font-mono [&_.name-chip]:text-[0.86em] [&_.name-chip]:text-primary [&_.name-chip:hover]:underline";

type MetaRow = { key: string; label: string; value: ReactNode };

/** 鍵值列：左鍵灰字、右值任意內容。 */
function MetaRows({ rows }: { rows: MetaRow[] }) {
  return (
    <dl className="divide-y divide-border/70">
      {rows.map((r) => (
        <div key={r.key} data-context-row={r.key} className="flex items-start gap-4 py-2 first:pt-0 last:pb-0">
          <dt className="w-28 shrink-0 text-sm text-muted-foreground">{r.label}</dt>
          <dd className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">{r.value}</dd>
        </div>
      ))}
    </dl>
  );
}

/**
 * 背景文件（desktop-discussion-doc-views design D4；spec「討論詳情頁檢視與轉出變更」）：收尾四行剝離後分三段——
 * 起因與範圍（灰底引言＋帶名稱籤的內文）、關聯（收尾行與內文命中的聯集）、記錄（建立、種類、檔案）。
 * 內文名稱籤只給有開啟回呼的種類。討論詳情頁與已封存討論頁共用。
 */
export function ContextDoc(props: ContextDocProps) {
  const { t } = useI18n();
  const { meta, rel, names } = props.model;
  // 背景全空白：沒有內文、也沒有任何收尾行。
  if (!meta.body && meta.specs === null && meta.changes === null && meta.prior === null && meta.sourceDoc === null) {
    return <Markdown content="" empty={props.empty} />;
  }
  const { lead, rest } = splitLead(meta.body);
  const open: Record<ChipKind, ((name: string) => void) | undefined> = {
    spec: props.onOpenSpec,
    change: props.onOpenChange,
    archived: props.onOpenArchivedChange,
    discussion: props.onOpenDiscussion,
  };
  const chips: NameChipRef[] = (
    [
      ["spec", rel.hits.specs],
      ["change", rel.hits.changes],
      ["archived", rel.hits.archived],
      ["discussion", rel.hits.discussions],
    ] as const
  ).flatMap(([kind, names]) => (open[kind] ? names.map((name) => ({ kind, name })) : []));

  // 內文名稱籤的點擊與鍵盤啟動：籤以 `kind:name` 標在 data 屬性上；事件不是來自籤時不攔（連結、複製鈕照常）。
  const chipOf = (e: SyntheticEvent) =>
    (e.target as HTMLElement).closest?.("[data-name-chip]")?.getAttribute("data-name-chip") ?? null;
  const activate = (value: string) => {
    const at = value.indexOf(":");
    open[value.slice(0, at) as ChipKind]?.(value.slice(at + 1));
  };
  const onClick = (e: SyntheticEvent) => {
    const value = chipOf(e);
    if (value) activate(value);
  };
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    const value = chipOf(e);
    if (!value) return;
    e.preventDefault();
    activate(value);
  };

  const chipRow = (names: string[], known: string[], onOpen?: (name: string) => void) =>
    names.map((name) => (
      <NameChip key={name} name={name} onClick={onOpen && known.includes(name) ? () => onOpen(name) : undefined} />
    ));
  const relations = ([
    rel.specs.length > 0 && {
      key: "specs",
      label: t("context.relSpecs"),
      value: chipRow(rel.specs, names.specs, props.onOpenSpec),
    },
    rel.changes.length > 0 && {
      key: "changes",
      label: t("context.relChanges"),
      value: chipRow(rel.changes, names.active, props.onOpenChange),
    },
    rel.archived.length > 0 && {
      key: "archived",
      label: t("context.relArchived"),
      value: chipRow(rel.archived, names.archived, props.onOpenArchivedChange),
    },
    rel.prior.length > 0 && {
      key: "prior",
      label: t("context.relPrior"),
      value: chipRow(rel.prior, names.discussions, props.onOpenDiscussion),
    },
    meta.sourceDoc !== null && {
      key: "source",
      label: t("context.relSource"),
      value: meta.sourceDoc ? <span className="font-mono text-sm break-all">{meta.sourceDoc}</span> : t("context.none"),
    },
  ] as (MetaRow | false)[]).filter((r): r is MetaRow => r !== false);

  const created = [props.created, props.createdBy && displayName(props.createdBy)].filter(Boolean).join(" · ");
  const record = ([
    !!created && { key: "created", label: t("context.created"), value: created },
    {
      key: "kind",
      label: t("context.kind"),
      value: t(isImproveKind(props.kind) ? "context.kindImprove" : "context.kindGeneral"),
    },
    !!props.recordPath && {
      key: "file",
      label: t("context.file"),
      value: (
        <>
          <span className="font-mono text-sm break-all">{props.recordPath}</span>
          <CopyButton value={props.recordPath!} label={t("common.copy")} />
        </>
      ),
    },
  ] as (MetaRow | false)[]).filter((r): r is MetaRow => r !== false);

  return (
    <div>
      {(lead || rest) && (
        <DocSection sectionKey="context.intro" title={t("context.intro")}>
          {/* 籤由 Markdown 外掛產生、不是元件：點擊與鍵盤在容器上攔。 */}
          <div className={`flex flex-col gap-3 ${NAME_CHIP_HOST_CLS}`} onClick={onClick} onKeyDown={onKeyDown}>
            {lead && (
              <div data-context-lead>
                <Lead>
                  <Markdown content={lead} nameChips={chips} className={ROW_MD_CLS} />
                </Lead>
              </div>
            )}
            {rest && <Markdown content={rest} nameChips={chips} />}
          </div>
        </DocSection>
      )}
      {relations.length > 0 && (
        <DocSection sectionKey="context.relations" title={t("context.relations")}>
          <MetaRows rows={relations} />
        </DocSection>
      )}
      <DocSection sectionKey="context.record" title={t("context.record")}>
        <MetaRows rows={record} />
      </DocSection>
    </div>
  );
}
