import { useEffect, useRef, useState, type ReactNode } from "react";
import { FileText, History } from "lucide-react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { parseSpecSections } from "../specSections";
import { calendarDay, specStats, specTiles, traceTiles } from "../specSummary";
import { parseTraceSources } from "../trace";
import { settle, useGenerationLoad } from "../lib/useGenerationLoad";
import { Badge, badgeVariants } from "./ui/badge";
import { Callout } from "./ui/callout";
import { Card } from "./ui/card";
import { StatusPill } from "./ui/status-pill";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import { StatusRow, type DocumentRef, type DocumentTarget } from "./DetailHeader";
import { DetailIdentity, DetailShell, DetailStatus, DetailTreeLabel, DocBody, DocHeader, DocTiles } from "./DetailShell";
import { InlineMarks, Lead, SECTION_HEADING_CLS } from "./DocParts";
import { ListRow } from "./ListRow";
import { Markdown } from "./Markdown";
import { RequirementBlock } from "./RequirementBlock";
import { SearchField } from "./SearchField";
import { DocSkeleton } from "./skeletons";

export interface SpecDetailPageProps {
  /** 目標 capability id；null＝未選定（不渲染）。 */
  capability: string | null;
  /** 刷新世代——遞增即就地重載（不清空、latest-wins；未傳＝0）。 */
  refreshGen?: number;
  /** 讀取正式規格 spec.md 全文（capability 定址）；缺件回 null。 */
  loadDocument: (capability: string) => Promise<string | null>;
  /** 封存變更清單：把 @trace 來源變更名對應到封存目錄名與封存日期；缺席／空＝溯源全部不可點。 */
  archivedChanges?: ArchivedItem[];
  /** 點可點的溯源列或需求錨點：宿主開啟該封存變更的已封存詳情頁（帶 datedName）。 */
  onOpenArchivedChange?: (datedName: string) => void;
  /** 「規格」閱讀卡標題列右端的文件動作（「開啟 ▾」，對應 spec.md）；溯源文件不對應檔案、不放。 */
  renderDocumentActions?: (target: DocumentTarget | null) => ReactNode;
  /** spec.md 的實際相對路徑（宿主向 core 查詢，跟著專案的 spec 目錄名走）：出處行首段與 Purpose 待補
   * 提示框引用它；缺席或查無時出處行不列路徑、提示框改用不帶路徑的說法。 */
  locateDocument?: (doc: DocumentRef) => Promise<{ relative: string } | null>;
}

type Doc = string | null | undefined;

/** 封存清單以變更名索引（同名取清單中的第一筆）。 */
function archivedByName(archived: ArchivedItem[]): Map<string, ArchivedItem> {
  const byName = new Map<string, ArchivedItem>();
  for (const item of archived) {
    if (!byName.has(item.name)) byName.set(item.name, item);
  }
  return byName;
}

/** 溯源列（design D2）：全文 @trace 來源去重；封存清單命中者依封存日期升冪（同日依文件
 * 首次出現序）在前，未命中者依文件出現序在後。 */
function traceItems(doc: Doc, byName: Map<string, ArchivedItem>) {
  const items = parseTraceSources(doc).map((name, index) => ({ name, index, archived: byName.get(name) }));
  const hits = items
    .filter((i) => i.archived)
    .sort((a, b) => a.archived!.date.localeCompare(b.archived!.date) || a.index - b.index);
  return [...hits, ...items.filter((i) => !i.archived)];
}

/** 需求段標題列右端的來源錨點：封存清單命中時可點開該封存變更。 */
function TraceAnchor({
  trace,
  byName,
  onOpenArchivedChange,
}: {
  trace: { source: string; updated: string | null } | null;
  byName: Map<string, ArchivedItem>;
  onOpenArchivedChange?: (datedName: string) => void;
}) {
  const { t } = useI18n();
  if (!trace) return null;
  const hit = byName.get(trace.source);
  const anchor = [`${t("sdrawer.fromChanges")} ${trace.source}`, trace.updated].filter(Boolean).join(" · ");
  return hit ? (
    <button
      type="button"
      data-trace-anchor
      className="shrink-0 text-[11px] text-muted-foreground hover:text-foreground hover:underline"
      onClick={() => onOpenArchivedChange?.(hit.datedName)}
    >
      {anchor}
    </button>
  ) : (
    <span data-trace-anchor className="shrink-0 text-[11px] text-muted-foreground">
      {anchor}
    </span>
  );
}

/**
 * 規格詳情頁（spec「桌面 app 呈現 change 與 spec 的清單與內容」；desktop-spec-manual-pages design
 * D1–D5）：詳情頁三欄骨架——左欄身分區、「文件」節點規格／溯源與「狀態」三列；「規格」閱讀卡為
 * 標題區（需求數籤、搜尋需求、「開啟 ▾」、出處行）、數字磚、提示框、Purpose 引言段與需求段（命中的
 * 第一條展開、其餘收合）；「溯源」閱讀卡為出處行、數字磚與來源變更列；右欄本頁目錄自渲染後的 DOM
 * 推導。換目標以 key 重掛（清空全量載入、搜尋字串與文件回到初始），世代重載不清空且 latest-wins。
 */
export function SpecDetailPage(props: SpecDetailPageProps) {
  if (!props.capability) return null;
  return <SpecDetailView key={props.capability} {...props} capability={props.capability} />;
}

function SpecDetailView({
  capability,
  refreshGen,
  loadDocument,
  archivedChanges,
  onOpenArchivedChange,
  renderDocumentActions,
  locateDocument,
}: SpecDetailPageProps & { capability: string }) {
  const { t, tn } = useI18n();
  const [doc, setDoc] = useState<Doc>();
  const [query, setQuery] = useState("");
  const [tab, setTab] = useState("spec");
  const [docPath, setDocPath] = useState<string | null>(null);
  // 需求段每次展開／收合都遞增：進 tocKey 讓右欄目錄重掃，展開後才渲染的內文子標題補進目錄。
  const [toggles, setToggles] = useState(0);
  const bodyRef = useRef<HTMLDivElement>(null);

  // 開啟全量載入；外部世代重載不清空、回應到達後單次替換（不重置捲動與搜尋）。
  useGenerationLoad(refreshGen ?? 0, (isLatest) => {
    void loadDocument(capability)
      .then((v) => {
        if (isLatest()) setDoc(v);
      })
      .catch(settle(setDoc, null, isLatest));
  });

  const loaded = doc !== undefined;
  const sections = doc ? parseSpecSections(doc) : null;
  const byName = archivedByName(archivedChanges ?? []);
  const sources = traceItems(doc, byName);
  const stats = specStats(sections, sources.length);
  const needle = query.trim().toLowerCase();
  const visible = (sections?.requirements ?? []).filter((r) => r.title.toLowerCase().includes(needle));
  const updatedLine =
    stats.latest &&
    t("sdetail.metaUpdated").replace("{date}", calendarDay(stats.latest.updated)).replace("{change}", stats.latest.source);
  const count = (n: number) =>
    loaded && (
      <Badge variant="secondary" className="tabular-nums">
        {n}
      </Badge>
    );
  const showTrace = () => setTab("trace");

  // 換目標以 key 重掛，掛載時向 core 查一次 spec.md 的位置；卸載作廢在途回應。
  const locateRef = useRef(locateDocument);
  locateRef.current = locateDocument;
  useEffect(() => {
    let live = true;
    void locateRef.current?.({ spec: capability }).then(
      (found) => {
        if (live && found) setDocPath(found.relative);
      },
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, [capability]);

  const tree = (
    <>
      <DetailIdentity
        title={capability}
        copyLabel={t("common.copyName")}
        badges={<StatusPill tone="neutral">{t("sdetail.canonical")}</StatusPill>}
        provenance={[
          sections && (
            <span className="shrink-0">{tn("sdetail.metaRequirements", stats.requirements)}</span>
          ),
          sections && (
            <span className="shrink-0">{tn("sdetail.metaScenarios", stats.scenarios)}</span>
          ),
          stats.latest && (
            <span className="truncate">{`${t("sdetail.stateUpdated")} ${calendarDay(stats.latest.updated)}`}</span>
          ),
        ]}
        relations={
          sources.length > 0
            ? [
                {
                  mark: "trace",
                  label: t("sdetail.relationTrace"),
                  children: (
                    <>
                      <button
                        type="button"
                        className={cn(badgeVariants({ variant: "outline" }), "min-w-0 max-w-full cursor-(--cursor-interaction) font-mono hover:bg-muted")}
                        onClick={showTrace}
                      >
                        <span className="truncate">{sources[0].name}</span>
                      </button>
                      {sources.length > 1 && (
                        <button
                          type="button"
                          className={cn(badgeVariants({ variant: "secondary" }), "cursor-(--cursor-interaction) tabular-nums hover:bg-accent")}
                          onClick={showTrace}
                        >
                          {`+${sources.length - 1}`}
                        </button>
                      )}
                    </>
                  ),
                },
              ]
            : []
        }
      />

      <DetailTreeLabel>{t("detail.tree")}</DetailTreeLabel>
      <TabsList variant="tree" aria-label={t("detail.tree")}>
        <TabsTrigger value="spec">
          <FileText className="h-3.5 w-3.5" /> {t("sdetail.tabSpec")}
          {loaded && (
            <Badge variant="secondary" className="ml-auto">
              {stats.requirements}
            </Badge>
          )}
        </TabsTrigger>
        <TabsTrigger value="trace">
          <History className="h-3.5 w-3.5" /> {t("sdetail.tabTrace")}
          {loaded && (
            <Badge variant="secondary" className="ml-auto">
              {sources.length}
            </Badge>
          )}
        </TabsTrigger>
      </TabsList>

      <DetailTreeLabel>{t("detail.status")}</DetailTreeLabel>
      <DetailStatus>
        <StatusRow mark="requirements" label={t("sdetail.stateRequirements")}>
          {loaded && <span className="font-mono text-[11px] tabular-nums">{stats.requirements}</span>}
        </StatusRow>
        <StatusRow mark="sources" label={t("sdetail.stateSources")}>
          {loaded && <span className="font-mono text-[11px] tabular-nums">{sources.length}</span>}
        </StatusRow>
        {stats.latest && (
          <StatusRow mark="updated" label={t("sdetail.stateUpdated")}>
            <span className="font-mono text-[11px] tabular-nums">{calendarDay(stats.latest.updated)}</span>
            <span className="truncate text-muted-foreground">{stats.latest.source}</span>
          </StatusRow>
        )}
      </DetailStatus>
    </>
  );

  // 提示框只放推得出的事實（design D4）：Purpose 待補在前、缺來源變更的需求在後；兩者皆無時缺席。
  const callouts = (stats.purposeTbd || stats.untraced.length > 0) && (
    <div data-doc-callout className="flex flex-col gap-2.5">
      {stats.purposeTbd && (
        <Callout data-callout="purpose-tbd" tone="warning" title={t("specs.purposeTbd")} className="my-0">
          <p className="text-sm">
            <InlineMarks
              text={docPath ? t("sdetail.purposeTbdBody").replace("{path}", docPath) : t("sdetail.purposeTbdBodyNoPath")}
            />
          </p>
        </Callout>
      )}
      {stats.untraced.length > 0 && (
        <Callout
          data-callout="untraced"
          tone="note"
          title={tn("sdetail.untraced", stats.untraced.length)}
          className="my-0"
        >
          <p className="text-sm">{stats.untraced.join("、")}</p>
          <p className="text-sm text-muted-foreground">{t("sdetail.untracedBody")}</p>
        </Callout>
      )}
    </div>
  );

  return (
    <Tabs value={tab} onValueChange={setTab} orientation="vertical" activationMode="manual" className="h-full">
      <DetailShell
        data-spec-detail
        tree={tree}
        activeDoc={tab}
        tocKey={`${tab}\u0000${doc ?? ""}\u0000${needle}\u0000${toggles}`}
        bodyRef={bodyRef}
      >
        <TabsContent value="spec">
          <Card className="flex flex-col gap-5 p-6">
            <DocHeader
              title={t("sdetail.tabSpec")}
              badges={count(stats.requirements)}
              actions={
                <div className="flex items-center gap-1.5">
                  <SearchField value={query} onChange={setQuery} placeholder={t("sdetail.searchRequirements")} />
                  {renderDocumentActions?.({ doc: { spec: capability }, content: doc ?? null })}
                </div>
              }
              provenance={[docPath && <span className="font-mono">{docPath}</span>, t("sdetail.canonical"), updatedLine]}
            />
            <DocTiles tiles={specTiles(stats, t)} />
            {callouts}
            {/* 沒有需求標題（或文件缺席）時整篇單一 markdown 檢視退回：不接 bodyRef，本頁目錄無錨點。 */}
            <DocBody ref={sections ? bodyRef : undefined}>
              {/* 載入中畫骨架；空態文案只在載入完成後出（spec「詳情頁文件載入以 skeleton 呈現」）。 */}
              {!loaded ? (
                <DocSkeleton />
              ) : !sections ? (
                <Markdown content={doc} empty={t("common.noContent")} />
              ) : (
                <div className="flex flex-col gap-6">
                  <section data-purpose className="flex flex-col gap-2.5">
                    <h2 data-doc-heading className={SECTION_HEADING_CLS}>
                      Purpose
                    </h2>
                    <Lead>
                      <Markdown content={sections.purpose} />
                    </Lead>
                  </section>
                  {/* 需求段之外的內容（引言、其他 `## ` 區段）照原文呈現，不丟。 */}
                  {sections.intro && (
                    <section data-spec-intro>
                      <Markdown content={sections.intro} />
                    </section>
                  )}
                  {visible.length === 0 ? (
                    <p className="py-2 text-sm text-muted-foreground">{t("sdetail.noMatch")}</p>
                  ) : (
                    // 收合規則依命中序：搜尋過濾後第一條命中展開（key 含命中序，序一變即重掛回預設）。
                    visible.map((r, i) => (
                      <RequirementBlock
                        key={`${i}:${r.title}`}
                        req={r}
                        collapse={{ defaultOpen: i === 0, onToggle: () => setToggles((n) => n + 1) }}
                        anchor={
                          <TraceAnchor trace={r.trace} byName={byName} onOpenArchivedChange={onOpenArchivedChange} />
                        }
                      />
                    ))
                  )}
                  {sections.appendix && (
                    <section data-spec-appendix>
                      <Markdown content={sections.appendix} />
                    </section>
                  )}
                </div>
              )}
            </DocBody>
          </Card>
        </TabsContent>
        <TabsContent value="trace">
          <Card className="flex flex-col gap-5 p-6">
            {/* 推導文件不對應檔案：無「開啟 ▾」。 */}
            <DocHeader
              title={t("sdetail.tabTrace")}
              badges={count(sources.length)}
              provenance={[t("sdetail.traceDerived"), t("sdetail.traceOrder")]}
            />
            <DocTiles tiles={traceTiles(stats, t)} />
            <DocBody>
              {!loaded ? (
                <DocSkeleton />
              ) : sources.length === 0 ? (
                <p className="py-6 text-sm text-muted-foreground">{t("sdetail.noSources")}</p>
              ) : (
                sources.map((s) => (
                  <ListRow
                    key={s.name}
                    data-trace-row={s.name}
                    title={s.name}
                    description={s.archived ? s.archived.date : t("sdetail.noArchive")}
                    disabled={!s.archived}
                    onClick={() => s.archived && onOpenArchivedChange?.(s.archived.datedName)}
                  />
                ))
              )}
            </DocBody>
          </Card>
        </TabsContent>
      </DetailShell>
    </Tabs>
  );
}
