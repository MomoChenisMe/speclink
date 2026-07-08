import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Archive, GitBranch, FileText, Settings, FolderOpen } from "lucide-react";
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
  I18nProvider,
  useI18n,
  type SpeclinkDataSource,
  type Verb,
} from "@speclink/ui";

import { createAppStore } from "./store";
import { ProjectTabs } from "./components/ProjectTabs";
import { SettingsView } from "./views/SettingsView";
import type { WorkspaceAdapter } from "./adapter/workspace";
import { APP_MESSAGES } from "./i18n/messages";
import {
  readLocalePreference,
  resolveUiLocale,
  writeLocalePreference,
  type LocalePreference,
} from "./i18n/locale";
import { setAppT } from "./i18n/runtime";

export interface AppProps {
  dataSource: SpeclinkDataSource;
  /** workspace 管理操作（開專案／init／設定）；未注入時對應 UI 不啟用。 */
  workspace?: WorkspaceAdapter;
}

/** 零分頁空狀態引導頁（spec：取代空看板；說明既有專案與一般目錄初始化兩條路）。 */
function EmptyState({ onOpen }: { onOpen: () => void }) {
  const { t } = useI18n();
  return (
    <div className="flex flex-col items-center justify-center h-full gap-3 text-center" data-empty-state>
      <FolderOpen className="h-10 w-10 text-muted-foreground/40" />
      <h2 className="text-lg font-semibold">{t("app.emptyTitle")}</h2>
      <p className="text-sm text-muted-foreground max-w-md">{t("app.emptyDesc")}</p>
      <button
        className="flex items-center gap-1.5 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        onClick={onOpen}
      >
        <FolderOpen className="h-4 w-4" /> {t("app.openProject")}
      </button>
    </div>
  );
}

/** 對話框描述文字內嵌粗體名稱：以 {name} 佔位切分渲染。 */
function BoldName({ text, name }: { text: string; name: string }) {
  const [before, after] = text.split("{name}");
  return (
    <>
      {before}
      <b>{name}</b>
      {after}
    </>
  );
}

function NavItem({
  icon,
  label,
  active,
  onClick,
  trailing,
  ariaLabel,
  className,
}: {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  onClick?: () => void;
  /** 尾隨元素（如計數徽章）；設 ariaLabel 使無障礙名稱不被徽章數字污染。 */
  trailing?: React.ReactNode;
  ariaLabel?: string;
  /** 附加版面 class（如 mt-auto 沉底）；不影響既有樣式與行為。 */
  className?: string;
}) {
  return (
    <button
      aria-label={ariaLabel}
      onClick={onClick}
      className={`flex items-center gap-2 w-full px-3 py-2 rounded-md text-sm transition-colors ${
        active ? "bg-primary text-primary-foreground font-medium" : "text-muted-foreground hover:bg-muted hover:text-foreground"
      }${className ? ` ${className}` : ""}`}
    >
      {icon}
      {label}
      {trailing && <span className="ml-auto">{trailing}</span>}
    </button>
  );
}

/** 桌面 app 進入點：解析 UI 語言（偏好優先、null 跟隨系統）並掛 I18nProvider。 */
export function App({ dataSource, workspace }: AppProps) {
  const [localePref, setLocalePrefState] = useState<LocalePreference>(() => readLocalePreference());
  // 切換即時生效並持久化（設定頁的 UI 語言三選接這裡）。
  const setLocalePref = (pref: LocalePreference) => {
    writeLocalePreference(pref);
    setLocalePrefState(pref);
  };
  const uiLocale = resolveUiLocale(
    localePref,
    typeof navigator !== "undefined" ? navigator.language : undefined,
  );
  return (
    <I18nProvider locale={uiLocale} messages={APP_MESSAGES}>
      <AppInner
        dataSource={dataSource}
        workspace={workspace}
        localePref={localePref}
        onLocalePrefChange={setLocalePref}
      />
    </I18nProvider>
  );
}

interface AppInnerProps extends AppProps {
  /** UI 語言偏好現值（null＝跟隨系統）；設定頁三選用。 */
  localePref: LocalePreference;
  onLocalePrefChange: (pref: LocalePreference) => void;
}

/** 桌面主畫面：生命週期看板（主視圖）＋已封存獨立頁＋設定頁＋Spectra 級詳情抽屜。 */
function AppInner({ dataSource, workspace, localePref, onLocalePrefChange }: AppInnerProps) {
  const useStore = useMemo(() => createAppStore(dataSource, workspace), [dataSource, workspace]);
  const s = useStore();
  // 初始化確認框的工具多選（預設勾 claude）；對話框每次開啟重設。
  const [initTools, setInitTools] = useState<string[]>(["claude"]);
  useEffect(() => {
    if (s.pendingInit) setInitTools(["claude"]);
  }, [s.pendingInit]);
  const { t } = useI18n();
  // 同步 store 層（非 React）的 t 橋——store 組使用者可見訊息時取當前語言。
  useEffect(() => {
    setAppT(t);
  }, [t]);

  // 轉為變更確認框的變更名草稿：預設由 slug 衍生，可改為第二刀名（再轉出扇出）。
  const [promoteName, setPromoteName] = useState("");
  useEffect(() => {
    setPromoteName(s.pendingPromote ?? "");
  }, [s.pendingPromote]);

  // 看板拖曳手勢讓路（design D6、TaskList 同款）：手勢中暫緩 workspace-changed
  // 的整批 refresh（避免拖曳中卡片重排打斷手勢），放開後補跑一次。
  const boardDragActive = useRef(false);
  const pendingRefresh = useRef(false);
  const handleBoardDragActive = (active: boolean) => {
    boardDragActive.current = active;
    if (!active && pendingRefresh.current) {
      pendingRefresh.current = false;
      void useStore.getState().refresh();
    }
  };

  useEffect(() => {
    // 啟動：有 workspace 即還原分頁列（含切回上次活躍專案與背景徽章快照）；
    // 否則維持既有的整批 refresh。
    if (workspace) void useStore.getState().restoreTabs();
    else void useStore.getState().refresh();
    // 檔案監看的宿主層 wiring：外部寫者（CLI、agent、編輯器）改動 openspec/
    // 後，後端去抖發出 workspace-changed，前端一律整批 refresh。卸載時解除。
    const unlisten = listen("workspace-changed", () => {
      if (boardDragActive.current) {
        pendingRefresh.current = true;
        return;
      }
      void useStore.getState().refresh();
    });
    return () => {
      void unlisten.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore]);

  // 鍵盤切換分頁：Ctrl+Tab 循環、Ctrl+1..9 直達（spec「專案分頁列存於 app 本機」）。
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey) return;
      if (e.key === "Tab") {
        e.preventDefault();
        void useStore.getState().cycleTab();
      } else if (e.key >= "1" && e.key <= "9") {
        e.preventDefault();
        void useStore.getState().gotoTab(Number(e.key));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
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
        {workspace !== undefined ? (
          // 專案分頁列取代「目前專案」佔位（design D10）：active 分頁即目前專案。
          <ProjectTabs
            tabs={s.tabs}
            activeRoot={s.activeRoot}
            tabErrors={s.tabErrors}
            onActivate={(root) => void s.activateTab(root)}
            onClose={s.closeTab}
            onOpen={() => void s.openProjectViaDialog()}
          />
        ) : (
          <span className="text-xs text-muted-foreground px-2 py-0.5 rounded border border-border">
            {t("app.currentProject")}
          </span>
        )}
        <div className="flex-1" />
        {s.verbResult && (
          <span className="text-xs font-mono text-muted-foreground truncate max-w-[40%]">{s.verbResult}</span>
        )}
        <button
          className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground px-2 py-1 rounded-md hover:bg-muted"
          onClick={() => void s.openProjectViaDialog()}
        >
          <FolderOpen className="h-4 w-4" /> {t("app.openProject")}
        </button>
      </header>

      <div className="flex flex-1 overflow-hidden">
        {/* 左側欄 */}
        <aside className="w-[200px] shrink-0 border-r border-border bg-card p-2 flex flex-col gap-1">
          <NavItem
            icon={<GitBranch className="h-4 w-4" />}
            label={t("app.navChanges")}
            active={s.boardView === "board"}
            onClick={() => s.setBoardView("board")}
          />
          <NavItem icon={<FileText className="h-4 w-4" />} label={t("app.navSpecs")} />
          {/* 已封存入口（獨立頁）：切頁語意，返回看板改點「變更」。 */}
          <NavItem
            icon={<Archive className="h-4 w-4" />}
            label={t("app.archived")}
            ariaLabel={t("app.archived")}
            active={s.boardView === "archived"}
            onClick={() => s.setBoardView("archived")}
            trailing={
              <span className="inline-flex items-center justify-center min-w-4 h-4 px-1 rounded-full bg-muted text-muted-foreground text-[10px] tabular-nums">
                {s.archived.length}
              </span>
            }
          />
          {/* 設定沉底：自動上邊距推至側欄底部（design D5），切頁與高亮語意不變。 */}
          <NavItem
            icon={<Settings className="h-4 w-4" />}
            label={t("app.navSettings")}
            active={s.boardView === "settings"}
            onClick={() => s.setBoardView("settings")}
            className="mt-auto"
          />
        </aside>

        {/* 主內容：看板填滿高度（欄內捲動）、封存頁整頁縱向捲動 */}
        <main className={`flex-1 p-5 ${s.boardView === "board" ? "overflow-hidden" : "overflow-y-auto"}`}>
          {s.boardView === "settings" && workspace !== undefined ? (
            <SettingsView
              workspace={workspace}
              localePref={localePref}
              onLocalePrefChange={onLocalePrefChange}
            />
          ) : workspace !== undefined && s.tabs.length === 0 ? (
            // 零分頁（首次使用）：空狀態引導頁取代空看板。
            <EmptyState onOpen={() => void s.openProjectViaDialog()} />
          ) : s.boardView === "board" ? (
            <KanbanBoard
              changes={s.changes}
              onOpenChange={s.openDetail}
              onArchive={s.requestArchive}
              discussions={s.discussions}
              archivedChanges={s.archived}
              onOpenDiscussion={s.openDiscussion}
              onPromoteDiscussion={s.requestPromote}
              onArchiveDiscussion={s.requestArchiveDiscussion}
              query={s.boardQuery}
              onQuery={s.setBoardQuery}
              onReorder={(kind, id, prevId, nextId) => void s.reorderCard(kind, id, prevId, nextId)}
              onDragActiveChange={handleBoardDragActive}
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
        onSetAllTasks={async (change, done) => {
          await dataSource.setAllTasks(change, done);
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
            <AlertDialogTitle>{t("app.promoteTitle")}</AlertDialogTitle>
            <AlertDialogDescription>{t("app.promoteDesc")}</AlertDialogDescription>
          </AlertDialogHeader>
          <div className="flex flex-col gap-1.5">
            <label htmlFor="promote-name" className="text-xs text-muted-foreground">
              {t("app.changeName")}
              <span className="ml-1 text-muted-foreground/70">{t("app.changeNameHint")}</span>
            </label>
            <Input
              id="promote-name"
              aria-label={t("app.changeName")}
              value={promoteName}
              onChange={(e) => setPromoteName(e.target.value)}
            />
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelPromote}>{t("app.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void s.confirmPromote(promoteName)}>
              {t("app.promoteConfirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 初始化確認（design D3：寫入型確認框——取消靠左持預設焦點、建立靠右拉開距離） */}
      <AlertDialog open={s.pendingInit !== null} onOpenChange={(o) => !o && s.cancelInit()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("app.initTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              <BoldName text={t("app.initDesc")} name={s.pendingInit ?? ""} />
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="flex gap-4">
            {["claude", "codex"].map((tool) => (
              <label key={tool} className="flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  className="accent-[var(--primary)]"
                  checked={initTools.includes(tool)}
                  onChange={(e) =>
                    setInitTools((prev) =>
                      e.target.checked ? [...prev, tool] : prev.filter((x) => x !== tool),
                    )
                  }
                />
                {tool}
              </label>
            ))}
          </div>
          <AlertDialogFooter className="justify-between sm:justify-between">
            <AlertDialogCancel autoFocus onClick={s.cancelInit}>
              {t("app.cancel")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={() => void s.confirmInit(initTools)}>
              {t("app.initConfirm")}
            </AlertDialogAction>
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
            <AlertDialogTitle>{t("app.archiveDiscussionTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              <BoldName text={t("app.archiveDiscussionDesc")} name={s.pendingArchiveDiscussion ?? ""} />
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelArchiveDiscussion}>{t("app.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={s.confirmArchiveDiscussion}>{t("app.archiveConfirm")}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 封存確認 */}
      <AlertDialog open={s.pendingArchive !== null} onOpenChange={(o) => !o && s.cancelArchive()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("app.archiveTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              <BoldName text={t("app.archiveDesc")} name={s.pendingArchive ?? ""} />
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelArchive}>{t("app.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={s.confirmArchive}>{t("app.archiveConfirm")}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 刪除確認 */}
      <AlertDialog open={s.pendingDelete !== null} onOpenChange={(o) => !o && s.cancelDelete()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("app.deleteTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              <BoldName text={t("app.deleteDesc")} name={s.pendingDelete ?? ""} />
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={s.cancelDelete}>{t("app.cancel")}</AlertDialogCancel>
            <AlertDialogAction className="bg-destructive hover:bg-destructive/90" onClick={s.confirmDelete}>
              {t("app.deleteConfirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
