import {
  useEffect,
  useRef,
  useState,
  type Dispatch,
  type ReactElement,
  type ReactNode,
  type SetStateAction,
} from "react";
import {
  Archive,
  Code2,
  FileText,
  GitBranch,
  Hand,
  ListChecks,
  ListOrdered,
  MoreHorizontal,
  PenTool,
  Sparkles,
  Trash2,
  Undo2,
} from "lucide-react";

import type {
  ChangeItem,
  ChangeMetaInfo,
  StationTicket,
  TicketStation,
  Verb,
  VerbDrawerResult,
} from "../adapter";
import { specDeltaCounts, sumDeltaCounts } from "../delta";
import {
  changeStage,
  planBlockedBy,
  planBlockedLabel,
  planWave,
  planWaveLabel,
} from "../stage";
import { newCapabilities } from "../docModel";
import { useI18n } from "../i18n";
import { relativeDays } from "../time";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Card } from "./ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { StatusPill, type StatusPillTone } from "./ui/status-pill";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { SourceChipRow } from "./SourceDiscussionChip";
import { changeDocSummary } from "./changeDocSummary";
import { DesignDoc } from "./DesignDoc";
import type { PathReveal } from "./DocParts";
import { ProposalDoc } from "./ProposalDoc";
import { DocSkeleton } from "./skeletons";
import { PlanDoc } from "./PlanDoc";
import { TaskList } from "./TaskList";
import { DeltaBadges, DeltaCapSection } from "./DeltaBadges";
import { AnalyzePanel } from "./AnalyzePanel";
import { TicketTabBody, useStationTickets } from "./TicketView";
import { useDocNav } from "./useDocNav";
import {
  IdentityName,
  STAGE_PILL_TONE,
  StatusRow,
  displayName,
  documentArtifact,
  type DocumentTarget,
} from "./DetailHeader";
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
  changeDocTocKey,
  type DetailRelation,
} from "./DetailShell";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_PILL_TONE, type ReviewBadgeStatus } from "./reviewStyle";
import { VERIFY_ICON, VERIFY_LABEL_KEY, VERIFY_PILL_TONE, type VerifyBadgeStatus } from "./verifyStyle";
import { setTaskMark } from "../tasks";
import { SecurityReviewPanel } from "./SecurityReviewPanel";
import type { SecurityInspection, SecurityReviewAdapter } from "../security";

import {useCheckRecords,CheckRecordBody,QualityCheckRecordList,checkTab} from "./CheckRecords";
export interface ChangeDetailPageProps {
  change: ChangeItem | null;
  /** 刷新世代——遞增即重載已載入的文件內容（未傳＝0，行為等同僅開啟時載入）。 */
  refreshGen?: number;
  loadDocument: (change: string, artifact: string) => Promise<string | null>;
  loadCapabilities: (change: string) => Promise<string[]>;
  loadMeta: (change: string) => Promise<ChangeMetaInfo | null>;
  /** 一站的活工單（spec desktop-app「詳情頁的工單分頁」）：分頁出現時載入、隨
   * refreshGen 重載；回 null 或 reject 皆為分頁空態。未提供時分頁仍依清單狀態出現、
   * 內容為空態。 */
  loadStationTicket?: (change: string, station: TicketStation) => Promise<StationTicket | null>;
  securityReview?: SecurityReviewAdapter;
  checkRecords?:import("../checkRecords").CheckRecordsAdapter;
  onOpenSecurityData?: () => void;
  onRunVerb?: (verb: Verb, change: string) => void;
  /** 頁內呈現的分析結構化結果（validate＋analyze 合併；僅當 change 相符時呈現；archive 不走此結果面）。 */
  detailVerb?: VerbDrawerResult | null;
  /** 收合分析結果（分析鈕再點、面板關閉鈕共用此路徑）。 */
  onClearVerb?: () => void;
  onDelete?: (change: string) => void;
  /** 認領請求（RemoteOnly——本地後端不提供，未提供時不渲染認領面）。 */
  onClaim?: (change: string) => void;
  /** 退回提案中請求（僅派生進行中呈現；app 端接確認流程；未提供時不渲染）。 */
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
  /** 點來源討論開討論詳情頁。 */
  onOpenDiscussion?: (slug: string) => void;
  /** 點同源 change 互跳。 */
  onOpenSibling?: (name: string) => void;
  /** 作用中變更清單（排程分頁：同波夥伴與前置候選自此派生）；缺席＝空清單。 */
  changes?: ChangeItem[];
  /** 排程分頁的前置新增／移除（add-change-plan-desktop design D6）。未提供
   * （capability `setDepends` 為假）時分頁唯讀：不渲染移除鈕與新增下拉。 */
  onSetDepends?: (change: string, on: string[], remove: boolean) => void;
  /** 排程分頁新增前置的候選名冊（add-change-plan-remote D9）：提供時以其結果為準
   * （載入完成前不長新增下拉），未提供或載入失敗時候選自 `changes` 派生。 */
  loadDependsCandidates?: (change: string) => Promise<string[]>;
  /** capability 缺口的停用說明（remote session）：欄位存在＝該 affordance
   * disabled 並以其文字說明。缺席＝全功能（本地不受影響）。 */
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
  /** 內容卡標頭右端的文件動作（design D4「開啟 ▾」）：頁依作用中分頁回報對應的文件與
   * 已載入原文；沒有對應檔案的分頁（排程）為 null。 */
  renderDocumentActions?: (target: DocumentTarget | null) => ReactNode;
  /** 開該能力的規格詳情頁（規格文件能力頭的「看正式規格」）；未提供時不渲染該鈕。 */
  onOpenSpec?: (capability: string) => void;
  /** 提案影響段路徑籤的本機路徑動作（文案依平台）；未提供（無本機目錄）時只有複製鈕。 */
  pathReveal?: PathReveal;
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

/** 一站的章籤資料：狀態非 none 才有。 */
interface Stamp {
  station: "review" | "verify";
  tone: StatusPillTone;
  icon: ReactElement;
  label: string;
  /** 蓋章日期與蓋章者——進行中（inReview／inVerify）不帶。 */
  at?: string | null;
  by?: string | null;
}

/** 「品質關卡」列的站章籤：圖示＋狀態詞，已蓋章時旁接蓋章日期；蓋章日期與蓋章者完整
    識別（含 email）收進提示，整枚章籤（含日期）為觸發點，可用鍵盤聚焦叫出。兩站同構，
    只換站別——資料缺席（進行中、無戳記）時整個提示缺席，章籤也不進 Tab 序。 */
function QualityStamp({ stamp, expanded, controlsId, onToggle }: { stamp: Stamp; expanded:boolean; controlsId:string; onToggle:()=>void }) {
  // 既有 data 標記維持站別各一份（回歸對照沿用 data-review-row／data-verify-row）。
  const rowMark = stamp.station === "review" ? { "data-review-row": "" } : { "data-verify-row": "" };
  const toneMark = stamp.station === "review" ? { "data-review-tone": "" } : { "data-verify-tone": "" };
  const detail = [stamp.at, stamp.by].filter(Boolean).join(" · ");
  const chip = (
    <button {...rowMark} type="button" aria-expanded={expanded} aria-controls={controlsId} aria-label={stamp.label} onClick={onToggle} className="inline-flex shrink-0 items-center gap-1.5 rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
      <StatusPill {...toneMark} tone={stamp.tone} icon={stamp.icon}>
        {stamp.label}
      </StatusPill>
      {stamp.at && <span className="font-mono text-[11px] text-muted-foreground">{stamp.at}</span>}
    </button>
  );
  if (!detail) return chip;
  return (
    <Tooltip>
      <TooltipTrigger asChild>{chip}</TooltipTrigger>
      <TooltipContent>{detail}</TooltipContent>
    </Tooltip>
  );
}

/** 「⋯」選單項：圖示＋標籤，停用時項目下方以灰字列出原因（選單項不接 hover 提示）。 */
function MenuAction({ icon, label, reason }: { icon: ReactElement; label: string; reason?: string }) {
  return (
    <span className="flex min-w-0 flex-col gap-0.5">
      <span className="flex items-center gap-2.5">
        {icon}
        <span data-menu-label>{label}</span>
      </span>
      {reason && (
        <span data-menu-reason className="pl-[26px] text-xs whitespace-normal text-muted-foreground">
          {reason}
        </span>
      )}
    </span>
  );
}

/** 分頁 → 該分頁對應的變更目錄內檔案（排程沒有檔案；規格另依 capability 組路徑）。 */
const TAB_FILES: Record<string, string> = {
  proposal: "proposal.md",
  design: "design.md",
  tasks: "tasks.md",
};

/** 「開啟 ▾」的對象（design D4）：目前文件對應的 artifact 與其已載入原文。規格文件取
 * 選定的 capability（未選定或已不在清單時取第一份）；排程沒有檔案；工單只載解析後的回合、不帶原文。 */
function documentTarget(
  name: string,
  tab: string,
  loaded: Record<string, Doc>,
  specDocs: Record<string, string | null> | undefined,
  specCap: string | null,
): DocumentTarget | null {
  if (tab === "specs") {
    const caps = Object.keys(specDocs ?? {});
    const cap = specCap && caps.includes(specCap) ? specCap : caps[0];
    return cap ? { doc: { change: name, artifact: `specs/${cap}/spec.md` }, content: specDocs?.[cap] ?? null } : null;
  }
  if (tab === "review" || tab === "verify") return { doc: { change: name, artifact: `${tab}.md` } };
  const artifact = TAB_FILES[tab];
  return artifact ? { doc: { change: name, artifact }, content: loaded[tab] ?? null } : null;
}

/**
 * 變更詳情頁（design D5；spec「詳情頁的文件樹、閱讀卡與本頁目錄」）：三欄骨架——左欄身分區
 * （名稱、籤、出身、來自／同源／規格列）、「文件」節點（提案／設計／互動任務／彩色規格／排程／
 * 條件式工單）、「狀態」列；中欄分析結果卡（有結果時）與閱讀卡；右欄本頁目錄。變更層動作經
 * `DetailActions` 進宿主標題列。宿主只在有選定變更時掛載本頁。
 */
export function ChangeDetailPage({
  change,
  refreshGen,
  loadDocument,
  loadCapabilities,
  loadMeta,
  loadStationTicket,
  securityReview,
  checkRecords,
  onOpenSecurityData,
  onRunVerb,
  detailVerb,
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
  changes,
  onSetDepends,
  loadDependsCandidates,
  unavailable,
  renderDocumentActions,
  onOpenSpec,
  pathReveal,
}: ChangeDetailPageProps) {
  const { t, tn } = useI18n();
  const [meta, setMeta] = useState<ChangeMetaInfo | null>(null);
  // 前置候選名冊：陣列＝查詢結果、null＝自清單派生（無查詢或查詢失敗）、undefined＝查詢中。
  const [roster, setRoster] = useState<string[] | null | undefined>(null);
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  // undefined＝capability 清單與其規格文件尚未載完（與「無 delta 規格」的空物件分流）。
  const [specDocs, setSpecDocs] = useState<Record<string, string | null> | undefined>();
  // 受控分頁：工單分頁退場時要能把當前分頁切回「提案」（design D3）。
  const [tab, setTab] = useState("proposal");
  // 能力籤→規格段、D 籤→決策卡、「前往」→任務列的跳轉請求（與已封存頁同一套）。
  const nav = useDocNav(setTab);
  const bodyRef = useRef<HTMLDivElement>(null);
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
  const [expandedChecks,setExpandedChecks]=useState<{change:string|null;review:boolean;verify:boolean}>({change:null,review:false,verify:false});
  const checksForCurrent=expandedChecks.change===name;
  const reviewHistoryOpen=checksForCurrent&&expandedChecks.review;
  const verifyHistoryOpen=checksForCurrent&&expandedChecks.verify;
  const checks=useCheckRecords(checkRecords,name,tab,gen,reviewHistoryOpen||verifyHistoryOpen);
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
      setRoster(loadDependsCandidates ? undefined : null);
    }
    const fresh = <T,>(apply: (v: T) => void) => (v: T) => {
      if (requestSeq.current === seq) apply(v);
    };
    // 失敗收斂：還沒有東西可顯示才落終態空文案（undefined 停著＝永久骨架）；
    // 已有內容維持前值——重載的短暫失敗不得抹成假空態。
    const settled = <T,>(apply: Dispatch<SetStateAction<T | null | undefined>>) => () => {
      if (requestSeq.current === seq) apply((prev) => (prev === undefined ? null : prev));
    };
    void loadMeta(target).then(fresh(setMeta)).catch(() => undefined);
    if (loadDependsCandidates) {
      // 失敗：首載退回自清單派生（null），重載維持前一次的名冊。
      void loadDependsCandidates(target).then(fresh(setRoster)).catch(settled(setRoster));
    }
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
    if (!name) return;
    loadAll(gen, name, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [name]);

  // 外部世代重載：不清空、回應到達後單次替換（不重置分頁與捲動）。
  // 互動進行中（拖曳手勢 dragActive、批次寫回 taskBusy、在途單發寫回 pendingWrites）
  // 讓路——皆為依賴，互動結束時本 effect 重跑補載一次。
  useEffect(() => {
    if (!name || taskBusy || dragActive || pendingWrites > 0) return;
    if (gen <= loadedGen.current) return;
    loadAll(gen, name, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [name, gen, taskBusy, dragActive, pendingWrites]);

  const inReview = change?.reviewStatus === "inReview";
  const inVerify = change?.verifyStatus === "inVerify";
  const [securityInspection, setSecurityInspection] = useState<SecurityInspection | null>(null);
  const [securityError, setSecurityError] = useState<string | null>(null);
  useEffect(() => {
    let current = true;
    setSecurityInspection(null); setSecurityError(null);
    if (securityReview && name) securityReview.inspect(name).then((next) => {
      if (current) setSecurityInspection(next);
    }).catch((error: unknown) => { if (current) setSecurityError(error instanceof Error ? error.message : String(error)); });
    return () => { current = false; };
  }, [securityReview, name, gen]);
  const securityVisible = !!securityReview && (!!securityError || !!securityInspection && (securityInspection.enabled !== false || !!securityInspection.latest));
  // 兩站工單（design D3）：分頁出現時載入、隨世代重載、分頁消失時清回未載入。
  const tickets = useStationTickets(name, { review: inReview, verify: inVerify }, gen, loadStationTicket);
  // 開啟／換 change 從「提案」起。
  useEffect(() => {
    setTab("proposal");
    nav.reset();
  }, [name]);
  // 停在工單分頁而該站翻為非進行中（工單已蓋章或放棄刪除）：分頁消失，退回「提案」。
  useEffect(() => {
    if ((tab === "review" && !inReview) || (tab === "verify" && !inVerify && !securityVisible)) setTab("proposal");
  }, [tab, inReview, inVerify, securityVisible]);

  if (!change) return null;

  // 分析結果是否對本 change 開啟——分析鈕切換態與面板呈現共用同一判定（design D2）。
  const verbOpen = !!(detailVerb && detailVerb.change === change.name);
  const pct = change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const taskBadge = `${change.completedTasks}/${change.totalTasks}`;
  const delta = sumDeltaCounts(Object.values(specDocs ?? {}).map(specDeltaCounts));
  const rel = relativeDays(meta?.created, { t, tn });

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

  // 兩站章籤：圖示＋狀態詞依四態上色（stamp／progress／warning／destructive），
  // 只在「狀態」區的「品質關卡」列渲染——身分區只放階段籤。
  const reviewStatus: ReviewBadgeStatus | null =
    !change.reviewStatus || change.reviewStatus === "none" ? null : change.reviewStatus;
  const verifyStatus: VerifyBadgeStatus | null =
    !change.verifyStatus || change.verifyStatus === "none" ? null : change.verifyStatus;
  const stamps: Stamp[] = [];
  if (reviewStatus) {
    stamps.push({
      station: "review",
      tone: REVIEW_PILL_TONE[reviewStatus],
      icon: REVIEW_ICON[reviewStatus],
      label: t(REVIEW_LABEL_KEY[reviewStatus]),
      at: reviewStatus === "inReview" ? undefined : change.reviewedAt,
      by: reviewStatus === "inReview" ? undefined : change.reviewedBy,
    });
  }
  if (verifyStatus) {
    stamps.push({
      station: "verify",
      tone: VERIFY_PILL_TONE[verifyStatus],
      icon: VERIFY_ICON[verifyStatus],
      label: t(VERIFY_LABEL_KEY[verifyStatus]),
      at: verifyStatus === "inVerify" ? undefined : change.verifiedAt,
      by: verifyStatus === "inVerify" ? undefined : change.verifiedBy,
    });
  }

  // 階段守門原因（archive-readiness-gating D4）：封存非已就緒、刪除非提案中時停用；
  // unavailable（remote 能力缺失）原因優先於階段原因——「這條通道做不到」比「現在還
  // 不能」更硬。守門為呈現層過濾，引擎拒絕是最終裁決。
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

  const wave = planWave(change);
  const blockedBy = planBlockedBy(change);

  const docTarget:DocumentTarget|null = checks.entry?{doc:{change:change.name,artifact:checks.entry.relativePath},content:checks.content}:documentTarget(change.name, tab, { proposal, design, tasks: tasksMd }, specDocs, nav.specCap);
  const docPath = documentArtifact(docTarget);
  // 排程沒有檔案，內容即變更的排程欄位。
  const tocKey = changeDocTocKey(tab, { content: tab === "plan" ? change : docTarget?.content, specs: specDocs, tickets });
  const specCaps = Object.keys(specDocs ?? {});
  // 提案列為新能力的能力在封存前沒有正式規格：其規格段不給「看正式規格」。
  const newCaps = proposal ? newCapabilities(proposal) : [];
  // 閱讀卡標題區的籤槽、數字磚與提示框（design D9）：提案的提示框接在能力段尾，其餘在磚與內文之間。
  const summary = changeDocSummary(
    tab,
    {
      proposal,
      design,
      tasksMd,
      specDocs,
      change,
      tickets,
      onLocateTask: nav.locateTask,
    },
    { t, tn },
  );
  const calloutSlot = summary.callout && <div data-doc-callout>{summary.callout}</div>;
  // 工單節點的徽章：末輪序號（載入完成且有輪次時）。
  const lastRound = (station: TicketStation) => {
    const rounds = tickets[station]?.rounds ?? [];
    return rounds.length > 0 ? Math.max(...rounds.map((r) => r.index)) : null;
  };

  const actions = (
    <>
      <UnavailableAction reason={unavailable?.analyze}>
        <Button
          variant="outline"
          size="sm"
          aria-pressed={verbOpen}
          disabled={unavailable?.analyze !== undefined}
          title={unavailable?.analyze}
          className={verbOpen ? "bg-accent" : undefined}
          onClick={() => (verbOpen ? onClearVerb?.() : onRunVerb?.("analyze", change.name))}
        >
          <Sparkles className="h-3.5 w-3.5" /> {t("common.analyze")}
        </Button>
      </UnavailableAction>
      {/* 認領（remote-claim-ownership D4）：僅未認領的 change 提供入口——已有持有人時
          出身列已寫明是誰。onClaim 缺席＝本地分頁，整段不渲染。 */}
      {onClaim && !change.claimedBy && (
        <UnavailableAction reason={unavailable?.claim}>
          <Button
            size="sm"
            disabled={unavailable?.claim !== undefined}
            title={unavailable?.claim}
            onClick={() => onClaim(change.name)}
          >
            <Hand className="h-3.5 w-3.5" /> {t("rdrawer.claim")}
          </Button>
        </UnavailableAction>
      )}
      {/* 依階段的主要鈕（實心主色）：退回提案中（僅派生進行中）點擊交宿主先確認，UI 不預判守門。 */}
      {stage === "in-progress" && onRevert && (
        <Button size="sm" onClick={() => onRevert(change.name)}>
          <Undo2 className="h-3.5 w-3.5" /> {t("common.revert")}
        </Button>
      )}
      {/* 已就緒才有「封存」主要鈕，與「⋯」選單項同一守門。 */}
      {stage === "ready" && (
        <UnavailableAction reason={unavailable?.archive}>
          <Button
            size="sm"
            disabled={unavailable?.archive !== undefined}
            title={unavailable?.archive}
            onClick={() => onRunVerb?.("archive", change.name)}
          >
            <Archive className="h-3.5 w-3.5" /> {t("common.archive")}
          </Button>
        </UnavailableAction>
      )}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="icon-sm" aria-label={t("detail.more")}>
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-72">
          <DropdownMenuItem
            className="h-auto py-1.5"
            disabled={archiveReason !== undefined}
            onSelect={() => onRunVerb?.("archive", change.name)}
          >
            <MenuAction icon={<Archive />} label={t("detail.archiveMenu")} reason={archiveReason} />
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            variant="destructive"
            className="h-auto py-1.5"
            disabled={deleteReason !== undefined}
            onSelect={() => onDelete?.(change.name)}
          >
            <MenuAction icon={<Trash2 />} label={t("detail.deleteMenu")} reason={deleteReason} />
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </>
  );

  const startedLabel = meta?.startedAt ? t("rdrawer.started").replace("{date}", meta.startedAt) : null;
  const sources = sourceDiscussions ?? [];
  const siblings = siblingChanges ?? [];
  const relations: DetailRelation[] = [];
  if (sources.length > 0) {
    relations.push({
      mark: "from",
      label: t("detail.relFrom"),
      children: <SourceChipRow items={sources} onOpen={(slug) => onOpenDiscussion?.(slug)} />,
    });
  }
  if (siblings.length > 0) {
    relations.push({
      mark: "sibling",
      label: t("detail.relSibling"),
      children: (
        <SourceChipRow items={siblings.map((sibling) => ({ slug: sibling }))} onOpen={(sibling) => onOpenSibling?.(sibling)} />
      ),
    });
  }
  if (specCaps.length > 0) {
    relations.push({
      mark: "specs",
      label: t("detail.relSpecs"),
      children: specCaps.map((cap) => (
        <button key={cap} type="button" className="min-w-0 max-w-full" onClick={() => nav.openSpec(cap)}>
          <Badge variant="outline" className="block max-w-full truncate font-mono hover:bg-accent">
            {cap}
          </Badge>
        </button>
      )),
    });
  }
  const ticketBadge = (station: TicketStation) => {
    const n = lastRound(station);
    return n === null ? null : (
      <Badge variant="secondary" className="ml-auto">
        {t("ticket.round").replace("{n}", String(n))}
      </Badge>
    );
  };

  const tree = (
    <>
      <DetailIdentity
        title={change.name}
        copyLabel={t("common.copyName")}
        badges={
          <StatusPill data-stage-pill tone={STAGE_PILL_TONE[stage]}>
            {t(`stage.${stage}`)}
          </StatusPill>
        }
        provenance={[
          meta?.createdBy && <IdentityName identity={meta.createdBy} />,
          meta?.createdWith && <span className="shrink-0">✳ {meta.createdWith}</span>,
          rel && <span className="shrink-0">{rel}</span>,
          startedLabel &&
            (meta?.startedBy ? (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="shrink-0">{startedLabel}</span>
                </TooltipTrigger>
                <TooltipContent>{meta.startedBy}</TooltipContent>
              </Tooltip>
            ) : (
              <span className="shrink-0">{startedLabel}</span>
            )),
          change.claimedBy && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="inline-flex shrink-0 items-center gap-1">
                  <Hand className="h-3 w-3" />
                  {t("rdrawer.claimed").replace("{name}", displayName(change.claimedBy))}
                </span>
              </TooltipTrigger>
              <TooltipContent>{change.claimedBy}</TooltipContent>
            </Tooltip>
          ),
          change.worktree && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="inline-flex shrink-0 items-center gap-1" title={change.worktree.path}>
                  <GitBranch className="h-3 w-3" />
                  {change.worktree.branch}
                </span>
              </TooltipTrigger>
              <TooltipContent>{change.worktree.path}</TooltipContent>
            </Tooltip>
          ),
        ]}
        relations={relations}
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
          <Badge variant={pct === 100 ? "default" : "secondary"} className="ml-auto">
            {taskBadge}
          </Badge>
        </TabsTrigger>
        <TabsTrigger value="specs">
          <Code2 className="h-3.5 w-3.5" /> {t("common.tabSpecs")}
          <span className="ml-auto">
            <DeltaBadges counts={delta} />
          </span>
        </TabsTrigger>
        <TabsTrigger value="plan">
          <ListOrdered className="h-3.5 w-3.5" /> {t("drawer.tab.plan")}
          {wave !== null && (
            <Badge variant="secondary" className="ml-auto">
              {planWaveLabel(wave, t)}
            </Badge>
          )}
        </TabsTrigger>
        {/* 條件式工單節點（spec desktop-app「詳情頁的工單分頁」）：排程之後、審查在前，
            與「品質關卡」列章籤同序；節點名即站名，徽章為末輪序號。 */}
        {inReview && (
          <TabsTrigger value="review">
            {REVIEW_ICON.inReview} {t("drawer.tab.review")}
            {ticketBadge("review")}
          </TabsTrigger>
        )}
        {(inVerify || securityVisible) && (
          <TabsTrigger value="verify">
            {VERIFY_ICON.inVerify} {t("drawer.tab.verify")}
            {ticketBadge("verify")}
          </TabsTrigger>
        )}
      </TabsList>

      <DetailTreeLabel>{t("detail.status")}</DetailTreeLabel>
      <DetailStatus>
        <StatusRow mark="tasks" label={t("detail.statusTasks")}>
          <div data-progress-track className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
            <div className="h-full rounded-full bg-primary" style={{ width: `${pct}%` }} />
          </div>
          <span className="shrink-0 font-mono text-[11px] tabular-nums text-muted-foreground">{taskBadge}</span>
        </StatusRow>
        <StatusRow mark="quality" label={t("detail.statusQuality")} wrap>
          {stamps.length === 0 ? (
            <span className="text-muted-foreground">{t("detail.noQuality")}</span>
          ) : (
            stamps.map((s) => {
              const id=`quality-records-${s.station}`;
              const expanded=s.station==="review"?reviewHistoryOpen:verifyHistoryOpen;
              const toggle=()=>setExpandedChecks(previous=>({
                change:name,
                review:(previous.change===name?previous.review:false)!==(s.station==="review"),
                verify:(previous.change===name?previous.verify:false)!==(s.station==="verify"),
              }));
              return <div key={s.station} className="flex min-w-0 flex-wrap items-center gap-x-2">
                <QualityStamp stamp={s} expanded={expanded} controlsId={id} onToggle={toggle}/>
                <QualityCheckRecordList id={id} kind={s.station} state={checks} onSelect={entry=>setTab(checkTab(entry))} hidden={!expanded}/>
              </div>;
            })
          )}
        </StatusRow>
        <StatusRow mark="plan" label={t("detail.statusPlan")}>
          {wave === null ? (
            <span className="truncate text-muted-foreground">
              {t(change.dependsOn ? "plan.cycle" : "plan.unavailable")}
            </span>
          ) : (
            <>
              <span className="font-medium">{planWaveLabel(wave, t)}</span>
              <span className="text-muted-foreground">·</span>
              <span
                className="truncate text-muted-foreground"
                title={blockedBy.length > 0 ? planBlockedLabel(blockedBy, t) : undefined}
              >
                {blockedBy.length === 0
                  ? t("plan.canStart")
                  : t("plan.blockedCount").replace("{n}", String(blockedBy.length))}
              </span>
            </>
          )}
        </StatusRow>
      </DetailStatus>
    </>
  );

  return (
    <TooltipProvider>
      {/* 直向節點、Enter／Space 才切換（design D2）：方向鍵掃過節點不逐一載文件。 */}
      <Tabs value={tab} onValueChange={setTab} orientation="vertical" activationMode="manual" className="h-full">
        {/* 換文件或換變更（‹ ›）都讓閱讀區自頂端呈現：頁不因換變更重掛。 */}
        <DetailShell
          data-change-detail
          tree={tree}
          activeDoc={`${change.name}\u0000${tab}`}
          tocKey={tocKey}
          bodyRef={bodyRef}
        >
          <DetailActions>{actions}</DetailActions>

          {/* 分析結果（validate＋analyze 合併）於閱讀卡之上呈現——僅當前 change 相符時。 */}
          {verbOpen && detailVerb && (
            <Card data-verb-result className="p-4">
              {detailVerb.error ? (
                <div className="text-xs text-destructive">{detailVerb.error}</div>
              ) : (
                <AnalyzePanel report={detailVerb.analyze} validate={detailVerb.validate} onClose={onClearVerb} />
              )}
            </Card>
          )}

          <Card className="flex flex-col gap-5 p-6">
            <DocHeader
              title={t(tab.startsWith("check:")?"checks.title":DOC_TITLE_KEYS[tab])}
              badges={summary.badges}
              actions={renderDocumentActions?.(docTarget)}
              provenance={[
                tab === "plan" ? t("plan.derivedFrom") : docPath && <span className="font-mono">{docPath}</span>,
              ]}
            />
            <DocTiles tiles={summary.tiles} />
            {tab !== "proposal" && calloutSlot}
            <DocBody ref={bodyRef}>
              <CheckRecordBody state={checks} tab={tab}/>
              {/* undefined＝載入在途、null＝文件不存在（空殼 change）——載入中畫骨架，
                  空態文案只在載入完成後出。 */}
              <TabsContent value="proposal">
                {proposal === undefined ? (
                  <DocSkeleton />
                ) : (
                  <ProposalDoc
                    content={proposal}
                    empty={t("list.noProposalDoc")}
                    deltaCaps={specCaps}
                    onOpenCapability={nav.openSpec}
                    pathReveal={pathReveal}
                    afterCapabilities={tab === "proposal" ? calloutSlot : null}
                  />
                )}
              </TabsContent>
              <TabsContent value="design">
                {design === undefined ? (
                  <DocSkeleton />
                ) : (
                  <DesignDoc content={design} empty={t("list.noDesignDoc")} jump={nav.designJump} onJumped={nav.clearDesignJump} />
                )}
              </TabsContent>
              <TabsContent value="tasks">
                {taskError && (
                  <div className="mb-2 text-sm text-destructive">
                    {t("tasks.writeFailed").replace("{msg}", taskError)}
                  </div>
                )}
                {unavailable?.tasks && (
                  <div data-testid="tasks-unavailable" className="mb-2 text-sm text-muted-foreground">
                    {unavailable.tasks}
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
                      onMoveTask ? (from, to, before) => void handleReorder(from, to, before) : undefined
                    }
                    onSetAll={(done) => void handleSetAll(done)}
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
                  <div className="py-6 text-sm text-muted-foreground">{t("list.noDeltaSpecs")}</div>
                ) : (
                  Object.entries(specDocs).map(([cap, doc]) => (
                    <DeltaCapSection
                      key={cap}
                      cap={cap}
                      doc={doc}
                      jump={nav.specJump}
                      onJumped={nav.clearSpecJump}
                      onOpenSpec={newCaps.includes(cap) ? undefined : onOpenSpec}
                    />
                  ))
                )}
              </TabsContent>
              <TabsContent value="plan">
                <PlanDoc change={change} changes={changes ?? []} roster={roster} onSetDepends={onSetDepends} />
              </TabsContent>
              {inReview && (
                <TabsContent value="review">
                  <TicketTabBody station="review" ticket={tickets.review} />
                </TabsContent>
              )}
              {(inVerify || securityVisible) && (
                <TabsContent value="verify">
                  {inVerify && <TicketTabBody station="verify" ticket={tickets.verify} />}
                  {securityError && <p role="alert">{securityError}</p>}
                  {securityReview && securityInspection && securityVisible && <SecurityReviewPanel adapter={securityReview} change={change.name} inspection={securityInspection} onInspection={setSecurityInspection} onOpenDataSettings={onOpenSecurityData} />}
                </TabsContent>
              )}
            </DocBody>
          </Card>
        </DetailShell>
      </Tabs>
    </TooltipProvider>
  );
}
