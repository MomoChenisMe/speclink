import { useState } from "react";
import { Check, ChevronDown, ChevronRight, Code2, Copy, FileText, ListChecks, PenTool } from "lucide-react";

import type { ArchivedItem, DiscussionItem } from "../adapter";
import { Badge } from "./ui/badge";
import { Input } from "./ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Markdown } from "./Markdown";
import { TaskList } from "./TaskList";
import { splitDiscussionSections } from "./DiscussionDrawer";

type Doc = string | null | undefined;

/** 展開列懶載入的文件載入器（dated name 定址；封存目錄為真相）。 */
export interface ArchivedLoaders {
  loadDocument: (datedName: string, artifact: string) => Promise<string | null>;
  loadCapabilities: (datedName: string) => Promise<string[]>;
}

/** Spectra 式封存列：日期＋名稱＋任務數徽章＋複製；點擊展開唯讀分頁檢視。 */
export function ArchivedRow({ item, loaders }: { item: ArchivedItem; loaders: ArchivedLoaders }) {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [specDocs, setSpecDocs] = useState<Record<string, string | null>>({});

  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(item.datedName);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const toggle = () => {
    const next = !expanded;
    setExpanded(next);
    // 內容懶載入：首次展開才讀封存文件，收合再展開不重讀。
    if (next && !loaded) {
      setLoaded(true);
      const { loadDocument, loadCapabilities } = loaders;
      void loadDocument(item.datedName, "proposal.md").then(setProposal);
      void loadDocument(item.datedName, "design.md").then(setDesign);
      void loadDocument(item.datedName, "tasks.md").then(setTasksMd);
      void loadCapabilities(item.datedName).then(async (caps) => {
        const entries = await Promise.all(
          caps.map(async (cap) => [cap, await loadDocument(item.datedName, `specs/${cap}/spec.md`)] as const),
        );
        setSpecDocs(Object.fromEntries(entries));
      });
    }
  };

  const badge =
    item.tasksTotal != null && item.tasksDone != null ? `${item.tasksDone}/${item.tasksTotal}` : null;
  const specCount = Object.keys(specDocs).length;

  return (
    <div className="rounded-lg border border-border bg-card">
      <button
        type="button"
        className="group flex items-center gap-2.5 w-full p-3 text-left"
        aria-expanded={expanded}
        onClick={toggle}
      >
        {expanded ? (
          <ChevronDown className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        )}
        <span className="text-xs text-muted-foreground tabular-nums shrink-0">{item.date}</span>
        <span className="font-medium text-sm truncate flex-1">{item.name}</span>
        {badge && (
          <Badge variant="secondary" className="shrink-0 tabular-nums">
            {badge}
          </Badge>
        )}
        <span
          role="button"
          aria-label="複製封存名稱"
          className={`shrink-0 text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
          onClick={copy}
        >
          {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
        </span>
      </button>
      {expanded && (
        <div className="px-3 pb-3 border-t border-border pt-3">
          <Tabs defaultValue="proposal" className="flex flex-col">
            <TabsList>
              <TabsTrigger value="proposal">
                <FileText className="h-3.5 w-3.5" /> 提案
              </TabsTrigger>
              <TabsTrigger value="design">
                <PenTool className="h-3.5 w-3.5" /> 設計
              </TabsTrigger>
              <TabsTrigger value="tasks">
                <ListChecks className="h-3.5 w-3.5" /> 任務
                {badge && <Badge variant="secondary" className="ml-1">{badge}</Badge>}
              </TabsTrigger>
              <TabsTrigger value="specs">
                <Code2 className="h-3.5 w-3.5" /> 規格{specCount > 0 ? `＋${specCount}` : ""}
              </TabsTrigger>
            </TabsList>
            <div className="pt-3 max-h-[50vh] overflow-y-auto">
              <TabsContent value="proposal">
                <Markdown content={proposal ?? null} empty="（無提案文件）" />
              </TabsContent>
              <TabsContent value="design">
                <Markdown content={design ?? null} empty="（此變更無設計文件）" />
              </TabsContent>
              <TabsContent value="tasks">
                {/* 封存檢視唯讀：不接 onToggle/onMove，核取方塊 disabled。 */}
                <TaskList markdown={tasksMd ?? null} readOnly />
              </TabsContent>
              <TabsContent value="specs">
                {specCount === 0 ? (
                  <div className="text-muted-foreground text-sm py-6">（此變更無 delta 規格）</div>
                ) : (
                  Object.entries(specDocs).map(([cap, doc]) => (
                    <div key={cap} className="mb-4">
                      <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">
                        {cap}
                      </div>
                      <Markdown content={doc} empty="（無內容）" />
                    </div>
                  ))
                )}
              </TabsContent>
            </div>
          </Tabs>
        </div>
      )}
    </div>
  );
}

/** 封存討論列：日期＋topic，展開為唯讀記錄檢視（區段切分渲染，無任何寫入動詞）。 */
function ArchivedDiscussionRow({
  item,
  loadDocument,
}: {
  item: DiscussionItem;
  loadDocument: (slug: string) => Promise<string | null>;
}) {
  const [expanded, setExpanded] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [doc, setDoc] = useState<Doc>();

  const toggle = () => {
    const next = !expanded;
    setExpanded(next);
    if (next && !loaded) {
      setLoaded(true);
      void loadDocument(item.slug).then(setDoc);
    }
  };

  const sections = doc ? splitDiscussionSections(doc) : null;

  return (
    <div className="rounded-lg border border-border bg-card">
      <button
        type="button"
        className="flex items-center gap-2.5 w-full p-3 text-left"
        aria-expanded={expanded}
        onClick={toggle}
      >
        {expanded ? (
          <ChevronDown className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        )}
        <span className="text-xs text-muted-foreground tabular-nums shrink-0">{item.created}</span>
        <span className="font-medium text-sm truncate flex-1">{item.topic}</span>
        <span className="text-xs text-muted-foreground tabular-nums shrink-0">{item.rounds} 輪</span>
      </button>
      {expanded && (
        <div className="px-3 pb-3 border-t border-border pt-3 max-h-[50vh] overflow-y-auto">
          {sections ? (
            <>
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">背景</h3>
              <Markdown content={sections.context} empty="（無背景）" />
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1 mt-4">討論過程</h3>
              <Markdown content={sections.rounds} empty="（無討論過程）" />
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1 mt-4">結論</h3>
              <Markdown content={sections.conclusion} empty="（無結論）" />
            </>
          ) : (
            // 區段缺失或格式非預期：整篇單一檢視退回。
            <Markdown content={doc ?? null} empty="載入中…" />
          )}
        </div>
      )}
    </div>
  );
}

export interface ArchivedListProps extends ArchivedLoaders {
  archived: ArchivedItem[];
  query: string;
  onQuery: (q: string) => void;
  /** 封存討論（討論節；缺席時不顯示該節，向後相容）。 */
  archivedDiscussions?: DiscussionItem[];
  /** 討論記錄全文載入（slug 定址；提供 archivedDiscussions 時必填）。 */
  loadDiscussionDocument?: (slug: string) => Promise<string | null>;
}

/** 已封存獨立頁（design D7 雙節）：搜尋同時過濾「變更」與「討論」兩節。 */
export function ArchivedList({
  archived,
  query,
  onQuery,
  loadDocument,
  loadCapabilities,
  archivedDiscussions,
  loadDiscussionDocument,
}: ArchivedListProps) {
  const q = query.trim().toLowerCase();
  const filtered = archived.filter((a) => a.name.toLowerCase().includes(q));
  const discussions = (archivedDiscussions ?? []).filter(
    (d) => d.topic.toLowerCase().includes(q) || d.slug.toLowerCase().includes(q),
  );
  const showDiscussions = archivedDiscussions !== undefined && loadDiscussionDocument !== undefined;
  return (
    <div className="flex flex-col gap-3 max-w-3xl mx-auto w-full">
      <Input placeholder="搜尋已封存的變更與討論…" value={query} onChange={(e) => onQuery(e.target.value)} />
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">已封存的變更</h2>
        <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
          {filtered.length}
        </span>
      </div>
      <div className="flex flex-col gap-2.5">
        {filtered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">沒有已封存的變更</div>
        ) : (
          filtered.map((a) => (
            <ArchivedRow key={a.datedName} item={a} loaders={{ loadDocument, loadCapabilities }} />
          ))
        )}
      </div>
      {showDiscussions && (
        <>
          <div className="flex items-center gap-2 pt-2">
            <h2 className="text-base font-semibold">已封存的討論</h2>
            <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
              {discussions.length}
            </span>
          </div>
          <div className="flex flex-col gap-2.5">
            {discussions.length === 0 ? (
              <div className="text-muted-foreground text-sm py-8 text-center">沒有已封存的討論</div>
            ) : (
              discussions.map((d) => (
                <ArchivedDiscussionRow key={d.slug} item={d} loadDocument={loadDiscussionDocument} />
              ))
            )}
          </div>
        </>
      )}
    </div>
  );
}
