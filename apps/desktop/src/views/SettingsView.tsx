// 設定頁（spec 需求「設定頁圖形化讀寫兩層設定」；design D5/D8/D9）：
// .speclink.yaml 的 tools 多選與 openspec/config.yaml 的政策欄位表單、
// UI 語言三選。欄位旁說明文字承接被 Mapping 讀-改-寫移除的範本註解教學角色。
import { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Checkbox,
  Markdown,
  NativeSelect,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  cn,
  useI18n,
} from "@speclink/ui";

import type { SettingsSnapshot, WorkspaceAdapter } from "../adapter/workspace";
import type { LocalePreference } from "../i18n/locale";

/** 行↔條目轉換（design D2）：一行一條規則——儲存時逐行修剪頭尾空白、空行滌除，行序即寫入順序。 */
const entriesToText = (entries: string[]) => entries.join("\n");
const textToEntries = (text: string) =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");

/** 收合門檻（design D3）：唯讀 markdown 超長截斷——渲染高度無法在 jsdom 量測，以原文規模判定。 */
const isLongContext = (text: string) => text.split("\n").length > 12 || text.length > 1200;

const TEXTAREA_CLS =
  "w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

export interface SettingsViewProps {
  workspace: WorkspaceAdapter;
  /** UI 語言偏好現值（null＝跟隨系統）。 */
  localePref: LocalePreference;
  /** 切換即時生效並持久化（App 層負責寫 localStorage）。 */
  onLocalePrefChange: (pref: LocalePreference) => void;
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

export function SettingsView({ workspace, localePref, onLocalePrefChange }: SettingsViewProps) {
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
  // 專案設定卡（design D1）：卡層級編輯態，兩分頁共享；草稿於進編輯時自現值播種。
  const [editing, setEditing] = useState(false);
  const [draftContext, setDraftContext] = useState("");
  const [draftRules, setDraftRules] = useState<Record<string, string>>({});
  const [contextExpanded, setContextExpanded] = useState(false);
  const [appMsg, setAppMsg] = useState<string | null>(null);
  const [wfMsg, setWfMsg] = useState<string | null>(null);
  const [projMsg, setProjMsg] = useState<string | null>(null);

  useEffect(() => {
    void workspace.readSettings().then((s) => {
      setSnap(s);
      setTools(s.app.tools);
      setLocale(s.workflow.locale ?? "");
      setSpecLocale(s.workflow.specLocale ?? "");
      setTdd(s.workflow.tdd);
      setAudit(s.workflow.audit);
      setContextText(s.workflow.context ?? "");
      setRules(
        Object.fromEntries(
          s.workflow.schemaArtifacts.map((id) => [id, s.workflow.rules[id] ?? []]),
        ),
      );
    });
  }, [workspace]);

  if (!snap) return null;

  const appDisabled = snap.app.parseError !== null;
  const wfDisabled = snap.workflow.parseError !== null;

  const toggleTool = (tool: string, on: boolean) =>
    setTools((prev) => (on ? [...prev.filter((x) => x !== tool), tool] : prev.filter((x) => x !== tool)));

  const saveApp = async () => {
    try {
      await workspace.writeAppTools(tools);
      setAppMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤（指明檔案與階段，來自 desktop-core），表單維持原值。
      setAppMsg(String(e));
    }
  };

  const saveWorkflow = async () => {
    try {
      await workspace.writeWorkflowConfig({
        locale: locale || null,
        specLocale: specLocale || null,
        tdd,
        audit,
      });
      setWfMsg(t("settings.saved"));
    } catch (e) {
      setWfMsg(String(e));
    }
  };

  const beginEdit = () => {
    setDraftContext(contextText);
    setDraftRules(
      Object.fromEntries(
        snap!.workflow.schemaArtifacts.map((id) => [id, entriesToText(rules[id] ?? [])]),
      ),
    );
    setProjMsg(null);
    setEditing(true);
  };

  const cancelEdit = () => setEditing(false);

  const saveProject = async () => {
    // 整份代換 payload：全部固定鍵依 schemaArtifacts 順序送出（空節＝移除鍵、全空＝移除 rules 鍵）；
    // 未動分頁寫回等值內容，檔案效果冪等（design 風險緩解）。
    const nextRules: Array<[string, string[]]> = snap!.workflow.schemaArtifacts.map((id) => [
      id,
      textToEntries(draftRules[id] ?? ""),
    ]);
    try {
      await workspace.writeWorkflowContext(draftContext);
      await workspace.writeWorkflowRules(nextRules);
      setContextText(draftContext);
      setRules(Object.fromEntries(nextRules));
      setContextExpanded(false);
      setEditing(false);
      setProjMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤（指明檔案與階段，來自 desktop-core），維持編輯態不遺失輸入。
      setProjMsg(String(e));
    }
  };

  const uiLocaleOptions: Array<{ value: LocalePreference; label: string }> = [
    { value: null, label: t("settings.followSystem") },
    { value: "zh-TW", label: "繁體中文" },
    { value: "en", label: "English" },
  ];

  const contextCollapsed = !editing && !contextExpanded && isLongContext(contextText);
  const populatedRuleKeys = snap.workflow.schemaArtifacts.filter((id) => (rules[id] ?? []).length > 0);

  return (
    <div className="flex flex-col gap-4 max-w-2xl mx-auto w-full">
      {/* config.yaml：專案設定卡（spec 需求「設定頁編輯專案說明與產出規則」；design D1–D4）——
          唯讀優先、卡層級就地編輯；產出規則整份文字編輯（一行一條規則）。 */}
      <Card data-testid="project-settings-card">
        <CardHeader className="flex-row items-start justify-between">
          <div className="flex flex-col gap-0.5">
            <CardTitle className="text-base">{t("settings.projectCard")}</CardTitle>
            <span className="font-mono text-xs text-muted-foreground">config.yaml</span>
          </div>
          <div className="flex items-center gap-2">
            {projMsg && <span className="text-xs text-muted-foreground">{projMsg}</span>}
            {editing ? (
              <>
                <button
                  type="button"
                  data-testid="project-cancel"
                  className="rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:bg-muted hover:text-foreground"
                  onClick={cancelEdit}
                >
                  {t("app.cancel")}
                </button>
                <button
                  type="button"
                  data-testid="project-save"
                  className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
                  onClick={() => void saveProject()}
                >
                  {t("settings.save")}
                </button>
              </>
            ) : (
              <button
                type="button"
                data-testid="project-edit"
                disabled={wfDisabled}
                className="rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-50"
                onClick={beginEdit}
              >
                {t("settings.edit")}
              </button>
            )}
          </div>
        </CardHeader>
        <CardContent className="gap-2.5">
          {snap.workflow.parseError !== null && <ParseErrorBanner message={snap.workflow.parseError} />}
          <Tabs defaultValue="context">
            <TabsList>
              <TabsTrigger value="context">{t("settings.contextLabel")}</TabsTrigger>
              <TabsTrigger value="rules">{t("settings.rulesLabel")}</TabsTrigger>
            </TabsList>
            <TabsContent value="context" className="pt-2.5">
              {editing ? (
                <div className="flex flex-col gap-2">
                  <textarea
                    data-testid="context-input"
                    value={draftContext}
                    rows={10}
                    className={TEXTAREA_CLS}
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
                    <button
                      type="button"
                      data-testid="context-show-more"
                      className="self-start text-xs text-primary hover:underline"
                      onClick={() => setContextExpanded(true)}
                    >
                      {t("settings.showMore")}
                    </button>
                  )}
                </div>
              )}
            </TabsContent>
            <TabsContent value="rules" className="pt-2.5">
              {editing ? (
                <div className="flex flex-col gap-3">
                  {snap.workflow.schemaArtifacts.map((id) => (
                    <div key={id} className="flex flex-col gap-1">
                      <label htmlFor={`rules-input-${id}`} className="text-sm font-medium font-mono">
                        {id}
                      </label>
                      <textarea
                        id={`rules-input-${id}`}
                        data-testid={`rules-input-${id}`}
                        value={draftRules[id] ?? ""}
                        rows={3}
                        className={TEXTAREA_CLS}
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
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* UI 語言（app 本機偏好——與 config.yaml 的 locale 是兩件事） */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("settings.uiLocaleLabel")}</CardTitle>
        </CardHeader>
        <CardContent className="gap-2">
          <div className="flex gap-1.5" data-testid="ui-locale">
            {uiLocaleOptions.map((opt) => (
              <button
                key={String(opt.value)}
                type="button"
                className={cn(
                  "rounded-md border px-3 py-1.5 text-sm transition-colors",
                  localePref === opt.value
                    ? "border-primary bg-primary/8 font-medium text-primary"
                    : "border-border text-muted-foreground hover:text-foreground hover:bg-muted",
                )}
                onClick={() => onLocalePrefChange(opt.value)}
              >
                {opt.label}
              </button>
            ))}
          </div>
          <FieldHelp>{t("settings.uiLocaleHelp")}</FieldHelp>
        </CardContent>
      </Card>

      {/* .speclink.yaml：tools 多選 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base font-mono">.speclink.yaml</CardTitle>
        </CardHeader>
        <CardContent className="gap-2.5">
          {snap.app.parseError !== null && <ParseErrorBanner message={snap.app.parseError} />}
          <div className="flex flex-col gap-1.5">
            <span className="text-sm font-medium">{t("settings.toolsLabel")}</span>
            <div className="flex gap-4">
              {["claude", "codex"].map((tool) => (
                <label key={tool} htmlFor={`tool-${tool}`} className="flex items-center gap-1.5 text-sm">
                  <Checkbox
                    id={`tool-${tool}`}
                    checked={tools.includes(tool)}
                    disabled={appDisabled}
                    onChange={(e) => toggleTool(tool, e.target.checked)}
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
            <button
              type="button"
              data-testid="save-app"
              disabled={appDisabled}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              onClick={() => void saveApp()}
            >
              {t("settings.save")}
            </button>
            {appMsg && <span className="text-xs text-muted-foreground">{appMsg}</span>}
          </div>
        </CardContent>
      </Card>

      {/* openspec/config.yaml：政策欄位 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base font-mono">openspec/config.yaml</CardTitle>
        </CardHeader>
        <CardContent className="gap-2.5">
          {snap.workflow.parseError !== null && <ParseErrorBanner message={snap.workflow.parseError} />}
          <div className="grid grid-cols-[110px_1fr] items-center gap-x-3 gap-y-1">
            <label htmlFor="cfg-locale" className="text-sm font-medium">locale</label>
            <NativeSelect
              id="cfg-locale"
              value={locale}
              disabled={wfDisabled}
              onChange={(e) => setLocale(e.target.value)}
            >
              <option value="">{t("settings.localeUnset")}</option>
              <option value="tw">tw（繁體中文）</option>
              <option value="ja">ja（日本語）</option>
              <option value="en">en（English）</option>
            </NativeSelect>
            <span />
            <FieldHelp>{t("settings.localeHelp")}</FieldHelp>

            <label htmlFor="cfg-spec-locale" className="text-sm font-medium">spec_locale</label>
            <NativeSelect
              id="cfg-spec-locale"
              value={specLocale}
              disabled={wfDisabled}
              onChange={(e) => setSpecLocale(e.target.value)}
            >
              <option value="">{t("settings.localeUnset")}</option>
              <option value="auto">auto</option>
              <option value="tw">tw（繁體中文）</option>
              <option value="ja">ja（日本語）</option>
              <option value="en">en（English）</option>
            </NativeSelect>
            <span />
            <FieldHelp>{t("settings.specLocaleHelp")}</FieldHelp>

            <label htmlFor="cfg-tdd" className="text-sm font-medium">tdd</label>
            <div className="flex items-center">
              <Checkbox
                id="cfg-tdd"
                checked={tdd}
                disabled={wfDisabled}
                onChange={(e) => setTdd(e.target.checked)}
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
                onChange={(e) => setAudit(e.target.checked)}
              />
            </div>
            <span />
            <FieldHelp>{t("settings.auditHelp")}</FieldHelp>
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              data-testid="save-workflow"
              disabled={wfDisabled}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              onClick={() => void saveWorkflow()}
            >
              {t("settings.save")}
            </button>
            {wfMsg && <span className="text-xs text-muted-foreground">{wfMsg}</span>}
          </div>
        </CardContent>
      </Card>

    </div>
  );
}
