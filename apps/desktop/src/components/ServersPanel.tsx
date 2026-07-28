// 伺服器管理最小面（規格「伺服器管理最小面」；design 決策 7）：saved servers
// 清單（顯示名、origin、登入狀態與身分）、新增（URL＋顯示名→隨即登入）、登入
// （device 預設；明確不支援就地現 PAT 輸入）、登出、移除。app 全域、不經
// session 綁定；credential 全程不出 Rust——此面只見狀態與身分顯示名。
import { useEffect, useRef, useState } from "react";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Input,
  useI18n,
} from "@speclink/ui";

import type { ConnectionView } from "../adapter/connections";
import type { ConnectionPhase } from "../store";
import { AwaitingApproval, PatLoginInput } from "./connectionLogin";

export interface ServersPanelProps {
  connections: ConnectionView[];
  /** 逐連線互動狀態（keyed by origin）。 */
  phases: Record<string, ConnectionPhase>;
  /** 新增並隨即登入；無效輸入 reject、由表單就地呈現。 */
  onAdd: (baseUrl: string, name: string) => Promise<void>;
  onLogin: (origin: string) => void;
  /** 取消等待授權：停止輪詢、回未登入。 */
  onCancelLogin?: (origin: string) => void;
  /** PAT 單次過境：提交後本面即清空草稿。 */
  onSubmitPat: (origin: string, pat: string) => void;
  onLogout: (origin: string) => void;
  onRemove: (id: string) => void;
  /** 掛載時載入清單（頁籤切入才觸發）。 */
  onRefresh?: () => void;
  /** 開啟統一 chooser，預選此 server 並直達 scopes；未注入時入口不顯示。 */
  onOpenWorkspace?: (id: string) => void;
  /** needs-reauth 導向時即使 registry 仍標 loggedIn，也呈現並聚焦重新登入。 */
  focusConnectionId?: string | null;
}

/** 單列的狀態文字：互動狀態優先於登入狀態。 */
function StatusLine({
  entry,
  phase,
  needsReauth = false,
}: {
  entry: ConnectionView;
  phase: ConnectionPhase;
  needsReauth?: boolean;
}) {
  const { t } = useI18n();
  if (phase.kind === "busy") {
    return <span className="text-xs text-muted-foreground">{t("servers.busy")}</span>;
  }
  // 等待授權的細節由 AwaitingApproval 就地展開；狀態行只給一句標題。
  if (phase.kind === "awaitingApproval") {
    return <span className="text-xs text-muted-foreground">{t("servers.awaitingTitle")}</span>;
  }
  if (phase.kind === "error") {
    return (
      <span role="alert" className="text-xs text-red-600 dark:text-red-400">
        {phase.message}
      </span>
    );
  }
  if (phase.kind === "notice") {
    return <span className="text-xs text-muted-foreground">{phase.message}</span>;
  }
  if (needsReauth) {
    return (
      <span className="text-xs text-amber-700 dark:text-amber-300">
        {t("remote.reauthTitle")}
      </span>
    );
  }
  if (entry.loggedIn) {
    return (
      <span className="text-xs text-teal-700 dark:text-teal-400">
        {t("servers.loggedIn")}
        {entry.lastActorDisplay ? ` · ${entry.lastActorDisplay}` : ""}
      </span>
    );
  }
  return <span className="text-xs text-muted-foreground">{t("servers.notLoggedIn")}</span>;
}

export function ServersPanel({
  connections,
  phases,
  onAdd,
  onLogin,
  onCancelLogin,
  onSubmitPat,
  onLogout,
  onRemove,
  onRefresh,
  onOpenWorkspace,
  focusConnectionId = null,
}: ServersPanelProps) {
  const { t } = useI18n();
  const [url, setUrl] = useState("");
  const [name, setName] = useState("");
  const [addError, setAddError] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const reauthButton = useRef<HTMLButtonElement | null>(null);
  /** 剛登入成功那一列的開啟工作區鈕（規格「登入成功聚焦開啟工作區」）。 */
  const openWorkspaceButton = useRef<HTMLButtonElement | null>(null);
  /** 上一輪已登入的 origins；null＝尚未初始化。初次渲染只記錄不聚焦——切入
   * 頁籤看見既有已登入條目不該被搶走焦點，聚焦只給「剛完成的登入」。 */
  const loggedInBefore = useRef<Set<string> | null>(null);
  const [justLoggedIn, setJustLoggedIn] = useState<string | null>(null);
  useEffect(() => {
    onRefresh?.();
  }, [onRefresh]);
  useEffect(() => {
    reauthButton.current?.focus();
  }, [focusConnectionId, connections]);
  useEffect(() => {
    const now = new Set(connections.filter((e) => e.loggedIn).map((e) => e.origin));
    const before = loggedInBefore.current;
    loggedInBefore.current = now;
    if (!before) return;
    const fresh = [...now].find((origin) => !before.has(origin));
    if (fresh) setJustLoggedIn(fresh);
  }, [connections]);
  useEffect(() => {
    if (justLoggedIn) openWorkspaceButton.current?.focus();
  }, [justLoggedIn]);

  async function submitAdd() {
    const trimmed = url.trim();
    if (!trimmed) return;
    setAdding(true);
    setAddError(null);
    try {
      await onAdd(trimmed, name.trim() || trimmed);
      setUrl("");
      setName("");
    } catch (e) {
      setAddError(String(e));
    } finally {
      setAdding(false);
    }
  }

  return (
    <Card data-testid="servers-card">
      <CardHeader>
        <CardTitle className="text-base">{t("servers.title")}</CardTitle>
      </CardHeader>
      <CardContent className="gap-4">
        <p className="text-xs text-muted-foreground m-0">{t("servers.help")}</p>

        {connections.length === 0 ? (
          <p className="text-sm text-muted-foreground m-0" data-testid="servers-empty">
            {t("servers.empty")}
          </p>
        ) : (
          <ul className="m-0 flex list-none flex-col gap-2 p-0">
            {connections.map((entry) => {
              const phase = phases[entry.origin] ?? { kind: "idle" as const };
              const busy = phase.kind === "busy";
              const needsReauth = entry.id === focusConnectionId;
              return (
                <li
                  key={entry.id}
                  data-testid={`server-row-${entry.origin}`}
                  className="flex flex-col gap-1.5 rounded-md border border-border p-3"
                >
                  <div className="flex items-center justify-between gap-3">
                    <div className="min-w-0">
                      <div className="truncate text-sm font-medium">{entry.name}</div>
                      <div className="truncate font-mono text-xs text-muted-foreground">
                        {entry.origin}
                      </div>
                    </div>
                    <div className="flex shrink-0 items-center gap-1.5">
                      {entry.loggedIn && onOpenWorkspace && (
                        <Button
                          ref={
                            entry.origin === justLoggedIn ? openWorkspaceButton : undefined
                          }
                          type="button"
                          size="sm"
                          disabled={busy}
                          data-testid={`open-workspace-${entry.origin}`}
                          onClick={() => onOpenWorkspace(entry.id)}
                        >
                          {t("servers.openWorkspace")}
                        </Button>
                      )}
                      {entry.loggedIn && !needsReauth ? (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          disabled={busy}
                          onClick={() => onLogout(entry.origin)}
                        >
                          {t("servers.logout")}
                        </Button>
                      ) : (
                        <Button
                          ref={needsReauth ? reauthButton : undefined}
                          type="button"
                          size="sm"
                          disabled={busy}
                          data-testid={needsReauth ? `reauth-login-${entry.id}` : undefined}
                          onClick={() => onLogin(entry.origin)}
                        >
                          {t("servers.login")}
                        </Button>
                      )}
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={busy}
                        className="text-muted-foreground hover:text-foreground"
                        onClick={() => onRemove(entry.id)}
                      >
                        {t("servers.remove")}
                      </Button>
                    </div>
                  </div>
                  <StatusLine entry={entry} phase={phase} needsReauth={needsReauth} />
                  {phase.kind === "awaitingApproval" && (
                    <AwaitingApproval
                      origin={entry.origin}
                      phase={phase}
                      onCancel={() => onCancelLogin?.(entry.origin)}
                    />
                  )}
                  {phase.kind === "patInput" && (
                    <div data-testid={`pat-input-${entry.origin}`}>
                      <PatLoginInput
                        error={phase.error}
                        onSubmit={(pat) => onSubmitPat(entry.origin, pat)}
                      />
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        )}

        <div className="flex flex-col gap-1.5" data-testid="server-add-form">
          <div className="text-sm font-medium">{t("servers.addTitle")}</div>
          <div className="flex items-center gap-1.5">
            <Input
              placeholder={t("servers.urlPlaceholder")}
              value={url}
              onChange={(e) => setUrl(e.target.value)}
            />
            <Input
              placeholder={t("servers.namePlaceholder")}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            <Button type="button" size="sm" disabled={adding} onClick={() => void submitAdd()}>
              {t("servers.add")}
            </Button>
          </div>
          {addError && (
            <span role="alert" className="text-xs text-red-600 dark:text-red-400">
              {addError}
            </span>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
