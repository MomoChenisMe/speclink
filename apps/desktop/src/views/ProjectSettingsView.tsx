// 專案設定頁（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」）：
// 兩頁簽組織——config.yaml（專案說明／產出規則／產出政策）、
// .speclink.yaml（AI 工具）。頁簽標籤檔名直出（字面常數，
// LANGUAGE.md 明文例外）；專案說明與產出規則為獨立卡各持編輯態；解析失敗掛簽級
// 警示點與簽首橫幅。欄位旁說明文字承接被 Mapping 讀-改-寫移除的範本註解教學角色。
import { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Checkbox,
  Input,
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
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

import type { SchemaEntry, SettingsSnapshot, WorkflowFields } from "../adapter/workspace";
import type { WorkspaceSettingsProvider } from "../session";

/** locale／spec_locale 的「未設定」在 config 裡是空字串，Radix Select 的 item 不接受空字串。 */
const LOCALE_UNSET = "__unset__";

/** 官方合法語系代碼（引擎 LOCALE_CODES／SPEC_LOCALE_CODES 的前端鏡像）。 */
const LOCALE_OPTIONS: readonly string[] = ["tw", "ja", "en"];
const SPEC_LOCALE_OPTIONS: readonly string[] = ["auto", "tw", "ja", "en"];

type PendingRemoteWrite =
  | { kind: "policy"; fields: WorkflowFields }
  | { kind: "context"; context: string }
  | { kind: "rules"; rules: Array<[string, string[]]> }
  | { kind: "schema"; name: string };

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

/** 選項集外儲存值的引導提示（spec 需求「設定頁政策下拉的未知值顯性呈現」）。 */
function InvalidLocaleHint({ testId, children }: { testId: string; children: React.ReactNode }) {
  return (
    <p data-testid={testId} className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.warning}`}>
      <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
      {children}
    </p>
  );
}

function ParseErrorBanner({ message }: { message: string }) {
  const { t } = useI18n();
  return (
    <p role="alert" className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.danger}`}>
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
      className={cn("ml-1.5 inline-block h-1.5 w-1.5 rounded-full bg-current", SEMANTIC_TONE.warning)}
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
  const [worktree, setWorktree] = useState(false);
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
  /** 編輯面凍結鍵集（review N4）：開編輯當下的 schemaArtifacts 快照——編輯中
   * 固定鍵換集（切換 schema、fork／建立／刪除改變活躍解析）不換編輯面，儲存
   * 也以此鍵集送草稿，新固定鍵的既有規則走非凍結鍵兜底原樣保留。 */
  const [editingRuleKeys, setEditingRuleKeys] = useState<string[]>([]);
  const [rulesMsg, setRulesMsg] = useState<string | null>(null);
  const [contextExpanded, setContextExpanded] = useState(false);
  const [appMsg, setAppMsg] = useState<string | null>(null);
  const [wfMsg, setWfMsg] = useState<string | null>(null);
  const [conflict, setConflict] = useState<PolicyConflict | null>(null);
  // 產出流程頁籤（desktop-schema-panel design D4/D5）：清單快照、展開中的項、
  // 建立表單草稿與表單訊息。
  const [schemas, setSchemas] = useState<SchemaEntry[]>([]);
  const [expandedSchema, setExpandedSchema] = useState<string | null>(null);
  const [schemaMsg, setSchemaMsg] = useState<string | null>(null);
  const [createName, setCreateName] = useState("");
  /** 待確認刪除的專案層 schema 名稱（AlertDialog 開關；D7 確認後才執行）。 */
  const [pendingDeleteSchema, setPendingDeleteSchema] = useState<string | null>(null);

  const hydrate = (next: SettingsSnapshot) => {
    setSnap(next);
    setTools(next.app.tools);
    setLocale(next.workflow.locale ?? "");
    setSpecLocale(next.workflow.specLocale ?? "");
    setTdd(next.workflow.tdd);
    setAudit(next.workflow.audit);
    setWorktree(next.workflow.worktree);
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
    // stale-guard＋錯誤浮出（review R7）：session 快速切換時舊回應不得覆寫
    // 新清單；讀取失敗以表單訊息浮出而非 unhandled rejection 靜默空白。
    let cancelled = false;
    void settings.readSettings().then((next) => {
      if (!cancelled) hydrate(next);
    });
    void settings.readSchemas().then(
      (list) => {
        if (!cancelled) setSchemas(list);
      },
      (e) => {
        if (!cancelled) setSchemaMsg(errorMessage(e));
      },
    );
    return () => {
      cancelled = true;
    };
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
            worktree,
          };
    const mineContext =
      pending.kind === "context" ? pending.context : ctxEditing ? draftContext : contextText;
    const mineSchema = pending.kind === "schema" ? pending.name : snap.workflow.schemaName;
    // 編輯中以凍結鍵集入表（review S-a）：payload 鍵集由 editingRuleKeys 決定，
    // 對照表列的行必須跟 payload 同源，凍結鍵才不會從對照中消失。
    const artifactIds = Array.from(
      new Set([
        ...(rulesEditing ? editingRuleKeys : snap.workflow.schemaArtifacts),
        ...latest.workflow.schemaArtifacts,
      ]),
    );
    const pendingRules = pending.kind === "rules" ? Object.fromEntries(pending.rules) : null;
    const show = (value: string | null | undefined) => value || "—";
    const rows: ConflictRow[] = [
      {
        key: "schema",
        label: "schema",
        server: show(latest.workflow.schemaName),
        mine: show(mineSchema),
      },
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
      // 編輯中：凍結鍵顯示草稿；非凍結鍵（編輯期間換入的新固定鍵）顯示將被
      // 兜底原樣保留的現值，而非誤導的空白（review S-a）。
      const mine = rulesEditing
        ? editingRuleKeys.includes(id)
          ? draftRules[id] ?? ""
          : entriesToText(snap.workflow.rules[id] ?? [])
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
      else if (pending.kind === "schema") setSchemaMsg(message);
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
      worktree,
    };
    try {
      const next = await settings.writeWorkflowConfig(fields);
      adoptRevision(next);
      setWfMsg(t("settings.saved"));
    } catch (e) {
      if (await handleWriteError(e, { kind: "policy", fields })) setWfMsg(null);
      else if (!isRemote) {
        // local 失敗有兩種身分：被拒（例如 worktree 掛著時關不掉，檔案沒變）
        // 或技能同步失敗（config 已寫入、新值為正典）。重讀檔案現值——開關與
        // 快照都以檔案為準，才不會在下次儲存時靜默把政策寫回舊值。
        try {
          const fresh = await settings.readSettings();
          setSnap(fresh);
          setWorktree(fresh.workflow.worktree);
        } catch {
          setWorktree(snap?.workflow.worktree ?? false);
        }
        setWfMsg(errorMessage(e));
      } else {
        // remote 不重讀：adapter 的 readSettings 會靜默採納最新 revision，
        // 之後再存等於帶新 revision 提交過期欄位值、繞過 409 衝突對照。
        // 維持舊 revision，讓併發修改照常走 revision_conflict 對話框。
        setWorktree(snap?.workflow.worktree ?? false);
        setWfMsg(errorMessage(e));
      }
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
    // 凍結編輯面鍵集（review N4）：編輯期間固定鍵換集不影響編輯中的分節。
    setEditingRuleKeys(snap.workflow.schemaArtifacts);
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
    // 以凍結鍵集送出草稿（review N4）：編輯中固定鍵換集時，草稿仍對應開編輯
    // 當下的分節，不對新鍵送空值。
    const nextRules: Array<[string, string[]]> = editingRuleKeys.map((id) => [
      id,
      textToEntries(draftRules[id] ?? ""),
    ]);
    // 非凍結鍵的既有分節原樣附上（review R3／N4）：整份代換不得靜默刪掉其他
    // schema 的分節——含編輯期間換入的新固定鍵既有規則。
    for (const [key, entries] of Object.entries(snap.workflow.rules)) {
      if (!editingRuleKeys.includes(key)) nextRules.push([key, entries]);
    }
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

  /** 產出流程來源層級的顯示詞（package／project／user → 內建／專案／使用者）。 */
  const SCHEMA_SOURCE_LABEL_KEY = {
    package: "settings.schemaSourceBuiltin",
    project: "settings.schemaSourceProject",
    user: "settings.schemaSourceUser",
  } as const;
  const schemaSourceLabel = (source: keyof typeof SCHEMA_SOURCE_LABEL_KEY) =>
    t(SCHEMA_SOURCE_LABEL_KEY[source]);

  /** 引擎 fork 以 project→user→built-in 第一命中解析來源；被同名前層 shadow 的
   * 項不給 fork 入口——按下去複製到的會是前層內容（review R8）。引擎的層命中
   * 只看 schema.yaml 檔案存在，前層壞檔也算 shadow（不看 error）。 */
  const SCHEMA_RESOLUTION_ORDER = { project: 0, user: 1, package: 2 } as const;
  const isResolutionHit = (entry: SchemaEntry) =>
    !schemas.some(
      (s) =>
        s.name === entry.name &&
        SCHEMA_RESOLUTION_ORDER[s.source] < SCHEMA_RESOLUTION_ORDER[entry.source],
    );

  /** 切換／fork／建立／刪除成功後的快照局部採納（review R4／N1／N4）：只更新
   * schema 面與 rules 固定鍵——不走 hydrate，任一卡的編輯態與草稿不得被重設。
   * 產出規則編輯中時只更新 snap，唯讀顯示值與草稿不動——編輯面凍結在
   * editingRuleKeys（開編輯當下的鍵集），儲存時換入的新固定鍵規則由非凍結鍵
   * 兜底保留。重讀失敗不改報成錯誤：寫入已成功，快照留舊值待下次載入補上。 */
  const refreshSchemaFacts = async () => {
    try {
      const fresh = await settings.readSettings();
      setSnap(fresh);
      if (!rulesEditing) {
        setRules(
          Object.fromEntries(
            fresh.workflow.schemaArtifacts.map((id) => [id, fresh.workflow.rules[id] ?? []]),
          ),
        );
        setDraftRules(
          Object.fromEntries(
            fresh.workflow.schemaArtifacts.map((id) => [
              id,
              entriesToText(fresh.workflow.rules[id] ?? []),
            ]),
          ),
        );
      }
    } catch {
      /* 寫入已成功；重讀失敗容忍。 */
    }
  };

  const switchSchema = async (name: string) => {
    if (name === snap.workflow.schemaName) return;
    try {
      const next = await settings.writeWorkflowSchema(name);
      adoptRevision(next);
      setSchemaMsg(t("settings.saved"));
      await refreshSchemaFacts();
    } catch (e) {
      // remote revision 落後走衝突對照（review R5）；其餘失敗單行浮出、下拉維持原值。
      if (await handleWriteError(e, { kind: "schema", name })) setSchemaMsg(null);
      else setSchemaMsg(errorMessage(e));
    }
  };

  /** schema 動作共用骨架（review S3）：執行 → 重拉清單＋局部採納快照（fork／
   * 建立／刪除都可能改變同名 shadow 下的活躍解析，固定鍵一併跟上）→ 已儲存；
   * 失敗單行浮出。 */
  const runSchemaAction = async (action: () => Promise<unknown>) => {
    try {
      await action();
      setSchemas(await settings.readSchemas());
      await refreshSchemaFacts();
      setSchemaMsg(t("settings.saved"));
    } catch (e) {
      setSchemaMsg(errorMessage(e));
    }
  };

  const forkSchemaAction = (source: string) => runSchemaAction(() => settings.forkSchema(source));

  const confirmDeleteSchema = () => {
    const name = pendingDeleteSchema;
    setPendingDeleteSchema(null);
    if (name === null) return Promise.resolve();
    return runSchemaAction(() => settings.deleteSchema(name));
  };

  const createSchemaAction = () => {
    // 名稱驗證在引擎（D5）：kebab-case 規則與已存在檢查不在前端重複，錯誤原樣浮出。
    const name = createName.trim();
    if (name === "") return Promise.resolve();
    return runSchemaAction(async () => {
      await settings.createSchema(name);
      setCreateName("");
    });
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
      } else if (pending.kind === "schema") {
        next = await settings.writeWorkflowSchema(pending.name);
        setSchemaMsg(t("settings.saved"));
        adoptRevision(next);
        setConflict(null);
        await refreshSchemaFacts();
        return;
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
        else if (pending.kind === "schema") setSchemaMsg(message);
        else setRulesMsg(message);
      }
    }
  };

  const contextCollapsed = !ctxEditing && !contextExpanded && isLongContext(contextText);
  const populatedRuleKeys = snap.workflow.schemaArtifacts.filter((id) => (rules[id] ?? []).length > 0);
  // 下拉選項：壞項不可選；同名跨層去重（解析語意本就取第一命中）。
  const selectableSchemaNames = Array.from(
    new Set(schemas.filter((s) => s.error === null).map((s) => s.name)),
  );

  return (
    <div className="max-w-2xl mx-auto w-full">
      {/* remote 409 對照面板：置於頁簽之外——政策與 schema 兩簽的寫入都可能
          撞衝突，面板必須在任一簽都可見（review R5）。 */}
      {conflict !== null && (
        <Card
          data-testid="policy-conflict-panel"
          className={cn(SEMANTIC_SURFACE.warning, "bg-muted/20", "mb-4")}
        >
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
      {/* 兩頁簽：標籤檔名直出為字面常數（LANGUAGE.md 明文例外）。 */}
      <Tabs defaultValue="config">
        <TabsList>
          <TabsTrigger value="config">
            {isRemote ? t("remote.workflowTab") : "config.yaml"}
            {wfDisabled && <TabWarningDot />}
          </TabsTrigger>
          {/* 產出流程為獨立頁籤（D4 改版）。標籤直出 Schema（D8；LANGUAGE.md
              明文例外——頁籤列全是技術 token，籤內文案仍用「產出流程」）。 */}
          <TabsTrigger value="schemas">{t("settings.schemaTab")}</TabsTrigger>
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
                  {/* 編輯面以凍結鍵集渲染（review N4），不跟 snap 的固定鍵換集。 */}
                  {editingRuleKeys.map((id) => (
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
                    {locale !== "" && !LOCALE_OPTIONS.includes(locale) && (
                      <SelectItem value={locale} disabled className="text-muted-foreground">
                        {locale}
                        {t("settings.localeInvalid")}
                      </SelectItem>
                    )}
                    <SelectItem value={LOCALE_UNSET}>{t("settings.localeUnset")}</SelectItem>
                    <SelectItem value="tw">tw（繁體中文）</SelectItem>
                    <SelectItem value="ja">ja（日本語）</SelectItem>
                    <SelectItem value="en">en（English）</SelectItem>
                  </SelectContent>
                </Select>
                <span />
                <FieldHelp>{t("settings.localeHelp")}</FieldHelp>
                {locale !== "" && !LOCALE_OPTIONS.includes(locale) && (
                  <>
                    <span />
                    <InvalidLocaleHint testId="locale-invalid-hint">
                      {t("settings.localeInvalidHint")}
                    </InvalidLocaleHint>
                  </>
                )}

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
                    {specLocale !== "" && !SPEC_LOCALE_OPTIONS.includes(specLocale) && (
                      <SelectItem value={specLocale} disabled className="text-muted-foreground">
                        {specLocale}
                        {t("settings.localeInvalid")}
                      </SelectItem>
                    )}
                    <SelectItem value={LOCALE_UNSET}>{t("settings.localeUnset")}</SelectItem>
                    <SelectItem value="auto">auto</SelectItem>
                    <SelectItem value="tw">tw（繁體中文）</SelectItem>
                    <SelectItem value="ja">ja（日本語）</SelectItem>
                    <SelectItem value="en">en（English）</SelectItem>
                  </SelectContent>
                </Select>
                <span />
                <FieldHelp>{t("settings.specLocaleHelp")}</FieldHelp>
                {specLocale !== "" && !SPEC_LOCALE_OPTIONS.includes(specLocale) && (
                  <>
                    <span />
                    <InvalidLocaleHint testId="spec-locale-invalid-hint">
                      {t("settings.specLocaleInvalidHint")}
                    </InvalidLocaleHint>
                  </>
                )}

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

                {!isRemote && (
                  <>
                    <label htmlFor="cfg-worktree" className="text-sm font-medium">worktree</label>
                    <div className="flex items-center">
                      <Checkbox
                        id="cfg-worktree"
                        checked={worktree}
                        disabled={wfDisabled}
                        onCheckedChange={(v) => setWorktree(v === true)}
                      />
                    </div>
                    <span />
                    <FieldHelp>{t("settings.worktreeHelp")}</FieldHelp>
                  </>
                )}
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

        {/* 產出流程頁籤（desktop-schema-panel design D4 改版＋D5）：清單→收合
            唯讀詳情、下拉切換即寫入、local 限定 fork 與建立、remote 狀態文案 */}
        <TabsContent value="schemas" className="pt-3 flex flex-col gap-4">
          <Card data-testid="schema-card">
            <CardHeader className="gap-1">
              <CardTitle className="text-base">{t("settings.schemaCard")}</CardTitle>
              <FieldHelp>{t("settings.schemaCardHelp")}</FieldHelp>
            </CardHeader>
            <CardContent className="gap-2.5">
              {!snap.workflow.schemaKnown && (
                <p
                  data-testid="schema-unknown-note"
                  className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.warning}`}
                >
                  <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                  {t("settings.schemaUnknownNote")}
                </p>
              )}
              <div className="flex flex-col gap-1">
                <label htmlFor="cfg-schema" className="text-sm font-medium">
                  {t("settings.schemaActiveLabel")}
                </label>
                <Select
                  // 恆受控（review N2）：value 一律等於快照現值，寫入失敗後
                  // 觸發器退回現值、不停在剛點選的名稱。壞檔空字串傳 undefined
                  // 讓 placeholder 顯示「—」。
                  value={snap.workflow.schemaName || undefined}
                  disabled={wfDisabled}
                  onValueChange={(v) => void switchSchema(v)}
                >
                  <SelectTrigger id="cfg-schema" className="w-64">
                    <SelectValue placeholder="—" />
                  </SelectTrigger>
                  <SelectContent>
                    {/* 現值不在選項集（remote 非內建、活躍項解析失敗）→ 沿 locale
                        未知值模式以停用項顯示原始值（review R6／N2）。 */}
                    {snap.workflow.schemaName !== "" &&
                      !selectableSchemaNames.includes(snap.workflow.schemaName) && (
                        <SelectItem
                          value={snap.workflow.schemaName}
                          disabled
                          className="text-muted-foreground"
                        >
                          {snap.workflow.schemaName}
                        </SelectItem>
                      )}
                    {selectableSchemaNames.map((name) => (
                      <SelectItem key={name} value={name}>
                        {name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {schemaMsg && <span className="text-xs text-muted-foreground">{schemaMsg}</span>}
              </div>
              <div className="flex flex-col gap-2">
                {schemas.map((entry) => {
                  const key = `${entry.source}-${entry.name}`;
                  const expanded = expandedSchema === key;
                  return (
                    <div
                      key={key}
                      data-testid={`schema-item-${entry.name}`}
                      className="flex flex-col gap-1.5 rounded-md border border-border px-3 py-2"
                    >
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium font-mono">{entry.name}</span>
                        <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
                          {schemaSourceLabel(entry.source)}
                        </span>
                        <span className="ml-auto flex items-center gap-2">
                          {entry.error === null && (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              data-testid={`schema-toggle-${entry.name}`}
                              className="text-sm font-normal text-muted-foreground hover:text-foreground"
                              onClick={() => setExpandedSchema(expanded ? null : key)}
                            >
                              {expanded
                                ? t("settings.schemaHideDetail")
                                : t("settings.schemaShowDetail")}
                            </Button>
                          )}
                          {!isRemote && entry.error === null && isResolutionHit(entry) && (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              data-testid={`schema-fork-${entry.name}`}
                              className="text-sm font-normal text-muted-foreground hover:text-foreground"
                              onClick={() => void forkSchemaAction(entry.name)}
                            >
                              {t("settings.schemaFork")}
                            </Button>
                          )}
                          {/* 刪除（D7）：僅專案層——內建無檔案、user 層跨專案共用。
                              按下先開確認對話框，確認後才執行。 */}
                          {!isRemote && entry.source === "project" && (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              data-testid={`schema-delete-${entry.name}`}
                              className={`text-sm font-normal ${SEMANTIC_TONE.danger}`}
                              onClick={() => setPendingDeleteSchema(entry.name)}
                            >
                              {t("settings.schemaDelete")}
                            </Button>
                          )}
                          {/* 編輯入口（D6）：有磁碟路徑才有資料夾可顯示——內建在
                              binary 內、remote 無本機檔案，均不渲染。 */}
                          {!isRemote && entry.path !== null && (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              data-testid={`schema-reveal-${entry.name}`}
                              className="text-sm font-normal text-muted-foreground hover:text-foreground"
                              onClick={() => {
                                void settings.revealSchema(entry.path as string).catch((e) => {
                                  setSchemaMsg(errorMessage(e));
                                });
                              }}
                            >
                              {t("settings.schemaReveal")}
                            </Button>
                          )}
                        </span>
                      </div>
                      {entry.error !== null ? (
                        <p className={`text-xs m-0 ${SEMANTIC_TONE.danger}`}>{entry.error}</p>
                      ) : (
                        <span className="text-xs text-muted-foreground font-mono">
                          {entry.artifactIds.join(" → ")}
                        </span>
                      )}
                      {expanded && (
                        <div
                          data-testid={`schema-detail-${entry.name}`}
                          className="flex max-h-96 flex-col gap-3 overflow-y-auto border-t border-border pt-2"
                        >
                          {entry.artifacts.map((a) => (
                            <div key={a.id} className="flex flex-col gap-1">
                              <span className="text-sm font-medium font-mono">{a.id}</span>
                              <p className="m-0 text-sm text-muted-foreground">{a.description}</p>
                              {a.instruction !== null && (
                                <>
                                  <span className="text-xs font-medium">
                                    {t("settings.schemaArtifactInstruction")}
                                  </span>
                                  <pre className="m-0 whitespace-pre-wrap rounded-md bg-muted/40 p-2 text-xs">
                                    {a.instruction}
                                  </pre>
                                </>
                              )}
                              {a.template !== null && (
                                <>
                                  <span className="text-xs font-medium">
                                    {t("settings.schemaArtifactTemplate")}
                                  </span>
                                  <pre className="m-0 whitespace-pre-wrap rounded-md bg-muted/40 p-2 text-xs">
                                    {a.template}
                                  </pre>
                                </>
                              )}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
              {/* 建立表單（D5；僅 local）：單一名稱輸入，佈局用引擎預設骨架，
                  kebab-case 與已存在檢查在引擎、錯誤原樣浮出。 */}
              {!isRemote && (
                <div className="flex flex-col gap-1">
                  <label htmlFor="schema-create-name" className="text-sm font-medium">
                    {t("settings.schemaCreateLabel")}
                  </label>
                  <div className="flex items-center gap-2">
                    <Input
                      id="schema-create-name"
                      data-testid="schema-create-name"
                      value={createName}
                      placeholder="my-flow"
                      className="w-64 font-mono"
                      onChange={(e) => setCreateName(e.target.value)}
                    />
                    <Button
                      type="button"
                      size="sm"
                      data-testid="schema-create"
                      className="text-sm"
                      onClick={() => void createSchemaAction()}
                    >
                      {t("settings.schemaCreate")}
                    </Button>
                  </div>
                  <FieldHelp>{t("settings.schemaCreateHelp")}</FieldHelp>
                </div>
              )}
            </CardContent>
          </Card>

          {/* 刪除確認（D7；沿變更刪除的 AlertDialog 模式） */}
          <AlertDialog
            open={pendingDeleteSchema !== null}
            onOpenChange={(o) => !o && setPendingDeleteSchema(null)}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{t("settings.schemaDeleteTitle")}</AlertDialogTitle>
                <AlertDialogDescription>
                  {t("settings.schemaDeleteDesc")}{" "}
                  <span className="font-mono font-medium">{pendingDeleteSchema}</span>
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel
                  data-testid="schema-delete-cancel"
                  onClick={() => setPendingDeleteSchema(null)}
                >
                  {t("app.cancel")}
                </AlertDialogCancel>
                <AlertDialogAction
                  data-testid="schema-delete-confirm"
                  className="bg-destructive hover:bg-destructive/90"
                  onClick={() => void confirmDeleteSchema()}
                >
                  {t("settings.schemaDelete")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
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
