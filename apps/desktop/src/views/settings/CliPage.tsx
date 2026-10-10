// CLI 頁（desktop-settings-mode design D5；spec「安裝 CLI 指令到 PATH」）。「speclink CLI」列由
// 一般頁的「CLI 指令」卡共用：狀態文字＋安裝／重新安裝鈕，說明為佈署方式，PATH 提示與錯誤在列下。
import { AlertTriangle } from "lucide-react";
import {
  Button,
  SEMANTIC_TONE,
  SettingsCard,
  SettingsPageHeader,
  SettingsRow,
  useI18n,
} from "@speclink/ui";

import type { CliInstallView } from "../../store";

export interface CliInstallProps {
  view: CliInstallView;
  onInstall: () => void;
}

/** 狀態文字：三態＋版本。 */
function CliInstallStatus({ view }: { view: CliInstallView }) {
  const { t } = useI18n();
  switch (view.status.kind) {
    case "not-installed":
      return <span className="text-xs text-muted-foreground">{t("cliInstall.notInstalled")}</span>;
    case "installed":
      return (
        <span className="text-xs text-muted-foreground">
          {t("cliInstall.installed")} {view.status.version}
        </span>
      );
    case "version-mismatch":
      return (
        <span className={`text-xs ${SEMANTIC_TONE.warning}`}>
          {t("cliInstall.mismatch")}（{view.status.version}）
        </span>
      );
  }
}

/** 列說明依佈署方式：可佈署時說明用途；Windows 由安裝器管理、Linux 非 AppImage 無可佈署的 CLI。 */
function cliDescriptionKey(view: CliInstallView): string | null {
  if (view.platform === "windows") return "cliInstall.installerManaged";
  if (view.platform === "linux-unpackaged") return "cliInstall.unpackaged";
  return view.canDeploy ? "cliInstall.help" : null;
}

export function CliInstallRow({ view, onInstall }: CliInstallProps) {
  const { t } = useI18n();
  const descriptionKey = cliDescriptionKey(view);
  const pathHint = view.pathHint && view.deployDir;
  return (
    <SettingsRow
      label={t("cliInstall.rowLabel")}
      description={descriptionKey ? t(descriptionKey) : undefined}
      control={
        <>
          <CliInstallStatus view={view} />
          {view.canDeploy && view.status.kind !== "installed" && (
            <Button type="button" variant="outline" size="sm" disabled={view.busy} onClick={onInstall}>
              {view.busy
                ? t("cliInstall.installing")
                : view.status.kind === "version-mismatch"
                  ? t("cliInstall.reinstall")
                  : t("cliInstall.install")}
            </Button>
          )}
        </>
      }
    >
      {(pathHint || view.error) && (
        <div className="flex flex-col gap-1.5">
          {pathHint && (
            <p data-testid="cli-path-hint" className={`m-0 flex items-start gap-1.5 text-xs ${SEMANTIC_TONE.warning}`}>
              <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
              <span>
                {view.deployDir} {t("cliInstall.pathHintSuffix")}{" "}
                <code>export PATH=&quot;$PATH:{view.deployDir}&quot;</code>
              </span>
            </p>
          )}
          {view.error && (
            <p role="alert" className={`m-0 text-xs ${SEMANTIC_TONE.danger}`}>
              {view.error}
            </p>
          )}
        </div>
      )}
    </SettingsRow>
  );
}

export function CliPage({ cliInstall }: { cliInstall?: CliInstallProps }) {
  const { t } = useI18n();
  return (
    <>
      <SettingsPageHeader title={t("settings.section.cli")} note={t("settings.pageNote")} />
      {cliInstall && (
        <SettingsCard title={t("cliInstall.cardTitle")}>
          <CliInstallRow {...cliInstall} />
          {cliInstall.view.deployDir && (
            <SettingsRow
              label={t("cli.page.deployDir")}
              control={<span className="font-mono text-xs text-muted-foreground">{cliInstall.view.deployDir}</span>}
            />
          )}
        </SettingsCard>
      )}
    </>
  );
}
