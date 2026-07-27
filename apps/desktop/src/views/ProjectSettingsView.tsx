// 專案設定頁（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」）：
// 兩頁簽組織——config.yaml（專案說明／產出規則／產出政策）、
// .speclink.yaml（AI 工具）。頁簽標籤檔名直出（字面常數，
// LANGUAGE.md 明文例外）；專案說明與產出規則為獨立卡各持編輯態；解析失敗掛簽級
// 警示點與簽首橫幅。欄位旁說明文字承接被 Mapping 讀-改-寫移除的範本註解教學角色。
import { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Checkbox,
  Markdown,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Textarea,
  cn,
  useI18n,
} from "@speclink/ui";

import type { SettingsSnapshot, WorkflowFields } from "../adapter/workspace";
import type { WorkspaceSettingsProvider } from "../session";

/** locale／spec_locale 的「未設定」在 config 裡是空字串，Radix Select 的 item 不接受空字串。 */
const LOCALE_UNSET = "__unset__";

type PendingRemoteWrite =
  | { kind: "policy"; fields: WorkflowFields }
  | { kind: "context"; context: string }
  | { kind: "rules"; rules: Array<[string, string[]]> };

interface ConflictRow {
  key: string;
  label: string;
  server: string;
  mine: string;
}

interface PolicyConflict {
  latest: SettingsSnapshot;
  pending: PendingRemoteWrite;
  rows: ConflictRow[];
}

function remoteErrorReason(error: unknown): string | null {
  if (typeof error === "object" && error !== null && "reason" in error) {
    const reason = (error as { reason?: unknown }).reason;
    return typeof reason === "string" ? reason : null;
  }
  if (typeof error === "string") {
    try {
      return remoteErrorReason(JSON.parse(error));
    } catch {
      return null;
    }
  }
  return null;
}

function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return String(error);
}

/** 行↔條目轉換（design D2）：一行一條規則——儲存時逐行修剪頭尾空白、空行滌除，行序即寫入順序。 */
const entriesToText = (entries: string[]) => entries.join("\n");
const textToEntries = (text: string) =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");

/** 收合門檻（design D3）：唯讀 markdown 超長截斷——渲染高度無法在 jsdom 量測，以原文規模判定。 */
const isLongContext = (text: string) => text.split("\n").length > 12 || text.length > 1200;

export interface ProjectSettingsViewProps {
  /** 活躍 session 的設定面（root 已綁定；workspace-session 決策 3）。 */
  settings: WorkspaceSettingsProvider;
}

function FieldHelp({ children }: { children: React.ReactNode }) {
  return <p className="text-xs text-muted-foreground m-0">{children}</p>;
}

function ParseErrorBanner({ message }: { message: string }) {
  const { t } = useI18n();
  return (
    <p role="alert" className="flex items-start gap-1.5 text-xs text-amber-600 dark:text-amber-400 m-0">
      <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
      <span>
        {t("settings.parseErrorHint")} {message}
      </span>
    </p>
  );
}

/** 頁簽標籤警示點（design D3）：解析失敗時未切至該簽也可見。 */
function TabWarningDot() {
  return (
    <span
      data-testid="tab-warning"
      aria-hidden="true"
      className="ml-1.5 inline-block h-1.5 w-1.5 rounded-full bg-amber-500"
    />
  );
}

/** 卡片右上編輯／取消／儲存按鈕列（兩卡共用的框架片段）。 */
function CardEditControls({
  editing,
  disabled,
  msg,
  testPrefix,
  onEdit,
  onCancel,
  onSave,
}: {
  editing: boolean;
  disabled: boolean;
  msg: string | null;
  /** data-testid 前綴（context／rules）。 */
  testPrefix: string;
  onEdit: () => void;
  onCancel: () => void;
  onSave: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="flex items-center gap-2">
      {msg && <span className="text-xs text-muted-foreground">{msg}</span>}
      {editing ? (
        <>
          <Button
            type="button"
            variant="outline"
            size="sm"
            data-testid={`${testPrefix}-cancel`}
            disabled={disabled}
            className="text-sm font-normal text-muted-foreground hover:text-foreground"
            onClick={onCancel}
          >
            {t("app.cancel")}
          </Button>
          <Button
            type="button"
            size="sm"
            data-testid={`${testPrefix}-save`}
            disabled={disabled}
            className="text-sm"
            onClick={onSave}
          >
            {t("settings.save")}
          </Button>
        </>
      ) : (
        <Button
          type="button"
          variant="outline"
          size="sm"
          data-testid={`${testPrefix}-edit`}
          disabled={disabled}
          className="text-sm font-normal text-muted-foreground hover:text-foreground"
          onClick={onEdit}
        >
          {t("settings.edit")}
        </Button>
      )}
    </div>
  );
}

export function ProjectSettingsView({ settings }: ProjectSettingsViewProps) {
  const { t } = useI18n();
  const [snap, setSnap] = useState<SettingsSnapshot | null>(null);
  const [tools, setTools] = useState<string[]>([]);
  const [locale, setLocale] = useState("");
  const [specLocale, setSpecLocale] = useState("");
  const [tdd, setTdd] = useState(false);
  const [audit, setAudit] = useState(false);
  const [contextText, setContextText] = useState("");
  /** 產出規則現值：schemaArtifacts 固定鍵 → 條目清單（清單順序即檔案順序）。 */
  const [rules, setRules] = useState<Record<string, string[]>>({});
  // 拆卡獨立編輯態（design D2）：專案說明卡與產出規則卡各持 editing 旗標與草稿，
  // 編輯／取消／儲存互不影響；各卡儲存僅寫對應鍵。
  const [ctxEditing, setCtxEditing] = useState(false);
  const [draftContext, setDraftContext] = useState("");
  const [ctxMsg, setCtxMsg] = useState<string | null>(null);
  const [rulesEditing, setRulesEditing] = useState(false);
  const [draftRules, setDraftRules] = useState<Record<string, string>>({});
  const [rulesMsg, setRulesMsg] = useState<string | null>(null);
  const [contextExpanded, setContextExpanded] = useState(false);
  const [appMsg, setAppMsg] = useState<string | null>(null);
  const [wfMsg, setWfMsg] = useState<string | null>(null);
  const [conflict, setConflict] = useState<PolicyConflict | null>(null);

  const hydrate = (next: SettingsSnapshot) => {
    setSnap(next);
    setTools(next.app.tools);
    setLocale(next.workflow.locale ?? "");
    setSpecLocale(next.workflow.specLocale ?? "");
    setTdd(next.workflow.tdd);
    setAudit(next.workflow.audit);
    setContextText(next.workflow.context ?? "");
    const nextRules = Object.fromEntries(
      next.workflow.schemaArtifacts.map((id) => [id, next.workflow.rules[id] ?? []]),
    );
    setRules(nextRules);
    setDraftContext(next.workflow.context ?? "");
    setDraftRules(
      Object.fromEntries(
        next.workflow.schemaArtifacts.map((id) => [
          id,
          entriesToText(next.workflow.rules[id] ?? []),
        ]),
      ),
    );
    setCtxEditing(false);
    setRulesEditing(false);
    setContextExpanded(false);
  };

  useEffect(() => {
    void settings.readSettings().then(hydrate);
  }, [settings]);

  if (!snap) return null;

  const isRemote = settings.kind === "remote";
  const appDisabled = snap.app.parseError !== null;
  const wfDisabled =
    snap.workflow.parseError !== null ||
    (isRemote && !settings.policyWrite) ||
    conflict !== null;

  const adoptRevision = (next: number | void) => {
    if (typeof next !== "number") return;
    setSnap((current) =>
      current === null
        ? current
        : { ...current, workflow: { ...current.workflow, revision: next } },
    );
  };

  const toggleTool = (tool: string, on: boolean) =>
    setTools((prev) => (on ? [...prev.filter((x) => x !== tool), tool] : prev.filter((x) => x !== tool)));

  const saveApp = async () => {
    try {
      await settings.writeAppTools(tools);
      setAppMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤（指明檔案與階段，來自 desktop-core），表單維持原值。
      setAppMsg(errorMessage(e));
    }
  };

  const buildConflictRows = (
    latest: SettingsSnapshot,
    pending: PendingRemoteWrite,
  ): ConflictRow[] => {
    const policy =
      pending.kind === "policy"
        ? pending.fields
        : {
            locale: locale || null,
            specLocale: specLocale || null,
            tdd,
            audit,
          };
    const mineContext =
      pending.kind === "context" ? pending.context : ctxEditing ? draftContext : contextText;
    const artifactIds = Array.from(
      new Set([...snap.workflow.schemaArtifacts, ...latest.workflow.schemaArtifacts]),
    );
    const pendingRules = pending.kind === "rules" ? Object.fromEntries(pending.rules) : null;
    const show = (value: string | null | undefined) => value || "—";
    const rows: ConflictRow[] = [
      {
        key: "context",
        label: t("settings.contextLabel"),
        server: show(latest.workflow.context),
        mine: show(mineContext),
      },
      {
        key: "locale",
        label: "locale",
        server: show(latest.workflow.locale),
        mine: show(policy.locale),
      },
      {
        key: "spec-locale",
        label: "spec_locale",
        server: show(latest.workflow.specLocale),
        mine: show(policy.specLocale),
      },
      {
        key: "tdd",
        label: "tdd",
        server: String(latest.workflow.tdd),
        mine: String(policy.tdd),
      },
      {
        key: "audit",
        label: "audit",
        server: String(latest.workflow.audit),
        mine: String(policy.audit),
      },
    ];
    for (const id of artifactIds) {
      const mine = rulesEditing
        ? draftRules[id] ?? ""
        : entriesToText(pendingRules?.[id] ?? rules[id] ?? []);
      rows.push({
        key: `rules-${id}`,
        label: `rules.${id}`,
        server: show(entriesToText(latest.workflow.rules[id] ?? [])),
        mine: show(mine),
      });
    }
    return rows;
  };

  const handleWriteError = async (
    error: unknown,
    pending: PendingRemoteWrite,
  ): Promise<boolean> => {
    if (!isRemote || remoteErrorReason(error) !== "revision_conflict") return false;
    try {
      const latest = await settings.readSettings();
      setConflict({ latest, pending, rows: buildConflictRows(latest, pending) });
    } catch (readError) {
      const message = errorMessage(readError);
      if (pending.kind === "policy") setWfMsg(message);
      else if (pending.kind === "context") setCtxMsg(message);
      else setRulesMsg(message);
    }
    return true;
  };

  const saveWorkflow = async () => {
    const fields: WorkflowFields = {
      locale: locale || null,
      specLocale: specLocale || null,
      tdd,
      audit,
    };
    try {
      const next = await settings.writeWorkflowConfig(fields);
      adoptRevision(next);
      setWfMsg(t("settings.saved"));
    } catch (e) {
      if (await handleWriteError(e, { kind: "policy", fields })) setWfMsg(null);
      else setWfMsg(errorMessage(e));
    }
  };

  const beginCtxEdit = () => {
    setDraftContext(contextText);
    setCtxMsg(null);
    setCtxEditing(true);
  };

  const saveContext = async () => {
    // 僅寫 context 鍵（清空＝移除鍵）；產出規則卡對應的 rules 鍵不觸碰。
    try {
      const next = await settings.writeWorkflowContext(draftContext);
      adoptRevision(next);
      setContextText(draftContext);
      setContextExpanded(false);
      setCtxEditing(false);
      setCtxMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤，維持編輯態不遺失輸入。
      if (await handleWriteError(e, { kind: "context", context: draftContext })) setCtxMsg(null);
      else setCtxMsg(errorMessage(e));
    }
  };

  const beginRulesEdit = () => {
    setDraftRules(
      Object.fromEntries(
        snap.workflow.schemaArtifacts.map((id) => [id, entriesToText(rules[id] ?? [])]),
      ),
    );
    setRulesMsg(null);
    setRulesEditing(true);
  };

  const saveRules = async () => {
    // 整份代換 payload：全部固定鍵依 schemaArtifacts 順序送出（空節＝移除鍵、全空＝移除 rules 鍵）；
    // 僅寫 rules 鍵，context 逐字元不變（不經寫入路徑）。
    const nextRules: Array<[string, string[]]> = snap.workflow.schemaArtifacts.map((id) => [
      id,
      textToEntries(draftRules[id] ?? ""),
    ]);
    try {
      const next = await settings.writeWorkflowRules(nextRules);
      adoptRevision(next);
      setRules(Object.fromEntries(nextRules));
      setRulesEditing(false);
      setRulesMsg(t("settings.saved"));
    } catch (e) {
      if (await handleWriteError(e, { kind: "rules", rules: nextRules })) setRulesMsg(null);
      else setRulesMsg(errorMessage(e));
    }
  };

  const reloadServerVersion = () => {
    if (conflict === null) return;
    hydrate(conflict.latest);
    setConflict(null);
    setCtxMsg(null);
    setRulesMsg(null);
    setWfMsg(null);
  };

  const retryConflict = async () => {
    if (conflict === null) return;
    const pending = conflict.pending;
    try {
      let next: number | void;
      if (pending.kind === "policy") {
        next = await settings.writeWorkflowConfig(pending.fields);
        setWfMsg(t("settings.saved"));
      } else if (pending.kind === "context") {
        next = await settings.writeWorkflowContext(pending.context);
        setContextText(pending.context);
        setContextExpanded(false);
        setCtxEditing(false);
        setCtxMsg(t("settings.saved"));
      } else {
        next = await settings.writeWorkflowRules(pending.rules);
        setRules(Object.fromEntries(pending.rules));
        setRulesEditing(false);
        setRulesMsg(t("settings.saved"));
      }
      adoptRevision(next);
      setConflict(null);
    } catch (error) {
      if (!(await handleWriteError(error, pending))) {
        const message = errorMessage(error);
        if (pending.kind === "policy") setWfMsg(message);
        else if (pending.kind === "context") setCtxMsg(message);
        else setRulesMsg(message);
      }
    }
  };

  const contextCollapsed = !ctxEditing && !contextExpanded && isLongContext(contextText);
  const populatedRuleKeys = snap.workflow.schemaArtifacts.filter((id) => (rules[id] ?? []).length > 0);

  return (
    <div className="max-w-2xl mx-auto w-full">
      {/* 兩頁簽：標籤檔名直出為字面常數（LANGUAGE.md 明文例外）。 */}
      <Tabs defaultValue="config">
        <TabsList>
          <TabsTrigger value="config">
            {isRemote ? t("remote.workflowTab") : "config.yaml"}
            {wfDisabled && <TabWarningDot />}
          </TabsTrigger>
          {!isRemote && (
            <TabsTrigger value="speclink">
              .speclink.yaml
              {appDisabled && <TabWarningDot />}
            </TabsTrigger>
          )}
        </TabsList>

        {/* config.yaml 簽：專案說明／產出規則／產出政策 */}
        <TabsContent value="config" className="pt-3 flex flex-col gap-4">
          {isRemote ? (
            <div className="flex flex-col gap-2">
              <span
                data-testid="policy-revision"
                className="font-mono text-xs tracking-wide text-muted-foreground"
              >
                {t("remote.policyRevision")} {snap.workflow.revision ?? "—"}
              </span>
              {!settings.policyWrite && (
                <p
                  data-testid="policy-reader-note"
                  className="m-0 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm text-muted-foreground"
                >
                  {t("remote.policyReader")}
                </p>
              )}
            </div>
          ) : (
            <span data-testid="file-note-config" className="font-mono text-xs text-muted-foreground">
              openspec/config.yaml
            </span>
          )}
          {snap.workflow.parseError !== null && <ParseErrorBanner message={snap.workflow.parseError} />}
          {conflict !== null && (
            <Card data-testid="policy-conflict-panel" className="border-primary/30 bg-muted/20">
              <CardHeader className="gap-1">
                <CardTitle className="text-base">{t("remote.conflictTitle")}</CardTitle>
                <p className="m-0 text-xs text-muted-foreground">{t("remote.conflictHint")}</p>
                <span
                  data-testid="conflict-revision"
                  className="font-mono text-xs tracking-wide text-muted-foreground"
                >
                  {t("remote.conflictRevision")} {conflict.latest.workflow.revision ?? "—"}
                </span>
              </CardHeader>
              <CardContent className="gap-3">
                <div className="overflow-hidden rounded-md border border-border">
                  <div className="grid grid-cols-[minmax(96px,0.7fr)_1fr_1fr] gap-px bg-border text-xs">
                    <span className="bg-muted px-2 py-1.5 font-medium" />
                    <span className="bg-muted px-2 py-1.5 font-medium">
                      {t("remote.serverValue")}
                    </span>
                    <span className="bg-muted px-2 py-1.5 font-medium">
                      {t("remote.myInput")}
                    </span>
                    {conflict.rows.map((row) => (
                      <div
                        key={row.key}
                        data-testid={`conflict-row-${row.key}`}
                        className="col-span-3 grid grid-cols-subgrid gap-px"
                      >
                        <span className="bg-card px-2 py-1.5 font-mono text-muted-foreground">
                          {row.label}
                        </span>
                        <span
                          data-testid="conflict-server"
                          className="whitespace-pre-wrap break-words bg-card px-2 py-1.5 font-mono"
                        >
                          {row.server}
                        </span>
                        <span
                          data-testid="conflict-mine"
                          className="whitespace-pre-wrap break-words bg-card px-2 py-1.5 font-mono"
                        >
                          {row.mine}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button type="button" size="sm" variant="outline" onClick={reloadServerVersion}>
                    {t("remote.reloadServer")}
                  </Button>
                  <Button type="button" size="sm" onClick={() => void retryConflict()}>
                    {t("remote.resubmitLatest")}
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {/* 專案說明卡（獨立編輯態） */}
          <Card data-testid="context-card">
            <CardHeader className="flex-row items-start justify-between">
              <CardTitle className="text-base">{t("settings.contextLabel")}</CardTitle>
              <CardEditControls
                editing={ctxEditing}
                disabled={wfDisabled}
                msg={ctxMsg}
                testPrefix="context"
                onEdit={beginCtxEdit}
                onCancel={() => setCtxEditing(false)}
                onSave={() => void saveContext()}
              />
            </CardHeader>
            <CardContent className="gap-2.5">
              {ctxEditing ? (
                <div className="flex flex-col gap-2">
                  <Textarea
                    data-testid="context-input"
                    value={draftContext}
                    disabled={wfDisabled}
                    rows={10}
                    className="font-mono"
                    onChange={(e) => setDraftContext(e.target.value)}
                  />
                  <FieldHelp>{t("settings.contextHelp")}</FieldHelp>
                </div>
              ) : (
                <div className="flex flex-col gap-1.5">
                  <div data-testid="context-readonly" className={cn(contextCollapsed && "max-h-52 overflow-hidden")}>
                    <Markdown content={contextText || null} empty={t("settings.contextEmpty")} />
                  </div>
                  {contextCollapsed && (
                    <Button
                      type="button"
                      variant="link"
                      data-testid="context-show-more"
                      className="h-auto self-start p-0 text-xs font-normal"
                      onClick={() => setContextExpanded(true)}
                    >
                      {t("settings.showMore")}
                    </Button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>

          {/* 產出規則卡（獨立編輯態） */}
          <Card data-testid="rules-card">
            <CardHeader className="flex-row items-start justify-between">
              <CardTitle className="text-base">{t("settings.rulesLabel")}</CardTitle>
              <CardEditControls
                editing={rulesEditing}
                disabled={wfDisabled}
                msg={rulesMsg}
                testPrefix="rules"
                onEdit={beginRulesEdit}
                onCancel={() => setRulesEditing(false)}
                onSave={() => void saveRules()}
              />
            </CardHeader>
            <CardContent className="gap-2.5">
              {rulesEditing ? (
                <div className="flex flex-col gap-3">
                  {snap.workflow.schemaArtifacts.map((id) => (
                    <div key={id} className="flex flex-col gap-1">
                      <label htmlFor={`rules-input-${id}`} className="text-sm font-medium font-mono">
                        {id}
                      </label>
                      <Textarea
                        id={`rules-input-${id}`}
                        data-testid={`rules-input-${id}`}
                        value={draftRules[id] ?? ""}
                        disabled={wfDisabled}
                        rows={3}
                        className="font-mono"
                        onChange={(e) =>
                          setDraftRules((prev) => ({ ...prev, [id]: e.target.value }))
                        }
                      />
                    </div>
                  ))}
                  <FieldHelp>{t("settings.rulesHelp")}</FieldHelp>
                </div>
              ) : populatedRuleKeys.length === 0 ? (
                <p className="text-sm text-muted-foreground py-4 m-0">{t("common.noContent")}</p>
              ) : (
                <div className="flex flex-col gap-3">
                  {populatedRuleKeys.map((id) => (
                    <div key={id} data-testid={`rules-readonly-${id}`} className="flex flex-col gap-1">
                      <span className="text-sm font-medium font-mono">{id}</span>
                      <ul className="m-0 flex list-disc flex-col gap-0.5 pl-5 text-sm text-muted-foreground">
                        {(rules[id] ?? []).map((entry, i) => (
                          <li key={i}>{entry}</li>
                        ))}
                      </ul>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* 產出政策卡（原 openspec/config.yaml 卡更名） */}
          <Card data-testid="policy-card">
            <CardHeader>
              <CardTitle className="text-base">{t("settings.policyCard")}</CardTitle>
            </CardHeader>
            <CardContent className="gap-2.5">
              <div className="grid grid-cols-[110px_1fr] items-center gap-x-3 gap-y-1">
                <label htmlFor="cfg-locale" className="text-sm font-medium">locale</label>
                <Select
                  value={locale === "" ? LOCALE_UNSET : locale}
                  disabled={wfDisabled}
                  onValueChange={(v) => setLocale(v === LOCALE_UNSET ? "" : v)}
                >
                  <SelectTrigger id="cfg-locale">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={LOCALE_UNSET}>{t("settings.localeUnset")}</SelectItem>
                    <SelectItem value="tw">tw（繁體中文）</SelectItem>
                    <SelectItem value="ja">ja（日本語）</SelectItem>
                    <SelectItem value="en">en（English）</SelectItem>
                  </SelectContent>
                </Select>
                <span />
                <FieldHelp>{t("settings.localeHelp")}</FieldHelp>

                <label htmlFor="cfg-spec-locale" className="text-sm font-medium">spec_locale</label>
                <Select
                  value={specLocale === "" ? LOCALE_UNSET : specLocale}
                  disabled={wfDisabled}
                  onValueChange={(v) => setSpecLocale(v === LOCALE_UNSET ? "" : v)}
                >
                  <SelectTrigger id="cfg-spec-locale">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={LOCALE_UNSET}>{t("settings.localeUnset")}</SelectItem>
                    <SelectItem value="auto">auto</SelectItem>
                    <SelectItem value="tw">tw（繁體中文）</SelectItem>
                    <SelectItem value="ja">ja（日本語）</SelectItem>
                    <SelectItem value="en">en（English）</SelectItem>
                  </SelectContent>
                </Select>
                <span />
                <FieldHelp>{t("settings.specLocaleHelp")}</FieldHelp>

                <label htmlFor="cfg-tdd" className="text-sm font-medium">tdd</label>
                <div className="flex items-center">
                  <Checkbox
                    id="cfg-tdd"
                    checked={tdd}
                    disabled={wfDisabled}
                    onCheckedChange={(v) => setTdd(v === true)}
                  />
                </div>
                <span />
                <FieldHelp>{t("settings.tddHelp")}</FieldHelp>

                <label htmlFor="cfg-audit" className="text-sm font-medium">audit</label>
                <div className="flex items-center">
                  <Checkbox
                    id="cfg-audit"
                    checked={audit}
                    disabled={wfDisabled}
                    onCheckedChange={(v) => setAudit(v === true)}
                  />
                </div>
                <span />
                <FieldHelp>{t("settings.auditHelp")}</FieldHelp>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  type="button"
                  size="sm"
                  data-testid="save-workflow"
                  disabled={wfDisabled}
                  className="text-sm"
                  onClick={() => void saveWorkflow()}
                >
                  {t("settings.save")}
                </Button>
                {wfMsg && <span className="text-xs text-muted-foreground">{wfMsg}</span>}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* .speclink.yaml 簽：AI 工具 */}
        {!isRemote && <TabsContent value="speclink" className="pt-3 flex flex-col gap-4">
          <span data-testid="file-note-speclink" className="font-mono text-xs text-muted-foreground">.speclink.yaml</span>
          {snap.app.parseError !== null && <ParseErrorBanner message={snap.app.parseError} />}
          <Card data-testid="tools-card">
            <CardHeader>
              <CardTitle className="text-base">{t("settings.toolsLabel")}</CardTitle>
            </CardHeader>
            <CardContent className="gap-2.5">
              <div className="flex flex-col gap-1.5">
                <div className="flex gap-4">
                  {["claude", "codex"].map((tool) => (
                    <label key={tool} htmlFor={`tool-${tool}`} className="flex items-center gap-1.5 text-sm">
                      <Checkbox
                        id={`tool-${tool}`}
                        checked={tools.includes(tool)}
                        disabled={appDisabled}
                        onCheckedChange={(v) => toggleTool(tool, v === true)}
                      />
                      {tool}
                    </label>
                  ))}
                </div>
                <FieldHelp>{t("settings.toolsHelp")}</FieldHelp>
              </div>
              {snap.app.customTools.length > 0 && (
                <div className="flex flex-col gap-1">
                  <span className="text-sm font-medium">{t("settings.customToolsLabel")}</span>
                  <div className="flex gap-1.5 flex-wrap">
                    {snap.app.customTools.map((name) => (
                      <span key={name} className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
                        {name}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              <div className="flex items-center gap-2">
                <Button
                  type="button"
                  size="sm"
                  data-testid="save-app"
                  disabled={appDisabled}
                  className="text-sm"
                  onClick={() => void saveApp()}
                >
                  {t("settings.save")}
                </Button>
                {appMsg && <span className="text-xs text-muted-foreground">{appMsg}</span>}
              </div>
            </CardContent>
          </Card>
        </TabsContent>}

      </Tabs>
    </div>
  );
}
