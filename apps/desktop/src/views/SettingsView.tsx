// 設定頁（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」；
// design D1–D3）：三頁簽組織——config.yaml（專案說明／產出規則／產出政策）、
// .speclink.yaml（AI 工具）、本機設定（介面語言）。頁簽標籤檔名直出（字面常數，
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
  NativeSelect,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Textarea,
  cn,
  useI18n,
} from "@speclink/ui";

import type { SettingsSnapshot } from "../adapter/workspace";
import type { WorkspaceSettingsProvider } from "../session";
import type { LocalePreference } from "../i18n/locale";
import { ServersPanel, type ServersPanelProps } from "../components/ServersPanel";

/** 行↔條目轉換（design D2）：一行一條規則——儲存時逐行修剪頭尾空白、空行滌除，行序即寫入順序。 */
const entriesToText = (entries: string[]) => entries.join("\n");
const textToEntries = (text: string) =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");

/** 收合門檻（design D3）：唯讀 markdown 超長截斷——渲染高度無法在 jsdom 量測，以原文規模判定。 */
const isLongContext = (text: string) => text.split("\n").length > 12 || text.length > 1200;

export interface SettingsViewProps {
  /** 活躍 session 的設定面（root 已綁定；workspace-session 決策 3）。 */
  settings: WorkspaceSettingsProvider;
  /** UI 語言偏好現值（null＝跟隨系統）。 */
  localePref: LocalePreference;
  /** 切換即時生效並持久化（App 層負責寫 localStorage）。 */
  onLocalePrefChange: (pref: LocalePreference) => void;
  /** 面板建立失敗的單行錯誤（spec：退回原生選單並於本機設定簽以獨立警示行浮出）。 */
  trayPanelError?: string | null;
  /** 伺服器頁籤（desktop-connections；app 全域、不經 session）；未注入即不顯示。 */
  servers?: ServersPanelProps;
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
            className="text-sm font-normal text-muted-foreground hover:text-foreground"
            onClick={onCancel}
          >
            {t("app.cancel")}
          </Button>
          <Button
            type="button"
            size="sm"
            data-testid={`${testPrefix}-save`}
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

export function SettingsView({
  settings,
  localePref,
  onLocalePrefChange,
  trayPanelError = null,
  servers,
}: SettingsViewProps) {
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

  useEffect(() => {
    void settings.readSettings().then((s) => {
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
  }, [settings]);

  if (!snap) return null;

  const appDisabled = snap.app.parseError !== null;
  const wfDisabled = snap.workflow.parseError !== null;

  const toggleTool = (tool: string, on: boolean) =>
    setTools((prev) => (on ? [...prev.filter((x) => x !== tool), tool] : prev.filter((x) => x !== tool)));

  const saveApp = async () => {
    try {
      await settings.writeAppTools(tools);
      setAppMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤（指明檔案與階段，來自 desktop-core），表單維持原值。
      setAppMsg(String(e));
    }
  };

  const saveWorkflow = async () => {
    try {
      await settings.writeWorkflowConfig({
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

  const beginCtxEdit = () => {
    setDraftContext(contextText);
    setCtxMsg(null);
    setCtxEditing(true);
  };

  const saveContext = async () => {
    // 僅寫 context 鍵（清空＝移除鍵）；產出規則卡對應的 rules 鍵不觸碰。
    try {
      await settings.writeWorkflowContext(draftContext);
      setContextText(draftContext);
      setContextExpanded(false);
      setCtxEditing(false);
      setCtxMsg(t("settings.saved"));
    } catch (e) {
      // 寫入失敗：單行錯誤，維持編輯態不遺失輸入。
      setCtxMsg(String(e));
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
      await settings.writeWorkflowRules(nextRules);
      setRules(Object.fromEntries(nextRules));
      setRulesEditing(false);
      setRulesMsg(t("settings.saved"));
    } catch (e) {
      setRulesMsg(String(e));
    }
  };

  const uiLocaleOptions: Array<{ value: LocalePreference; label: string }> = [
    { value: null, label: t("settings.followSystem") },
    { value: "zh-TW", label: "繁體中文" },
    { value: "en", label: "English" },
  ];

  const contextCollapsed = !ctxEditing && !contextExpanded && isLongContext(contextText);
  const populatedRuleKeys = snap.workflow.schemaArtifacts.filter((id) => (rules[id] ?? []).length > 0);

  return (
    <div className="max-w-2xl mx-auto w-full">
      {/* 三頁簽（design D1）：標籤檔名直出為字面常數（LANGUAGE.md 明文例外）、本機設定經字典。 */}
      <Tabs defaultValue="config">
        <TabsList>
          <TabsTrigger value="config">
            config.yaml
            {wfDisabled && <TabWarningDot />}
          </TabsTrigger>
          <TabsTrigger value="speclink">
            .speclink.yaml
            {appDisabled && <TabWarningDot />}
          </TabsTrigger>
          {/* 本機設定簽不掛任何解析錯誤（design D3）。 */}
          <TabsTrigger value="local">{t("settings.localTabLabel")}</TabsTrigger>
          {/* 伺服器簽（desktop-connections 決策 7）：app 全域、不經 session 綁定。 */}
          {servers && <TabsTrigger value="servers">{t("settings.serversTabLabel")}</TabsTrigger>}
        </TabsList>

        {/* config.yaml 簽：專案說明／產出規則／產出政策 */}
        <TabsContent value="config" className="pt-3 flex flex-col gap-4">
          <span data-testid="file-note-config" className="font-mono text-xs text-muted-foreground">openspec/config.yaml</span>
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
        <TabsContent value="speclink" className="pt-3 flex flex-col gap-4">
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
        </TabsContent>

        {/* 本機設定簽：介面語言（app 本機偏好——與 config.yaml 的 locale 是兩件事） */}
        <TabsContent value="local" className="pt-3 flex flex-col gap-4">
          <span data-testid="local-note" className="text-xs text-muted-foreground">{t("settings.localTabNote")}</span>
          <Card data-testid="ui-locale-card">
            <CardHeader>
              <CardTitle className="text-base">{t("settings.uiLocaleLabel")}</CardTitle>
            </CardHeader>
            <CardContent className="gap-2">
              <div className="flex gap-1.5" data-testid="ui-locale">
                {uiLocaleOptions.map((opt) => (
                  <Button
                    key={String(opt.value)}
                    type="button"
                    variant="outline"
                    size="sm"
                    className={cn(
                      "text-sm font-normal",
                      localePref === opt.value
                        ? "border-primary bg-primary/8 font-medium text-primary hover:bg-primary/8 hover:text-primary"
                        : "text-muted-foreground hover:text-foreground",
                    )}
                    onClick={() => onLocalePrefChange(opt.value)}
                  >
                    {opt.label}
                  </Button>
                ))}
              </div>
              <FieldHelp>{t("settings.uiLocaleHelp")}</FieldHelp>
            </CardContent>
          </Card>
          {/* 面板建立失敗的獨立警示行（spec「面板樣式（macOS）」失敗退回）：
              系統匣樣式由平台決定、無設定卡，錯誤仍於本機設定簽浮出。 */}
          {trayPanelError && <ParseErrorBanner message={trayPanelError} />}
        </TabsContent>

        {/* 伺服器簽：saved servers 清單與登入管理（desktop-connections）。 */}
        {servers && (
          <TabsContent value="servers" className="pt-3 flex flex-col gap-4">
            <ServersPanel {...servers} />
          </TabsContent>
        )}
      </Tabs>
    </div>
  );
}
