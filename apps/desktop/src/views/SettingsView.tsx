// 設定頁（spec 需求「設定頁圖形化讀寫兩層設定」；design D5/D8/D9）：
// .speclink.yaml 的 tools 多選與 openspec/config.yaml 的政策欄位表單、
// UI 語言三選。欄位旁說明文字承接被 Mapping 讀-改-寫移除的範本註解教學角色。
import { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, Checkbox, NativeSelect, cn, useI18n } from "@speclink/ui";

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
  const [appMsg, setAppMsg] = useState<string | null>(null);
  const [wfMsg, setWfMsg] = useState<string | null>(null);

  useEffect(() => {
    void workspace.readSettings().then((s) => {
      setSnap(s);
      setTools(s.app.tools);
      setLocale(s.workflow.locale ?? "");
      setSpecLocale(s.workflow.specLocale ?? "");
      setTdd(s.workflow.tdd);
      setAudit(s.workflow.audit);
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
    </div>
  );
}
