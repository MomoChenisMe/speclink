import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Archive, GitBranch, FileText, StickyNote, Settings, FolderOpen } from "lucide-react";
import {
  KanbanBoard,
  ArchivedList,
  RichDetailDrawer,
  DiscussionDrawer,
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
  Input,
  type SpeclinkDataSource,
  type Verb,
} from "@speclink/ui";

import { createAppStore } from "./store";

export interface AppProps {
  dataSource: SpeclinkDataSource;
}

function NavItem({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 w-full px-3 py-2 rounded-md text-sm transition-colors ${
        active ? "bg-primary text-primary-foreground font-medium" : "text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}

/** 桌面主畫面：生命週期看板（主視圖）＋已封存獨立頁＋Spectra 級詳情抽屜。 */
export function App({ dataSource }: AppProps) {
  const useStore = useMemo(() => createAppStore(dataSource), [dataSource]);
  const s = useStore();

  // 轉為變更確認框的變更名草稿：預設由 slug 衍生，可改為第二刀名（再轉出扇出）。
  const [promoteName, setPromoteName] = useState("");
  useEffect(() => {
    setPromoteName(s.pendingPromote ?? "");
  }, [s.pendingPromote]);

  useEffect(() => {
    void s.refresh();
    // 檔案監看的宿主層 wiring：外部寫者（CLI、agent、編輯器）改動 openspec/
    // 後，後端去抖發出 workspace-changed，前端一律整批 refresh。卸載時解除。
    const unlisten = listen("workspace-changed", () => {
      void s.refresh();
    });
    return () => {
      void unlisten.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore]);

  const onRunVerb = (verb: Verb, change: string) => {
    if (verb === "archive") s.requestArchive(change);
    else void s.runVerb(verb, change);
  };

  // 同源連結資料（design D6）：來源討論（記錄已不在時以 slug 充當 topic）與
  // 同一討論扇出的 active 兄弟刀（可互跳的對象）。
  const fromSlug = s.detailChange?.fromDiscussion ?? null;
  const allDiscussions = [...s.discussions.active, ...s.discussions.archived];
  const sourceDiscussion = fromSlug
    ? { slug: fromSlug, topic: allDiscussions.find((d) => d.slug === fromSlug)?.topic ?? fromSlug }
    : null;
  const siblingChanges = fromSlug
    ? s.changes
        .filter((c) => c.fromDiscussion === fromSlug && c.name !== s.detailChange?.name)
        .map((c) => c.name)
    : [];

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      {/* 頂欄 */}
      <header className="flex items-center gap-3 px-4 h-12 border-b border-border shrink-0">
        <span className="font-bold text-sm">Speclink</span>
        <span className="text-xs text-muted-foreground px-2 py-0.5 rounded border border-border">
          {/* TODO(desktop-config-multiproject)：專案名／開啟專案＋自動 init */}目前專案
        </span>
        {/* 已封存入口（獨立頁） */}
        <button
          aria-label="已封存"
          className={`flex items-center gap-1.5 text-sm px-2 py-1 rounded-md transition-colors ${
            s.boardView === "archived"
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:text-foreground hover:bg-muted"
          }`}
          onClick={() => s.setBoardView(s.boardView === "archived" ? "board" : "archived")}
        >
          <Archive className="h-4 w-4" /> 已封存
          <span className="inline-flex items-center justify-center min-w-4 h-4 px-1 rounded-full bg-muted text-muted-foreground text-[10px] tabular-nums">
            {s.archived.length}
          </span>
        </button>
        <div className="flex-1" />
        {s.verbResult && (
          <span className="text-xs font-mono text-muted-foreground truncate max-w-[40%]">{s.verbResult}</span>
        )}
        <button className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground px-2 py-1 rounded-md hover:bg-muted">
          <FolderOpen className="h-4 w-4" /> 開啟專案
        </button>
      </header>

      <div className="flex flex-1 overflow-hidden">
        {/* 左側欄 */}
        <aside className="w-[200px] shrink-0 border-r border-border bg-card p-2 flex flex-col gap-1">
          <NavItem
            icon={<GitBranch className="h-4 w-4" />}
            label="變更"
            active={s.boardView === "board"}
            onClick={() => s.setBoardView("board")}
          />
          <NavItem icon={<FileText className="h-4 w-4" />} label="規格" />
          <NavItem icon={<StickyNote className="h-4 w-4" />} label="備忘" />
          <div className="flex-1" />
          <NavItem icon={<Settings className="h-4 w-4" />} label="設定" />
        </aside>

        {/* 主內容：看板填滿高度（欄內捲動）、封存頁整頁縱向捲動 */}
        <main className={`flex-1 p-5 ${s.boardView === "board" ? "overflow-hidden" : "overflow-y-auto"}`}>
          {s.boardView === "board" ? (
            <KanbanBoard
              changes={s.changes}
              onOpenChange={s.openDetail}
              onArchive={s.requestArchive}
              discussions={s.discussions}
              archivedChanges={s.archived}
              onOpenDiscussion={s.openDiscussion}
              onPromoteDiscussion={s.requestPromote}
              onArchiveDiscussion={s.requestArchiveDiscussion}
            />
          ) : (
            <ArchivedList
              archived={s.archived}
              query={s.query}
              onQuery={s.setQuery}
              loadDocument={(datedName, artifact) => dataSource.getArchivedDocument(datedName, artifact)}
              loadCapabilities={(datedName) => dataSource.archivedCapabilities(datedName)}
              archivedDiscussions={s.discussions.archived}
              loadDiscussionDocument={(slug) => dataSource.getDiscussionDocument(slug)}
            />
          )}
        </main>
      </div>

      {/* Spectra 級詳情抽屜（含互動任務） */}
      <RichDetailDrawer
        open={s.detailChange !== null}
        onOpenChange={(o) => !o && s.closeDetail()}
        change={s.detailChange}
        refreshGen={s.refreshGen}
        loadDocument={(change, artifact) => dataSource.getDocument(change, artifact)}
        loadCapabilities={(change) => dataSource.changeCapabilities(change)}
        loadMeta={(change) => dataSource.changeMeta(change)}
        onRunVerb={onRunVerb}
        onDelete={s.requestDelete}
        onToggleTask={async (change, ordinal, done) => {
          await dataSource.setTaskDone(change, ordinal, done);
          await s.refresh();
        }}
        onMoveTask={async (change, from, to, before) => {
          await dataSource.moveTask(change, from, to, before);
          await s.refresh();
        }}
        sourceDiscussion={sourceDiscussion}
        siblingChanges={siblingChanges}
        onOpenDiscussion={(slug) => {
          s.closeDetail();
          s.openDiscussion(slug);
        }}
        onOpenSibling={s.openDetail}
      />

      {/* 討論抽屜（結論/討論過程/背景/衍生變更） */}
      <DiscussionDrawer
        open={s.detailDiscussion !== null}
        onOpenChange={(o) => !o && s.closeDiscussion()}
        discussion={s.detailDiscussion}
        refreshGen={s.refreshGen}
        loadDocument={(slug) => dataSource.getDiscussionDocument(slug)}
        changes={s.changes}
        archivedChanges={s.archived}
        onPromote={s.requestPromote}
        onOpenChangeCard={(name) => {
          s.closeDiscussion();
          s.openDetail(name);
        }}
        error={s.promoteError}
      />

      {/* 轉為變更確認（design D4：說「會發生什麼」，不暴露工程詞） */}
      <AlertDialog open={s.pendingPromote !== null} onOpenChange={(o) => !o && s.cancelPromote()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>轉為變更？</AlertDialogTitle>
            <AlertDialogDescription>
              會在「提案中」新增一張變更卡，提案內容以本討論的結論開頭；討論會移到「已轉出變更」區。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="flex flex-col gap-1.5">
            <label htmlFor="promote-name" className="text-xs text-muted-foreground">
              變更名稱<span className="ml-1 text-muted-foreground/70">（英文小寫，字間用 -）</span>
            </label>
            <Input
              id="promote-name"
              aria-label="變更名稱"
              value={promoteName}
              onChange={(e) => setPromoteName(e.target.value)}
            />
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelPromote}>取消</AlertDialogCancel>
            <AlertDialogAction onClick={() => void s.confirmPromote(promoteName)}>轉為變更</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 討論封存確認 */}
      <AlertDialog
        open={s.pendingArchiveDiscussion !== null}
        onOpenChange={(o) => !o && s.cancelArchiveDiscussion()}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>封存討論？</AlertDialogTitle>
            <AlertDialogDescription>
              討論 <b>{s.pendingArchiveDiscussion}</b> 會移到已封存頁的討論節，可隨時唯讀檢視。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelArchiveDiscussion}>取消</AlertDialogCancel>
            <AlertDialogAction onClick={s.confirmArchiveDiscussion}>封存</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 封存確認 */}
      <AlertDialog open={s.pendingArchive !== null} onOpenChange={(o) => !o && s.cancelArchive()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>封存變更？</AlertDialogTitle>
            <AlertDialogDescription>
              將 <b>{s.pendingArchive}</b> 封存——執行 speclink archive（先驗證，前置未滿足會失敗）。此動作會移動檔案。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelArchive}>取消</AlertDialogCancel>
            <AlertDialogAction onClick={s.confirmArchive}>封存</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 刪除確認 */}
      <AlertDialog open={s.pendingDelete !== null} onOpenChange={(o) => !o && s.cancelDelete()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>刪除變更？</AlertDialogTitle>
            <AlertDialogDescription>
              將永久刪除 <b>{s.pendingDelete}</b> 的整個目錄（proposal/design/specs/tasks）。此動作無法復原。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelDelete}>取消</AlertDialogCancel>
            <AlertDialogAction className="bg-destructive hover:bg-destructive/90" onClick={s.confirmDelete}>
              刪除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
