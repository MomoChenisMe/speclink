import { useEffect, useRef, useState, type ReactElement } from "react";
import {
  Archive,
  Check,
  Code2,
  Copy,
  FileText,
  GitBranch,
  Hand,
  ListChecks,
  Maximize2,
  Minimize2,
  PenTool,
  Sparkles,
  Trash2,
  Undo2,
} from "lucide-react";

import type { ChangeItem, ChangeMetaInfo, Verb, VerbDrawerResult } from "../adapter";
import { specDeltaCounts, sumDeltaCounts } from "../delta";
import { changeStage } from "../stage";
import { useI18n } from "../i18n";
import { useLingering } from "../lib/useLingering";
import { relativeDays } from "../time";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { SourceChipRow } from "./SourceDiscussionChip";
import { READING_COLUMN_CLS } from "./Markdown";
import { SectionedDoc } from "./SectionedDoc";
import { DocSkeleton } from "./skeletons";
import { TaskList } from "./TaskList";
import { DeltaBadges, DeltaSpecView } from "./DeltaBadges";
import { AnalyzePanel } from "./AnalyzePanel";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_TONE, type ReviewBadgeStatus } from "./reviewStyle";
import { VERIFY_ICON, VERIFY_LABEL_KEY, VERIFY_TONE, type VerifyBadgeStatus } from "./verifyStyle";
import { setTaskMark } from "../tasks";
import { useCopied } from "./useCopied";

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
  /** 認領請求（RemoteOnly——本地後端不提供，未提供時不渲染認領面）。 */
  onClaim?: (change: string) => void;
  /** 退回提案中請求（僅派生進行中呈現;app 端接確認流程;未提供時不渲染）。 */
  onRevert?: (change: string) => void;
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
    /** 認領變更。 */
    claim?: string;
  };
}

type Doc = string | null | undefined;

/** 出身列僅顯示名字：去除 email 尖括號段（無尖括號時整串直出），完整識別由提示承載。
 * 封存抽屜的出身列共用此處（同構呈現的單一實作落點）。 */
export function displayName(identity: string): string {
  const name = identity.replace(/\s*<[^>]*>\s*/g, " ").trim();
  return name || identity;
}

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

/** 狀態列的站章籤：可視只有圖示＋狀態詞，蓋章日期與蓋章者完整識別（含 email）
    收進提示。兩站同構，只換站別——資料缺席（進行中、無戳記）時整個提示缺席。 */
function StationStamp({
  station,
  tone,
  icon,
  label,
  at,
  by,
}: {
  station: "review" | "verify";
  tone: string;
  icon: ReactElement;
  label: string;
  at?: string | null;
  by?: string | null;
}) {
  // 既有 data 標記維持站別各一份（回歸對照沿用 data-review-row／data-verify-row）。
  const rowMark = station === "review" ? { "data-review-row": "" } : { "data-verify-row": "" };
  const toneMark = station === "review" ? { "data-review-tone": "" } : { "data-verify-tone": "" };
  const chip = (
    <span
      {...rowMark}
      className="inline-flex shrink-0 items-center gap-1 text-xs text-muted-foreground"
    >
      <span {...toneMark} className={`inline-flex items-center gap-1 font-medium ${tone}`}>
        {icon}
        {label}
      </span>
    </span>
  );
  const detail = [at, by].filter(Boolean).join(" · ");
  if (!detail) return chip;
  return (
    <Tooltip>
      <TooltipTrigger asChild>{chip}</TooltipTrigger>
      <TooltipContent>{detail}</TooltipContent>
    </Tooltip>
  );
}

/** 富詳情抽屜：metadata、進度、動作列、icon 分頁（提案/設計/互動任務/彩色規格）。 */
export function RichDetailDrawer({
  open,
  onOpenChange,
  change: changeProp,
  refreshGen,
  loadDocument,
  loadCapabilities,
  loadMeta,
  onRunVerb,
  drawerVerb,
  onClearVerb,
  onDelete,
  onClaim,
  onRevert,
  onToggleTask,
  onMoveTask,
  onSetAllTasks,
  sourceDiscussions,
  siblingChanges,
  onOpenDiscussion,
  onOpenSibling,
  unavailable,
}: RichDetailDrawerProps) {
  const change = useLingering(changeProp);
  const { t } = useI18n();
  const [meta, setMeta] = useState<ChangeMetaInfo | null>(null);
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  // undefined＝capability 清單與其規格文件尚未載完（與「無 delta 規格」的空物件分流）。
  const [specDocs, setSpecDocs] = useState<Record<string, string | null> | undefined>();
  const [copied, markCopied] = useCopied();
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
      setSpecDocs(undefined);
    }
    const fresh = <T,>(apply: (v: T) => void) => (v: T) => {
      if (requestSeq.current === seq) apply(v);
    };
    // 失敗收斂：還沒有東西可顯示才落終態空文案（undefined 停著＝永久骨架）；
    // 已有內容維持前值——重載的短暫失敗不得抹成假空態。
    const settled = (apply: typeof setProposal) => () => {
      if (requestSeq.current === seq) apply((prev) => (prev === undefined ? null : prev));
    };
    void loadMeta(target).then(fresh(setMeta)).catch(() => undefined);
    void loadDocument(target, "proposal.md").then(fresh(setProposal)).catch(settled(setProposal));
    void loadDocument(target, "design.md").then(fresh(setDesign)).catch(settled(setDesign));
    void loadDocument(target, "tasks.md").then(fresh(setTasksMd)).catch(settled(setTasksMd));
    void loadCapabilities(target)
      .then(async (caps) => {
        const entries = await Promise.all(
          caps.map(
            async (cap) => [cap, await loadDocument(target, `specs/${cap}/spec.md`)] as const,
          ),
        );
        if (requestSeq.current === seq) setSpecDocs(Object.fromEntries(entries));
      })
      // 首載失敗收斂為空集（undefined 停著＝永久骨架）；重載失敗維持前值，不抹成假空態。
      .catch(() => {
        if (requestSeq.current === seq) {
          setSpecDocs((prev) => (prev === undefined ? {} : prev));
        }
      });
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
  const delta = sumDeltaCounts(Object.values(specDocs ?? {}).map(specDeltaCounts));
  const rel = relativeDays(meta?.created, t);

  const copyName = () => {
    void navigator.clipboard?.writeText(change.name);
    markCopied();
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

  // 階段守門原因（archive-readiness-gating D4）：封存鈕非已就緒、刪除鈕非提案中
  // 時 disabled；unavailable（remote 能力缺失）原因優先於階段原因——「這條通道
  // 做不到」比「現在還不能」更硬。守門為呈現層過濾，引擎拒絕是最終裁決。
  // 審查資訊列的狀態呈現：圖示＋狀態詞依四態上色，配色與卡片章共用 REVIEW_TONE。
  const reviewStatus: ReviewBadgeStatus | null =
    !change.reviewStatus || change.reviewStatus === "none" ? null : change.reviewStatus;
  const review = !reviewStatus
    ? null
    : {
        key: REVIEW_LABEL_KEY[reviewStatus],
        icon: REVIEW_ICON[reviewStatus],
        cls: REVIEW_TONE[reviewStatus],
      };
  // 驗證資訊列：與審查資訊列同構（同一組欄位、同一套版面），只換站別對照表。
  const verifyStatus: VerifyBadgeStatus | null =
    !change.verifyStatus || change.verifyStatus === "none" ? null : change.verifyStatus;
  const verify = !verifyStatus
    ? null
    : {
        key: VERIFY_LABEL_KEY[verifyStatus],
        icon: VERIFY_ICON[verifyStatus],
        cls: VERIFY_TONE[verifyStatus],
      };
  const stage = changeStage(change);
  const archiveReason =
    unavailable?.archive ??
    (stage !== "ready"
      ? t("rdrawer.archiveNotReady")
          .replace("{done}", String(change.completedTasks))
          .replace("{total}", String(change.totalTasks))
      : undefined);
  const deleteReason =
    unavailable?.delete ?? (stage !== "proposed" ? t("rdrawer.deleteStarted") : undefined);

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
          {/* 狀態列（spec「變更詳情抽屜標頭的四層結構」）：進度條＋百分比；兩站狀態
              非 none 時同列呈章籤（圖示＋狀態詞），蓋章日期與蓋章者完整識別收進提示
              ——與出身列的 email 收納同構。恆定單行、不撐寬抽屜。任務計數不上標頭
              （任務分頁徽章與進度條已承載）。 */}
          <TooltipProvider>
            <div
              data-status-row
              className="flex min-w-0 items-center gap-2 whitespace-nowrap overflow-hidden"
            >
              <div
                data-progress-track
                className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden"
              >
                <div className="h-full rounded-full bg-primary" style={{ width: `${pct}%` }} />
              </div>
              <span className="text-xs text-muted-foreground tabular-nums">{pct}%</span>
              {review && (
                <StationStamp
                  station="review"
                  tone={review.cls}
                  icon={review.icon}
                  label={t(review.key)}
                  at={change.reviewStatus === "inReview" ? undefined : change.reviewedAt}
                  by={change.reviewStatus === "inReview" ? undefined : change.reviewedBy}
                />
              )}
              {verify && (
                <StationStamp
                  station="verify"
                  tone={verify.cls}
                  icon={verify.icon}
                  label={t(verify.key)}
                  at={change.verifyStatus === "inVerify" ? undefined : change.verifiedAt}
                  by={change.verifyStatus === "inVerify" ? undefined : change.verifiedBy}
                />
              )}
            </div>
          </TooltipProvider>
          {/* 出身列（spec「變更詳情抽屜標頭的四層結構」）：恆定單行——不折行、
              溢出裁切兜底；email 與開工者完整識別收進提示，可視文字僅名字／日期。 */}
          <TooltipProvider>
            <div
              data-provenance-row
              className="flex min-w-0 items-center gap-2 whitespace-nowrap overflow-hidden text-xs text-muted-foreground"
            >
              {meta?.createdBy && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span className="inline-flex shrink-0 items-center gap-1">
                      <span className="inline-flex h-4 w-4 items-center justify-center rounded-full bg-muted text-muted-foreground text-[9px] font-bold">
                        {meta.createdBy.charAt(0).toUpperCase()}
                      </span>
                      {displayName(meta.createdBy)}
                    </span>
                  </TooltipTrigger>
                  <TooltipContent>{meta.createdBy}</TooltipContent>
                </Tooltip>
              )}
              {meta?.createdWith && <span className="shrink-0">✳ {meta.createdWith}</span>}
              {rel && <span className="shrink-0">{rel}</span>}
              {meta?.startedAt &&
                (meta.startedBy ? (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="inline-flex shrink-0 items-center gap-1">
                        ⚒ {t("rdrawer.started").replace("{date}", meta.startedAt)}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>{meta.startedBy}</TooltipContent>
                  </Tooltip>
                ) : (
                  <span className="inline-flex shrink-0 items-center gap-1">
                    ⚒ {t("rdrawer.started").replace("{date}", meta.startedAt)}
                  </span>
                ))}
              {change.claimedBy && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span className="inline-flex shrink-0 items-center gap-1">
                      <Hand className="h-3 w-3" />
                      {t("rdrawer.claimed").replace("{name}", displayName(change.claimedBy))}
                    </span>
                  </TooltipTrigger>
                  <TooltipContent>{change.claimedBy}</TooltipContent>
                </Tooltip>
              )}
              {change.worktree && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span
                      className="inline-flex shrink-0 items-center gap-1"
                      title={change.worktree.path}
                    >
                      <GitBranch className="h-3 w-3" />
                      {change.worktree.branch}
                    </span>
                  </TooltipTrigger>
                  <TooltipContent>{change.worktree.path}</TooltipContent>
                </Tooltip>
              )}
              {(sourceDiscussions ?? []).length > 0 && (
                <SourceChipRow
                  label={t("rdrawer.fromDiscussion")}
                  items={sourceDiscussions ?? []}
                  onOpen={(slug) => onOpenDiscussion?.(slug)}
                />
              )}
              {(siblingChanges ?? []).length > 0 && (
                <SourceChipRow
                  label={t("rdrawer.siblings")}
                  items={(siblingChanges ?? []).map((name) => ({ slug: name }))}
                  onOpen={(name) => onOpenSibling?.(name)}
                />
              )}
            </div>
          </TooltipProvider>
          {/* 動作列 */}
          <TooltipProvider>
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
              {/* 退回提案中（僅派生進行中;樣式沿動作列 outline 鈕）：點擊交宿主
                  先確認,UI 不預判守門。 */}
              {/* 認領（remote-claim-ownership D4）：僅未認領的 change 提供入口
                  ——已有持有人時出身列已寫明是誰，再擺一顆按鈕只會誘人去撞
                  引擎的衝突拒絕。onClaim 缺席＝本地分頁，整段不渲染。 */}
              {onClaim && !change.claimedBy && (
                <UnavailableAction reason={unavailable?.claim}>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={unavailable?.claim !== undefined}
                    title={unavailable?.claim}
                    className="h-7 gap-1"
                    onClick={() => onClaim(change.name)}
                  >
                    <Hand className="h-3.5 w-3.5" /> {t("rdrawer.claim")}
                  </Button>
                </UnavailableAction>
              )}
              {stage === "in-progress" && onRevert && (
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 gap-1"
                  onClick={() => onRevert(change.name)}
                >
                  <Undo2 className="h-3.5 w-3.5" /> {t("common.revert")}
                </Button>
              )}
              <UnavailableAction reason={archiveReason}>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={archiveReason !== undefined}
                  title={archiveReason}
                  className="h-7 gap-1"
                  onClick={() => onRunVerb?.("archive", change.name)}
                >
                  <Archive className="h-3.5 w-3.5" /> {t("common.archive")}
                </Button>
              </UnavailableAction>
              <UnavailableAction reason={deleteReason}>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={deleteReason !== undefined}
                  title={deleteReason}
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
            {/* undefined＝載入在途、null＝文件不存在（空殼 change）——載入中畫骨架，
                空態文案只在載入完成後出：不存在不得掛「載入中」（remote 空 change 曾長駐
                載入中），載入中也不得謊稱「無文件」。 */}
            <TabsContent value="proposal">
              {proposal === undefined ? (
                <DocSkeleton />
              ) : (
                <SectionedDoc content={proposal} empty={t("list.noProposalDoc")} />
              )}
            </TabsContent>
            <TabsContent value="design">
              {design === undefined ? (
                <DocSkeleton />
              ) : (
                <SectionedDoc content={design} empty={t("list.noDesignDoc")} />
              )}
            </TabsContent>
            <TabsContent value="tasks">
              {taskError && (
                <div className="mb-2 text-sm text-destructive">
                  {t("tasks.writeFailed").replace("{msg}", taskError)}
                </div>
              )}
              {tasksMd === undefined ? (
                <DocSkeleton />
              ) : (
                <TaskList
                  markdown={tasksMd}
                  busy={taskBusy}
                  readOnly={unavailable?.tasks !== undefined}
                  onDragActiveChange={setDragActive}
                  onToggle={(ordinal, done, stableId) => void handleToggle(ordinal, done, stableId)}
                  // 拖排寫回未提供（remote capability 缺口）即整段停用——把手不渲染。
                  onReorder={
                    onMoveTask
                      ? (from, to, before) => void handleReorder(from, to, before)
                      : undefined
                  }
                  onSetAll={(done) => void handleSetAll(done)}
                />
              )}
            </TabsContent>
            <TabsContent value="specs">
              {specDocs === undefined ? (
                <DocSkeleton />
              ) : Object.keys(specDocs).length === 0 ? (
                <div className="text-muted-foreground text-sm py-6">{t("list.noDeltaSpecs")}</div>
              ) : (
                Object.entries(specDocs).map(([cap, doc]) => (
                  <div key={cap} className="mb-4">
                    <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-2">
                      {cap} <DeltaBadges counts={specDeltaCounts(doc)} />
                    </div>
                    {/* 逐項空態（capability 有列出、其 spec.md 為空）不套整體性
                        文案——與已封存抽屜同一位置的處置一致。 */}
                    <DeltaSpecView markdown={doc} />
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
