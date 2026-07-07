import { useEffect, useRef, useState } from "react";
import { ChevronDown, ChevronRight, GripVertical, Archive } from "lucide-react";

import type { ChangeItem, Verb } from "../adapter";
import { useI18n } from "../i18n";
import { parseTasks } from "../tasks";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Markdown } from "./Markdown";

export interface ChangeListItemProps {
  change: ChangeItem;
  expanded: boolean;
  onToggle: (name: string) => void;
  /** 刷新世代——遞增且展開中即重載已載入的文件（未傳＝0，行為等同僅展開時載入）。 */
  refreshGen?: number;
  /** 讀取此 change 的 artifact（proposal.md/design.md/tasks.md/specs/<cap>/spec.md）。 */
  loadDocument: (artifact: string) => Promise<string | null>;
  loadCapabilities: () => Promise<string[]>;
  onRunVerb?: (verb: Verb, change: string) => void;
}

type Doc = string | null | undefined; // undefined = 尚未載入

export function ChangeListItem({
  change,
  expanded,
  onToggle,
  refreshGen,
  loadDocument,
  loadCapabilities,
  onRunVerb,
}: ChangeListItemProps) {
  const { t } = useI18n();
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [caps, setCaps] = useState<string[]>([]);
  const [specs, setSpecs] = useState<Record<string, string | null>>({});

  const gen = refreshGen ?? 0;
  // latest-wins：回應帶發起序號，落後即丟棄。
  const requestSeq = useRef(0);
  const loadedGen = useRef(-1);

  const loadAll = (g: number) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    const fresh = <T,>(apply: (v: T) => void) => (v: T) => {
      if (requestSeq.current === seq) apply(v);
    };
    void loadDocument("proposal.md").then(fresh(setProposal));
    void loadDocument("design.md").then(fresh(setDesign));
    void loadDocument("tasks.md").then(fresh(setTasksMd));
    void loadCapabilities().then(async (cs) => {
      if (requestSeq.current !== seq) return;
      setCaps(cs);
      const entries = await Promise.all(
        cs.map(async (cap) => [cap, await loadDocument(`specs/${cap}/spec.md`)] as const),
      );
      if (requestSeq.current === seq) setSpecs(Object.fromEntries(entries));
    });
  };

  // 展開即抓取——不做一次性快取，收合再展開以檔案現況重讀。
  useEffect(() => {
    if (!expanded) return;
    loadAll(gen);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [expanded]);

  // 展開中外部世代遞增 → 就地重載（不清空、回應到達後替換）。
  useEffect(() => {
    if (!expanded) return;
    if (gen <= loadedGen.current) return;
    loadAll(gen);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [expanded, gen]);

  const pct = change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const taskCount = `${change.completedTasks}/${change.totalTasks}`;

  return (
    <div className="rounded-lg border border-border bg-card">
      {/* 卡片標頭 */}
      <div className="flex items-center gap-2 p-3">
        <GripVertical className="h-4 w-4 text-muted-foreground/40 shrink-0" />
        <button
          type="button"
          className="flex items-center gap-1.5 min-w-0 flex-1 text-left"
          onClick={() => onToggle(change.name)}
        >
          {expanded ? (
            <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
          )}
          <span className="font-semibold text-sm truncate">{change.name}</span>
        </button>
        <div className="flex items-center gap-2 shrink-0">
          <div className="w-28 h-1.5 rounded-full bg-muted overflow-hidden">
            <div className="h-full rounded-full bg-primary" style={{ width: `${pct}%` }} />
          </div>
          <span className="text-xs text-muted-foreground tabular-nums w-9 text-right">{pct}%</span>
          <Button variant="outline" size="sm" className="h-7 gap-1" onClick={() => onRunVerb?.("archive", change.name)}>
            <Archive className="h-3.5 w-3.5" /> {t("common.archive")}
          </Button>
        </div>
      </div>

      {expanded && (
        <div className="border-t border-border px-3 pb-3">
          <div className="flex items-center gap-2 py-2 text-xs text-muted-foreground">
            <Badge variant="secondary">{t("common.tasksCount").replace("{n}", taskCount)}</Badge>
            <span>·</span>
            <button className="hover:text-foreground" onClick={() => onRunVerb?.("analyze", change.name)}>{t("common.analyze")}</button>
            <button className="hover:text-foreground" onClick={() => onRunVerb?.("validate", change.name)}>{t("common.validate")}</button>
          </div>

          <Tabs defaultValue="proposal">
            <TabsList>
              <TabsTrigger value="proposal">{t("common.tabProposal")}</TabsTrigger>
              <TabsTrigger value="design">{t("common.tabDesign")}</TabsTrigger>
              <TabsTrigger value="tasks">{t("common.tabTasks")} <span className="text-xs opacity-70">{taskCount}</span></TabsTrigger>
              <TabsTrigger value="specs">{t("common.tabSpecs")} {caps.length > 0 && <span className="text-xs opacity-70">{caps.length}</span>}</TabsTrigger>
            </TabsList>
            <div className="pt-3 max-h-[52vh] overflow-y-auto">
              <TabsContent value="proposal">
                <Markdown content={proposal ?? null} empty={t("common.loading")} />
              </TabsContent>
              <TabsContent value="design">
                <Markdown content={design ?? null} empty={t("list.noDesignDoc")} />
              </TabsContent>
              <TabsContent value="tasks">
                <Markdown content={tasksMd ?? null} empty={t("common.loading")} />
              </TabsContent>
              <TabsContent value="specs">
                {caps.length === 0 ? (
                  <div className="text-muted-foreground text-sm py-6">{t("list.noDeltaSpecs")}</div>
                ) : (
                  caps.map((cap) => (
                    <div key={cap} className="mb-4">
                      <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">{cap}</div>
                      <Markdown content={specs[cap] ?? null} empty={t("common.loading")} />
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
