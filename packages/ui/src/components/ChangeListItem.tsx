import { useEffect, useState } from "react";
import { ChevronDown, ChevronRight, GripVertical, Archive } from "lucide-react";

import type { ChangeItem, Verb } from "../adapter";
import { parseTasks } from "../tasks";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Markdown } from "./Markdown";

export interface ChangeListItemProps {
  change: ChangeItem;
  expanded: boolean;
  onToggle: (name: string) => void;
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
  loadDocument,
  loadCapabilities,
  onRunVerb,
}: ChangeListItemProps) {
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [caps, setCaps] = useState<string[]>([]);
  const [specs, setSpecs] = useState<Record<string, string | null>>({});

  useEffect(() => {
    if (!expanded) return;
    if (proposal === undefined) void loadDocument("proposal.md").then(setProposal);
    if (design === undefined) void loadDocument("design.md").then(setDesign);
    if (tasksMd === undefined) void loadDocument("tasks.md").then(setTasksMd);
    if (caps.length === 0) void loadCapabilities().then(setCaps);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [expanded]);

  useEffect(() => {
    for (const cap of caps) {
      if (!(cap in specs)) {
        void loadDocument(`specs/${cap}/spec.md`).then((c) => setSpecs((s) => ({ ...s, [cap]: c })));
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [caps]);

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
            <Archive className="h-3.5 w-3.5" /> 封存
          </Button>
        </div>
      </div>

      {expanded && (
        <div className="border-t border-border px-3 pb-3">
          <div className="flex items-center gap-2 py-2 text-xs text-muted-foreground">
            <Badge variant="secondary">{taskCount} 任務</Badge>
            <span>·</span>
            <button className="hover:text-foreground" onClick={() => onRunVerb?.("analyze", change.name)}>分析</button>
            <button className="hover:text-foreground" onClick={() => onRunVerb?.("validate", change.name)}>驗證</button>
          </div>

          <Tabs defaultValue="proposal">
            <TabsList>
              <TabsTrigger value="proposal">提案</TabsTrigger>
              <TabsTrigger value="design">設計</TabsTrigger>
              <TabsTrigger value="tasks">任務 <span className="text-xs opacity-70">{taskCount}</span></TabsTrigger>
              <TabsTrigger value="specs">規格 {caps.length > 0 && <span className="text-xs opacity-70">{caps.length}</span>}</TabsTrigger>
            </TabsList>
            <div className="pt-3 max-h-[52vh] overflow-y-auto">
              <TabsContent value="proposal">
                <Markdown content={proposal ?? null} empty="載入中…" />
              </TabsContent>
              <TabsContent value="design">
                <Markdown content={design ?? null} empty="（此 change 無設計文件）" />
              </TabsContent>
              <TabsContent value="tasks">
                <Markdown content={tasksMd ?? null} empty="載入中…" />
              </TabsContent>
              <TabsContent value="specs">
                {caps.length === 0 ? (
                  <div className="text-muted-foreground text-sm py-6">（此 change 無 delta 規格）</div>
                ) : (
                  caps.map((cap) => (
                    <div key={cap} className="mb-4">
                      <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">{cap}</div>
                      <Markdown content={specs[cap] ?? null} empty="載入中…" />
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
