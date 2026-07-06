import { useEffect, useState } from "react";
import {
  Archive,
  Check,
  Code2,
  Copy,
  FileText,
  ListChecks,
  Maximize2,
  Minimize2,
  PenTool,
  ShieldCheck,
  Sparkles,
  Trash2,
} from "lucide-react";

import type { ChangeItem, ChangeMetaInfo, Verb } from "../adapter";
import { specDeltaCounts, sumDeltaCounts } from "../delta";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { Markdown } from "./Markdown";
import { TaskList } from "./TaskList";
import { DeltaBadges } from "./DeltaBadges";

export interface RichDetailDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  change: ChangeItem | null;
  loadDocument: (change: string, artifact: string) => Promise<string | null>;
  loadCapabilities: (change: string) => Promise<string[]>;
  loadMeta: (change: string) => Promise<ChangeMetaInfo | null>;
  onRunVerb?: (verb: Verb, change: string) => void;
  onDelete?: (change: string) => void;
  /** 勾選/取消任務並回寫 tasks.md；resolve 後抽屜會重載任務。 */
  onToggleTask?: (change: string, ordinal: number, done: boolean) => Promise<void>;
  /** 移動任務順序並回寫 tasks.md（before 為可選側別）；resolve 後抽屜會重載任務。 */
  onMoveTask?: (change: string, from: number, to: number, before?: boolean) => Promise<void>;
}

type Doc = string | null | undefined;

/** 相對時間（天級即可；meta.created 通常是 YYYY-MM-DD）。 */
function relativeDays(created?: string | null): string | null {
  if (!created) return null;
  const t = Date.parse(created);
  if (Number.isNaN(t)) return created;
  const days = Math.floor((Date.now() - t) / 86_400_000);
  if (days <= 0) return "今天";
  if (days === 1) return "昨天";
  return `${days} 天前`;
}

/** Spectra 級詳情抽屜：metadata、進度、動作列、icon 分頁（提案/設計/互動任務/彩色規格）。 */
export function RichDetailDrawer({
  open,
  onOpenChange,
  change,
  loadDocument,
  loadCapabilities,
  loadMeta,
  onRunVerb,
  onDelete,
  onToggleTask,
  onMoveTask,
}: RichDetailDrawerProps) {
  const [meta, setMeta] = useState<ChangeMetaInfo | null>(null);
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [specDocs, setSpecDocs] = useState<Record<string, string | null>>({});
  const [copied, setCopied] = useState(false);
  const [full, setFull] = useState(false);
  const [taskBusy, setTaskBusy] = useState(false);

  const name = change?.name ?? null;

  useEffect(() => {
    if (!open || !name) return;
    setMeta(null);
    setProposal(undefined);
    setDesign(undefined);
    setTasksMd(undefined);
    setSpecDocs({});
    void loadMeta(name).then(setMeta);
    void loadDocument(name, "proposal.md").then(setProposal);
    void loadDocument(name, "design.md").then(setDesign);
    void loadDocument(name, "tasks.md").then(setTasksMd);
    void loadCapabilities(name).then(async (caps) => {
      const entries = await Promise.all(
        caps.map(async (cap) => [cap, await loadDocument(name, `specs/${cap}/spec.md`)] as const),
      );
      setSpecDocs(Object.fromEntries(entries));
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, name]);

  if (!change) return null;

  const pct = change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const taskBadge = `${change.completedTasks}/${change.totalTasks}`;
  const delta = sumDeltaCounts(Object.values(specDocs).map(specDeltaCounts));
  const rel = relativeDays(meta?.created);

  const copyName = () => {
    void navigator.clipboard?.writeText(change.name);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const reloadTasks = async () => {
    if (name) setTasksMd(await loadDocument(name, "tasks.md"));
  };

  const handleToggle = async (ordinal: number, done: boolean) => {
    if (!name || !onToggleTask) return;
    setTaskBusy(true);
    try {
      await onToggleTask(name, ordinal, done);
      await reloadTasks();
    } finally {
      setTaskBusy(false);
    }
  };

  // 拖放落點一次到位轉發（含側別）；寫回後重讀 tasks.md——重編號後文字已變，檔案為真相。
  const handleReorder = async (from: number, to: number, before?: boolean) => {
    if (!name || !onMoveTask) return;
    setTaskBusy(true);
    try {
      await onMoveTask(name, from, to, before);
      await reloadTasks();
    } finally {
      setTaskBusy(false);
    }
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        className={full ? "w-[96vw] max-w-none" : "w-[max(720px,42vw)] max-w-[95vw]"}
      >
        <SheetHeader>
          <div className="flex items-center gap-2 pr-14">
            <SheetTitle className="truncate">{change.name}</SheetTitle>
            <button
              type="button"
              aria-label="複製名稱"
              className="text-muted-foreground hover:text-foreground"
              onClick={copyName}
            >
              {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            </button>
            <div className="flex-1" />
            <button
              type="button"
              aria-label={full ? "還原大小" : "全螢幕"}
              className="text-muted-foreground hover:text-foreground"
              onClick={() => setFull((f) => !f)}
            >
              {full ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
            </button>
          </div>
          {/* metadata 列 */}
          <div className="flex items-center gap-2 text-xs text-muted-foreground flex-wrap">
            {meta?.createdBy && (
              <span className="inline-flex items-center gap-1">
                <span className="inline-flex h-4 w-4 items-center justify-center rounded-full bg-primary text-primary-foreground text-[9px] font-bold">
                  {meta.createdBy.charAt(0).toUpperCase()}
                </span>
                {meta.createdBy}
              </span>
            )}
            {meta?.createdWith && <span>✳ {meta.createdWith}</span>}
            {rel && <span>{rel}</span>}
            <span>{taskBadge} 任務</span>
            {meta?.startedAt && (
              <span className="inline-flex items-center gap-1">
                ⚒ {meta.startedBy ? `${meta.startedBy} · ` : ""}
                {meta.startedAt} 開工
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <div className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden">
              <div className="h-full rounded-full bg-primary" style={{ width: `${pct}%` }} />
            </div>
            <span className="text-xs text-muted-foreground tabular-nums">{pct}%</span>
          </div>
          {/* 動作列 */}
          <div className="flex items-center gap-1.5 pt-1">
            <Button variant="outline" size="sm" className="h-7 gap-1" onClick={() => onRunVerb?.("analyze", change.name)}>
              <Sparkles className="h-3.5 w-3.5" /> 分析
            </Button>
            <Button variant="outline" size="sm" className="h-7 gap-1" onClick={() => onRunVerb?.("validate", change.name)}>
              <ShieldCheck className="h-3.5 w-3.5" /> 驗證
            </Button>
            <div className="flex-1" />
            <Button variant="outline" size="sm" className="h-7 gap-1" onClick={() => onRunVerb?.("archive", change.name)}>
              <Archive className="h-3.5 w-3.5" /> 封存
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-7 gap-1 text-destructive hover:text-destructive"
              onClick={() => onDelete?.(change.name)}
            >
              <Trash2 className="h-3.5 w-3.5" /> 刪除
            </Button>
          </div>
        </SheetHeader>

        <Tabs defaultValue="proposal" className="flex-1 min-h-0 flex flex-col">
          <TabsList>
            <TabsTrigger value="proposal">
              <FileText className="h-3.5 w-3.5" /> 提案
            </TabsTrigger>
            <TabsTrigger value="design">
              <PenTool className="h-3.5 w-3.5" /> 設計
            </TabsTrigger>
            <TabsTrigger value="tasks">
              <ListChecks className="h-3.5 w-3.5" /> 任務
              <Badge variant={pct === 100 ? "default" : "secondary"} className="ml-1">{taskBadge}</Badge>
            </TabsTrigger>
            <TabsTrigger value="specs">
              <Code2 className="h-3.5 w-3.5" /> 規格
              <span className="ml-1"><DeltaBadges counts={delta} /></span>
            </TabsTrigger>
          </TabsList>
          <div className="flex-1 overflow-y-auto pt-3">
            <TabsContent value="proposal"><Markdown content={proposal ?? null} empty="載入中…" /></TabsContent>
            <TabsContent value="design"><Markdown content={design ?? null} empty="（此 change 無設計文件）" /></TabsContent>
            <TabsContent value="tasks">
              <TaskList
                markdown={tasksMd ?? null}
                busy={taskBusy}
                onToggle={(ordinal, done) => void handleToggle(ordinal, done)}
                onReorder={(from, to, before) => void handleReorder(from, to, before)}
              />
            </TabsContent>
            <TabsContent value="specs">
              {Object.keys(specDocs).length === 0 ? (
                <div className="text-muted-foreground text-sm py-6">（此 change 無 delta 規格）</div>
              ) : (
                Object.entries(specDocs).map(([cap, doc]) => (
                  <div key={cap} className="mb-4">
                    <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-2">
                      {cap} <DeltaBadges counts={specDeltaCounts(doc)} />
                    </div>
                    <Markdown content={doc} empty="載入中…" />
                  </div>
                ))
              )}
            </TabsContent>
          </div>
        </Tabs>
      </SheetContent>
    </Sheet>
  );
}
