import { useEffect, useRef, useState, type ReactElement } from "react";
import {
  Archive,
  Check,
  Code2,
  Copy,
  FileText,
  ListChecks,
  Maximize2,
  Minimize2,
  PenTool,
  Sparkles,
  Trash2,
} from "lucide-react";

import type { ChangeItem, ChangeMetaInfo, Verb, VerbDrawerResult } from "../adapter";
import { specDeltaCounts, sumDeltaCounts } from "../delta";
import { useI18n } from "../i18n";
import { relativeDays } from "../time";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { SourceDiscussionChip } from "./SourceDiscussionChip";
import { READING_COLUMN_CLS } from "./Markdown";
import { SectionedDoc } from "./SectionedDoc";
import { TaskList } from "./TaskList";
import { DeltaBadges, DeltaSpecView } from "./DeltaBadges";
import { AnalyzePanel } from "./AnalyzePanel";
import { setTaskMark } from "../tasks";

export interface RichDetailDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  change: ChangeItem | null;
  /** 刷新世代——遞增即重載已載入的文件內容（未傳＝0，行為等同僅開啟時載入）。 */
  refreshGen?: number;
  loadDocument: (change: string, artifact: string) => Promise<string | null>;
  loadCapabilities: (change: string) => Promise<string[]>;
  loadMeta: (change: string) => Promise<ChangeMetaInfo | null>;
  onRunVerb?: (verb: Verb, change: string) => void;
  /** 抽屜內呈現的分析結構化結果（validate＋analyze 合併；僅當 change 相符時呈現；archive 不走此結果面）。 */
  drawerVerb?: VerbDrawerResult | null;
  /** 收合分析結果（design D2：分析鈕再點、面板關閉鈕共用此路徑）。 */
  onClearVerb?: () => void;
  onDelete?: (change: string) => void;
  /** 勾選/取消任務並回寫 tasks.md；task 為 tsk_ stable ID（帶 ID 任務）或
   * ordinal 字串（無 ID 相容路徑）。重載由宿主 refresh 後的刷新世代遞增驅動（單一資料流）。 */
  onToggleTask?: (change: string, task: string, done: boolean) => Promise<void>;
  /** 移動任務順序並回寫 tasks.md（before 為可選側別）；重載由宿主 refresh 後的刷新世代遞增驅動。 */
  onMoveTask?: (change: string, from: number, to: number, before?: boolean) => Promise<void>;
  /** 批次設定全部任務完成狀態（工具列「全部已完成」／「重置任務」），單次寫回。 */
  onSetAllTasks?: (change: string, done: boolean) => Promise<void>;
  /** 來源討論清單（change.fromDiscussions 解析出的 slug＋topic，出身討論在前）；空/缺席＝非討論而來。 */
  sourceDiscussions?: { slug: string; topic: string }[];
  /** 同源 change 名（與此 change 共享至少一份來源討論，不含自己）。 */
  siblingChanges?: string[];
  /** 點來源討論開討論抽屜。 */
  onOpenDiscussion?: (slug: string) => void;
  /** 點同源 change 互跳。 */
  onOpenSibling?: (name: string) => void;
  /** capability 缺口的停用說明（remote session）：欄位存在＝該 affordance
   * disabled 並以其文字顯示 tooltip。缺席＝全功能（本地不受影響）。 */
  unavailable?: {
    /** 分析（validate＋analyze 合併鈕）。 */
    analyze?: string;
    /** 封存變更。 */
    archive?: string;
    /** 刪除變更。 */
    delete?: string;
    /** 任務勾選、批次與拖排。 */
    tasks?: string;
  };
}

type Doc = string | null | undefined;

/** disabled 元素不可靠地接收 hover；用可互動 wrapper 承接 tooltip trigger。 */
function UnavailableAction({ reason, children }: { reason?: string; children: ReactElement }) {
  if (!reason) return children;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex" tabIndex={0}>
          {children}
        </span>
      </TooltipTrigger>
      <TooltipContent>{reason}</TooltipContent>
    </Tooltip>
  );
}

/** 富詳情抽屜：metadata、進度、動作列、icon 分頁（提案/設計/互動任務/彩色規格）。 */
export function RichDetailDrawer({
  open,
  onOpenChange,
  change,
  refreshGen,
  loadDocument,
  loadCapabilities,
  loadMeta,
  onRunVerb,
  drawerVerb,
  onClearVerb,
  onDelete,
  onToggleTask,
  onMoveTask,
  onSetAllTasks,
  sourceDiscussions,
  siblingChanges,
  onOpenDiscussion,
  onOpenSibling,
  unavailable,
}: RichDetailDrawerProps) {
  const { t } = useI18n();
  const [meta, setMeta] = useState<ChangeMetaInfo | null>(null);
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [specDocs, setSpecDocs] = useState<Record<string, string | null>>({});
  const [copied, setCopied] = useState(false);
  const [full, setFull] = useState(false);
  // 批次操作／拖放寫回進行中——鎖工具列與清單（design D4 例外）。單發勾選不設此旗標。
  const [taskBusy, setTaskBusy] = useState(false);
  // 在途單發寫回計數——僅作為世代重載的讓路條件，不鎖清單（勾選樂觀更新）。
  const [pendingWrites, setPendingWrites] = useState(0);
  // 最近一次勾選寫回失敗的單行錯誤（null＝無）。
  const [taskError, setTaskError] = useState<string | null>(null);
  // 拖曳手勢進行中（按住～放開，TaskList 回報）——與 taskBusy 同為讓路條件。
  const [dragActive, setDragActive] = useState(false);

  const name = change?.name ?? null;
  const gen = refreshGen ?? 0;
  // latest-wins：每次載入取遞增序號，回應到達時序號已過期即丟棄（涵蓋世代與換 change 的交錯）。
  const requestSeq = useRef(0);
  // 已發起載入的世代——判斷外部世代是否落後需補載。
  const loadedGen = useRef(-1);

  const loadAll = (g: number, target: string, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) {
      setMeta(null);
      setProposal(undefined);
      setDesign(undefined);
      setTasksMd(undefined);
      setSpecDocs({});
    }
    const fresh = <T,>(apply: (v: T) => void) => (v: T) => {
      if (requestSeq.current === seq) apply(v);
    };
    void loadMeta(target).then(fresh(setMeta)).catch(() => undefined);
    void loadDocument(target, "proposal.md").then(fresh(setProposal)).catch(() => undefined);
    void loadDocument(target, "design.md").then(fresh(setDesign)).catch(() => undefined);
    void loadDocument(target, "tasks.md").then(fresh(setTasksMd)).catch(() => undefined);
    void loadCapabilities(target)
      .then(async (caps) => {
        const entries = await Promise.all(
          caps.map(
            async (cap) => [cap, await loadDocument(target, `specs/${cap}/spec.md`)] as const,
          ),
        );
        if (requestSeq.current === seq) setSpecDocs(Object.fromEntries(entries));
      })
      .catch(() => undefined);
  };

  // 開啟／換 change：清空後全量載入（載入中狀態屬新內容的正確呈現）。
  useEffect(() => {
    if (!open || !name) return;
    loadAll(gen, name, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, name]);

  // 外部世代重載：不清空、回應到達後單次替換（不重置分頁與捲動）。
  // 互動進行中（拖曳手勢 dragActive、批次寫回 taskBusy、在途單發寫回 pendingWrites）
  // 讓路——皆為依賴，互動結束時本 effect 重跑補載一次。
  useEffect(() => {
    if (!open || !name || taskBusy || dragActive || pendingWrites > 0) return;
    if (gen <= loadedGen.current) return;
    loadAll(gen, name, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, name, gen, taskBusy, dragActive, pendingWrites]);

  if (!change) return null;

  // 分析結果是否對本 change 開啟——分析鈕切換態與面板呈現共用同一判定（design D2）。
  const verbOpen = !!(drawerVerb && drawerVerb.change === change.name);
  const pct = change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const taskBadge = `${change.completedTasks}/${change.totalTasks}`;
  const delta = sumDeltaCounts(Object.values(specDocs).map(specDeltaCounts));
  const rel = relativeDays(meta?.created, t);

  const copyName = () => {
    void navigator.clipboard?.writeText(change.name);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  // 勾選走樂觀更新（design D3）：本地先翻轉 tasksMd 立即反映，再發寫回；失敗
  // 還原快照並顯示單行錯誤。不鎖清單——僅以 pendingWrites 讓世代重載讓路，
  // 寫回成功後的重載仍統一由宿主 refresh 的世代遞增驅動（單一資料流）。
  const handleToggle = async (ordinal: number, done: boolean, stableId?: string) => {
    if (!name || !onToggleTask) return;
    setTaskError(null);
    // 作廢在途載入（design D4）：更早發起、較晚到達的舊回應不得覆蓋樂觀狀態；
    // 寫回後的磁碟現況仍由宿主 refresh 的世代遞增補載收斂。
    requestSeq.current += 1;
    let snapshot: Doc;
    setTasksMd((cur) => {
      snapshot = cur;
      const next = cur ? setTaskMark(cur, ordinal, done) : null;
      return next ?? cur;
    });
    setPendingWrites((n) => n + 1);
    try {
      // 帶 ID 任務以 stable ID 定址；無 ID 舊檔走 ordinal 相容路徑。
      await onToggleTask(name, stableId ?? String(ordinal), done);
    } catch (e) {
      setTasksMd(snapshot ?? null);
      setTaskError(e instanceof Error ? e.message : String(e));
    } finally {
      setPendingWrites((n) => n - 1);
    }
  };

  const handleReorder = async (from: number, to: number, before?: boolean) => {
    if (!name || !onMoveTask) return;
    setTaskBusy(true);
    try {
      await onMoveTask(name, from, to, before);
    } finally {
      setTaskBusy(false);
    }
  };

  // 批次操作（design D4 例外）：執行期間工具列與清單短暫 disabled，完成後由世代重載收斂。
  const handleSetAll = async (done: boolean) => {
    if (!name || !onSetAllTasks) return;
    setTaskBusy(true);
    try {
      await onSetAllTasks(name, done);
    } finally {
      setTaskBusy(false);
    }
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        className={full ? "w-[96vw] max-w-none" : "w-[max(720px,42vw)] max-w-[95vw]"}
      >
        {/* 放大鈕與 Sheet 關閉鈕同高同尺寸並排（shadcn ghost icon，帶 hover 回饋），
            不再一高一低（design D5 附帶視覺修正）。 */}
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
            <SheetTitle className="truncate">{change.name}</SheetTitle>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={t("common.copyName")}
              className="h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground"
              onClick={copyName}
            >
              {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            </Button>
          </div>
          {/* metadata 列 */}
          <div className="flex items-center gap-2 text-xs text-muted-foreground flex-wrap">
            {meta?.createdBy && (
              <span className="inline-flex items-center gap-1">
                <span className="inline-flex h-4 w-4 items-center justify-center rounded-full bg-primary text-primary-foreground text-[9px] font-bold">
                  {meta.createdBy.charAt(0).toUpperCase()}
                </span>
                {meta.createdBy}
              </span>
            )}
            {meta?.createdWith && <span>✳ {meta.createdWith}</span>}
            {rel && <span>{rel}</span>}
            <span>{t("common.tasksCount").replace("{n}", taskBadge)}</span>
            {meta?.startedAt && (
              <span className="inline-flex items-center gap-1">
                ⚒ {meta.startedBy ? `${meta.startedBy} · ` : ""}
                {t("rdrawer.started").replace("{date}", meta.startedAt)}
              </span>
            )}
          </div>
          {/* 同源連結：來源討論（可多份）＋同源刀（fromDiscussions 帶出）。 */}
          {(sourceDiscussions ?? []).length > 0 && (
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground flex-wrap">
              <span>{t("rdrawer.fromDiscussion")}</span>
              {(sourceDiscussions ?? []).map((src) => (
                <SourceDiscussionChip
                  key={src.slug}
                  topic={src.topic}
                  onClick={() => onOpenDiscussion?.(src.slug)}
                />
              ))}
              {(siblingChanges ?? []).length > 0 && (
                <>
                  <span>{t("rdrawer.siblings")}</span>
                  {(siblingChanges ?? []).map((sib) => (
                    <Button
                      key={sib}
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-auto rounded-full bg-muted px-2 py-0.5 font-medium hover:bg-accent"
                      onClick={() => onOpenSibling?.(sib)}
                    >
                      {sib}
                    </Button>
                  ))}
                </>
              )}
            </div>
          )}
          <div className="flex items-center gap-2">
            <div className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden">
              <div className="h-full rounded-full bg-primary" style={{ width: `${pct}%` }} />
            </div>
            <span className="text-xs text-muted-foreground tabular-nums">{pct}%</span>
          </div>
          {/* 動作列 */}
          <TooltipProvider delayDuration={0}>
            <div className="flex items-center gap-1.5 pt-1">
              <UnavailableAction reason={unavailable?.analyze}>
                <Button
                  variant="outline"
                  size="sm"
                  aria-pressed={verbOpen}
                  disabled={unavailable?.analyze !== undefined}
                  title={unavailable?.analyze}
                  className={`h-7 gap-1 ${verbOpen ? "bg-accent" : ""}`}
                  onClick={() => (verbOpen ? onClearVerb?.() : onRunVerb?.("analyze", change.name))}
                >
                  <Sparkles className="h-3.5 w-3.5" /> {t("common.analyze")}
                </Button>
              </UnavailableAction>
              <div className="flex-1" />
              <UnavailableAction reason={unavailable?.archive}>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={unavailable?.archive !== undefined}
                  title={unavailable?.archive}
                  className="h-7 gap-1"
                  onClick={() => onRunVerb?.("archive", change.name)}
                >
                  <Archive className="h-3.5 w-3.5" /> {t("common.archive")}
                </Button>
              </UnavailableAction>
              <UnavailableAction reason={unavailable?.delete}>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={unavailable?.delete !== undefined}
                  title={unavailable?.delete}
                  className="h-7 gap-1 text-destructive hover:text-destructive"
                  onClick={() => onDelete?.(change.name)}
                >
                  <Trash2 className="h-3.5 w-3.5" /> {t("rdrawer.delete")}
                </Button>
              </UnavailableAction>
            </div>
          </TooltipProvider>
          {/* 分析結果（validate＋analyze 合併）於動作列近處呈現——僅當前 change 相符時（design D1）。 */}
          {verbOpen && drawerVerb && (
            <div data-verb-result className="pt-1">
              {drawerVerb.error ? (
                <div className="text-xs text-destructive">{drawerVerb.error}</div>
              ) : (
                <AnalyzePanel
                  report={drawerVerb.analyze}
                  validate={drawerVerb.validate}
                  onClose={onClearVerb}
                />
              )}
            </div>
          )}
        </SheetHeader>

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
              <Badge variant={pct === 100 ? "default" : "secondary"} className="ml-1">{taskBadge}</Badge>
            </TabsTrigger>
            <TabsTrigger value="specs">
              <Code2 className="h-3.5 w-3.5" /> {t("common.tabSpecs")}
              <span className="ml-1"><DeltaBadges counts={delta} /></span>
            </TabsTrigger>
          </TabsList>
          <div className="flex-1 overflow-y-auto pt-3">
            {/* 共用置中容器包住分頁全部內容——區段標籤、任務清單與內文同欄（design D4）。 */}
            <div data-reading-column className={READING_COLUMN_CLS}>
            {/* undefined＝載入在途、null＝文件不存在（空殼 change）——兩態文案分流，
                不存在不得掛「載入中」（remote 空 change 曾長駐載入中）。 */}
            <TabsContent value="proposal">
              <SectionedDoc
                content={proposal ?? null}
                empty={proposal === undefined ? t("common.loading") : t("list.noProposalDoc")}
              />
            </TabsContent>
            <TabsContent value="design"><SectionedDoc content={design ?? null} empty={t("list.noDesignDoc")} /></TabsContent>
            <TabsContent value="tasks">
              {taskError && (
                <div className="mb-2 text-sm text-destructive">
                  {t("tasks.writeFailed").replace("{msg}", taskError)}
                </div>
              )}
              <TaskList
                markdown={tasksMd ?? null}
                busy={taskBusy}
                readOnly={unavailable?.tasks !== undefined}
                onDragActiveChange={setDragActive}
                onToggle={(ordinal, done, stableId) => void handleToggle(ordinal, done, stableId)}
                // 拖排寫回未提供（remote capability 缺口）即整段停用——把手不渲染。
                onReorder={
                  onMoveTask ? (from, to, before) => void handleReorder(from, to, before) : undefined
                }
                onSetAll={(done) => void handleSetAll(done)}
              />
            </TabsContent>
            <TabsContent value="specs">
              {Object.keys(specDocs).length === 0 ? (
                <div className="text-muted-foreground text-sm py-6">{t("list.noDeltaSpecs")}</div>
              ) : (
                Object.entries(specDocs).map(([cap, doc]) => (
                  <div key={cap} className="mb-4">
                    <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-2">
                      {cap} <DeltaBadges counts={specDeltaCounts(doc)} />
                    </div>
                    <DeltaSpecView markdown={doc} empty={t("common.loading")} />
                  </div>
                ))
              )}
            </TabsContent>
            </div>
          </div>
        </Tabs>
      </SheetContent>
    </Sheet>
  );
}
