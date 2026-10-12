// 關於頁（desktop-about-page design D2／D6；spec「關於頁」）：品牌卡（鎖版、版本列附可點的
// 更新狀態籤與複製版本資訊鈕、產品描述）、連結卡（原始碼、說明文件、回報問題）、授權卡（授權
// 條款、第三方授權）。檢查更新與更新日誌只在更新頁，這裡不設。
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Button,
  Card,
  CopyButton,
  SettingsCard,
  SettingsLinkRow,
  SettingsPageHeader,
  SettingsRow,
  StatusPill,
  Wordmark,
  useI18n,
} from "@speclink/ui";

import type { AppInfo } from "../../adapter/appInfo";
import type { UpdaterState } from "../../core/updater";
import {
  BUNDLED_THIRD_PARTY_LICENSES,
  LicenseDialog,
  type LicenseDialogContent,
  type ThirdPartyLicenses,
} from "../../components/LicenseDialog";
import { latestOf } from "./UpdatesPage";

/** 倉庫網址（原生選單的 GitHub／回報問題也用它；能力檔的 opener 權限只放行這個前綴）。 */
export const REPO_URL = "https://github.com/MomoChenisMe/speclink";
export const ISSUE_URL = `${REPO_URL}/issues/new`;
const DOCS_URL = `${REPO_URL}/tree/main/docs`;

/** 以系統瀏覽器開啟網址，失敗以失敗 toast 呈現（原生選單的外部連結共用）。 */
export function openExternal(url: string) {
  openUrl(url).catch((e) => toast.error(String(e)));
}

/** 「複製版本資訊」與回報問題 issue 內文共用的三行。 */
function versionLines(info: AppInfo): string {
  return [
    `Speclink ${info.appVersion}`,
    `Engine ${info.engineVersion}`,
    `Platform ${info.platform} ${info.arch}`,
  ].join("\n");
}

export interface AboutPageProps {
  /** null＝尚未取得（版本列顯示「—」、無複製鈕）。 */
  appInfo: AppInfo | null;
  /** 進頁時要求載入版本資訊（store 快取，重複呼叫不重取）。 */
  onLoadAppInfo: () => void;
  /** 與更新頁「最新版本」列同一份更新狀態；未注入時不顯示狀態籤。 */
  updaterState?: UpdaterState;
  /** 點狀態籤：進更新頁。 */
  onOpenUpdates: () => void;
  thirdPartyLicenses?: ThirdPartyLicenses | null;
}

export function AboutPage({
  appInfo,
  onLoadAppInfo,
  updaterState,
  onOpenUpdates,
  thirdPartyLicenses = BUNDLED_THIRD_PARTY_LICENSES,
}: AboutPageProps) {
  const { t } = useI18n();
  const [dialog, setDialog] = useState<LicenseDialogContent | null>(null);
  useEffect(() => {
    onLoadAppInfo();
  }, [onLoadAppInfo]);

  const lines = appInfo ? versionLines(appInfo) : null;
  const pill = updaterState ? latestOf(updaterState, null).pill : null;
  const viewButton = (content: LicenseDialogContent) => (
    <Button type="button" variant="outline" size="sm" onClick={() => setDialog(content)}>
      {t("about.view")}
    </Button>
  );

  return (
    <>
      <SettingsPageHeader title={t("about.title")} description={t("about.description")} />
      <Card data-testid="about-brand" className="flex flex-col gap-3 p-5">
        <Wordmark className="h-10 self-start" />
        <div className="flex flex-wrap items-center gap-2 text-[13px] text-muted-foreground">
          <span>
            {appInfo
              ? `${appInfo.appVersion} · ${t("about.engine")} ${appInfo.engineVersion} · ${appInfo.platform} ${appInfo.arch}`
              : "—"}
          </span>
          {pill && (
            <button
              type="button"
              onClick={onOpenUpdates}
              aria-label={t("about.updateStatus").replace("{status}", t(pill.key))}
              className="rounded-full transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <StatusPill tone={pill.tone}>{t(pill.key)}</StatusPill>
            </button>
          )}
          {lines && (
            <span className="ml-auto">
              <CopyButton size="sm" value={lines} label={t("about.copyInfo")} />
            </span>
          )}
        </div>
        <p className="m-0 text-[15px]">{t("about.tagline")}</p>
      </Card>

      <SettingsCard title={t("about.linksCard")}>
        <SettingsLinkRow
          label={t("about.source")}
          description={<span className="font-mono">{REPO_URL.replace(/^https:\/\//, "")}</span>}
          onClick={() => openExternal(REPO_URL)}
        />
        <SettingsLinkRow label={t("about.docs")} description={t("about.docsHint")} onClick={() => openExternal(DOCS_URL)} />
        <SettingsLinkRow
          label={t("about.report")}
          description={t("about.reportHint")}
          onClick={() => openExternal(`${ISSUE_URL}${lines ? `?body=${encodeURIComponent(lines)}` : ""}`)}
        />
      </SettingsCard>

      <SettingsCard title={t("about.licenseCard")}>
        <SettingsRow label={t("about.license")} description={t("about.licenseHint")} control={viewButton("license")} />
        <SettingsRow
          label={t("about.thirdParty")}
          description={t("about.thirdPartyHint")}
          control={viewButton("thirdParty")}
        />
      </SettingsCard>

      <LicenseDialog content={dialog} thirdPartyLicenses={thirdPartyLicenses} onClose={() => setDialog(null)} />
    </>
  );
}
