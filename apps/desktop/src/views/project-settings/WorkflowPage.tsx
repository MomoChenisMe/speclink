// 產出流程頁（desktop-settings-mode design D8；spec「產出流程頁」）：產出政策卡（locale、spec_locale
// 下拉與 tdd、audit、worktree 開關）與產出流程卡（清單、唯讀詳情、切換、fork、建立、刪除——原樣
// 自原產出流程頁籤搬入）。
import { AlertTriangle } from "lucide-react";
import { useEffect, useState } from "react";
import {
  Button,
  ConfirmDialog,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SEMANTIC_TONE,
  SettingsCard,
  SettingsChoice,
  SettingsPageHeader,
  SettingsRow,
  Switch,
  Textarea,
  useI18n,
} from "@speclink/ui";

import { FieldHelp, InvalidLocaleHint, PageAlerts, PageNote } from "./parts";
import {
  LOCALE_OPTIONS,
  LOCALE_UNSET,
  SPEC_LOCALE_OPTIONS,
  errorMessage,
  type ProjectSettingsState,
} from "./useProjectSettings";
import { listSecurityData, type StandardDataList, type SecurityPolicy } from "../../adapter/security";

const LOCALE_LABEL: Record<string, string> = { tw: "tw（繁體中文）", ja: "ja（日本語）", en: "en（English）", auto: "auto" };

/** 儲存值落在選項集外（手改 config.yaml）：下拉以停用項顯示原值，列與說明轉警示。 */
const isInvalidLocale = (value: string, codes: readonly string[]) => value !== "" && !codes.includes(value);

/** locale／spec_locale 的選項：未設定＋合法代碼；選項集外的儲存值以停用項顯示原始值（不改寫）。 */
function localeOptions(value: string, codes: readonly string[], unsetLabel: string, invalidSuffix: string) {
  return [
    ...(isInvalidLocale(value, codes) ? [{ value, label: `${value}${invalidSuffix}`, disabled: true }] : []),
    { value: LOCALE_UNSET, label: unsetLabel },
    ...codes.map((code) => ({ value: code, label: LOCALE_LABEL[code] ?? code })),
  ];
}

export function WorkflowPage({ ps }: { ps: ProjectSettingsState }) {
  const { t } = useI18n();
  const [data, setData] = useState<StandardDataList | null>(null);
  const [dataError, setDataError] = useState<string | null>(null);
  useEffect(() => {
    if (!ps.security.enabled || ps.settings.securityReview===false) return;
    let active = true;
    void listSecurityData().then(
      (value) => { if (active) { setData(value); setDataError(null); } },
      (error) => { if (active) setDataError(errorMessage(error)); },
    );
    return () => { active = false; };
  }, [ps.security.enabled, ps.settings.securityReview]);
  const {
    settings,
    snap,
    isRemote,
    wfDisabled,
    schemas,
    expandedSchema,
    setExpandedSchema,
    schemaMsg,
    setSchemaMsg,
    createName,
    setCreateName,
    pendingDeleteSchema,
    setPendingDeleteSchema,
    schemaSourceLabel,
    isResolutionHit,
    switchSchema,
    forkSchemaAction,
    confirmDeleteSchema,
    createSchemaAction,
    selectableSchemaNames,
  } = ps;
  const localeInvalid = isInvalidLocale(ps.locale, LOCALE_OPTIONS);
  const specLocaleInvalid = isInvalidLocale(ps.specLocale, SPEC_LOCALE_OPTIONS);
  const policyKeys = isRemote ? "schema, locale, spec_locale, tdd, audit" : "schema, locale, spec_locale, tdd, audit, worktree";
  const toggle = (key: "tdd" | "audit" | "worktree", checked: boolean, set: (v: boolean) => void) => (
    <SettingsRow
      label={key}
      htmlFor={`cfg-${key}`}
      description={t(`settings.${key}Help`)}
      control={<Switch id={`cfg-${key}`} checked={checked} disabled={wfDisabled} onCheckedChange={set} />}
    />
  );
  const standardChoice = (key: keyof SecurityPolicy["standards"], label: string, versions: string[], help?: string) => {
    const selected = ps.security.standards[key];
    const installed = data?.standards.find((s) => s.standardId === key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`))?.versions ?? [];
    const choices = key === "cwe" ? installed.filter((v) => v.status === "installed").map((v) => v.version) : versions;
    const missing = selected.version !== null && !choices.includes(selected.version);
    return <SettingsRow key={key} label={label} htmlFor={`cfg-security-${key}`} description={help} control={
      <SettingsChoice id={`cfg-security-${key}`} disabled={wfDisabled} value={selected.enabled ? selected.version! : "__off__"}
        options={[
          { value: "__off__", label: t("security.off") },
          ...(missing ? [{ value: selected.version!, label: `${selected.version} · ${t("security.missingData")}`, disabled: true }] : []),
          ...choices.map((v) => ({ value: v, label: v })),
        ]}
        onChange={(value) => ps.setSecurity((current) => ({ ...current, standards: { ...current.standards, [key]: { ...current.standards[key], enabled: value !== "__off__", version: value === "__off__" ? current.standards[key].version : value } } }))} />
    }>{selected.enabled && <FieldHelp>{installed.some((v) => v.version === selected.version && v.status === "installed") ? t("security.installed") : t("security.missingData")}</FieldHelp>}</SettingsRow>;
  };
  return (
    <>
      <SettingsPageHeader
        title={t("settings.projectSection.workflow")}
        description={t("settings.schemaCardHelp")}
        note={<PageNote remote={ps.remoteNote} fileKey={`openspec/config.yaml · ${policyKeys}`} testId="file-note-config" />}
      />
      <PageAlerts ps={ps} parseError={snap.workflow.parseError} />
      <div data-testid="policy-card">
        <SettingsCard title={t("settings.policyCard")}>
          <SettingsRow
            label="locale"
            htmlFor="cfg-locale"
            description={t("settings.localeHelp")}
            warning={localeInvalid}
            control={
              <SettingsChoice
                id="cfg-locale"
                value={ps.locale === "" ? LOCALE_UNSET : ps.locale}
                options={localeOptions(ps.locale, LOCALE_OPTIONS, t("settings.localeUnset"), t("settings.localeInvalid"))}
                disabled={wfDisabled}
                onChange={(v) => ps.setLocale(v === LOCALE_UNSET ? "" : v)}
              />
            }
          >
            {localeInvalid && (
              <InvalidLocaleHint testId="locale-invalid-hint">{t("settings.localeInvalidHint")}</InvalidLocaleHint>
            )}
          </SettingsRow>
          <SettingsRow
            label="spec_locale"
            htmlFor="cfg-spec-locale"
            description={t("settings.specLocaleHelp")}
            warning={specLocaleInvalid}
            control={
              <SettingsChoice
                id="cfg-spec-locale"
                value={ps.specLocale === "" ? LOCALE_UNSET : ps.specLocale}
                options={localeOptions(ps.specLocale, SPEC_LOCALE_OPTIONS, t("settings.localeUnset"), t("settings.localeInvalid"))}
                disabled={wfDisabled}
                onChange={(v) => ps.setSpecLocale(v === LOCALE_UNSET ? "" : v)}
              />
            }
          >
            {specLocaleInvalid && (
              <InvalidLocaleHint testId="spec-locale-invalid-hint">{t("settings.specLocaleInvalidHint")}</InvalidLocaleHint>
            )}
          </SettingsRow>
          {toggle("tdd", ps.tdd, ps.setTdd)}
          {toggle("audit", ps.audit, ps.setAudit)}
          <SettingsRow label={t("security.enable")} htmlFor="cfg-security-enabled" description={t("security.help")} control={
            <Switch id="cfg-security-enabled" checked={ps.security.enabled} disabled={wfDisabled || ps.settings.securityReview===false} onCheckedChange={(enabled) => ps.setSecurity((current) => ({ ...current, enabled }))} />
          } />
          {ps.settings.securityReview===false && <p role="status" className="px-4 text-xs text-muted-foreground">unsupported_version</p>}
          {ps.security.enabled && ps.settings.securityReview!==false && <div data-testid="security-policy-fields">
            <SettingsRow label={t("security.scope")} htmlFor="cfg-security-scope" description={t(`security.scopeHelp.${ps.security.scope}`)} control={
              <SettingsChoice id="cfg-security-scope" disabled={wfDisabled} value={ps.security.scope}
                options={["off", "web-api", "api", "web"].map((value) => ({ value, label: t(`security.scope.${value}`) }))}
                onChange={(scope) => ps.setSecurity((current) => ({ ...current, scope: scope as SecurityPolicy["scope"] }))} />
            } />
            {!Object.values(ps.security.standards).some((s) => s.enabled) && <p className="px-4 text-sm text-muted-foreground" role="status">{t("security.unconfigured")}</p>}
            {standardChoice("owaspTop10", "OWASP Top 10", ["2021", "2025"], t("security.top10Help"))}
            {standardChoice("owaspApiTop10", "OWASP API Security Top 10", ["2019", "2023"], t("security.apiHelp"))}
            {standardChoice("owaspAsvs", "OWASP ASVS", ["4.0.3", "5.0.0"])}
            {ps.security.standards.owaspAsvs.enabled && <SettingsRow label={t("security.asvsLevel")} htmlFor="cfg-security-level" control={
              <SettingsChoice id="cfg-security-level" disabled={wfDisabled} value={ps.security.standards.owaspAsvs.level} options={["L1", "L2", "L3"].map((value) => ({ value, label: value }))}
                onChange={(level) => ps.setSecurity((current) => ({ ...current, standards: { ...current.standards, owaspAsvs: { ...current.standards.owaspAsvs, level: level as "L1" | "L2" | "L3" } } }))} />
            } />}
            {standardChoice("cwe", "MITRE CWE", [])}
            {(["spec", "code", "apiBoundary"] as const).map((key) => <SettingsRow key={key} label={t(`security.review.${key}`)} htmlFor={`cfg-security-review-${key}`} control={
              <Switch id={`cfg-security-review-${key}`} disabled={wfDisabled} checked={ps.security.reviews[key]} onCheckedChange={(checked) => ps.setSecurity((current) => ({ ...current, reviews: { ...current.reviews, [key]: checked } }))} />
            } />)}
            {(["excludeDirs", "disabledRules"] as const).map((key) => <SettingsRow key={key} label={t(`security.${key}`)} htmlFor={`cfg-security-${key}`} description={t(`security.${key}Help`)} control={null}>
              <Textarea id={`cfg-security-${key}`} disabled={wfDisabled} value={ps.security[key].join("\n")} onChange={(event) => ps.setSecurity((current) => ({ ...current, [key]: event.target.value.split("\n") }))} />
            </SettingsRow>)}
            <p className="px-4 text-xs text-muted-foreground">{t("security.dataLocation")}</p>
            {dataError && <p role="alert" className={`px-4 text-xs ${SEMANTIC_TONE.warning}`}>{t("security.dataError")}: {dataError}</p>}
          </div>}
          {/* worktree 屬本機 checkout 概念：remote 不顯示（spec「產出政策的 worktree 開關」）。 */}
          {!isRemote && toggle("worktree", ps.worktree, ps.setWorktree)}
          <div className="flex items-center justify-end gap-2 px-4 py-3">
            {ps.wfMsg && <span className="mr-auto text-xs text-muted-foreground">{ps.wfMsg}</span>}
            <Button type="button" size="sm" data-testid="save-workflow" disabled={wfDisabled} onClick={() => void ps.saveWorkflow()}>
              {t("settings.save")}
            </Button>
          </div>
        </SettingsCard>
      </div>
      <div data-testid="schema-card">
        <SettingsCard title={t("settings.schemaCard")}>
          <div className="flex flex-col gap-2.5 p-4">
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
          </div>
        </SettingsCard>
      </div>

      {/* 刪除確認（D7；沿變更刪除的確認框模式） */}
      <ConfirmDialog
        open={pendingDeleteSchema !== null}
        onOpenChange={(o) => !o && setPendingDeleteSchema(null)}
        title={t("settings.schemaDeleteTitle")}
        description={
          <>
            {t("settings.schemaDeleteDesc")}{" "}
            <span className="font-mono font-medium">{pendingDeleteSchema}</span>
          </>
        }
        confirmLabel={t("settings.schemaDelete")}
        destructive
        onConfirm={confirmDeleteSchema}
      />
    </>
  );
}
