import { useState, type Dispatch, type ReactNode, type SetStateAction } from "react";
import { Code2, FileText, Flag, ListChecks, MessagesSquare, PenTool, Rocket } from "lucide-react";

import type { ArchivedItem, StationTicket, TicketStation } from "../adapter";
import { useI18n } from "../i18n";
import { parseTasks } from "../tasks";
import { settle, useGenerationLoad } from "../lib/useGenerationLoad";
import { Badge } from "./ui/badge";
import { StatusPill } from "./ui/status-pill";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import { TooltipProvider } from "./ui/tooltip";
import { DeltaSpecView } from "./DeltaBadges";
import {
  ConclusionView,
  defaultDiscussionTab,
  DiscussionStageRow,
  PromotedChangeChip,
  PromotedChangeRow,
  RoundsView,
  splitDiscussionSections,
  splitRounds,
} from "./DiscussionDetailPage";
import {
  DetailContentCard,
  DetailHeader,
  DetailPageFrame,
  IdentityName,
  StatusCard,
  StatusRow,
  type DocumentTarget,
} from "./DetailHeader";
import { ImproveChip } from "./ImproveStamp";
import { isImproveKind } from "./improveStyle";
import { Markdown } from "./Markdown";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_PILL_TONE } from "./reviewStyle";
import { SectionedDoc } from "./SectionedDoc";
import { DocSkeleton } from "./skeletons";
import { SourceChipRow, type SourceLinkItem } from "./SourceDiscussionChip";
import { TaskList } from "./TaskList";
import { TicketTabBody, useStationTickets } from "./TicketView";
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
  const { t } = useI18n();
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  // undefined＝capability 清單與其規格文件尚未載完（與「無規格差異」的空物件分流）。
  const [specDocs, setSpecDocs] = useState<Record<string, string | null> | undefined>();
  const [tab, setTab] = useState("proposal");
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

  // 結局籤（spec「已封存側的審查標示」「已封存側的驗證標示」）：標題列與狀態卡共用。
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
  const docTarget: DocumentTarget | null =
    tab === "specs"
      ? specCaps.length > 0
        ? { doc: { archived: datedName, artifact: `specs/${specCaps[0]}/spec.md` }, content: specDocs![specCaps[0]] }
        : null
      : tab === "review" || tab === "verify"
        ? { doc: { archived: datedName, artifact: TAB_FILES[tab] } }
        : {
            doc: { archived: datedName, artifact: TAB_FILES[tab] },
            content: ({ proposal, design, tasks: tasksMd } as Record<string, Doc>)[tab] ?? null,
          };

  return (
    <TooltipProvider>
      <DetailPageFrame data-archived-detail="change">
        {/* 標題為變更名、複製含日期前綴的封存目錄名；無動作區——封存是唯讀定格。 */}
        <DetailHeader
          title={datedName.replace(/^\d{4}-\d{2}-\d{2}-/, "")}
          copyLabel={t("archived.copyName")}
          copyValue={datedName}
          badges={review || verify ? outcomes : undefined}
          provenance={[
            ...provenance(t, props),
            (props.sourceDiscussions ?? []).length > 0 && (
              <SourceChipRow
                label={t("rdrawer.fromDiscussion")}
                items={props.sourceDiscussions ?? []}
                onOpen={(slug) => props.onOpenDiscussion?.(slug)}
              />
            ),
          ]}
        />

        <StatusCard>
          <StatusRow mark="tasks" label={t("detail.statusTasks")}>
            {tasksMd !== undefined && (
              <span className="font-mono text-xs tabular-nums">
                {tasksMd === null
                  ? t("adetail.tasksNone")
                  : `${tasks.filter((l) => l.done).length}/${tasks.length}`}
              </span>
            )}
          </StatusRow>
          <StatusRow mark="quality" label={t("detail.statusQuality")}>
            {review || verify ? outcomes : <span className="text-muted-foreground">{t("adetail.noQuality")}</span>}
          </StatusRow>
          <StatusRow mark="archived" label={t("adetail.statusArchived")}>
            {props.archivedDate && <span className="font-mono text-xs tabular-nums">{props.archivedDate}</span>}
          </StatusRow>
        </StatusCard>

        <Tabs value={tab} onValueChange={setTab}>
          <TabsList variant="card" actions={props.renderDocumentActions?.(docTarget)}>
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
                <Badge variant="secondary" className="ml-1">
                  {specCaps.length}
                </Badge>
              )}
            </TabsTrigger>
            {/* 條件式工單分頁：規格之後、審查在前，與標題列結局籤同序。 */}
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
          <DetailContentCard>
            {/* 各分頁三態：載入中畫骨架，空態文案只在載入完成後出。 */}
            <TabsContent value="proposal">
              {proposal === undefined ? <DocSkeleton /> : <SectionedDoc content={proposal} empty={t("archived.noProposal")} />}
            </TabsContent>
            <TabsContent value="design">
              {design === undefined ? <DocSkeleton /> : <SectionedDoc content={design} empty={t("archived.noDesign")} />}
            </TabsContent>
            <TabsContent value="tasks">
              {/* 唯讀：不接任何回呼——核取方塊 disabled、無把手、無批次工具列、整列不回應。 */}
              {tasksMd === undefined ? <DocSkeleton /> : <TaskList markdown={tasksMd} readOnly />}
            </TabsContent>
            <TabsContent value="specs">
              {specDocs === undefined ? (
                <DocSkeleton />
              ) : specCaps.length === 0 ? (
                <div className="py-6 text-sm text-muted-foreground">{t("archived.noSpecs")}</div>
              ) : (
                Object.entries(specDocs).map(([cap, doc]) => (
                  <div key={cap} className="mb-4">
                    <div className="mb-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground">{cap}</div>
                    <DeltaSpecView markdown={doc} />
                  </div>
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
          </DetailContentCard>
        </Tabs>
      </DetailPageFrame>
    </TooltipProvider>
  );
}

/** 封存討論變體（design D3）。 */
function ArchivedDiscussionView({ slug, ...props }: ArchivedDetailPageProps & { slug: string }) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<Doc>();
  useGenerationLoad(props.refreshGen ?? 0, (isLatest) => {
    void props
      .loadDiscussionDocument(slug)
      .then((v) => {
        if (isLatest()) setDoc(v);
      })
      .catch(settle(setDoc, null, isLatest));
  });

  const sections = doc ? splitDiscussionSections(doc) : null;
  const promoted = props.promotedChanges ?? [];
  const roundCount = sections ? (splitRounds(sections.rounds)?.length ?? null) : null;
  const defaultTab = defaultDiscussionTab(sections);
  // 衍生列與衍生籤共用：不可點（已刪除）者無開啟動作。
  const openPromoted = (item: SourceLinkItem) =>
    item.disabled ? undefined : () => props.onOpenPromotedChange?.(item.slug);
  // 封存討論的檔名帶封存日期而清單資料不帶：「開啟 ▾」只能複製內容。
  const tabsActions = props.renderDocumentActions?.({ doc: null, content: doc ?? null });

  const promoteTrigger = (
    <TabsTrigger value="promote">
      <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")} {promoted.length}
    </TabsTrigger>
  );
  const promotePane = (
    <TabsContent value="promote">
      {promoted.length === 0 ? (
        <p className="py-4 text-sm text-muted-foreground">{t("ddrawer.notPromoted")}</p>
      ) : (
        <div className="flex flex-col gap-1.5">
          {promoted.map((item) => (
            <PromotedChangeRow
              key={item.slug}
              name={item.slug}
              badge={<StatusPill tone="neutral">{item.topic}</StatusPill>}
              onOpen={openPromoted(item)}
            />
          ))}
        </div>
      )}
    </TabsContent>
  );

  return (
    <TooltipProvider>
      <DetailPageFrame data-archived-detail="discussion">
        <DetailHeader
          title={slug}
          copyLabel={t("discussion.copySlug")}
          badges={
            <>
              <StatusPill data-archived-status tone="neutral">
                {t("ddetail.stationArchived")}
              </StatusPill>
              {isImproveKind(props.discussionKind) && <ImproveChip />}
            </>
          }
          subtitle={props.discussionTopic}
          provenance={provenance(t, props)}
        />

        <StatusCard>
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
        </StatusCard>

        {sections ? (
          // key 區分分頁組與預設分頁——自載入中 fallback 切換過來時，Radix Tabs 的未受控
          // state 不得停在不存在的分頁值。
          <Tabs key={`sections-${defaultTab}`} defaultValue={defaultTab}>
            <TabsList variant="card" actions={tabsActions}>
              <TabsTrigger value="conclusion">
                <Flag className="h-3.5 w-3.5" /> {t("ddrawer.tabConclusion")}
              </TabsTrigger>
              <TabsTrigger value="rounds">
                <MessagesSquare className="h-3.5 w-3.5" /> {t("ddrawer.tabRounds")} {roundCount ?? ""}
              </TabsTrigger>
              <TabsTrigger value="context">
                <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabContext")}
              </TabsTrigger>
              {promoteTrigger}
            </TabsList>
            <DetailContentCard>
              <TabsContent value="conclusion">
                <ConclusionView text={sections.conclusion} empty={t("archived.noConclusion")} />
              </TabsContent>
              <TabsContent value="rounds">
                <RoundsView text={sections.rounds} empty={t("archived.noRounds")} />
              </TabsContent>
              <TabsContent value="context">
                <Markdown content={sections.context} empty={t("archived.noContext")} />
              </TabsContent>
              {promotePane}
            </DetailContentCard>
          </Tabs>
        ) : (
          <Tabs key="fallback" defaultValue="record">
            <TabsList variant="card" actions={tabsActions}>
              <TabsTrigger value="record">
                <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabRecord")}
              </TabsTrigger>
              {promoteTrigger}
            </TabsList>
            {/* 區段缺失或格式非預期：整篇單一檢視退回（不報錯）；載入中畫骨架。 */}
            <DetailContentCard>
              <TabsContent value="record">
                {doc === undefined ? <DocSkeleton /> : <Markdown content={doc} empty={t("common.noContent")} />}
              </TabsContent>
              {promotePane}
            </DetailContentCard>
          </Tabs>
        )}
      </DetailPageFrame>
    </TooltipProvider>
  );
}

/**
 * 已封存詳情頁（spec「已封存項目以詳情頁檢視」；design D3）：與變更／討論詳情頁同骨架、唯讀。
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
