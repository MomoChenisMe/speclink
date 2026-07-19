// 伺服器管理最小面（規格「伺服器管理最小面」；design 決策 7）：saved servers
// 清單（顯示名、origin、登入狀態與身分）、新增（URL＋顯示名→隨即登入）、登入
// （device 預設；明確不支援就地現 PAT 輸入）、登出、移除。app 全域、不經
// session 綁定；credential 全程不出 Rust——此面只見狀態與身分顯示名。
import { useEffect, useState } from "react";
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

export interface ServersPanelProps {
  connections: ConnectionView[];
  /** 逐連線互動狀態（keyed by origin）。 */
  phases: Record<string, ConnectionPhase>;
  /** 新增並隨即登入；無效輸入 reject、由表單就地呈現。 */
  onAdd: (baseUrl: string, name: string) => Promise<void>;
  onLogin: (origin: string) => void;
  /** PAT 單次過境：提交後本面即清空草稿。 */
  onSubmitPat: (origin: string, pat: string) => void;
  onLogout: (origin: string) => void;
  onRemove: (id: string) => void;
  /** 掛載時載入清單（頁籤切入才觸發）。 */
  onRefresh?: () => void;
  /** 開啟 remote workspace（remote-data-source 決策 6）：以 workspace 識別
   * handshake，失敗 reject、就地呈現 server 錯誤；未注入時入口不顯示。 */
  onOpenWorkspace?: (id: string, target: string) => Promise<void>;
}

/** 單列的狀態文字：互動狀態優先於登入狀態。 */
function StatusLine({ entry, phase }: { entry: ConnectionView; phase: ConnectionPhase }) {
  const { t } = useI18n();
  if (phase.kind === "busy") {
    return <span className="text-xs text-muted-foreground">{t("servers.busy")}</span>;
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
  onSubmitPat,
  onLogout,
  onRemove,
  onRefresh,
  onOpenWorkspace,
}: ServersPanelProps) {
  const { t } = useI18n();
  const [url, setUrl] = useState("");
  const [name, setName] = useState("");
  const [addError, setAddError] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  /** PAT 草稿（keyed by origin）：僅存在提交前的元件狀態，提交即清空。 */
  const [patDrafts, setPatDrafts] = useState<Record<string, string>>({});
  /** 開啟 workspace 的就地表單（keyed by connection id）：展開態、識別草稿與
   * handshake 失敗的就地錯誤（fail-closed——不建分頁、錯誤原樣呈現）。 */
  const [workspaceForms, setWorkspaceForms] = useState<
    Record<string, { open: boolean; draft: string; error: string | null; busy: boolean }>
  >({});

  function workspaceForm(id: string) {
    return workspaceForms[id] ?? { open: false, draft: "", error: null, busy: false };
  }

  function patchWorkspaceForm(
    id: string,
    patch: Partial<{ open: boolean; draft: string; error: string | null; busy: boolean }>,
  ) {
    setWorkspaceForms((forms) => ({ ...forms, [id]: { ...workspaceForm(id), ...patch } }));
  }

  async function submitWorkspace(id: string) {
    const target = workspaceForm(id).draft.trim();
    if (!target || !onOpenWorkspace) return;
    patchWorkspaceForm(id, { busy: true, error: null });
    try {
      await onOpenWorkspace(id, target);
      patchWorkspaceForm(id, { open: false, draft: "", busy: false });
    } catch (e) {
      patchWorkspaceForm(id, { error: String(e), busy: false });
    }
  }

  useEffect(() => {
    onRefresh?.();
  }, [onRefresh]);

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

  function submitPat(origin: string) {
    const pat = (patDrafts[origin] ?? "").trim();
    if (!pat) return;
    // 單次過境：先清草稿再送出，元件不留任何拷貝。
    setPatDrafts((drafts) => ({ ...drafts, [origin]: "" }));
    onSubmitPat(origin, pat);
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
                          type="button"
                          size="sm"
                          disabled={busy}
                          data-testid={`open-workspace-${entry.origin}`}
                          onClick={() =>
                            patchWorkspaceForm(entry.id, {
                              open: !workspaceForm(entry.id).open,
                              error: null,
                            })
                          }
                        >
                          {t("servers.openWorkspace")}
                        </Button>
                      )}
                      {entry.loggedIn ? (
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
                          type="button"
                          size="sm"
                          disabled={busy}
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
                  <StatusLine entry={entry} phase={phase} />
                  {entry.loggedIn && onOpenWorkspace && workspaceForm(entry.id).open && (
                    <div
                      className="flex flex-col gap-1.5"
                      data-testid={`workspace-form-${entry.origin}`}
                    >
                      <p className="text-xs text-muted-foreground m-0">
                        {t("servers.workspaceHint")}
                      </p>
                      <div className="flex items-center gap-1.5">
                        <Input
                          placeholder={t("servers.workspacePlaceholder")}
                          value={workspaceForm(entry.id).draft}
                          onChange={(e) =>
                            patchWorkspaceForm(entry.id, { draft: e.target.value })
                          }
                        />
                        <Button
                          type="button"
                          size="sm"
                          disabled={workspaceForm(entry.id).busy}
                          onClick={() => void submitWorkspace(entry.id)}
                        >
                          {t("servers.workspaceSubmit")}
                        </Button>
                      </div>
                      {workspaceForm(entry.id).error && (
                        <span role="alert" className="text-xs text-red-600 dark:text-red-400">
                          {workspaceForm(entry.id).error}
                        </span>
                      )}
                    </div>
                  )}
                  {phase.kind === "patInput" && (
                    <div className="flex flex-col gap-1.5" data-testid={`pat-input-${entry.origin}`}>
                      <p className="text-xs text-muted-foreground m-0">{t("servers.patHint")}</p>
                      <div className="flex items-center gap-1.5">
                        <Input
                          type="password"
                          placeholder={t("servers.patPlaceholder")}
                          value={patDrafts[entry.origin] ?? ""}
                          onChange={(e) =>
                            setPatDrafts((drafts) => ({
                              ...drafts,
                              [entry.origin]: e.target.value,
                            }))
                          }
                        />
                        <Button type="button" size="sm" onClick={() => submitPat(entry.origin)}>
                          {t("servers.patSubmit")}
                        </Button>
                      </div>
                      {phase.error && (
                        <span role="alert" className="text-xs text-red-600 dark:text-red-400">
                          {phase.error}
                        </span>
                      )}
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
