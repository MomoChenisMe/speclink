// 伺服器管理最小面（規格「伺服器管理最小面」；desktop-settings-mode design D7）：頁頂的新增卡
// （URL＋顯示名→隨即登入，成功即收起）與「連線」卡——一列一個 saved server：顯示名＋狀態籤、
// 等寬 origin、身分；右側登出或登入／重新登入＋「⋯」（開啟專案、移除）；等待授權與 PAT 輸入
// 於列下就地展開。app 全域、不經 session 綁定；credential 全程不出 Rust——此面只見狀態與身分。
import { useEffect, useRef, useState } from "react";
import { MoreHorizontal } from "lucide-react";
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Input,
  SEMANTIC_TONE,
  SettingsCard,
  SettingsRow,
  StatusPill,
  useI18n,
  type StatusPillLabel,
} from "@speclink/ui";

import type { ConnectionView } from "../adapter/connections";
import type { ConnectionPhase } from "../store";
import { AwaitingApproval, PatLoginInput } from "./connectionLogin";

export interface ServersPanelProps {
  connections: ConnectionView[];
  /** 逐連線互動狀態（keyed by origin）。 */
  phases: Record<string, ConnectionPhase>;
  /** 新增並隨即登入；無效輸入 reject、由表單就地呈現。 */
  onAdd: (baseUrl: string, name: string) => Promise<unknown>;
  onLogin: (origin: string) => void;
  /** 取消等待授權：停止輪詢、回未登入。 */
  onCancelLogin?: (origin: string) => void;
  /** PAT 單次過境：提交後本面即清空草稿。 */
  onSubmitPat: (origin: string, pat: string) => void;
  onLogout: (origin: string) => void;
  onRemove: (id: string) => void;
  /** 掛載時載入清單（進伺服器頁才觸發）。 */
  onRefresh?: () => void;
  /** 開啟統一 chooser，預選此 server 並直達 scopes；未注入時入口不顯示。 */
  onOpenWorkspace?: (id: string) => void;
  /** needs-reauth 導向時即使 registry 仍標 loggedIn，也呈現並聚焦重新登入。 */
  focusConnectionId?: string | null;
  /** 開著的 remote session 回報需重新登入的連線：registry 仍標 loggedIn，也呈現登入過期與重新登入（不搶焦點）。 */
  expiredConnectionIds?: readonly string[];
  /** 頁頂新增卡是否展開（伺服器頁標題右側「新增連線」切換）。 */
  addOpen?: boolean;
  onAddOpenChange?: (open: boolean) => void;
}

/** 狀態籤：互動狀態優先於登入狀態。 */
function statusPill(entry: ConnectionView, phase: ConnectionPhase, needsReauth: boolean): StatusPillLabel {
  if (phase.kind === "busy") return { key: "servers.busy", tone: "neutral" };
  if (phase.kind === "awaitingApproval") return { key: "servers.page.pill.awaiting", tone: "progress" };
  if (phase.kind === "error") return { key: "servers.page.pill.failed", tone: "destructive" };
  if (needsReauth) return { key: "servers.page.pill.expired", tone: "warning" };
  if (entry.loggedIn) return { key: "servers.loggedIn", tone: "success" };
  return { key: "servers.notLoggedIn", tone: "neutral" };
}

function AddConnectionCard({
  onAdd,
  onClose,
}: {
  onAdd: ServersPanelProps["onAdd"];
  onClose: () => void;
}) {
  const { t } = useI18n();
  const [url, setUrl] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);

  async function submit() {
    const trimmed = url.trim();
    if (!trimmed) return;
    setAdding(true);
    setError(null);
    try {
      await onAdd(trimmed, name.trim() || trimmed);
      onClose();
    } catch (e) {
      setError(String(e));
      setAdding(false);
    }
  }

  return (
    <SettingsCard title={t("servers.addTitle")}>
      <div className="flex flex-col gap-2 px-4 py-3" data-testid="server-add-form">
        <div className="flex items-center gap-1.5">
          <Input placeholder={t("servers.urlPlaceholder")} value={url} onChange={(e) => setUrl(e.target.value)} />
          <Input placeholder={t("servers.namePlaceholder")} value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div className="flex items-center justify-end gap-1.5">
          {error && (
            <span role="alert" className={`mr-auto text-xs ${SEMANTIC_TONE.danger}`}>
              {error}
            </span>
          )}
          <Button type="button" variant="ghost" size="sm" onClick={onClose}>
            {t("app.cancel")}
          </Button>
          <Button type="button" size="sm" disabled={adding} onClick={() => void submit()}>
            {t("servers.add")}
          </Button>
        </div>
      </div>
    </SettingsCard>
  );
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
  expiredConnectionIds = [],
  addOpen = false,
  onAddOpenChange,
}: ServersPanelProps) {
  const { t } = useI18n();
  const reauthButton = useRef<HTMLButtonElement | null>(null);
  /** 剛登入成功那一列的開啟專案鈕（規格「登入成功聚焦開啟工作區」）。 */
  const openWorkspaceButton = useRef<HTMLButtonElement | null>(null);
  /** 上一輪已登入的 origins；null＝尚未初始化。初次渲染只記錄不聚焦——進頁看見既有已登入
   * 條目不該被搶走焦點，聚焦與行動呼籲只給「剛完成的登入」。 */
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

  return (
    <>
      {addOpen && <AddConnectionCard onAdd={onAdd} onClose={() => onAddOpenChange?.(false)} />}
      <SettingsCard title={t("servers.page.connectionsCard")} count={connections.length}>
        {connections.length === 0 ? (
          <p className="m-0 px-4 py-3 text-sm text-muted-foreground" data-testid="servers-empty">
            {t("servers.empty")}
          </p>
        ) : (
          connections.map((entry) => {
            const phase = phases[entry.origin] ?? { kind: "idle" as const };
            const busy = phase.kind === "busy";
            const needsReauth = entry.id === focusConnectionId || expiredConnectionIds.includes(entry.id);
            const pill = statusPill(entry, phase, needsReauth);
            const canOpen = entry.loggedIn && onOpenWorkspace !== undefined;
            return (
              <div key={entry.id} data-testid={`server-row-${entry.origin}`}>
                <SettingsRow
                  label={
                    <span className="flex min-w-0 items-center gap-2">
                      <span className="truncate font-medium">{entry.name}</span>
                      <StatusPill tone={pill.tone}>{t(pill.key)}</StatusPill>
                    </span>
                  }
                  description={
                    <span className="flex min-w-0 flex-col">
                      <span className="truncate font-mono">{entry.origin}</span>
                      {entry.lastActorDisplay && <span className="truncate">{entry.lastActorDisplay}</span>}
                    </span>
                  }
                  control={
                    <>
                      {/* 登入成功的行動呼籲：只給剛完成登入的那一列，並取得鍵盤焦點。 */}
                      {canOpen && entry.origin === justLoggedIn && (
                        <Button
                          ref={openWorkspaceButton}
                          type="button"
                          size="sm"
                          disabled={busy}
                          data-testid={`open-workspace-${entry.origin}`}
                          onClick={() => onOpenWorkspace!(entry.id)}
                        >
                          {t("servers.openProject")}
                        </Button>
                      )}
                      {entry.loggedIn && !needsReauth ? (
                        <Button type="button" variant="outline" size="sm" disabled={busy} onClick={() => onLogout(entry.origin)}>
                          {t("servers.logout")}
                        </Button>
                      ) : (
                        <Button
                          ref={entry.id === focusConnectionId ? reauthButton : undefined}
                          type="button"
                          size="sm"
                          disabled={busy}
                          data-testid={needsReauth ? `reauth-login-${entry.id}` : undefined}
                          onClick={() => onLogin(entry.origin)}
                        >
                          {needsReauth ? t("remote.reauthAction") : t("servers.login")}
                        </Button>
                      )}
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            type="button"
                            variant="subtle"
                            size="icon-sm"
                            disabled={busy}
                            aria-label={t("servers.page.moreActions")}
                          >
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          {canOpen && (
                            <DropdownMenuItem onSelect={() => onOpenWorkspace!(entry.id)}>
                              {t("servers.openProject")}
                            </DropdownMenuItem>
                          )}
                          <DropdownMenuItem variant="destructive" onSelect={() => onRemove(entry.id)}>
                            {t("servers.remove")}
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </>
                  }
                >
                  {(phase.kind === "error" || phase.kind === "notice" || phase.kind === "awaitingApproval" || phase.kind === "patInput") && (
                    <div className="flex flex-col gap-2">
                      {phase.kind === "error" && (
                        <span role="alert" className={`text-xs ${SEMANTIC_TONE.danger}`}>
                          {phase.message}
                        </span>
                      )}
                      {phase.kind === "notice" && <span className="text-xs text-muted-foreground">{phase.message}</span>}
                      {phase.kind === "awaitingApproval" && (
                        <AwaitingApproval origin={entry.origin} phase={phase} onCancel={() => onCancelLogin?.(entry.origin)} />
                      )}
                      {phase.kind === "patInput" && (
                        <div data-testid={`pat-input-${entry.origin}`}>
                          <PatLoginInput error={phase.error} onSubmit={(pat) => onSubmitPat(entry.origin, pat)} />
                        </div>
                      )}
                    </div>
                  )}
                </SettingsRow>
              </div>
            );
          })
        )}
      </SettingsCard>
    </>
  );
}
