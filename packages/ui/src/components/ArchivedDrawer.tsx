import { useEffect, useRef, useState } from "react";
import { Check, Code2, Copy, FileText, ListChecks, Maximize2, Minimize2, PenTool } from "lucide-react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { SourceChipRow } from "./SourceDiscussionChip";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { DeltaSpecView } from "./DeltaBadges";
import { ConclusionView, RoundsView, splitDiscussionSections } from "./DiscussionDrawer";
import { Markdown, READING_COLUMN_CLS } from "./Markdown";
import { DocSkeleton } from "./skeletons";
import { displayName } from "./RichDetailDrawer";
import { LABEL_CLS, SectionedDoc } from "./SectionedDoc";
import { TaskList } from "./TaskList";
import { ImproveChip } from "./ImproveStamp";
import { isImproveKind } from "./improveStyle";
import { REVIEW_LABEL_KEY, REVIEW_TONE } from "./reviewStyle";
import { useCopied } from "./useCopied";
import { VERIFY_LABEL_KEY, VERIFY_TONE } from "./verifyStyle";

/** 抽屜目標（design D1：discriminated target 兩型同檔）：封存變更或封存討論。 */
export type ArchivedTarget =
  | { kind: "change"; datedName: string }
  | { kind: "discussion"; slug: string };

export interface ArchivedDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: ArchivedTarget | null;
  /** 刷新世代——遞增即就地重載（不清空、latest-wins；未傳＝0）。 */
  refreshGen?: number;
  /** 讀取封存變更的 artifact 原文（dated name 定址）；缺件回 null。 */
  loadDocument: (datedName: string, artifact: string) => Promise<string | null>;
  /** 列出封存變更的 delta capability 名。 */
  loadCapabilities: (datedName: string) => Promise<string[]>;
  /** 讀取封存討論記錄全文（slug 定址）；缺席回 null。 */
  loadDiscussionDocument: (slug: string) => Promise<string | null>;
  /** 封存變更的來源討論（slug＋topic，App 端解析；缺席/空＝不顯示 chips）。 */
  sourceDiscussions?: { slug: string; topic: string }[];
  /** 點來源討論 chip：宿主於同一抽屜切換至該討論的唯讀檢視。 */
  onOpenDiscussion?: (slug: string) => void;
  /** 封存時的審查結局（清單項帶出；spec「已封存側的審查標示」）。 */
  reviewStatus?: ArchivedItem["reviewStatus"];
  /** 封存時的驗證結局（spec「已封存側的驗證標示」）；與審查結局並存。 */
  verifyStatus?: ArchivedItem["verifyStatus"];
  /** 封存討論的 kind（App 端由封存清單帶出；標示隨 kind 恆定，不隨生命週期變化）。 */
  discussionKind?: string | null;
  /** 出身列的建立者（"Name <email>"；清單項帶出）；缺席＝該欄缺席。 */
  createdBy?: string | null;
  /** 出身列的建立日期 YYYY-MM-DD（清單項帶出）；缺席＝該欄缺席。 */
  created?: string;
  /** 出身列的封存日期 YYYY-MM-DD（清單項的 date）；缺席＝該欄缺席。 */
  archivedDate?: string;
}

type Doc = string | null | undefined;

/** 唯讀封存抽屜（spec「已封存項目以抽屜檢視」）：封存變更呈四分頁（提案／設計／
 * 任務／規格，任務核取方塊 disabled、無工具列），封存討論呈「背景」「討論過程」
 * 「結論」區段；無任何寫入動詞。寬度與全螢幕切換與變更詳情抽屜同款；開啟／換目標
 * 清空全量載入、世代重載不清空且 latest-wins（design D3）。 */
export function ArchivedDrawer({
  open,
  onOpenChange,
  target,
  refreshGen,
  loadDocument,
  loadCapabilities,
  loadDiscussionDocument,
  sourceDiscussions,
  onOpenDiscussion,
  reviewStatus,
  verifyStatus,
  discussionKind,
  createdBy,
  created,
  archivedDate,
}: ArchivedDrawerProps) {
  const { t } = useI18n();
  const [copied, markCopied] = useCopied();
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  // undefined＝capability 清單與其規格文件尚未載完（與「無規格差異」的空物件分流）。
  const [specDocs, setSpecDocs] = useState<Record<string, string | null> | undefined>();
  const [discussionDoc, setDiscussionDoc] = useState<Doc>();
  const [full, setFull] = useState(false);

  const gen = refreshGen ?? 0;
  // target 的識別鍵——換目標（含同 kind 換對象）觸發清空重載。
  const targetKey = target ? (target.kind === "change" ? `c:${target.datedName}` : `d:${target.slug}`) : null;
  // latest-wins：每次載入取遞增序號，回應到達時序號已過期即丟棄（涵蓋世代與換目標的交錯）。
  const requestSeq = useRef(0);
  // 已發起載入的世代——判斷外部世代是否落後需補載。
  const loadedGen = useRef(-1);

  const loadAll = (g: number, tgt: ArchivedTarget, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) {
      setProposal(undefined);
      setDesign(undefined);
      setTasksMd(undefined);
      setSpecDocs(undefined);
      setDiscussionDoc(undefined);
    }
    const fresh = <T,>(apply: (v: T) => void) => (v: T) => {
      if (requestSeq.current === seq) apply(v);
    };
    if (tgt.kind === "discussion") {
      void loadDiscussionDocument(tgt.slug).then(fresh(setDiscussionDoc));
      return;
    }
    const name = tgt.datedName;
    void loadDocument(name, "proposal.md").then(fresh(setProposal));
    void loadDocument(name, "design.md").then(fresh(setDesign));
    void loadDocument(name, "tasks.md").then(fresh(setTasksMd));
    void loadCapabilities(name).then(async (caps) => {
      const entries = await Promise.all(
        caps.map(async (cap) => [cap, await loadDocument(name, `specs/${cap}/spec.md`)] as const),
      );
      if (requestSeq.current === seq) setSpecDocs(Object.fromEntries(entries));
    });
  };

  // 開啟／換目標：清空後全量載入（載入中狀態屬新內容的正確呈現）。
  useEffect(() => {
    if (!open || !target) return;
    loadAll(gen, target, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, targetKey]);

  // 外部世代重載：不清空、回應到達後單次替換（不重置分頁與捲動）。
  useEffect(() => {
    if (!open || !target) return;
    if (gen <= loadedGen.current) return;
    loadAll(gen, target, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, targetKey, gen]);

  if (!target) return null;

  const title = target.kind === "change" ? target.datedName : target.slug;
  const specCount = Object.keys(specDocs ?? {}).length;
  const sections = discussionDoc ? splitDiscussionSections(discussionDoc) : null;
  // 複製鈕（spec「已封存項目以抽屜檢視」）：封存變更複製含日期前綴的封存目錄名、
  // 封存討論複製 slug——標題文字本身即複製值。
  const copyTitle = () => {
    void navigator.clipboard?.writeText(title);
    markCopied();
  };
  const copyLabel = target.kind === "change" ? t("archived.copyName") : t("discussion.copySlug");
  // 出身列：三欄各自缺席獨立——任一欄資料不可得時該欄缺席，其餘照常。
  const hasProvenance = Boolean(createdBy || created || archivedDate);

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        data-archived-drawer
        className={full ? "w-[96vw] max-w-none" : "w-[max(720px,42vw)] max-w-[95vw]"}
      >
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label={full ? t("rdrawer.restore") : t("rdrawer.fullScreen")}
          className="absolute right-11 top-3 h-7 w-7 text-muted-foreground"
          onClick={() => setFull((f) => !f)}
        >
          {full ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
        </Button>
        <SheetHeader>
          <div className="flex items-center gap-2 pr-14">
            <SheetTitle className="truncate">{title}</SheetTitle>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={copied ? t("specs.copied") : copyLabel}
              className="h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground"
              onClick={copyTitle}
            >
              {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            </Button>
          </div>
          {/* 出身列（spec「已封存項目以抽屜檢視」）：建立者（首字母圓標＋名字，
              完整識別收提示）、建立日期、封存日期；恆定單行、溢出裁切——與變更
              詳情抽屜出身列同構。無進度條與動詞動作列：封存是唯讀定格。 */}
          {hasProvenance && (
            <TooltipProvider>
              <div
                data-provenance-row
                className="flex min-w-0 items-center gap-2 whitespace-nowrap overflow-hidden text-xs text-muted-foreground"
              >
                {createdBy && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="inline-flex shrink-0 items-center gap-1">
                        <span className="inline-flex h-4 w-4 items-center justify-center rounded-full bg-muted text-muted-foreground text-[9px] font-bold">
                          {createdBy.charAt(0).toUpperCase()}
                        </span>
                        {displayName(createdBy)}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>{createdBy}</TooltipContent>
                  </Tooltip>
                )}
                {created && (
                  <span className="shrink-0">
                    {t("archived.createdOn").replace("{date}", created)}
                  </span>
                )}
                {archivedDate && (
                  <span className="shrink-0">
                    {t("archived.archivedOn").replace("{date}", archivedDate)}
                  </span>
                )}
              </div>
            </TooltipProvider>
          )}
          {/* 改進標示（spec「討論抽屜的改進標示」）：與活討論抽屜同一章籤。 */}
          {target.kind === "discussion" && isImproveKind(discussionKind) && (
            <div>
              <ImproveChip />
            </div>
          )}
          {/* 審查結局標示（spec「已封存側的審查標示」）：封存即定格，不重算凍結度。 */}
          {target.kind === "change" &&
            (reviewStatus === "reviewed" || reviewStatus === "reviewedNotPassed") && (
              <div
                data-review-outcome
                className={`text-xs font-medium ${REVIEW_TONE[reviewStatus]}`}
              >
                {t(REVIEW_LABEL_KEY[reviewStatus])}
              </div>
            )}
          {target.kind === "change" &&
            (verifyStatus === "verified" || verifyStatus === "verifiedNotPassed") && (
              <div
                data-verify-outcome
                className={`text-xs font-medium ${VERIFY_TONE[verifyStatus]}`}
              >
                {t(VERIFY_LABEL_KEY[verifyStatus])}
              </div>
            )}
          {/* 同源連結（design D1 增補）：點 chip 由宿主於同一抽屜切至該討論唯讀檢視。
              呈現與變更詳情抽屜同構——首籤直出、其餘收 +N 浮層（SourceChipRow 共用）。 */}
          {target.kind === "change" && (sourceDiscussions ?? []).length > 0 && (
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <SourceChipRow
                label={t("rdrawer.fromDiscussion")}
                items={sourceDiscussions ?? []}
                onOpen={(slug) => onOpenDiscussion?.(slug)}
              />
            </div>
          )}
        </SheetHeader>
        {target.kind === "change" ? (
          <Tabs defaultValue="proposal" className="flex-1 min-h-0 flex flex-col">
            <TabsList>
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
                {specCount > 0 ? `＋${specCount}` : ""}
              </TabsTrigger>
            </TabsList>
            <div className="flex-1 overflow-y-auto pt-3">
              {/* 共用置中容器包住分頁全部內容——區段標籤、任務清單與內文同欄（design D4）。 */}
              <div data-reading-column className={READING_COLUMN_CLS}>
              {/* 各分頁三態分流：載入中畫骨架，空態文案只在載入完成後出
                  （spec「抽屜文件載入以 skeleton 呈現」）。 */}
              <TabsContent value="proposal">
                {proposal === undefined ? (
                  <DocSkeleton />
                ) : (
                  <SectionedDoc content={proposal} empty={t("archived.noProposal")} />
                )}
              </TabsContent>
              <TabsContent value="design">
                {design === undefined ? (
                  <DocSkeleton />
                ) : (
                  <SectionedDoc content={design} empty={t("archived.noDesign")} />
                )}
              </TabsContent>
              <TabsContent value="tasks">
                {/* 封存檢視唯讀：不接 onToggle/onMove，核取方塊 disabled、無批次工具列。 */}
                {tasksMd === undefined ? <DocSkeleton /> : <TaskList markdown={tasksMd} readOnly />}
              </TabsContent>
              <TabsContent value="specs">
                {specDocs === undefined ? (
                  <DocSkeleton />
                ) : specCount === 0 ? (
                  <div className="text-muted-foreground text-sm py-6">{t("archived.noSpecs")}</div>
                ) : (
                  Object.entries(specDocs).map(([cap, doc]) => (
                    <div key={cap} className="mb-4">
                      <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">
                        {cap}
                      </div>
                      <DeltaSpecView markdown={doc} />
                    </div>
                  ))
                )}
              </TabsContent>
              </div>
            </div>
          </Tabs>
        ) : (
          <div className="flex-1 min-h-0 overflow-y-auto">
            {/* 共用置中容器——討論三區段與內文同欄（design D4）。 */}
            <div data-reading-column className={READING_COLUMN_CLS}>
              {sections ? (
                <>
                  <h3 className={`${LABEL_CLS} mb-1`}>{t("archived.sectionContext")}</h3>
                  <Markdown content={sections.context} empty={t("archived.noContext")} />
                  <h3 className={`${LABEL_CLS} mb-1 mt-4`}>{t("archived.sectionRounds")}</h3>
                  <RoundsView text={sections.rounds} empty={t("archived.noRounds")} />
                  <h3 className={`${LABEL_CLS} mb-1 mt-4`}>{t("archived.sectionConclusion")}</h3>
                  <ConclusionView text={sections.conclusion} empty={t("archived.noConclusion")} />
                </>
              ) : (
                // 區段缺失或格式非預期：整篇單一檢視退回。載入中畫骨架，
                // 空態文案只在載入完成後出（spec「抽屜文件載入以 skeleton 呈現」）。
                discussionDoc === undefined ? (
                  <DocSkeleton />
                ) : (
                  <Markdown content={discussionDoc} empty={t("common.noContent")} />
                )
              )}
            </div>
          </div>
        )}
      </SheetContent>
    </Sheet>
  );
}
