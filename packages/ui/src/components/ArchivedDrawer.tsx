import { useEffect, useRef, useState } from "react";
import { Code2, FileText, ListChecks, Maximize2, Minimize2, PenTool } from "lucide-react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { SourceChipRow } from "./SourceDiscussionChip";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { DeltaSpecView } from "./DeltaBadges";
import { ConclusionView, RoundsView, splitDiscussionSections } from "./DiscussionDrawer";
import { Markdown, READING_COLUMN_CLS } from "./Markdown";
import { LABEL_CLS, SectionedDoc } from "./SectionedDoc";
import { TaskList } from "./TaskList";
import { REVIEW_LABEL_KEY, REVIEW_TONE } from "./reviewStyle";

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
}: ArchivedDrawerProps) {
  const { t } = useI18n();
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [specDocs, setSpecDocs] = useState<Record<string, string | null>>({});
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
      setSpecDocs({});
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
  const specCount = Object.keys(specDocs).length;
  const sections = discussionDoc ? splitDiscussionSections(discussionDoc) : null;

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
          <SheetTitle className="truncate pr-14">{title}</SheetTitle>
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
              <TabsContent value="proposal">
                <SectionedDoc content={proposal ?? null} empty={t("archived.noProposal")} />
              </TabsContent>
              <TabsContent value="design">
                <SectionedDoc content={design ?? null} empty={t("archived.noDesign")} />
              </TabsContent>
              <TabsContent value="tasks">
                {/* 封存檢視唯讀：不接 onToggle/onMove，核取方塊 disabled、無批次工具列。 */}
                <TaskList markdown={tasksMd ?? null} readOnly />
              </TabsContent>
              <TabsContent value="specs">
                {specCount === 0 ? (
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
                // 區段缺失或格式非預期：整篇單一檢視退回；缺件顯示空狀態。
                <Markdown
                  content={discussionDoc ?? null}
                  empty={discussionDoc === undefined ? t("common.loading") : t("common.noContent")}
                />
              )}
            </div>
          </div>
        )}
      </SheetContent>
    </Sheet>
  );
}
