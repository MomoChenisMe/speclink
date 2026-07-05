import { useEffect, useMemo } from "react";
import { Archive, GitBranch, FileText, StickyNote, Settings, FolderOpen } from "lucide-react";
import {
  KanbanBoard,
  ArchivedList,
  RichDetailDrawer,
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
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

  useEffect(() => {
    void s.refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore]);

  const onRunVerb = (verb: Verb, change: string) => {
    if (verb === "archive") s.requestArchive(change);
    else void s.runVerb(verb, change);
  };

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
            />
          ) : (
            <ArchivedList archived={s.archived} query={s.query} onQuery={s.setQuery} />
          )}
        </main>
      </div>

      {/* Spectra 級詳情抽屜（含互動任務） */}
      <RichDetailDrawer
        open={s.detailChange !== null}
        onOpenChange={(o) => !o && s.closeDetail()}
        change={s.detailChange}
        loadDocument={(change, artifact) => dataSource.getDocument(change, artifact)}
        loadCapabilities={(change) => dataSource.changeCapabilities(change)}
        loadMeta={(change) => dataSource.changeMeta(change)}
        onRunVerb={onRunVerb}
        onDelete={s.requestDelete}
        onToggleTask={async (change, ordinal, done) => {
          await dataSource.setTaskDone(change, ordinal, done);
          await s.refresh();
        }}
        onMoveTask={async (change, from, to) => {
          await dataSource.moveTask(change, from, to);
          await s.refresh();
        }}
      />

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
