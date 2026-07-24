import { useEffect, useState } from "react";
import { ArrowLeft, Cloud, Folder, GitBranch, Plus, Server } from "lucide-react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  Checkbox,
  Input,
  cn,
  useI18n,
} from "@speclink/ui";

import type {
  ConnectionView,
  ConnectionsAdapter,
  ProjectScopeView,
  ScopeRefView,
  ScopesView,
} from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";

type Step = "source" | "server" | "scopes" | "checkout";

export interface ScopeChoice {
  project: ProjectScopeView;
  repo: ScopeRefView;
}

export interface WorkspaceChooserProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  connections: ConnectionView[];
  connectionAdapter: Pick<
    ConnectionsAdapter,
    "scopes" | "inspectCheckout" | "bindCheckout"
  >;
  workspace: Pick<WorkspaceAdapter, "pickFolder" | "openProject">;
  onOpenLocal: (path: string) => Promise<void>;
  onRequestMigration?: (root: string) => Promise<void>;
  onAddServer: (baseUrl: string, name: string) => Promise<void>;
  onRefreshConnections: () => Promise<void>;
  onOpenRemote: (
    connectionId: string,
    target: string,
    checkoutRoot?: string,
  ) => Promise<void>;
  /** 伺服器頁入口：預選已登入 connection 並直達 scopes。 */
  initialConnectionId?: string | null;
  /** remote marker 未登入時：server 步驟預填 marker url。 */
  initialServerUrl?: string | null;
  /** 既有 marker 缺工具選集：直達 checkout 步驟並預選此 scope。 */
  initialScope?: { projectKey: string; repoKey: string } | null;
  /** 既有 marker 缺工具選集：checkout 步驟預填此資料夾路徑並自動 inspect。 */
  initialCheckoutPath?: string | null;
}

const STEP_NUMBER: Record<Step, number> = {
  source: 1,
  server: 2,
  scopes: 3,
  checkout: 4,
};

function ChoiceCard({
  icon,
  title,
  description,
  onClick,
  selected = false,
  disabled = false,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  onClick: () => void;
  selected?: boolean;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      data-selected={String(selected)}
      onClick={onClick}
      className={cn(
        "group flex w-full items-start gap-3 rounded-lg border p-3 text-left transition-colors",
        selected
          ? "border-primary bg-primary/8"
          : "border-border hover:border-primary/50 hover:bg-muted/60",
        disabled && "cursor-not-allowed opacity-50",
      )}
    >
      <span
        className={cn(
          "mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground",
          selected && "bg-primary/12 text-primary",
        )}
      >
        {icon}
      </span>
      <span className="min-w-0">
        <span className="block text-sm font-medium text-foreground">{title}</span>
        <span className="mt-0.5 block text-xs leading-5 text-muted-foreground">
          {description}
        </span>
      </span>
    </button>
  );
}

/** chooser 與正式遷移共用的 membership scope 單選面。 */
export function ScopeSelection({
  scopes,
  selected,
  busy,
  onSelect,
}: {
  scopes: ScopesView | null;
  selected: ScopeChoice | null;
  busy: boolean;
  onSelect: (scope: ScopeChoice) => void;
}) {
  const { t } = useI18n();
  const repoCount =
    scopes?.projects.reduce((sum, project) => sum + project.repos.length, 0) ?? 0;
  return (
    <div className="flex max-h-[360px] flex-col gap-3 overflow-y-auto pr-1">
      {busy && (
        <p className="m-0 text-sm text-muted-foreground">{t("chooser.loadingScopes")}</p>
      )}
      {!busy && scopes && repoCount === 0 && (
        <p
          className="m-0 rounded-md border border-dashed border-border p-3 text-sm text-muted-foreground"
          data-testid="scopes-empty"
        >
          {t("chooser.noMemberships")}
        </p>
      )}
      {scopes?.projects.map((project) =>
        project.repos.length > 0 ? (
          <section key={project.id} className="flex flex-col gap-1.5">
            <div className="flex items-baseline justify-between gap-3 px-1">
              <h3 className="m-0 text-sm font-semibold">{project.name}</h3>
              <span className="font-mono text-[11px] text-muted-foreground">
                {project.key}
              </span>
            </div>
            <div role="radiogroup" aria-label={project.name} className="flex flex-col gap-1.5">
              {project.repos.map((repo) => {
                const checked =
                  selected?.project.id === project.id && selected.repo.id === repo.id;
                return (
                  <button
                    key={repo.id}
                    type="button"
                    role="radio"
                    aria-checked={checked}
                    onClick={() => onSelect({ project, repo })}
                    className={cn(
                      "flex items-center gap-2 rounded-md border px-3 py-2 text-left text-sm",
                      checked
                        ? "border-primary bg-primary/8"
                        : "border-border hover:bg-muted/60",
                    )}
                  >
                    <GitBranch className="h-3.5 w-3.5 text-muted-foreground" />
                    <span className="font-medium">{repo.name}</span>
                    <span className="ml-auto font-mono text-xs text-muted-foreground">
                      {repo.key}
                    </span>
                  </button>
                );
              })}
            </div>
          </section>
        ) : null,
      )}
    </div>
  );
}

export function WorkspaceChooser({
  open,
  onOpenChange,
  connections,
  connectionAdapter,
  workspace,
  onOpenLocal,
  onRequestMigration,
  onAddServer,
  onRefreshConnections,
  onOpenRemote,
  initialConnectionId = null,
  initialServerUrl = null,
  initialScope = null,
  initialCheckoutPath = null,
}: WorkspaceChooserProps) {
  const { t } = useI18n();
  const [step, setStep] = useState<Step>("source");
  const [connection, setConnection] = useState<ConnectionView | null>(null);
  const [scopes, setScopes] = useState<ScopesView | null>(null);
  const [scope, setScope] = useState<ScopeChoice | null>(null);
  const [checkoutMode, setCheckoutMode] = useState<"skip" | "folder" | null>(null);
  const [checkoutRoot, setCheckoutRoot] = useState<string | null>(null);
  // folder mode 的 built-in 工具選集：inspect 回傳既有選集作為初值，開啟時交給 bind。
  const [checkoutTools, setCheckoutTools] = useState<string[]>([]);
  const [serverUrl, setServerUrl] = useState("");
  const [serverName, setServerName] = useState("");
  const [showAddServer, setShowAddServer] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [localProject, setLocalProject] = useState<{ root: string; name: string } | null>(null);

  async function selectConnection(selected: ConnectionView) {
    setConnection(selected);
    setStep("scopes");
    setScope(null);
    setScopes(null);
    setError(null);
    setBusy(true);
    try {
      setScopes(await connectionAdapter.scopes(selected.id));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  // 既有 marker 缺工具選集的入口：載入 scopes、依 key 預選 scope、直達 checkout
  // 步驟並以預填 path inspect（顯示既有選集），讓使用者明示選擇 Claude／Codex。
  async function startAtCheckout(
    selected: ConnectionView,
    scopeKeys: { projectKey: string; repoKey: string },
    path: string,
  ) {
    setConnection(selected);
    setScope(null);
    setScopes(null);
    setError(null);
    setBusy(true);
    try {
      const loaded = await connectionAdapter.scopes(selected.id);
      setScopes(loaded);
      const project = loaded.projects.find((p) => p.key === scopeKeys.projectKey);
      const repo = project?.repos.find((r) => r.key === scopeKeys.repoKey);
      if (!project || !repo) {
        setStep("scopes");
        return;
      }
      setScope({ project, repo });
      setStep("checkout");
      const inspection = await connectionAdapter.inspectCheckout(
        path,
        selected.origin,
        project.key,
        repo.key,
      );
      setCheckoutMode("folder");
      setCheckoutRoot(inspection.root);
      setCheckoutTools(inspection.tools);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    if (!open) return;
    void onRefreshConnections();
    setConnection(null);
    setScopes(null);
    setScope(null);
    setCheckoutMode(null);
    setCheckoutRoot(null);
    setCheckoutTools([]);
    setServerUrl(initialServerUrl ?? "");
    setServerName("");
    setShowAddServer(Boolean(initialServerUrl));
    setBusy(false);
    setError(null);
    setNotice(null);
    setLocalProject(null);
    const selected = initialConnectionId
      ? connections.find((entry) => entry.id === initialConnectionId && entry.loggedIn)
      : undefined;
    if (selected && initialScope && initialCheckoutPath) {
      void startAtCheckout(selected, initialScope, initialCheckoutPath);
    } else if (selected) {
      void selectConnection(selected);
    } else {
      setStep(initialConnectionId || initialServerUrl ? "server" : "source");
    }
    // 開啟時只消費當次入口意圖；連線清單更新不得把進行中的 chooser 重設。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, initialConnectionId, initialServerUrl]);

  async function chooseLocal() {
    setBusy(true);
    setError(null);
    try {
      const picked = await workspace.pickFolder();
      if (!picked) return;
      const probe = await workspace.openProject(picked);
      if (probe.status === "project") {
        setLocalProject({ root: probe.root, name: probe.name });
        return;
      }
      onOpenChange(false);
      await onOpenLocal(picked);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function openLocal(root: string) {
    onOpenChange(false);
    await onOpenLocal(root);
  }

  async function migrateLocal(root: string) {
    if (!onRequestMigration) return;
    setBusy(true);
    setError(null);
    try {
      await onRequestMigration(root);
      onOpenChange(false);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function addServer() {
    const url = serverUrl.trim();
    if (!url) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await onAddServer(url, serverName.trim() || url);
      await onRefreshConnections();
      setServerName("");
      setNotice(t("chooser.serverAdded"));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  // 先檢查階段（零寫入）：選好資料夾後 inspect 驗證 marker 一致性，成功才顯示
  // 工具 checkbox 並以既有選集預選。失敗保留在 checkout 步驟顯示錯誤、不寫入。
  async function chooseCheckoutFolder() {
    if (!connection || !scope) return;
    const picked = await workspace.pickFolder();
    if (!picked) return;
    setBusy(true);
    setError(null);
    try {
      const inspection = await connectionAdapter.inspectCheckout(
        picked,
        connection.origin,
        scope.project.key,
        scope.repo.key,
      );
      setCheckoutMode("folder");
      setCheckoutRoot(inspection.root);
      setCheckoutTools(inspection.tools);
    } catch (e) {
      setCheckoutMode(null);
      setCheckoutRoot(null);
      setCheckoutTools([]);
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  function toggleCheckoutTool(tool: string, on: boolean) {
    setCheckoutTools((prev) =>
      on ? [...prev.filter((t) => t !== tool), tool] : prev.filter((t) => t !== tool),
    );
  }

  // 開啟：skip 直接 handshake；folder mode 先同步受管產物（bind），全部成功才
  // openRemote 帶 checkoutRoot——同步任一步失敗保留步驟與選集供重試，不開分頁。
  async function openRemote() {
    if (!connection || !scope || checkoutMode === null) return;
    if (checkoutMode === "folder" && (checkoutTools.length === 0 || !checkoutRoot)) return;
    setBusy(true);
    setError(null);
    try {
      let checkoutTarget: string | undefined;
      if (checkoutMode === "folder") {
        await connectionAdapter.bindCheckout(
          checkoutRoot!,
          connection.origin,
          scope.project.key,
          scope.repo.key,
          checkoutTools,
        );
        checkoutTarget = checkoutRoot!;
      }
      await onOpenRemote(
        connection.id,
        `${scope.project.key}/${scope.repo.key}`,
        checkoutTarget,
      );
      onOpenChange(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  function back() {
    setError(null);
    if (step === "server") setStep("source");
    else if (step === "scopes") setStep("server");
    else if (step === "checkout") setStep("scopes");
  }

  const loggedInConnections = connections.filter((entry) => entry.loggedIn);
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent className="max-w-xl gap-4" data-testid="workspace-chooser">
        <AlertDialogHeader>
          <div className="mb-1 flex items-center justify-between gap-4">
            <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary">
              {t("chooser.step")
                .replace("{current}", String(STEP_NUMBER[step]))
                .replace("{total}", "4")}
            </span>
            <div className="flex gap-1" aria-hidden="true">
              {[1, 2, 3, 4].map((number) => (
                <span
                  key={number}
                  className={cn(
                    "h-1 w-8 rounded-full bg-muted",
                    number <= STEP_NUMBER[step] && "bg-primary",
                  )}
                />
              ))}
            </div>
          </div>
          <AlertDialogTitle>{t(`chooser.${step}Title`)}</AlertDialogTitle>
          <AlertDialogDescription>{t(`chooser.${step}Desc`)}</AlertDialogDescription>
        </AlertDialogHeader>

        {step === "source" && !localProject && (
          <div className="grid gap-2 sm:grid-cols-2">
            <ChoiceCard
              icon={<Folder className="h-4 w-4" />}
              title={t("chooser.local")}
              description={t("chooser.localDesc")}
              disabled={busy}
              onClick={() => void chooseLocal()}
            />
            <ChoiceCard
              icon={<Cloud className="h-4 w-4" />}
              title={t("chooser.server")}
              description={t("chooser.serverSourceDesc")}
              onClick={() => setStep("server")}
            />
          </div>
        )}

        {step === "source" && localProject && (
          <div className="flex flex-col gap-3">
            <div className="rounded-lg border border-border bg-muted/25 p-3">
              <p className="m-0 text-sm font-semibold">{localProject.name}</p>
              <p className="mt-1 mb-0 break-all font-mono text-xs text-muted-foreground">
                {localProject.root}
              </p>
            </div>
            <div className="flex flex-col gap-2 sm:flex-row">
              <Button
                type="button"
                className="sm:flex-1"
                onClick={() => void openLocal(localProject.root)}
              >
                {t("chooser.openLocal")}
              </Button>
              {onRequestMigration && (
                <Button
                  type="button"
                  variant="outline"
                  className="sm:flex-1"
                  disabled={busy}
                  onClick={() => void migrateLocal(localProject.root)}
                >
                  {t("chooser.migrateToServer")}
                </Button>
              )}
            </div>
          </div>
        )}

        {step === "server" && (
          <div className="flex max-h-[360px] flex-col gap-3 overflow-y-auto pr-1">
            {loggedInConnections.length === 0 ? (
              <p className="m-0 rounded-md border border-dashed border-border p-3 text-sm text-muted-foreground">
                {t("chooser.noLoggedInServers")}
              </p>
            ) : (
              <div className="flex flex-col gap-2">
                {loggedInConnections.map((entry) => (
                  <ChoiceCard
                    key={entry.id}
                    icon={<Server className="h-4 w-4" />}
                    title={entry.name}
                    description={`${entry.origin}${entry.lastActorDisplay ? ` · ${entry.lastActorDisplay}` : ""}`}
                    onClick={() => void selectConnection(entry)}
                  />
                ))}
              </div>
            )}
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="self-start gap-1.5"
              onClick={() => setShowAddServer((shown) => !shown)}
            >
              <Plus className="h-3.5 w-3.5" /> {t("chooser.addServer")}
            </Button>
            {showAddServer && (
              <div className="flex flex-col gap-2 rounded-lg border border-border bg-muted/30 p-3">
                <Input
                  aria-label={t("servers.urlPlaceholder")}
                  placeholder={t("servers.urlPlaceholder")}
                  value={serverUrl}
                  onChange={(event) => setServerUrl(event.target.value)}
                />
                <Input
                  aria-label={t("servers.namePlaceholder")}
                  placeholder={t("servers.namePlaceholder")}
                  value={serverName}
                  onChange={(event) => setServerName(event.target.value)}
                />
                <Button type="button" size="sm" disabled={busy} onClick={() => void addServer()}>
                  {t("chooser.addAndLogin")}
                </Button>
              </div>
            )}
          </div>
        )}

        {step === "scopes" && (
          <ScopeSelection scopes={scopes} selected={scope} busy={busy} onSelect={setScope} />
        )}

        {step === "checkout" && scope && (
          <div className="flex flex-col gap-2">
            <ChoiceCard
              icon={<Cloud className="h-4 w-4" />}
              title={t("chooser.skipCheckout")}
              description={t("chooser.skipCheckoutDesc")}
              selected={checkoutMode === "skip"}
              onClick={() => {
                setCheckoutMode("skip");
                setCheckoutRoot(null);
                setCheckoutTools([]);
                setError(null);
              }}
            />
            <ChoiceCard
              icon={<Folder className="h-4 w-4" />}
              title={t("chooser.connectCheckout")}
              description={checkoutRoot ?? t("chooser.connectCheckoutDesc")}
              selected={checkoutMode === "folder"}
              disabled={busy}
              onClick={() => void chooseCheckoutFolder()}
            />
            {checkoutMode === "folder" && checkoutRoot && (
              <div className="flex flex-col gap-2 rounded-lg border border-border bg-muted/30 p-3">
                <span className="text-sm font-medium">{t("chooser.checkoutTools")}</span>
                <div className="flex gap-4">
                  {["claude", "codex"].map((tool) => (
                    <label
                      key={tool}
                      htmlFor={`checkout-tool-${tool}`}
                      className="flex items-center gap-1.5 text-sm"
                    >
                      <Checkbox
                        id={`checkout-tool-${tool}`}
                        aria-label={tool}
                        checked={checkoutTools.includes(tool)}
                        disabled={busy}
                        onCheckedChange={(v) => toggleCheckoutTool(tool, v === true)}
                      />
                      {tool}
                    </label>
                  ))}
                </div>
                <span className="text-xs leading-5 text-muted-foreground">
                  {t("chooser.checkoutToolsHelp")}
                </span>
              </div>
            )}
          </div>
        )}

        {(error || notice) && (
          <p
            role={error ? "alert" : "status"}
            className={cn(
              "m-0 rounded-md px-3 py-2 text-xs",
              error
                ? "bg-destructive/10 text-destructive"
                : "bg-primary/8 text-muted-foreground",
            )}
          >
            {error ?? notice}
          </p>
        )}

        <div className="flex items-center justify-between gap-3 border-t border-border pt-3">
          <div>
            {step !== "source" && (
              <Button type="button" variant="ghost" size="sm" className="gap-1" onClick={back}>
                <ArrowLeft className="h-3.5 w-3.5" /> {t("chooser.back")}
              </Button>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button type="button" variant="outline" size="sm" onClick={() => onOpenChange(false)}>
              {t("app.cancel")}
            </Button>
            {step === "scopes" && (
              <Button
                type="button"
                size="sm"
                disabled={!scope || busy}
                onClick={() => {
                  setCheckoutMode(null);
                  setCheckoutRoot(null);
                  setStep("checkout");
                }}
              >
                {t("chooser.next")}
              </Button>
            )}
            {step === "checkout" && (
              <Button
                type="button"
                size="sm"
                disabled={
                  checkoutMode === null ||
                  busy ||
                  (checkoutMode === "folder" && checkoutTools.length === 0)
                }
                onClick={() => void openRemote()}
              >
                {t("chooser.open")}
              </Button>
            )}
          </div>
        </div>
      </AlertDialogContent>
    </AlertDialog>
  );
}
