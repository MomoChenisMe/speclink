import { useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { Route } from "lucide-react";

import type { ChangeItem, DiscussionLists, UnpromotedCutItem } from "../adapter";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import {
  roadmapEdgeKey,
  roadmapModel,
  roadmapRoute,
  type RoadmapEdge,
  type RoadmapStation,
} from "../roadmapModel";
import { planBlockedLabel, planWaveLabel, STAGE_BADGE } from "../stage";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { CommandChip } from "./DocParts";
import { EmptyState } from "./EmptyState";
import { ColumnLoadFailed, ColumnSkeleton } from "./skeletons";
import { PageHeader } from "./ui/page-header";
import { StatusPill } from "./ui/status-pill";
import { Tabs, TabsList, TabsTrigger } from "./ui/tabs";

/** 看板頁的兩種檢視（spec desktop-roadmap「看板頁的路線圖檢視」）：卡片看板與路線圖。 */
export type BoardMode = "cards" | "roadmap";

/** 看板頁頁首的「看板｜路線圖」切換：卡片檢視與路線圖的頁首放同一個。 */
export function BoardViewSwitch({ value, onChange }: { value: BoardMode; onChange: (mode: BoardMode) => void }) {
  const { t } = useI18n();
  return (
    <Tabs value={value} onValueChange={(mode) => onChange(mode as BoardMode)}>
      <TabsList variant="segmented" aria-label={t("roadmap.viewSwitch")}>
        <TabsTrigger value="cards">{t("roadmap.view.cards")}</TabsTrigger>
        <TabsTrigger value="roadmap">{t("roadmap.view.roadmap")}</TabsTrigger>
      </TabsList>
    </Tabs>
  );
}

export interface RoadmapViewProps {
  /** 頁標題與說明：與卡片檢視同一頁，宿主給同一個標題。 */
  title?: string;
  description?: string;
  /** 清單項（plan 的配置序）；帶 wave 的成為站。 */
  changes: ChangeItem[];
  /** plan 的 next：同名的站帶「下一個」籤。 */
  next: string | null;
  unpromotedCuts: UnpromotedCutItem[];
  /** 非 null（依賴成環）時不畫波次欄、站與線，改顯示成環提示。 */
  planError: string | null;
  /** 目前專案的討論清單：尚未轉出卡的討論 slug 在清單內時可點。 */
  discussions: DiscussionLists;
  onOpenChange?: (name: string) => void;
  onOpenDiscussion?: (slug: string) => void;
  /** 頁首右側的「看板｜路線圖」切換（宿主給 BoardViewSwitch）。 */
  viewSwitch?: ReactNode;
  /** 與卡片檢視相同的載入狀態：首訪載入中畫佔位卡、首訪失敗畫失敗提示。 */
  loading?: boolean;
  loadFailed?: boolean;
}

/** 進行中站的任務進度：SVG 圓弧，滿圈＝全部任務。 */
function ProgressRing({ done, total }: { done: number; total: number }) {
  const r = 6;
  const circumference = 2 * Math.PI * r;
  const ratio = total > 0 ? done / total : 0;
  return (
    <svg data-progress-ring viewBox="0 0 16 16" aria-hidden="true" className="h-3.5 w-3.5 -rotate-90">
      <circle cx="8" cy="8" r={r} fill="none" strokeWidth="2.5" className="stroke-stage-in-progress/20" />
      <circle
        cx="8"
        cy="8"
        r={r}
        fill="none"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeDasharray={circumference}
        strokeDashoffset={circumference * (1 - ratio)}
        className="stroke-stage-in-progress"
      />
    </svg>
  );
}

/** 站的次行：已就緒實心、進行中圓圈加 n/m、提案中外框 n/m。 */
function StationStatus({ station }: { station: RoadmapStation }) {
  const { t } = useI18n();
  const count = `${station.completedTasks}/${station.totalTasks}`;
  if (station.stage === "ready") {
    return (
      <span data-station-status className={`rounded px-1.5 py-0.5 font-medium ${STAGE_BADGE.ready}`}>
        {t("stage.ready")}
      </span>
    );
  }
  if (station.stage === "in-progress") {
    return (
      <span data-station-status className="inline-flex items-center gap-1.5 tabular-nums text-stage-in-progress">
        <ProgressRing done={station.completedTasks} total={station.totalTasks} />
        {count}
      </span>
    );
  }
  return (
    <span
      data-station-status
      className="rounded border border-stage-proposed/40 px-1.5 py-0.5 tabular-nums text-stage-proposed"
    >
      {count}
    </span>
  );
}

/** 站與路線高亮之間的接線：量測用的 DOM 登記與滑鼠／焦點進出。 */
interface StationWiring {
  register: (name: string, el: HTMLElement | null) => void;
  setHovered: (name: string | null) => void;
  setFocused: (name: string | null) => void;
}

function StationButton({
  station,
  dimmed,
  wiring,
  onOpen,
}: {
  station: RoadmapStation;
  dimmed: boolean;
  wiring: StationWiring;
  onOpen?: (name: string) => void;
}) {
  const { t } = useI18n();
  const blocked = station.blockedBy.length > 0;
  return (
    <button
      type="button"
      ref={(el) => wiring.register(station.name, el)}
      data-station={station.name}
      data-stage={station.stage}
      data-next={station.next ? "" : undefined}
      data-blocked={blocked ? "" : undefined}
      data-dimmed={dimmed ? "" : undefined}
      onClick={() => onOpen?.(station.name)}
      onMouseEnter={() => wiring.setHovered(station.name)}
      onMouseLeave={() => wiring.setHovered(null)}
      onFocus={() => wiring.setFocused(station.name)}
      onBlur={() => wiring.setFocused(null)}
      className={cn(
        "flex w-full flex-col gap-1.5 rounded-xl border bg-card px-3 py-2.5 text-left transition-opacity hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        station.next ? "border-primary ring-1 ring-primary" : "border-border",
        blocked && "opacity-55",
        dimmed && "opacity-25",
      )}
    >
      <span className="flex min-w-0 items-center gap-2">
        <span className="min-w-0 truncate font-mono text-[13px]">{station.name}</span>
        {station.next && (
          <StatusPill data-next-badge tone="primary" className="ml-auto">
            {t("roadmap.next")}
          </StatusPill>
        )}
      </span>
      <span className="flex items-center text-xs">
        <StationStatus station={station} />
      </span>
      {blocked && (
        <span data-station-blocked className="text-xs text-muted-foreground">
          {planBlockedLabel(station.blockedBy, t)}
        </span>
      )}
    </button>
  );
}

function CutCard({
  cut,
  canOpen,
  onOpenDiscussion,
}: {
  cut: UnpromotedCutItem;
  canOpen: boolean;
  onOpenDiscussion?: (slug: string) => void;
}) {
  const { t } = useI18n();
  return (
    <article
      data-roadmap-cut
      className={`flex flex-col gap-1.5 rounded-xl border border-dashed px-3 py-2.5 ${SEMANTIC_SURFACE.manual}`}
    >
      <header data-cut-head className="flex min-w-0 items-center gap-2 text-xs">
        <span className="min-w-0 truncate font-mono">
          {canOpen ? (
            <button
              type="button"
              className="text-primary underline-offset-2 hover:underline"
              onClick={() => onOpenDiscussion?.(cut.discussion)}
            >
              {cut.discussion}
            </button>
          ) : (
            cut.discussion
          )}
          {` · ${cut.cut}`}
        </span>
        <StatusPill tone="manual" className="ml-auto">
          {t("conclusion.notPromoted")}
        </StatusPill>
      </header>
      <span data-cut-unpromoted className="font-mono text-[13px]">
        {cut.unpromoted.join(t("common.listSeparator"))}
      </span>
      {cut.title && (
        <p data-cut-title className="text-xs text-muted-foreground">
          {cut.title}
        </p>
      )}
      <span className="mt-0.5">
        <CommandChip command={`/speclink-propose --from-discussion ${cut.discussion}`} />
      </span>
    </article>
  );
}

/** 站在畫布座標系內的左右緣與垂直中點。 */
interface StationBox {
  left: number;
  right: number;
  mid: number;
}

/** 依賴線疊層：不接收指標事件；起點為前置站右緣中點、終點為依賴者站左緣中點的三次貝茲曲線。 */
function EdgeLayer({
  edges,
  boxes,
  route,
}: {
  edges: RoadmapEdge[];
  boxes: Map<string, StationBox>;
  route: Set<string> | null;
}) {
  return (
    <svg aria-hidden="true" className="pointer-events-none absolute inset-0 h-full w-full overflow-visible">
      {edges.map((edge) => {
        const from = boxes.get(edge.from);
        const to = boxes.get(edge.to);
        if (!from || !to) return null;
        const key = roadmapEdgeKey(edge);
        const active = route !== null && route.has(key);
        const dimmed = route !== null && !active;
        const bend = (to.left - from.right) / 2;
        return (
          <path
            key={key}
            data-roadmap-edge={key}
            data-active={active ? "" : undefined}
            data-dimmed={dimmed ? "" : undefined}
            d={`M ${from.right} ${from.mid} C ${from.right + bend} ${from.mid}, ${to.left - bend} ${to.mid}, ${to.left} ${to.mid}`}
            fill="none"
            strokeWidth={active ? 2 : 1.5}
            className={cn("transition-opacity", active ? "stroke-primary" : "stroke-border", dimmed && "opacity-30")}
          />
        );
      })}
    </svg>
  );
}

const COLUMN_CLS = "flex w-60 shrink-0 flex-col gap-2";
const COLUMN_HEAD_CLS = "px-1 text-xs text-muted-foreground [&>:first-child]:font-medium [&>:first-child]:text-foreground";

/**
 * 看板頁的路線圖檢視（spec desktop-roadmap）：每一波一欄、每個變更一站，最右一欄列出尚未
 * 轉出的刀。只看不寫——點站開變更詳情、點討論 slug 開討論詳情，其餘都在詳情頁。
 */
export function RoadmapView({
  title,
  description,
  changes,
  next,
  unpromotedCuts,
  planError,
  discussions,
  onOpenChange,
  onOpenDiscussion,
  viewSwitch,
  loading,
  loadFailed,
}: RoadmapViewProps) {
  const { t, tn } = useI18n();
  const model = roadmapModel(changes, next);
  // 成環時引擎不排波次：清單項全部沒有 wave，不是 metadata 壞掉，不另寫無法排程。
  const waves = planError ? [] : model.waves;
  const unplaced = planError ? [] : model.unplaced;

  // 路線高亮：滑鼠停留優先、其次鍵盤焦點；兩者都離開才恢復。只改呈現。停留中的站因
  // 刷新離開路線圖時（另一個 session 封存了它）不再算數，免得整張圖一直變淡。
  const [hovered, setHovered] = useState<string | null>(null);
  const [focused, setFocused] = useState<string | null>(null);
  const onMap = new Set(waves.flatMap((w) => w.stations.map((s) => s.name)));
  const highlighted = [hovered, focused].find((name) => name !== null && onMap.has(name));
  const route = highlighted ? roadmapRoute(model, highlighted) : null;

  // 依賴線的座標：量測每站相對於畫布的位置。畫布隨捲動一起移動，捲動不需重算；
  // 資料變動與畫布尺寸改變（ResizeObserver）時重算。
  const canvasRef = useRef<HTMLDivElement>(null);
  const stationEls = useRef(new Map<string, HTMLElement>());
  const [boxes, setBoxes] = useState(() => new Map<string, StationBox>());
  const wiring: StationWiring = {
    register: (name, el) => {
      if (el) stationEls.current.set(name, el);
      else stationEls.current.delete(name);
    },
    setHovered,
    setFocused,
  };
  const layoutKey = JSON.stringify([waves.map((w) => w.stations.map((s) => s.name)), model.edges]);
  useLayoutEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const measure = () => {
      const base = canvas.getBoundingClientRect();
      const measured = new Map<string, StationBox>();
      for (const [name, el] of stationEls.current) {
        const rect = el.getBoundingClientRect();
        measured.set(name, {
          left: rect.left - base.left,
          right: rect.right - base.left,
          mid: rect.top - base.top + rect.height / 2,
        });
      }
      setBoxes(measured);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(canvas);
    return () => observer.disconnect();
    // 畫布只在載入結束後掛上：載入狀態一變就重新接上量測。
  }, [layoutKey, loading, loadFailed]);
  const openable = new Set(discussions.active.map((d) => d.slug));
  const failed = !loading && !!loadFailed;
  const empty = !planError && waves.length === 0 && unpromotedCuts.length === 0;

  let body: ReactNode;
  if (loading) {
    body = (
      <div className="flex gap-6">
        <section className={COLUMN_CLS}>
          <ColumnSkeleton />
        </section>
        <section className={COLUMN_CLS}>
          <ColumnSkeleton />
        </section>
      </div>
    );
  } else if (failed) {
    body = <ColumnLoadFailed />;
  } else if (empty) {
    body = <EmptyState icon={Route} title={t("roadmap.empty")} className="h-full" />;
  } else {
    body = (
      <div ref={canvasRef} className="relative min-w-max">
        {waves.length > 0 && (
          <EdgeLayer edges={model.edges} boxes={boxes} route={route ? route.edges : null} />
        )}
        <div data-roadmap-columns className="relative flex items-start gap-10 p-1">
          {waves.map((wave) => (
            <section key={wave.wave} data-roadmap-wave={wave.wave} className={COLUMN_CLS}>
              <header data-wave-head className={COLUMN_HEAD_CLS}>
                <span>{planWaveLabel(wave.wave, t)}</span>
                <span>{` · ${wave.stations.length}`}</span>
                <span>
                  {` · ${
                    wave.wave === 1 ? t("roadmap.parallel") : t("roadmap.waitsFor").replace("{n}", String(wave.wave - 1))
                  }`}
                </span>
              </header>
              {wave.stations.map((station) => (
                <StationButton
                  key={station.name}
                  station={station}
                  dimmed={route !== null && !route.stations.has(station.name)}
                  wiring={wiring}
                  onOpen={onOpenChange}
                />
              ))}
            </section>
          ))}
          {unpromotedCuts.length > 0 && (
            <section data-roadmap-cuts className={COLUMN_CLS}>
              <header data-cuts-head className={COLUMN_HEAD_CLS}>
                <span>{t("conclusion.notPromoted")}</span>
                <span>{` · ${unpromotedCuts.length}`}</span>
              </header>
              {unpromotedCuts.map((cut) => (
                <CutCard
                  key={`${cut.discussion}/${cut.cut}`}
                  cut={cut}
                  canOpen={openable.has(cut.discussion)}
                  onOpenDiscussion={onOpenDiscussion}
                />
              ))}
            </section>
          )}
        </div>
      </div>
    );
  }

  return (
    <div data-roadmap className="flex h-full min-h-0 flex-col gap-3">
      {planError && (
        <div
          role="note"
          className={`rounded-md border px-3 py-2 text-xs ${SEMANTIC_SURFACE.warning} ${SEMANTIC_TONE.warning}`}
        >
          {t("board.planCycle")}
          {planError}
        </div>
      )}
      {title ? <PageHeader title={title} description={description} actions={viewSwitch} /> : viewSwitch}
      {unplaced.length > 0 && (
        <p data-roadmap-unplaced className="text-xs text-muted-foreground">
          {tn("roadmap.unplaced", unplaced.length)}
        </p>
      )}
      <div className="relative min-h-0 flex-1 overflow-auto">{body}</div>
    </div>
  );
}
