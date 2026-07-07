import { useEffect, useRef, useState } from "react";
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
  /** 刷新世代——遞增即重載已載入的文件內容（未傳＝0，行為等同僅開啟時載入）。 */
  refreshGen?: number;
  loadDocument: (change: string, artifact: string) => Promise<string | null>;
  loadCapabilities: (change: string) => Promise<string[]>;
  loadMeta: (change: string) => Promise<ChangeMetaInfo | null>;
  onRunVerb?: (verb: Verb, change: string) => void;
  onDelete?: (change: string) => void;
  /** 勾選/取消任務並回寫 tasks.md；重載由宿主 refresh 後的刷新世代遞增驅動（單一資料流）。 */
  onToggleTask?: (change: string, ordinal: number, done: boolean) => Promise<void>;
  /** 移動任務順序並回寫 tasks.md（before 為可選側別）；重載由宿主 refresh 後的刷新世代遞增驅動。 */
  onMoveTask?: (change: string, from: number, to: number, before?: boolean) => Promise<void>;
  /** 來源討論（change.fromDiscussion 解析出的 slug＋topic）；null/缺席＝非討論而來。 */
  sourceDiscussion?: { slug: string; topic: string } | null;
  /** 同一討論扇出的同源 change 名（不含此 change 自己）。 */
  siblingChanges?: string[];
  /** 點來源討論開討論抽屜。 */
  onOpenDiscussion?: (slug: string) => void;
  /** 點同源 change 互跳。 */
  onOpenSibling?: (name: string) => void;
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
  refreshGen,
  loadDocument,
  loadCapabilities,
  loadMeta,
  onRunVerb,
  onDelete,
  onToggleTask,
  onMoveTask,
  sourceDiscussion,
  siblingChanges,
  onOpenDiscussion,
  onOpenSibling,
}: RichDetailDrawerProps) {
  const [meta, setMeta] = useState<ChangeMetaInfo | null>(null);
  const [proposal, setProposal] = useState<Doc>();
  const [design, setDesign] = useState<Doc>();
  const [tasksMd, setTasksMd] = useState<Doc>();
  const [specDocs, setSpecDocs] = useState<Record<string, string | null>>({});
  const [copied, setCopied] = useState(false);
  const [full, setFull] = useState(false);
  const [taskBusy, setTaskBusy] = useState(false);
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
      setSpecDocs({});
    }
    const fresh = <T,>(apply: (v: T) => void) => (v: T) => {
      if (requestSeq.current === seq) apply(v);
    };
    void loadMeta(target).then(fresh(setMeta));
    void loadDocument(target, "proposal.md").then(fresh(setProposal));
    void loadDocument(target, "design.md").then(fresh(setDesign));
    void loadDocument(target, "tasks.md").then(fresh(setTasksMd));
    void loadCapabilities(target).then(async (caps) => {
      const entries = await Promise.all(
        caps.map(async (cap) => [cap, await loadDocument(target, `specs/${cap}/spec.md`)] as const),
      );
      if (requestSeq.current === seq) setSpecDocs(Object.fromEntries(entries));
    });
  };

  // 開啟／換 change：清空後全量載入（載入中狀態屬新內容的正確呈現）。
  useEffect(() => {
    if (!open || !name) return;
    loadAll(gen, name, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, name]);

  // 外部世代重載：不清空、回應到達後單次替換（不重置分頁與捲動）。
  // 互動進行中（拖曳手勢 dragActive、寫回等待 taskBusy）讓路——兩旗標皆為依賴，
  // 互動結束時本 effect 重跑補載一次。
  useEffect(() => {
    if (!open || !name || taskBusy || dragActive) return;
    if (gen <= loadedGen.current) return;
    loadAll(gen, name, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, name, gen, taskBusy, dragActive]);

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

  // 勾選／拖放僅轉發寫回；重載統一由宿主 refresh 的世代遞增驅動（design D2 單一資料流）。
  // busy 旗標讓外部世代在互動中讓路，finally 釋放時世代 effect 補載。
  const handleToggle = async (ordinal: number, done: boolean) => {
    if (!name || !onToggleTask) return;
    setTaskBusy(true);
    try {
      await onToggleTask(name, ordinal, done);
    } finally {
      setTaskBusy(false);
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
          {/* 同源連結：來源討論＋兄弟刀（design D6，fromDiscussion 帶出）。 */}
          {sourceDiscussion && (
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground flex-wrap">
              <span>來自討論：</span>
              <button
                type="button"
                className="rounded-full bg-primary/10 px-2 py-0.5 font-medium text-primary hover:bg-primary/20"
                onClick={() => onOpenDiscussion?.(sourceDiscussion.slug)}
              >
                {sourceDiscussion.topic}
              </button>
              {(siblingChanges ?? []).length > 0 && (
                <>
                  <span>同源：</span>
                  {(siblingChanges ?? []).map((sib) => (
                    <button
                      key={sib}
                      type="button"
                      className="rounded-full bg-muted px-2 py-0.5 font-medium hover:bg-accent"
                      onClick={() => onOpenSibling?.(sib)}
                    >
                      {sib}
                    </button>
                  ))}
                </>
              )}
            </div>
          )}
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
                onDragActiveChange={setDragActive}
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
