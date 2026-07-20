import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Archive, GitBranch, FileText, Settings, SlidersHorizontal, FolderOpen } from "lucide-react";
import {
  KanbanBoard,
  ArchivedList,
  ArchivedDrawer,
  SpecList,
  SpecDrawer,
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
  Button,
  Checkbox,
  I18nProvider,
  Toaster,
  useI18n,
  siblingChangesOf,
  type Verb,
} from "@speclink/ui";

import { createAppStore } from "./store";
import type { WorkspaceSession } from "./session";
import { initTray, type TrayController } from "./tray";
import { ProjectTabs } from "./components/ProjectTabs";
import { AppSettingsView } from "./views/AppSettingsView";
import { ProjectSettingsView } from "./views/ProjectSettingsView";
import type { ConnectionsAdapter } from "./adapter/connections";
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
  /** local session 工廠（root 綁定 dataSource／settings／events；workspace-session 決策 3/6）。 */
  createSession: (root: string, name: string) => WorkspaceSession;
  /** remote session 工廠（remote-data-source 決策 6/7）：handshake 成功回
   * session、失敗上拋；未注入時 remote 開啟入口不啟用。 */
  openRemote?: (connectionId: string, target: string) => Promise<WorkspaceSession>;
  /** workspace 探測面（開專案／init／統計／監看重掛）；未注入時對應 UI 不啟用。 */
  workspace?: WorkspaceAdapter;
  /** server 連線面（desktop-connections）；未注入時伺服器頁籤不啟用。 */
  connections?: ConnectionsAdapter;
}

/** 零分頁空狀態引導頁（spec：取代空看板；說明既有專案與一般目錄初始化兩條路）。 */
function EmptyState({ onOpen }: { onOpen: () => void }) {
  const { t } = useI18n();
  return (
    <div className="flex flex-col items-center justify-center h-full gap-3 text-center" data-empty-state>
      <FolderOpen className="h-10 w-10 text-muted-foreground/40" />
      <h2 className="text-lg font-semibold">{t("app.emptyTitle")}</h2>
      <p className="text-sm text-muted-foreground max-w-md">{t("app.emptyDesc")}</p>
      <Button className="gap-1.5" onClick={onOpen}>
        <FolderOpen className="h-4 w-4" /> {t("app.openProject")}
      </Button>
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
    <Button
      variant="ghost"
      aria-label={ariaLabel}
      onClick={onClick}
      className={`h-auto w-full justify-start gap-2 px-3 py-2 text-sm font-normal ${
        active
          ? "bg-primary text-primary-foreground font-medium hover:bg-primary hover:text-primary-foreground"
          : "text-muted-foreground hover:bg-muted hover:text-foreground"
      }${className ? ` ${className}` : ""}`}
    >
      {icon}
      {label}
      {trailing && <span className="ml-auto">{trailing}</span>}
    </Button>
  );
}

/** 桌面 app 進入點：解析 UI 語言（偏好優先、null 跟隨系統）並掛 I18nProvider。 */
export function App({ createSession, openRemote, workspace, connections }: AppProps) {
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
        createSession={createSession}
        openRemote={openRemote}
        workspace={workspace}
        connections={connections}
        localePref={localePref}
        onLocalePrefChange={setLocalePref}
      />
      <Toaster />
    </I18nProvider>
  );
}

interface AppInnerProps extends AppProps {
  /** UI 語言偏好現值（null＝跟隨系統）；設定頁三選用。 */
  localePref: LocalePreference;
  onLocalePrefChange: (pref: LocalePreference) => void;
}

/** 桌面主畫面：生命週期看板（主視圖）＋已封存獨立頁＋設定頁＋Spectra 級詳情抽屜。 */
function AppInner({
  createSession,
  openRemote,
  workspace,
  connections,
  localePref,
  onLocalePrefChange,
}: AppInnerProps) {
  const useStore = useMemo(
    () => createAppStore({ createSession, openRemote, workspace, connections }),
    [createSession, openRemote, workspace, connections],
  );
  const s = useStore();
  // 活躍 session（workspace-session 決策 6）：詳情／規格／封存抽屜的文件載入
  // 與設定頁一律經它——App 不再持有全域 dataSource。
  const activeSession = s.activeKey ? s.sessions[s.activeKey] : undefined;
  const dataSource = activeSession?.dataSource;
  // capability 驅動停用（remote-data-source 決策 2）：remote session 依 server
  // 端點覆蓋停用 affordance；本地 session 全真、同一路徑零分岐。
  const caps = activeSession?.capabilities;
  const servers = connections && {
    connections: s.connections,
    phases: s.connectionPhases,
    onAdd: s.addConnection,
    onLogin: s.loginConnection,
    onSubmitPat: s.submitPat,
    onLogout: s.logoutConnection,
    onRemove: s.removeConnection,
    onRefresh: s.refreshConnections,
    onOpenWorkspace: openRemote && ((id: string, target: string) => s.openRemoteWorkspace(id, target)),
  };
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

  // 看板拖曳手勢讓路（design D6、TaskList 同款）：手勢中暫緩 workspace-changed
  // 的整批 refresh（避免拖曳中卡片重排打斷手勢），放開後補跑一次。
  const boardDragActive = useRef(false);
  const pendingRefresh = useRef(false);
  // Radix AlertDialog 從 Sheet 的 portal 開關時，原生 WebView 可能讓底下 Sheet
  // 短暫收到 onOpenChange(false)。確認中的成功／失敗應由 store 決定是否關抽屜，
  // 不讓這個 portal 生命週期誤清 detailChange。
  const detailConfirmInFlight = useRef(false);
  const handleBoardDragActive = (active: boolean) => {
    boardDragActive.current = active;
    if (!active && pendingRefresh.current) {
      pendingRefresh.current = false;
      void useStore.getState().refresh();
    }
  };

  useEffect(() => {
    // 啟動：有 workspace 即還原分頁列（依持久化 activeKey 切回上次活躍專案、
    // 背景徽章快照）；否則維持既有的整批 refresh（無活躍 session 時為無事）。
    if (workspace) void useStore.getState().restoreTabs();
    else void useStore.getState().refresh();
    return () => {
      // 卸載時取消漏出的搜尋去抖，杜絕在途 timer 於 store 卸載後才開火。
      useStore.getState().disposeSearch();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore]);

  // 檔案監看的宿主層 wiring（workspace-session 決策 5）：訂閱活躍 session 的
  // 事件來源（workspace-changed 以自身 root 過濾），觸發既有整批 refresh；
  // 切換活躍分頁即換訂閱、卸載時解除。
  useEffect(() => {
    const st = useStore.getState();
    const session = st.activeKey ? st.sessions[st.activeKey] : undefined;
    if (!session) return;
    return session.events.subscribe(() => {
      if (boardDragActive.current) {
        pendingRefresh.current = true;
        return;
      }
      void useStore.getState().refresh();
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore, s.activeKey]);

  // 系統匣狀態選單（tray-status-menu）：訂閱同一 store 於選單列／系統匣呈現狀態並可切專案。
  // store 為本元件範圍，故於此接線（非 main.tsx）。建立失敗（如非 Tauri 環境）只靜默降級——
  // app 照常、僅無系統匣（與檔案監看不可用時的降級一致）。卸載時 dispose。
  useEffect(() => {
    let controller: TrayController | null = null;
    let disposed = false;
    // 面板樣式（macOS）：點擊圖示 toggle 面板；建立失敗退回原生選單並於設定頁浮出錯誤。
    const onPanelToggle = () => {
      void invoke("toggle_tray_panel").catch((e) => {
        useStore.getState().panelFallback(String(e));
      });
    };
    initTray(useStore, { onPanelToggle })
      .then((c) => {
        if (disposed) c.dispose();
        else controller = c;
      })
      .catch(() => {
        /* 系統匣不可用：app 照常運作 */
      });
    return () => {
      disposed = true;
      controller?.dispose();
    };
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

  const confirmDetailAction = async (confirm: () => Promise<void>) => {
    detailConfirmInFlight.current = true;
    try {
      await confirm();
    } finally {
      // 讓 AlertDialog 的收合 callback 先完成；之後使用者仍可正常手動關閉抽屜。
      queueMicrotask(() => {
        detailConfirmInFlight.current = false;
      });
    }
  };

  // 同源連結資料：來源討論清單（記錄已不在時以 slug 充當 topic，出身討論在前）與
  // 同源刀（與此變更共享至少一份來源討論的 active 變更，可互跳）。
  const fromSlugs = s.detailChange?.fromDiscussions ?? [];
  const allDiscussions = [...s.discussions.active, ...s.discussions.archived];
  const sourceDiscussions = fromSlugs.map((slug) => ({
    slug,
    topic: allDiscussions.find((d) => d.slug === slug)?.topic ?? slug,
  }));
  const siblingChanges = siblingChangesOf(s.changes, fromSlugs, s.detailChange?.name ?? "");
  // 封存變更抽屜的來源討論（design D1 增補）：自封存清單以 datedName 查 fromDiscussions，
  // topic 同上解析、記錄已不在時退回 slug。
  const archivedFromSlugs =
    s.detailArchived?.kind === "change"
      ? (s.archived.find((a) => a.datedName === s.detailArchived?.datedName)?.fromDiscussions ?? [])
      : [];
  const archivedSourceDiscussions = archivedFromSlugs.map((slug) => ({
    slug,
    topic: allDiscussions.find((d) => d.slug === slug)?.topic ?? slug,
  }));

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      {/* 頂欄 */}
      <header className="flex items-center gap-3 px-4 h-12 border-b border-border shrink-0">
        <div className="flex items-center gap-1.5 shrink-0">
          <img
            src="./logo-mark.png"
            alt=""
            aria-hidden="true"
            className="h-5 w-5"
          />
          <img src="./speclink-wordmark.png" alt="Speclink" className="h-5 w-auto" />
        </div>
        {workspace !== undefined ? (
          // 專案分頁列取代「目前專案」佔位（design D10）：active 分頁即目前專案。
          <ProjectTabs
            tabs={s.tabs}
            activeKey={s.activeKey}
            tabErrors={s.tabErrors}
            onActivate={(key) => void s.activateTab(key)}
            onClose={s.closeTab}
            onOpen={() => void s.openProjectViaDialog()}
          />
        ) : (
          <span className="text-xs text-muted-foreground px-2 py-0.5 rounded border border-border">
            {t("app.currentProject")}
          </span>
        )}
        <div className="flex-1" />
        <Button
          variant="ghost"
          size="sm"
          className="gap-1.5 px-2 text-sm font-normal text-muted-foreground hover:text-foreground"
          onClick={() => void s.openProjectViaDialog()}
        >
          <FolderOpen className="h-4 w-4" /> {t("app.openProject")}
        </Button>
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
          {/* 規格頁入口：切頁語意（與已封存頁同型），返回看板改點「變更」。 */}
          <NavItem
            icon={<FileText className="h-4 w-4" />}
            label={t("app.navSpecs")}
            active={s.boardView === "specs"}
            onClick={() => s.setBoardView("specs")}
          />
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
          <NavItem
            icon={<SlidersHorizontal className="h-4 w-4" />}
            label={t("app.navProjectSettings")}
            active={s.boardView === "project-settings"}
            onClick={() => s.setBoardView("project-settings")}
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

        {/* 主內容：看板、規格頁、已封存頁填滿高度（清單於內部容器捲動、換頁控
            制列沉底常駐）；設定頁維持整頁縱向捲動 */}
        <main className={`flex-1 p-5 ${s.boardView === "settings" || s.boardView === "project-settings" ? "overflow-y-auto" : "overflow-hidden"}`}>
          {s.boardView === "settings" ? (
            <AppSettingsView
              localePref={localePref}
              onLocalePrefChange={onLocalePrefChange}
              trayPanelError={s.trayPanelError}
              servers={servers}
            />
          ) : workspace !== undefined && s.tabs.length === 0 ? (
            // 零分頁（首次使用）：空狀態引導頁取代空看板。
            <EmptyState onOpen={() => void s.openProjectViaDialog()} />
          ) : s.boardView === "project-settings" && activeSession !== undefined ? (
            <ProjectSettingsView settings={activeSession.settings} />
          ) : s.boardView === "specs" ? (
            <SpecList specs={s.specs} onOpen={s.openSpec} />
          ) : s.boardView === "board" ? (
            <KanbanBoard
              changes={s.changes}
              onOpenChange={s.openDetail}
              onArchive={s.requestArchive}
              discussions={s.discussions}
              archivedChanges={s.archived}
              onOpenDiscussion={s.openDiscussion}
              onArchiveDiscussion={s.requestArchiveDiscussion}
              query={s.boardQuery}
              onQuery={s.setBoardQuery}
              fulltextHits={s.searchHits}
              searchUnavailableReason={
                caps && !caps.searchWorkspace ? t("remote.searchUnavailable") : undefined
              }
              onReorder={
                caps && !caps.reorderCard
                  ? undefined
                  : (kind, id, prevId, nextId) => void s.reorderCard(kind, id, prevId, nextId)
              }
              onDragActiveChange={handleBoardDragActive}
            />
          ) : caps && !caps.listArchived ? (
            // remote capability 缺口（決策 2）：不偽造封存頁——提示卡如實呈現。
            <div
              data-testid="archived-unavailable"
              className="rounded-md border border-border bg-card p-6 text-sm"
            >
              <div className="font-medium">{t("remote.archivedUnavailableTitle")}</div>
              <p className="mt-1 text-muted-foreground">{t("remote.archivedUnavailableBody")}</p>
            </div>
          ) : (
            <ArchivedList
              archived={s.archived}
              query={s.query}
              onQuery={s.setQuery}
              archivedDiscussions={s.discussions.archived}
              onOpen={s.openArchived}
            />
          )}
        </main>
      </div>

      {/* Spectra 級詳情抽屜（含互動任務） */}
      <RichDetailDrawer
        open={s.detailChange !== null}
        onOpenChange={(o) => !o && !detailConfirmInFlight.current && s.closeDetail()}
        change={s.detailChange}
        refreshGen={s.refreshGen}
        loadDocument={(change, artifact) => dataSource?.getDocument(change, artifact) ?? Promise.resolve(null)}
        // capability 缺口的讀取（changeCapabilities/changeMeta 無 server 來源）
        // 以空集呈現——資料缺口不偽造、也不讓抽屜載入失敗。
        loadCapabilities={(change) =>
          caps && !caps.changeCapabilities
            ? Promise.resolve([])
            : (dataSource?.changeCapabilities(change) ?? Promise.resolve([]))
        }
        loadMeta={(change) =>
          caps && !caps.changeMeta
            ? Promise.resolve(null)
            : (dataSource?.changeMeta(change) ?? Promise.resolve(null))
        }
        onRunVerb={onRunVerb}
        drawerVerb={s.drawerVerb}
        onClearVerb={s.clearDrawerVerb}
        onDelete={s.requestDelete}
        unavailable={
          caps && {
            analyze:
              caps.validate && caps.analyze ? undefined : t("remote.analyzeUnavailable"),
            delete: caps.deleteChange ? undefined : t("remote.deleteUnavailable"),
          }
        }
        onToggleTask={async (change, task, done) => {
          await dataSource?.setTaskDone(change, task, done);
          await s.refresh();
        }}
        onMoveTask={
          caps && !caps.moveTask
            ? undefined
            : async (change, from, to, before) => {
                await dataSource?.moveTask(change, from, to, before);
                await s.refresh();
              }
        }
        onSetAllTasks={async (change, done) => {
          await dataSource?.setAllTasks(change, done);
          await s.refresh();
        }}
        sourceDiscussions={sourceDiscussions}
        siblingChanges={siblingChanges}
        onOpenDiscussion={s.openDiscussion}
        onOpenSibling={s.openDetail}
      />

      {/* 唯讀規格抽屜（spec-archive-drawer design D1/D2） */}
      <SpecDrawer
        open={s.detailSpec !== null}
        onOpenChange={(o) => !o && s.closeSpec()}
        capability={s.detailSpec}
        refreshGen={s.refreshGen}
        // capability 缺口（正典 spec 內文無 server 端點）：內文區以繁中提示卡
        // 如實呈現缺口，不偽造、不以「找不到」誤導。
        loadDocument={(capability) =>
          caps && !caps.getSpecDocument
            ? Promise.resolve(t("remote.specDocUnavailable"))
            : (dataSource?.getSpecDocument(capability) ?? Promise.resolve(null))
        }
      />

      {/* 唯讀封存抽屜（封存變更四分頁／封存討論三區段） */}
      <ArchivedDrawer
        open={s.detailArchived !== null}
        onOpenChange={(o) => !o && s.closeArchived()}
        target={s.detailArchived}
        refreshGen={s.refreshGen}
        loadDocument={(datedName, artifact) => dataSource?.getArchivedDocument(datedName, artifact) ?? Promise.resolve(null)}
        loadCapabilities={(datedName) => dataSource?.archivedCapabilities(datedName) ?? Promise.resolve([])}
        loadDiscussionDocument={(slug) => dataSource?.getDiscussionDocument(slug) ?? Promise.resolve(null)}
        sourceDiscussions={archivedSourceDiscussions}
        onOpenDiscussion={(slug) => s.openArchived({ kind: "discussion", slug })}
      />

      {/* 討論抽屜（結論/討論過程/背景/衍生變更） */}
      <DiscussionDrawer
        open={s.detailDiscussion !== null}
        onOpenChange={(o) => !o && s.closeDiscussion()}
        discussion={s.detailDiscussion}
        refreshGen={s.refreshGen}
        loadDocument={(slug) => dataSource?.getDiscussionDocument(slug) ?? Promise.resolve(null)}
        changes={s.changes}
        archivedChanges={s.archived}
        onOpenChangeCard={s.openDetail}
      />

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
                <Checkbox
                  checked={initTools.includes(tool)}
                  onCheckedChange={(v) =>
                    setInitTools((prev) =>
                      v === true ? [...prev, tool] : prev.filter((x) => x !== tool),
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
            <AlertDialogAction onClick={() => void confirmDetailAction(s.confirmArchive)}>
              {t("app.archiveConfirm")}
            </AlertDialogAction>
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
            <AlertDialogAction
              className="bg-destructive hover:bg-destructive/90"
              onClick={() => void confirmDetailAction(s.confirmDelete)}
            >
              {t("app.deleteConfirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
