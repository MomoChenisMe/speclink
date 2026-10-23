import { useRef, useState, type Dispatch, type ReactNode, type SetStateAction } from "react";
import { Code2, FileText, Flag, ListChecks, MessagesSquare, PenTool, Rocket } from "lucide-react";

import type { ArchivedItem, ChangeItem, StationTicket, TicketStation } from "../adapter";
import { useI18n } from "../i18n";
import { parseTasks } from "../tasks";
import { settle, useGenerationLoad } from "../lib/useGenerationLoad";
import { Badge } from "./ui/badge";
import { Card } from "./ui/card";
import { StatusPill } from "./ui/status-pill";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import { TooltipProvider } from "./ui/tooltip";
import { changeDocSummary } from "./changeDocSummary";
import { DeltaCapSection } from "./DeltaBadges";
import { DesignDoc } from "./DesignDoc";
import type { PathReveal } from "./DocParts";
import { discussionDoc, DiscussionDocs, DiscussionStageRow, PromotedChangeChip } from "./DiscussionDetailPage";
import { discussionDocData, discussionDocProvenance, discussionDocSummary, discussionNames } from "./discussionDocSummary";
import { IdentityName, StatusRow, documentArtifact, type DocumentTarget } from "./DetailHeader";
import {
  DOC_TITLE_KEYS,
  DetailIdentity,
  DetailShell,
  DetailStatus,
  DetailTreeLabel,
  DocBody,
  DocHeader,
  DocTiles,
  changeDocTocKey,
} from "./DetailShell";
import { ImproveChip } from "./ImproveStamp";
import { isImproveKind } from "./improveStyle";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_PILL_TONE } from "./reviewStyle";
import { ProposalDoc } from "./ProposalDoc";
import { promotedRows, promotedRowsFromLinks } from "./PromotedDoc";
import { DocSkeleton } from "./skeletons";
import { SourceChipRow, type SourceLinkItem } from "./SourceDiscussionChip";
import { TaskList } from "./TaskList";
import { TicketTabBody, useStationTickets } from "./TicketView";
import { useDocNav } from "./useDocNav";
import { VERIFY_ICON, VERIFY_LABEL_KEY, VERIFY_PILL_TONE } from "./verifyStyle";

/** 頁目標（discriminated target 兩型同檔）：封存變更或封存討論。 */
export type ArchivedTarget =
  | { kind: "change"; datedName: string }
  | { kind: "discussion"; slug: string };

export interface ArchivedDetailPageProps {
  target: ArchivedTarget | null;
  /** 刷新世代——遞增即就地重載（不清空、latest-wins；未傳＝0）。 */
  refreshGen?: number;
  /** 讀取封存變更的 artifact 原文（dated name 定址）；缺件回 null。 */
  loadDocument: (datedName: string, artifact: string) => Promise<string | null>;
  /** 列出封存變更的 delta capability 名。 */
  loadCapabilities: (datedName: string) => Promise<string[]>;
  /** 封存目錄帶走的化石工單（spec「已封存詳情頁的工單分頁」）：對應分頁出現時載入；
   * 回 null 或 reject 皆為分頁空態。未提供時內容為空態。 */
  loadStationTicket?: (datedName: string, station: TicketStation) => Promise<StationTicket | null>;
  /** 讀取封存討論記錄全文（slug 定址）；缺席回 null。 */
  loadDiscussionDocument: (slug: string) => Promise<string | null>;
  /** 封存變更的來源討論（slug＋topic，宿主解析；缺席／空＝出身列無「來自」）。 */
  sourceDiscussions?: { slug: string; topic: string }[];
  /** 點來源討論籤：宿主開啟該討論的已封存詳情頁。 */
  onOpenDiscussion?: (slug: string) => void;
  /** 封存討論背景文件的討論籤（內文名稱籤與先前討論）：宿主依討論所在開頁——作用中開討論詳情頁、其餘開已封存
   * 詳情頁（`discussionSlugs` 含作用中的討論）。缺席時這些籤不可開。 */
  onOpenContextDiscussion?: (slug: string) => void;
  /** 封存討論的衍生變更籤項（宿主派生三態：已封存→副標封存日期、活躍→副標看板階段詞、
   * 已刪除→disabled「無封存記錄」）；缺席／空＝狀態卡的衍生列缺席。 */
  promotedChanges?: SourceLinkItem[];
  /** 點可點的衍生變更籤：宿主依子變更所在開啟已封存詳情頁或變更詳情頁。 */
  onOpenPromotedChange?: (name: string) => void;
  /** 封存時的審查結局（spec「已封存側的審查標示」）。 */
  reviewStatus?: ArchivedItem["reviewStatus"];
  /** 封存時的驗證結局（spec「已封存側的驗證標示」）；與審查結局並存。 */
  verifyStatus?: ArchivedItem["verifyStatus"];
  /** 封存討論的 kind（標示隨 kind 恆定）。 */
  discussionKind?: string | null;
  /** 封存討論的 topic（標題下方一行）。 */
  discussionTopic?: string;
  /** 出身列的建立者（"Name <email>"）；缺席＝該欄缺席。 */
  createdBy?: string | null;
  /** 出身列的建立日期 YYYY-MM-DD；缺席＝該欄缺席。 */
  created?: string;
  /** 出身列與狀態卡的封存日期 YYYY-MM-DD；缺席＝該欄缺席。 */
  archivedDate?: string;
  /** 內容卡標頭右端的文件動作（「開啟 ▾」）。 */
  renderDocumentActions?: (target: DocumentTarget | null) => ReactNode;
  /** 開該能力的規格詳情頁（規格文件能力頭的「看正式規格」）；未提供時不渲染該鈕。 */
  onOpenSpec?: (capability: string) => void;
  /** 提案影響段路徑籤的本機路徑動作（文案依平台）；未提供時只有複製鈕。 */
  pathReveal?: PathReveal;
  /** 作用中與已封存變更清單：兩份都有時封存討論的衍生列帶任務進度、兩站章與日期，否則退回 `promotedChanges`。 */
  changes?: ChangeItem[];
  archivedChanges?: ArchivedItem[];
  /** 正式規格名與全部討論 slug：封存討論背景的名稱籤。 */
  specNames?: string[];
  discussionSlugs?: string[];
}

type Doc = string | null | undefined;

/** 出身列各段：建立者、建立日期、「封存 日期」——任一缺席該段缺席。 */
function provenance(
  t: (key: string) => string,
  { createdBy, created, archivedDate }: Pick<ArchivedDetailPageProps, "createdBy" | "created" | "archivedDate">,
): ReactNode[] {
  return [
    createdBy && <IdentityName identity={createdBy} />,
    created && <span className="shrink-0">{created}</span>,
    archivedDate && <span className="shrink-0">{t("adetail.archivedOn").replace("{date}", archivedDate)}</span>,
  ];
}

/** 身分區的「已封存」籤：封存變更與封存討論共用。 */
function ArchivedPill() {
  const { t } = useI18n();
  return (
    <StatusPill data-archived-status tone="neutral">
      {t("ddetail.stationArchived")}
    </StatusPill>
  );
}

/** 分頁 → 封存目錄內的檔案（規格另依 capability 組路徑）。 */
const TAB_FILES: Record<string, string> = {
  proposal: "proposal.md",
  design: "design.md",
  tasks: "tasks.md",
  review: "review.md",
  verify: "verify.md",
};

/** 封存變更變體（design D3）。 */
function ArchivedChangeView({ datedName, ...props }: ArchivedDetailPageProps & { datedName: string }) {
  const { t, tn } = useI18n();
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  // undefined＝capability 清單與其規格文件尚未載完（與「無規格差異」的空物件分流）。
  const [specDocs, setSpecDocs] = useState<Record<string, string | null> | undefined>();
  const [tab, setTab] = useState("proposal");
  // 能力籤→規格段、D 籤→決策卡、「前往」→任務列的跳轉請求（與變更詳情頁同一套；換變更時以 key 重掛）。
  const nav = useDocNav(setTab);
  const bodyRef = useRef<HTMLDivElement>(null);
  const gen = props.refreshGen ?? 0;

  useGenerationLoad(gen, (isLatest) => {
    const fresh = (apply: (v: string | null) => void) => (v: string | null) => {
      if (isLatest()) apply(v);
    };
    const load = (artifact: string, apply: Dispatch<SetStateAction<Doc>>) =>
      void props.loadDocument(datedName, artifact).then(fresh(apply)).catch(settle(apply, null, isLatest));
    load("proposal.md", setProposal);
    load("design.md", setDesign);
    load("tasks.md", setTasksMd);
    void props
      .loadCapabilities(datedName)
      .then(async (caps) => {
        const entries = await Promise.all(
          caps.map(async (cap) => [cap, await props.loadDocument(datedName, `specs/${cap}/spec.md`)] as const),
        );
        if (isLatest()) setSpecDocs(Object.fromEntries(entries));
      })
      .catch(settle(setSpecDocs, {}, isLatest));
  });

  // 工單分頁條件（spec「已封存詳情頁的工單分頁」）：帶走化石工單而無章的結局才出現、才載入。
  const carriedReview = props.reviewStatus === "reviewedNotPassed";
  const carriedVerify = props.verifyStatus === "verifiedNotPassed";
  const tickets = useStationTickets(datedName, { review: carriedReview, verify: carriedVerify }, gen, props.loadStationTicket);

  // 結局籤（spec「已封存側的審查標示」「已封存側的驗證標示」）：只在「狀態」區的「品質關卡」
  // 列渲染——身分區只放「已封存」籤。
  const review = props.reviewStatus === "reviewed" || props.reviewStatus === "reviewedNotPassed" ? props.reviewStatus : null;
  const verify = props.verifyStatus === "verified" || props.verifyStatus === "verifiedNotPassed" ? props.verifyStatus : null;
  const outcomes = (
    <>
      {review && (
        <StatusPill data-review-outcome tone={REVIEW_PILL_TONE[review]} icon={REVIEW_ICON[review]}>
          {t(REVIEW_LABEL_KEY[review])}
        </StatusPill>
      )}
      {verify && (
        <StatusPill data-verify-outcome tone={VERIFY_PILL_TONE[verify]} icon={VERIFY_ICON[verify]}>
          {t(VERIFY_LABEL_KEY[verify])}
        </StatusPill>
      )}
    </>
  );

  const tasks = parseTasks(tasksMd);
  const specCaps = Object.keys(specDocs ?? {});
  const shownCap = nav.specCap && specCaps.includes(nav.specCap) ? nav.specCap : specCaps[0];
  const docTarget: DocumentTarget | null =
    tab === "specs"
      ? shownCap
        ? { doc: { archived: datedName, artifact: `specs/${shownCap}/spec.md` }, content: specDocs![shownCap] }
        : null
      : tab === "review" || tab === "verify"
        ? { doc: { archived: datedName, artifact: TAB_FILES[tab] } }
        : {
            doc: { archived: datedName, artifact: TAB_FILES[tab] },
            content: ({ proposal, design, tasks: tasksMd } as Record<string, Doc>)[tab] ?? null,
          };

  const docPath = documentArtifact(docTarget);
  const tocKey = changeDocTocKey(tab, { content: docTarget?.content, specs: specDocs, tickets });
  // 閱讀卡標題區（design D9）：唯讀、無排程；化石工單不給「可以蓋章」提示框。
  const summary = changeDocSummary(
    tab,
    {
      proposal,
      design,
      tasksMd,
      specDocs,
      change: null,
      tickets,
      readOnly: true,
      onLocateTask: nav.locateTask,
    },
    { t, tn },
  );
  const calloutSlot = summary.callout && <div data-doc-callout>{summary.callout}</div>;

  const tree = (
    <>
      {/* 標題為變更名、複製含日期前綴的封存目錄名；無動作——封存是唯讀定格。 */}
      <DetailIdentity
        title={datedName.replace(/^\d{4}-\d{2}-\d{2}-/, "")}
        copyLabel={t("archived.copyName")}
        copyValue={datedName}
        badges={<ArchivedPill />}
        provenance={provenance(t, props)}
        relations={
          (props.sourceDiscussions ?? []).length > 0
            ? [
                {
                  mark: "from",
                  label: t("detail.relFrom"),
                  children: (
                    <SourceChipRow
                      items={props.sourceDiscussions ?? []}
                      onOpen={(slug) => props.onOpenDiscussion?.(slug)}
                    />
                  ),
                },
              ]
            : []
        }
      />

      <DetailTreeLabel>{t("detail.tree")}</DetailTreeLabel>
      <TabsList variant="tree" aria-label={t("detail.tree")}>
        <TabsTrigger value="proposal">
          <FileText className="h-3.5 w-3.5" /> {t("common.tabProposal")}
        </TabsTrigger>
        <TabsTrigger value="design">
          <PenTool className="h-3.5 w-3.5" /> {t("common.tabDesign")}
        </TabsTrigger>
        <TabsTrigger value="tasks">
          <ListChecks className="h-3.5 w-3.5" /> {t("common.tabTasks")}
        </TabsTrigger>
        <TabsTrigger value="specs">
          <Code2 className="h-3.5 w-3.5" /> {t("common.tabSpecs")}
          {specCaps.length > 0 && (
            <Badge variant="secondary" className="ml-auto">
              {specCaps.length}
            </Badge>
          )}
        </TabsTrigger>
        {/* 條件式工單節點：規格之後、審查在前，與身分區結局籤同序。 */}
        {carriedReview && (
          <TabsTrigger value="review">
            {REVIEW_ICON.reviewedNotPassed} {t("drawer.tab.review")}
          </TabsTrigger>
        )}
        {carriedVerify && (
          <TabsTrigger value="verify">
            {VERIFY_ICON.verifiedNotPassed} {t("drawer.tab.verify")}
          </TabsTrigger>
        )}
      </TabsList>

      <DetailTreeLabel>{t("detail.status")}</DetailTreeLabel>
      <DetailStatus>
        <StatusRow mark="tasks" label={t("detail.statusTasks")}>
          {tasksMd !== undefined && (
            <span className="font-mono text-[11px] tabular-nums">
              {tasksMd === null ? t("adetail.tasksNone") : `${tasks.filter((l) => l.done).length}/${tasks.length}`}
            </span>
          )}
        </StatusRow>
        <StatusRow mark="quality" label={t("detail.statusQuality")} wrap>
          {review || verify ? outcomes : <span className="text-muted-foreground">{t("adetail.noQuality")}</span>}
        </StatusRow>
        <StatusRow mark="archived" label={t("adetail.statusArchived")}>
          {props.archivedDate && <span className="font-mono text-[11px] tabular-nums">{props.archivedDate}</span>}
        </StatusRow>
      </DetailStatus>
    </>
  );

  return (
    <TooltipProvider>
      <Tabs value={tab} onValueChange={setTab} orientation="vertical" activationMode="manual" className="h-full">
        <DetailShell data-archived-detail="change" tree={tree} activeDoc={tab} tocKey={tocKey} bodyRef={bodyRef}>
          <Card className="flex flex-col gap-5 p-6">
            <DocHeader
              title={t(DOC_TITLE_KEYS[tab])}
              badges={summary.badges}
              actions={props.renderDocumentActions?.(docTarget)}
              provenance={[docPath && <span className="font-mono">{docPath}</span>]}
            />
            <DocTiles tiles={summary.tiles} />
            {tab !== "proposal" && calloutSlot}
            <DocBody ref={bodyRef}>
              {/* 各文件三態：載入中畫骨架，空態文案只在載入完成後出。 */}
              <TabsContent value="proposal">
                {proposal === undefined ? (
                  <DocSkeleton />
                ) : (
                  <ProposalDoc
                    content={proposal}
                    empty={t("archived.noProposal")}
                    deltaCaps={specCaps}
                    onOpenCapability={nav.openSpec}
                    pathReveal={props.pathReveal}
                    afterCapabilities={tab === "proposal" ? calloutSlot : null}
                  />
                )}
              </TabsContent>
              <TabsContent value="design">
                {design === undefined ? (
                  <DocSkeleton />
                ) : (
                  <DesignDoc content={design} empty={t("archived.noDesign")} jump={nav.designJump} onJumped={nav.clearDesignJump} />
                )}
              </TabsContent>
              <TabsContent value="tasks">
                {/* 唯讀：勾選回呼不接——核取方塊 disabled、無把手、無批次工具列、整列不回應；展開與 D 籤照常。 */}
                {tasksMd === undefined ? (
                  <DocSkeleton />
                ) : (
                  <TaskList
                    markdown={tasksMd}
                    readOnly
                    onJumpDecision={nav.jumpDecision}
                    locate={nav.taskLocate}
                    onLocated={nav.clearTaskLocate}
                  />
                )}
              </TabsContent>
              <TabsContent value="specs">
                {specDocs === undefined ? (
                  <DocSkeleton />
                ) : specCaps.length === 0 ? (
                  <div className="py-6 text-sm text-muted-foreground">{t("archived.noSpecs")}</div>
                ) : (
                  Object.entries(specDocs).map(([cap, doc]) => (
                    <DeltaCapSection
                      key={cap}
                      cap={cap}
                      doc={doc}
                      jump={nav.specJump}
                      onJumped={nav.clearSpecJump}
                      onOpenSpec={props.onOpenSpec}
                    />
                  ))
                )}
              </TabsContent>
              {carriedReview && (
                <TabsContent value="review">
                  <TicketTabBody station="review" ticket={tickets.review} />
                </TabsContent>
              )}
              {carriedVerify && (
                <TabsContent value="verify">
                  <TicketTabBody station="verify" ticket={tickets.verify} />
                </TabsContent>
              )}
            </DocBody>
          </Card>
        </DetailShell>
      </Tabs>
    </TooltipProvider>
  );
}

/** 封存討論變體（design D3）。 */
function ArchivedDiscussionView({ slug, ...props }: ArchivedDetailPageProps & { slug: string }) {
  const { t, tn } = useI18n();
  const [doc, setDoc] = useState<Doc>();
  // 使用者選的文件；null＝依預設規則（記錄載入前後、結論有無都跟著走）。
  const [picked, setPicked] = useState<string | null>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  useGenerationLoad(props.refreshGen ?? 0, (isLatest) => {
    void props
      .loadDiscussionDocument(slug)
      .then((v) => {
        if (isLatest()) setDoc(v);
      })
      .catch(settle(setDoc, null, isLatest));
  });

  const promoted = props.promotedChanges ?? [];
  const { changes, archivedChanges, onOpenPromotedChange } = props;
  // 衍生列：兩份清單都有時與討論詳情頁同一套推導（開啟都委派宿主，宿主依所在開頁），否則退回宿主派生的三態。
  const rows =
    changes && archivedChanges
      ? promotedRows(
          promoted.map((item) => item.slug),
          changes,
          archivedChanges,
          t,
          onOpenPromotedChange,
          onOpenPromotedChange,
        )
      : promotedRowsFromLinks(promoted, onOpenPromotedChange);
  // 已封存：沒有記錄路徑段、不出「只差最後一刀」。
  const data = discussionDocData({
    text: doc,
    status: "archived",
    hold: false,
    created: props.created ?? null,
    recordPath: null,
    rows,
    names: discussionNames(slug, {
      specNames: props.specNames,
      changes,
      archivedChanges,
      discussionSlugs: props.discussionSlugs,
    }),
  });
  const { sections } = data;
  const roundCount = data.rounds?.length ?? null;
  // 衍生列與衍生籤共用：不可點（已刪除）者無開啟動作。
  const openPromoted = (item: SourceLinkItem) =>
    item.disabled ? undefined : () => props.onOpenPromotedChange?.(item.slug);
  const tab = discussionDoc(picked, sections);
  const summary = discussionDocSummary(tab, data, { t, tn });

  const tree = (
    <>
      <DetailIdentity
        title={slug}
        copyLabel={t("discussion.copySlug")}
        badges={
          <>
            <ArchivedPill />
            {isImproveKind(props.discussionKind) && <ImproveChip />}
          </>
        }
        subtitle={props.discussionTopic}
        provenance={provenance(t, props)}
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
              {roundCount !== null && (
                <Badge variant="secondary" className="ml-auto">
                  {roundCount}
                </Badge>
              )}
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
        <TabsTrigger value="promote">
          <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")}
          <Badge variant="secondary" className="ml-auto">
            {promoted.length}
          </Badge>
        </TabsTrigger>
      </TabsList>

      <DetailTreeLabel>{t("detail.status")}</DetailTreeLabel>
      <DetailStatus>
        <DiscussionStageRow current={3} />
        {promoted.length > 0 && (
          <StatusRow mark="promoted" label={t("ddetail.promotedLabel")} wrap>
            {promoted.map((item) => (
              <PromotedChangeChip
                key={item.slug}
                name={item.slug}
                badge={<span className="text-xs text-muted-foreground">{item.topic}</span>}
                onOpen={openPromoted(item)}
              />
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
          data-archived-detail="discussion"
          tree={tree}
          activeDoc={tab}
          tocKey={`${tab}\u0000${doc ?? ""}`}
          bodyRef={bodyRef}
        >
          <Card className="flex flex-col gap-5 p-6">
            {/* 封存討論的檔名帶封存日期而清單資料不帶：「開啟 ▾」只能複製內容。 */}
            <DocHeader
              title={t(DOC_TITLE_KEYS[tab])}
              actions={props.renderDocumentActions?.({ doc: null, content: doc ?? null })}
              provenance={discussionDocProvenance(tab, data, { t, tn })}
            />
            <DocTiles tiles={summary.tiles} />
            {summary.callout && <div data-doc-callout>{summary.callout}</div>}
            <DocBody ref={bodyRef}>
              <DiscussionDocs
                doc={doc}
                data={data}
                context={{
                  createdBy: props.createdBy,
                  kind: props.discussionKind,
                  onOpenSpec: props.onOpenSpec,
                  onOpenChange: onOpenPromotedChange,
                  onOpenArchivedChange: onOpenPromotedChange,
                  onOpenDiscussion: props.onOpenContextDiscussion,
                }}
                empty={{
                  conclusion: t("archived.noConclusion"),
                  rounds: t("archived.noRounds"),
                  context: t("archived.noContext"),
                }}
              />
            </DocBody>
          </Card>
        </DetailShell>
      </Tabs>
    </TooltipProvider>
  );
}

/**
 * 已封存詳情頁（spec「已封存項目以詳情頁檢視」；design D5）：與變更／討論詳情頁同一三欄骨架、
 * 唯讀——身分區帶「已封存」籤，不渲染標題列動作。
 * 目標一換即以 key 重掛對應變體——清空後全量載入；世代遞增就地重載。
 */
export function ArchivedDetailPage(props: ArchivedDetailPageProps) {
  const { target } = props;
  if (!target) return null;
  return target.kind === "change" ? (
    <ArchivedChangeView key={`c:${target.datedName}`} {...props} datedName={target.datedName} />
  ) : (
    <ArchivedDiscussionView key={`d:${target.slug}`} {...props} slug={target.slug} />
  );
}
