import { useEffect, useRef, useState, type ReactNode } from "react";
import { Archive, Check, Circle, FileText, Flag, MessagesSquare, Rocket } from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionItem } from "../adapter";
import { countPriorDiscussions, type DiscussionSections } from "../discussionModel";
import { useI18n } from "../i18n";
import { changeStage } from "../stage";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Card } from "./ui/card";
import { StatusPill, type StatusPillTone } from "./ui/status-pill";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { Markdown } from "./Markdown";
import { DocSkeleton } from "./skeletons";
import { discussionChipStage, isCollapsedPromoted } from "./DiscussionColumn";
import { IdentityName, STAGE_PILL_TONE, StatusRow, type DocumentTarget } from "./DetailHeader";
import {
  DOC_TITLE_KEYS,
  DetailActions,
  DetailIdentity,
  DetailShell,
  DetailStatus,
  DetailTreeLabel,
  DocBody,
  DocHeader,
  DocTiles,
  type DetailRelation,
} from "./DetailShell";
import { ImproveChip } from "./ImproveStamp";
import { PromotedDoc, promotedRows, rowStage } from "./PromotedDoc";
import { ConclusionDoc } from "./ConclusionDoc";
import { ContextDoc, type ContextDocProps } from "./ContextDoc";
import { RoundsDoc } from "./RoundsDoc";
import { discussionDocData, discussionDocProvenance, discussionDocSummary, type DiscussionDocData } from "./discussionDocSummary";
import { isImproveKind } from "./improveStyle";

export interface DiscussionDetailPageProps {
  discussion: DiscussionItem | null;
  /** 刷新世代——遞增即重載討論記錄內容（未傳＝0，行為等同僅開啟時載入）。 */
  refreshGen?: number;
  /** 記錄全文載入（slug 定址）。 */
  loadDocument: (slug: string) => Promise<string | null>;
  /** active change 清單（衍生變更現況派生）。 */
  changes: ChangeItem[];
  /** 已封存 change 清單（衍生變更已封存態派生）。 */
  archivedChanges: ArchivedItem[];
  /** 衍生變更籤與分頁的跳轉（開該變更的詳情頁）。 */
  onOpenChangeCard?: (name: string) => void;
  /** 開已封存的變更（封存目錄名）：衍生列、分刀卡與背景的已封存變更籤；缺席時這些不可開。 */
  onOpenArchivedChange?: (datedName: string) => void;
  /** 正式規格名（背景的名稱籤、記錄去向籤）。 */
  specNames?: string[];
  /** 全部討論的 slug（背景的名稱籤與先前討論）。 */
  discussionSlugs?: string[];
  /** 開規格詳情頁。 */
  onOpenSpec?: (capability: string) => void;
  /** 開討論詳情頁。 */
  onOpenDiscussion?: (slug: string) => void;
  /** 封存動詞——與討論卡同一確認流程（宿主接確認對話框）。未提供時不渲染（宿主以此表達
   * 「已封存」或 capability 缺口）。 */
  onArchiveDiscussion?: (slug: string) => void;
  /** 內容卡標頭右端的文件動作（design D4「開啟 ▾」）：各分頁皆為討論記錄檔，原文未載完為 null。 */
  renderDocumentActions?: (target: DocumentTarget | null) => ReactNode;
}

/** 標題列狀態籤（design D7）：判準與看板討論卡一致——尚無結論的已轉出、保留中的已轉出
 * 同為警示色；其餘已轉出與已結論為主色；討論中為中性。 */
function statusPill(d: DiscussionItem): { labelKey: string; tone: StatusPillTone } {
  if (d.status === "promoted") {
    if (d.concluded === false) return { labelKey: "discussion.statusPromotedUnconcluded", tone: "warning" };
    if (!isCollapsedPromoted(d)) return { labelKey: "discussion.statusPromotedHeld", tone: "warning" };
    return { labelKey: "ddrawer.stationPromoted", tone: "primary" };
  }
  if (d.status === "concluded") return { labelKey: "discussion.statusConcluded", tone: "primary" };
  return { labelKey: "discussion.statusOpen", tone: "neutral" };
}

/** 狀態卡的階段步驟列（討論中 → 已結論 → 已轉出 → 已封存）：現站主色籤、已過站打勾、
 * 未到站灰圈。討論詳情頁與已封存討論頁共用（後者現站恆為「已封存」）。 */
export function DiscussionStageRow({ current }: { current: number }) {
  const { t } = useI18n();
  const stations = [
    t("discussion.statusOpen"),
    t("discussion.statusConcluded"),
    t("ddrawer.stationPromoted"),
    t("ddetail.stationArchived"),
  ];
  return (
    <StatusRow mark="stage" label={t("ddetail.stageLabel")} wrap>
      {stations.map((label, i) => (
        <span key={label} className="flex items-center gap-1.5 text-xs">
          {i > 0 && <span className="text-muted-foreground/50">→</span>}
          {i === current ? (
            <StatusPill tone="primary" aria-current="step">
              {label}
            </StatusPill>
          ) : i < current ? (
            <span className="inline-flex items-center gap-1 text-muted-foreground">
              <Check aria-hidden="true" className="h-3 w-3" />
              {label}
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-muted-foreground/50">
              <Circle aria-hidden="true" className="h-2.5 w-2.5" />
              {label}
            </span>
          )}
        </span>
      ))}
    </StatusRow>
  );
}

/** 預設分頁＝讀者第一想看的：結論非空（去除鷹架註解後有內容）開結論，否則背景。 */
function defaultDiscussionTab(sections: DiscussionSections | null): "conclusion" | "context" {
  const hasConclusion = !!sections && sections.conclusion.replace(/<!--[\s\S]*?-->/g, "").trim().length > 0;
  return hasConclusion ? "conclusion" : "context";
}

/** 受控文件的實際值：所選文件不在目前的節點組（記錄切分前後不同）時退回預設——切分成功取
 * `defaultDiscussionTab`，失敗取「記錄」。討論詳情頁與已封存討論頁共用。 */
export function discussionDoc(picked: string | null, sections: DiscussionSections | null): string {
  const docs = sections ? ["conclusion", "rounds", "context", "promote"] : ["record", "promote"];
  if (picked && docs.includes(picked)) return picked;
  return sections ? defaultDiscussionTab(sections) : "record";
}

/** 衍生變更（列與籤共用）：變更名、狀態標示；`onOpen` 缺席＝不可開啟。 */
interface PromotedChangeProps {
  name: string;
  badge: ReactNode;
  onOpen?: () => void;
}

/** 狀態卡「衍生變更」列的一顆籤：變更名鈕（不可開啟時停用）＋狀態標示。討論詳情頁與已封存討論頁共用。 */
export function PromotedChangeChip({ name, badge, onOpen }: PromotedChangeProps) {
  return (
    <span data-promoted-chip={name} className="inline-flex items-center gap-1">
      <Button
        type="button"
        variant="ghost"
        size="sm"
        disabled={!onOpen}
        className="h-auto rounded-full bg-muted px-2 py-0.5 font-mono text-xs font-medium hover:bg-accent"
        onClick={onOpen}
      >
        {name}
      </Button>
      {badge}
    </span>
  );
}

/** 背景的已封存變更籤以變更名開啟：換成封存目錄名再交給宿主。 */
export function openArchivedByName(archived: ArchivedItem[], open: (datedName: string) => void) {
  return (name: string) => {
    const hit = archived.find((a) => a.name === name);
    if (hit) open(hit.datedName);
  };
}

/**
 * 討論記錄四份文件的閱讀卡內文（desktop-discussion-doc-views design D7）：記錄切分成功時結論、討論過程、背景
 * 各自的渲染器，失敗時「記錄」整篇 Markdown（載入中畫骨架）；衍生變更一律 PromotedDoc。討論詳情頁與已封存
 * 討論頁共用。
 */
export function DiscussionDocs({
  doc,
  data,
  context,
  empty,
}: {
  doc: string | null | undefined;
  data: DiscussionDocData;
  context: Omit<ContextDocProps, "text" | "empty">;
  empty: { conclusion: string; rounds: string; context: string };
}) {
  const { t } = useI18n();
  return (
    <>
      {data.sections ? (
        <>
          <TabsContent value="conclusion">
            <ConclusionDoc
              text={data.sections.conclusion}
              empty={empty.conclusion}
              stageOf={rowStage(data.rows)}
              specNames={context.specNames}
              onOpenSpec={context.onOpenSpec}
            />
          </TabsContent>
          <TabsContent value="rounds">
            {/* 換討論時展開狀態歸零。 */}
            <RoundsDoc key={context.slug} text={data.sections.rounds} empty={empty.rounds} />
          </TabsContent>
          <TabsContent value="context">
            <ContextDoc {...context} text={data.sections.context} empty={empty.context} />
          </TabsContent>
        </>
      ) : (
        // 區段缺失或格式非預期：整篇以單一檢視退回（不報錯）；載入中畫骨架、空態文案只在載入完成後出。
        <TabsContent value="record">
          {doc === undefined ? <DocSkeleton /> : <Markdown content={doc} empty={t("common.noContent")} />}
        </TabsContent>
      )}
      <TabsContent value="promote">
        <PromotedDoc rows={data.rows} cuts={data.cuts} />
      </TabsContent>
    </>
  );
}

/**
 * 討論詳情頁（design D5；spec「詳情頁的文件樹、閱讀卡與本頁目錄」）：左欄身分區（slug＋複製＋
 * 狀態籤＋改進籤、topic、出身、「關聯」列）、「文件」節點 結論／討論過程 N／背景／衍生變更 N
 * （結論非空時預設開結論）、「狀態」列（階段步驟列、保留在途、衍生變更籤）；封存動作經
 * `DetailActions` 進標題列。衍生變更唯讀——無轉出動作（promote 已自 GUI 撤除，轉出改由
 * CLI／agent）。記錄格式非預期時整篇單一檢視退回（節點為 記錄／衍生變更）。
 */
export function DiscussionDetailPage({
  discussion,
  refreshGen,
  loadDocument,
  changes,
  archivedChanges,
  onOpenChangeCard,
  onOpenArchivedChange,
  specNames,
  discussionSlugs,
  onOpenSpec,
  onOpenDiscussion,
  onArchiveDiscussion,
  renderDocumentActions,
}: DiscussionDetailPageProps) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<string | null | undefined>();
  // 使用者選的文件；null＝依預設規則（記錄載入前後、結論有無都跟著走）。
  const [picked, setPicked] = useState<string | null>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const slug = discussion?.slug ?? null;
  const gen = refreshGen ?? 0;
  // latest-wins：回應帶發起序號，落後即丟棄（涵蓋世代與換討論的交錯）。
  const requestSeq = useRef(0);
  const loadedGen = useRef(-1);

  const loadRecord = (g: number, target: string, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) setDoc(undefined);
    void loadDocument(target)
      .then((v) => {
        if (requestSeq.current === seq) setDoc(v);
      })
      // 失敗收斂：還沒有東西可顯示才落終態空文案（undefined 停著＝永久骨架）；
      // 已有內容維持前值——重載的短暫失敗不得抹成假空態。
      .catch(() => {
        if (requestSeq.current === seq) setDoc((prev) => (prev === undefined ? null : prev));
      });
  };

  // 開啟／換討論：清空後載入，文件回到預設。
  useEffect(() => {
    if (!slug) return;
    setPicked(null);
    loadRecord(gen, slug, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [slug]);

  // 外部世代重載（add-round／conclude／轉出後）：不清空、回應到達後就地替換，分頁選擇不重置。
  useEffect(() => {
    if (!slug) return;
    if (gen <= loadedGen.current) return;
    loadRecord(gen, slug, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [slug, gen]);

  if (!discussion) return null;

  const rows = promotedRows(discussion.promotedTo, changes, archivedChanges, t, onOpenChangeCard, onOpenArchivedChange);
  const names = {
    specs: specNames ?? [],
    active: changes.map((c) => c.name),
    archived: archivedChanges.map((a) => a.name),
    discussions: discussionSlugs ?? [],
    self: discussion.slug,
  };
  const recordPath = `openspec/discussions/${discussion.slug}.md`;
  const data = discussionDocData({
    text: doc,
    status: discussion.status,
    hold: discussion.hold === true,
    created: discussion.created ?? null,
    recordPath,
    rows,
    names,
  });
  const { sections } = data;
  const pill = statusPill(discussion);
  const hold = discussion.hold === true;

  const childStage = (name: string): { label: string; tone: StatusPillTone; alive: boolean } => {
    const active = changes.find((c) => c.name === name);
    const label = t(discussionChipStage(name, changes, archivedChanges));
    if (active) return { label, tone: STAGE_PILL_TONE[changeStage(active)], alive: true };
    if (archivedChanges.some((a) => a.name === name)) return { label, tone: "neutral", alive: false };
    return { label, tone: "destructive", alive: false };
  };
  // 衍生列與衍生籤共用：階段籤，與只有作用中的子變更可開啟。
  const stageBadge = (name: string) => {
    const stage = childStage(name);
    return (
      <StatusPill tone={stage.tone} className={stage.tone === "destructive" ? "line-through" : undefined}>
        {stage.label}
      </StatusPill>
    );
  };
  const openChild = (name: string) => (childStage(name).alive ? () => onOpenChangeCard?.(name) : undefined);

  // 階段步驟列：現站由 status 派生；「已封存」站留給已封存討論頁。
  const stationIndex = discussion.status === "promoted" ? 2 : discussion.status === "concluded" ? 1 : 0;

  // 封存動詞：concluded 才可及；保留在途（hold）時停用並說明最後一刀轉出後自動封存。
  const archiveAction =
    onArchiveDiscussion && (discussion.status === "concluded" || hold) ? (
      hold ? (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="inline-flex" tabIndex={0}>
              <Button variant="outline" size="sm" disabled title={t("ddetail.holdHint")}>
                <Archive className="h-3.5 w-3.5" /> {t("common.archive")}
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>{t("ddetail.holdHint")}</TooltipContent>
        </Tooltip>
      ) : (
        <Button variant="outline" size="sm" onClick={() => onArchiveDiscussion(discussion.slug)}>
          <Archive className="h-3.5 w-3.5" /> {t("common.archive")}
        </Button>
      )
    ) : undefined;

  const promoteCount = discussion.promotedTo.length;
  // 受控文件：關聯列要能切到背景。
  const tab = discussionDoc(picked, sections);
  const summary = discussionDocSummary(tab, data, t);
  const prior = sections ? countPriorDiscussions(sections.context) : null;
  const relations: DetailRelation[] =
    prior === null
      ? []
      : [
          {
            mark: "related",
            label: t("ddetail.relations"),
            children: (
              <button type="button" className="min-w-0" onClick={() => setPicked("context")}>
                <Badge variant="outline" className="hover:bg-accent">
                  {t("ddetail.priorDiscussions").replace("{n}", String(prior))}
                </Badge>
              </button>
            ),
          },
        ];

  const promoteTrigger = (
    <TabsTrigger value="promote">
      <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")}
      <Badge variant="secondary" className="ml-auto">
        {promoteCount}
      </Badge>
    </TabsTrigger>
  );

  const tree = (
    <>
      {/* slug（檔名）為題＋複製鈕——與變更詳情頁「名稱＋複製鈕」對稱（LANGUAGE.md 受控例外）；
          topic 為其下最多三行。 */}
      <DetailIdentity
        title={discussion.slug}
        copyLabel={t("discussion.copySlug")}
        badges={
          <>
            <StatusPill data-discussion-status tone={pill.tone}>
              {t(pill.labelKey)}
            </StatusPill>
            {/* 改進標示（spec「討論詳情頁的改進標示」）：與卡片小章同一詞條。 */}
            {isImproveKind(discussion.kind) && <ImproveChip />}
          </>
        }
        subtitle={discussion.topic}
        provenance={[
          discussion.createdBy && <IdentityName identity={discussion.createdBy} />,
          <span className="shrink-0 tabular-nums">{t("common.rounds").replace("{n}", String(discussion.rounds))}</span>,
          discussion.created && <span className="shrink-0">{discussion.created}</span>,
        ]}
        relations={relations}
      />

      <DetailTreeLabel>{t("detail.tree")}</DetailTreeLabel>
      <TabsList variant="tree" aria-label={t("detail.tree")}>
        {sections ? (
          <>
            <TabsTrigger value="conclusion">
              <Flag className="h-3.5 w-3.5" /> {t("ddrawer.tabConclusion")}
            </TabsTrigger>
            <TabsTrigger value="rounds">
              <MessagesSquare className="h-3.5 w-3.5" /> {t("ddrawer.tabRounds")}
              <Badge variant="secondary" className="ml-auto">
                {discussion.rounds}
              </Badge>
            </TabsTrigger>
            <TabsTrigger value="context">
              <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabContext")}
            </TabsTrigger>
          </>
        ) : (
          <TabsTrigger value="record">
            <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabRecord")}
          </TabsTrigger>
        )}
        {promoteTrigger}
      </TabsList>

      <DetailTreeLabel>{t("detail.status")}</DetailTreeLabel>
      <DetailStatus>
        <DiscussionStageRow current={stationIndex} />
        {hold && (
          <StatusRow mark="hold" label={t("ddetail.holdLabel")} wrap>
            <span className="text-muted-foreground">{t("ddetail.holdHint")}</span>
          </StatusRow>
        )}
        {promoteCount > 0 && (
          <StatusRow mark="promoted" label={t("ddetail.promotedLabel")} wrap>
            {discussion.promotedTo.map((name) => (
              <PromotedChangeChip key={name} name={name} badge={stageBadge(name)} onOpen={openChild(name)} />
            ))}
          </StatusRow>
        )}
      </DetailStatus>
    </>
  );

  return (
    <TooltipProvider>
      <Tabs value={tab} onValueChange={setPicked} orientation="vertical" activationMode="manual" className="h-full">
        <DetailShell
          data-discussion-detail
          tree={tree}
          activeDoc={`${discussion.slug}\u0000${tab}`}
          tocKey={`${tab}\u0000${doc ?? ""}`}
          bodyRef={bodyRef}
        >
          {archiveAction && <DetailActions>{archiveAction}</DetailActions>}
          <Card className="flex flex-col gap-5 p-6">
            <DocHeader
              title={t(DOC_TITLE_KEYS[tab])}
              actions={renderDocumentActions?.({ doc: { discussion: discussion.slug }, content: doc ?? null })}
              provenance={discussionDocProvenance(tab, data, t)}
            />
            <DocTiles tiles={summary.tiles} />
            {summary.callout && <div data-doc-callout>{summary.callout}</div>}
            <DocBody ref={bodyRef}>
              <DiscussionDocs
                doc={doc}
                data={data}
                context={{
                  slug: discussion.slug,
                  created: discussion.created,
                  createdBy: discussion.createdBy,
                  kind: discussion.kind,
                  recordPath,
                  specNames: names.specs,
                  activeNames: names.active,
                  archivedNames: names.archived,
                  discussionSlugs: names.discussions,
                  onOpenSpec,
                  onOpenChange: onOpenChangeCard,
                  onOpenArchivedChange: onOpenArchivedChange && openArchivedByName(archivedChanges, onOpenArchivedChange),
                  onOpenDiscussion,
                }}
                empty={{
                  conclusion: t("ddrawer.noConclusion"),
                  rounds: t("ddrawer.noRounds"),
                  context: t("ddrawer.noContext"),
                }}
              />
            </DocBody>
          </Card>
        </DetailShell>
      </Tabs>
    </TooltipProvider>
  );
}
