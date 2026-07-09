import { useEffect, useRef, useState } from "react";
import { ArrowUpRight, FileText, Flag, MessagesSquare, Rocket } from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionItem } from "../adapter";
import { useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { Markdown } from "./Markdown";
import { LABEL_CLS } from "./SectionedDoc";
import { discussionChipStage } from "./DiscussionColumn";

/** 討論記錄的三個標準區段。 */
export interface DiscussionSections {
  context: string;
  rounds: string;
  conclusion: string;
}

/**
 * 把討論記錄全文按 `## Context`／`## Rounds`／`## Conclusion` 切分（design D5）。
 * 任一區段缺失（手寫、pre-scaffold 格式）回 `null`——呼叫端整篇以單一檢視退回。
 */
export function splitDiscussionSections(text: string): DiscussionSections | null {
  const lines = text.split(/\r?\n/);
  const body = (name: string): string | null => {
    const start = lines.findIndex((l) => l.trimEnd() === `## ${name}`);
    if (start < 0) return null;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i++) {
      if (lines[i].startsWith("## ") && !lines[i].startsWith("###")) {
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

  for (const line of text.split(/\r?\n/)) {
    if (line.startsWith("### ")) {
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
 * DiscussionDrawer 討論過程分頁與 ArchivedList 討論檢視共用。
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
            <span className="rounded-full bg-primary/12 px-2 py-0.5 text-xs font-semibold text-primary">
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
 * DiscussionDrawer 結論分頁與 ArchivedList 結論區共用。
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
          <div className={`${LABEL_CLS} mb-1`}>{t(CONCLUSION_LABEL_KEYS[l])}</div>
          <Markdown content={fields[l]!} />
        </div>
      ))}
    </div>
  );
}

export interface DiscussionDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  discussion: DiscussionItem | null;
  /** 刷新世代——遞增即重載討論記錄內容（未傳＝0，行為等同僅開啟時載入）。 */
  refreshGen?: number;
  /** 記錄全文載入（slug 定址）。 */
  loadDocument: (slug: string) => Promise<string | null>;
  /** active change 清單（衍生變更分頁現況派生）。 */
  changes: ChangeItem[];
  /** 已封存 change 清單（衍生變更分頁已封存態派生）。 */
  archivedChanges: ArchivedItem[];
  /** 衍生變更分頁「開啟卡片」跳轉。 */
  onOpenChangeCard?: (name: string) => void;
}

/**
 * 討論抽屜：結論／討論過程／背景／衍生變更四分頁（結論非空時預設開結論），
 * 標題下方生命週期階梯標示現站（design D3）；衍生變更分頁唯讀——列各子變更
 * 現況與跳轉，無轉出動作（promote 已自 GUI 撤除，轉出改由 CLI／agent）。
 * 記錄格式非預期時整篇單一檢視退回。GUI 不提供 conclude 等寫入。
 */
export function DiscussionDrawer({
  open,
  onOpenChange,
  discussion,
  refreshGen,
  loadDocument,
  changes,
  archivedChanges,
  onOpenChangeCard,
}: DiscussionDrawerProps) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<string | null | undefined>();
  const slug = discussion?.slug ?? null;
  const gen = refreshGen ?? 0;
  // latest-wins：回應帶發起序號，落後即丟棄（涵蓋世代與換討論的交錯）。
  const requestSeq = useRef(0);
  const loadedGen = useRef(-1);

  const loadRecord = (g: number, target: string, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) setDoc(undefined);
    void loadDocument(target).then((v) => {
      if (requestSeq.current === seq) setDoc(v);
    });
  };

  // 開啟／換討論：清空後載入。
  useEffect(() => {
    if (!open || !slug) return;
    loadRecord(gen, slug, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, slug]);

  // 外部世代重載（add-round／conclude／轉出後）：不清空、回應到達後就地替換，分頁選擇不重置。
  useEffect(() => {
    if (!open || !slug) return;
    if (gen <= loadedGen.current) return;
    loadRecord(gen, slug, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, slug, gen]);

  if (!discussion) return null;

  const sections = doc ? splitDiscussionSections(doc) : null;

  const promotePane = (
    <div className="flex h-full flex-col gap-2">
      {discussion.promotedTo.length === 0 ? (
        <p className="text-sm text-muted-foreground py-4">{t("ddrawer.notPromoted")}</p>
      ) : (
        <div className="flex flex-col gap-1.5">
          {discussion.promotedTo.map((name) => {
            const stage = t(discussionChipStage(name, changes, archivedChanges));
            const alive = changes.some((c) => c.name === name);
            return (
              <div
                key={name}
                data-promoted-row={name}
                className="flex items-center gap-2 rounded-md border border-border/60 px-2.5 py-1.5"
              >
                <span className="text-sm font-medium min-w-0 flex-1 truncate">{name}</span>
                <span className="shrink-0 rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
                  {stage}
                </span>
                {alive && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-6 gap-1 px-2 text-xs shrink-0"
                    onClick={() => onOpenChangeCard?.(name)}
                  >
                    <ArrowUpRight className="h-3 w-3" /> {t("ddrawer.openCard")}
                  </Button>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );

  // 預設分頁＝讀者第一想看的：結論非空（去除鷹架註解後有內容）開結論，否則背景。
  const conclusionHasContent =
    !!sections && sections.conclusion.replace(/<!--[\s\S]*?-->/g, "").trim().length > 0;
  const defaultTab = conclusionHasContent ? "conclusion" : "context";

  // 生命週期階梯：現站由 status 決定，封存不入階梯（封存後只在已封存頁）。
  const stationIndex = discussion.status === "promoted" ? 2 : discussion.status === "concluded" ? 1 : 0;
  const stations = [
    t("discussion.statusOpen"),
    t("discussion.statusConcluded"),
    t("ddrawer.stationPromoted"),
  ];

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-[max(720px,42vw)] max-w-[95vw]">
        <SheetHeader>
          <div className="flex items-center gap-2 pr-14">
            <SheetTitle className="truncate">{discussion.topic}</SheetTitle>
          </div>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span className="tabular-nums">{t("common.rounds").replace("{n}", String(discussion.rounds))}</span>
            {discussion.created && <span>{discussion.created}</span>}
            {discussion.createdBy && (
              <span className="inline-flex items-center gap-1">
                <span className="inline-flex h-4 w-4 items-center justify-center rounded-full bg-primary text-[9px] font-bold text-primary-foreground">
                  {discussion.createdBy.charAt(0).toUpperCase()}
                </span>
                {discussion.createdBy}
              </span>
            )}
          </div>
          {/* 生命週期階梯：狀態模型可見，免使用者腦補（design D3）。 */}
          <div className="flex items-center gap-1.5 text-xs">
            {stations.map((label, i) => (
              <span key={label} className="flex items-center gap-1.5">
                {i > 0 && <span className="text-muted-foreground/50">→</span>}
                <span
                  aria-current={i === stationIndex ? "step" : undefined}
                  className={
                    i === stationIndex
                      ? "inline-flex items-center gap-1 rounded-full bg-primary/12 px-2 py-0.5 font-semibold text-primary"
                      : i < stationIndex
                        ? "inline-flex items-center gap-1 text-muted-foreground"
                        : "inline-flex items-center gap-1 text-muted-foreground/50"
                  }
                >
                  <span aria-hidden="true">{i < stationIndex ? "✓" : i === stationIndex ? "●" : "○"}</span>
                  <span>{label}</span>
                </span>
              </span>
            ))}
          </div>
        </SheetHeader>

        {sections ? (
          // key 區分分頁組與預設分頁——否則自載入中 fallback 切換過來時，Radix
          // Tabs 的未受控 state 停留在不存在的分頁值，內容全數隱藏。
          <Tabs
            key={`sections-${defaultTab}`}
            defaultValue={defaultTab}
            className="flex-1 min-h-0 flex flex-col"
          >
            <TabsList>
              <TabsTrigger value="conclusion">
                <Flag className="h-3.5 w-3.5" /> {t("ddrawer.tabConclusion")}
              </TabsTrigger>
              <TabsTrigger value="rounds">
                <MessagesSquare className="h-3.5 w-3.5" /> {t("ddrawer.tabRounds")} {discussion.rounds}
              </TabsTrigger>
              <TabsTrigger value="context">
                <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabContext")}
              </TabsTrigger>
              <TabsTrigger value="promote">
                <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")}
              </TabsTrigger>
            </TabsList>
            <div className="flex-1 overflow-y-auto pt-3">
              <TabsContent value="conclusion"><ConclusionView text={sections.conclusion} empty={t("ddrawer.noConclusion")} /></TabsContent>
              <TabsContent value="rounds"><RoundsView text={sections.rounds} empty={t("ddrawer.noRounds")} /></TabsContent>
              <TabsContent value="context"><Markdown content={sections.context} empty={t("ddrawer.noContext")} /></TabsContent>
              <TabsContent value="promote">{promotePane}</TabsContent>
            </div>
          </Tabs>
        ) : (
          <Tabs key="fallback" defaultValue="record" className="flex-1 min-h-0 flex flex-col">
            <TabsList>
              <TabsTrigger value="record">
                <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabRecord")}
              </TabsTrigger>
              <TabsTrigger value="promote">
                <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")}
              </TabsTrigger>
            </TabsList>
            <div className="flex-1 overflow-y-auto pt-3">
              {/* 區段缺失或格式非預期：整篇以單一檢視退回（不報錯）。 */}
              <TabsContent value="record"><Markdown content={doc ?? null} empty={t("common.loading")} /></TabsContent>
              <TabsContent value="promote">{promotePane}</TabsContent>
            </div>
          </Tabs>
        )}
      </SheetContent>
    </Sheet>
  );
}
