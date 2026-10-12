import {
  useCallback,
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
  X,
} from "lucide-react";

import type {
  ChangeItem,
  ChangeMetaInfo,
  DeltaOperation,
  StationTicket,
  TicketStation,
  Verb,
  VerbDrawerResult,
} from "../adapter";
import { specDeltaCounts, sumDeltaCounts } from "../delta";
import {
  changeStage,
  planArchiveAfter,
  planBlockedBy,
  planBlockedLabel,
  planRequirementOverlap,
  planWave,
  planWaveLabel,
} from "../stage";
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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { SourceChipRow } from "./SourceDiscussionChip";
import { SectionedDoc } from "./SectionedDoc";
import { DocSkeleton } from "./skeletons";
import { TaskList } from "./TaskList";
import { DELTA_COLORS, DELTA_LABEL_KEYS, DeltaBadges, DeltaSpecView } from "./DeltaBadges";
import { AnalyzePanel } from "./AnalyzePanel";
import { TicketTabBody, useStationTickets } from "./TicketView";
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

/** 標題列的站章籤：可視只有圖示＋狀態詞，蓋章日期與蓋章者完整識別（含 email）
    收進提示。兩站同構，只換站別——資料缺席（進行中、無戳記）時整個提示缺席。 */
function TitleStamp({ stamp }: { stamp: Stamp }) {
  // 既有 data 標記維持站別各一份（回歸對照沿用 data-review-row／data-verify-row）。
  const rowMark = stamp.station === "review" ? { "data-review-row": "" } : { "data-verify-row": "" };
  const toneMark = stamp.station === "review" ? { "data-review-tone": "" } : { "data-verify-tone": "" };
  const chip = (
    <span {...rowMark} className="inline-flex shrink-0">
      <StatusPill {...toneMark} tone={stamp.tone} icon={stamp.icon}>
        {stamp.label}
      </StatusPill>
    </span>
  );
  const detail = [stamp.at, stamp.by].filter(Boolean).join(" · ");
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
 * 規格文件的一個 capability 段。`jump` 指到它時捲到段首並回報清除：捲動放在段自己的 effect，
 * 因為 Radix 的 TabsContent 在切換後的下一輪 render 才掛上內文，頁層 effect 那時還找不到段。
 */
function SpecCapSection({
  cap,
  doc,
  jump,
  onJumped,
}: {
  cap: string;
  doc: string | null;
  jump: { cap: string } | null;
  onJumped: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (jump?.cap !== cap) return;
    ref.current?.scrollIntoView?.({ block: "start" });
    onJumped();
  }, [jump, cap, onJumped]);
  return (
    <div ref={ref} data-spec-cap={cap} className="mb-4">
      <div className="mb-1 flex items-center gap-2 text-xs font-semibold tracking-wider text-muted-foreground uppercase">
        {cap} <DeltaBadges counts={specDeltaCounts(doc)} />
      </div>
      {/* 逐項空態（capability 有列出、其 spec.md 為空）不套整體性文案。 */}
      <DeltaSpecView markdown={doc} />
    </div>
  );
}

/** 排程分頁內的名稱籤（同波夥伴、前置、重疊、封存順序共用）。 */
function PlanName({ name }: { name: string }) {
  return <span className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{name}</span>;
}

/** 重疊卡的 delta 操作小標籤：用詞與配色和規格分頁的 delta 區段同源；title 標出屬於哪一方。 */
function OpTag({ op, owner }: { op: DeltaOperation; owner: string }) {
  const { t } = useI18n();
  const key = op.toLowerCase() as Lowercase<DeltaOperation>;
  return (
    <span title={owner} className={`rounded border border-border/60 px-1 py-0.5 text-[10px] ${DELTA_COLORS[key]}`}>
      {t(DELTA_LABEL_KEYS[key])}
    </span>
  );
}

/** 排程分頁的一張卡片：外觀沿工單分頁的邊框卡片，標題列（右側可放徽章）與內容區以分隔線分格。 */
function PlanCard({
  section,
  title,
  badge,
  children,
}: {
  section: "wave" | "depends" | "overlaps" | "archive";
  title: string;
  badge?: ReactElement;
  children: ReactNode;
}) {
  return (
    <section data-plan-section={section} className="rounded-md border border-border/60 bg-muted/20">
      <div
        data-plan-card-header
        className="flex items-center justify-between px-2 py-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground"
      >
        <span data-doc-heading>{title}</span>
        {badge}
      </div>
      <div data-plan-card-body className="border-t border-border/60 px-2 py-1.5 text-sm">
        {children}
      </div>
    </section>
  );
}

/**
 * 排程分頁（spec desktop-app「詳情頁的排程分頁」）：波次（同波夥伴）、前置（可增減，
 * 標題列的狀態徽章取代獨立的阻擋段）、重疊（唯讀）、封存順序四張卡片。清單項缺 wave
 * 時：remote 摘要（亦缺 dependsOn）只剩一句說明；plan 成環（payload 仍帶 dependsOn
 * 原文）另渲染前置卡（無徽章）、只能移除——解環的出口。判定沿 stage.ts 的 planWave／
 * planBlockedBy／planRequirementOverlap／planArchiveAfter 單一入口。
 */
function PlanTab({
  change,
  changes,
  roster,
  onSetDepends,
}: {
  change: ChangeItem;
  changes: ChangeItem[];
  /** 候選名冊：陣列＝查詢結果、null＝自清單派生、undefined＝查詢中（不給候選）。 */
  roster: string[] | null | undefined;
  onSetDepends?: (change: string, on: string[], remove: boolean) => void;
}) {
  const { t } = useI18n();
  const wave = planWave(change);
  const dependsOn = change.dependsOn ?? [];
  // 前置卡內文（成環與一般兩個分支共用）：宣告的前置每項一列，空時「尚無前置」。
  const dependsBody = (
    <>
      {dependsOn.length === 0 && <div className="text-muted-foreground">{t("plan.noDepends")}</div>}
      <ul className="flex flex-col gap-1">
        {dependsOn.map((n) => (
          <li key={n} className="flex items-center gap-2">
            <PlanName name={n} />
            {onSetDepends && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                aria-label={t("plan.remove").replace("{name}", n)}
                onClick={() => onSetDepends(change.name, [n], true)}
              >
                <X className="h-3.5 w-3.5" />
              </Button>
            )}
          </li>
        ))}
      </ul>
    </>
  );
  if (wave === null) {
    return (
      <div className="flex flex-col gap-3 text-sm">
        <div className="text-muted-foreground py-6">{t(change.dependsOn ? "plan.cycle" : "plan.unavailable")}</div>
        {change.dependsOn && (
          <PlanCard section="depends" title={t("plan.depends")}>
            {dependsBody}
          </PlanCard>
        )}
      </div>
    );
  }
  const requirementOverlaps = planRequirementOverlap(change);
  const archiveAfter = planArchiveAfter(change);
  const blockedBy = planBlockedBy(change);
  // 舊 server 的 blockedBy 另含先配置、共用 capability 的變更（本機與新 server 只含宣告
  // 前置）：列出 dependsOn 以外的名稱，徽章的數字才對得上卡片內容。
  const undeclaredWaits = blockedBy.filter((n) => !dependsOn.includes(n));
  const others = changes.filter((c) => c.name !== change.name);
  const mates = others.filter((c) => planWave(c) === wave).map((c) => c.name);
  // 新增候選：該變更所在名冊的其他作用中變更（add-change-plan-remote D9；未提供名冊時
  // 自清單派生），扣掉自己與已宣告的前置（成環留給引擎拒絕）。
  const pool = roster === undefined ? [] : (roster ?? others.map((c) => c.name));
  const candidates = pool.filter((n) => n !== change.name && !dependsOn.includes(n));
  // 封存卡的兩種警示：同名衝突（雙方都帶進或都拿走，先後救不了）與互相等待（雙方互列
  // archiveAfter，封存互卡）——兩者引擎都不會讓任一方「可直接封存」。
  const conflictWith = [...new Set(requirementOverlaps.filter((o) => o.conflict).map((o) => o.change))];
  const mutualWaits = archiveAfter.filter((n) =>
    others.some((c) => c.name === n && planArchiveAfter(c).includes(change.name)),
  );
  const names = (list: string[]) => list.join(t("common.listSeparator"));
  // 徽章是狀態不是標題：不繼承標題列的全大寫與字距。
  const statusBadge =
    blockedBy.length === 0 ? (
      <Badge variant="secondary" className="normal-case tracking-normal">
        {t("plan.canStart")}
      </Badge>
    ) : (
      <Badge variant="outline" className="normal-case tracking-normal" title={planBlockedLabel(blockedBy, t)}>
        {t("plan.blockedCount").replace("{n}", String(blockedBy.length))}
      </Badge>
    );
  return (
    <div className="flex flex-col gap-3 text-sm">
      <PlanCard section="wave" title={t("plan.waveLabel")}>
        <div className="font-medium">{planWaveLabel(wave, t)}</div>
        {mates.length === 0 ? (
          <div className="text-muted-foreground">{t("plan.onlyOne")}</div>
        ) : (
          <div className="mt-1 flex flex-wrap items-center gap-1.5">
            <span className="text-muted-foreground">{t("plan.waveMates")}</span>
            {mates.map((n) => (
              <PlanName key={n} name={n} />
            ))}
          </div>
        )}
      </PlanCard>
      <PlanCard section="depends" title={t("plan.depends")} badge={statusBadge}>
        {dependsBody}
        {undeclaredWaits.length > 0 && (
          <div data-plan-undeclared-waits className="mt-1 flex flex-wrap items-center gap-1.5">
            <span className="text-muted-foreground">{t("plan.undeclaredWaits")}</span>
            {undeclaredWaits.map((n) => (
              <PlanName key={n} name={n} />
            ))}
          </div>
        )}
        {onSetDepends && candidates.length > 0 && (
          // 只當挑選器用：value 固定為空，選擇即寫入、trigger 回到提示文字。
          <Select value="" onValueChange={(name) => onSetDepends(change.name, [name], false)}>
            <SelectTrigger aria-label={t("plan.add")} className="mt-2 w-full">
              <SelectValue placeholder={t("plan.add")} />
            </SelectTrigger>
            <SelectContent>
              {candidates.map((n) => (
                <SelectItem key={n} value={n}>
                  {n}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </PlanCard>
      <PlanCard section="overlaps" title={t("plan.overlaps")}>
        {requirementOverlaps.length === 0 && <div className="text-muted-foreground">{t("plan.noOverlaps")}</div>}
        <ul className="flex flex-col gap-1">
          {requirementOverlaps.map((o) => (
            <li key={`${o.change}/${o.capability}/${o.requirement}`} className="flex flex-wrap items-center gap-1.5">
              <PlanName name={o.change} />
              <span className="text-xs">{`${o.capability} › ${o.requirement}`}</span>
              {/* 雙方操作：本變更在前、對方在後。 */}
              <OpTag op={o.ownOperation} owner={change.name} />
              <OpTag op={o.otherOperation} owner={o.change} />
              {o.conflict && (
                <span className="rounded border border-destructive/60 px-1 py-0.5 text-[10px] text-destructive">
                  {t("plan.conflict")}
                </span>
              )}
            </li>
          ))}
        </ul>
      </PlanCard>
      <PlanCard section="archive" title={t("plan.archiveAfter")}>
        {archiveAfter.length === 0 && conflictWith.length === 0 && (
          <div className="text-muted-foreground">{t("plan.noArchiveAfter")}</div>
        )}
        {archiveAfter.length > 0 && (
          <>
            <div className="flex flex-wrap items-center gap-1.5">
              {archiveAfter.map((n) => (
                <PlanName key={n} name={n} />
              ))}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">{t("plan.archiveHint")}</p>
          </>
        )}
        {mutualWaits.length > 0 && (
          <p className="mt-1 text-xs text-destructive">{t("plan.archiveDeadlock").replace("{names}", names(mutualWaits))}</p>
        )}
        {conflictWith.length > 0 && (
          <p className="mt-1 text-xs text-destructive">{t("plan.archiveConflict").replace("{names}", names(conflictWith))}</p>
        )}
      </PlanCard>
    </div>
  );
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
}: ChangeDetailPageProps) {
  const { t } = useI18n();
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
  // 規格列能力籤選定的 capability：「開啟 ▾」指向它。
  const [specCap, setSpecCap] = useState<string | null>(null);
  // 能力籤的捲動請求：每點一次換一個物件，該規格段捲到後即清——自文件樹切回規格文件時自頂端呈現。
  const [specJump, setSpecJump] = useState<{ cap: string } | null>(null);
  const clearSpecJump = useCallback(() => setSpecJump(null), []);
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
  // 兩站工單（design D3）：分頁出現時載入、隨世代重載、分頁消失時清回未載入。
  const tickets = useStationTickets(name, { review: inReview, verify: inVerify }, gen, loadStationTicket);
  // 開啟／換 change 從「提案」起。
  useEffect(() => {
    setTab("proposal");
    setSpecCap(null);
    setSpecJump(null);
  }, [name]);
  // 停在工單分頁而該站翻為非進行中（工單已蓋章或放棄刪除）：分頁消失，退回「提案」。
  useEffect(() => {
    if ((tab === "review" && !inReview) || (tab === "verify" && !inVerify)) setTab("proposal");
  }, [tab, inReview, inVerify]);

  if (!change) return null;

  // 分析結果是否對本 change 開啟——分析鈕切換態與面板呈現共用同一判定（design D2）。
  const verbOpen = !!(detailVerb && detailVerb.change === change.name);
  const pct = change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const taskBadge = `${change.completedTasks}/${change.totalTasks}`;
  const delta = sumDeltaCounts(Object.values(specDocs ?? {}).map(specDeltaCounts));
  const rel = relativeDays(meta?.created, t);

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

  // 兩站章籤：圖示＋狀態詞依四態上色（stamp／progress／warning／destructive）；
  // 標題列與狀態卡「品質關卡」列共用同一份資料。
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

  const docTarget = documentTarget(change.name, tab, { proposal, design, tasks: tasksMd }, specDocs, specCap);
  const docPath = documentArtifact(docTarget);
  // 排程沒有檔案，內容即變更的排程欄位。
  const tocKey = changeDocTocKey(tab, { content: tab === "plan" ? change : docTarget?.content, specs: specDocs, tickets });
  const specCaps = Object.keys(specDocs ?? {});
  const openSpec = (cap: string) => {
    setSpecCap(cap);
    setSpecJump({ cap });
    setTab("specs");
  };
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
        <button key={cap} type="button" className="min-w-0 max-w-full" onClick={() => openSpec(cap)}>
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
          <>
            <StatusPill data-stage-pill tone={STAGE_PILL_TONE[stage]}>
              {t(`stage.${stage}`)}
            </StatusPill>
            {stamps.map((s) => (
              <TitleStamp key={s.station} stamp={s} />
            ))}
          </>
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
            與身分區章籤同序；節點名即站名，徽章為末輪序號。 */}
        {inReview && (
          <TabsTrigger value="review">
            {REVIEW_ICON.inReview} {t("drawer.tab.review")}
            {ticketBadge("review")}
          </TabsTrigger>
        )}
        {inVerify && (
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
            stamps.map((s) => (
              <span key={s.station} className="inline-flex shrink-0 items-center gap-1.5">
                <StatusPill tone={s.tone} icon={s.icon}>
                  {s.label}
                </StatusPill>
                {s.at && <span className="font-mono text-[11px] text-muted-foreground">{s.at}</span>}
              </span>
            ))
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
              title={t(DOC_TITLE_KEYS[tab])}
              actions={renderDocumentActions?.(docTarget)}
              provenance={[
                tab === "plan" ? t("plan.derivedFrom") : docPath && <span className="font-mono">{docPath}</span>,
              ]}
            />
            {/* 數字磚與提示框的容器：本刀不放內容（cut 2／3 由文件結構推導）。 */}
            <DocTiles tiles={[]} />
            <DocBody ref={bodyRef}>
              {/* undefined＝載入在途、null＝文件不存在（空殼 change）——載入中畫骨架，
                  空態文案只在載入完成後出。 */}
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
                    <SpecCapSection key={cap} cap={cap} doc={doc} jump={specJump} onJumped={clearSpecJump} />
                  ))
                )}
              </TabsContent>
              <TabsContent value="plan">
                <PlanTab change={change} changes={changes ?? []} roster={roster} onSetDepends={onSetDepends} />
              </TabsContent>
              {inReview && (
                <TabsContent value="review">
                  <TicketTabBody station="review" ticket={tickets.review} />
                </TabsContent>
              )}
              {inVerify && (
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
