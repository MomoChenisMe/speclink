import { useRef, useState, type ReactNode } from "react";
import { FileText, History } from "lucide-react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { parseSpecSections } from "../specSections";
import { parseTraceSources } from "../trace";
import { settle, useGenerationLoad } from "../lib/useGenerationLoad";
import { Badge } from "./ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import { DetailContentCard, DetailHeader, DetailPageFrame, type DocumentTarget } from "./DetailHeader";
import { ListRow } from "./ListRow";
import { ManualToc } from "./ManualToc";
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
  /** 內容卡標頭右端的文件動作（「開啟 ▾」）：兩個分頁都對應同一份 spec.md。 */
  renderDocumentActions?: (target: DocumentTarget | null) => ReactNode;
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
 * 規格詳情頁（spec「桌面 app 呈現 change 與 spec 的清單與內容」；design D2）：標題列（capability
 * ＋複製、右端搜尋需求）與 meta 列；內容卡分頁「規格 N」（Purpose 與逐條需求段）與「溯源 M」
 * （來源變更列）。頁寬 ≥1200px 時 768px 內容欄左側空白處以 sticky 列出需求目錄。換目標以 key
 * 重掛（清空全量載入、搜尋字串與分頁回到初始），世代重載不清空且 latest-wins。
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
}: SpecDetailPageProps & { capability: string }) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<Doc>();
  const [query, setQuery] = useState("");
  const [tab, setTab] = useState("spec");
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
  // 最近更新：各需求段尾 @trace 中最大的 updated 與其 source。
  let latest: { source: string; updated: string } | null = null;
  for (const r of sections?.requirements ?? []) {
    const u = r.trace?.updated;
    if (u && (!latest || u > latest.updated)) latest = { source: r.trace!.source, updated: u };
  }
  const needle = query.trim().toLowerCase();
  const visible = (sections?.requirements ?? []).filter((r) => r.title.toLowerCase().includes(needle));

  return (
    <div data-spec-detail className="@container relative w-full">
      {/* 需求目錄（design D2）：量的是頁寬——768px 內容欄左側空白處，sticky 隨頁層捲動高亮。 */}
      {tab === "spec" && sections && (
        <div
          data-spec-toc
          className="absolute inset-y-0 left-0 hidden w-[calc((100%_-_768px)_/_2)] @min-[1200px]:block"
        >
          <div className="sticky top-0 flex max-h-[calc(100vh_-_3rem)] justify-end">
            <ManualToc
              bodyRef={bodyRef}
              markdownBody={`${doc}\u0000${needle}`}
              headings="[data-spec-heading]"
              className="flex border-l-0 border-r pl-5 pr-3"
            />
          </div>
        </div>
      )}
      <DetailPageFrame>
        <DetailHeader
          title={capability}
          copyLabel={t("common.copyName")}
          actions={
            <SearchField
              value={query}
              onChange={setQuery}
              placeholder={t("sdetail.searchRequirements")}
            />
          }
          provenance={[
            sections && (
              <span className="shrink-0">
                {t("sdetail.metaRequirements").replace("{n}", String(sections.requirements.length))}
              </span>
            ),
            sources.length > 0 && (
              <span className="shrink-0">{t("sdetail.metaSources").replace("{n}", String(sources.length))}</span>
            ),
            latest && (
              <span className="truncate">
                {t("sdetail.metaUpdated").replace("{date}", latest.updated).replace("{change}", latest.source)}
              </span>
            ),
          ]}
        />

        <Tabs value={tab} onValueChange={setTab}>
          <TabsList
            variant="card"
            actions={renderDocumentActions?.({ doc: { spec: capability }, content: doc ?? null })}
          >
            <TabsTrigger value="spec">
              <FileText className="h-3.5 w-3.5" /> {t("sdetail.tabSpec")}
              {loaded && (
                <Badge variant="secondary" className="ml-1">
                  {sections?.requirements.length ?? 0}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value="trace">
              <History className="h-3.5 w-3.5" /> {t("sdetail.tabTrace")}
              {loaded && (
                <Badge variant="secondary" className="ml-1">
                  {sources.length}
                </Badge>
              )}
            </TabsTrigger>
          </TabsList>
          <DetailContentCard>
            {/* 載入中畫骨架；空態文案只在載入完成後出（spec「詳情頁文件載入以 skeleton 呈現」）。 */}
            <TabsContent value="spec">
              {!loaded ? (
                <DocSkeleton />
              ) : !sections ? (
                // 沒有需求標題（或文件缺席）：整篇單一 markdown 檢視退回，無目錄。
                <Markdown content={doc} empty={t("common.noContent")} />
              ) : (
                <div
                  ref={bodyRef}
                  data-spec-body
                  className="flex flex-col gap-6 [&_.markdown]:text-[15px] [&_.markdown]:leading-[1.75]"
                >
                  <section data-purpose>
                    <h2 data-spec-heading className="mb-2 text-base font-semibold">
                      Purpose
                    </h2>
                    <Markdown content={sections.purpose} />
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
                    visible.map((r, i) => (
                      <RequirementBlock
                        key={`${i}:${r.title}`}
                        req={r}
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
            </TabsContent>
            <TabsContent value="trace">
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
            </TabsContent>
          </DetailContentCard>
        </Tabs>
      </DetailPageFrame>
    </div>
  );
}
