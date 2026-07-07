// 設定頁（spec 需求「設定頁圖形化讀寫兩層設定」；design D5/D8/D9）：
// .speclink.yaml 的 tools 多選與 openspec/config.yaml 的政策欄位表單、
// UI 語言三選。欄位旁說明文字承接被 Mapping 讀-改-寫移除的範本註解教學角色。
import { useEffect, useState } from "react";
import { AlertTriangle, ChevronDown, ChevronUp, Plus, Trash2 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, Checkbox, Input, NativeSelect, cn, useI18n } from "@speclink/ui";

import type { SettingsSnapshot, WorkspaceAdapter } from "../adapter/workspace";
import type { LocalePreference } from "../i18n/locale";

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
  /** 產出規則編輯狀態：schemaArtifacts 固定鍵 → 條目清單（清單順序即寫入順序）。 */
  const [rules, setRules] = useState<Record<string, string[]>>({});
  const [appMsg, setAppMsg] = useState<string | null>(null);
  const [wfMsg, setWfMsg] = useState<string | null>(null);
  const [ctxMsg, setCtxMsg] = useState<string | null>(null);
  const [rulesMsg, setRulesMsg] = useState<string | null>(null);

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

  const saveContext = async () => {
    try {
      await workspace.writeWorkflowContext(contextText);
      setCtxMsg(t("settings.saved"));
    } catch (e) {
      setCtxMsg(String(e));
    }
  };

  const saveRules = async () => {
    // 整份代換 payload：全部固定鍵依 schemaArtifacts 順序送出（空節＝移除鍵）。
    const payload: Array<[string, string[]]> = snap!.workflow.schemaArtifacts.map((id) => [
      id,
      rules[id] ?? [],
    ]);
    try {
      await workspace.writeWorkflowRules(payload);
      setRulesMsg(t("settings.saved"));
    } catch (e) {
      setRulesMsg(String(e));
    }
  };

  const setEntry = (id: string, idx: number, value: string) =>
    setRules((prev) => ({ ...prev, [id]: prev[id].map((e, i) => (i === idx ? value : e)) }));
  const addEntry = (id: string) =>
    setRules((prev) => ({ ...prev, [id]: [...(prev[id] ?? []), ""] }));
  const removeEntry = (id: string, idx: number) =>
    setRules((prev) => ({ ...prev, [id]: prev[id].filter((_, i) => i !== idx) }));
  const moveEntry = (id: string, idx: number, delta: -1 | 1) =>
    setRules((prev) => {
      const list = [...prev[id]];
      const target = idx + delta;
      if (target < 0 || target >= list.length) return prev;
      [list[idx], list[target]] = [list[target], list[idx]];
      return { ...prev, [id]: list };
    });

  const uiLocaleOptions: Array<{ value: LocalePreference; label: string }> = [
    { value: null, label: t("settings.followSystem") },
    { value: "zh-TW", label: "繁體中文" },
    { value: "en", label: "English" },
  ];

  return (
    <div className="flex flex-col gap-4 max-w-2xl mx-auto w-full">
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

      {/* config.yaml：專案說明（context）——spec 需求「設定頁編輯專案說明與產出規則」 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("settings.contextLabel")}</CardTitle>
        </CardHeader>
        <CardContent className="gap-2">
          {snap.workflow.parseError !== null && <ParseErrorBanner message={snap.workflow.parseError} />}
          <textarea
            data-testid="context-input"
            value={contextText}
            disabled={wfDisabled}
            rows={6}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
            onChange={(e) => setContextText(e.target.value)}
          />
          <FieldHelp>{t("settings.contextHelp")}</FieldHelp>
          <div className="flex items-center gap-2">
            <button
              type="button"
              data-testid="save-context"
              disabled={wfDisabled}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              onClick={() => void saveContext()}
            >
              {t("settings.save")}
            </button>
            {ctxMsg && <span className="text-xs text-muted-foreground">{ctxMsg}</span>}
          </div>
        </CardContent>
      </Card>

      {/* config.yaml：產出規則（rules）——schema 固定鍵分節、上下移排序（design D2） */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("settings.rulesLabel")}</CardTitle>
        </CardHeader>
        <CardContent className="gap-3">
          {snap.workflow.parseError !== null && <ParseErrorBanner message={snap.workflow.parseError} />}
          <FieldHelp>{t("settings.rulesHelp")}</FieldHelp>
          {snap.workflow.schemaArtifacts.map((id) => {
            const entries = rules[id] ?? [];
            return (
              <div key={id} data-testid={`rules-section-${id}`} className="flex flex-col gap-1.5">
                <span className="text-sm font-medium font-mono">{id}</span>
                {entries.map((entry, idx) => (
                  <div key={idx} className="flex items-center gap-1">
                    <Input
                      value={entry}
                      disabled={wfDisabled}
                      className="flex-1 font-mono text-xs"
                      onChange={(e) => setEntry(id, idx, e.target.value)}
                    />
                    <button
                      type="button"
                      aria-label={t("settings.ruleUp")}
                      disabled={wfDisabled || idx === 0}
                      className="rounded-md border border-border p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-40"
                      onClick={() => moveEntry(id, idx, -1)}
                    >
                      <ChevronUp className="h-3.5 w-3.5" />
                    </button>
                    <button
                      type="button"
                      aria-label={t("settings.ruleDown")}
                      disabled={wfDisabled || idx === entries.length - 1}
                      className="rounded-md border border-border p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-40"
                      onClick={() => moveEntry(id, idx, 1)}
                    >
                      <ChevronDown className="h-3.5 w-3.5" />
                    </button>
                    <button
                      type="button"
                      aria-label={t("settings.ruleDelete")}
                      disabled={wfDisabled}
                      className="rounded-md border border-border p-1.5 text-muted-foreground hover:text-destructive disabled:opacity-40"
                      onClick={() => removeEntry(id, idx)}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  </div>
                ))}
                <button
                  type="button"
                  disabled={wfDisabled}
                  className="self-start inline-flex items-center gap-1 rounded-md border border-dashed border-border px-2 py-1 text-xs text-muted-foreground hover:text-foreground disabled:opacity-40"
                  onClick={() => addEntry(id)}
                >
                  <Plus className="h-3 w-3" /> {t("settings.addRule")}
                </button>
              </div>
            );
          })}
          <div className="flex items-center gap-2">
            <button
              type="button"
              data-testid="save-rules"
              disabled={wfDisabled}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              onClick={() => void saveRules()}
            >
              {t("settings.save")}
            </button>
            {rulesMsg && <span className="text-xs text-muted-foreground">{rulesMsg}</span>}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
