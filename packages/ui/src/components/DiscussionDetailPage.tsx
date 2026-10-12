import { useEffect, useRef, useState, type ReactNode } from "react";
import { Archive, ArrowUpRight, Check, Circle, FileText, Flag, MessagesSquare, Rocket } from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionItem } from "../adapter";
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
import { LABEL_CLS } from "./SectionedDoc";
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
import { isImproveKind } from "./improveStyle";

/** 討論記錄的三個標準區段。 */
export interface DiscussionSections {
  context: string;
  rounds: string;
  conclusion: string;
}

/** 結構標題白名單——與引擎（speclink-core discuss）同一條規則：
 * 只有這三個整行標題是區段邊界，輪內文的其他「## 」行不得截斷區段。 */
const STRUCTURAL_HEADERS: readonly string[] = ["## Context", "## Rounds", "## Conclusion"];

/** Fenced code block 圍欄行（``` 或 ~~~ 開頭；容忍前導空白）——圍欄內的行不是結構。 */
function isFenceLine(line: string): boolean {
  const t = line.trimStart();
  return t.startsWith("```") || t.startsWith("~~~");
}

/** 逐行標記是否位於 fenced code block（圍欄行自身視為圍欄內）。 */
function fenceFlags(lines: string[]): boolean[] {
  let fence = false;
  return lines.map((l) => {
    if (isFenceLine(l)) {
      fence = !fence;
      return true;
    }
    return fence;
  });
}

/** 區段邊界：結構標題白名單＋pre-scaffold「## Round 」前綴容忍（與引擎同一條規則）。 */
function isSectionBoundary(line: string): boolean {
  return STRUCTURAL_HEADERS.includes(line) || line.startsWith("## Round ");
}

/**
 * 把討論記錄全文按 `## Context`／`## Rounds`／`## Conclusion` 切分（design D5）。
 * 任一區段缺失（手寫、pre-scaffold 格式）回 `null`——呼叫端整篇以單一檢視退回。
 */
export function splitDiscussionSections(text: string): DiscussionSections | null {
  const lines = text.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const body = (name: string): string | null => {
    const start = lines.findIndex((l, i) => !inFence[i] && l.trimEnd() === `## ${name}`);
    if (start < 0) return null;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i++) {
      if (!inFence[i] && isSectionBoundary(lines[i].trimEnd())) {
        end = i;
        break;
      }
    }
    return lines.slice(start + 1, end).join("\n");
  };
  const context = body("Context");
  const rounds = body("Rounds");
  const conclusion = body("Conclusion");
  if (context === null || rounds === null || conclusion === null) return null;
  return { context, rounds, conclusion };
}

/** 輪欄位白名單——scaffold 固定四詞，其餘粗體前綴按內文照排（design D2）。 */
export type RoundLabel = "Focus" | "Position" | "Ruled out" | "Open";
const ROUND_LABELS: readonly RoundLabel[] = ["Focus", "Position", "Ruled out", "Open"];

/** 討論的一輪：卡頭（輪次/mode/日期）＋欄位標籤區塊（缺席欄位無該鍵）。 */
export interface DiscussionRound {
  round: number;
  mode: string;
  date: string;
  /** 首個欄位標籤前的內文（scaffold 記錄通常為空）。 */
  lead: string;
  fields: Partial<Record<RoundLabel, string>>;
}

/** 欄位切分結果：首個標籤前的內文（lead）＋標籤→內文對應。 */
interface LabeledFields<L extends string> {
  lead: string;
  fields: Partial<Record<L, string>>;
}

/**
 * 以標籤白名單切欄位（design D2／D7 共用實作）：行首「**<Label>**:」起新欄位，
 * 其餘行（含非白名單的粗體前綴行）歸屬當前欄位；首個標籤前的內容歸 lead。
 */
function splitLabeledFields<L extends string>(lines: string[], labels: readonly L[]): LabeledFields<L> {
  const re = new RegExp(`^\\*\\*(${labels.join("|")})\\*\\*:\\s?(.*)$`);
  const fields: Partial<Record<L, string>> = {};
  let lead = "";
  let field: L | null = null;
  const buf: string[] = [];

  const flush = () => {
    const content = buf.join("\n").trim();
    if (field) fields[field] = content;
    else lead = content;
    buf.length = 0;
  };

  for (const line of lines) {
    const m = re.exec(line);
    if (m) {
      flush();
      field = m[1] as L;
      buf.push(m[2]);
      continue;
    }
    buf.push(line);
  }
  flush();
  return { lead, fields };
}

/**
 * 把 Rounds 區段全文切成輪陣列（design D1 行掃描解析 scaffold）。
 * 每個 `### ` 標題必須符合「### Round N — <mode> (<date>)」；任一不符、或首輪前
 * 出現非註解內容即回 `null`——呼叫端整篇以單一 markdown 檢視退回。零輪回空陣列。
 */
export function splitRounds(text: string): DiscussionRound[] | null {
  const heading = /^### Round (\d+) — (.+?) \((.+)\)\s*$/;
  const rounds: { round: number; mode: string; date: string; body: string[] }[] = [];
  let current: { body: string[] } | null = null;

  const lines = text.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  for (const [i, line] of lines.entries()) {
    // 圍欄內的行不是輪標題——歸當前輪內文（首輪前出現則按既有規則整篇退回）。
    if (!inFence[i] && line.startsWith("### ")) {
      const m = heading.exec(line);
      if (!m) return null;
      current = { body: [] };
      rounds.push({ round: Number(m[1]), mode: m[2], date: m[3], body: current.body });
      continue;
    }
    if (!current) {
      // 首輪前僅容許空行與 scaffold 註解；其餘內容代表非標準記錄，整篇退回。
      if (line.trim() === "" || /^<!--.*-->\s*$/.test(line.trim())) continue;
      return null;
    }
    current.body.push(line);
  }
  return rounds.map((r) => ({ round: r.round, mode: r.mode, date: r.date, ...splitLabeledFields(r.body, ROUND_LABELS) }));
}

/** 輪欄位標籤的 i18n key（scaffold 四詞白名單 → 各語系標籤）。 */
const ROUND_LABEL_KEYS: Record<RoundLabel, string> = {
  Focus: "rounds.focus",
  Position: "rounds.position",
  "Ruled out": "rounds.ruledOut",
  Open: "rounds.open",
};

/**
 * 討論輪卡片檢視（spec「討論輪以卡片呈現」）：scaffold 記錄逐輪成卡——卡頭輪次
 * 徽章＋mode chip＋日期、卡身欄位標籤區塊；非標準格式整篇單一 markdown 檢視退回。
 * DiscussionDetailPage 討論過程分頁與 ArchivedList 討論檢視共用。
 */
export function RoundsView({ text, empty }: { text: string; empty?: string }) {
  const { t } = useI18n();
  const rounds = splitRounds(text);
  if (rounds === null) return <Markdown content={text} empty={empty} />;
  if (rounds.length === 0) return <Markdown content="" empty={empty} />;
  return (
    <div className="flex flex-col gap-3">
      {rounds.map((r, i) => (
        <section key={i} data-round={r.round} className="rounded-lg border border-border bg-card p-4">
          <header className="flex items-center gap-2">
            {/* 輪次籤是靜態編號，與同列 mode／日期同層級——走中性，不佔用主色。 */}
            <span data-doc-heading className="rounded-full bg-muted px-2 py-0.5 text-xs font-semibold text-muted-foreground">
              {t("rounds.roundN").replace("{n}", String(r.round))}
            </span>
            <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
              {r.mode}
            </span>
            <span className="text-xs text-muted-foreground tabular-nums">{r.date}</span>
          </header>
          {r.lead && (
            <div className="mt-3">
              <Markdown content={r.lead} />
            </div>
          )}
          {ROUND_LABELS.filter((l) => r.fields[l] !== undefined).map((l) => (
            <div key={l} className="mt-3">
              <div className={`${LABEL_CLS} mb-1`}>{t(ROUND_LABEL_KEYS[l])}</div>
              <Markdown content={r.fields[l]!} />
            </div>
          ))}
        </section>
      ))}
    </div>
  );
}

/** 結論欄位白名單——conclude scaffold 固定六詞（design D7）。 */
type ConclusionLabel =
  | "Decision"
  | "Rationale"
  | "Rejected alternatives"
  | "Deferred"
  | "Capture to"
  | "Next";
const CONCLUSION_LABELS: readonly ConclusionLabel[] = [
  "Decision",
  "Rationale",
  "Rejected alternatives",
  "Deferred",
  "Capture to",
  "Next",
];
const CONCLUSION_LABEL_KEYS: Record<ConclusionLabel, string> = {
  Decision: "conclusion.decision",
  Rationale: "conclusion.rationale",
  "Rejected alternatives": "conclusion.rejected",
  Deferred: "conclusion.deferred",
  "Capture to": "conclusion.captureTo",
  Next: "conclusion.next",
};

/**
 * 討論結論欄位檢視（spec「討論結論以欄位標籤呈現」）：conclude scaffold 六欄位
 * 拆標籤區塊；無任何白名單欄位（手寫自由格式）整篇單一 markdown 檢視退回。
 * DiscussionDetailPage 結論分頁與 ArchivedList 結論區共用。
 */
export function ConclusionView({ text, empty }: { text: string; empty?: string }) {
  const { t } = useI18n();
  const { lead, fields } = splitLabeledFields(text.split(/\r?\n/), CONCLUSION_LABELS);
  const present = CONCLUSION_LABELS.filter((l) => fields[l] !== undefined);
  if (present.length === 0) return <Markdown content={text} empty={empty} />;
  return (
    <div>
      {lead && (
        <div className="mb-3">
          <Markdown content={lead} />
        </div>
      )}
      {present.map((l) => (
        <div key={l} className="mt-4 first:mt-0">
          <div data-doc-heading className={`${LABEL_CLS} mb-1`}>
            {t(CONCLUSION_LABEL_KEYS[l])}
          </div>
          <Markdown content={fields[l]!} />
        </div>
      ))}
    </div>
  );
}

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

/** 背景的 `Prior discussions:` 行列出的先前討論數；缺行或 `none` 開頭（`None.`、`none — …`）為 null。
 * 項目以逗號（含全形）或頓號分隔，括號內的備註先拿掉——記錄是人寫的，備註裡的逗號不算一筆。 */
export function countPriorDiscussions(context: string): number | null {
  const line = context.split(/\r?\n/).find((l) => l.trimStart().startsWith("Prior discussions:"));
  if (!line) return null;
  const list = line
    .trimStart()
    .slice("Prior discussions:".length)
    .replace(/[（(][^）)]*[）)]/g, "")
    .trim();
  if (!list || /^none(?![\w-])/i.test(list)) return null;
  return list.split(/[,，、]/).filter((item) => item.trim()).length;
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

/** 「衍生變更」分頁的一列：等寬變更名、狀態標示，可開啟時附「開啟」鈕。討論詳情頁與已封存討論頁共用。 */
export function PromotedChangeRow({ name, badge, onOpen }: PromotedChangeProps) {
  const { t } = useI18n();
  return (
    <div data-promoted-row={name} className="flex items-center gap-2 rounded-md border border-border/60 px-2.5 py-1.5">
      <span className="min-w-0 flex-1 truncate font-mono text-sm font-medium">{name}</span>
      {badge}
      {onOpen && (
        <Button variant="outline" size="sm" className="h-6 shrink-0 gap-1 px-2 text-xs" onClick={onOpen}>
          <ArrowUpRight className="h-3 w-3" /> {t("ddrawer.openCard")}
        </Button>
      )}
    </div>
  );
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

  const sections = doc ? splitDiscussionSections(doc) : null;
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

  const promotePane = (
    <div className="flex h-full flex-col gap-2">
      {discussion.promotedTo.length === 0 ? (
        <p className="py-4 text-sm text-muted-foreground">{t("ddrawer.notPromoted")}</p>
      ) : (
        <div className="flex flex-col gap-1.5">
          {discussion.promotedTo.map((name) => (
            <PromotedChangeRow key={name} name={name} badge={stageBadge(name)} onOpen={openChild(name)} />
          ))}
        </div>
      )}
    </div>
  );

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
            />
            {/* 數字磚與提示框的容器：本刀不放內容（cut 2／3 由文件結構推導）。 */}
            <DocTiles tiles={[]} />
            <DocBody ref={bodyRef}>
              {sections ? (
                <>
                  <TabsContent value="conclusion">
                    <ConclusionView text={sections.conclusion} empty={t("ddrawer.noConclusion")} />
                  </TabsContent>
                  <TabsContent value="rounds">
                    <RoundsView text={sections.rounds} empty={t("ddrawer.noRounds")} />
                  </TabsContent>
                  <TabsContent value="context">
                    <Markdown content={sections.context} empty={t("ddrawer.noContext")} />
                  </TabsContent>
                </>
              ) : (
                // 區段缺失或格式非預期：整篇以單一檢視退回（不報錯）；載入中畫骨架、空態文案只在載入完成後出。
                <TabsContent value="record">
                  {doc === undefined ? <DocSkeleton /> : <Markdown content={doc} empty={t("common.noContent")} />}
                </TabsContent>
              )}
              <TabsContent value="promote">{promotePane}</TabsContent>
            </DocBody>
          </Card>
        </DetailShell>
      </Tabs>
    </TooltipProvider>
  );
}
