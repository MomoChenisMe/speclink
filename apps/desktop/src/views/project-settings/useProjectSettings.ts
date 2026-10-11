// 專案設定的狀態與動作（desktop-settings-mode design D8）：自 ProjectSettingsView 原樣搬出——
// 兩層設定的載入、儲存、remote revision 衝突對照與產出流程的切換／fork／建立／刪除，邏輯不變；
// 四個分類頁共用同一份狀態，切頁不重載、各頁的編輯態與草稿跨頁保留。
import { useEffect, useRef, useState } from "react";
import { useI18n } from "@speclink/ui";

import type { SchemaEntry, SettingsSnapshot, WorkflowFields } from "../../adapter/workspace";
import type { WorkspaceSettingsProvider } from "../../session";

/** locale／spec_locale 的「未設定」在 config 裡是空字串，Radix Select 的 item 不接受空字串。 */
export const LOCALE_UNSET = "__unset__";

/** 官方合法語系代碼（引擎 LOCALE_CODES／SPEC_LOCALE_CODES 的前端鏡像）。 */
export const LOCALE_OPTIONS: readonly string[] = ["tw", "ja", "en"];
export const SPEC_LOCALE_OPTIONS: readonly string[] = ["auto", "tw", "ja", "en"];

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

export interface PolicyConflict {
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

export function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return String(error);
}

/** 行↔條目轉換（design D2）：一行一條規則——儲存時逐行修剪頭尾空白、空行滌除，行序即寫入順序。 */
const entriesToText = (entries: string[]) => entries.join("\n");
export const textToEntries = (text: string) =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");



export function useProjectSettings(settings: WorkspaceSettingsProvider) {
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
  // 寫入中（desktop-settings-editors design D2）：state 驅動按鈕停用與編輯器唯讀，ref 擋同一輪
  // 事件內的重送；放在四頁共用的狀態裡，儲存中切頁再切回也不重送。
  const [ctxSaving, setCtxSaving] = useState(false);
  const [rulesSaving, setRulesSaving] = useState(false);
  const writing = useRef({ context: false, rules: false });
  const [appMsg, setAppMsg] = useState<string | null>(null);
  const [wfMsg, setWfMsg] = useState<string | null>(null);
  const [conflict, setConflictState] = useState<PolicyConflict | null>(null);
  /** 衝突狀態的同步鏡像：409 設下衝突到下一次 render 之間，寫入守門也要看得到它。 */
  const conflictOpen = useRef(false);
  const setConflict = (next: PolicyConflict | null) => {
    conflictOpen.current = next !== null;
    setConflictState(next);
  };
  // 產出流程頁籤（desktop-schema-panel design D4/D5）：清單快照、展開中的項、
  // 建立表單草稿與表單訊息。
  const [schemas, setSchemas] = useState<SchemaEntry[]>([]);
  const [expandedSchema, setExpandedSchema] = useState<string | null>(null);
  const [schemaMsg, setSchemaMsg] = useState<string | null>(null);
  const [createName, setCreateName] = useState("");
  /** 待確認刪除的專案層 schema 名稱（確認框開關；D7 確認後才執行）。 */
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

  // 寫入成功後把已存值併進快照：快照＝檔案現值，未存修改（dirty）以它為基準比對。
  const adoptTools = (saved: string[]) =>
    setSnap((current) => current && { ...current, app: { ...current.app, tools: saved } });
  const adoptPolicy = (fields: WorkflowFields) =>
    setSnap((current) => current && { ...current, workflow: { ...current.workflow, ...fields } });

  const toggleTool = (tool: string, on: boolean) =>
    setTools((prev) => (on ? [...prev.filter((x) => x !== tool), tool] : prev.filter((x) => x !== tool)));

  const saveApp = async () => {
    try {
      await settings.writeAppTools(tools);
      adoptTools(tools);
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
      adoptPolicy(fields);
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

  /** 按鈕、編輯器內 ⌘S 與頁層 ⌘S 共用的寫入守門：停用時（解析失敗、檢視者、衝突對照中）
   * 不送——衝突對照中 adapter 已採用最新 revision，再送會繞過對照卡蓋掉 server 版；
   * 同一份寫入進行中也不重送。 */
  const guardWrite = async (
    kind: "context" | "rules",
    setSaving: (saving: boolean) => void,
    write: () => Promise<void>,
  ) => {
    if (wfDisabled || conflictOpen.current || writing.current[kind]) return;
    writing.current[kind] = true;
    setSaving(true);
    try {
      await write();
    } finally {
      writing.current[kind] = false;
      setSaving(false);
    }
  };

  const saveContext = () => guardWrite("context", setCtxSaving, async () => {
    // 僅寫 context 鍵（清空＝移除鍵）；產出規則卡對應的 rules 鍵不觸碰。
    try {
      const next = await settings.writeWorkflowContext(draftContext);
      adoptRevision(next);
      setContextText(draftContext);
      setCtxEditing(false);
      setCtxMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤，維持編輯態不遺失輸入。
      if (await handleWriteError(e, { kind: "context", context: draftContext })) setCtxMsg(null);
      else setCtxMsg(errorMessage(e));
    }
  });

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

  const saveRules = () => guardWrite("rules", setRulesSaving, async () => {
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
  });

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
        adoptPolicy(pending.fields);
        setWfMsg(t("settings.saved"));
      } else if (pending.kind === "context") {
        next = await settings.writeWorkflowContext(pending.context);
        setContextText(pending.context);
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

  // 未存修改（App 的返回據此先問要不要捨棄）：表單值對照快照，兩張文字卡只看編輯中的草稿。
  const dirty =
    tools.length !== snap.app.tools.length ||
    tools.some((tool) => !snap.app.tools.includes(tool)) ||
    locale !== (snap.workflow.locale ?? "") ||
    specLocale !== (snap.workflow.specLocale ?? "") ||
    tdd !== snap.workflow.tdd ||
    audit !== snap.workflow.audit ||
    worktree !== snap.workflow.worktree ||
    (ctxEditing && draftContext !== contextText) ||
    (rulesEditing && editingRuleKeys.some((id) => (draftRules[id] ?? "") !== entriesToText(rules[id] ?? [])));
  // 頁標題下的 remote 註記：policy revision 與檢視者說明（local 為 null，改顯示檔案鍵）。
  const remoteNote = isRemote
    ? { revision: snap.workflow.revision ?? null, reader: !settings.policyWrite }
    : null;

  const populatedRuleKeys = snap.workflow.schemaArtifacts.filter((id) => (rules[id] ?? []).length > 0);
  // 下拉選項：壞項不可選；同名跨層去重（解析語意本就取第一命中）。
  const selectableSchemaNames = Array.from(
    new Set(schemas.filter((s) => s.error === null).map((s) => s.name)),
  );


  return {
    settings, snap, isRemote, appDisabled, wfDisabled, dirty, remoteNote,
    tools, toggleTool, saveApp, appMsg,
    locale, setLocale, specLocale, setSpecLocale, tdd, setTdd, audit, setAudit, worktree, setWorktree,
    saveWorkflow, wfMsg,
    contextText, ctxEditing, setCtxEditing, draftContext, setDraftContext, ctxMsg, beginCtxEdit, saveContext, ctxSaving,
    rules, rulesEditing, setRulesEditing, draftRules, setDraftRules, editingRuleKeys, rulesMsg,
    beginRulesEdit, saveRules, rulesSaving, populatedRuleKeys,
    conflict, reloadServerVersion, retryConflict,
    schemas, expandedSchema, setExpandedSchema, schemaMsg, setSchemaMsg, createName, setCreateName,
    pendingDeleteSchema, setPendingDeleteSchema, schemaSourceLabel, isResolutionHit, switchSchema,
    forkSchemaAction, confirmDeleteSchema, createSchemaAction, selectableSchemaNames,
  };
}

export type ProjectSettingsState = NonNullable<ReturnType<typeof useProjectSettings>>;
